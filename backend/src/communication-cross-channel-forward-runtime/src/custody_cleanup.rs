use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    ManagedBlobCustodyReleaseRequestV1, request_managed_blob_custody_release_v2,
};
use makosh_communication_cross_channel_forward_persistence::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardCleanupJobV1,
    CrossChannelForwardCleanupReasonV1, CrossChannelForwardPersistenceErrorV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::BlobCustodyReleaseReasonV1,
};

use crate::{
    COMMUNICATION_CROSS_CHANNEL_FORWARD_BLOB_CAPABILITY_ID_V1,
    CrossChannelForwardBlobTransferErrorV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardCustodyCleanupErrorV1 {
    Blob(CrossChannelForwardBlobTransferErrorV1),
    Persistence(CrossChannelForwardPersistenceErrorV1),
}

pub trait CrossChannelForwardCustodyReleasePortV1 {
    fn release(
        &mut self,
        job: &CrossChannelForwardCleanupJobV1,
    ) -> Result<(), CrossChannelForwardBlobTransferErrorV1>;
}

pub struct ManagedCrossChannelForwardCustodyReleasePortV1<'a> {
    pub control_channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
}

impl CrossChannelForwardCustodyReleasePortV1
    for ManagedCrossChannelForwardCustodyReleasePortV1<'_>
{
    fn release(
        &mut self,
        job: &CrossChannelForwardCleanupJobV1,
    ) -> Result<(), CrossChannelForwardBlobTransferErrorV1> {
        let reason = match job.reason {
            CrossChannelForwardCleanupReasonV1::DeliveryAccepted => {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
            }
            CrossChannelForwardCleanupReasonV1::Rejected => {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
            }
        };
        request_managed_blob_custody_release_v2(
            self.control_channel,
            self.dispatcher,
            ManagedBlobCustodyReleaseRequestV1 {
                operation_id: &job.forward_id,
                capability_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_BLOB_CAPABILITY_ID_V1,
                reference_id: &job.blob_reference,
                declared_size: job.declared_bytes,
                receipt_sha256: &job.sha256,
                custody_source_proof: &job.custody_proof,
                reason,
            },
        )
        .map(|_| ())
        .map_err(|_| CrossChannelForwardBlobTransferErrorV1::Unavailable)
    }
}

pub async fn process_cross_channel_custody_cleanup_once_v1(
    persistence: &CommunicationCrossChannelForwardPersistenceV1,
    logical_owner_id: &str,
    now_unix_millis: i64,
    release_port: &mut dyn CrossChannelForwardCustodyReleasePortV1,
) -> Result<bool, CrossChannelForwardCustodyCleanupErrorV1> {
    let Some(job) = persistence
        .next_cleanup(logical_owner_id, now_unix_millis)
        .await
        .map_err(CrossChannelForwardCustodyCleanupErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    match release_port.release(&job) {
        Ok(()) => persistence
            .complete_cleanup(logical_owner_id, &job.forward_id, now_unix_millis)
            .await
            .map_err(CrossChannelForwardCustodyCleanupErrorV1::Persistence)?,
        Err(CrossChannelForwardBlobTransferErrorV1::Unavailable) => {
            let next_attempt = now_unix_millis
                .checked_add(cleanup_backoff_millis(job.attempt_count))
                .ok_or(CrossChannelForwardCustodyCleanupErrorV1::Blob(
                    CrossChannelForwardBlobTransferErrorV1::InvalidReceipt,
                ))?;
            persistence
                .reschedule_cleanup(
                    logical_owner_id,
                    &job.forward_id,
                    job.attempt_count,
                    next_attempt,
                    now_unix_millis,
                )
                .await
                .map_err(CrossChannelForwardCustodyCleanupErrorV1::Persistence)?;
        }
        Err(error) => return Err(CrossChannelForwardCustodyCleanupErrorV1::Blob(error)),
    }
    Ok(true)
}

const fn cleanup_backoff_millis(attempt_count: u16) -> i64 {
    250 * (1_i64 << if attempt_count > 6 { 6 } else { attempt_count })
}

#[cfg(test)]
mod tests {
    use super::cleanup_backoff_millis;

    #[test]
    fn cleanup_backoff_is_bounded() {
        assert_eq!(cleanup_backoff_millis(0), 250);
        assert_eq!(cleanup_backoff_millis(6), 16_000);
        assert_eq!(cleanup_backoff_millis(32), 16_000);
    }
}
