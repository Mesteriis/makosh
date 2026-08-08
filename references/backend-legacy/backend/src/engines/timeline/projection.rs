use chrono::{DateTime, Utc};
use std::future;

use super::errors::TimelineProjectionError;
use super::models::TimelineProjectionRun;
use super::replay::replay_event_log;
use crate::platform::projections::{ProjectionHandlerError, run_projection_batch};
use makosh_events_postgres::cursors::ProjectionCursorStore;
use makosh_events_postgres::store::EventStore;

pub(super) async fn run_event_log_projection(
    events: &EventStore,
    cursors: &ProjectionCursorStore,
    projection_name: &str,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    batch_size: u32,
    timeline_limit: i64,
) -> Result<TimelineProjectionRun, TimelineProjectionError> {
    let mut replay_batch = Vec::new();
    let outcome = run_projection_batch(events, cursors, projection_name, batch_size, |event| {
        let validation =
            replay_event_log(std::slice::from_ref(&event), period_start, period_end, 1)
                .map(|_| ())
                .map_err(|error| ProjectionHandlerError::new(error.to_string()));
        if validation.is_ok() {
            replay_batch.push(event);
        }
        future::ready(validation)
    })
    .await?;

    let replay = replay_event_log(&replay_batch, period_start, period_end, timeline_limit)?;

    Ok(TimelineProjectionRun {
        processed_count: outcome.processed_count,
        last_processed_position: outcome.last_processed_position,
        entries: replay.entries,
    })
}
