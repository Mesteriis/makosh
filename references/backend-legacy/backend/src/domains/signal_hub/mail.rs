use chrono::{DateTime, Utc};
use makosh_events_api::{EventEnvelope, NewEventEnvelope};
use makosh_signal_hub_postgres::raw_signals::adapter::RawSignalStore;
use std::path::Path;

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

pub struct MailDeliverySignalRequest<'a> {
    pub occurred_at: DateTime<Utc>,
    pub account_id: &'a str,
    pub provider_message_id: &'a str,
    pub event_kind: &'a str,
    pub payload: serde_json::Value,
    pub source_kind: &'a str,
    pub provider_record_id: Option<&'a str>,
    pub raw_record_id: Option<&'a str>,
    pub correlation_id: Option<&'a str>,
}

pub async fn dispatch_mail_raw_signal(
    pool: PgPool,
    raw_record: &StoredRawCommunicationRecord,
    raw_blob_root: Option<&Path>,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let raw_signal = build_mail_raw_signal(raw_record, raw_blob_root)?;
    dispatch_mail_signal(pool, event_store, raw_signal).await
}

pub async fn dispatch_mail_delivery_event_signal(
    pool: PgPool,
    request: MailDeliverySignalRequest<'_>,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let raw_signal = build_mail_delivery_event_signal(&request)?;
    dispatch_mail_signal(pool, event_store, raw_signal).await
}

async fn dispatch_mail_signal(
    pool: PgPool,
    event_store: EventStore,
    raw_signal: NewEventEnvelope,
) -> Result<Option<EventEnvelope>, SignalHubError> {
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

fn build_mail_raw_signal(
    raw_record: &StoredRawCommunicationRecord,
    raw_blob_root: Option<&Path>,
) -> Result<NewEventEnvelope, SignalHubError> {
    let occurred_at = raw_record.occurred_at.unwrap_or(raw_record.captured_at);
    let source = json!({
        "kind": "signal_source",
        "source_code": "mail",
        "source_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
    });
    let subject = json!({
        "kind": "communication_raw_record",
        "source_code": "mail",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
    });
    let mut provenance = json!({
        "source": "communications_raw_record",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
        "import_batch_id": raw_record.import_batch_id,
        "raw_record_provenance": raw_record.provenance,
    });
    if let Some(root) = raw_blob_root.and_then(|value| value.to_str()) {
        provenance["blob_root"] = json!(root);
    }

    Ok(NewEventEnvelope::builder(
        mail_raw_signal_event_id(&raw_record.raw_record_id),
        "signal.raw.mail.message.observed",
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

fn build_mail_delivery_event_signal(
    request: &MailDeliverySignalRequest<'_>,
) -> Result<NewEventEnvelope, SignalHubError> {
    let builder = NewEventEnvelope::builder(
        mail_delivery_signal_event_id(
            request.account_id,
            request.provider_message_id,
            request.event_kind,
            request.source_kind,
            request.provider_record_id,
            request.raw_record_id,
        ),
        format!("signal.raw.mail.{}.observed", request.event_kind),
        request.occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "mail",
            "account_id": request.account_id,
            "provider_message_id": request.provider_message_id,
            "source_kind": request.source_kind,
            "provider_record_id": request.provider_record_id,
            "raw_record_id": request.raw_record_id,
        }),
        json!({
            "kind": "mail_provider_delivery_event",
            "source_code": "mail",
            "account_id": request.account_id,
            "provider_message_id": request.provider_message_id,
            "event_kind": request.event_kind,
            "source_kind": request.source_kind,
            "provider_record_id": request.provider_record_id,
            "raw_record_id": request.raw_record_id,
        }),
    )
    .payload(request.payload.clone())
    .provenance(json!({
        "source": "mail_provider_delivery_event",
        "source_kind": request.source_kind,
        "account_id": request.account_id,
        "provider_message_id": request.provider_message_id,
        "provider_record_id": request.provider_record_id,
        "raw_record_id": request.raw_record_id,
    }));
    let builder = match request.correlation_id {
        Some(value) if !value.trim().is_empty() => builder.correlation_id(value.to_owned()),
        _ => builder,
    };
    Ok(builder.build()?)
}

fn mail_raw_signal_event_id(raw_record_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_record_id.as_bytes());
    format!("evt_signal_raw_mail_{:x}", hasher.finalize())
}

fn mail_delivery_signal_event_id(
    account_id: &str,
    provider_message_id: &str,
    event_kind: &str,
    source_kind: &str,
    provider_record_id: Option<&str>,
    raw_record_id: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(provider_message_id.as_bytes());
    hasher.update(event_kind.as_bytes());
    hasher.update(source_kind.as_bytes());
    if let Some(value) = provider_record_id {
        hasher.update(value.as_bytes());
    }
    if let Some(value) = raw_record_id {
        hasher.update(value.as_bytes());
    }
    format!("evt_signal_raw_mail_delivery_{:x}", hasher.finalize())
}
