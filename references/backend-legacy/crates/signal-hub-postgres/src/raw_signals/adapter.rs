use makosh_events_api::EventEnvelope;
use makosh_signal_hub_api::policies::SignalPolicy;
use makosh_signal_hub_api::raw_signals::{
    RawSignalPersistenceError, RawSignalPersistenceFuture, RawSignalPersistencePort,
};
use sqlx::postgres::PgPool;

use super::{connection_lookup, paused_events, policies};

#[derive(Clone)]
pub struct RawSignalStore {
    pool: PgPool,
}

impl RawSignalStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn resolve_connection_id(
        &self,
        source_code: &str,
        event: &EventEnvelope,
    ) -> Result<Option<String>, RawSignalPersistenceError> {
        connection_lookup::resolve_connection_id(&self.pool, source_code, event).await
    }

    pub async fn list_active_policies(
        &self,
    ) -> Result<Vec<SignalPolicy>, RawSignalPersistenceError> {
        policies::list_active(&self.pool).await
    }

    pub async fn record_paused_event(
        &self,
        event: &EventEnvelope,
        source_code: &str,
        connection_id: Option<&str>,
        reason: &str,
    ) -> Result<(), RawSignalPersistenceError> {
        paused_events::record(&self.pool, event, source_code, connection_id, reason).await
    }
}

impl RawSignalPersistencePort for RawSignalStore {
    fn resolve_connection_id<'a>(
        &'a self,
        source_code: &'a str,
        event: &'a EventEnvelope,
    ) -> RawSignalPersistenceFuture<'a, Option<String>> {
        Box::pin(async move { self.resolve_connection_id(source_code, event).await })
    }

    fn list_active_policies<'a>(&'a self) -> RawSignalPersistenceFuture<'a, Vec<SignalPolicy>> {
        Box::pin(async move { self.list_active_policies().await })
    }

    fn record_paused_event<'a>(
        &'a self,
        event: &'a EventEnvelope,
        source_code: &'a str,
        connection_id: Option<&'a str>,
        reason: &'a str,
    ) -> RawSignalPersistenceFuture<'a, ()> {
        Box::pin(async move {
            self.record_paused_event(event, source_code, connection_id, reason)
                .await
        })
    }
}
