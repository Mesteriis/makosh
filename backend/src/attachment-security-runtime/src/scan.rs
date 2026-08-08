//! Receipt-bound Blob materialization and loopback ClamAV scan adapter.

use std::{os::unix::net::UnixStream, time::Duration};

use makosh_attachment_security_clamav::{
    ClamAvInstreamLimitsV1, ClamAvLoopbackEndpointV1, ClamAvTimeoutsV1, scan_clamav_loopback_v1,
};
use makosh_attachment_security_core::ScannerOutcomeV1;
use makosh_attachment_security_persistence::{
    AttachmentSecurityTargetBlobReceiptV1, ClaimedAttachmentSecurityScanJobV1,
};
use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_transfer_v2, request_managed_blob_session_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::BlobDataOperationV1,
};

use crate::{
    admission::ATTACHMENT_SECURITY_BLOB_CAPABILITY_ID,
    settings::AttachmentSecurityRuntimeSettingsV1,
};

const CLAMAV_CHUNK_BYTES: u32 = 64 * 1024;
const CLAMAV_MAX_RESPONSE_BYTES: u32 = 4 * 1024;

pub struct AttachmentSecurityScannerV1 {
    endpoint: ClamAvLoopbackEndpointV1,
    limits: ClamAvInstreamLimitsV1,
    timeouts: ClamAvTimeoutsV1,
}

impl AttachmentSecurityScannerV1 {
    pub fn new(
        settings: AttachmentSecurityRuntimeSettingsV1,
    ) -> Result<Self, AttachmentSecurityScanAdapterErrorV1> {
        Ok(Self {
            endpoint: ClamAvLoopbackEndpointV1::new(settings.clamav_port)
                .map_err(|_| AttachmentSecurityScanAdapterErrorV1::InvalidConfiguration)?,
            limits: ClamAvInstreamLimitsV1::new(
                settings.max_scan_bytes,
                CLAMAV_CHUNK_BYTES,
                CLAMAV_MAX_RESPONSE_BYTES,
            )
            .map_err(|_| AttachmentSecurityScanAdapterErrorV1::InvalidConfiguration)?,
            timeouts: ClamAvTimeoutsV1::new(
                Duration::from_millis(settings.clamav_connect_timeout_millis),
                Duration::from_millis(settings.clamav_io_timeout_millis),
            )
            .map_err(|_| AttachmentSecurityScanAdapterErrorV1::InvalidConfiguration)?,
        })
    }

    pub fn scan_claimed(
        &self,
        control_channel: &mut ManagedControlChannelV2<UnixStream>,
        claimed: &ClaimedAttachmentSecurityScanJobV1,
        target_blob: AttachmentSecurityTargetBlobReceiptV1,
    ) -> Result<ScannerOutcomeV1, AttachmentSecurityScanAdapterErrorV1> {
        let bytes = read_blob(control_channel, claimed, target_blob)?;
        scan_clamav_loopback_v1(
            self.endpoint,
            &bytes,
            claimed.job.declared_size,
            target_blob.receipt_sha256,
            self.limits,
            self.timeouts,
        )
        .map_err(|_| AttachmentSecurityScanAdapterErrorV1::Scanner)
    }

    pub fn transfer_claimed_blob(
        &self,
        control_channel: &mut ManagedControlChannelV2<UnixStream>,
        claimed: &ClaimedAttachmentSecurityScanJobV1,
    ) -> Result<AttachmentSecurityTargetBlobReceiptV1, AttachmentSecurityScanAdapterErrorV1> {
        transfer_blob_custody(control_channel, claimed)
    }
}

fn read_blob(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedAttachmentSecurityScanJobV1,
    target_blob: AttachmentSecurityTargetBlobReceiptV1,
) -> Result<Vec<u8>, AttachmentSecurityScanAdapterErrorV1> {
    if prepare_blocking_control_channel(control_channel).is_err() {
        let _ = restore_nonblocking_control_channel(control_channel);
        return Err(AttachmentSecurityScanAdapterErrorV1::ControlChannel);
    }
    let result: Result<Vec<u8>, AttachmentSecurityScanAdapterErrorV1> = (|| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let session = request_managed_blob_session_v2(
            control_channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: ATTACHMENT_SECURITY_BLOB_CAPABILITY_ID,
                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                reference_id: &target_blob.reference_id,
                declared_size: claimed.job.declared_size,
                backup_class: 1,
                receipt_sha256: Some(&target_blob.receipt_sha256),
                custody_target: None,
            },
        )
        .map_err(|_| AttachmentSecurityScanAdapterErrorV1::BlobReadGrant)?;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(
                    session.grant,
                    session.channel_binding,
                    0,
                    claimed.job.declared_size,
                )
            })
            .map_err(|_| AttachmentSecurityScanAdapterErrorV1::BlobReadDataPlane)
    })();
    let restored = restore_nonblocking_control_channel(control_channel);
    match (result, restored) {
        (Ok(bytes), Ok(())) => Ok(bytes),
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
    }
}

fn transfer_blob_custody(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedAttachmentSecurityScanJobV1,
) -> Result<AttachmentSecurityTargetBlobReceiptV1, AttachmentSecurityScanAdapterErrorV1> {
    if prepare_blocking_control_channel(control_channel).is_err() {
        let _ = restore_nonblocking_control_channel(control_channel);
        return Err(AttachmentSecurityScanAdapterErrorV1::ControlChannel);
    }
    let result: Result<
        AttachmentSecurityTargetBlobReceiptV1,
        AttachmentSecurityScanAdapterErrorV1,
    > = (|| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let transfer = request_managed_blob_custody_transfer_v2(
            control_channel,
            &mut dispatcher,
            ManagedBlobCustodyTransferRequestV1 {
                capability_id: ATTACHMENT_SECURITY_BLOB_CAPABILITY_ID,
                source_reference_id: &claimed.job.blob_reference_id,
                declared_size: claimed.job.declared_size,
                receipt_sha256: &claimed.job.blob_receipt_sha256,
                custody_source_proof: &claimed.custody_transfer_source_proof,
                evidence_id: &claimed.job.candidate_message_id,
                evidence_envelope_sha256: &claimed.candidate_envelope_sha256,
            },
        )
        .map_err(|_| AttachmentSecurityScanAdapterErrorV1::CustodyGrant)?;
        let reference_id: [u8; 16] = transfer
            .grant
            .target_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| AttachmentSecurityScanAdapterErrorV1::CustodyGrant)?;
        BlobDataClient::new(transfer.data_socket_path)
            .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
            .map_err(|_| AttachmentSecurityScanAdapterErrorV1::CustodyDataPlane)?;
        Ok(AttachmentSecurityTargetBlobReceiptV1 {
            reference_id,
            receipt_sha256: claimed.job.blob_receipt_sha256,
        })
    })();
    let restored = restore_nonblocking_control_channel(control_channel);
    match (result, restored) {
        (Ok(receipt), Ok(())) => Ok(receipt),
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
    }
}

fn prepare_blocking_control_channel(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<(), AttachmentSecurityScanAdapterErrorV1> {
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
        .map_err(|_| AttachmentSecurityScanAdapterErrorV1::ControlChannel)
}

fn restore_nonblocking_control_channel(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<(), AttachmentSecurityScanAdapterErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .and_then(|_| channel.inner_mut().set_nonblocking(true))
        .map_err(|_| AttachmentSecurityScanAdapterErrorV1::ControlChannel)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityScanAdapterErrorV1 {
    InvalidConfiguration,
    ControlChannel,
    CustodyGrant,
    CustodyDataPlane,
    BlobReadGrant,
    BlobReadDataPlane,
    Scanner,
}
