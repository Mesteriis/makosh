use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    AI_ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1, AI_INFERENCE_BLOB_CAPABILITY_ID_V1,
    AI_INFERENCE_MODULE_ID_V1, AI_OWNER_V1, wire::AiPrivateSourceReceiptV1,
};
use makosh_attachment_translation_api::ATTACHMENT_TRANSLATION_MAX_RESULT_BYTES_V1;
use makosh_attachment_translation_persistence::AttachmentTranslationSourceAuthorityV1;
use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyReleaseRequestV1, ManagedBlobCustodyTargetV1,
    ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_release_v2, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::ATTACHMENT_TRANSLATION_BLOB_CAPABILITY_ID_V1;

const MAX_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationSourceBlobReceiptV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AttachmentTranslationBlobMaterializationV1 {
    pub ai_source: AiPrivateSourceReceiptV1,
    pub source_authority: AttachmentTranslationSourceAuthorityV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AttachmentTranslationArtifactReceiptV1 {
    pub reference_id: [u8; 16],
    pub sha256: [u8; 32],
    pub declared_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationBlobErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub(crate) fn materialize_translation_source_for_ai_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    source: &AttachmentTranslationSourceBlobReceiptV1,
) -> Result<AttachmentTranslationBlobMaterializationV1, AttachmentTranslationBlobErrorV1> {
    validate_source(source)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: ATTACHMENT_TRANSLATION_BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source.reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source.sha256,
            custody_source_proof: &source.custody_proof,
            evidence_id: &source.result_message_id,
            evidence_envelope_sha256: &source.envelope_sha256,
        },
    )
    .map_err(|_| AttachmentTranslationBlobErrorV1::Unavailable)?;
    let local_reference = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(&transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(|_| AttachmentTranslationBlobErrorV1::Unavailable)?;
    let authority = AttachmentTranslationSourceAuthorityV1 {
        reference_id: local_reference,
        declared_bytes: source.declared_bytes,
        sha256: source.sha256,
        custody_proof: source.custody_proof.clone(),
    };
    let ai_source =
        materialize_ai_source_from_authority_v1(channel, dispatcher, run_id, &authority)?;
    Ok(AttachmentTranslationBlobMaterializationV1 {
        ai_source,
        source_authority: authority,
    })
}

pub(crate) fn materialize_ai_source_from_authority_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    authority: &AttachmentTranslationSourceAuthorityV1,
) -> Result<AiPrivateSourceReceiptV1, AttachmentTranslationBlobErrorV1> {
    if authority.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=AI_ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1).contains(&authority.declared_bytes)
        || authority.sha256.iter().all(|byte| *byte == 0)
    {
        return Err(AttachmentTranslationBlobErrorV1::InvalidReceipt);
    }
    let source = read_exact(
        channel,
        dispatcher,
        &authority.reference_id,
        authority.declared_bytes,
        &authority.sha256,
    )?;
    std::str::from_utf8(source.as_slice())
        .ok()
        .filter(|text| !text.is_empty())
        .ok_or(AttachmentTranslationBlobErrorV1::InvalidReceipt)?;
    let ai_reference = ai_reference_id(run_id, authority.reference_id, authority.sha256);
    let write = write_exact(
        channel,
        dispatcher,
        &ai_reference,
        source.as_slice(),
        &authority.sha256,
        Some(ManagedBlobCustodyTargetV1 {
            owner_id: AI_OWNER_V1,
            module_id: AI_INFERENCE_MODULE_ID_V1,
            capability_id: AI_INFERENCE_BLOB_CAPABILITY_ID_V1,
        }),
    )?;
    if write.is_empty() || write.len() > MAX_PROOF_BYTES_V1 {
        return Err(AttachmentTranslationBlobErrorV1::InvalidReceipt);
    }
    Ok(AiPrivateSourceReceiptV1 {
        reference_id: ai_reference.to_vec(),
        declared_bytes: authority.declared_bytes,
        sha256: authority.sha256.to_vec(),
        custody_transfer_source_proof: write,
    })
}

pub(crate) fn materialize_translation_result_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    translated_utf8: &[u8],
) -> Result<AttachmentTranslationArtifactReceiptV1, AttachmentTranslationBlobErrorV1> {
    let declared_bytes = u64::try_from(translated_utf8.len())
        .map_err(|_| AttachmentTranslationBlobErrorV1::InvalidReceipt)?;
    if !(1..=ATTACHMENT_TRANSLATION_MAX_RESULT_BYTES_V1).contains(&declared_bytes)
        || std::str::from_utf8(translated_utf8)
            .ok()
            .filter(|text| !text.is_empty())
            .is_none()
    {
        return Err(AttachmentTranslationBlobErrorV1::InvalidReceipt);
    }
    let sha256: [u8; 32] = Sha256::digest(translated_utf8).into();
    let reference_id = artifact_reference_id(run_id, sha256);
    write_exact(
        channel,
        dispatcher,
        &reference_id,
        translated_utf8,
        &sha256,
        None,
    )?;
    Ok(AttachmentTranslationArtifactReceiptV1 {
        reference_id,
        sha256,
        declared_bytes,
    })
}

pub(crate) fn release_translation_source_blobs_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    ai_source: &AiPrivateSourceReceiptV1,
    source_authority: &AttachmentTranslationSourceAuthorityV1,
    accepted: bool,
) -> Result<(), AttachmentTranslationBlobErrorV1> {
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
            capability_id: ATTACHMENT_TRANSLATION_BLOB_CAPABILITY_ID_V1,
            reference_id: &id16(&ai_source.reference_id)?,
            declared_size: ai_source.declared_bytes,
            receipt_sha256: &id32(&ai_source.sha256)?,
            custody_source_proof: &ai_source.custody_transfer_source_proof,
            reason,
        },
    )?;
    release(
        channel,
        dispatcher,
        ManagedBlobCustodyReleaseRequestV1 {
            operation_id: &release_operation_id(run_id, b"source"),
            capability_id: ATTACHMENT_TRANSLATION_BLOB_CAPABILITY_ID_V1,
            reference_id: &source_authority.reference_id,
            declared_size: source_authority.declared_bytes,
            receipt_sha256: &source_authority.sha256,
            custody_source_proof: &source_authority.custody_proof,
            reason,
        },
    )
}

fn write_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    reference_id: &[u8; 16],
    bytes: &[u8],
    sha256: &[u8; 32],
    custody_target: Option<ManagedBlobCustodyTargetV1<'_>>,
) -> Result<Vec<u8>, AttachmentTranslationBlobErrorV1> {
    let declared_size =
        u64::try_from(bytes.len()).map_err(|_| AttachmentTranslationBlobErrorV1::InvalidReceipt)?;
    let write = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: ATTACHMENT_TRANSLATION_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id,
            declared_size,
            backup_class: 1,
            receipt_sha256: Some(sha256),
            custody_target,
        },
    )
    .map_err(|_| AttachmentTranslationBlobErrorV1::Unavailable)?;
    let proof = write.custody_transfer_source_proof;
    if BlobDataClient::new(write.data_socket_path)
        .and_then(|client| client.write(write.grant, write.channel_binding, bytes.to_vec()))
        .is_err()
    {
        let existing = read_exact(channel, dispatcher, reference_id, declared_size, sha256)?;
        if existing.as_slice() != bytes {
            return Err(AttachmentTranslationBlobErrorV1::InvalidReceipt);
        }
    }
    Ok(proof)
}

fn read_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    reference_id: &[u8; 16],
    declared_bytes: u64,
    sha256: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, AttachmentTranslationBlobErrorV1> {
    let read = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: ATTACHMENT_TRANSLATION_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(sha256),
            custody_target: None,
        },
    )
    .map_err(|_| AttachmentTranslationBlobErrorV1::Unavailable)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(read.data_socket_path)
            .and_then(|client| {
                client.read_range(read.grant, read.channel_binding, 0, declared_bytes)
            })
            .map_err(|_| AttachmentTranslationBlobErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != sha256
    {
        return Err(AttachmentTranslationBlobErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

fn release(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: ManagedBlobCustodyReleaseRequestV1<'_>,
) -> Result<(), AttachmentTranslationBlobErrorV1> {
    request_managed_blob_custody_release_v2(channel, dispatcher, request)
        .map(|_| ())
        .map_err(|_| AttachmentTranslationBlobErrorV1::Unavailable)
}

fn validate_source(
    source: &AttachmentTranslationSourceBlobReceiptV1,
) -> Result<(), AttachmentTranslationBlobErrorV1> {
    if source.result_message_id.iter().all(|byte| *byte == 0)
        || source.envelope_sha256.iter().all(|byte| *byte == 0)
        || source.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=AI_ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1).contains(&source.declared_bytes)
        || source.sha256.iter().all(|byte| *byte == 0)
        || source.custody_proof.is_empty()
        || source.custody_proof.len() > MAX_PROOF_BYTES_V1
    {
        return Err(AttachmentTranslationBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn ai_reference_id(run_id: [u8; 16], local_reference: [u8; 16], sha256: [u8; 32]) -> [u8; 16] {
    reference_id(
        b"makosh.attachment_translation.ai-source.v1\0",
        run_id,
        local_reference,
        sha256,
    )
}

fn artifact_reference_id(run_id: [u8; 16], sha256: [u8; 32]) -> [u8; 16] {
    reference_id(
        b"makosh.attachment_translation.artifact.v1\0",
        run_id,
        [0; 16],
        sha256,
    )
}

fn reference_id(
    label: &[u8],
    run_id: [u8; 16],
    local_reference: [u8; 16],
    sha256: [u8; 32],
) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(label);
    digest.update(run_id);
    digest.update(local_reference);
    digest.update(sha256);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn release_operation_id(run_id: [u8; 16], label: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment_translation.blob-release.v1\0");
    digest.update(label);
    digest.update(run_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], AttachmentTranslationBlobErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(AttachmentTranslationBlobErrorV1::InvalidReceipt)
}

fn id32(value: &[u8]) -> Result<[u8; 32], AttachmentTranslationBlobErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(AttachmentTranslationBlobErrorV1::InvalidReceipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_artifact_and_cleanup_identities_are_distinct() {
        assert_ne!(
            ai_reference_id([1; 16], [2; 16], [3; 32]),
            artifact_reference_id([1; 16], [3; 32])
        );
        assert_ne!(
            release_operation_id([1; 16], b"ai"),
            release_operation_id([1; 16], b"source")
        );
    }
}
