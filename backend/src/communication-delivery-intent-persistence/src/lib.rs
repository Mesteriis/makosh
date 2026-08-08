#![forbid(unsafe_code)]

mod client_realtime;
#[cfg(feature = "conformance-test-support")]
mod conformance;
mod ingress_cleanup;
mod ingress_events;
mod intents;
mod provider_events;
pub mod schema;

pub use client_realtime::DeliveryIntentClientRealtimeTransitionV1;
#[cfg(feature = "conformance-test-support")]
pub use conformance::DeliveryIntentPersistenceConformanceV1;
pub use ingress_cleanup::{
    DeliveryIntentIngressCleanupJobV1, DeliveryIntentIngressCleanupReasonV1,
};
pub use ingress_events::{
    DeliveryIntentIngressBlobReceiptV1, DeliveryIntentIngressDispositionV1,
    DeliveryIntentIngressEventV1, DeliveryIntentIngressResultKindV1,
};
pub use intents::{
    CreateDeliveryIntentOutcomeV1, CreateDeliveryIntentV1, DeliveryIntentBodyBlobReceiptV1,
    DeliveryIntentClaimV1, DeliveryIntentPersistenceErrorV1, DeliveryIntentStateV1,
    DeliveryIntentStatusRecordV1,
};
use makosh_storage_protocol::StorageBindingV1;
pub use provider_events::{
    ApplyTerminalDeliveryResultOutcomeV1, EnqueueProviderCommandOutcomeV1,
    ProviderCommandOutboxEntryV1, TerminalDeliveryResultV1, TerminalDeliveryResultValueV1,
};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub const PACKAGE: &str = "makosh-communication-delivery-intent-persistence";

#[derive(Clone)]
pub struct CommunicationDeliveryIntentPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl CommunicationDeliveryIntentPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, DeliveryIntentPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(DeliveryIntentPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
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
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), DeliveryIntentPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)
    }
}

pub(crate) fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_id32(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_timestamp(value: i64) -> bool {
    value > 0
}

pub(crate) fn valid_bounded_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}
