use crate::platform::audit::store::ApiAuditLog;
use std::sync::Arc;

use chrono::Utc;
use serde_json::json;

use crate::application::communication_fixture_event::build as build_event;
use crate::application::communication_provider_command_events::{
    CommandEventInput, build_command_event, merge_json_objects,
};
use crate::application::communication_provider_error::TelegramMessageWriteError;
use crate::application::communication_provider_models::{
    CommunicationConversationMessageRequest, CommunicationForwardRequest,
    CommunicationReplyRequest, TelegramMessageMarkReadRequest, TelegramMessageMarkReadResponse,
};
use crate::application::communication_provider_reactions::canonical_reaction_summary;
use crate::application::communication_provider_reference_chains::forward_chain;
use crate::application::communication_provider_reference_chains::reply_chain;
use crate::application::communication_provider_validation::required_provider_chat_id;
use crate::application::telegram_fixture_snapshot::message_snapshot_payload as telegram_message_snapshot_payload;
use crate::application::telegram_runtime::{self, TelegramRuntimeUseCaseContext};
use crate::domains::communications::messages::provider_observation_projection::project_accepted_signal_if_runtime_allows;
use crate::domains::signal_hub::telegram::dispatch_telegram_raw_signal;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::lifecycle::{message_versions, operations, tombstones};
use crate::integrations::telegram::client::models::message_references::{
    TelegramForwardChainResponse, TelegramReplyChainResponse,
};
use crate::integrations::telegram::client::models::messages::{
    TelegramDeleteRequest, TelegramEditRequest, TelegramForwardRequest, TelegramLifecycleResponse,
    TelegramManualSendRequest, TelegramManualSendResponse, TelegramMessageTombstone,
    TelegramMessageTombstoneListResponse, TelegramMessageVersion,
    TelegramMessageVersionListResponse, TelegramPinRequest, TelegramReaction,
    TelegramReactionListResponse, TelegramReactionRequest, TelegramReactionResponse,
    TelegramReplyRequest, TelegramRestoreVisibilityRequest,
};
use crate::integrations::telegram::client::store::TelegramStore;
use crate::integrations::telegram::client::{commands, reactions, references};
use crate::platform::audit::models::NewApiAuditRecord;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;
use makosh_communications_api::canonical::CanonicalMessageReadPort;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::store::EventStore;

#[path = "communication_provider_writes_runtime.rs"]
mod communication_provider_writes_runtime;

const AUDIT_ACTOR_ID: &str = "makosh-frontend";

pub(crate) fn new_telegram_command_id() -> String {
    commands::new_command_id()
}

#[derive(Clone)]
pub(crate) struct TelegramMessageWriteApplicationService {
    store: TelegramStore,
    canonical_message_reads: Arc<dyn CanonicalMessageReadPort>,
    audit_log: ApiAuditLog,
    event_store: EventStore,
    event_bus: InMemoryEventBus,
}
