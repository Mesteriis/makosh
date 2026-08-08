// ADR-0073: mail handlers are grouped by bounded context for the first
// handlers.rs extraction; split by communications, accounts and workflow next.
pub(crate) mod account_management;
pub(crate) mod account_provider_resources;
pub(crate) mod account_setup;
pub(crate) mod account_support;
pub(crate) mod communication_messages;
pub(crate) mod communication_queries;
pub(crate) mod eml_import;
pub(crate) mod finance_analytics;
pub(crate) mod legal_export;
pub(crate) mod message_actions;
pub(crate) mod message_ai_state;
pub(crate) mod provider_command_recovery;
pub(crate) mod remote_images;
pub(crate) mod sending;
pub(crate) mod templates_status;
pub(crate) mod workflow_actions;
pub(crate) mod workflow_state;

use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, RawQuery, State};
use axum::http::StatusCode;
use axum::response::Html;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::platform::audit::models::NewApiAuditRecord;

use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::projection::parse_raw_email_message_from_blob;
use crate::domains::communications::messages::states::{LocalMessageState, WorkflowState};
use crate::domains::communications::messages::store::MessageProjectionStore;
use crate::domains::communications::storage::models::StoredCommunicationAttachmentWithBlob;
use crate::integrations::mail::accounts::{
    errors::EmailAccountSetupError,
    models::{GmailOAuthPendingGrant, GmailOAuthSetupRequest, ImapAccountSetupRequest},
};
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::secrets::models::{SecretKind, SecretStoreKind};
use crate::vault::errors::HostVaultError;
use crate::vault::models::{EntropyEvent, VaultMode};
use crate::workflows::address_book_sync::{
    AddressBookSyncError, AddressBookSyncRunResponse, AddressBookSyncService,
    AddressBookSyncTrigger,
};
use crate::workflows::email_intelligence::models::EmailSummaryContract;
use crate::workflows::email_intelligence::service::EmailIntelligenceService;
use crate::workflows::mail_background_sync::{
    errors::MailSyncError,
    models::{runs::MailSyncRunResponse, settings::MailSyncSettings, status::MailSyncStatus},
    service::MailBackgroundSyncService,
    store::MailSyncStore,
};
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;

use crate::app::api_support::{
    communications::*,
    platform_dtos::*,
    query_parsing::communication::*,
    stores::{ai_runtime::*, domain_stores::*, integration_stores::*},
};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
