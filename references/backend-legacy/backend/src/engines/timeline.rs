use makosh_events_api::StoredEventEnvelope;
mod analysis;
mod cross_domain;
pub mod errors;
pub mod models;
mod policy;
mod projection;
mod replay;
mod summaries;
mod validation;

use chrono::{DateTime, Utc};

use makosh_events_postgres::cursors::ProjectionCursorStore;
use makosh_events_postgres::store::EventStore;

use self::errors::{TimelineEngineError, TimelineProjectionError};
use self::models::{
    TimelineChangeDiff, TimelineEntry, TimelineEventDraft, TimelineGap, TimelinePeriodSummary,
    TimelineProjectionRun, TimelineRecencySignal, TimelineReplay,
};

pub struct TimelineEngine;

impl TimelineEngine {
    pub fn bounded_entity_limit(limit: i64) -> i64 {
        policy::bounded_entity_limit(limit)
    }

    pub fn validate_event(event: &TimelineEventDraft<'_>) -> Result<(), TimelineEngineError> {
        policy::validate_event(event)
    }

    pub fn period_summary(
        events: &[TimelineEventDraft<'_>],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<TimelinePeriodSummary, TimelineEngineError> {
        summaries::period_summary(events, period_start, period_end)
    }

    pub fn recency_signal(
        events: &[TimelineEventDraft<'_>],
        entity_kind: &str,
        entity_id: &str,
        as_of: DateTime<Utc>,
    ) -> Result<TimelineRecencySignal, TimelineEngineError> {
        analysis::recency_signal(events, entity_kind, entity_id, as_of)
    }

    pub fn timeline_gaps(
        events: &[TimelineEventDraft<'_>],
        entity_kind: &str,
        entity_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        max_gap_seconds: i64,
    ) -> Result<Vec<TimelineGap>, TimelineEngineError> {
        analysis::timeline_gaps(
            events,
            entity_kind,
            entity_id,
            period_start,
            period_end,
            max_gap_seconds,
        )
    }

    pub fn change_diff(
        previous_events: &[TimelineEventDraft<'_>],
        current_events: &[TimelineEventDraft<'_>],
        entity_kind: &str,
        entity_id: &str,
    ) -> Result<TimelineChangeDiff, TimelineEngineError> {
        analysis::change_diff(previous_events, current_events, entity_kind, entity_id)
    }

    pub fn cross_domain_timeline(
        events: &[TimelineEventDraft<'_>],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<TimelineEntry>, TimelineEngineError> {
        cross_domain::cross_domain_timeline(events, period_start, period_end, limit)
    }

    pub fn replay_event_log(
        stored_events: &[StoredEventEnvelope],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit: i64,
    ) -> Result<TimelineReplay, TimelineEngineError> {
        replay::replay_event_log(stored_events, period_start, period_end, limit)
    }

    pub async fn run_event_log_projection(
        events: &EventStore,
        cursors: &ProjectionCursorStore,
        projection_name: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        batch_size: u32,
        timeline_limit: i64,
    ) -> Result<TimelineProjectionRun, TimelineProjectionError> {
        projection::run_event_log_projection(
            events,
            cursors,
            projection_name,
            period_start,
            period_end,
            batch_size,
            timeline_limit,
        )
        .await
    }
}
