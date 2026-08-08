use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, AI_MAX_OUTPUT_BYTES_V1, AI_MAX_OUTPUT_TOKENS_V1,
    seal_attachment_translation_inference_request_v1,
    wire::{
        AiContextReceiptV1, AiEgressPolicyV1, AiTranslationLanguageV1, AiUseCaseV1,
        AttachmentTranslationInferenceRequestV1,
    },
};
use makosh_attachment_translation_core::{
    AttachmentTranslationLanguageV1, AttachmentTranslationRejectionCodeV1,
    AttachmentTranslationStateV1, AttachmentTranslationTransitionV1,
};
use makosh_attachment_translation_ingress::{
    attachment_translation_source_prepared_contract_reference_v1,
    attachment_translation_source_rejected_contract_reference_v1,
    attachment_translation_source_request_id_v1,
    wire::{
        AttachmentTranslationSourcePreparedV1, AttachmentTranslationSourceRejectCodeV1,
        AttachmentTranslationSourceRejectedV1,
    },
};
use makosh_attachment_translation_persistence::{
    AttachmentTranslationPersistenceErrorV1, AttachmentTranslationPersistenceV1,
    AttachmentTranslationSourceResultV1,
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::ContractReferenceV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    AttachmentTranslationBlobErrorV1, AttachmentTranslationInferenceErrorV1,
    AttachmentTranslationInferenceExecutionV1,
    blob_materialization::{
        AttachmentTranslationSourceBlobReceiptV1, materialize_translation_source_for_ai_v1,
        release_translation_source_blobs_v1,
    },
    inference::complete_attachment_translation_inference_v1,
};

const ATTACHMENT_TEXT_EXTRACTION_RUNTIME_MODULE_ID_V1: &str =
    "makosh-attachment-text-extraction-runtime";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationSourceResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Blob(AttachmentTranslationBlobErrorV1),
    Inference(AttachmentTranslationInferenceErrorV1),
    Persistence(AttachmentTranslationPersistenceErrorV1),
    EventUnavailable,
}

#[allow(clippy::too_many_arguments)]
pub async fn consume_translation_source_prepared_once_v1(
    persistence: &AttachmentTranslationPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    expected_logical_owner_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
    consumed_at_unix_millis: i64,
) -> Result<bool, AttachmentTranslationSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| AttachmentTranslationSourceResultErrorV1::InvalidEnvelope)?;
    let prepared = decode_prepared(&record, expected_logical_owner_id)?;
    let run = persistence
        .load_run(expected_logical_owner_id, &prepared.translation_run_id)
        .await
        .map_err(AttachmentTranslationSourceResultErrorV1::Persistence)?;
    validate_prepared_for_run(&prepared, &run)?;
    if matches!(
        run.status.state,
        AttachmentTranslationStateV1::Ready | AttachmentTranslationStateV1::Rejected
    ) && run.cleanup_completed_at_unix_millis.is_some()
    {
        delivery.acknowledge().await.map_err(event_error)?;
        return Ok(true);
    }
    let materialized = materialize_translation_source_for_ai_v1(
        channel,
        dispatcher,
        prepared.translation_run_id,
        &AttachmentTranslationSourceBlobReceiptV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            reference_id: prepared.source_reference_id,
            declared_bytes: prepared.declared_size,
            sha256: prepared.receipt_sha256,
            custody_proof: prepared.custody_transfer_source_proof.clone(),
        },
    )
    .map_err(AttachmentTranslationSourceResultErrorV1::Blob)?;
    let sealed = seal_request(&run, &prepared, materialized.ai_source.clone())?;
    let request_digest = array32(
        &sealed
            .context
            .as_ref()
            .ok_or(AttachmentTranslationSourceResultErrorV1::InvalidPayload)?
            .request_digest,
    )?;
    let source_sha256 = array32(
        &sealed
            .source
            .as_ref()
            .ok_or(AttachmentTranslationSourceResultErrorV1::InvalidPayload)?
            .sha256,
    )?;
    let persisted = persistence
        .persist_source_result(AttachmentTranslationSourceResultV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            logical_owner_id: expected_logical_owner_id.to_owned(),
            run_id: prepared.translation_run_id,
            transition: AttachmentTranslationTransitionV1::SourcePrepared {
                source_sha256,
                source_size_bytes: prepared.declared_size,
                inference_request_digest: request_digest,
            },
            inference_request_bytes: Some(sealed.encode_to_vec()),
            source_authority: Some(materialized.source_authority.clone()),
            occurred_at_unix_millis: consumed_at_unix_millis,
        })
        .await
        .map_err(AttachmentTranslationSourceResultErrorV1::Persistence)?;
    let persisted = match persisted {
        makosh_attachment_translation_persistence::AttachmentTranslationInboxResultV1::Applied(value)
        | makosh_attachment_translation_persistence::AttachmentTranslationInboxResultV1::Duplicate(value) => value,
    };
    let accepted = match persisted.status.state {
        AttachmentTranslationStateV1::AwaitingInference
        | AttachmentTranslationStateV1::MaterializingResult => {
            complete_attachment_translation_inference_v1(
                persistence,
                channel,
                dispatcher,
                &persisted,
                sealed,
                AttachmentTranslationInferenceExecutionV1 {
                    runtime_generation,
                    grant_epoch,
                    occurred_at_unix_millis: consumed_at_unix_millis,
                },
            )
            .await
            .map_err(AttachmentTranslationSourceResultErrorV1::Inference)?
        }
        AttachmentTranslationStateV1::Ready => true,
        AttachmentTranslationStateV1::Rejected => false,
        _ => return Err(AttachmentTranslationSourceResultErrorV1::InvalidPayload),
    };
    release_translation_source_blobs_v1(
        channel,
        dispatcher,
        prepared.translation_run_id,
        &materialized.ai_source,
        &materialized.source_authority,
        accepted,
    )
    .map_err(AttachmentTranslationSourceResultErrorV1::Blob)?;
    persistence
        .complete_source_cleanup(
            expected_logical_owner_id,
            &prepared.translation_run_id,
            &materialized.source_authority,
            consumed_at_unix_millis,
        )
        .await
        .map_err(AttachmentTranslationSourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub async fn consume_translation_source_rejected_once_v1(
    persistence: &AttachmentTranslationPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, AttachmentTranslationSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| AttachmentTranslationSourceResultErrorV1::InvalidEnvelope)?;
    let rejected = decode_rejected(&record, expected_logical_owner_id)?;
    persistence
        .persist_source_result(AttachmentTranslationSourceResultV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            logical_owner_id: expected_logical_owner_id.to_owned(),
            run_id: rejected.translation_run_id,
            transition: AttachmentTranslationTransitionV1::Reject(rejected.rejection),
            inference_request_bytes: None,
            source_authority: None,
            occurred_at_unix_millis: consumed_at_unix_millis,
        })
        .await
        .map_err(AttachmentTranslationSourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

struct PreparedSourceV1 {
    request_id: [u8; 16],
    translation_run_id: [u8; 16],
    source_extraction_run_id: [u8; 16],
    source_revision: u64,
    source_reference_id: [u8; 16],
    declared_size: u64,
    receipt_sha256: [u8; 32],
    custody_transfer_source_proof: Vec<u8>,
}

struct RejectedSourceV1 {
    translation_run_id: [u8; 16],
    rejection: AttachmentTranslationRejectionCodeV1,
}

fn decode_prepared(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<PreparedSourceV1, AttachmentTranslationSourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| AttachmentTranslationSourceResultErrorV1::InvalidEnvelope)?;
    validate_result_envelope(
        envelope.contract.as_ref(),
        &attachment_translation_source_prepared_contract_reference_v1(),
        envelope
            .source
            .as_ref()
            .map(|source| source.module_id.as_str()),
        envelope.semantics.as_ref(),
        ResultOutcomeV1::Succeeded,
    )?;
    let payload = AttachmentTranslationSourcePreparedV1::decode(envelope.payload.as_slice())
        .map_err(|_| AttachmentTranslationSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id {
        return Err(AttachmentTranslationSourceResultErrorV1::InvalidPayload);
    }
    Ok(PreparedSourceV1 {
        request_id: id16(&payload.request_id)?,
        translation_run_id: id16(&payload.translation_run_id)?,
        source_extraction_run_id: id16(&payload.source_extraction_run_id)?,
        source_revision: nonzero_revision(payload.source_revision)?,
        source_reference_id: id16(&payload.source_reference_id)?,
        declared_size: nonzero_revision(payload.declared_size)?,
        receipt_sha256: array32(&payload.receipt_sha256)?,
        custody_transfer_source_proof: payload.custody_transfer_source_proof,
    })
}

fn decode_rejected(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<RejectedSourceV1, AttachmentTranslationSourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| AttachmentTranslationSourceResultErrorV1::InvalidEnvelope)?;
    validate_result_envelope(
        envelope.contract.as_ref(),
        &attachment_translation_source_rejected_contract_reference_v1(),
        envelope
            .source
            .as_ref()
            .map(|source| source.module_id.as_str()),
        envelope.semantics.as_ref(),
        ResultOutcomeV1::Rejected,
    )?;
    let payload = AttachmentTranslationSourceRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| AttachmentTranslationSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id {
        return Err(AttachmentTranslationSourceResultErrorV1::InvalidPayload);
    }
    let code = AttachmentTranslationSourceRejectCodeV1::try_from(payload.code)
        .map_err(|_| AttachmentTranslationSourceResultErrorV1::InvalidPayload)?;
    let rejection = match code {
        AttachmentTranslationSourceRejectCodeV1::InvalidRequest => {
            AttachmentTranslationRejectionCodeV1::InvalidRequest
        }
        AttachmentTranslationSourceRejectCodeV1::Policy => {
            AttachmentTranslationRejectionCodeV1::Policy
        }
        AttachmentTranslationSourceRejectCodeV1::NotReady
        | AttachmentTranslationSourceRejectCodeV1::StaleRevision
        | AttachmentTranslationSourceRejectCodeV1::CustodyUnavailable => {
            AttachmentTranslationRejectionCodeV1::SourceRejected
        }
        AttachmentTranslationSourceRejectCodeV1::Unspecified => {
            return Err(AttachmentTranslationSourceResultErrorV1::InvalidPayload);
        }
    };
    Ok(RejectedSourceV1 {
        translation_run_id: id16(&payload.translation_run_id)?,
        rejection,
    })
}

fn validate_prepared_for_run(
    prepared: &PreparedSourceV1,
    run: &makosh_attachment_translation_persistence::PersistedAttachmentTranslationRunV1,
) -> Result<(), AttachmentTranslationSourceResultErrorV1> {
    if prepared.translation_run_id != run.draft.run_id
        || prepared.source_extraction_run_id != run.draft.source_extraction_run_id
        || prepared.source_revision != run.draft.expected_source_revision
        || prepared.request_id
            != attachment_translation_source_request_id_v1(
                run.draft.run_id,
                run.draft.source_extraction_run_id,
                run.draft.expected_source_revision,
            )
    {
        return Err(AttachmentTranslationSourceResultErrorV1::InvalidPayload);
    }
    Ok(())
}

fn seal_request(
    run: &makosh_attachment_translation_persistence::PersistedAttachmentTranslationRunV1,
    prepared: &PreparedSourceV1,
    source: makosh_ai_contracts::wire::AiPrivateSourceReceiptV1,
) -> Result<AttachmentTranslationInferenceRequestV1, AttachmentTranslationSourceResultErrorV1> {
    seal_attachment_translation_inference_request_v1(AttachmentTranslationInferenceRequestV1 {
        run_id: run.draft.run_id.to_vec(),
        context: Some(AiContextReceiptV1 {
            context_id: context_id(run.draft.run_id, prepared.source_extraction_run_id).to_vec(),
            use_case: AiUseCaseV1::AiUseCaseAttachmentTranslation as i32,
            source_evidence_id: prepared.source_extraction_run_id.to_vec(),
            source_evidence_revision: prepared.source_revision,
            contract_major: AI_CONTRACT_MAJOR_V1,
            contract_revision: AI_CONTRACT_REVISION_V1,
            contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
            request_digest: Vec::new(),
        }),
        source: Some(source),
        target_language: ai_target_language(run.draft.target_language) as i32,
        maximum_output_bytes: AI_MAX_OUTPUT_BYTES_V1,
        maximum_output_tokens: AI_MAX_OUTPUT_TOKENS_V1,
        egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
        egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        logical_owner_id: run.logical_owner_id.clone(),
    })
    .map_err(|_| AttachmentTranslationSourceResultErrorV1::InvalidPayload)
}

fn validate_result_envelope(
    actual_contract: Option<&makosh_events_protocol::v1::ContractRefV1>,
    expected_contract: &ContractReferenceV1,
    source_module_id: Option<&str>,
    semantics: Option<&Semantics>,
    expected_outcome: ResultOutcomeV1,
) -> Result<(), AttachmentTranslationSourceResultErrorV1> {
    let contract =
        actual_contract.ok_or(AttachmentTranslationSourceResultErrorV1::InvalidEnvelope)?;
    let exact_contract = contract.owner == expected_contract.owner
        && contract.name == expected_contract.name
        && contract.major == expected_contract.major
        && contract.revision == expected_contract.revision
        && contract.schema_sha256 == expected_contract.schema_sha256;
    let exact_source = source_module_id == Some(ATTACHMENT_TEXT_EXTRACTION_RUNTIME_MODULE_ID_V1);
    let exact_outcome = matches!(semantics, Some(Semantics::Result(result)) if result.outcome == expected_outcome as i32);
    if exact_contract && exact_source && exact_outcome {
        Ok(())
    } else {
        Err(AttachmentTranslationSourceResultErrorV1::InvalidEnvelope)
    }
}

const fn ai_target_language(value: AttachmentTranslationLanguageV1) -> AiTranslationLanguageV1 {
    match value {
        AttachmentTranslationLanguageV1::English => {
            AiTranslationLanguageV1::AiTranslationLanguageEnglish
        }
        AttachmentTranslationLanguageV1::Russian => {
            AiTranslationLanguageV1::AiTranslationLanguageRussian
        }
        AttachmentTranslationLanguageV1::Spanish => {
            AiTranslationLanguageV1::AiTranslationLanguageSpanish
        }
    }
}

fn context_id(run_id: [u8; 16], source_id: [u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment-translation.ai-context.v1\0");
    digest.update(run_id);
    digest.update(source_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], AttachmentTranslationSourceResultErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(AttachmentTranslationSourceResultErrorV1::InvalidPayload)
}

fn array32(value: &[u8]) -> Result<[u8; 32], AttachmentTranslationSourceResultErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(AttachmentTranslationSourceResultErrorV1::InvalidPayload)
}

fn nonzero_revision(value: u64) -> Result<u64, AttachmentTranslationSourceResultErrorV1> {
    (value > 0)
        .then_some(value)
        .ok_or(AttachmentTranslationSourceResultErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> AttachmentTranslationSourceResultErrorV1 {
    AttachmentTranslationSourceResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_and_source_request_identities_are_distinct() {
        assert_ne!(
            context_id([1; 16], [2; 16]),
            attachment_translation_source_request_id_v1([1; 16], [2; 16], 1)
        );
    }
}
