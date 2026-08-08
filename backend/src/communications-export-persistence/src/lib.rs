#![forbid(unsafe_code)]

mod jobs;
mod realtime;
pub mod schema;

pub use jobs::{
    CommunicationsExportArtifactReceiptV1, CommunicationsExportClaimV1,
    CommunicationsExportJobStatusV1, CommunicationsExportPreparedItemV1,
    CommunicationsExportSourceReceiptV1,
};
use makosh_storage_protocol::StorageBindingV1;
pub use realtime::CommunicationsExportRealtimeTransitionV1;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub const PACKAGE: &str = "makosh-communications-export-persistence";

#[derive(Clone)]
pub struct CommunicationsExportPersistenceV1 {
    pub(crate) pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsExportPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    Conflict,
    ClaimLost,
}

impl CommunicationsExportPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, CommunicationsExportPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(CommunicationsExportPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
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
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), CommunicationsExportPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)
    }
}

pub(crate) fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_timestamp(value: i64) -> bool {
    value > 0
}
