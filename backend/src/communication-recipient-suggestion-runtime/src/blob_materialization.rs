use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyReleaseRequestV1, ManagedBlobCustodyTransferRequestV1,
    ManagedBlobSessionRequestV1, request_managed_blob_custody_release_v2,
    request_managed_blob_custody_transfer_v2, request_managed_blob_session_v2,
};
use makosh_communication_recipient_suggestion_persistence::CommunicationRecipientSuggestionBlobCleanupV1;
use makosh_communications_recipient_source_api::COMMUNICATION_RECIPIENT_SOURCE_MAX_BYTES_V1;
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobCustodyReleaseReasonV1, BlobDataOperationV1},
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::COMMUNICATION_RECIPIENT_SUGGESTION_BLOB_CAPABILITY_ID_V1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationRecipientSuggestionSourceBlobReceiptV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

pub(crate) struct CommunicationRecipientSuggestionBlobMaterializationV1 {
    pub body_utf8: Zeroizing<Vec<u8>>,
    pub source_cleanup: CommunicationRecipientSuggestionBlobCleanupV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionBlobErrorV1 {
    InvalidReceipt,
    Unavailable,
}

pub(crate) fn materialize_recipient_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source: &CommunicationRecipientSuggestionSourceBlobReceiptV1,
) -> Result<
    CommunicationRecipientSuggestionBlobMaterializationV1,
    CommunicationRecipientSuggestionBlobErrorV1,
> {
    validate_source(source)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: COMMUNICATION_RECIPIENT_SUGGESTION_BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source.reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source.sha256,
            custody_source_proof: &source.custody_proof,
            evidence_id: &source.result_message_id,
            evidence_envelope_sha256: &source.envelope_sha256,
        },
    )
    .map_err(|_| CommunicationRecipientSuggestionBlobErrorV1::Unavailable)?;
    let local_reference = id16(&transfer.grant.target_reference_id)?;
    BlobDataClient::new(&transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(|_| CommunicationRecipientSuggestionBlobErrorV1::Unavailable)?;
    let body_utf8 = read_exact(
        channel,
        dispatcher,
        &local_reference,
        source.declared_bytes,
        &source.sha256,
    )?;
    Ok(CommunicationRecipientSuggestionBlobMaterializationV1 {
        body_utf8,
        source_cleanup: CommunicationRecipientSuggestionBlobCleanupV1 {
            reference_id: local_reference,
            declared_bytes: source.declared_bytes,
            sha256: source.sha256,
            custody_proof: source.custody_proof.clone(),
        },
    })
}

pub(crate) fn release_recipient_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    source_cleanup: &CommunicationRecipientSuggestionBlobCleanupV1,
    accepted: bool,
) -> Result<(), CommunicationRecipientSuggestionBlobErrorV1> {
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
            capability_id: COMMUNICATION_RECIPIENT_SUGGESTION_BLOB_CAPABILITY_ID_V1,
            reference_id: &source_cleanup.reference_id,
            declared_size: source_cleanup.declared_bytes,
            receipt_sha256: &source_cleanup.sha256,
            custody_source_proof: &source_cleanup.custody_proof,
            reason,
        },
    )
    .map(|_| ())
    .map_err(|_| CommunicationRecipientSuggestionBlobErrorV1::Unavailable)
}

pub(crate) fn read_recipient_source_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source_cleanup: &CommunicationRecipientSuggestionBlobCleanupV1,
) -> Result<Zeroizing<Vec<u8>>, CommunicationRecipientSuggestionBlobErrorV1> {
    read_exact(
        channel,
        dispatcher,
        &source_cleanup.reference_id,
        source_cleanup.declared_bytes,
        &source_cleanup.sha256,
    )
}

fn read_exact(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    reference_id: &[u8; 16],
    declared_bytes: u64,
    sha256: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, CommunicationRecipientSuggestionBlobErrorV1> {
    let read = request_managed_blob_session_v2(
        channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATION_RECIPIENT_SUGGESTION_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(sha256),
            custody_target: None,
        },
    )
    .map_err(|_| CommunicationRecipientSuggestionBlobErrorV1::Unavailable)?;
    let bytes = Zeroizing::new(
        BlobDataClient::new(read.data_socket_path)
            .and_then(|client| {
                client.read_range(read.grant, read.channel_binding, 0, declared_bytes)
            })
            .map_err(|_| CommunicationRecipientSuggestionBlobErrorV1::Unavailable)?,
    );
    if bytes.len() != usize::try_from(declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(bytes.as_slice()).as_slice() != sha256
    {
        return Err(CommunicationRecipientSuggestionBlobErrorV1::InvalidReceipt);
    }
    Ok(bytes)
}

fn validate_source(
    source: &CommunicationRecipientSuggestionSourceBlobReceiptV1,
) -> Result<(), CommunicationRecipientSuggestionBlobErrorV1> {
    if source.result_message_id.iter().all(|byte| *byte == 0)
        || source.envelope_sha256.iter().all(|byte| *byte == 0)
        || source.reference_id.iter().all(|byte| *byte == 0)
        || !(1..=COMMUNICATION_RECIPIENT_SOURCE_MAX_BYTES_V1).contains(&source.declared_bytes)
        || source.sha256.iter().all(|byte| *byte == 0)
        || source.custody_proof.is_empty()
        || source.custody_proof.len() > 2_048
    {
        return Err(CommunicationRecipientSuggestionBlobErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationRecipientSuggestionBlobErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| CommunicationRecipientSuggestionBlobErrorV1::InvalidReceipt)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(CommunicationRecipientSuggestionBlobErrorV1::InvalidReceipt)
}

fn release_operation_id(run_id: [u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_recipient_suggestion.release_source.v1\0");
    digest.update(run_id);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}
