use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyReleaseRequestV1,
    ManagedBlobCustodyTargetV1, ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_call_transcription_api::{MODULE_ID_V1, OWNER_ID_V1};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use makosh_speech_to_text_api::{
    SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1, SPEECH_TO_TEXT_MODULE_ID_V1, SPEECH_TO_TEXT_OWNER_V1,
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::admission::BLOB_CAPABILITY_ID_V1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingCustodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub receipt_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TranscriptCustodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub receipt_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionBlobErrorV1 {
    InvalidReceipt,
    Rejected,
    Unavailable,
}

#[allow(clippy::too_many_arguments)]
pub fn accept_recording_custody_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source_reference_id: [u8; 16],
    declared_bytes: u64,
    receipt_sha256: [u8; 32],
    custody_source_proof: &[u8],
    event_id: [u8; 16],
    envelope_sha256: [u8; 32],
) -> Result<RecordingCustodyReceiptV1, CallTranscriptionBlobErrorV1> {
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source_reference_id,
            declared_size: declared_bytes,
            receipt_sha256: &receipt_sha256,
            custody_source_proof,
            evidence_id: &event_id,
            evidence_envelope_sha256: &envelope_sha256,
        },
    )
    .map_err(classify)?;
    let accepted_reference_id = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(classify)?;

    let bytes = read_exact(
        channel,
        dispatcher,
        &accepted_reference_id,
        declared_bytes,
        &receipt_sha256,
    )?;
    let replay_reference_id = replayable_source_reference_id(
        accepted_reference_id,
        event_id,
        envelope_sha256,
        receipt_sha256,
    );
    let replay_proof = write_replayable_source(
        channel,
        dispatcher,
        replay_reference_id,
        declared_bytes,
        receipt_sha256,
        bytes,
        ManagedBlobCustodyTargetV1 {
            owner_id: SPEECH_TO_TEXT_OWNER_V1,
            module_id: SPEECH_TO_TEXT_MODULE_ID_V1,
            capability_id: SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1,
        },
    )?;
    release(
        channel,
        dispatcher,
        release_operation_id(event_id, receipt_sha256),
        accepted_reference_id,
        declared_bytes,
        receipt_sha256,
        custody_source_proof,
        true,
    )?;
    Ok(RecordingCustodyReceiptV1 {
        reference_id: replay_reference_id,
        declared_bytes,
        receipt_sha256,
        custody_transfer_source_proof: replay_proof,
    })
}

pub fn fresh_stt_source_proof_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source: &RecordingCustodyReceiptV1,
) -> Result<Vec<u8>, CallTranscriptionBlobErrorV1> {
    let bytes = read_exact(
        channel,
        dispatcher,
        &source.reference_id,
        source.declared_bytes,
        &source.receipt_sha256,
    )?;
    write_replayable_source(
        channel,
        dispatcher,
        source.reference_id,
        source.declared_bytes,
        source.receipt_sha256,
        bytes,
        ManagedBlobCustodyTargetV1 {
            owner_id: SPEECH_TO_TEXT_OWNER_V1,
            module_id: SPEECH_TO_TEXT_MODULE_ID_V1,
            capability_id: SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1,
        },
    )
}

pub fn fresh_source_cleanup_proof_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source: &RecordingCustodyReceiptV1,
) -> Result<Vec<u8>, CallTranscriptionBlobErrorV1> {
    let bytes = read_exact(
        channel,
        dispatcher,
        &source.reference_id,
        source.declared_bytes,
        &source.receipt_sha256,
    )?;
    write_replayable_source(
        channel,
        dispatcher,
        source.reference_id,
        source.declared_bytes,
        source.receipt_sha256,
        bytes,
        ManagedBlobCustodyTargetV1 {
            owner_id: OWNER_ID_V1,
            module_id: MODULE_ID_V1,
            capability_id: BLOB_CAPABILITY_ID_V1,
        },
    )
}

pub fn verify_transcript_custody_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    transcript: &TranscriptCustodyReceiptV1,
) -> Result<(), CallTranscriptionBlobErrorV1> {
    request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id: &transcript.reference_id,
            declared_size: transcript.declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&transcript.receipt_sha256),
            custody_target: None,
        },
    )
    .map_err(classify)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn accept_transcript_custody_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source_reference_id: [u8; 16],
    declared_bytes: u64,
    receipt_sha256: [u8; 32],
    custody_source_proof: &[u8],
    request_id: [u8; 16],
    result_receipt_sha256: [u8; 32],
) -> Result<TranscriptCustodyReceiptV1, CallTranscriptionBlobErrorV1> {
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source_reference_id,
            declared_size: declared_bytes,
            receipt_sha256: &receipt_sha256,
            custody_source_proof,
            evidence_id: &request_id,
            evidence_envelope_sha256: &result_receipt_sha256,
        },
    )
    .map_err(classify)?;
    let target_reference_id = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(classify)?;
    Ok(TranscriptCustodyReceiptV1 {
        reference_id: target_reference_id,
        declared_bytes,
        receipt_sha256,
    })
}

pub fn release_recording_custody_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    source: &RecordingCustodyReceiptV1,
    fresh_source_proof: &[u8],
    accepted: bool,
) -> Result<(), CallTranscriptionBlobErrorV1> {
    release(
        channel,
        dispatcher,
        release_operation_id(run_id, source.receipt_sha256),
        source.reference_id,
        source.declared_bytes,
        source.receipt_sha256,
        fresh_source_proof,
        accepted,
    )
}

#[allow(clippy::too_many_arguments)]
fn release(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    operation_id: [u8; 16],
    reference_id: [u8; 16],
    declared_bytes: u64,
    receipt_sha256: [u8; 32],
    custody_source_proof: &[u8],
    accepted: bool,
) -> Result<(), CallTranscriptionBlobErrorV1> {
    request_managed_blob_custody_release_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyReleaseRequestV1 {
            operation_id: &operation_id,
            capability_id: BLOB_CAPABILITY_ID_V1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            receipt_sha256: &receipt_sha256,
            custody_source_proof,
            reason: if accepted {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
            } else {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
            },
        },
    )
    .map_err(classify)?;
    Ok(())
}

fn read_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    reference_id: &[u8; 16],
    declared_bytes: u64,
    receipt_sha256: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, CallTranscriptionBlobErrorV1> {
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(receipt_sha256),
            custody_target: None,
        },
    )
    .map_err(classify)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(session.grant, session.channel_binding, 0, declared_bytes)
            })
            .map_err(classify)?,
    );
    if bytes.len() as u64 != declared_bytes
        || Sha256::digest(bytes.as_slice()).as_slice() != receipt_sha256
    {
        return Err(CallTranscriptionBlobErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

fn write_replayable_source(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    reference_id: [u8; 16],
    declared_bytes: u64,
    receipt_sha256: [u8; 32],
    bytes: Zeroizing<Vec<u8>>,
    custody_target: ManagedBlobCustodyTargetV1<'_>,
) -> Result<Vec<u8>, CallTranscriptionBlobErrorV1> {
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&receipt_sha256),
            custody_target: Some(custody_target),
        },
    )
    .map_err(classify)?;
    if session.custody_transfer_source_proof.is_empty()
        || session.custody_transfer_source_proof.len() > 2_048
    {
        return Err(CallTranscriptionBlobErrorV1::InvalidReceipt);
    }
    let proof = session.custody_transfer_source_proof;
    if BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes.to_vec()))
        .is_err()
    {
        let existing = read_exact(
            channel,
            dispatcher,
            &reference_id,
            declared_bytes,
            &receipt_sha256,
        )?;
        if existing.as_slice() != bytes.as_slice() {
            return Err(CallTranscriptionBlobErrorV1::InvalidReceipt);
        }
    }
    Ok(proof)
}

fn replayable_source_reference_id(
    accepted_reference_id: [u8; 16],
    event_id: [u8; 16],
    envelope_sha256: [u8; 32],
    receipt_sha256: [u8; 32],
) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.call-transcription.replayable-stt-source.v1\0");
    digest.update(accepted_reference_id);
    digest.update(event_id);
    digest.update(envelope_sha256);
    digest.update(receipt_sha256);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn release_operation_id(run_id: [u8; 16], receipt_sha256: [u8; 32]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.call-transcription.release-recording.v1\0");
    digest.update(run_id);
    digest.update(receipt_sha256);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn id16(value: &[u8]) -> Result<[u8; 16], CallTranscriptionBlobErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(CallTranscriptionBlobErrorV1::InvalidReceipt)
}

fn classify(error: BlobClientError) -> CallTranscriptionBlobErrorV1 {
    match error {
        BlobClientError::Unavailable => CallTranscriptionBlobErrorV1::Unavailable,
        BlobClientError::Rejected(_) => CallTranscriptionBlobErrorV1::Rejected,
        _ => CallTranscriptionBlobErrorV1::InvalidReceipt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_identity_is_content_and_run_bound() {
        assert_eq!(
            release_operation_id([1; 16], [2; 32]),
            release_operation_id([1; 16], [2; 32])
        );
        assert_ne!(
            release_operation_id([1; 16], [2; 32]),
            release_operation_id([1; 16], [3; 32])
        );
    }
}
