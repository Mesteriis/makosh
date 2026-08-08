use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    ManagedBlobCustodyReleaseRequestV1, request_managed_blob_custody_release_v2,
};
use makosh_communication_delivery_intent_persistence::{
    CommunicationDeliveryIntentPersistenceV1, DeliveryIntentIngressCleanupJobV1,
    DeliveryIntentIngressCleanupReasonV1, DeliveryIntentPersistenceErrorV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::BlobCustodyReleaseReasonV1,
};

use crate::{
    admission::COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1,
    runtime::{DeliveryIntentManagedRuntimeV1, DeliveryIntentRuntimeErrorV1},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentIngressCleanupErrorV1 {
    InvalidJob,
    Persistence(DeliveryIntentPersistenceErrorV1),
    Unavailable,
}

pub trait DeliveryIntentIngressCustodyReleasePortV1 {
    fn release(
        &mut self,
        job: &DeliveryIntentIngressCleanupJobV1,
    ) -> Result<(), DeliveryIntentIngressCleanupErrorV1>;
}

pub struct ManagedDeliveryIntentIngressCustodyReleasePortV1<'a> {
    pub control_channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
}

impl DeliveryIntentIngressCustodyReleasePortV1
    for ManagedDeliveryIntentIngressCustodyReleasePortV1<'_>
{
    fn release(
        &mut self,
        job: &DeliveryIntentIngressCleanupJobV1,
    ) -> Result<(), DeliveryIntentIngressCleanupErrorV1> {
        let reason = match job.reason {
            DeliveryIntentIngressCleanupReasonV1::Submitted => {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
            }
            DeliveryIntentIngressCleanupReasonV1::Rejected => {
                BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
            }
        };
        request_managed_blob_custody_release_v2(
            self.control_channel,
            self.dispatcher,
            ManagedBlobCustodyReleaseRequestV1 {
                operation_id: &job.intent_id,
                capability_id: COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1,
                reference_id: &job.body_receipt.reference_id,
                declared_size: job.body_receipt.declared_bytes,
                receipt_sha256: &job.body_receipt.sha256,
                custody_source_proof: &job.body_receipt.custody_source_proof,
                reason,
            },
        )
        .map(|_| ())
        .map_err(|_| DeliveryIntentIngressCleanupErrorV1::Unavailable)
    }
}

pub async fn process_delivery_intent_ingress_cleanup_once_v1(
    persistence: &CommunicationDeliveryIntentPersistenceV1,
    logical_owner_id: &str,
    now_unix_seconds: i64,
    release_port: &mut dyn DeliveryIntentIngressCustodyReleasePortV1,
) -> Result<bool, DeliveryIntentIngressCleanupErrorV1> {
    let Some(job) = persistence
        .next_ingress_cleanup(logical_owner_id, now_unix_seconds)
        .await
        .map_err(DeliveryIntentIngressCleanupErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    match release_port.release(&job) {
        Ok(()) => persistence
            .complete_ingress_cleanup(logical_owner_id, &job.intent_id, now_unix_seconds)
            .await
            .map_err(DeliveryIntentIngressCleanupErrorV1::Persistence)?,
        Err(DeliveryIntentIngressCleanupErrorV1::Unavailable) => {
            let next_attempt = now_unix_seconds
                .checked_add(cleanup_backoff_seconds(job.attempt_count))
                .ok_or(DeliveryIntentIngressCleanupErrorV1::InvalidJob)?;
            persistence
                .reschedule_ingress_cleanup(
                    logical_owner_id,
                    &job.intent_id,
                    job.attempt_count,
                    next_attempt,
                    now_unix_seconds,
                )
                .await
                .map_err(DeliveryIntentIngressCleanupErrorV1::Persistence)?;
        }
        Err(error) => return Err(error),
    }
    Ok(true)
}

impl DeliveryIntentManagedRuntimeV1 {
    pub async fn process_ingress_cleanup_once_v1(
        &mut self,
        now_unix_seconds: i64,
    ) -> Result<bool, DeliveryIntentRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        let mut dispatcher =
            makosh_runtime_protocol::managed_control::RejectManagedControlRequestsV2;
        let result = {
            let mut release_port = ManagedDeliveryIntentIngressCustodyReleasePortV1 {
                control_channel: &mut self.control_channel,
                dispatcher: &mut dispatcher,
            };
            process_delivery_intent_ingress_cleanup_once_v1(
                &self.persistence,
                &self.logical_owner_id,
                now_unix_seconds,
                &mut release_port,
            )
            .await
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        result.map_err(|error| match error {
            DeliveryIntentIngressCleanupErrorV1::InvalidJob => {
                DeliveryIntentRuntimeErrorV1::EventContract
            }
            DeliveryIntentIngressCleanupErrorV1::Persistence(error) => {
                DeliveryIntentRuntimeErrorV1::Persistence(error)
            }
            DeliveryIntentIngressCleanupErrorV1::Unavailable => {
                DeliveryIntentRuntimeErrorV1::Unavailable
            }
        })
    }
}

const fn cleanup_backoff_seconds(attempt_count: u16) -> i64 {
    1_i64 << if attempt_count > 6 { 6 } else { attempt_count }
}

#[cfg(test)]
mod tests {
    use super::cleanup_backoff_seconds;

    #[test]
    fn cleanup_backoff_is_bounded() {
        assert_eq!(cleanup_backoff_seconds(0), 1);
        assert_eq!(cleanup_backoff_seconds(6), 64);
        assert_eq!(cleanup_backoff_seconds(32), 64);
    }
}
