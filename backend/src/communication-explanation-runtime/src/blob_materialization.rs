use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    AI_INFERENCE_BLOB_CAPABILITY_ID_V1, AI_INFERENCE_MODULE_ID_V1, AI_MAX_PRIVATE_SOURCE_BYTES_V1,
    AI_OWNER_V1, encode_explanation_source_content_v1,
    wire::{AiExplanationSourceContentV1, AiPrivateSourceReceiptV1},
};
use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyReleaseRequestV1, ManagedBlobCustodyTargetV1,
    ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_communication_explanation_persistence::CommunicationExplanationBlobCleanupV1;
use makosh_communications_ai_source_api::decode_communication_explanation_source_content_v1;
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::COMMUNICATION_EXPLANATION_BLOB_CAPABILITY_ID_V1;

const MAX_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationExplanationSourceBlobReceiptV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommunicationExplanationBlobMaterializationV1 {
    pub ai_source: AiPrivateSourceReceiptV1,
    pub source_cleanup: CommunicationExplanationBlobCleanupV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationExplanationBlobErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub(crate) fn materialize_explanation_source_for_ai_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    source: &CommunicationExplanationSourceBlobReceiptV1,
) -> Result<CommunicationExplanationBlobMaterializationV1, CommunicationExplanationBlobErrorV1> {
    validate_source(source)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: COMMUNICATION_EXPLANATION_BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source.reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source.sha256,
            custody_source_proof: &source.custody_proof,
            evidence_id: &source.result_message_id,
            evidence_envelope_sha256: &source.envelope_sha256,
        },
    )
    .map_err(|_| CommunicationExplanationBlobErrorV1::Unavailable)?;
    let local_reference = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(&transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(|_| CommunicationExplanationBlobErrorV1::Unavailable)?;
    let raw_source = read_exact(
        channel,
        dispatcher,
        &local_reference,
        source.declared_bytes,
        &source.sha256,
    )?;
    let source_content = decode_communication_explanation_source_content_v1(&raw_source)
        .map_err(|_| CommunicationExplanationBlobErrorV1::InvalidReceipt)?;
    let encoded = Zeroizing::new(
        encode_explanation_source_content_v1(&AiExplanationSourceContentV1 {
            sender_utf8: source_content.sender_utf8,
            subject_utf8: source_content.subject_utf8,
            body_utf8: source_content.body_utf8,
        })
        .map_err(|_| CommunicationExplanationBlobErrorV1::InvalidReceipt)?,
    );
    let declared_bytes = u64::try_from(encoded.len())
        .map_err(|_| CommunicationExplanationBlobErrorV1::InvalidReceipt)?;
    if !(1..=AI_MAX_PRIVATE_SOURCE_BYTES_V1).contains(&declared_bytes) {
        return Err(CommunicationExplanationBlobErrorV1::InvalidReceipt);
    }
    let sha256: [u8; 32] = Sha256::digest(encoded.as_slice()).into();
    let ai_reference = ai_reference_id(run_id, local_reference, sha256);
    let write = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATION_EXPLANATION_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &ai_reference,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: AI_OWNER_V1,
                module_id: AI_INFERENCE_MODULE_ID_V1,
                capability_id: AI_INFERENCE_BLOB_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(|_| CommunicationExplanationBlobErrorV1::Unavailable)?;
    let proof = write.custody_transfer_source_proof;
    if proof.is_empty() || proof.len() > MAX_PROOF_BYTES_V1 {
        return Err(CommunicationExplanationBlobErrorV1::InvalidReceipt);
    }
    if BlobDataClient::new(write.data_socket_path)
        .and_then(|client| {
            client.write(
                write.grant,
                write.channel_binding,
                encoded.as_slice().to_vec(),
            )
        })
        .is_err()
    {
        let existing = read_exact(channel, dispatcher, &ai_reference, declared_bytes, &sha256)?;
        if existing.as_slice() != encoded.as_slice() {
            return Err(CommunicationExplanationBlobErrorV1::InvalidReceipt);
        }
    }
    Ok(CommunicationExplanationBlobMaterializationV1 {
        ai_source: AiPrivateSourceReceiptV1 {
            reference_id: ai_reference.to_vec(),
            declared_bytes,
            sha256: sha256.to_vec(),
            custody_transfer_source_proof: proof,
        },
        source_cleanup: CommunicationExplanationBlobCleanupV1 {
            reference_id: local_reference,
            declared_bytes: source.declared_bytes,
            sha256: source.sha256,
            custody_proof: source.custody_proof.clone(),
        },
    })
}

pub(crate) fn release_explanation_source_blobs_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    ai_source: &AiPrivateSourceReceiptV1,
    source_cleanup: &CommunicationExplanationBlobCleanupV1,
    accepted: bool,
) -> Result<(), CommunicationExplanationBlobErrorV1> {
    let ai_reference = id16(&ai_source.reference_id)?;
    let ai_sha256 = id32(&ai_source.sha256)?;
    let reason = if accepted {
        BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
    } else {
        BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
    };
    release(
        channel,
        dispatcher,
        ManagedBlobCustodyReleaseRequestV1 {
            operation_id: &release_operation_id(run_id, b"ai"),
            capability_id: COMMUNICATION_EXPLANATION_BLOB_CAPABILITY_ID_V1,
            reference_id: &ai_reference,
            declared_size: ai_source.declared_bytes,
            receipt_sha256: &ai_sha256,
            custody_source_proof: &ai_source.custody_transfer_source_proof,
            reason,
        },
    )?;
    release(
        channel,
        dispatcher,
        ManagedBlobCustodyReleaseRequestV1 {
            operation_id: &release_operation_id(run_id, b"source"),
            capability_id: COMMUNICATION_EXPLANATION_BLOB_CAPABILITY_ID_V1,
            reference_id: &source_cleanup.reference_id,
            declared_size: source_cleanup.declared_bytes,
            receipt_sha256: &source_cleanup.sha256,
            custody_source_proof: &source_cleanup.custody_proof,
            reason,
        },
    )
}

fn release(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: ManagedBlobCustodyReleaseRequestV1<'_>,
) -> Result<(), CommunicationExplanationBlobErrorV1> {
    request_managed_blob_custody_release_v2(channel, dispatcher, request)
        .map(|_| ())
        .map_err(|_| CommunicationExplanationBlobErrorV1::Unavailable)
}

fn read_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    reference_id: &[u8; 16],
    declared_bytes: u64,
    sha256: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, CommunicationExplanationBlobErrorV1> {
    let read = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATION_EXPLANATION_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(sha256),
            custody_target: None,
        },
    )
    .map_err(|_| CommunicationExplanationBlobErrorV1::Unavailable)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(read.data_socket_path)
            .and_then(|client| {
                client.read_range(read.grant, read.channel_binding, 0, declared_bytes)
            })
            .map_err(|_| CommunicationExplanationBlobErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != sha256
    {
        return Err(CommunicationExplanationBlobErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

fn validate_source(
    source: &CommunicationExplanationSourceBlobReceiptV1,
) -> Result<(), CommunicationExplanationBlobErrorV1> {
    if source.result_message_id.iter().all(|byte| *byte == 0)
        || source.envelope_sha256.iter().all(|byte| *byte == 0)
        || source.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=AI_MAX_PRIVATE_SOURCE_BYTES_V1).contains(&source.declared_bytes)
        || source.sha256.iter().all(|byte| *byte == 0)
        || source.custody_proof.is_empty()
        || source.custody_proof.len() > MAX_PROOF_BYTES_V1
    {
        return Err(CommunicationExplanationBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn ai_reference_id(run_id: [u8; 16], local_reference: [u8; 16], sha256: [u8; 32]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_explanation.ai-source.v1\0");
    digest.update(run_id);
    digest.update(local_reference);
    digest.update(sha256);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn release_operation_id(run_id: [u8; 16], label: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_explanation.blob-release.v1\0");
    digest.update(label);
    digest.update(run_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationExplanationBlobErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationExplanationBlobErrorV1::InvalidReceipt)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationExplanationBlobErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationExplanationBlobErrorV1::InvalidReceipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_reference_and_cleanup_operations_are_deterministic_and_distinct() {
        assert_eq!(
            ai_reference_id([1; 16], [2; 16], [3; 32]),
            ai_reference_id([1; 16], [2; 16], [3; 32])
        );
        assert_ne!(
            release_operation_id([1; 16], b"ai"),
            release_operation_id([1; 16], b"source")
        );
    }
}
