#![forbid(unsafe_code)]

mod creation;
mod execution;
mod realtime;
pub mod schema;
mod status;

pub use creation::{CreateBulkDeliveryOutcomeV1, CreateBulkDeliveryV1};
pub use execution::{
    BulkDeliveryTargetClaimV1, CompleteTargetOutcomeV1, MAX_TARGET_ATTEMPTS_V1,
    TARGET_LEASE_SECONDS_V1,
};
use makosh_storage_protocol::StorageBindingV1;
pub use realtime::BulkDeliveryClientRealtimeTransitionV1;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
pub use status::{
    BulkDeliveryBatchStateV1, BulkDeliveryStatusPageV1, BulkDeliveryTargetStateV1,
    BulkDeliveryTargetStatusV1,
};

pub const PACKAGE: &str = "makosh-communication-bulk-action-persistence";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkDeliveryPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    Conflict,
    ClaimLost,
    NotFound,
}

#[derive(Clone)]
pub struct CommunicationBulkActionPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl CommunicationBulkActionPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, BulkDeliveryPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(BulkDeliveryPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
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
            .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), BulkDeliveryPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)
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
