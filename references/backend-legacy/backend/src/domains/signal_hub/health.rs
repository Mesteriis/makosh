use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;

use super::store::{
    SignalHealth, SignalHealthCheckRequest, SignalHealthSnapshotWrite, SignalHubError,
    SignalHubStore,
};
use makosh_events_postgres::store::EventStore;

#[derive(Clone)]
pub struct SignalHubHealthService {
    signal_store: SignalHubStore,
    event_store: EventStore,
}

impl SignalHubHealthService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        Self {
            signal_store,
            event_store,
        }
    }

    pub async fn run_health_check(
        &self,
        request: &SignalHealthCheckRequest,
    ) -> Result<SignalHealth, SignalHubError> {
        let health = self.signal_store.run_health_check(request).await?;
        self.append_health_changed_event(&health).await?;
        Ok(health)
    }

    pub async fn record_snapshot(
        &self,
        request: &SignalHealthCheckRequest,
        snapshot: SignalHealthSnapshotWrite,
    ) -> Result<SignalHealth, SignalHubError> {
        let health = self
            .signal_store
            .upsert_health_snapshot(request, snapshot)
            .await?;
        self.append_health_changed_event(&health).await?;
        Ok(health)
    }

    async fn append_health_changed_event(
        &self,
        health: &SignalHealth,
    ) -> Result<(), SignalHubError> {
        let event = NewEventEnvelope::builder(
            format!(
                "evt_signal_health_changed_{}_{}",
                health.id,
                Utc::now().timestamp_nanos_opt().unwrap_or(0)
            ),
            "signal.source.health_changed",
            Utc::now(),
            json!({
                "kind": "signal_source",
                "source_code": health.source_code,
                "source_id": health.connection_id.clone().unwrap_or_else(|| health.source_code.clone()),
            }),
            json!({
                "kind": "signal_health",
                "entity_id": health.id,
                "source_code": health.source_code,
                "connection_id": health.connection_id,
            }),
        )
        .payload(json!({
            "level": health.level,
            "summary": health.summary,
            "failure_count": health.failure_count,
            "consecutive_failure_count": health.consecutive_failure_count,
            "next_retry_at": health.next_retry_at,
        }))
        .provenance(json!({
            "source": "signal_hub_health_service",
            "source_code": health.source_code,
            "connection_id": health.connection_id,
        }))
        .build()?;
        self.event_store.append_for_dispatch(&event).await?;
        Ok(())
    }
}
