use makosh_ai_contracts::{
    AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, AI_MAX_OUTPUT_BYTES_V1, AI_MAX_OUTPUT_TOKENS_V1,
    seal_summary_inference_request_v1,
    wire::{
        AiContextReceiptV1, AiEgressPolicyV1, AiPrivateSourceReceiptV1, AiSummaryLanguageV1,
        AiSummaryLengthV1, AiUseCaseV1, CommunicationSummaryInferenceRequestV1,
    },
};
use std::os::unix::net::UnixStream;

use makosh_communication_summary_core::{
    CommunicationSummaryLanguageV1, CommunicationSummaryLengthV1,
    CommunicationSummaryRejectionCodeV1, CommunicationSummaryStateV1,
    CommunicationSummaryTransitionV1,
};
use makosh_communication_summary_persistence::{
    CommunicationSummaryPersistenceErrorV1, CommunicationSummaryPersistenceV1,
    CommunicationSummarySourceResultV1,
};
use makosh_communications_ai_source_api::{
    communication_summary_source_prepared_contract_reference_v1,
    communication_summary_source_rejected_contract_reference_v1,
    wire::{CommunicationSummarySourcePreparedV1, CommunicationSummarySourceRejectedV1},
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use sha2::{Digest, Sha256};

const COMMUNICATIONS_RUNTIME_MODULE_ID_V1: &str = "makosh-communications-runtime";

use crate::{
    CommunicationSummaryBlobErrorV1, CommunicationSummaryInferenceErrorV1,
    blob_materialization::{
        CommunicationSummarySourceBlobReceiptV1, materialize_summary_source_for_ai_v1,
        release_summary_source_blobs_v1,
    },
    inference::complete_communication_summary_inference_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationSummarySourceResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Blob(CommunicationSummaryBlobErrorV1),
    Inference(CommunicationSummaryInferenceErrorV1),
    Persistence(CommunicationSummaryPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_summary_source_prepared_once_v1(
    persistence: &CommunicationSummaryPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, CommunicationSummarySourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationSummarySourceResultErrorV1::InvalidEnvelope)?;
    let prepared = decode_prepared(&record, expected_logical_owner_id)?;
    let run = persistence
        .load_run(expected_logical_owner_id, &prepared.run_id)
        .await
        .map_err(CommunicationSummarySourceResultErrorV1::Persistence)?;
    if run.draft.source_message_id.as_slice() != prepared.source_message_id {
        return Err(CommunicationSummarySourceResultErrorV1::InvalidPayload);
    }
    if matches!(
        run.status.state,
        CommunicationSummaryStateV1::Ready | CommunicationSummaryStateV1::Rejected
    ) && run.cleanup_completed_at_unix_millis.is_some()
    {
        delivery.acknowledge().await.map_err(event_error)?;
        return Ok(true);
    }
    let source = prepared
        .source_content
        .as_ref()
        .ok_or(CommunicationSummarySourceResultErrorV1::InvalidPayload)?;
    let materialized = materialize_summary_source_for_ai_v1(
        channel,
        dispatcher,
        prepared.run_id,
        &CommunicationSummarySourceBlobReceiptV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            reference_id: id16(&source.reference_id)?,
            declared_bytes: source.declared_bytes,
            sha256: sha256(&source.sha256)?,
            custody_proof: source.custody_transfer_source_proof.clone(),
        },
    )
    .map_err(CommunicationSummarySourceResultErrorV1::Blob)?;
    let sealed = seal_request(
        &run.draft,
        &prepared,
        materialized.ai_source.clone(),
        expected_logical_owner_id,
    )?;
    let request_digest = sha256(
        &sealed
            .context
            .as_ref()
            .ok_or(CommunicationSummarySourceResultErrorV1::InvalidPayload)?
            .request_digest,
    )?;
    let source_sha256 = sha256(
        &sealed
            .source
            .as_ref()
            .ok_or(CommunicationSummarySourceResultErrorV1::InvalidPayload)?
            .sha256,
    )?;
    let inference_request_bytes = sealed.encode_to_vec();
    let persisted = persistence
        .persist_source_result(CommunicationSummarySourceResultV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            logical_owner_id: expected_logical_owner_id.to_owned(),
            run_id: prepared.run_id,
            transition: CommunicationSummaryTransitionV1::SourcePrepared {
                source_evidence_id: prepared.source_evidence_id,
                source_evidence_revision: prepared.source_evidence_revision,
                source_sha256,
                inference_request_digest: request_digest,
            },
            inference_request_bytes: Some(inference_request_bytes.clone()),
            source_cleanup: Some(materialized.source_cleanup.clone()),
            occurred_at_unix_millis: consumed_at_unix_millis,
        })
        .await
        .map_err(CommunicationSummarySourceResultErrorV1::Persistence)?;
    let persisted = match persisted {
        makosh_communication_summary_persistence::CommunicationSummaryInboxResultV1::Applied(
            value,
        )
        | makosh_communication_summary_persistence::CommunicationSummaryInboxResultV1::Duplicate(
            value,
        ) => value,
    };
    match persisted.status.state {
        CommunicationSummaryStateV1::AwaitingInference => {
            let persisted = persistence
                .refresh_inference_request(
                    expected_logical_owner_id,
                    &prepared.run_id,
                    &request_digest,
                    &materialized.source_cleanup,
                    &inference_request_bytes,
                    consumed_at_unix_millis,
                )
                .await
                .map_err(CommunicationSummarySourceResultErrorV1::Persistence)?;
            complete_communication_summary_inference_v1(
                persistence,
                channel,
                dispatcher,
                &persisted,
                sealed,
                consumed_at_unix_millis,
            )
            .await
            .map_err(CommunicationSummarySourceResultErrorV1::Inference)?;
        }
        CommunicationSummaryStateV1::Ready | CommunicationSummaryStateV1::Rejected => {}
        _ => return Err(CommunicationSummarySourceResultErrorV1::InvalidPayload),
    }
    let terminal = persistence
        .load_run(expected_logical_owner_id, &prepared.run_id)
        .await
        .map_err(CommunicationSummarySourceResultErrorV1::Persistence)?;
    let accepted = match terminal.status.state {
        CommunicationSummaryStateV1::Ready => true,
        CommunicationSummaryStateV1::Rejected => false,
        _ => return Err(CommunicationSummarySourceResultErrorV1::InvalidPayload),
    };
    release_summary_source_blobs_v1(
        channel,
        dispatcher,
        prepared.run_id,
        &materialized.ai_source,
        &materialized.source_cleanup,
        accepted,
    )
    .map_err(CommunicationSummarySourceResultErrorV1::Blob)?;
    persistence
        .complete_blob_cleanup(
            expected_logical_owner_id,
            &prepared.run_id,
            &materialized.source_cleanup,
            consumed_at_unix_millis,
        )
        .await
        .map_err(CommunicationSummarySourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub async fn consume_summary_source_rejected_once_v1(
    persistence: &CommunicationSummaryPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, CommunicationSummarySourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationSummarySourceResultErrorV1::InvalidEnvelope)?;
    let rejected = decode_rejected(&record, expected_logical_owner_id)?;
    persistence
        .persist_source_result(CommunicationSummarySourceResultV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            logical_owner_id: expected_logical_owner_id.to_owned(),
            run_id: rejected.run_id,
            transition: CommunicationSummaryTransitionV1::Reject(rejected.rejection),
            inference_request_bytes: None,
            source_cleanup: None,
            occurred_at_unix_millis: consumed_at_unix_millis,
        })
        .await
        .map_err(CommunicationSummarySourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

struct PreparedSourceV1 {
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    source_evidence_id: [u8; 16],
    source_evidence_revision: u64,
    source_content: Option<
        makosh_communications_ai_source_api::wire::CommunicationSummarySourceContentReceiptV1,
    >,
}

struct RejectedSourceV1 {
    run_id: [u8; 16],
    rejection: CommunicationSummaryRejectionCodeV1,
}

fn decode_prepared(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<PreparedSourceV1, CommunicationSummarySourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationSummarySourceResultErrorV1::InvalidEnvelope)?;
    let run_id = validate_result_envelope(
        &envelope.contract,
        &communication_summary_source_prepared_contract_reference_v1(),
        envelope
            .source
            .as_ref()
            .map(|source| source.module_id.as_str()),
        envelope
            .source
            .as_ref()
            .map_or(0, |source| source.runtime_generation),
        envelope.semantics.as_ref(),
        ResultOutcomeV1::Succeeded,
    )?;
    let payload = CommunicationSummarySourcePreparedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationSummarySourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id || payload.run_id.as_slice() != run_id
    {
        return Err(CommunicationSummarySourceResultErrorV1::InvalidPayload);
    }
    Ok(PreparedSourceV1 {
        run_id,
        source_message_id: id16(&payload.source_message_id)?,
        source_evidence_id: id16(&payload.source_evidence_id)?,
        source_evidence_revision: (payload.source_evidence_revision > 0)
            .then_some(payload.source_evidence_revision)
            .ok_or(CommunicationSummarySourceResultErrorV1::InvalidPayload)?,
        source_content: payload.source_content,
    })
}

fn decode_rejected(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<RejectedSourceV1, CommunicationSummarySourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationSummarySourceResultErrorV1::InvalidEnvelope)?;
    let run_id = validate_result_envelope(
        &envelope.contract,
        &communication_summary_source_rejected_contract_reference_v1(),
        envelope
            .source
            .as_ref()
            .map(|source| source.module_id.as_str()),
        envelope
            .source
            .as_ref()
            .map_or(0, |source| source.runtime_generation),
        envelope.semantics.as_ref(),
        ResultOutcomeV1::Rejected,
    )?;
    let payload = CommunicationSummarySourceRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationSummarySourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id || payload.run_id.as_slice() != run_id
    {
        return Err(CommunicationSummarySourceResultErrorV1::InvalidPayload);
    }
    let rejection = match payload.code {
        1 => CommunicationSummaryRejectionCodeV1::InvalidRequest,
        7 => CommunicationSummaryRejectionCodeV1::Policy,
        2..=6 => CommunicationSummaryRejectionCodeV1::SourceRejected,
        _ => return Err(CommunicationSummarySourceResultErrorV1::InvalidPayload),
    };
    Ok(RejectedSourceV1 { run_id, rejection })
}

fn seal_request(
    draft: &makosh_communication_summary_core::CommunicationSummaryDraftV1,
    prepared: &PreparedSourceV1,
    source: AiPrivateSourceReceiptV1,
    logical_owner_id: &str,
) -> Result<CommunicationSummaryInferenceRequestV1, CommunicationSummarySourceResultErrorV1> {
    seal_summary_inference_request_v1(CommunicationSummaryInferenceRequestV1 {
        run_id: prepared.run_id.to_vec(),
        context: Some(AiContextReceiptV1 {
            context_id: context_id(&prepared.run_id, &prepared.source_evidence_id).to_vec(),
            use_case: AiUseCaseV1::AiUseCaseCommunicationSummary as i32,
            source_evidence_id: prepared.source_evidence_id.to_vec(),
            source_evidence_revision: prepared.source_evidence_revision,
            contract_major: AI_CONTRACT_MAJOR_V1,
            contract_revision: AI_CONTRACT_REVISION_V1,
            contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
            request_digest: Vec::new(),
        }),
        source: Some(source),
        length: ai_length(draft.length) as i32,
        language: ai_language(draft.language) as i32,
        maximum_output_bytes: AI_MAX_OUTPUT_BYTES_V1,
        maximum_output_tokens: AI_MAX_OUTPUT_TOKENS_V1,
        egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
        egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        logical_owner_id: logical_owner_id.to_owned(),
    })
    .map_err(|_| CommunicationSummarySourceResultErrorV1::InvalidPayload)
}

fn validate_result_envelope(
    actual_contract: &Option<ContractRefV1>,
    expected_contract: &ContractReferenceV1,
    source_module_id: Option<&str>,
    source_runtime_generation: u64,
    semantics: Option<&Semantics>,
    expected_outcome: ResultOutcomeV1,
) -> Result<[u8; 16], CommunicationSummarySourceResultErrorV1> {
    if !exact_contract(actual_contract.as_ref(), expected_contract)
        || source_module_id != Some(COMMUNICATIONS_RUNTIME_MODULE_ID_V1)
        || source_runtime_generation == 0
    {
        return Err(CommunicationSummarySourceResultErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Result(result)) = semantics else {
        return Err(CommunicationSummarySourceResultErrorV1::InvalidEnvelope);
    };
    if result.command_id.len() != 16
        || result.command_message_id.as_slice() != result.command_id
        || result.outcome != expected_outcome as i32
        || result.execution_attempt == 0
    {
        return Err(CommunicationSummarySourceResultErrorV1::InvalidEnvelope);
    }
    id16(&result.command_id)
}

fn exact_contract(actual: Option<&ContractRefV1>, expected: &ContractReferenceV1) -> bool {
    actual.is_some_and(|actual| {
        actual.owner == expected.owner
            && actual.name == expected.name
            && actual.major == expected.major
            && actual.revision == expected.revision
            && actual.schema_sha256 == expected.schema_sha256
    })
}

fn context_id(run_id: &[u8; 16], evidence_id: &[u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_summary.context.v1\0");
    digest.update(run_id);
    digest.update(evidence_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

const fn ai_length(value: CommunicationSummaryLengthV1) -> AiSummaryLengthV1 {
    match value {
        CommunicationSummaryLengthV1::Short => AiSummaryLengthV1::AiSummaryLengthShort,
        CommunicationSummaryLengthV1::Standard => AiSummaryLengthV1::AiSummaryLengthStandard,
        CommunicationSummaryLengthV1::Detailed => AiSummaryLengthV1::AiSummaryLengthDetailed,
    }
}

const fn ai_language(value: CommunicationSummaryLanguageV1) -> AiSummaryLanguageV1 {
    match value {
        CommunicationSummaryLanguageV1::Auto => AiSummaryLanguageV1::AiSummaryLanguageAuto,
        CommunicationSummaryLanguageV1::English => AiSummaryLanguageV1::AiSummaryLanguageEnglish,
        CommunicationSummaryLanguageV1::Russian => AiSummaryLanguageV1::AiSummaryLanguageRussian,
        CommunicationSummaryLanguageV1::Spanish => AiSummaryLanguageV1::AiSummaryLanguageSpanish,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationSummarySourceResultErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| CommunicationSummarySourceResultErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(CommunicationSummarySourceResultErrorV1::InvalidPayload)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], CommunicationSummarySourceResultErrorV1> {
    let value: [u8; 32] = value
        .try_into()
        .map_err(|_| CommunicationSummarySourceResultErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(CommunicationSummarySourceResultErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> CommunicationSummarySourceResultErrorV1 {
    CommunicationSummarySourceResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_mapping_is_fixed_and_provider_neutral() {
        assert_eq!(
            ai_length(CommunicationSummaryLengthV1::Standard),
            AiSummaryLengthV1::AiSummaryLengthStandard
        );
        assert_eq!(
            ai_language(CommunicationSummaryLanguageV1::Auto),
            AiSummaryLanguageV1::AiSummaryLanguageAuto
        );
        assert_ne!(
            context_id(&[1; 16], &[2; 16]),
            context_id(&[1; 16], &[3; 16])
        );
    }
}
