use makosh_events_api::{EventEnvelope, NewEventEnvelope};
use makosh_signal_hub_postgres::raw_signals::adapter::RawSignalStore;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use std::sync::Arc;

use super::service::signal_hub_raw_dispatcher_allows_processing;
use super::service::{SignalHubSignalService, SignalProcessingOutcome};
use super::store::{SignalHubError, SignalHubStore};
use makosh_communications_api::evidence::StoredRawCommunicationRecord;
use makosh_events_postgres::store::EventStore;
use makosh_observations_postgres::store::observation_captured_event_id;
use makosh_signal_hub_api::raw_signals::{
    ProviderRawSignalInput, ProviderRawSignalPort, ProviderRawSignalPortFuture, RawSignalPortError,
};

#[derive(Clone)]
pub struct PostgresZulipSignalDispatch {
    pool: PgPool,
}

impl PostgresZulipSignalDispatch {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ProviderRawSignalPort for PostgresZulipSignalDispatch {
    fn dispatch_provider_record<'a>(
        &'a self,
        record: &'a ProviderRawSignalInput,
    ) -> ProviderRawSignalPortFuture<'a> {
        Box::pin(async move {
            let raw_record = StoredRawCommunicationRecord {
                raw_record_id: record.raw_record_id.clone(),
                observation_id: record.observation_id.clone(),
                account_id: record.account_id.clone(),
                record_kind: record.record_kind.clone(),
                provider_record_id: record.provider_record_id.clone(),
                source_fingerprint: record.source_fingerprint.clone(),
                import_batch_id: record.import_batch_id.clone(),
                occurred_at: record.occurred_at,
                captured_at: record.captured_at,
                payload: record.payload.clone(),
                provenance: record.provenance.clone(),
            };
            dispatch_zulip_raw_signal_with_pool(self.pool.clone(), &raw_record)
                .await
                .map_err(RawSignalPortError::new)
        })
    }
}

pub async fn dispatch_zulip_raw_signal(
    pool: PgPool,
    raw_record: &StoredRawCommunicationRecord,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    dispatch_zulip_raw_signal_with_pool(pool, raw_record).await
}

async fn dispatch_zulip_raw_signal_with_pool(
    pool: PgPool,
    raw_record: &StoredRawCommunicationRecord,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let raw_signal = build_zulip_raw_signal(raw_record)?;
    event_store
        .append_for_dispatch_idempotent(&raw_signal)
        .await?;

    let raw_event = event_store
        .get_by_id(&raw_signal.event_id)
        .await?
        .ok_or_else(|| SignalHubError::InvalidRawSignalEventType(raw_signal.event_type.clone()))?;

    let signal_store = SignalHubStore::new(pool.clone());
    if !signal_hub_raw_dispatcher_allows_processing(&signal_store).await? {
        return Ok(None);
    }

    let service =
        SignalHubSignalService::new(Arc::new(RawSignalStore::new(pool)), event_store.clone());
    match service.process_raw_signal(&raw_event).await? {
        SignalProcessingOutcome::Accepted { event_id } => {
            Ok(event_store.get_by_id(&event_id).await?)
        }
        SignalProcessingOutcome::Rejected { .. }
        | SignalProcessingOutcome::Muted { .. }
        | SignalProcessingOutcome::Paused { .. } => Ok(None),
    }
}

fn build_zulip_raw_signal(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewEventEnvelope, SignalHubError> {
    let occurred_at = raw_record.occurred_at.unwrap_or(raw_record.captured_at);
    let provider_event_type = raw_record
        .payload
        .get("provider_event_type")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let event_kind = zulip_signal_event_kind(provider_event_type);
    let source = json!({
        "kind": "signal_source",
        "source_code": "zulip",
        "source_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
    });
    let subject = json!({
        "kind": "communication_raw_record",
        "source_code": "zulip",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
    });
    let provenance = json!({
        "source": "communications_raw_record",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
        "import_batch_id": raw_record.import_batch_id,
        "raw_record_provenance": raw_record.provenance,
    });

    Ok(NewEventEnvelope::builder(
        zulip_raw_signal_event_id(&raw_record.raw_record_id),
        format!("signal.raw.zulip.{event_kind}.observed"),
        occurred_at,
        source,
        subject,
    )
    .payload(raw_record.payload.clone())
    .provenance(provenance)
    .causation_id(observation_captured_event_id(&raw_record.observation_id))
    .correlation_id(raw_record.observation_id.clone())
    .build()?)
}

fn zulip_raw_signal_event_id(raw_record_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_record_id.as_bytes());
    format!("evt_signal_raw_zulip_{:x}", hasher.finalize())
}

fn zulip_signal_event_kind(provider_event_type: &str) -> &'static str {
    match provider_event_type {
        "message" => "message",
        "reaction" => "reaction",
        "update_message" => "message_update",
        "delete_message" => "message_delete",
        _ => "unknown",
    }
}
