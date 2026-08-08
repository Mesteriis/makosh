use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyReleaseRequestV1,
    ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_knowledge_command_api::{
    KNOWLEDGE_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1,
    KNOWLEDGE_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1, wire::ReviewedKnowledgeNoteContentV1,
};
use makosh_knowledge_persistence::{KnowledgeBlobCleanupV1, KnowledgeBlobReceiptV1};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KnowledgeBlobErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub(crate) fn transfer_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    command_message_id: [u8; 16],
    command_envelope_sha256: [u8; 32],
    source: &KnowledgeBlobReceiptV1,
) -> Result<KnowledgeBlobCleanupV1, KnowledgeBlobErrorV1> {
    validate_receipt(source)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: KNOWLEDGE_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1,
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
        .map_err(|_| KnowledgeBlobErrorV1::Unavailable)?;
    Ok(KnowledgeBlobCleanupV1 {
        reference_id,
        declared_bytes: source.declared_bytes,
        sha256: source.sha256,
        custody_proof: source.custody_transfer_source_proof.clone(),
    })
}

pub(crate) fn read_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    receipt: &KnowledgeBlobCleanupV1,
) -> Result<Zeroizing<Vec<u8>>, KnowledgeBlobErrorV1> {
    validate_cleanup(receipt)?;
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: KNOWLEDGE_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1,
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
            .map_err(|_| KnowledgeBlobErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(receipt.declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != receipt.sha256
    {
        return Err(KnowledgeBlobErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

pub(crate) fn decode_candidate_content_v1(
    bytes: &[u8],
) -> Result<ReviewedKnowledgeNoteContentV1, KnowledgeBlobErrorV1> {
    let content = ReviewedKnowledgeNoteContentV1::decode(bytes)
        .map_err(|_| KnowledgeBlobErrorV1::InvalidReceipt)?;
    let valid_text = |value: &str, limit: usize| {
        !value.trim().is_empty()
            && value.chars().count() <= limit
            && !value.chars().any(char::is_control)
    };
    let valid_excerpt = !content.excerpt.trim().is_empty()
        && content.excerpt.chars().count() <= 2_000
        && !content
            .excerpt
            .chars()
            .any(|character| character.is_control() && character != '\n');
    if !valid_text(&content.title, 240)
        || !valid_excerpt
        || content.topic_hints.is_empty()
        || content.topic_hints.len() > 4
        || !content
            .topic_hints
            .iter()
            .all(|hint| (1..=4).contains(hint))
        || !content.topic_hints.windows(2).all(|pair| pair[0] < pair[1])
        || !(1..=3).contains(&content.source_basis)
        || !(1..=10_000).contains(&content.confidence_basis_points)
    {
        return Err(KnowledgeBlobErrorV1::InvalidReceipt);
    }
    Ok(content)
}

pub(crate) fn release_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    command_id: [u8; 16],
    cleanup: &KnowledgeBlobCleanupV1,
    accepted: bool,
) -> Result<(), KnowledgeBlobErrorV1> {
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
            capability_id: KNOWLEDGE_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            reference_id: &cleanup.reference_id,
            declared_size: cleanup.declared_bytes,
            receipt_sha256: &cleanup.sha256,
            custody_source_proof: &cleanup.custody_proof,
            reason,
        },
    )
    .map(|_| ())
    .map_err(|_| KnowledgeBlobErrorV1::Unavailable)
}

fn validate_receipt(receipt: &KnowledgeBlobReceiptV1) -> Result<(), KnowledgeBlobErrorV1> {
    if receipt.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=KNOWLEDGE_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&receipt.declared_bytes)
        || receipt.sha256.iter().all(|byte| *byte == 0)
        || receipt.custody_transfer_source_proof.is_empty()
        || receipt.custody_transfer_source_proof.len()
            > makosh_knowledge_command_api::KNOWLEDGE_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1
    {
        return Err(KnowledgeBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn validate_cleanup(receipt: &KnowledgeBlobCleanupV1) -> Result<(), KnowledgeBlobErrorV1> {
    if receipt.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=KNOWLEDGE_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&receipt.declared_bytes)
        || receipt.sha256.iter().all(|byte| *byte == 0)
        || receipt.custody_proof.is_empty()
        || receipt.custody_proof.len()
            > makosh_knowledge_command_api::KNOWLEDGE_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1
    {
        return Err(KnowledgeBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], KnowledgeBlobErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(KnowledgeBlobErrorV1::InvalidReceipt)
}

fn classify_blob_client_error_v1(error: BlobClientError) -> KnowledgeBlobErrorV1 {
    match error {
        BlobClientError::Rejected(_) | BlobClientError::InvalidSessionRequest => {
            KnowledgeBlobErrorV1::InvalidReceipt
        }
        _ => KnowledgeBlobErrorV1::Unavailable,
    }
}

fn release_operation_id(command_id: [u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.knowledge.reviewed-candidate.release.v1\0");
    digest.update(command_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_knowledge_command_api::wire::ReviewedKnowledgeNoteContentV1;

    #[test]
    fn release_identity_is_command_stable() {
        assert_eq!(release_operation_id([1; 16]), release_operation_id([1; 16]));
        assert_ne!(release_operation_id([1; 16]), release_operation_id([2; 16]));
    }

    #[test]
    fn denied_custody_is_terminal_while_transport_outage_is_retryable() {
        assert_eq!(
            classify_blob_client_error_v1(BlobClientError::Rejected("expired".to_owned())),
            KnowledgeBlobErrorV1::InvalidReceipt
        );
        assert_eq!(
            classify_blob_client_error_v1(BlobClientError::Unavailable),
            KnowledgeBlobErrorV1::Unavailable
        );
    }

    #[test]
    fn candidate_content_is_bounded_typed_and_ordered() {
        let content = ReviewedKnowledgeNoteContentV1 {
            title: "Contract approved".to_owned(),
            excerpt: "Invoice amount\nPayment by Friday".to_owned(),
            topic_hints: vec![1, 2],
            source_basis: 3,
            confidence_basis_points: 8_300,
        };
        assert_eq!(
            decode_candidate_content_v1(&content.encode_to_vec()).expect("content"),
            content,
        );

        let mut invalid = content.clone();
        invalid.topic_hints = vec![2, 1];
        assert_eq!(
            decode_candidate_content_v1(&invalid.encode_to_vec()),
            Err(KnowledgeBlobErrorV1::InvalidReceipt),
        );

        let mut invalid = content;
        invalid.excerpt = "secret\u{0000}".to_owned();
        assert_eq!(
            decode_candidate_content_v1(&invalid.encode_to_vec()),
            Err(KnowledgeBlobErrorV1::InvalidReceipt),
        );
    }
}
