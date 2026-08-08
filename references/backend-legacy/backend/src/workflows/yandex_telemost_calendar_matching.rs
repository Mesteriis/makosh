use makosh_events_api::{EventEnvelope, StoredEventEnvelope};
use std::collections::HashSet;

use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::calendar::core::errors::CalendarCoreError;
use crate::domains::calendar::events::errors::CalendarError;
use crate::domains::calendar::ports::CalendarEventQueryPort;
use crate::domains::calendar::ports::{EventParticipantPort, EventRelationPort};
use crate::platform::events::bus::yandex_telemost_event_types;
use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

pub const YANDEX_TELEMOST_CALENDAR_MATCHING_CONSUMER: &str = "yandex_telemost_calendar_matching";
pub const YANDEX_TELEMOST_CALENDAR_MATCHING_PROJECTION: &str = "yandex_telemost_calendar_matching";
pub const YANDEX_TELEMOST_CALENDAR_RELATION_TYPE: &str = "conference_call";
const YANDEX_TELEMOST_CALENDAR_PARTICIPANT_SOURCE: &str = "yandex_telemost_cohost_observed";

#[derive(Debug, Deserialize)]
struct TelemostCohostObservation {
    email: String,
}

#[derive(Debug, Error)]
pub enum YandexTelemostCalendarMatchingWorkflowError {
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

pub async fn project_yandex_telemost_calendar_matching_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_yandex_telemost_calendar_matching(&pool, &event.event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn project_yandex_telemost_calendar_matching(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), YandexTelemostCalendarMatchingWorkflowError> {
    if !supports_yandex_telemost_calendar_matching_event(&event.event_type) {
        return Ok(());
    }

    if event.event_type == yandex_telemost_event_types::COHOSTS_OBSERVED {
        return project_yandex_telemost_cohosts_into_calendar(pool, event).await;
    }

    let conference = event
        .payload
        .get("conference")
        .ok_or(YandexTelemostCalendarMatchingWorkflowError::MissingPayloadField("conference"))?;
    let conference_id = required_nested_string(conference, "id")?;
    let join_url = required_nested_string(conference, "join_url")?;

    let event_store = CalendarEventQueryPort::new(pool.clone());
    let Some(calendar_event) = event_store
        .find_yandex_telemost_conference_match(Some(join_url), conference_id)
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
                    "matched_entity_type": "telemost_conference",
                    "matched_entity_id": conference_id,
                    "conference_id": conference_id,
                    "join_url": join_url,
                    "source_event_id": event.event_id,
                    "match_strategy": "telemost_conference_url",
                }),
                format!(
                    "calendar-event://{}/matches/telemost-conference/{}",
                    calendar_event.event_id, conference_id
                ),
            )
            .provenance(json!({
                "captured_by": "yandex_telemost_calendar_matching",
                "event_id": event.event_id,
                "event_type": event.event_type,
            })),
        )
        .await?;

    EventRelationPort::new(pool.clone())
        .link_with_observation(
            &calendar_event.event_id,
            "telemost_conference",
            conference_id,
            YANDEX_TELEMOST_CALENDAR_RELATION_TYPE,
            event.event_type.as_str(),
            Some(&observation.observation_id),
        )
        .await?;

    Ok(())
}

async fn project_yandex_telemost_cohosts_into_calendar(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), YandexTelemostCalendarMatchingWorkflowError> {
    let conference_id = required_string(&event.payload, "conference_id")?;
    let cohosts = event
        .payload
        .get("cohosts")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let cohosts: Vec<TelemostCohostObservation> = serde_json::from_value(cohosts)?;

    if cohosts.is_empty() {
        return Ok(());
    }

    let event_store = CalendarEventQueryPort::new(pool.clone());
    let Some(calendar_event) = event_store
        .find_yandex_telemost_conference_match(None, conference_id)
        .await?
    else {
        return Ok(());
    };

    let participant_store = EventParticipantPort::new(pool.clone());
    let existing = participant_store.list(&calendar_event.event_id).await?;
    let mut known_emails = existing
        .into_iter()
        .map(|participant| participant.email.trim().to_ascii_lowercase())
        .filter(|email| !email.is_empty())
        .collect::<HashSet<_>>();

    let observation_store = ObservationStore::new(pool.clone());
    for cohost in cohosts {
        let email = cohost.email.trim().to_ascii_lowercase();
        if email.is_empty() || known_emails.contains(&email) {
            continue;
        }

        let observation = observation_store
            .capture(
                &NewObservation::new(
                    "CALENDAR_EVENT",
                    ObservationOriginKind::LocalRuntime,
                    event.occurred_at,
                    json!({
                        "event_id": calendar_event.event_id,
                        "conference_id": conference_id,
                        "participant_email": email,
                        "participant_role": "attendee",
                        "provider_role": "cohost",
                        "source_event_id": event.event_id,
                        "source_kind": "telemost_cohost",
                    }),
                    format!(
                        "calendar-event://{}/participants/telemost-cohost/{}",
                        calendar_event.event_id, email
                    ),
                )
                .provenance(json!({
                    "captured_by": "yandex_telemost_calendar_matching",
                    "event_id": event.event_id,
                    "event_type": event.event_type,
                })),
            )
            .await?;

        let _ = participant_store
            .add_with_observation(
                &calendar_event.event_id,
                &email,
                None,
                Some("attendee"),
                None,
                None,
                YANDEX_TELEMOST_CALENDAR_PARTICIPANT_SOURCE,
                Some(&observation.observation_id),
            )
            .await?;
        known_emails.insert(email);
    }

    Ok(())
}

pub fn supports_yandex_telemost_calendar_matching_event(event_type: &str) -> bool {
    matches!(
        event_type,
        yandex_telemost_event_types::CONFERENCE_CREATED
            | yandex_telemost_event_types::CONFERENCE_OBSERVED
            | yandex_telemost_event_types::CONFERENCE_UPDATED
            | yandex_telemost_event_types::COHOSTS_OBSERVED
    )
}

fn required_string<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a str, YandexTelemostCalendarMatchingWorkflowError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .ok_or(YandexTelemostCalendarMatchingWorkflowError::MissingPayloadField(field))
}

fn required_nested_string<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a str, YandexTelemostCalendarMatchingWorkflowError> {
    required_string(value, field)
}
