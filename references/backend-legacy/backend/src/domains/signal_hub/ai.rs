use chrono::Utc;
use makosh_events_api::{EventEnvelope, NewEventEnvelope};
use makosh_signal_hub_postgres::raw_signals::adapter::RawSignalStore;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use std::sync::Arc;

use super::service::signal_hub_raw_dispatcher_allows_processing;
use super::service::{SignalHubSignalService, SignalProcessingOutcome};
use super::store::{SignalHubError, SignalHubStore};
use makosh_events_postgres::store::EventStore;

pub async fn dispatch_ai_helper_signal(
    pool: PgPool,
    event_kind: &str,
    source_id: &str,
    subject: Value,
    payload: Value,
    provenance: Value,
    correlation_id: Option<&str>,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let signal_store = SignalHubStore::new(pool.clone());
    signal_store.restore_system_sources().await?;
    let raw_signal = build_ai_helper_signal(
        event_kind,
        source_id,
        subject,
        payload,
        provenance,
        correlation_id,
    )?;
    let inserted = event_store
        .append_for_dispatch_idempotent(&raw_signal)
        .await?;

    let raw_event = match inserted {
        Some(_) => event_store.get_by_id(&raw_signal.event_id).await?,
        None => {
            event_store
                .get_by_source_idempotency(&raw_signal.event_type, "signal_source", None, source_id)
                .await?
        }
    }
    .ok_or_else(|| SignalHubError::InvalidRawSignalEventType(raw_signal.event_type.clone()))?;

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

fn build_ai_helper_signal(
    event_kind: &str,
    source_id: &str,
    subject: Value,
    payload: Value,
    provenance: Value,
    correlation_id: Option<&str>,
) -> Result<NewEventEnvelope, SignalHubError> {
    let builder = NewEventEnvelope::builder(
        ai_helper_raw_signal_event_id(event_kind, source_id),
        format!("signal.raw.ai.{event_kind}.observed"),
        Utc::now(),
        json!({
            "kind": "signal_source",
            "source_code": "ai",
            "source_id": source_id,
        }),
        subject,
    )
    .payload(payload)
    .provenance(provenance);

    let builder = match correlation_id {
        Some(value) if !value.trim().is_empty() => builder.correlation_id(value.to_owned()),
        _ => builder,
    };

    Ok(builder.build()?)
}

fn ai_helper_raw_signal_event_id(event_kind: &str, source_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(event_kind.as_bytes());
    hasher.update([0]);
    hasher.update(source_id.as_bytes());
    format!("evt_signal_raw_ai_{:x}", hasher.finalize())
}

pub async fn dispatch_ai_helper_signal_best_effort(
    pool: PgPool,
    event_kind: &str,
    source_id: &str,
    subject: Value,
    payload: Value,
    provenance: Value,
    correlation_id: Option<&str>,
) -> Option<EventEnvelope> {
    match dispatch_ai_helper_signal(
        pool,
        event_kind,
        source_id,
        subject,
        payload,
        provenance,
        correlation_id,
    )
    .await
    {
        Ok(event) => event,
        Err(error) => {
            tracing::warn!(
                error = %error,
                event_kind,
                source_id,
                "AI helper Signal Hub dispatch failed"
            );
            None
        }
    }
}
