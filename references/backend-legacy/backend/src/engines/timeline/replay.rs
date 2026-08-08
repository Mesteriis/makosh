use chrono::{DateTime, Utc};
use makosh_events_api::StoredEventEnvelope;

use super::errors::TimelineEngineError;
use super::models::{TimelineEntry, TimelineReplay};
use super::policy::bounded_entity_limit;
use super::validation::{event_log_source_ref, optional_json_string, required_json_string};

pub(super) fn replay_event_log(
    stored_events: &[StoredEventEnvelope],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    limit: i64,
) -> Result<TimelineReplay, TimelineEngineError> {
    if period_start > period_end {
        return Err(TimelineEngineError::InvalidPeriod);
    }

    let last_replayed_position = stored_events
        .iter()
        .map(|stored| stored.position)
        .max()
        .unwrap_or(0);
    let limit = bounded_entity_limit(limit) as usize;
    let mut entries = Vec::new();

    for stored in stored_events {
        let event = &stored.event;
        super::validation::validate_non_empty("event_type", &event.event_type)?;
        if event.occurred_at < period_start || event.occurred_at > period_end {
            continue;
        }

        entries.push(TimelineEntry {
            entity_kind: required_json_string(&event.subject, "subject", "kind", &event.event_id)?,
            entity_id: required_json_string(
                &event.subject,
                "subject",
                "entity_id",
                &event.event_id,
            )?,
            event_type: event.event_type.trim().to_owned(),
            title: optional_json_string(&event.payload, "title")
                .unwrap_or_else(|| event.event_type.trim().to_owned()),
            occurred_at: event.occurred_at,
            source: event_log_source_ref(event),
        });
    }

    entries.sort_by(|left, right| {
        left.occurred_at
            .cmp(&right.occurred_at)
            .then_with(|| left.source.cmp(&right.source))
    });
    entries.truncate(limit);

    Ok(TimelineReplay {
        last_replayed_position,
        entries,
    })
}
