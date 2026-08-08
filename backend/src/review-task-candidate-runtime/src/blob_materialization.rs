use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyReleaseRequestV1, ManagedBlobCustodyTargetV1,
    ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_review_task_candidate_api::{
    REVIEW_TASK_CANDIDATE_BLOB_CAPABILITY_ID_V1, REVIEW_TASK_CANDIDATE_MAX_BLOB_BYTES_V1,
    REVIEW_TASK_CANDIDATE_MAX_PROOF_BYTES_V1,
    TASKS_REVIEWED_CANDIDATE_BLOB_TARGET_CAPABILITY_ID_V1,
    TASKS_REVIEWED_CANDIDATE_BLOB_TARGET_MODULE_ID_V1,
    TASKS_REVIEWED_CANDIDATE_BLOB_TARGET_OWNER_ID_V1,
    wire::{ReviewTargetBoundCandidateReceiptV1, ReviewTaskCandidateContentV1},
};
use makosh_review_task_candidate_core::ReviewTaskCandidateV1;
use makosh_review_task_candidate_persistence::{
    ReviewTaskCandidateBlobCleanupV1, ReviewTaskCandidateBlobReceiptV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewTaskCandidateBlobErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub(crate) fn transfer_review_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    evidence_message_id: [u8; 16],
    evidence_envelope_sha256: [u8; 32],
    source: &ReviewTaskCandidateBlobReceiptV1,
) -> Result<ReviewTaskCandidateBlobCleanupV1, ReviewTaskCandidateBlobErrorV1> {
    validate_receipt(source)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: REVIEW_TASK_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source.reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source.sha256,
            custody_source_proof: &source.custody_transfer_source_proof,
            evidence_id: &evidence_message_id,
            evidence_envelope_sha256: &evidence_envelope_sha256,
        },
    )
    .map_err(|_| ReviewTaskCandidateBlobErrorV1::Unavailable)?;
    let local_reference_id = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(&transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(|_| ReviewTaskCandidateBlobErrorV1::Unavailable)?;
    Ok(ReviewTaskCandidateBlobCleanupV1 {
        reference_id: local_reference_id,
        declared_bytes: source.declared_bytes,
        sha256: source.sha256,
        custody_proof: source.custody_transfer_source_proof.clone(),
    })
}

pub(crate) fn read_materialized_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    cleanup: &ReviewTaskCandidateBlobCleanupV1,
) -> Result<Zeroizing<Vec<u8>>, ReviewTaskCandidateBlobErrorV1> {
    read_exact(
        channel,
        dispatcher,
        &cleanup.reference_id,
        cleanup.declared_bytes,
        &cleanup.sha256,
    )
}

pub(crate) fn decode_candidate_content_v1(
    bytes: &[u8],
) -> Result<ReviewTaskCandidateContentV1, ReviewTaskCandidateBlobErrorV1> {
    let content = ReviewTaskCandidateContentV1::decode(bytes)
        .map_err(|_| ReviewTaskCandidateBlobErrorV1::InvalidReceipt)?;
    validate_content(&content)?;
    Ok(content)
}

pub(crate) fn release_materialized_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    submission_id: [u8; 16],
    cleanup: &ReviewTaskCandidateBlobCleanupV1,
    accepted: bool,
) -> Result<(), ReviewTaskCandidateBlobErrorV1> {
    let reason = if accepted {
        BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
    } else {
        BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
    };
    request_managed_blob_custody_release_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyReleaseRequestV1 {
            operation_id: &release_operation_id(submission_id),
            capability_id: REVIEW_TASK_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            reference_id: &cleanup.reference_id,
            declared_size: cleanup.declared_bytes,
            receipt_sha256: &cleanup.sha256,
            custody_source_proof: &cleanup.custody_proof,
            reason,
        },
    )
    .map(|_| ())
    .map_err(|_| ReviewTaskCandidateBlobErrorV1::Unavailable)
}

pub(crate) fn write_tasks_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    review: &ReviewTaskCandidateV1,
) -> Result<ReviewTargetBoundCandidateReceiptV1, ReviewTaskCandidateBlobErrorV1> {
    let bytes = Zeroizing::new(
        ReviewTaskCandidateContentV1 {
            title: review.title.clone(),
            due_text_hint: review.due_text_hint.clone(),
            assignee_label_hint: review.assignee_label_hint.clone(),
        }
        .encode_to_vec(),
    );
    let declared_bytes =
        u64::try_from(bytes.len()).map_err(|_| ReviewTaskCandidateBlobErrorV1::InvalidReceipt)?;
    if !(1..=REVIEW_TASK_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&declared_bytes) {
        return Err(ReviewTaskCandidateBlobErrorV1::InvalidReceipt);
    }
    let sha256: [u8; 32] = Sha256::digest(bytes.as_slice()).into();
    let reference_id = tasks_reference_id(review, sha256);
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: REVIEW_TASK_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: TASKS_REVIEWED_CANDIDATE_BLOB_TARGET_OWNER_ID_V1,
                module_id: TASKS_REVIEWED_CANDIDATE_BLOB_TARGET_MODULE_ID_V1,
                capability_id: TASKS_REVIEWED_CANDIDATE_BLOB_TARGET_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(|_| ReviewTaskCandidateBlobErrorV1::Unavailable)?;
    let proof = session.custody_transfer_source_proof;
    if proof.is_empty() || proof.len() > REVIEW_TASK_CANDIDATE_MAX_PROOF_BYTES_V1 {
        return Err(ReviewTaskCandidateBlobErrorV1::InvalidReceipt);
    }
    if BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes.to_vec()))
        .is_err()
    {
        let existing = read_exact(channel, dispatcher, &reference_id, declared_bytes, &sha256)?;
        if existing.as_slice() != bytes.as_slice() {
            return Err(ReviewTaskCandidateBlobErrorV1::InvalidReceipt);
        }
    }
    Ok(ReviewTargetBoundCandidateReceiptV1 {
        reference_id: reference_id.to_vec(),
        declared_bytes,
        sha256: sha256.to_vec(),
        custody_transfer_source_proof: proof,
    })
}

fn read_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    reference_id: &[u8; 16],
    declared_bytes: u64,
    sha256: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, ReviewTaskCandidateBlobErrorV1> {
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: REVIEW_TASK_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(sha256),
            custody_target: None,
        },
    )
    .map_err(|_| ReviewTaskCandidateBlobErrorV1::Unavailable)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(session.grant, session.channel_binding, 0, declared_bytes)
            })
            .map_err(|_| ReviewTaskCandidateBlobErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != sha256
    {
        return Err(ReviewTaskCandidateBlobErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

fn validate_receipt(
    receipt: &ReviewTaskCandidateBlobReceiptV1,
) -> Result<(), ReviewTaskCandidateBlobErrorV1> {
    if receipt.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=REVIEW_TASK_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&receipt.declared_bytes)
        || receipt.sha256.iter().all(|byte| *byte == 0)
        || receipt.custody_transfer_source_proof.is_empty()
        || receipt.custody_transfer_source_proof.len() > REVIEW_TASK_CANDIDATE_MAX_PROOF_BYTES_V1
    {
        return Err(ReviewTaskCandidateBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn validate_content(
    content: &ReviewTaskCandidateContentV1,
) -> Result<(), ReviewTaskCandidateBlobErrorV1> {
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
        return Err(ReviewTaskCandidateBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn tasks_reference_id(review: &ReviewTaskCandidateV1, sha256: [u8; 32]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.review.task-candidate.tasks-copy.v1\0");
    digest.update(review.review_id);
    digest.update(review.review_revision.saturating_add(1).to_be_bytes());
    digest.update(review.candidate_id);
    digest.update(sha256);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn release_operation_id(submission_id: [u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.review.task-candidate.release-source.v1\0");
    digest.update(submission_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReviewTaskCandidateBlobErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewTaskCandidateBlobErrorV1::InvalidReceipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_review_task_candidate_core::{
        ReviewTaskCandidatePromotionStatusV1, ReviewTaskCandidateStateV1,
        ReviewTaskCandidateTimestampV1,
    };

    fn pending_review() -> ReviewTaskCandidateV1 {
        ReviewTaskCandidateV1 {
            review_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            candidate_id: [2; 16],
            candidate_digest: [3; 32],
            source_evidence_id: [4; 16],
            source_evidence_revision: 1,
            title: "Подготовить ответ".to_owned(),
            due_text_hint: None,
            assignee_label_hint: None,
            state: ReviewTaskCandidateStateV1::Pending,
            promotion_status: ReviewTaskCandidatePromotionStatusV1::NotRequested,
            review_revision: 1,
            decided_by_owner_device_id: None,
            decided_at: None,
            promoted_task_id: None,
            updated_at: ReviewTaskCandidateTimestampV1 {
                unix_seconds: 1_800_000_000,
                nanos: 0,
            },
        }
    }

    #[test]
    fn tasks_blob_identity_is_revision_and_content_bound() {
        let review = pending_review();
        assert_eq!(
            tasks_reference_id(&review, [5; 32]),
            tasks_reference_id(&review, [5; 32])
        );
        assert_ne!(
            tasks_reference_id(&review, [5; 32]),
            tasks_reference_id(&review, [6; 32])
        );
    }
}
