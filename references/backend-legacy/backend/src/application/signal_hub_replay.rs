use chrono::{DateTime, Utc};
use makosh_events_api::{NewEventEnvelope, StoredEventEnvelope};
use makosh_signal_hub_postgres::raw_signals::adapter::RawSignalStore;
use serde_json::json;
use std::sync::Arc;

use crate::domains::communications::messages::provider_observation_projection::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, replay_accepted_signal_event,
    supports_communication_projection_signal_event,
};
use crate::domains::personas::core::roles::{
    PERSONA_ROLE_ASSIGNED_EVENT_TYPE, PERSONA_ROLE_REMOVED_EVENT_TYPE,
};
use crate::domains::personas::enrichment::PERSONA_TRUST_SCORE_CHANGED_EVENT_TYPE;
use crate::domains::personas::trust::promises::PERSONA_PROMISE_CREATED_EVENT_TYPE;
use crate::domains::signal_hub::replay_contracts::SignalReplayRequest;
use crate::domains::signal_hub::service::SignalHubSignalService;
use crate::domains::signal_hub::store::{SignalHubError, SignalHubStore};
use crate::engines::timeline::TimelineEngine;
use crate::workflows::persona_derived_evidence::{
    PERSONA_DERIVED_EVIDENCE_CONSUMER, project_persona_derived_evidence_event,
};
use crate::workflows::project_link_review_effects::PROJECT_LINK_REVIEW_EVENT_TYPE;
use crate::workflows::project_link_review_effects::{
    PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, project_link_review_effect_event,
};
use crate::workflows::realtime_conversation_transcript_projection::REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER;
use crate::workflows::realtime_conversation_transcript_projection::project_realtime_conversation_transcript_event;
use crate::workflows::yandex_telemost_calendar_matching::project_yandex_telemost_calendar_matching_event;
use crate::workflows::zoom_calendar_matching::{
    ZOOM_CALENDAR_MATCHING_CONSUMER, project_zoom_calendar_matching_event,
};
use makosh_events_api::EventLogQuery;
use makosh_events_postgres::consumers::EventConsumerStore;
use makosh_events_postgres::cursors::ProjectionCursorStore;
use makosh_events_postgres::store::EventStore;

#[path = "signal_hub_replay_runtime.rs"]
mod signal_hub_replay_runtime;

const DEFAULT_REPLAY_BATCH_SIZE: u32 = 500;
const COMMUNICATION_MESSAGES_PROJECTION: &str = "communication_messages";
const PERSONA_DERIVED_EVIDENCE_PROJECTION: &str = "persona_derived_evidence";
const PROJECT_LINK_REVIEW_EFFECTS_PROJECTION: &str = "project_link_review_effects";
const REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION: &str =
    "realtime_conversation_transcript_projection";
const TIMELINE_EVENT_LOG_PROJECTION: &str = "timeline_event_log";
const YANDEX_TELEMOST_CALENDAR_MATCHING_PROJECTION: &str = "yandex_telemost_calendar_matching";
const ZOOM_CALENDAR_MATCHING_PROJECTION: &str = "zoom_calendar_matching";
const TIMELINE_EVENT_LOG_CURSOR: &str = "signal_hub.timeline_event_log";

#[derive(Clone)]
pub struct SignalHubReplayService {
    signal_store: SignalHubStore,
    raw_signal_store: RawSignalStore,
    signal_service: SignalHubSignalService,
    event_store: EventStore,
}

fn uses_event_log_replay(request: &SignalReplayRequest) -> bool {
    request.from_position.is_some()
        || request.to_position.is_some()
        || request.from_time.is_some()
        || request.to_time.is_some()
}

fn supports_persona_derived_evidence_projection_event(event_type: &str) -> bool {
    matches!(
        event_type,
        PERSONA_ROLE_ASSIGNED_EVENT_TYPE
            | PERSONA_ROLE_REMOVED_EVENT_TYPE
            | PERSONA_TRUST_SCORE_CHANGED_EVENT_TYPE
            | PERSONA_PROMISE_CREATED_EVENT_TYPE
            | "person.role.assigned"
            | "person.role.removed"
            | "person.enrichment.trust_score_changed"
            | "person.promise.created"
    )
}

fn supports_project_link_review_effects_projection_event(event_type: &str) -> bool {
    event_type == PROJECT_LINK_REVIEW_EVENT_TYPE
}

fn supports_realtime_conversation_transcript_projection_event(event_type: &str) -> bool {
    event_type == crate::platform::realtime_conversation::events::REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED
}

fn supports_yandex_telemost_calendar_matching_projection_event(event_type: &str) -> bool {
    crate::workflows::yandex_telemost_calendar_matching::supports_yandex_telemost_calendar_matching_event(event_type)
}

fn supports_zoom_calendar_matching_projection_event(event_type: &str) -> bool {
    event_type == crate::platform::events::bus::zoom_event_types::MEETING_OBSERVED
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalReplayRunReport {
    pub request_id: String,
    pub replayed_count: u32,
}

fn max_timestamp(left: DateTime<Utc>, right: DateTime<Utc>) -> DateTime<Utc> {
    if left >= right { left } else { right }
}
