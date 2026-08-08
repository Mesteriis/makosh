use std::{os::unix::net::UnixStream, time::Duration};

use makosh_attachment_text_extraction_persistence::{
    ClaimedAttachmentTextExtractionJobV1, PersistedAttachmentTextArtifactV1,
    TextExtractionTargetBlobReceiptV1,
};
use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobCustodyTransferRequestV1,
    ManagedBlobSessionRequestV1, request_managed_blob_custody_transfer_v2,
    request_managed_blob_session_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::BlobDataOperationV1,
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::ATTACHMENT_TEXT_EXTRACTION_BLOB_CAPABILITY_ID_V1;
use makosh_attachment_translation_ingress::{
    ATTACHMENT_TRANSLATION_BLOB_TARGET_CAPABILITY_ID_V1,
    ATTACHMENT_TRANSLATION_BLOB_TARGET_MODULE_ID_V1,
    ATTACHMENT_TRANSLATION_BLOB_TARGET_OWNER_ID_V1, ATTACHMENT_TRANSLATION_MAX_PROOF_BYTES_V1,
    ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1,
};

pub(crate) fn transfer_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedAttachmentTextExtractionJobV1,
) -> Result<TextExtractionTargetBlobReceiptV1, BlobErrorV1> {
    blocking(channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let transfer = request_managed_blob_custody_transfer_v2(
            channel,
            &mut dispatcher,
            ManagedBlobCustodyTransferRequestV1 {
                capability_id: ATTACHMENT_TEXT_EXTRACTION_BLOB_CAPABILITY_ID_V1,
                source_reference_id: &claimed.source_reference_id,
                declared_size: claimed.source_declared_size,
                receipt_sha256: &claimed.source_receipt_sha256,
                custody_source_proof: &claimed.custody_transfer_source_proof,
                evidence_id: &claimed.delegation_result_message_id,
                evidence_envelope_sha256: &claimed.delegation_result_envelope_sha256,
            },
        )
        .map_err(|_| BlobErrorV1::Unavailable)?;
        let reference_id = id16(&transfer.grant.target_reference_id)?;
        BlobDataClient::new(transfer.data_socket_path)
            .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
            .map_err(|_| BlobErrorV1::Unavailable)?;
        Ok(TextExtractionTargetBlobReceiptV1 {
            reference_id,
            receipt_sha256: claimed.source_receipt_sha256,
        })
    })
}

pub(crate) fn read_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedAttachmentTextExtractionJobV1,
    receipt: TextExtractionTargetBlobReceiptV1,
) -> Result<Zeroizing<Vec<u8>>, BlobErrorV1> {
    blocking(channel, |channel| {
        read_exact(
            channel,
            &receipt.reference_id,
            claimed.source_declared_size,
            &receipt.receipt_sha256,
        )
    })
}

pub(crate) fn write_derived_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedAttachmentTextExtractionJobV1,
    text_utf8: Zeroizing<Vec<u8>>,
    parser_identity_sha256: [u8; 32],
) -> Result<([u8; 16], [u8; 32], u64), BlobErrorV1> {
    let declared_size = u64::try_from(text_utf8.len()).map_err(|_| BlobErrorV1::InvalidReceipt)?;
    if declared_size == 0 {
        return Err(BlobErrorV1::InvalidReceipt);
    }
    let receipt_sha256: [u8; 32] = Sha256::digest(text_utf8.as_slice()).into();
    let reference_id = derived_reference_id(
        claimed.request.run_id,
        claimed.source_receipt_sha256,
        parser_identity_sha256,
        receipt_sha256,
    );
    blocking(channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let session = request_managed_blob_session_v2(
            channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: ATTACHMENT_TEXT_EXTRACTION_BLOB_CAPABILITY_ID_V1,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id: &reference_id,
                declared_size,
                backup_class: 1,
                receipt_sha256: Some(&receipt_sha256),
                custody_target: None,
            },
        )
        .map_err(|_| BlobErrorV1::Unavailable)?;
        if BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.write(session.grant, session.channel_binding, text_utf8.to_vec())
            })
            .is_err()
        {
            let existing = read_exact(channel, &reference_id, declared_size, &receipt_sha256)?;
            if existing.as_slice() != text_utf8.as_slice() {
                return Err(BlobErrorV1::InvalidReceipt);
            }
        }
        Ok((reference_id, receipt_sha256, declared_size))
    })
}

pub(crate) fn read_artifact_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    artifact: &PersistedAttachmentTextArtifactV1,
) -> Result<Zeroizing<Vec<u8>>, BlobErrorV1> {
    blocking(channel, |channel| {
        read_exact(
            channel,
            &artifact.derived_reference_id,
            artifact.extracted_size_bytes,
            &artifact.derived_receipt_sha256,
        )
    })
}

pub(crate) fn write_translation_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    translation_run_id: [u8; 16],
    source_revision: u64,
    artifact: &PersistedAttachmentTextArtifactV1,
    bytes: Zeroizing<Vec<u8>>,
) -> Result<TranslationSourceBlobReceiptV1, BlobErrorV1> {
    let declared_size = u64::try_from(bytes.len()).map_err(|_| BlobErrorV1::InvalidReceipt)?;
    if !(1..=ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1).contains(&declared_size)
        || source_revision == 0
        || Sha256::digest(bytes.as_slice()).as_slice() != artifact.derived_receipt_sha256
    {
        return Err(BlobErrorV1::InvalidReceipt);
    }
    let reference_id = translation_source_reference_id(
        translation_run_id,
        artifact.run_id,
        source_revision,
        artifact.derived_receipt_sha256,
    );
    blocking(channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let session = request_managed_blob_session_v2(
            channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: ATTACHMENT_TEXT_EXTRACTION_BLOB_CAPABILITY_ID_V1,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id: &reference_id,
                declared_size,
                backup_class: 1,
                receipt_sha256: Some(&artifact.derived_receipt_sha256),
                custody_target: Some(ManagedBlobCustodyTargetV1 {
                    owner_id: ATTACHMENT_TRANSLATION_BLOB_TARGET_OWNER_ID_V1,
                    module_id: ATTACHMENT_TRANSLATION_BLOB_TARGET_MODULE_ID_V1,
                    capability_id: ATTACHMENT_TRANSLATION_BLOB_TARGET_CAPABILITY_ID_V1,
                }),
            },
        )
        .map_err(|_| BlobErrorV1::Unavailable)?;
        if session.custody_transfer_source_proof.is_empty()
            || session.custody_transfer_source_proof.len()
                > ATTACHMENT_TRANSLATION_MAX_PROOF_BYTES_V1
        {
            return Err(BlobErrorV1::InvalidReceipt);
        }
        if BlobDataClient::new(session.data_socket_path)
            .and_then(|client| client.write(session.grant, session.channel_binding, bytes.to_vec()))
            .is_err()
        {
            let existing = read_exact(
                channel,
                &reference_id,
                declared_size,
                &artifact.derived_receipt_sha256,
            )?;
            if existing.as_slice() != bytes.as_slice() {
                return Err(BlobErrorV1::InvalidReceipt);
            }
        }
        Ok(TranslationSourceBlobReceiptV1 {
            reference_id,
            declared_size,
            receipt_sha256: artifact.derived_receipt_sha256,
            custody_transfer_source_proof: session.custody_transfer_source_proof,
        })
    })
}

fn read_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    reference_id: &[u8; 16],
    declared_size: u64,
    receipt_sha256: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, BlobErrorV1> {
    let mut dispatcher = RejectManagedControlRequestsV2;
    let session = request_managed_blob_session_v2(
        channel,
        &mut dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: ATTACHMENT_TEXT_EXTRACTION_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id,
            declared_size,
            backup_class: 1,
            receipt_sha256: Some(receipt_sha256),
            custody_target: None,
        },
    )
    .map_err(|_| BlobErrorV1::Unavailable)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(session.grant, session.channel_binding, 0, declared_size)
            })
            .map_err(|_| BlobErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(declared_size).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != receipt_sha256
    {
        return Err(BlobErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

fn blocking<T>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    operation: impl FnOnce(&mut ManagedControlChannelV2<UnixStream>) -> Result<T, BlobErrorV1>,
) -> Result<T, BlobErrorV1> {
    channel
        .inner_mut()
        .set_nonblocking(false)
        .and_then(|_| {
            channel
                .inner_mut()
                .set_read_timeout(Some(Duration::from_secs(5)))
        })
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| BlobErrorV1::Unavailable)?;
    let result = operation(channel);
    let restored = channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .and_then(|_| channel.inner_mut().set_nonblocking(true))
        .map_err(|_| BlobErrorV1::Unavailable);
    match (result, restored) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
    }
}

fn derived_reference_id(
    run_id: [u8; 16],
    source_receipt: [u8; 32],
    parser_identity: [u8; 32],
    derived_receipt: [u8; 32],
) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment-text-extraction.derived-blob.v1\0");
    digest.update(run_id);
    digest.update(source_receipt);
    digest.update(parser_identity);
    digest.update(derived_receipt);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn translation_source_reference_id(
    translation_run_id: [u8; 16],
    source_extraction_run_id: [u8; 16],
    source_revision: u64,
    receipt_sha256: [u8; 32],
) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment-text-extraction.translation-source.v1\0");
    digest.update(translation_run_id);
    digest.update(source_extraction_run_id);
    digest.update(source_revision.to_be_bytes());
    digest.update(receipt_sha256);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

pub(crate) struct TranslationSourceBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_size: u64,
    pub receipt_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

fn id16(value: &[u8]) -> Result<[u8; 16], BlobErrorV1> {
    let value: [u8; 16] = value.try_into().map_err(|_| BlobErrorV1::InvalidReceipt)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(BlobErrorV1::InvalidReceipt)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BlobErrorV1 {
    InvalidReceipt,
    Unavailable,
}
