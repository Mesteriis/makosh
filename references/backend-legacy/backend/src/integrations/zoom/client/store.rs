use crate::platform::calls::store::CallIntelligenceStore;
use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::ProviderSecretBindingCommandPort;
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountCommandPort, ProviderAccountSecretPurpose,
};
use makosh_events_api::NewEventEnvelope;
use makosh_provider_zoom::protocol::{
    ZOOM_EXPLICIT_TOKEN_REFRESH_THRESHOLD_SECONDS, ZOOM_LIVE_AUTHORIZED_RUNTIME_KIND,
    ZOOM_MAX_RECORDING_MEDIA_DOWNLOAD_BYTES, ZOOM_MAX_TOKEN_REFRESH_THRESHOLD_SECONDS,
    ZOOM_PROVIDER_KIND_STR, ZOOM_RUNTIME_KIND, ZOOM_TOKEN_EXPIRY_SAFETY_MARGIN_SECONDS,
    ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS, ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER,
    ZoomAuthShape, random_zoom_oauth_token, sanitize_zoom_payload, zoom_authorization_url,
    zoom_client_secret_ref, zoom_oauth_expires_at, zoom_oauth_token_secret_ref,
};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::platform::calls::models::{NewCallTranscript, NewProviderCall, TranscriptStatus};
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::zoom_event_types;
use crate::platform::secrets::models::{NewSecretReference, SecretKind, SecretStoreKind};
use crate::platform::settings::store::ApplicationSettingsStore;
use crate::platform::storage::communication_media::{
    ImportedAttachmentRecord, ImportedAttachmentStoragePort, ImportedAttachmentUpsert,
    SafetyScanRequest, delete_local_blob, new_attachment_import_id, put_local_blob,
    scan_attachment,
};
use crate::vault::HostVault;
use crate::vault::models::SecretEntryContext;
use makosh_events_postgres::store::EventStore;

use super::errors::ZoomError;
#[path = "account_state.rs"]
mod account_state;
mod api_models;
mod auth;
#[path = "oauth_api.rs"]
mod oauth_api;
mod policy;
mod recording;
#[path = "remote_api.rs"]
mod remote_api;
#[path = "store/runtime.rs"]
mod runtime;
#[path = "secret_persistence.rs"]
mod secret_persistence;
#[path = "secret_resolution.rs"]
mod secret_resolution;
mod status;
use super::models::oauth_models::{
    ZoomOAuthPendingGrant, ZoomOAuthStartRequest, ZoomServerToServerAuthorizeRequest,
    ZoomTokenMaintenanceItem, ZoomTokenMaintenanceRequest, ZoomTokenMaintenanceResult,
    ZoomTokenRefreshRequest, ZoomTokenRefreshResult,
};
use super::models::{
    MAX_TRANSCRIPT_FILE_TEXT_BYTES, ZOOM_PROVIDER_KIND, ZoomAccount, ZoomAccountListResponse,
    ZoomAccountSetupRequest, ZoomAccountSetupResponse, ZoomAuditEventItem, ZoomAuditEventResponse,
    ZoomLiveAccountSetupRequest, ZoomMeetingIngestResult, ZoomMeetingObservationRequest,
    ZoomRecordingImportAuditItem, ZoomRecordingImportAuditResponse,
    ZoomRecordingImportRemoveRequest, ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingMediaImportResult,
    ZoomRecordingObservationRequest, ZoomRecordingRef, ZoomRecordingSyncFailure,
    ZoomRecordingSyncRequest, ZoomRecordingSyncResult, ZoomRetentionCleanupItem,
    ZoomRetentionCleanupRequest, ZoomRetentionCleanupResponse, ZoomRuntimeRemoveRequest,
    ZoomRuntimeRemoveResponse, ZoomRuntimeStartRequest, ZoomRuntimeStatus, ZoomRuntimeStopRequest,
    ZoomTranscriptFileImportRequest, ZoomTranscriptFileImportResult, ZoomTranscriptIngestResult,
    ZoomTranscriptObservationRequest, ZoomWebhookSubscription,
    ZoomWebhookSubscriptionReconcileRequest, ZoomWebhookSubscriptionReconcileResult,
    ZoomWebhookSubscriptionRemoveRequest, ZoomWebhookSubscriptionRemoveResult,
    ZoomWebhookSubscriptionStatusRequest, ZoomWebhookSubscriptionStatusResult,
};
use super::models::{ZoomAuthorizationResult, ZoomOAuthTokenBundle};
use api_models::ZoomApiRecordingFile;
use auth::{
    resolve_client_secret, validate_account_id, validate_required, zoom_client_secret_metadata,
    zoom_http_client, zoom_oauth_secret_metadata,
};
use policy::{
    authorization_result, canonical_zoom_webhook_event_types,
    find_managed_zoom_webhook_subscription, refresh_flow_label,
};
use recording::{
    json_string_or_number, provider_sync_user_id, stable_zoom_call_id, stable_zoom_transcript_id,
    zoom_api_datetime, zoom_event, zoom_event_id, zoom_recording_content_type,
    zoom_recording_file_is_transcript, zoom_recording_import_audit_item,
    zoom_transcript_content_type,
};
use status::{
    add_zoom_token_refresh_blocker, clear_zoom_live_authorization_required_blocker,
    clear_zoom_provider_workers_not_enabled_blocker, clear_zoom_token_refresh_blocker,
    ensure_zoom_account_is_authorized, parse_optional_datetime, runtime_status_from_account,
    zoom_account_is_authorized,
};

#[derive(Clone)]
pub struct ZoomStore {
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    imported_attachment_store: Arc<dyn ImportedAttachmentStoragePort>,
    call_store: CallIntelligenceStore,
    event_store: EventStore,
    event_bus: InMemoryEventBus,
    http: reqwest::Client,
}

struct ZoomAuthorizedAccountUpdate<'a> {
    auth_shape: &'a str,
    token_secret_ref: &'a str,
    client_secret_ref: Option<&'a str>,
    expires_at: Option<DateTime<Utc>>,
    metadata: Value,
    authorized_at: DateTime<Utc>,
}

const ZOOM_RECORDING_IMPORT_RETENTION_DAYS_SETTING_KEY: &str =
    "privacy.zoom_recording_import_retention_days";
const ZOOM_TRANSCRIPT_RETENTION_DAYS_SETTING_KEY: &str = "privacy.zoom_transcript_retention_days";

impl ZoomStore {
    async fn resolved_retention_policy(
        &self,
        setting_key: &str,
        observed_at: DateTime<Utc>,
    ) -> Result<Value, ZoomError> {
        let settings = ApplicationSettingsStore::new(self.pool.clone());
        settings.repair_declared_settings().await?;
        let retention_days = settings
            .setting(setting_key)
            .await?
            .and_then(|setting| setting.value.as_i64())
            .unwrap_or(0)
            .max(0);
        let expires_at = if retention_days > 0 {
            Some(observed_at + chrono::TimeDelta::days(retention_days))
        } else {
            None
        };
        Ok(json!({
            "setting_key": setting_key,
            "retention_days": retention_days,
            "mode": if retention_days > 0 {
                "delete_after_n_days"
            } else {
                "explicit_remove_only"
            },
            "observed_at": observed_at,
            "expires_at": expires_at,
        }))
    }
}
mod events;
