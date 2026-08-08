use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyReleaseRequestV1,
    ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use makosh_tasks_command_api::{
    TASKS_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1, TASKS_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1,
    wire::ReviewedTaskCandidateContentV1,
};
use makosh_tasks_persistence::{TasksBlobCleanupV1, TasksBlobReceiptV1};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TasksBlobErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub(crate) fn transfer_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    command_message_id: [u8; 16],
    command_envelope_sha256: [u8; 32],
    source: &TasksBlobReceiptV1,
) -> Result<TasksBlobCleanupV1, TasksBlobErrorV1> {
    validate_receipt(source)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: TASKS_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source.reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source.sha256,
            custody_source_proof: &source.custody_transfer_source_proof,
            evidence_id: &command_message_id,
            evidence_envelope_sha256: &command_envelope_sha256,
        },
    )
    .map_err(classify_blob_client_error_v1)?;
    let reference_id = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(&transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(|_| TasksBlobErrorV1::Unavailable)?;
    Ok(TasksBlobCleanupV1 {
        reference_id,
        declared_bytes: source.declared_bytes,
        sha256: source.sha256,
        custody_proof: source.custody_transfer_source_proof.clone(),
    })
}

pub(crate) fn read_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    receipt: &TasksBlobCleanupV1,
) -> Result<Zeroizing<Vec<u8>>, TasksBlobErrorV1> {
    validate_cleanup(receipt)?;
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: TASKS_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id: &receipt.reference_id,
            declared_size: receipt.declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&receipt.sha256),
            custody_target: None,
        },
    )
    .map_err(classify_blob_client_error_v1)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(
                    session.grant,
                    session.channel_binding,
                    0,
                    receipt.declared_bytes,
                )
            })
            .map_err(|_| TasksBlobErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(receipt.declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != receipt.sha256
    {
        return Err(TasksBlobErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

pub(crate) fn decode_candidate_content_v1(
    bytes: &[u8],
) -> Result<ReviewedTaskCandidateContentV1, TasksBlobErrorV1> {
    let content = ReviewedTaskCandidateContentV1::decode(bytes)
        .map_err(|_| TasksBlobErrorV1::InvalidReceipt)?;
    let valid = |value: &str, limit: usize| {
        !value.trim().is_empty()
            && value.chars().count() <= limit
            && !value.chars().any(char::is_control)
    };
    if !valid(&content.title, 240)
        || content
            .due_text_hint
            .as_deref()
            .is_some_and(|value| !valid(value, 120))
        || content
            .assignee_label_hint
            .as_deref()
            .is_some_and(|value| !valid(value, 120))
    {
        return Err(TasksBlobErrorV1::InvalidReceipt);
    }
    Ok(content)
}

pub(crate) fn release_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    command_id: [u8; 16],
    cleanup: &TasksBlobCleanupV1,
    accepted: bool,
) -> Result<(), TasksBlobErrorV1> {
    let reason = if accepted {
        BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
    } else {
        BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
    };
    request_managed_blob_custody_release_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyReleaseRequestV1 {
            operation_id: &release_operation_id(command_id),
            capability_id: TASKS_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            reference_id: &cleanup.reference_id,
            declared_size: cleanup.declared_bytes,
            receipt_sha256: &cleanup.sha256,
            custody_source_proof: &cleanup.custody_proof,
            reason,
        },
    )
    .map(|_| ())
    .map_err(|_| TasksBlobErrorV1::Unavailable)
}

fn validate_receipt(receipt: &TasksBlobReceiptV1) -> Result<(), TasksBlobErrorV1> {
    if receipt.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=TASKS_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&receipt.declared_bytes)
        || receipt.sha256.iter().all(|byte| *byte == 0)
        || receipt.custody_transfer_source_proof.is_empty()
        || receipt.custody_transfer_source_proof.len()
            > makosh_tasks_command_api::TASKS_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1
    {
        return Err(TasksBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn validate_cleanup(receipt: &TasksBlobCleanupV1) -> Result<(), TasksBlobErrorV1> {
    if receipt.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=TASKS_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&receipt.declared_bytes)
        || receipt.sha256.iter().all(|byte| *byte == 0)
        || receipt.custody_proof.is_empty()
        || receipt.custody_proof.len()
            > makosh_tasks_command_api::TASKS_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1
    {
        return Err(TasksBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], TasksBlobErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(TasksBlobErrorV1::InvalidReceipt)
}

fn classify_blob_client_error_v1(error: BlobClientError) -> TasksBlobErrorV1 {
    match error {
        BlobClientError::Rejected(_) | BlobClientError::InvalidSessionRequest => {
            TasksBlobErrorV1::InvalidReceipt
        }
        _ => TasksBlobErrorV1::Unavailable,
    }
}

fn release_operation_id(command_id: [u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.tasks.reviewed-candidate.release.v1\0");
    digest.update(command_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_identity_is_command_stable() {
        assert_eq!(release_operation_id([1; 16]), release_operation_id([1; 16]));
        assert_ne!(release_operation_id([1; 16]), release_operation_id([2; 16]));
    }

    #[test]
    fn denied_custody_is_terminal_while_transport_outage_is_retryable() {
        assert_eq!(
            classify_blob_client_error_v1(BlobClientError::Rejected("expired".to_owned())),
            TasksBlobErrorV1::InvalidReceipt
        );
        assert_eq!(
            classify_blob_client_error_v1(BlobClientError::Unavailable),
            TasksBlobErrorV1::Unavailable
        );
    }
}
