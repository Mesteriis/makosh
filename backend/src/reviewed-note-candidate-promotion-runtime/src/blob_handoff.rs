use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyReleaseRequestV1, ManagedBlobCustodyTargetV1,
    ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_knowledge_command_api::{
    KNOWLEDGE_MODULE_ID_V1, KNOWLEDGE_OWNER_ID_V1,
    KNOWLEDGE_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1,
    KNOWLEDGE_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1,
    KNOWLEDGE_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1,
    wire::{KnowledgeTargetBoundCandidateReceiptV1, ReviewedKnowledgeNoteContentV1},
};
use makosh_review_note_candidate_api::wire::ReviewNoteCandidateContentV1;
use makosh_reviewed_note_candidate_promotion_persistence::PromotionBlobReceiptV1;
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::admission::REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_CAPABILITY_ID_V1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PromotionBlobHandoffErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub(crate) fn transfer_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    approval_message_id: [u8; 16],
    approval_envelope_sha256: [u8; 32],
    source: &PromotionBlobReceiptV1,
) -> Result<[u8; 16], PromotionBlobHandoffErrorV1> {
    validate_receipt(source)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source.reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source.sha256,
            custody_source_proof: &source.custody_proof,
            evidence_id: &approval_message_id,
            evidence_envelope_sha256: &approval_envelope_sha256,
        },
    )
    .map_err(|_| PromotionBlobHandoffErrorV1::Unavailable)?;
    let local_reference_id = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(&transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(|_| PromotionBlobHandoffErrorV1::Unavailable)?;
    Ok(local_reference_id)
}

pub(crate) fn build_knowledge_receipt_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    command_id: [u8; 16],
    source: &PromotionBlobReceiptV1,
    materialized_reference_id: [u8; 16],
) -> Result<KnowledgeTargetBoundCandidateReceiptV1, PromotionBlobHandoffErrorV1> {
    let source_bytes = read_exact(
        channel,
        dispatcher,
        &materialized_reference_id,
        source.declared_bytes,
        &source.sha256,
    )?;
    let content = map_content(source_bytes.as_slice())?;
    let bytes = Zeroizing::new(content.encode_to_vec());
    let declared_bytes =
        u64::try_from(bytes.len()).map_err(|_| PromotionBlobHandoffErrorV1::InvalidReceipt)?;
    if !(1..=KNOWLEDGE_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&declared_bytes) {
        return Err(PromotionBlobHandoffErrorV1::InvalidReceipt);
    }
    let sha256: [u8; 32] = Sha256::digest(bytes.as_slice()).into();
    let reference_id = knowledge_reference_id(command_id, sha256);
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: KNOWLEDGE_OWNER_ID_V1,
                module_id: KNOWLEDGE_MODULE_ID_V1,
                capability_id: KNOWLEDGE_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(|_| PromotionBlobHandoffErrorV1::Unavailable)?;
    let proof = session.custody_transfer_source_proof;
    if proof.is_empty() || proof.len() > KNOWLEDGE_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1 {
        return Err(PromotionBlobHandoffErrorV1::InvalidReceipt);
    }
    if BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes.to_vec()))
        .is_err()
    {
        let existing = read_exact(channel, dispatcher, &reference_id, declared_bytes, &sha256)?;
        if existing.as_slice() != bytes.as_slice() {
            return Err(PromotionBlobHandoffErrorV1::InvalidReceipt);
        }
    }
    Ok(KnowledgeTargetBoundCandidateReceiptV1 {
        reference_id: reference_id.to_vec(),
        declared_bytes,
        sha256: sha256.to_vec(),
        custody_transfer_source_proof: proof,
    })
}

pub(crate) fn release_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    approval_message_id: [u8; 16],
    source: &PromotionBlobReceiptV1,
    materialized_reference_id: [u8; 16],
) -> Result<(), PromotionBlobHandoffErrorV1> {
    request_managed_blob_custody_release_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyReleaseRequestV1 {
            operation_id: &release_operation_id(approval_message_id),
            capability_id: REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_CAPABILITY_ID_V1,
            reference_id: &materialized_reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source.sha256,
            custody_source_proof: &source.custody_proof,
            reason: BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1,
        },
    )
    .map(|_| ())
    .map_err(|_| PromotionBlobHandoffErrorV1::Unavailable)
}

fn read_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    reference_id: &[u8; 16],
    declared_bytes: u64,
    sha256: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, PromotionBlobHandoffErrorV1> {
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(sha256),
            custody_target: None,
        },
    )
    .map_err(|_| PromotionBlobHandoffErrorV1::Unavailable)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(session.grant, session.channel_binding, 0, declared_bytes)
            })
            .map_err(|_| PromotionBlobHandoffErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != sha256
    {
        return Err(PromotionBlobHandoffErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

fn map_content(
    bytes: &[u8],
) -> Result<ReviewedKnowledgeNoteContentV1, PromotionBlobHandoffErrorV1> {
    let source = ReviewNoteCandidateContentV1::decode(bytes)
        .map_err(|_| PromotionBlobHandoffErrorV1::InvalidReceipt)?;
    let valid_title = !source.title.trim().is_empty()
        && source.title.chars().count() <= 240
        && !source.title.chars().any(char::is_control);
    let valid_excerpt = !source.excerpt.trim().is_empty()
        && source.excerpt.chars().count() <= 2_000
        && !source
            .excerpt
            .chars()
            .any(|character| character.is_control() && character != '\n');
    let valid_hints = !source.topic_hints.is_empty()
        && source.topic_hints.len() <= 4
        && source
            .topic_hints
            .iter()
            .all(|value| (1..=4).contains(value))
        && source.topic_hints.windows(2).all(|pair| pair[0] < pair[1]);
    if !valid_title
        || !valid_excerpt
        || !valid_hints
        || !(1..=3).contains(&source.source_basis)
        || !(1..=10_000).contains(&source.confidence_basis_points)
    {
        return Err(PromotionBlobHandoffErrorV1::InvalidReceipt);
    }
    Ok(ReviewedKnowledgeNoteContentV1 {
        title: source.title,
        excerpt: source.excerpt,
        topic_hints: source.topic_hints,
        source_basis: source.source_basis,
        confidence_basis_points: source.confidence_basis_points,
    })
}

fn validate_receipt(source: &PromotionBlobReceiptV1) -> Result<(), PromotionBlobHandoffErrorV1> {
    if source.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=KNOWLEDGE_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&source.declared_bytes)
        || source.sha256.iter().all(|byte| *byte == 0)
        || source.custody_proof.is_empty()
        || source.custody_proof.len() > KNOWLEDGE_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1
    {
        return Err(PromotionBlobHandoffErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn knowledge_reference_id(command_id: [u8; 16], sha256: [u8; 32]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.reviewed-note-candidate-promotion.knowledge-blob.v1\0");
    digest.update(command_id);
    digest.update(sha256);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn release_operation_id(approval_message_id: [u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.reviewed-note-candidate-promotion.release-source.v1\0");
    digest.update(approval_message_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], PromotionBlobHandoffErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(PromotionBlobHandoffErrorV1::InvalidReceipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_review_content_to_typed_knowledge_content() {
        let bytes = ReviewNoteCandidateContentV1 {
            title: "Contract approved".to_owned(),
            excerpt: "Invoice amount\nPayment by Friday".to_owned(),
            topic_hints: vec![1, 4],
            source_basis: 3,
            confidence_basis_points: 8_300,
        }
        .encode_to_vec();
        let mapped = map_content(&bytes).expect("mapped");
        assert_eq!(mapped.topic_hints, vec![1, 4]);
        assert_eq!(mapped.source_basis, 3);
    }

    #[test]
    fn rejects_unknown_or_unordered_typed_content() {
        let bytes = ReviewNoteCandidateContentV1 {
            title: "Contract approved".to_owned(),
            excerpt: "Invoice amount".to_owned(),
            topic_hints: vec![4, 1],
            source_basis: 3,
            confidence_basis_points: 8_300,
        }
        .encode_to_vec();
        assert_eq!(
            map_content(&bytes),
            Err(PromotionBlobHandoffErrorV1::InvalidReceipt)
        );
    }
}
