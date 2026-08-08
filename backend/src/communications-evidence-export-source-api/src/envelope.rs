use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ResultMetadataV1, ResultOutcomeV1, SourceFenceV1, SourceRefV1,
        durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    COMMUNICATIONS_EVIDENCE_EXPORT_SOURCE_SCHEMA_SHA256, EVIDENCE_EXPORT_MAX_MESSAGES_V1,
    EVIDENCE_EXPORT_PREPARE_CONTRACT_NAME_V1, EVIDENCE_EXPORT_PREPARED_CONTRACT_NAME_V1,
    EVIDENCE_EXPORT_REJECTED_CONTRACT_NAME_V1, EVIDENCE_EXPORT_SOURCE_CONTRACT_MAJOR_V1,
    EVIDENCE_EXPORT_SOURCE_CONTRACT_REVISION_V1, EVIDENCE_EXPORT_SOURCE_OWNER_V1,
    wire::{EvidenceExportPreparedV1, EvidenceExportRejectedV1, PrepareEvidenceExportCommandV1},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceExportEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceExportEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_evidence_export_prepare_outbox_record_v1(
    command_id: [u8; 16],
    logical_owner_id: &str,
    message_ids: &[[u8; 16]],
    deadline_unix_seconds: i64,
    context: &EvidenceExportEnvelopeContextV1,
) -> Result<OutboxRecordV1, EvidenceExportEnvelopeBuildErrorV1> {
    validate_context(context)?;
    if !valid_id(&command_id)
        || !valid_logical_owner_id(logical_owner_id)
        || message_ids.is_empty()
        || message_ids.len() > EVIDENCE_EXPORT_MAX_MESSAGES_V1
        || message_ids.iter().any(|id| !valid_id(id))
        || has_duplicates(message_ids)
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(EvidenceExportEnvelopeBuildErrorV1::InvalidPayload);
    }
    let payload = PrepareEvidenceExportCommandV1 {
        export_id: command_id.to_vec(),
        message_ids: message_ids.iter().map(|id| id.to_vec()).collect(),
        logical_owner_id: logical_owner_id.to_owned(),
    }
    .encode_to_vec();
    build_envelope(
        command_id,
        &command_id,
        &[],
        EVIDENCE_EXPORT_PREPARE_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: command_id.to_vec(),
            target_capability: "communications.export-source.v1".to_owned(),
            idempotency_key: Sha256::digest(
                [b"communications-export-prepare-v1".as_slice(), &command_id].concat(),
            )
            .to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        }),
        payload,
        context,
    )
}

pub fn build_evidence_export_prepared_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: EvidenceExportPreparedV1,
    context: &EvidenceExportEnvelopeContextV1,
) -> Result<OutboxRecordV1, EvidenceExportEnvelopeBuildErrorV1> {
    validate_result_payload(
        &payload.export_id,
        &payload.logical_owner_id,
        payload.items.len(),
    )?;
    let export_id = id16(&payload.export_id)?;
    build_envelope(
        result_message_id(b"prepared", &export_id),
        &export_id,
        &command_message_id,
        EVIDENCE_EXPORT_PREPARED_CONTRACT_NAME_V1,
        result_semantics(
            &export_id,
            &command_message_id,
            ResultOutcomeV1::Succeeded,
            context,
        ),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_evidence_export_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: EvidenceExportRejectedV1,
    context: &EvidenceExportEnvelopeContextV1,
) -> Result<OutboxRecordV1, EvidenceExportEnvelopeBuildErrorV1> {
    validate_result_payload(&payload.export_id, &payload.logical_owner_id, 1)?;
    if payload.code == 0 {
        return Err(EvidenceExportEnvelopeBuildErrorV1::InvalidPayload);
    }
    let export_id = id16(&payload.export_id)?;
    build_envelope(
        result_message_id(b"rejected", &export_id),
        &export_id,
        &command_message_id,
        EVIDENCE_EXPORT_REJECTED_CONTRACT_NAME_V1,
        result_semantics(
            &export_id,
            &command_message_id,
            ResultOutcomeV1::Rejected,
            context,
        ),
        payload.encode_to_vec(),
        context,
    )
}

fn build_envelope(
    message_id: [u8; 16],
    partition_key: &[u8; 16],
    causation_message_id: &[u8],
    contract_name: &str,
    semantics: Semantics,
    payload: Vec<u8>,
    context: &EvidenceExportEnvelopeContextV1,
) -> Result<OutboxRecordV1, EvidenceExportEnvelopeBuildErrorV1> {
    let timestamp = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: EVIDENCE_EXPORT_SOURCE_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: EVIDENCE_EXPORT_SOURCE_CONTRACT_MAJOR_V1,
            revision: EVIDENCE_EXPORT_SOURCE_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATIONS_EVIDENCE_EXPORT_SOURCE_SCHEMA_SHA256.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp),
        partition_key: partition_key.to_vec(),
        causation_message_id: causation_message_id.to_vec(),
        correlation_id: partition_key.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(semantics),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| EvidenceExportEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn result_semantics(
    command_id: &[u8; 16],
    command_message_id: &[u8; 16],
    outcome: ResultOutcomeV1,
    context: &EvidenceExportEnvelopeContextV1,
) -> Semantics {
    Semantics::Result(ResultMetadataV1 {
        command_id: command_id.to_vec(),
        command_message_id: command_message_id.to_vec(),
        outcome: outcome as i32,
        completed_at: Some(Timestamp {
            seconds: context.recorded_at_unix_seconds,
            nanos: context.recorded_at_nanos,
        }),
        execution_attempt: 1,
    })
}

fn validate_context(
    context: &EvidenceExportEnvelopeContextV1,
) -> Result<(), EvidenceExportEnvelopeBuildErrorV1> {
    if context.module_id.is_empty()
        || context.module_id.len() > 128
        || !context.module_id.is_ascii()
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 256
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(EvidenceExportEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn validate_result_payload(
    export_id: &[u8],
    logical_owner_id: &str,
    item_count: usize,
) -> Result<(), EvidenceExportEnvelopeBuildErrorV1> {
    if export_id.len() != 16
        || export_id.iter().all(|byte| *byte == 0)
        || !valid_logical_owner_id(logical_owner_id)
        || item_count == 0
    {
        return Err(EvidenceExportEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(())
}

fn has_duplicates(ids: &[[u8; 16]]) -> bool {
    ids.iter()
        .enumerate()
        .any(|(index, id)| ids[..index].contains(id))
}

fn valid_id(id: &[u8; 16]) -> bool {
    id.iter().any(|byte| *byte != 0)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], EvidenceExportEnvelopeBuildErrorV1> {
    bytes
        .try_into()
        .ok()
        .filter(valid_id)
        .ok_or(EvidenceExportEnvelopeBuildErrorV1::InvalidPayload)
}

fn result_message_id(label: &[u8], export_id: &[u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-evidence-export-source-result-v1");
    hasher.update(label);
    hasher.update(export_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    Sha256::digest(runtime_instance_id.as_bytes())[..16]
        .try_into()
        .expect("digest prefix")
}

fn outbox_error(_: OutboxRecordError) -> EvidenceExportEnvelopeBuildErrorV1 {
    EvidenceExportEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::DurableEnvelopeV1;

    fn context() -> EvidenceExportEnvelopeContextV1 {
        EvidenceExportEnvelopeContextV1 {
            module_id: "makosh-communications-export-runtime".to_owned(),
            runtime_instance_id: "runtime-export-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 12,
        }
    }

    #[test]
    fn command_contains_only_canonical_ids_and_exact_target_capability() {
        let record = build_evidence_export_prepare_outbox_record_v1(
            [1; 16],
            "owner-1",
            &[[2; 16], [3; 16]],
            1_800_000_030,
            &context(),
        )
        .expect("command");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        let Some(Semantics::Command(command)) = envelope.semantics else {
            panic!("command semantics");
        };
        assert_eq!(command.target_capability, "communications.export-source.v1");
        let payload =
            PrepareEvidenceExportCommandV1::decode(envelope.payload.as_slice()).expect("payload");
        assert_eq!(payload.message_ids, vec![vec![2; 16], vec![3; 16]]);
        assert_eq!(payload.logical_owner_id, "owner-1");
    }

    #[test]
    fn command_rejects_duplicate_or_unbounded_ids() {
        assert_eq!(
            build_evidence_export_prepare_outbox_record_v1(
                [1; 16],
                "owner-1",
                &[[2; 16], [2; 16]],
                1_800_000_030,
                &context(),
            ),
            Err(EvidenceExportEnvelopeBuildErrorV1::InvalidPayload)
        );
        assert_eq!(
            build_evidence_export_prepare_outbox_record_v1(
                [1; 16],
                "owner-1",
                &vec![[2; 16]; EVIDENCE_EXPORT_MAX_MESSAGES_V1 + 1],
                1_800_000_030,
                &context(),
            ),
            Err(EvidenceExportEnvelopeBuildErrorV1::InvalidPayload)
        );
    }
}
