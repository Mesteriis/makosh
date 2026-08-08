use makosh_events_api::{EventEnvelope, EventEnvelopeError, NewEventEnvelope, StoredEventEnvelope};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::signal_hub::raw_signal_port::RawSignalProcessingPort;
use crate::domains::signal_hub::store::SignalHubError;
use crate::platform::events::bus::zoom_event_types;
use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;
use makosh_signal_hub_api::raw_signals::{
    RawSignalCommandPort, RawSignalInput, RawSignalPortError, RawSignalRuntimeQueryPort,
};

pub const ZOOM_SIGNAL_DETECTION_CONSUMER: &str = "zoom_signal_detection";

#[derive(Debug, Error)]
pub enum ZoomSignalDetectionWorkflowError {
    #[error(transparent)]
    SignalHub(#[from] SignalHubError),

    #[error(transparent)]
    SignalHubPort(#[from] RawSignalPortError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error("event `{0}` is missing required field `{1}`")]
    MissingField(&'static str, &'static str),
}

pub async fn project_zoom_signal_detection_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_zoom_signal_detection(&pool, &event.event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn project_zoom_signal_detection(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ZoomSignalDetectionWorkflowError> {
    let Some(raw_signal) = build_zoom_raw_signal(event)? else {
        return Ok(());
    };

    let event_store = EventStore::new(pool.clone());
    let raw_signal_id = raw_signal.event_id.clone();
    event_store
        .append_for_dispatch_idempotent(&raw_signal)
        .await?;

    let raw_event = event_store.get_by_id(&raw_signal_id).await?.ok_or(
        SignalHubError::InvalidRawSignalEventType(raw_signal.event_type.clone()),
    )?;

    let signal_port = RawSignalProcessingPort::new(pool.clone());
    if !RawSignalRuntimeQueryPort::allows_processing(&signal_port).await? {
        return Ok(());
    }

    let _ = RawSignalCommandPort::process_raw_signal(&signal_port, &RawSignalInput::new(raw_event))
        .await?;
    Ok(())
}

fn build_zoom_raw_signal(
    event: &EventEnvelope,
) -> Result<Option<NewEventEnvelope>, ZoomSignalDetectionWorkflowError> {
    let Some(event_kind) = zoom_signal_event_kind(&event.event_type) else {
        return Ok(None);
    };

    let account_id = required_string(
        &event.source,
        "account_id",
        "zoom signal detection source.account_id",
    )?;
    let subject = zoom_raw_signal_subject(event, event_kind, account_id)?;
    let source = json!({
        "kind": "signal_source",
        "source_code": "zoom",
        "source_id": event.event_id,
        "account_id": account_id,
        "provider_kind": event.source.get("provider_kind").cloned(),
        "zoom_event_type": event.event_type,
    });
    let provenance = json!({
        "source": "zoom_signal_detection",
        "zoom_event_id": event.event_id,
        "zoom_event_type": event.event_type,
        "zoom_event_provenance": event.provenance,
    });

    let builder = NewEventEnvelope::builder(
        zoom_raw_signal_event_id(&event.event_id),
        format!("signal.raw.zoom.{event_kind}.observed"),
        event.occurred_at,
        source,
        subject,
    )
    .payload(event.payload.clone())
    .provenance(provenance)
    .causation_id(event.event_id.clone());

    let builder = match &event.correlation_id {
        Some(value) => builder.correlation_id(value.clone()),
        None => builder,
    };

    Ok(Some(builder.build()?))
}

fn zoom_signal_event_kind(event_type: &str) -> Option<&'static str> {
    match event_type {
        zoom_event_types::MEETING_OBSERVED => Some("meeting"),
        zoom_event_types::RECORDING_OBSERVED => Some("recording"),
        zoom_event_types::TRANSCRIPT_OBSERVED => Some("transcript"),
        _ => None,
    }
}

fn zoom_raw_signal_subject(
    event: &EventEnvelope,
    event_kind: &str,
    account_id: &str,
) -> Result<Value, ZoomSignalDetectionWorkflowError> {
    let mut subject = json!({
        "kind": "signal",
        "source_code": "zoom",
        "account_id": account_id,
        "zoom_event_id": event.event_id,
        "zoom_event_type": event.event_type,
    });

    match event_kind {
        "meeting" => {
            let call_id = required_string(&event.subject, "call_id", "zoom.meeting.observed")?;
            let meeting_id =
                required_string(&event.payload, "meeting_id", "zoom.meeting.observed")?;
            subject["entity_id"] = json!(call_id);
            subject["call_id"] = json!(call_id);
            subject["meeting_id"] = json!(meeting_id);
        }
        "recording" => {
            let recording_id =
                required_string(&event.subject, "recording_id", "zoom.recording.observed")?;
            let meeting_id =
                required_string(&event.subject, "meeting_id", "zoom.recording.observed")?;
            subject["entity_id"] = json!(recording_id);
            subject["recording_id"] = json!(recording_id);
            subject["meeting_id"] = json!(meeting_id);
        }
        "transcript" => {
            let transcript_id =
                required_string(&event.subject, "transcript_id", "zoom.transcript.observed")?;
            let call_id = required_string(&event.subject, "call_id", "zoom.transcript.observed")?;
            let meeting_id =
                required_string(&event.subject, "meeting_id", "zoom.transcript.observed")?;
            subject["entity_id"] = json!(transcript_id);
            subject["transcript_id"] = json!(transcript_id);
            subject["call_id"] = json!(call_id);
            subject["meeting_id"] = json!(meeting_id);
        }
        _ => {}
    }

    Ok(subject)
}

fn required_string<'a>(
    value: &'a Value,
    field: &'static str,
    event_type: &'static str,
) -> Result<&'a str, ZoomSignalDetectionWorkflowError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZoomSignalDetectionWorkflowError::MissingField(
            event_type, field,
        ))
}

fn zoom_raw_signal_event_id(zoom_event_id: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(zoom_event_id.as_bytes());
    format!("evt_signal_raw_zoom_{:x}", digest.finalize())
}
