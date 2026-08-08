use makosh_events_api::{EventEnvelope, StoredEventEnvelope};
use serde::Deserialize;
use serde_json::Value;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::personas::identity::errors::PersonaIdentityError;
use crate::domains::personas::identity::ports::PersonaIdentityReviewPort;
use crate::platform::events::bus::zoom_event_types;
use makosh_events_postgres::errors::EventStoreError;

pub const ZOOM_PARTICIPANT_IDENTITY_CONSUMER: &str = "zoom_participant_identity";

const ATTACH_EMAIL_CANDIDATE_LIMIT_PER_PARTICIPANT: i64 = 10;
const ATTACH_EMAIL_CANDIDATE_CONFIDENCE: f64 = 0.68;

#[derive(Debug, Deserialize)]
struct ZoomParticipantObservation {
    display_name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Error)]
pub enum ZoomParticipantIdentityWorkflowError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    PersonaIdentity(#[from] PersonaIdentityError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),
}

pub async fn project_zoom_participant_identity_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_zoom_participant_identity(&pool, &event.event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn project_zoom_participant_identity(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ZoomParticipantIdentityWorkflowError> {
    if event.event_type != zoom_event_types::MEETING_OBSERVED {
        return Ok(());
    }

    let meeting_id = required_payload_string(&event.payload, "meeting_id")?;
    let topic = optional_payload_string(&event.payload, "topic");
    let participants = event
        .payload
        .get("participants")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let participants: Vec<ZoomParticipantObservation> = serde_json::from_value(participants)?;

    let store = PersonaIdentityReviewPort::new(pool.clone());
    for participant in participants {
        let Some(display_name) = participant
            .display_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(email_address) = participant
            .email
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };

        let evidence_summary = if let Some(topic) = topic {
            format!(
                "Zoom participant {display_name} <{email_address}> observed in meeting {meeting_id} ({topic})"
            )
        } else {
            format!(
                "Zoom participant {display_name} <{email_address}> observed in meeting {meeting_id}"
            )
        };
        store
            .suggest_attach_email_candidates(
                display_name,
                email_address,
                &evidence_summary,
                ATTACH_EMAIL_CANDIDATE_CONFIDENCE,
                ATTACH_EMAIL_CANDIDATE_LIMIT_PER_PARTICIPANT,
            )
            .await?;
    }

    Ok(())
}

fn required_payload_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, ZoomParticipantIdentityWorkflowError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZoomParticipantIdentityWorkflowError::MissingPayloadField(
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
