use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyReleaseRequestV1, ManagedBlobCustodyTargetV1,
    ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_review_obligation_candidate_api::{
    OBLIGATIONS_REVIEWED_CANDIDATE_BLOB_TARGET_CAPABILITY_ID_V1,
    OBLIGATIONS_REVIEWED_CANDIDATE_BLOB_TARGET_MODULE_ID_V1,
    OBLIGATIONS_REVIEWED_CANDIDATE_BLOB_TARGET_OWNER_ID_V1,
    REVIEW_OBLIGATION_CANDIDATE_BLOB_CAPABILITY_ID_V1,
    REVIEW_OBLIGATION_CANDIDATE_MAX_BLOB_BYTES_V1, REVIEW_OBLIGATION_CANDIDATE_MAX_PROOF_BYTES_V1,
    wire::{
        ReviewObligationCandidateContentV1, ReviewObligationEvidenceLinkV1 as WireEvidenceLink,
        ReviewTargetBoundCandidateReceiptV1, TimestampV1 as WireTimestamp,
    },
};
use makosh_review_obligation_candidate_core::ReviewObligationCandidateV1;
use makosh_review_obligation_candidate_persistence::{
    ReviewObligationCandidateBlobCleanupV1, ReviewObligationCandidateBlobReceiptV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewObligationCandidateBlobErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub(crate) fn transfer_review_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    evidence_message_id: [u8; 16],
    evidence_envelope_sha256: [u8; 32],
    source: &ReviewObligationCandidateBlobReceiptV1,
) -> Result<ReviewObligationCandidateBlobCleanupV1, ReviewObligationCandidateBlobErrorV1> {
    validate_receipt(source)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: REVIEW_OBLIGATION_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source.reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source.sha256,
            custody_source_proof: &source.custody_transfer_source_proof,
            evidence_id: &evidence_message_id,
            evidence_envelope_sha256: &evidence_envelope_sha256,
        },
    )
    .map_err(|_| ReviewObligationCandidateBlobErrorV1::Unavailable)?;
    let local_reference_id = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(&transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(|_| ReviewObligationCandidateBlobErrorV1::Unavailable)?;
    Ok(ReviewObligationCandidateBlobCleanupV1 {
        reference_id: local_reference_id,
        declared_bytes: source.declared_bytes,
        sha256: source.sha256,
        custody_proof: source.custody_transfer_source_proof.clone(),
    })
}

pub(crate) fn read_materialized_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    cleanup: &ReviewObligationCandidateBlobCleanupV1,
) -> Result<Zeroizing<Vec<u8>>, ReviewObligationCandidateBlobErrorV1> {
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
) -> Result<ReviewObligationCandidateContentV1, ReviewObligationCandidateBlobErrorV1> {
    let content = ReviewObligationCandidateContentV1::decode(bytes)
        .map_err(|_| ReviewObligationCandidateBlobErrorV1::InvalidReceipt)?;
    validate_content(&content)?;
    Ok(content)
}

pub(crate) fn release_materialized_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    submission_id: [u8; 16],
    cleanup: &ReviewObligationCandidateBlobCleanupV1,
    accepted: bool,
) -> Result<(), ReviewObligationCandidateBlobErrorV1> {
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
            capability_id: REVIEW_OBLIGATION_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            reference_id: &cleanup.reference_id,
            declared_size: cleanup.declared_bytes,
            receipt_sha256: &cleanup.sha256,
            custody_source_proof: &cleanup.custody_proof,
            reason,
        },
    )
    .map(|_| ())
    .map_err(|_| ReviewObligationCandidateBlobErrorV1::Unavailable)
}

pub(crate) fn write_obligations_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    review: &ReviewObligationCandidateV1,
) -> Result<ReviewTargetBoundCandidateReceiptV1, ReviewObligationCandidateBlobErrorV1> {
    let bytes = Zeroizing::new(
        ReviewObligationCandidateContentV1 {
            statement: review.statement.clone(),
            due_at: review.due_at.map(|value| WireTimestamp {
                unix_seconds: value.unix_seconds,
                nanos: value.nanos,
            }),
            condition: review.condition.clone(),
            obligated_party_id: review.obligated_party_id.to_vec(),
            beneficiary_party_id: review.beneficiary_party_id.map(|value| value.to_vec()),
            evidence_links: review
                .evidence_links
                .iter()
                .map(|value| WireEvidenceLink {
                    evidence_link_id: value.evidence_link_id.to_vec(),
                    evidence_owner_id: value.evidence_owner_id.clone(),
                    evidence_record_id: value.evidence_record_id.to_vec(),
                    evidence_revision: value.evidence_revision,
                    evidence_digest: value.evidence_digest.to_vec(),
                })
                .collect(),
        }
        .encode_to_vec(),
    );
    let declared_bytes = u64::try_from(bytes.len())
        .map_err(|_| ReviewObligationCandidateBlobErrorV1::InvalidReceipt)?;
    if !(1..=REVIEW_OBLIGATION_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&declared_bytes) {
        return Err(ReviewObligationCandidateBlobErrorV1::InvalidReceipt);
    }
    let sha256: [u8; 32] = Sha256::digest(bytes.as_slice()).into();
    let reference_id = obligations_reference_id(review, sha256);
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: REVIEW_OBLIGATION_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: OBLIGATIONS_REVIEWED_CANDIDATE_BLOB_TARGET_OWNER_ID_V1,
                module_id: OBLIGATIONS_REVIEWED_CANDIDATE_BLOB_TARGET_MODULE_ID_V1,
                capability_id: OBLIGATIONS_REVIEWED_CANDIDATE_BLOB_TARGET_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(|_| ReviewObligationCandidateBlobErrorV1::Unavailable)?;
    let proof = session.custody_transfer_source_proof;
    if proof.is_empty() || proof.len() > REVIEW_OBLIGATION_CANDIDATE_MAX_PROOF_BYTES_V1 {
        return Err(ReviewObligationCandidateBlobErrorV1::InvalidReceipt);
    }
    if BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes.to_vec()))
        .is_err()
    {
        let existing = read_exact(channel, dispatcher, &reference_id, declared_bytes, &sha256)?;
        if existing.as_slice() != bytes.as_slice() {
            return Err(ReviewObligationCandidateBlobErrorV1::InvalidReceipt);
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
) -> Result<Zeroizing<Vec<u8>>, ReviewObligationCandidateBlobErrorV1> {
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: REVIEW_OBLIGATION_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(sha256),
            custody_target: None,
        },
    )
    .map_err(|_| ReviewObligationCandidateBlobErrorV1::Unavailable)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(session.grant, session.channel_binding, 0, declared_bytes)
            })
            .map_err(|_| ReviewObligationCandidateBlobErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != sha256
    {
        return Err(ReviewObligationCandidateBlobErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

fn validate_receipt(
    receipt: &ReviewObligationCandidateBlobReceiptV1,
) -> Result<(), ReviewObligationCandidateBlobErrorV1> {
    if receipt.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=REVIEW_OBLIGATION_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&receipt.declared_bytes)
        || receipt.sha256.iter().all(|byte| *byte == 0)
        || receipt.custody_transfer_source_proof.is_empty()
        || receipt.custody_transfer_source_proof.len()
            > REVIEW_OBLIGATION_CANDIDATE_MAX_PROOF_BYTES_V1
    {
        return Err(ReviewObligationCandidateBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn validate_content(
    content: &ReviewObligationCandidateContentV1,
) -> Result<(), ReviewObligationCandidateBlobErrorV1> {
    let valid = |value: &str, limit: usize| {
        !value.trim().is_empty()
            && value.chars().count() <= limit
            && !value.chars().any(char::is_control)
    };
    if !valid(&content.statement, 240)
        || content.due_at.as_ref().is_some_and(|value| {
            value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos)
        })
        || content
            .condition
            .as_deref()
            .is_some_and(|value| !valid(value, 120))
        || content.obligated_party_id.len() != 16
        || content.obligated_party_id.iter().all(|byte| *byte == 0)
        || content
            .beneficiary_party_id
            .as_ref()
            .is_some_and(|value| value.len() != 16 || value.iter().all(|byte| *byte == 0))
        || content.evidence_links.iter().any(|value| {
            value.evidence_link_id.len() != 16
                || value.evidence_link_id.iter().all(|byte| *byte == 0)
                || value.evidence_owner_id.is_empty()
                || value.evidence_owner_id.len() > 128
                || value.evidence_record_id.len() != 16
                || value.evidence_record_id.iter().all(|byte| *byte == 0)
                || value.evidence_revision == 0
                || value.evidence_digest.len() != 32
                || value.evidence_digest.iter().all(|byte| *byte == 0)
        })
    {
        return Err(ReviewObligationCandidateBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn obligations_reference_id(review: &ReviewObligationCandidateV1, sha256: [u8; 32]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.review.obligation-candidate.obligations-copy.v1\0");
    digest.update(review.review_id);
    digest.update(review.review_revision.saturating_add(1).to_be_bytes());
    digest.update(review.candidate_id);
    digest.update(sha256);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn release_operation_id(submission_id: [u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.review.obligation-candidate.release-source.v1\0");
    digest.update(submission_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReviewObligationCandidateBlobErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewObligationCandidateBlobErrorV1::InvalidReceipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_review_obligation_candidate_core::{
        ReviewObligationCandidatePromotionStatusV1, ReviewObligationCandidateStateV1,
        ReviewObligationCandidateTimestampV1,
    };

    fn pending_review() -> ReviewObligationCandidateV1 {
        ReviewObligationCandidateV1 {
            review_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            candidate_id: [2; 16],
            candidate_digest: [3; 32],
            source_evidence_id: [4; 16],
            source_evidence_revision: 1,
            statement: "Подготовить ответ".to_owned(),
            condition: None,
            due_at: None,
            obligated_party_id: [5; 16],
            beneficiary_party_id: Some([6; 16]),
            evidence_links: Vec::new(),
            state: ReviewObligationCandidateStateV1::Pending,
            promotion_status: ReviewObligationCandidatePromotionStatusV1::NotRequested,
            review_revision: 1,
            decided_by_owner_device_id: None,
            decided_at: None,
            promoted_obligation_id: None,
            updated_at: ReviewObligationCandidateTimestampV1 {
                unix_seconds: 1_800_000_000,
                nanos: 0,
            },
        }
    }

    #[test]
    fn obligations_blob_identity_is_revision_and_content_bound() {
        let review = pending_review();
        assert_eq!(
            obligations_reference_id(&review, [5; 32]),
            obligations_reference_id(&review, [5; 32])
        );
        assert_ne!(
            obligations_reference_id(&review, [5; 32]),
            obligations_reference_id(&review, [6; 32])
        );
    }
}
