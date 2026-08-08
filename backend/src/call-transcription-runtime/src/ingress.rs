use std::os::unix::net::UnixStream;

use makosh_call_transcription_api::run_id_v1;
use makosh_call_transcription_core::{CallTranscriptionRejectionV1, RecordingSourceV1};
use makosh_call_transcription_ingress::{
    RECORDING_READY_CONTRACT_NAME_V1, RECORDING_REJECTED_CONTRACT_NAME_V1, contract_reference_v1,
    recording_ready_event_id_v1, recording_rejected_event_id_v1,
    wire::{RecordingReadyV1, RecordingRejectedV1},
};
use makosh_call_transcription_persistence::{
    CallTranscriptionInboxOutcomeV1, CallTranscriptionPersistenceErrorV1,
    CallTranscriptionPersistenceV1, PersistRecordingIngressV1, RecordingIngressOutcomeV1,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{DurableEnvelopeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::ContractReferenceV1,
};
use prost::Message;

use crate::{
    blob::{CallTranscriptionBlobErrorV1, accept_recording_custody_v1},
    stt::{build_stt_request_v1, stt_request_id_v1},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionIngressErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Blob(CallTranscriptionBlobErrorV1),
    Persistence(CallTranscriptionPersistenceErrorV1),
}

pub async fn apply_recording_ready_v1(
    persistence: &CallTranscriptionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
    occurred_at_unix_millis: i64,
) -> Result<CallTranscriptionInboxOutcomeV1, CallTranscriptionIngressErrorV1> {
    let envelope = decode_exact_event(record, RECORDING_READY_CONTRACT_NAME_V1)?;
    let payload = RecordingReadyV1::decode(envelope.payload.as_slice())
        .map_err(|_| CallTranscriptionIngressErrorV1::InvalidPayload)?;
    let ready = validate_ready(payload, record, expected_logical_owner_id)?;
    let run_id = run_id_v1(ready.request_id);
    let run = persistence
        .load_run(expected_logical_owner_id, run_id)
        .await
        .map_err(CallTranscriptionIngressErrorV1::Persistence)?;
    let custody = accept_recording_custody_v1(
        channel,
        dispatcher,
        ready.source_reference_id,
        ready.declared_bytes,
        ready.audio_sha256,
        &ready.custody_source_proof,
        *record.message_id(),
        *record.envelope_sha256(),
    )
    .map_err(CallTranscriptionIngressErrorV1::Blob)?;
    let source = RecordingSourceV1 {
        recording_evidence_id: ready.recording_evidence_id,
        recording_revision: ready.recording_revision,
        call_evidence_id: ready.call_evidence_id,
        call_evidence_revision: ready.call_evidence_revision,
        consent_receipt_id: ready.consent_receipt_id,
        consent_policy_revision: ready.consent_policy_revision,
        audio_reference_id: custody.reference_id,
        audio_sha256: custody.receipt_sha256,
        declared_bytes: custody.declared_bytes,
        duration_millis: ready.duration_millis,
    };
    let stt_request_id = stt_request_id_v1(run_id, source.audio_sha256);
    let request = build_stt_request_v1(
        expected_logical_owner_id,
        stt_request_id,
        &source,
        run.draft.requested_language,
        &custody.custody_transfer_source_proof,
    )
    .map_err(|_| CallTranscriptionIngressErrorV1::InvalidPayload)?;
    let stt_request_digest = id32(&request.request_digest)?;
    persistence
        .persist_recording_ingress(PersistRecordingIngressV1 {
            logical_owner_id: expected_logical_owner_id.to_owned(),
            run_id,
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            outcome: RecordingIngressOutcomeV1::Ready {
                source: Box::new(source),
                source_receipt_sha256: custody.receipt_sha256,
                stt_request_id,
                stt_request_digest,
            },
            outbox: None,
            occurred_at_unix_millis,
        })
        .await
        .map_err(CallTranscriptionIngressErrorV1::Persistence)
}

pub async fn apply_recording_rejected_v1(
    persistence: &CallTranscriptionPersistenceV1,
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
    occurred_at_unix_millis: i64,
) -> Result<CallTranscriptionInboxOutcomeV1, CallTranscriptionIngressErrorV1> {
    let envelope = decode_exact_event(record, RECORDING_REJECTED_CONTRACT_NAME_V1)?;
    let payload = RecordingRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CallTranscriptionIngressErrorV1::InvalidPayload)?;
    let rejected = validate_rejected(payload, record, expected_logical_owner_id)?;
    persistence
        .persist_recording_ingress(PersistRecordingIngressV1 {
            logical_owner_id: expected_logical_owner_id.to_owned(),
            run_id: run_id_v1(rejected.request_id),
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            outcome: RecordingIngressOutcomeV1::Rejected(
                CallTranscriptionRejectionV1::RecordingRejected,
            ),
            outbox: None,
            occurred_at_unix_millis,
        })
        .await
        .map_err(CallTranscriptionIngressErrorV1::Persistence)
}

struct ReadyV1 {
    request_id: [u8; 16],
    call_evidence_id: [u8; 16],
    call_evidence_revision: u64,
    recording_evidence_id: [u8; 16],
    recording_revision: u64,
    consent_receipt_id: [u8; 16],
    consent_policy_revision: u32,
    source_reference_id: [u8; 16],
    declared_bytes: u64,
    duration_millis: u64,
    audio_sha256: [u8; 32],
    custody_source_proof: Vec<u8>,
}

struct RejectedV1 {
    request_id: [u8; 16],
}

fn validate_ready(
    payload: RecordingReadyV1,
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<ReadyV1, CallTranscriptionIngressErrorV1> {
    let recording_evidence_id = id16(&payload.recording_evidence_id)?;
    let event_id = id16(&payload.event_id)?;
    if event_id != *record.message_id()
        || event_id
            != recording_ready_event_id_v1(recording_evidence_id, payload.recording_revision)
        || payload.logical_owner_id != expected_logical_owner_id
        || payload.consent_scope != "call_transcription"
        || payload.audio_format != "wav_pcm_s16le_mono_16000"
        || payload.call_evidence_revision == 0
        || payload.recording_revision == 0
        || payload.consent_policy_revision == 0
        || payload.declared_bytes == 0
        || payload.duration_millis == 0
        || payload.custody_transfer_source_proof.is_empty()
        || payload.custody_transfer_source_proof.len() > 2_048
    {
        return Err(CallTranscriptionIngressErrorV1::InvalidPayload);
    }
    Ok(ReadyV1 {
        request_id: id16(&payload.request_id)?,
        call_evidence_id: id16(&payload.call_evidence_id)?,
        call_evidence_revision: payload.call_evidence_revision,
        recording_evidence_id,
        recording_revision: payload.recording_revision,
        consent_receipt_id: id16(&payload.consent_receipt_id)?,
        consent_policy_revision: payload.consent_policy_revision,
        source_reference_id: id16(&payload.target_blob_reference_id)?,
        declared_bytes: payload.declared_bytes,
        duration_millis: payload.duration_millis,
        audio_sha256: id32(&payload.audio_sha256)?,
        custody_source_proof: payload.custody_transfer_source_proof,
    })
}

fn validate_rejected(
    payload: RecordingRejectedV1,
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<RejectedV1, CallTranscriptionIngressErrorV1> {
    let recording_evidence_id = id16(&payload.recording_evidence_id)?;
    let event_id = id16(&payload.event_id)?;
    if event_id != *record.message_id()
        || event_id
            != recording_rejected_event_id_v1(recording_evidence_id, payload.recording_revision)
        || payload.logical_owner_id != expected_logical_owner_id
        || payload.call_evidence_revision == 0
        || payload.recording_revision == 0
        || payload.rejection_code.is_empty()
        || payload.rejection_code.len() > 128
    {
        return Err(CallTranscriptionIngressErrorV1::InvalidPayload);
    }
    id16(&payload.call_evidence_id)?;
    Ok(RejectedV1 {
        request_id: id16(&payload.request_id)?,
    })
}

fn decode_exact_event(
    record: &OutboxRecordV1,
    name: &str,
) -> Result<DurableEnvelopeV1, CallTranscriptionIngressErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CallTranscriptionIngressErrorV1::InvalidEnvelope)?;
    if !exact_contract(envelope.contract.as_ref(), &contract_reference_v1(name))
        || !matches!(envelope.semantics, Some(Semantics::Event(_)))
        || envelope
            .source
            .as_ref()
            .is_none_or(|source| source.module_id.is_empty() || source.runtime_generation == 0)
    {
        return Err(CallTranscriptionIngressErrorV1::InvalidEnvelope);
    }
    Ok(envelope)
}

fn exact_contract(
    actual: Option<&makosh_events_protocol::v1::ContractRefV1>,
    expected: &ContractReferenceV1,
) -> bool {
    actual.is_some_and(|actual| {
        actual.owner == expected.owner
            && actual.name == expected.name
            && actual.major == expected.major
            && actual.revision == expected.revision
            && actual.schema_sha256 == expected.schema_sha256
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], CallTranscriptionIngressErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(CallTranscriptionIngressErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CallTranscriptionIngressErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(CallTranscriptionIngressErrorV1::InvalidPayload)
}
