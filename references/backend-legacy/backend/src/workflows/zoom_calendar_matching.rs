use chrono::{DateTime, Utc};
use makosh_events_api::{EventEnvelope, StoredEventEnvelope};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::calendar::core::errors::CalendarCoreError;
use crate::domains::calendar::events::errors::CalendarError;
use crate::domains::calendar::ports::CalendarEventQueryPort;
use crate::domains::calendar::ports::EventRelationPort;
use crate::platform::events::bus::zoom_event_types;
use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

pub const ZOOM_CALENDAR_MATCHING_CONSUMER: &str = "zoom_calendar_matching";
pub const ZOOM_CALENDAR_MATCHING_PROJECTION: &str = "zoom_calendar_matching";
pub const ZOOM_CALENDAR_RELATION_TYPE: &str = "conference_call";

#[derive(Debug, Error)]
pub enum ZoomCalendarMatchingWorkflowError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Calendar(#[from] CalendarError),

    #[error(transparent)]
    CalendarCore(#[from] CalendarCoreError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),
}

pub async fn project_zoom_calendar_matching_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_zoom_calendar_matching(&pool, &event.event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn project_zoom_calendar_matching(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ZoomCalendarMatchingWorkflowError> {
    if event.event_type != zoom_event_types::MEETING_OBSERVED {
        return Ok(());
    }

    let call_id = required_subject_string(&event.subject, "call_id")?;
    let meeting_id = required_payload_string(&event.payload, "meeting_id")?;
    let join_url = optional_payload_string(&event.payload, "join_url");
    let started_at = optional_payload_datetime(&event.payload, "started_at");
    let ended_at = optional_payload_datetime(&event.payload, "ended_at");

    let event_store = CalendarEventQueryPort::new(pool.clone());
    let Some(calendar_event) = event_store
        .find_zoom_conference_match(join_url, meeting_id, started_at, ended_at)
        .await?
    else {
        return Ok(());
    };

    let observation = ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "CALENDAR_EVENT",
                ObservationOriginKind::LocalRuntime,
                event.occurred_at,
                json!({
                    "event_id": calendar_event.event_id,
                    "matched_entity_type": "call",
                    "matched_entity_id": call_id,
                    "meeting_id": meeting_id,
                    "join_url": join_url,
                    "source_event_id": event.event_id,
                    "match_strategy": "zoom_conference_url_and_time_overlap",
                }),
                format!(
                    "calendar-event://{}/matches/zoom-call/{}",
                    calendar_event.event_id, call_id
                ),
            )
            .provenance(json!({
                "captured_by": "zoom_calendar_matching",
                "event_id": event.event_id,
                "event_type": event.event_type,
            })),
        )
        .await?;

    EventRelationPort::new(pool.clone())
        .link_with_observation(
            &calendar_event.event_id,
            "call",
            call_id,
            ZOOM_CALENDAR_RELATION_TYPE,
            zoom_event_types::MEETING_OBSERVED,
            Some(&observation.observation_id),
        )
        .await?;

    Ok(())
}

fn required_subject_string<'a>(
    subject: &'a Value,
    field: &'static str,
) -> Result<&'a str, ZoomCalendarMatchingWorkflowError> {
    subject
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZoomCalendarMatchingWorkflowError::MissingPayloadField(
            field,
        ))
}

fn required_payload_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, ZoomCalendarMatchingWorkflowError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZoomCalendarMatchingWorkflowError::MissingPayloadField(
            field,
        ))
}

fn optional_payload_string<'a>(payload: &'a Value, field: &'static str) -> Option<&'a str> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn optional_payload_datetime(payload: &Value, field: &'static str) -> Option<DateTime<Utc>> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc))
}
