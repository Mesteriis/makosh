use makosh_events_api::{EventEnvelopeError, NewEventEnvelope};
// ADR-0073: shared API support is split into bounded helper modules while
// route modules still import this facade during the backend decomposition phase.
pub(crate) mod automation_calls;
pub(crate) mod communications;
pub(crate) mod formatting;
pub(crate) mod messaging_integrations;
pub(crate) mod platform_dtos;
pub(crate) mod query_parsing;
pub(crate) mod review_commands;
pub(crate) mod review_lists;
pub(crate) mod stores;
pub(crate) mod telegram_capabilities;
pub(crate) mod telegram_capability_catalog;
pub(crate) mod telegram_capability_catalog_extended;
pub(crate) mod telegram_capability_catalog_foundation;
pub(crate) mod telegram_capability_catalog_messages;
pub(crate) mod whatsapp_capabilities;
pub(crate) mod whatsapp_capability_catalog;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::ai::control_center::errors::AiControlCenterError;
use crate::ai::control_center::models::AiProviderAccount;
use crate::ai::control_center::store::AiControlCenterStore;
use crate::ai::core::runs::AiAgentRun;
use crate::ai::core::service::core::AiService;
use crate::ai::core::types::AiModelRouting;
use crate::engines::automation::{
    models::{
        AutomationPolicy, AutomationPolicyScope, AutomationTemplate, NewAutomationPolicy,
        NewAutomationTemplate,
    },
    store::AutomationStore,
};
use crate::platform::audit::models::ApiAuditRecord;
use crate::platform::calls::models::{
    CallDirection, CallState, CallTranscript, NewProviderCall, ProviderCall,
};
use makosh_communications_postgres::store::CommunicationIngestionStore;

use crate::platform::config::ai::AiRuntimeProvider;
use crate::platform::config::app_config::AppConfig;

use crate::domains::personas::identity::models::{
    PersonaIdentityCandidate, PersonaIdentityDetail, PersonaIdentityReviewCommand,
};
use crate::domains::personas::identity::store::PersonaIdentityReviewStore;

use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::projection::parse_raw_email_message_from_blob;
use crate::domains::communications::messages::store::MessageProjectionStore;
use crate::domains::communications::storage::models::StoredCommunicationAttachmentWithBlob;
use crate::domains::communications::storage::store::CommunicationStorageStore;
use crate::domains::documents::processing::models::{
    DocumentProcessingJob, DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult,
    DocumentProcessingStatus,
};
use crate::domains::documents::processing::store::DocumentProcessingStore;
use crate::domains::projects::link_reviews::models::ProjectLinkReviewCommand;
use crate::domains::projects::link_reviews::store::ProjectLinkReviewStore;
use crate::domains::tasks::candidates::models::{TaskCandidate, TaskCandidateReviewCommand};
use crate::domains::tasks::candidates::store::TaskCandidateStore;
use crate::integrations::ai_runtime::{AiRuntimeClient, AiRuntimeError};
use crate::integrations::mail::accounts::service::EmailAccountSetupService;
use crate::integrations::ollama::client::OllamaClient;
use crate::integrations::ollama::client::config::OllamaClientConfig;
use crate::integrations::omniroute::client::OmniRouteClient;
use crate::integrations::omniroute::client::config::OmniRouteClientConfig;
use crate::integrations::omniroute::client::error::OmniRouteError;
use crate::integrations::telegram::client::ProviderCommunicationMessage;
use crate::integrations::telegram::client::models::chats::TelegramChat;
use crate::integrations::telegram::tdjson;
use crate::integrations::whatsapp::client::models::WhatsappWebSession;
use crate::platform::settings::ai_runtime::AiRuntimeSettings;
use crate::platform::settings::models::ApplicationSetting;
use crate::vault::models::VaultStatus;
use makosh_events_postgres::store::EventStore;

use crate::app::error::types::ApiError;
use crate::app::state::AppState;

pub(crate) fn ensure_fixture_routes_enabled(state: &AppState) -> Result<(), ApiError> {
    if state.config.dev_mode() || cfg!(test) {
        return Ok(());
    }
    Err(ApiError::NotFound)
}
