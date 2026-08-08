use makosh_ai_contracts::{
    AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, AI_MAX_OUTPUT_BYTES_V1, AI_MAX_OUTPUT_TOKENS_V1,
    seal_reply_inference_request_v1,
    wire::{
        AiContextReceiptV1, AiEgressPolicyV1, AiPrivateSourceReceiptV1, AiReplyLanguageV1,
        AiReplySubjectPolicyV1, AiReplyToneV1, AiUseCaseV1,
        CommunicationReplySuggestionInferenceRequestV1,
    },
};
use std::os::unix::net::UnixStream;

use makosh_communication_reply_suggestion_core::{
    ReplySuggestionLanguageV1, ReplySuggestionRejectionCodeV1, ReplySuggestionStateV1,
    ReplySuggestionToneV1, ReplySuggestionTransitionV1,
};
use makosh_communication_reply_suggestion_persistence::{
    CommunicationReplySuggestionPersistenceV1, ReplySuggestionPersistenceErrorV1,
    ReplySuggestionSourceResultV1,
};
use makosh_communications_ai_source_api::{
    communication_reply_source_prepared_contract_reference_v1,
    communication_reply_source_rejected_contract_reference_v1,
    wire::{CommunicationReplySourcePreparedV1, CommunicationReplySourceRejectedV1},
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
    ReplySuggestionBlobErrorV1, ReplySuggestionInferenceErrorV1,
    blob_materialization::{
        ReplySuggestionSourceBlobReceiptV1, materialize_reply_source_for_ai_v1,
        release_reply_source_blobs_v1,
    },
    inference::complete_reply_suggestion_inference_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplySuggestionSourceResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Blob(ReplySuggestionBlobErrorV1),
    Inference(ReplySuggestionInferenceErrorV1),
    Persistence(ReplySuggestionPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_reply_source_prepared_once_v1(
    persistence: &CommunicationReplySuggestionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, ReplySuggestionSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReplySuggestionSourceResultErrorV1::InvalidEnvelope)?;
    let prepared = decode_prepared(&record, expected_logical_owner_id)?;
    let run = persistence
        .load_run(expected_logical_owner_id, &prepared.run_id)
        .await
        .map_err(ReplySuggestionSourceResultErrorV1::Persistence)?;
    if run.draft.source_message_id.as_slice() != prepared.source_message_id {
        return Err(ReplySuggestionSourceResultErrorV1::InvalidPayload);
    }
    if matches!(
        run.status.state,
        ReplySuggestionStateV1::Ready | ReplySuggestionStateV1::Rejected
    ) && run.cleanup_completed_at_unix_millis.is_some()
    {
        delivery.acknowledge().await.map_err(event_error)?;
        return Ok(true);
    }
    let source = prepared
        .source_content
        .as_ref()
        .ok_or(ReplySuggestionSourceResultErrorV1::InvalidPayload)?;
    let materialized = materialize_reply_source_for_ai_v1(
        channel,
        dispatcher,
        prepared.run_id,
        &ReplySuggestionSourceBlobReceiptV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            reference_id: id16(&source.reference_id)?,
            declared_bytes: source.declared_bytes,
            sha256: sha256(&source.sha256)?,
            custody_proof: source.custody_transfer_source_proof.clone(),
        },
    )
    .map_err(ReplySuggestionSourceResultErrorV1::Blob)?;
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
            .ok_or(ReplySuggestionSourceResultErrorV1::InvalidPayload)?
            .request_digest,
    )?;
    let source_sha256 = sha256(
        &sealed
            .source
            .as_ref()
            .ok_or(ReplySuggestionSourceResultErrorV1::InvalidPayload)?
            .sha256,
    )?;
    let inference_request_bytes = sealed.encode_to_vec();
    let persisted = persistence
        .persist_source_result(ReplySuggestionSourceResultV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            logical_owner_id: expected_logical_owner_id.to_owned(),
            run_id: prepared.run_id,
            transition: ReplySuggestionTransitionV1::SourcePrepared {
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
        .map_err(ReplySuggestionSourceResultErrorV1::Persistence)?;
    let persisted = match persisted {
        makosh_communication_reply_suggestion_persistence::ReplySuggestionInboxResultV1::Applied(
            value,
        )
        | makosh_communication_reply_suggestion_persistence::ReplySuggestionInboxResultV1::Duplicate(
            value,
        ) => value,
    };
    match persisted.status.state {
        ReplySuggestionStateV1::AwaitingInference => {
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
                .map_err(ReplySuggestionSourceResultErrorV1::Persistence)?;
            complete_reply_suggestion_inference_v1(
                persistence,
                channel,
                dispatcher,
                &persisted,
                sealed,
                consumed_at_unix_millis,
            )
            .await
            .map_err(ReplySuggestionSourceResultErrorV1::Inference)?;
        }
        ReplySuggestionStateV1::Ready | ReplySuggestionStateV1::Rejected => {}
        _ => return Err(ReplySuggestionSourceResultErrorV1::InvalidPayload),
    }
    let terminal = persistence
        .load_run(expected_logical_owner_id, &prepared.run_id)
        .await
        .map_err(ReplySuggestionSourceResultErrorV1::Persistence)?;
    let accepted = match terminal.status.state {
        ReplySuggestionStateV1::Ready => true,
        ReplySuggestionStateV1::Rejected => false,
        _ => return Err(ReplySuggestionSourceResultErrorV1::InvalidPayload),
    };
    release_reply_source_blobs_v1(
        channel,
        dispatcher,
        prepared.run_id,
        &materialized.ai_source,
        &materialized.source_cleanup,
        accepted,
    )
    .map_err(ReplySuggestionSourceResultErrorV1::Blob)?;
    persistence
        .complete_blob_cleanup(
            expected_logical_owner_id,
            &prepared.run_id,
            &materialized.source_cleanup,
            consumed_at_unix_millis,
        )
        .await
        .map_err(ReplySuggestionSourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub async fn consume_reply_source_rejected_once_v1(
    persistence: &CommunicationReplySuggestionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, ReplySuggestionSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReplySuggestionSourceResultErrorV1::InvalidEnvelope)?;
    let rejected = decode_rejected(&record, expected_logical_owner_id)?;
    persistence
        .persist_source_result(ReplySuggestionSourceResultV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            logical_owner_id: expected_logical_owner_id.to_owned(),
            run_id: rejected.run_id,
            transition: ReplySuggestionTransitionV1::Reject(rejected.rejection),
            inference_request_bytes: None,
            source_cleanup: None,
            occurred_at_unix_millis: consumed_at_unix_millis,
        })
        .await
        .map_err(ReplySuggestionSourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

struct PreparedSourceV1 {
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    source_evidence_id: [u8; 16],
    source_evidence_revision: u64,
    source_content:
        Option<makosh_communications_ai_source_api::wire::CommunicationReplySourceContentReceiptV1>,
}

struct RejectedSourceV1 {
    run_id: [u8; 16],
    rejection: ReplySuggestionRejectionCodeV1,
}

fn decode_prepared(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<PreparedSourceV1, ReplySuggestionSourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ReplySuggestionSourceResultErrorV1::InvalidEnvelope)?;
    let run_id = validate_result_envelope(
        &envelope.contract,
        &communication_reply_source_prepared_contract_reference_v1(),
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
    let payload = CommunicationReplySourcePreparedV1::decode(envelope.payload.as_slice())
        .map_err(|_| ReplySuggestionSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id || payload.run_id.as_slice() != run_id
    {
        return Err(ReplySuggestionSourceResultErrorV1::InvalidPayload);
    }
    Ok(PreparedSourceV1 {
        run_id,
        source_message_id: id16(&payload.source_message_id)?,
        source_evidence_id: id16(&payload.source_evidence_id)?,
        source_evidence_revision: (payload.source_evidence_revision > 0)
            .then_some(payload.source_evidence_revision)
            .ok_or(ReplySuggestionSourceResultErrorV1::InvalidPayload)?,
        source_content: payload.source_content,
    })
}

fn decode_rejected(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<RejectedSourceV1, ReplySuggestionSourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ReplySuggestionSourceResultErrorV1::InvalidEnvelope)?;
    let run_id = validate_result_envelope(
        &envelope.contract,
        &communication_reply_source_rejected_contract_reference_v1(),
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
    let payload = CommunicationReplySourceRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| ReplySuggestionSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id || payload.run_id.as_slice() != run_id
    {
        return Err(ReplySuggestionSourceResultErrorV1::InvalidPayload);
    }
    let rejection = match payload.code {
        1 => ReplySuggestionRejectionCodeV1::InvalidRequest,
        7 => ReplySuggestionRejectionCodeV1::Policy,
        2..=6 => ReplySuggestionRejectionCodeV1::SourceRejected,
        _ => return Err(ReplySuggestionSourceResultErrorV1::InvalidPayload),
    };
    Ok(RejectedSourceV1 { run_id, rejection })
}

fn seal_request(
    draft: &makosh_communication_reply_suggestion_core::ReplySuggestionDraftV1,
    prepared: &PreparedSourceV1,
    source: AiPrivateSourceReceiptV1,
    logical_owner_id: &str,
) -> Result<CommunicationReplySuggestionInferenceRequestV1, ReplySuggestionSourceResultErrorV1> {
    seal_reply_inference_request_v1(CommunicationReplySuggestionInferenceRequestV1 {
        run_id: prepared.run_id.to_vec(),
        context: Some(AiContextReceiptV1 {
            context_id: context_id(&prepared.run_id, &prepared.source_evidence_id).to_vec(),
            use_case: AiUseCaseV1::AiUseCaseCommunicationReplySuggestion as i32,
            source_evidence_id: prepared.source_evidence_id.to_vec(),
            source_evidence_revision: prepared.source_evidence_revision,
            contract_major: AI_CONTRACT_MAJOR_V1,
            contract_revision: AI_CONTRACT_REVISION_V1,
            contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
            request_digest: Vec::new(),
        }),
        source: Some(source),
        tone: ai_tone(draft.tone) as i32,
        language: ai_language(draft.language) as i32,
        subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyPreserve as i32,
        maximum_output_bytes: AI_MAX_OUTPUT_BYTES_V1,
        maximum_output_tokens: AI_MAX_OUTPUT_TOKENS_V1,
        egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
        egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        logical_owner_id: logical_owner_id.to_owned(),
    })
    .map_err(|_| ReplySuggestionSourceResultErrorV1::InvalidPayload)
}

fn validate_result_envelope(
    actual_contract: &Option<ContractRefV1>,
    expected_contract: &ContractReferenceV1,
    source_module_id: Option<&str>,
    source_runtime_generation: u64,
    semantics: Option<&Semantics>,
    expected_outcome: ResultOutcomeV1,
) -> Result<[u8; 16], ReplySuggestionSourceResultErrorV1> {
    if !exact_contract(actual_contract.as_ref(), expected_contract)
        || source_module_id != Some(COMMUNICATIONS_RUNTIME_MODULE_ID_V1)
        || source_runtime_generation == 0
    {
        return Err(ReplySuggestionSourceResultErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Result(result)) = semantics else {
        return Err(ReplySuggestionSourceResultErrorV1::InvalidEnvelope);
    };
    if result.command_id.len() != 16
        || result.command_message_id.as_slice() != result.command_id
        || result.outcome != expected_outcome as i32
        || result.execution_attempt == 0
    {
        return Err(ReplySuggestionSourceResultErrorV1::InvalidEnvelope);
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
    digest.update(b"makosh.communication_reply_suggestion.context.v1\0");
    digest.update(run_id);
    digest.update(evidence_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

const fn ai_tone(value: ReplySuggestionToneV1) -> AiReplyToneV1 {
    match value {
        ReplySuggestionToneV1::Professional | ReplySuggestionToneV1::Formal => {
            AiReplyToneV1::AiReplyToneFormal
        }
        ReplySuggestionToneV1::Friendly => AiReplyToneV1::AiReplyToneWarm,
        ReplySuggestionToneV1::Concise => AiReplyToneV1::AiReplyToneConcise,
    }
}

const fn ai_language(value: ReplySuggestionLanguageV1) -> AiReplyLanguageV1 {
    match value {
        ReplySuggestionLanguageV1::Source => AiReplyLanguageV1::AiReplyLanguageAuto,
        ReplySuggestionLanguageV1::English => AiReplyLanguageV1::AiReplyLanguageEnglish,
        ReplySuggestionLanguageV1::Russian => AiReplyLanguageV1::AiReplyLanguageRussian,
        ReplySuggestionLanguageV1::Spanish => AiReplyLanguageV1::AiReplyLanguageSpanish,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReplySuggestionSourceResultErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| ReplySuggestionSourceResultErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(ReplySuggestionSourceResultErrorV1::InvalidPayload)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], ReplySuggestionSourceResultErrorV1> {
    let value: [u8; 32] = value
        .try_into()
        .map_err(|_| ReplySuggestionSourceResultErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(ReplySuggestionSourceResultErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> ReplySuggestionSourceResultErrorV1 {
    ReplySuggestionSourceResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_mapping_is_fixed_and_provider_neutral() {
        assert_eq!(
            ai_tone(ReplySuggestionToneV1::Friendly),
            AiReplyToneV1::AiReplyToneWarm
        );
        assert_eq!(
            ai_language(ReplySuggestionLanguageV1::Source),
            AiReplyLanguageV1::AiReplyLanguageAuto
        );
        assert_ne!(
            context_id(&[1; 16], &[2; 16]),
            context_id(&[1; 16], &[3; 16])
        );
    }
}
