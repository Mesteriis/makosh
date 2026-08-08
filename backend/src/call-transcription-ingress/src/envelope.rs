use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    CALL_TRANSCRIPTION_INGRESS_SCHEMA_SHA256, CONTRACT_MAJOR_V1, CONTRACT_REVISION_V1, OWNER_ID_V1,
    RECORDING_READY_CONTRACT_NAME_V1, RECORDING_REJECTED_CONTRACT_NAME_V1,
    recording_ready_event_id_v1, recording_rejected_event_id_v1,
    wire::{RecordingReadyV1, RecordingRejectedV1},
};

const MAX_PROOF_BYTES_V1: usize = 2_048;
const MAX_AUDIO_BYTES_V1: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallTranscriptionIngressEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionIngressEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_recording_ready_outbox_record_v1(
    payload: RecordingReadyV1,
    context: &CallTranscriptionIngressEnvelopeContextV1,
) -> Result<OutboxRecordV1, CallTranscriptionIngressEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let recording_id = id16(&payload.recording_evidence_id)?;
    let expected_event_id = recording_ready_event_id_v1(recording_id, payload.recording_revision);
    if id16(&payload.event_id)? != expected_event_id
        || id16(&payload.request_id)?.iter().all(|byte| *byte == 0)
        || id16(&payload.call_evidence_id)?
            .iter()
            .all(|byte| *byte == 0)
        || id16(&payload.consent_receipt_id)?
            .iter()
            .all(|byte| *byte == 0)
        || id16(&payload.target_blob_reference_id)?
            .iter()
            .all(|byte| *byte == 0)
        || payload.call_evidence_revision == 0
        || payload.recording_revision == 0
        || payload.consent_policy_revision == 0
        || payload.consent_scope != "call_transcription"
        || payload.audio_format != "wav_pcm_s16le_mono_16000"
        || !(1..=MAX_AUDIO_BYTES_V1).contains(&payload.declared_bytes)
        || payload.duration_millis == 0
        || sha256(&payload.audio_sha256).is_err()
        || payload.custody_transfer_source_proof.is_empty()
        || payload.custody_transfer_source_proof.len() > MAX_PROOF_BYTES_V1
        || !valid_identity(&payload.logical_owner_id)
    {
        return Err(CallTranscriptionIngressEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_event(
        expected_event_id,
        recording_id,
        RECORDING_READY_CONTRACT_NAME_V1,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_recording_rejected_outbox_record_v1(
    payload: RecordingRejectedV1,
    context: &CallTranscriptionIngressEnvelopeContextV1,
) -> Result<OutboxRecordV1, CallTranscriptionIngressEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let recording_id = id16(&payload.recording_evidence_id)?;
    let expected_event_id =
        recording_rejected_event_id_v1(recording_id, payload.recording_revision);
    if id16(&payload.event_id)? != expected_event_id
        || id16(&payload.request_id)?.iter().all(|byte| *byte == 0)
        || id16(&payload.call_evidence_id)?
            .iter()
            .all(|byte| *byte == 0)
        || payload.call_evidence_revision == 0
        || payload.recording_revision == 0
        || !valid_identity(&payload.rejection_code)
        || !valid_identity(&payload.logical_owner_id)
    {
        return Err(CallTranscriptionIngressEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_event(
        expected_event_id,
        recording_id,
        RECORDING_REJECTED_CONTRACT_NAME_V1,
        payload.encode_to_vec(),
        context,
    )
}

fn build_event(
    message_id: [u8; 16],
    recording_id: [u8; 16],
    contract_name: &str,
    payload: Vec<u8>,
    context: &CallTranscriptionIngressEnvelopeContextV1,
) -> Result<OutboxRecordV1, CallTranscriptionIngressEnvelopeBuildErrorV1> {
    let occurred_at = timestamp(context);
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: OWNER_ID_V1.to_owned(),
            name: contract_name.to_owned(),
            major: CONTRACT_MAJOR_V1,
            revision: CONTRACT_REVISION_V1,
            schema_sha256: CALL_TRANSCRIPTION_INGRESS_SCHEMA_SHA256.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: Sha256::digest(context.runtime_instance_id.as_bytes())[..16]
                .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(occurred_at),
        partition_key: recording_id.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: recording_id.to_vec(),
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
        semantics: Some(Semantics::Event(EventMetadataV1 {
            occurred_at: Some(occurred_at),
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| CallTranscriptionIngressEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &CallTranscriptionIngressEnvelopeContextV1,
) -> Result<(), CallTranscriptionIngressEnvelopeBuildErrorV1> {
    if !valid_identity(&context.module_id)
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(CallTranscriptionIngressEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], CallTranscriptionIngressEnvelopeBuildErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(CallTranscriptionIngressEnvelopeBuildErrorV1::InvalidPayload)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], CallTranscriptionIngressEnvelopeBuildErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(CallTranscriptionIngressEnvelopeBuildErrorV1::InvalidPayload)
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn timestamp(context: &CallTranscriptionIngressEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn outbox_error(_: OutboxRecordError) -> CallTranscriptionIngressEnvelopeBuildErrorV1 {
    CallTranscriptionIngressEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::{
        v1::durable_envelope_v1::Semantics, validation::envelope::decode_envelope_v1,
    };

    fn context() -> CallTranscriptionIngressEnvelopeContextV1 {
        CallTranscriptionIngressEnvelopeContextV1 {
            module_id: "makosh-desktop-call-recording-runtime".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 4,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 0,
        }
    }

    #[test]
    fn ready_event_is_target_owned_and_exact() {
        let recording_id = [5; 16];
        let event_id = recording_ready_event_id_v1(recording_id, 4);
        let record = build_recording_ready_outbox_record_v1(
            RecordingReadyV1 {
                event_id: event_id.to_vec(),
                request_id: vec![1; 16],
                call_evidence_id: vec![2; 16],
                call_evidence_revision: 3,
                recording_evidence_id: recording_id.to_vec(),
                recording_revision: 4,
                consent_receipt_id: vec![6; 16],
                consent_policy_revision: 1,
                consent_scope: "call_transcription".to_owned(),
                audio_format: "wav_pcm_s16le_mono_16000".to_owned(),
                declared_bytes: 64,
                duration_millis: 1_000,
                audio_sha256: vec![7; 32],
                target_blob_reference_id: vec![8; 16],
                custody_transfer_source_proof: vec![9; 64],
                logical_owner_id: "owner-1".to_owned(),
            },
            &context(),
        )
        .expect("recording ready envelope");
        assert_eq!(record.message_id(), &event_id);
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("valid envelope");
        assert_eq!(envelope.contract.expect("contract").owner, OWNER_ID_V1);
        assert!(matches!(envelope.semantics, Some(Semantics::Event(_))));
    }
}
