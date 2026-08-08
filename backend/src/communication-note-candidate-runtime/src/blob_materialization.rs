use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyReleaseRequestV1, ManagedBlobCustodyTargetV1,
    ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_communication_note_candidate_core::{
    CommunicationNoteCandidateV1, CommunicationNoteSourceBasisV1, CommunicationNoteTopicHintV1,
};
use makosh_communication_note_candidate_persistence::CommunicationNoteCandidateBlobCleanupV1;
use makosh_communications_note_source_api::COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1;
use makosh_review_note_candidate_api::{
    REVIEW_NOTE_CANDIDATE_BLOB_TARGET_CAPABILITY_ID_V1,
    REVIEW_NOTE_CANDIDATE_BLOB_TARGET_MODULE_ID_V1, REVIEW_NOTE_CANDIDATE_BLOB_TARGET_OWNER_ID_V1,
    REVIEW_NOTE_CANDIDATE_MAX_BLOB_BYTES_V1, REVIEW_NOTE_CANDIDATE_MAX_PROOF_BYTES_V1,
    wire::{
        ReviewNoteCandidateContentV1, ReviewNoteSourceBasisV1, ReviewNoteTopicHintV1,
        ReviewTargetBoundCandidateReceiptV1,
    },
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::COMMUNICATION_NOTE_CANDIDATE_BLOB_CAPABILITY_ID_V1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationNoteCandidateSourceBlobReceiptV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

pub(crate) struct CommunicationNoteCandidateBlobMaterializationV1 {
    pub body_utf8: Zeroizing<Vec<u8>>,
    pub source_cleanup: CommunicationNoteCandidateBlobCleanupV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidateBlobErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub(crate) fn materialize_note_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source: &CommunicationNoteCandidateSourceBlobReceiptV1,
) -> Result<CommunicationNoteCandidateBlobMaterializationV1, CommunicationNoteCandidateBlobErrorV1>
{
    validate_source(source)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: COMMUNICATION_NOTE_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source.reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source.sha256,
            custody_source_proof: &source.custody_proof,
            evidence_id: &source.result_message_id,
            evidence_envelope_sha256: &source.envelope_sha256,
        },
    )
    .map_err(|_| CommunicationNoteCandidateBlobErrorV1::Unavailable)?;
    let local_reference = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(&transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(|_| CommunicationNoteCandidateBlobErrorV1::Unavailable)?;
    let body_utf8 = read_exact(
        channel,
        dispatcher,
        &local_reference,
        source.declared_bytes,
        &source.sha256,
    )?;
    Ok(CommunicationNoteCandidateBlobMaterializationV1 {
        body_utf8,
        source_cleanup: CommunicationNoteCandidateBlobCleanupV1 {
            reference_id: local_reference,
            declared_bytes: source.declared_bytes,
            sha256: source.sha256,
            custody_proof: source.custody_proof.clone(),
        },
    })
}

pub(crate) fn release_note_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    source_cleanup: &CommunicationNoteCandidateBlobCleanupV1,
    accepted: bool,
) -> Result<(), CommunicationNoteCandidateBlobErrorV1> {
    let reason = if accepted {
        BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
    } else {
        BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
    };
    request_managed_blob_custody_release_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyReleaseRequestV1 {
            operation_id: &release_operation_id(run_id),
            capability_id: COMMUNICATION_NOTE_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            reference_id: &source_cleanup.reference_id,
            declared_size: source_cleanup.declared_bytes,
            receipt_sha256: &source_cleanup.sha256,
            custody_source_proof: &source_cleanup.custody_proof,
            reason,
        },
    )
    .map(|_| ())
    .map_err(|_| CommunicationNoteCandidateBlobErrorV1::Unavailable)
}

pub(crate) fn read_note_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source_cleanup: &CommunicationNoteCandidateBlobCleanupV1,
) -> Result<Zeroizing<Vec<u8>>, CommunicationNoteCandidateBlobErrorV1> {
    read_exact(
        channel,
        dispatcher,
        &source_cleanup.reference_id,
        source_cleanup.declared_bytes,
        &source_cleanup.sha256,
    )
}

pub(crate) fn write_review_candidate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    candidate: &CommunicationNoteCandidateV1,
) -> Result<ReviewTargetBoundCandidateReceiptV1, CommunicationNoteCandidateBlobErrorV1> {
    let bytes = Zeroizing::new(
        ReviewNoteCandidateContentV1 {
            title: candidate.title.clone(),
            excerpt: candidate.excerpt.clone(),
            topic_hints: candidate
                .topic_hints
                .iter()
                .copied()
                .map(review_topic_hint)
                .map(|value| value as i32)
                .collect(),
            source_basis: review_source_basis(candidate.source_basis) as i32,
            confidence_basis_points: candidate.confidence_basis_points,
        }
        .encode_to_vec(),
    );
    let declared_bytes = u64::try_from(bytes.len())
        .map_err(|_| CommunicationNoteCandidateBlobErrorV1::InvalidReceipt)?;
    if !(1..=REVIEW_NOTE_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&declared_bytes) {
        return Err(CommunicationNoteCandidateBlobErrorV1::InvalidReceipt);
    }
    let sha256: [u8; 32] = Sha256::digest(bytes.as_slice()).into();
    let reference_id = review_reference_id(candidate, sha256);
    let session = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATION_NOTE_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: REVIEW_NOTE_CANDIDATE_BLOB_TARGET_OWNER_ID_V1,
                module_id: REVIEW_NOTE_CANDIDATE_BLOB_TARGET_MODULE_ID_V1,
                capability_id: REVIEW_NOTE_CANDIDATE_BLOB_TARGET_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(|_| CommunicationNoteCandidateBlobErrorV1::Unavailable)?;
    let proof = session.custody_transfer_source_proof;
    if proof.is_empty() || proof.len() > REVIEW_NOTE_CANDIDATE_MAX_PROOF_BYTES_V1 {
        return Err(CommunicationNoteCandidateBlobErrorV1::InvalidReceipt);
    }
    if BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes.to_vec()))
        .is_err()
    {
        let existing = read_exact(channel, dispatcher, &reference_id, declared_bytes, &sha256)?;
        if existing.as_slice() != bytes.as_slice() {
            return Err(CommunicationNoteCandidateBlobErrorV1::InvalidReceipt);
        }
    }
    Ok(ReviewTargetBoundCandidateReceiptV1 {
        reference_id: reference_id.to_vec(),
        declared_bytes,
        sha256: sha256.to_vec(),
        custody_transfer_source_proof: proof,
    })
}

const fn review_source_basis(value: CommunicationNoteSourceBasisV1) -> ReviewNoteSourceBasisV1 {
    match value {
        CommunicationNoteSourceBasisV1::Subject => {
            ReviewNoteSourceBasisV1::ReviewNoteSourceBasisSubject
        }
        CommunicationNoteSourceBasisV1::Body => ReviewNoteSourceBasisV1::ReviewNoteSourceBasisBody,
        CommunicationNoteSourceBasisV1::Combined => {
            ReviewNoteSourceBasisV1::ReviewNoteSourceBasisCombined
        }
    }
}

const fn review_topic_hint(value: CommunicationNoteTopicHintV1) -> ReviewNoteTopicHintV1 {
    match value {
        CommunicationNoteTopicHintV1::Financial => {
            ReviewNoteTopicHintV1::ReviewNoteTopicHintFinancial
        }
        CommunicationNoteTopicHintV1::Legal => ReviewNoteTopicHintV1::ReviewNoteTopicHintLegal,
        CommunicationNoteTopicHintV1::DecisionStatement => {
            ReviewNoteTopicHintV1::ReviewNoteTopicHintDecisionStatement
        }
        CommunicationNoteTopicHintV1::DeadlineStatement => {
            ReviewNoteTopicHintV1::ReviewNoteTopicHintDeadlineStatement
        }
    }
}

fn read_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    reference_id: &[u8; 16],
    declared_bytes: u64,
    sha256: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, CommunicationNoteCandidateBlobErrorV1> {
    let read = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATION_NOTE_CANDIDATE_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(sha256),
            custody_target: None,
        },
    )
    .map_err(|_| CommunicationNoteCandidateBlobErrorV1::Unavailable)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(read.data_socket_path)
            .and_then(|client| {
                client.read_range(read.grant, read.channel_binding, 0, declared_bytes)
            })
            .map_err(|_| CommunicationNoteCandidateBlobErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != sha256
    {
        return Err(CommunicationNoteCandidateBlobErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

fn validate_source(
    source: &CommunicationNoteCandidateSourceBlobReceiptV1,
) -> Result<(), CommunicationNoteCandidateBlobErrorV1> {
    if source.result_message_id.iter().all(|byte| *byte == 0)
        || source.envelope_sha256.iter().all(|byte| *byte == 0)
        || source.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1).contains(&source.declared_bytes)
        || source.sha256.iter().all(|byte| *byte == 0)
        || source.custody_proof.is_empty()
        || source.custody_proof.len() > 2_048
    {
        return Err(CommunicationNoteCandidateBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationNoteCandidateBlobErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| CommunicationNoteCandidateBlobErrorV1::InvalidReceipt)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(CommunicationNoteCandidateBlobErrorV1::InvalidReceipt)
}

fn release_operation_id(run_id: [u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_note_candidate.release_source.v1\0");
    digest.update(run_id);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn review_reference_id(candidate: &CommunicationNoteCandidateV1, sha256: [u8; 32]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_note_candidate.review-copy.v1\0");
    digest.update(candidate.candidate_id);
    digest.update(candidate.candidate_digest);
    digest.update(sha256);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}
