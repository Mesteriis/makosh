#![forbid(unsafe_code)]

mod cleanup;
#[cfg(feature = "conformance-test-support")]
mod conformance;
mod delivery_results;
mod event_io;
mod event_outbox;
mod operations;
mod realtime;
pub mod schema;
mod source_queue;
mod work_queue;

pub use cleanup::{CrossChannelForwardCleanupJobV1, CrossChannelForwardCleanupReasonV1};
#[cfg(feature = "conformance-test-support")]
pub use conformance::CrossChannelForwardPersistenceConformanceV1;
pub use delivery_results::{
    CrossChannelForwardDeliveryRejectedEventV1, CrossChannelForwardDeliverySubmittedEventV1,
};
pub use event_io::{
    CrossChannelForwardBlobReceiptV1, CrossChannelForwardPreparedEventV1,
    CrossChannelForwardRejectedEventV1,
};
use makosh_storage_protocol::StorageBindingV1;
pub use operations::{
    CreateCrossChannelForwardOutcomeV1, CreateCrossChannelForwardV1,
    CrossChannelForwardStatusRecordV1,
};
pub use realtime::CrossChannelForwardClientRealtimeTransitionV1;
pub use source_queue::CrossChannelForwardSourcePrepareCandidateV1;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
pub use work_queue::{
    CrossChannelForwardClaimV1, CrossChannelForwardPreparedSourceV1,
    CrossChannelForwardWorkStageV1, FORWARD_WORK_LEASE_MILLIS_V1,
};

pub const PACKAGE: &str = "makosh-communication-cross-channel-forward-persistence";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    Conflict,
    ClaimLost,
    NotFound,
}

#[derive(Clone)]
pub struct CommunicationCrossChannelForwardPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl CommunicationCrossChannelForwardPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, CrossChannelForwardPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(CrossChannelForwardPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
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
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)
    }
}

pub(crate) fn valid_bounded_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

pub(crate) fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_timestamp(value: i64) -> bool {
    value > 0
}
