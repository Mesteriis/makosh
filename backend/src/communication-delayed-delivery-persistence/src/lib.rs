#![forbid(unsafe_code)]

mod cleanup;
#[cfg(feature = "conformance-test-support")]
mod conformance;
mod execution;
mod operations;
mod realtime;
mod relay;
pub mod schema;
mod status;

pub use cleanup::{DelayedDeliveryBodyCleanupJobV1, DelayedDeliveryBodyCleanupReasonV1};
#[cfg(feature = "conformance-test-support")]
pub use conformance::DelayedDeliveryPersistenceConformanceV1;
pub use execution::{
    ClaimDueExecutionOutcomeV1, ClaimDueExecutionV1, DelayedDeliveryExecutionClaimV1,
    MarkDeliveryAcceptedV1, MarkDeliveryFailedV1,
};
use makosh_storage_protocol::StorageBindingV1;
pub use operations::{
    ApplySchedulerResultOutcomeV1, ApplySchedulerResultV1, CreateDelayedDeliveryOperationOutcomeV1,
    CreateDelayedDeliveryOperationV1, DelayedDeliveryOperationStatusV1,
    RequestDelayedDeliveryCancellationV1, SchedulerScheduleResultV1,
};
pub use realtime::DelayedDeliveryClientRealtimeTransitionV1;
pub use relay::{DelayedDeliveryOutboxRecordV1, DelayedDeliveryOutboxStreamV1};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub const PACKAGE: &str = "makosh-communication-delayed-delivery-persistence";
pub const MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;
pub const MAX_DURABLE_ENVELOPE_BYTES_V1: usize = 128 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    Conflict,
    StaleRevision,
    ClaimLost,
    NotFound,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryBodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryDurableMessageV1 {
    pub message_id: [u8; 16],
    pub contract_kind: &'static str,
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerExecutionFenceV1 {
    pub run_id: [u8; 16],
    pub schedule_revision: u64,
    pub lease_epoch: u64,
    pub lease_expires_at_unix_millis: u64,
}

#[derive(Clone)]
pub struct CommunicationDelayedDeliveryPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl CommunicationDelayedDeliveryPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, DelayedDeliveryPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(DelayedDeliveryPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(port)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)
    }
}

pub fn valid_body_receipt(receipt: &DelayedDeliveryBodyReceiptV1) -> bool {
    valid_id16(&receipt.reference_id)
        && (1..=65_536).contains(&receipt.declared_bytes)
        && receipt.sha256.iter().any(|byte| *byte != 0)
        && !receipt.custody_proof.is_empty()
        && receipt.custody_proof.len() <= MAX_CUSTODY_PROOF_BYTES_V1
}

pub fn valid_durable_message(message: &DelayedDeliveryDurableMessageV1) -> bool {
    valid_id16(&message.message_id)
        && matches!(
            message.contract_kind,
            "scheduler.schedule.command.v1"
                | "communication.delayed_delivery.status_changed.v1"
                | "scheduler.job_run.acceptance.v1"
                | "scheduler.job_run.result.v1"
        )
        && message.envelope_sha256.iter().any(|byte| *byte != 0)
        && !message.envelope_bytes.is_empty()
        && message.envelope_bytes.len() <= MAX_DURABLE_ENVELOPE_BYTES_V1
}

pub fn valid_execution_fence(fence: &SchedulerExecutionFenceV1, now_unix_millis: u64) -> bool {
    valid_id16(&fence.run_id)
        && fence.schedule_revision > 0
        && fence.lease_epoch > 0
        && fence.lease_expires_at_unix_millis > now_unix_millis
}

pub(crate) fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_blob_receipt_without_accepting_plaintext() {
        let receipt = DelayedDeliveryBodyReceiptV1 {
            reference_id: [1; 16],
            declared_bytes: 12,
            sha256: [2; 32],
            custody_proof: vec![3; 64],
        };
        assert!(valid_body_receipt(&receipt));
    }

    #[test]
    fn durable_messages_are_exact_and_bounded() {
        let message = DelayedDeliveryDurableMessageV1 {
            message_id: [1; 16],
            contract_kind: "scheduler.schedule.command.v1",
            envelope_sha256: [2; 32],
            envelope_bytes: vec![3; 128],
        };
        assert!(valid_durable_message(&message));
    }

    #[test]
    fn execution_fence_requires_a_live_scheduler_lease() {
        let fence = SchedulerExecutionFenceV1 {
            run_id: [1; 16],
            schedule_revision: 2,
            lease_epoch: 3,
            lease_expires_at_unix_millis: 20_000,
        };
        assert!(valid_execution_fence(&fence, 19_999));
        assert!(!valid_execution_fence(&fence, 20_000));
    }
}
