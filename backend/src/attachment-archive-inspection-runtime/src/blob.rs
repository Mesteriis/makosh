//! One-use, receipt-bound Blob transfer and read for a claimed archive job.

use std::{os::unix::net::UnixStream, time::Duration};

use makosh_attachment_archive_inspection_persistence::{
    ArchiveInspectionTargetBlobReceiptV1, ClaimedArchiveInspectionJobV1,
};
use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_transfer_v2, request_managed_blob_session_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::BlobDataOperationV1,
};

use crate::admission::ATTACHMENT_ARCHIVE_INSPECTION_BLOB_CAPABILITY_ID;

pub fn transfer_archive_blob_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedArchiveInspectionJobV1,
) -> Result<ArchiveInspectionTargetBlobReceiptV1, ArchiveInspectionBlobErrorV1> {
    prepare(channel)?;
    let result = (|| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let transfer = request_managed_blob_custody_transfer_v2(
            channel,
            &mut dispatcher,
            ManagedBlobCustodyTransferRequestV1 {
                capability_id: ATTACHMENT_ARCHIVE_INSPECTION_BLOB_CAPABILITY_ID,
                source_reference_id: &claimed.source_reference_id,
                declared_size: claimed.declared_size,
                receipt_sha256: &claimed.blob_receipt_sha256,
                custody_source_proof: &claimed.custody_transfer_source_proof,
                evidence_id: &claimed.delegation_result_message_id,
                evidence_envelope_sha256: &claimed.delegation_result_envelope_sha256,
            },
        )
        .map_err(|_| ArchiveInspectionBlobErrorV1::Unavailable)?;
        let reference_id = transfer
            .grant
            .target_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| ArchiveInspectionBlobErrorV1::InvalidEvidence)?;
        BlobDataClient::new(transfer.data_socket_path)
            .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
            .map_err(|_| ArchiveInspectionBlobErrorV1::Unavailable)?;
        Ok(ArchiveInspectionTargetBlobReceiptV1 {
            reference_id,
            receipt_sha256: claimed.blob_receipt_sha256,
        })
    })();
    finish(channel, result)
}

pub fn read_archive_blob_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedArchiveInspectionJobV1,
    receipt: ArchiveInspectionTargetBlobReceiptV1,
) -> Result<Vec<u8>, ArchiveInspectionBlobErrorV1> {
    prepare(channel)?;
    let result = (|| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let session = request_managed_blob_session_v2(
            channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: ATTACHMENT_ARCHIVE_INSPECTION_BLOB_CAPABILITY_ID,
                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                reference_id: &receipt.reference_id,
                declared_size: claimed.declared_size,
                backup_class: 1,
                receipt_sha256: Some(&receipt.receipt_sha256),
                custody_target: None,
            },
        )
        .map_err(|_| ArchiveInspectionBlobErrorV1::Unavailable)?;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(
                    session.grant,
                    session.channel_binding,
                    0,
                    claimed.declared_size,
                )
            })
            .map_err(|_| ArchiveInspectionBlobErrorV1::Unavailable)
    })();
    finish(channel, result)
}

fn prepare(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<(), ArchiveInspectionBlobErrorV1> {
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
        .map_err(|_| ArchiveInspectionBlobErrorV1::Unavailable)
}

fn finish<T>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    result: Result<T, ArchiveInspectionBlobErrorV1>,
) -> Result<T, ArchiveInspectionBlobErrorV1> {
    let restored = channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .and_then(|_| channel.inner_mut().set_nonblocking(true))
        .map_err(|_| ArchiveInspectionBlobErrorV1::Unavailable);
    match (result, restored) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionBlobErrorV1 {
    InvalidEvidence,
    Unavailable,
}
