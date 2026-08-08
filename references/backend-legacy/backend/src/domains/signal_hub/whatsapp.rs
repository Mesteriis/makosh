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

pub async fn dispatch_whatsapp_raw_signal(
    pool: PgPool,
    raw_record: &StoredRawCommunicationRecord,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let raw_signal = build_whatsapp_raw_signal(raw_record)?;
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

fn build_whatsapp_raw_signal(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewEventEnvelope, SignalHubError> {
    let occurred_at = raw_record.occurred_at.unwrap_or(raw_record.captured_at);
    let source = json!({
        "kind": "signal_source",
        "source_code": "whatsapp",
        "source_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
    });
    let subject = json!({
        "kind": "communication_raw_record",
        "source_code": "whatsapp",
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

    let event_kind = match raw_record.record_kind.as_str() {
        "whatsapp_web_reaction" => "reaction",
        "whatsapp_web_media" => "media",
        "whatsapp_web_status" => "status",
        "whatsapp_web_status_view" => "status_view",
        "whatsapp_web_status_delete" => "status_delete",
        "whatsapp_web_presence" => "presence",
        "whatsapp_web_call" => "call_metadata",
        "whatsapp_web_runtime_event" => "runtime_event",
        "whatsapp_web_dialog" => "dialog",
        "whatsapp_web_participant" => "participant",
        "whatsapp_web_message_update" => "message_update",
        "whatsapp_web_message_delete" => "message_delete",
        "whatsapp_web_receipt" => "receipt",
        _ => "message",
    };

    Ok(NewEventEnvelope::builder(
        whatsapp_raw_signal_event_id(&raw_record.raw_record_id),
        format!("signal.raw.whatsapp.{event_kind}.observed"),
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

fn whatsapp_raw_signal_event_id(raw_record_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_record_id.as_bytes());
    format!("evt_signal_raw_whatsapp_{:x}", hasher.finalize())
}
