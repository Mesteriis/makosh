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
    COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1, COMMUNICATION_NOTE_SOURCE_MAX_PROOF_BYTES_V1,
    COMMUNICATION_NOTE_SOURCE_PREPARE_CONTRACT_NAME_V1,
    COMMUNICATION_NOTE_SOURCE_PREPARED_CONTRACT_NAME_V1,
    COMMUNICATION_NOTE_SOURCE_REJECTED_CONTRACT_NAME_V1,
    COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID_V1, COMMUNICATIONS_NOTE_SOURCE_CONTRACT_MAJOR_V1,
    COMMUNICATIONS_NOTE_SOURCE_CONTRACT_REVISION_V1, COMMUNICATIONS_NOTE_SOURCE_OWNER_V1,
    COMMUNICATIONS_NOTE_SOURCE_SCHEMA_SHA256,
    wire::{
        CommunicationNoteSourceContentReceiptV1, CommunicationNoteSourcePreparedV1,
        CommunicationNoteSourceRejectedV1, PrepareCommunicationNoteSourceCommandV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationNoteSourceEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteSourceEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_communication_note_source_prepare_outbox_record_v1(
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
    logical_owner_id: &str,
    deadline_unix_seconds: i64,
    context: &CommunicationNoteSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationNoteSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    validate_common_payload(
        &run_id,
        &source_message_id,
        expected_source_revision,
        logical_owner_id,
    )?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(CommunicationNoteSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    let payload = PrepareCommunicationNoteSourceCommandV1 {
        run_id: run_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
        logical_owner_id: logical_owner_id.to_owned(),
    }
    .encode_to_vec();
    build_envelope(
        run_id,
        &run_id,
        &[],
        COMMUNICATION_NOTE_SOURCE_PREPARE_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: run_id.to_vec(),
            target_capability: COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(
                [b"communications-note-source-prepare-v1".as_slice(), &run_id].concat(),
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

pub fn build_communication_note_source_prepared_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationNoteSourcePreparedV1,
    context: &CommunicationNoteSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationNoteSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = validate_prepared_payload(&payload)?;
    if !valid_id(&command_message_id) {
        return Err(CommunicationNoteSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_envelope(
        result_message_id(b"prepared", &run_id),
        &run_id,
        &command_message_id,
        COMMUNICATION_NOTE_SOURCE_PREPARED_CONTRACT_NAME_V1,
        result_semantics(
            &run_id,
            &command_message_id,
            ResultOutcomeV1::Succeeded,
            context,
        ),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_note_source_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationNoteSourceRejectedV1,
    context: &CommunicationNoteSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationNoteSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = id16(&payload.run_id)?;
    if !valid_id(&command_message_id)
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload.code == 0
    {
        return Err(CommunicationNoteSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_envelope(
        result_message_id(b"rejected", &run_id),
        &run_id,
        &command_message_id,
        COMMUNICATION_NOTE_SOURCE_REJECTED_CONTRACT_NAME_V1,
        result_semantics(
            &run_id,
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
    context: &CommunicationNoteSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationNoteSourceEnvelopeBuildErrorV1> {
    let timestamp = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: COMMUNICATIONS_NOTE_SOURCE_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: COMMUNICATIONS_NOTE_SOURCE_CONTRACT_MAJOR_V1,
            revision: COMMUNICATIONS_NOTE_SOURCE_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATIONS_NOTE_SOURCE_SCHEMA_SHA256.to_vec(),
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
        .map_err(|_| CommunicationNoteSourceEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn result_semantics(
    command_id: &[u8; 16],
    command_message_id: &[u8; 16],
    outcome: ResultOutcomeV1,
    context: &CommunicationNoteSourceEnvelopeContextV1,
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
    context: &CommunicationNoteSourceEnvelopeContextV1,
) -> Result<(), CommunicationNoteSourceEnvelopeBuildErrorV1> {
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
        return Err(CommunicationNoteSourceEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn validate_prepared_payload(
    payload: &CommunicationNoteSourcePreparedV1,
) -> Result<[u8; 16], CommunicationNoteSourceEnvelopeBuildErrorV1> {
    let run_id = id16(&payload.run_id)?;
    let source_message_id = id16(&payload.source_message_id)?;
    validate_common_payload(
        &run_id,
        &source_message_id,
        payload.expected_source_revision,
        &payload.logical_owner_id,
    )?;
    id16(&payload.source_evidence_id)?;
    if payload.source_evidence_revision == 0
        || payload
            .source_content
            .as_ref()
            .is_none_or(|receipt| !valid_source_receipt(receipt))
    {
        return Err(CommunicationNoteSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(run_id)
}

fn validate_common_payload(
    run_id: &[u8; 16],
    source_message_id: &[u8; 16],
    expected_source_revision: u64,
    logical_owner_id: &str,
) -> Result<(), CommunicationNoteSourceEnvelopeBuildErrorV1> {
    if !valid_id(run_id)
        || !valid_id(source_message_id)
        || expected_source_revision == 0
        || !valid_logical_owner_id(logical_owner_id)
    {
        return Err(CommunicationNoteSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(())
}

fn valid_source_receipt(receipt: &CommunicationNoteSourceContentReceiptV1) -> bool {
    receipt.reference_id.len() == 16
        && receipt.reference_id.iter().any(|byte| *byte != 0)
        && (1..=COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1).contains(&receipt.declared_bytes)
        && receipt.sha256.len() == 32
        && receipt.sha256.iter().any(|byte| *byte != 0)
        && !receipt.custody_transfer_source_proof.is_empty()
        && receipt.custody_transfer_source_proof.len()
            <= COMMUNICATION_NOTE_SOURCE_MAX_PROOF_BYTES_V1
}

fn valid_id(id: &[u8; 16]) -> bool {
    id.iter().any(|byte| *byte != 0)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], CommunicationNoteSourceEnvelopeBuildErrorV1> {
    bytes
        .try_into()
        .ok()
        .filter(valid_id)
        .ok_or(CommunicationNoteSourceEnvelopeBuildErrorV1::InvalidPayload)
}

fn result_message_id(label: &[u8], run_id: &[u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-note-source-result-v1");
    hasher.update(label);
    hasher.update(run_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    Sha256::digest(runtime_instance_id.as_bytes())[..16]
        .try_into()
        .expect("digest prefix")
}

fn outbox_error(_: OutboxRecordError) -> CommunicationNoteSourceEnvelopeBuildErrorV1 {
    CommunicationNoteSourceEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::DurableEnvelopeV1;

    fn context() -> CommunicationNoteSourceEnvelopeContextV1 {
        CommunicationNoteSourceEnvelopeContextV1 {
            module_id: "makosh-communication-note-candidate-runtime".to_owned(),
            runtime_instance_id: "runtime-note-source-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 12,
        }
    }

    fn prepared_payload() -> CommunicationNoteSourcePreparedV1 {
        CommunicationNoteSourcePreparedV1 {
            run_id: vec![1; 16],
            source_message_id: vec![2; 16],
            expected_source_revision: 3,
            source_evidence_id: vec![4; 16],
            source_evidence_revision: 5,
            source_content: Some(CommunicationNoteSourceContentReceiptV1 {
                reference_id: vec![6; 16],
                declared_bytes: 42,
                sha256: vec![7; 32],
                custody_transfer_source_proof: vec![8; 64],
            }),
            logical_owner_id: "owner-1".to_owned(),
        }
    }

    #[test]
    fn command_contains_only_canonical_ids_and_exact_capability() {
        let record = build_communication_note_source_prepare_outbox_record_v1(
            [1; 16],
            [2; 16],
            3,
            "owner-1",
            1_800_000_030,
            &context(),
        )
        .expect("record");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("valid envelope");
        let Some(Semantics::Command(command)) = envelope.semantics else {
            panic!("command semantics");
        };
        assert_eq!(command.target_capability, "communications.note-source.v1");
        let payload = PrepareCommunicationNoteSourceCommandV1::decode(envelope.payload.as_slice())
            .expect("payload");
        assert_eq!(payload.expected_source_revision, 3);
        assert_eq!(payload.logical_owner_id, "owner-1");
    }

    #[test]
    fn prepared_result_is_receipt_only_and_hash_bound() {
        let record = build_communication_note_source_prepared_outbox_record_v1(
            [9; 16],
            prepared_payload(),
            &context(),
        )
        .expect("record");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("valid envelope");
        assert_eq!(
            envelope.contract.expect("contract").name,
            "communication_note_source_prepared"
        );
        let payload = CommunicationNoteSourcePreparedV1::decode(envelope.payload.as_slice())
            .expect("payload");
        assert_eq!(payload.source_content.expect("receipt").sha256, vec![7; 32]);
    }

    #[test]
    fn rejects_invalid_revision_and_oversized_proof() {
        assert_eq!(
            build_communication_note_source_prepare_outbox_record_v1(
                [1; 16],
                [2; 16],
                0,
                "owner-1",
                1_800_000_030,
                &context(),
            ),
            Err(CommunicationNoteSourceEnvelopeBuildErrorV1::InvalidPayload)
        );
        let mut payload = prepared_payload();
        payload
            .source_content
            .as_mut()
            .expect("receipt")
            .custody_transfer_source_proof =
            vec![8; COMMUNICATION_NOTE_SOURCE_MAX_PROOF_BYTES_V1 + 1];
        assert_eq!(
            build_communication_note_source_prepared_outbox_record_v1([9; 16], payload, &context(),),
            Err(CommunicationNoteSourceEnvelopeBuildErrorV1::InvalidPayload)
        );
    }
}
