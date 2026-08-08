use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use makosh_call_transcription_ingress::{
    OWNER_ID_V1 as TARGET_OWNER_ID_V1, TARGET_BLOB_CAPABILITY_ID_V1, TARGET_MODULE_ID_V1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::BlobDataOperationV1,
};
use sha2::{Digest, Sha256};

use crate::admission::BLOB_CAPABILITY_ID_V1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordingBlobErrorV1 {
    InvalidAudio,
    Unavailable,
}

pub fn write_recording_blob_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    recording_evidence_id: [u8; 16],
    bytes: Vec<u8>,
    expected_sha256: [u8; 32],
) -> Result<RecordingBlobReceiptV1, RecordingBlobErrorV1> {
    let declared_bytes =
        u64::try_from(bytes.len()).map_err(|_| RecordingBlobErrorV1::InvalidAudio)?;
    if declared_bytes == 0 || declared_bytes > 64 * 1024 * 1024 {
        return Err(RecordingBlobErrorV1::InvalidAudio);
    }
    let sha256: [u8; 32] = Sha256::digest(&bytes).into();
    if sha256 != expected_sha256 {
        return Err(RecordingBlobErrorV1::InvalidAudio);
    }
    let reference_id = source_reference_id(recording_evidence_id, sha256);
    let session = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: TARGET_OWNER_ID_V1,
                module_id: TARGET_MODULE_ID_V1,
                capability_id: TARGET_BLOB_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(|_| RecordingBlobErrorV1::Unavailable)?;
    if session.custody_transfer_source_proof.is_empty()
        || session.custody_transfer_source_proof.len() > 2_048
    {
        return Err(RecordingBlobErrorV1::InvalidAudio);
    }
    BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes))
        .map_err(|_| RecordingBlobErrorV1::Unavailable)?;
    Ok(RecordingBlobReceiptV1 {
        reference_id,
        declared_bytes,
        sha256,
        custody_transfer_source_proof: session.custody_transfer_source_proof,
    })
}

fn source_reference_id(recording_evidence_id: [u8; 16], sha256: [u8; 32]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.desktop-call-recording.source-copy.v1\0");
    hash.update(recording_evidence_id);
    hash.update(sha256);
    hash.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_reference_is_content_bound() {
        assert_ne!(
            source_reference_id([1; 16], [2; 32]),
            source_reference_id([1; 16], [3; 32])
        );
    }
}
