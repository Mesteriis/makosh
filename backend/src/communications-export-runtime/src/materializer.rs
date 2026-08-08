//! Cross-owner Blob custody transfer and deterministic artifact materialization.

use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyTransferRequestV1,
    ManagedBlobSessionRequestV1, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_communications_export_core::{
    EvidenceExportBodyV1, EvidenceExportItemV1, EvidenceExportManifestV1,
    MAX_EXPORT_ARTIFACT_BYTES_V1, encode_evidence_export_jsonl_v1,
};
use makosh_communications_export_persistence::{
    CommunicationsExportArtifactReceiptV1, CommunicationsExportClaimV1,
    CommunicationsExportPersistenceErrorV1, CommunicationsExportPersistenceV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::BlobDataOperationV1,
};
use sha2::{Digest, Sha256};

use crate::admission::COMMUNICATIONS_EXPORT_BLOB_CAPABILITY_ID_V1;

const MATERIALIZATION_LEASE_SECONDS_V1: i64 = 60;
const POLICY_REJECTION_CODE_V1: u16 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsExportMaterializerErrorV1 {
    RetryPending,
    StorageUnavailable,
}

pub async fn process_next_communications_export_v1(
    persistence: &CommunicationsExportPersistenceV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    worker_id: &str,
    now_unix_seconds: i64,
) -> Result<bool, CommunicationsExportMaterializerErrorV1> {
    let claim = persistence
        .claim_next_materialization(
            worker_id,
            now_unix_seconds,
            now_unix_seconds
                .checked_add(MATERIALIZATION_LEASE_SECONDS_V1)
                .ok_or(CommunicationsExportMaterializerErrorV1::StorageUnavailable)?,
        )
        .await
        .map_err(persistence_error)?;
    let Some(claim) = claim else {
        return Ok(false);
    };
    match materialize(control_channel, dispatcher, &claim) {
        Ok(artifact) => persistence
            .complete_materialization(&claim, artifact, now_unix_seconds)
            .await
            .map(|_| true)
            .map_err(persistence_error),
        Err(MaterializationFailureV1::Policy) => persistence
            .reject_materialization(&claim, POLICY_REJECTION_CODE_V1, now_unix_seconds)
            .await
            .map(|_| true)
            .map_err(persistence_error),
        Err(MaterializationFailureV1::Retry) => {
            persistence
                .release_materialization_claim(&claim, now_unix_seconds)
                .await
                .map_err(persistence_error)?;
            Err(CommunicationsExportMaterializerErrorV1::RetryPending)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MaterializationFailureV1 {
    Policy,
    Retry,
}

fn materialize(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    claim: &CommunicationsExportClaimV1,
) -> Result<CommunicationsExportArtifactReceiptV1, MaterializationFailureV1> {
    let mut items = Vec::with_capacity(claim.items.len());
    for item in &claim.items {
        let body = match &item.body_source {
            Some(source) => {
                let transfer = request_managed_blob_custody_transfer_v2(
                    control_channel,
                    dispatcher,
                    ManagedBlobCustodyTransferRequestV1 {
                        capability_id: COMMUNICATIONS_EXPORT_BLOB_CAPABILITY_ID_V1,
                        source_reference_id: &source.reference_id,
                        declared_size: source.declared_bytes,
                        receipt_sha256: &source.sha256,
                        custody_source_proof: &source.custody_transfer_source_proof,
                        evidence_id: &claim.source_result_message_id,
                        evidence_envelope_sha256: &claim.source_result_envelope_sha256,
                    },
                )
                .map_err(blob_failure)?;
                let target_reference_id: [u8; 16] = transfer
                    .grant
                    .target_reference_id
                    .as_slice()
                    .try_into()
                    .ok()
                    .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
                    .ok_or(MaterializationFailureV1::Policy)?;
                BlobDataClient::new(transfer.data_socket_path)
                    .and_then(|client| {
                        client.custody_transfer(transfer.grant, transfer.channel_binding)
                    })
                    .map_err(blob_failure)?;
                let bytes = read_target(
                    control_channel,
                    dispatcher,
                    target_reference_id,
                    source.declared_bytes,
                    source.sha256,
                )?;
                EvidenceExportBodyV1::AdmittedUtf8(bytes)
            }
            None => EvidenceExportBodyV1::Unavailable,
        };
        items.push(EvidenceExportItemV1 {
            message_id: item.message_id,
            conversation_id: item.conversation_id,
            evidence_id: item.evidence_id,
            evidence_revision: item.evidence_revision,
            direction: item.direction,
            occurred_at_unix_seconds: item.occurred_at_unix_seconds,
            observed_at_unix_seconds: item.observed_at_unix_seconds,
            participant_display_label: item.participant_display_label.clone(),
            body,
        });
    }
    let bytes = encode_evidence_export_jsonl_v1(
        EvidenceExportManifestV1 {
            export_id: claim.export_id,
            logical_owner_id: claim.logical_owner_id.clone(),
            created_at_unix_seconds: claim.created_at_unix_seconds,
        },
        &items,
    )
    .map_err(|_| MaterializationFailureV1::Policy)?;
    write_artifact(control_channel, dispatcher, claim.export_id, bytes)
}

fn read_target(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    reference_id: [u8; 16],
    declared_bytes: u64,
    expected_sha256: [u8; 32],
) -> Result<Vec<u8>, MaterializationFailureV1> {
    let session = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATIONS_EXPORT_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&expected_sha256),
            custody_target: None,
        },
    )
    .map_err(blob_failure)?;
    let bytes = BlobDataClient::new(session.data_socket_path)
        .and_then(|client| {
            client.read_range(session.grant, session.channel_binding, 0, declared_bytes)
        })
        .map_err(blob_failure)?;
    if bytes.len() != usize::try_from(declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(&bytes).as_slice() != expected_sha256
        || std::str::from_utf8(&bytes).is_err()
    {
        return Err(MaterializationFailureV1::Policy);
    }
    Ok(bytes)
}

fn write_artifact(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    export_id: [u8; 16],
    bytes: Vec<u8>,
) -> Result<CommunicationsExportArtifactReceiptV1, MaterializationFailureV1> {
    let declared_bytes =
        u64::try_from(bytes.len()).map_err(|_| MaterializationFailureV1::Policy)?;
    if declared_bytes == 0 || declared_bytes > MAX_EXPORT_ARTIFACT_BYTES_V1 as u64 {
        return Err(MaterializationFailureV1::Policy);
    }
    let sha256: [u8; 32] = Sha256::digest(&bytes).into();
    let reference_id = artifact_reference_id(export_id, sha256);
    let session = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATIONS_EXPORT_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: None,
        },
    )
    .map_err(blob_failure)?;
    BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes))
        .map_err(blob_failure)?;
    Ok(CommunicationsExportArtifactReceiptV1 {
        reference_id,
        declared_bytes,
        sha256,
    })
}

fn blob_failure(error: BlobClientError) -> MaterializationFailureV1 {
    match error {
        BlobClientError::Unavailable
        | BlobClientError::Connect(_)
        | BlobClientError::Io(_)
        | BlobClientError::InvalidTimeout => MaterializationFailureV1::Retry,
        BlobClientError::InvalidSocketPath
        | BlobClientError::FrameTooLarge
        | BlobClientError::InvalidFrame
        | BlobClientError::InvalidResponse
        | BlobClientError::Rejected(_)
        | BlobClientError::InvalidCustodyDelegationRequest
        | BlobClientError::InvalidCustodyReleaseRequest
        | BlobClientError::InvalidSessionRequest => MaterializationFailureV1::Policy,
    }
}

fn artifact_reference_id(export_id: [u8; 16], sha256: [u8; 32]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-export-artifact-v1");
    hasher.update(export_id);
    hasher.update(sha256);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn persistence_error(
    _: CommunicationsExportPersistenceErrorV1,
) -> CommunicationsExportMaterializerErrorV1 {
    CommunicationsExportMaterializerErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_reference_is_deterministic_and_export_bound() {
        let first = artifact_reference_id([1; 16], [2; 32]);
        assert_eq!(first, artifact_reference_id([1; 16], [2; 32]));
        assert_ne!(first, artifact_reference_id([3; 16], [2; 32]));
        assert!(first.iter().any(|byte| *byte != 0));
    }
}
