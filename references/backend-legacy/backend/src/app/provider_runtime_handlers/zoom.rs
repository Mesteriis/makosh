use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;

use crate::app::api_support::{
    ensure_fixture_routes_enabled,
    stores::{integration_stores::*, settings_vault::*},
};
use crate::app::error::types::ApiError;
use crate::app::signal_hub_support::{
    provider_account_or_not_found, sync_provider_account_signal_connection,
};
use crate::app::state::AppState;
use crate::integrations::zoom::client::errors::ZoomError;
use crate::integrations::zoom::client::models::oauth_models::{
    ZoomOAuthCompleteRequest, ZoomOAuthStartRequest, ZoomOAuthStartResponse,
    ZoomServerToServerAuthorizeRequest, ZoomTokenMaintenanceRequest, ZoomTokenMaintenanceResult,
    ZoomTokenRefreshRequest, ZoomTokenRefreshResult,
};
use crate::integrations::zoom::client::models::{
    ZoomAccountListResponse, ZoomAccountSetupRequest, ZoomAccountSetupResponse,
    ZoomAuditEventResponse, ZoomAuthorizationResult, ZoomLiveAccountSetupRequest,
    ZoomMeetingIngestResult, ZoomMeetingObservationRequest, ZoomRecordingImportAuditResponse,
    ZoomRecordingImportRemoveRequest, ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingObservationRequest, ZoomRecordingRef,
    ZoomRecordingSyncRequest, ZoomRecordingSyncResult, ZoomRetentionCleanupRequest,
    ZoomRetentionCleanupResponse, ZoomRuntimeRemoveRequest, ZoomRuntimeRemoveResponse,
    ZoomRuntimeStartRequest, ZoomRuntimeStatus, ZoomRuntimeStopRequest,
    ZoomTranscriptFileImportRequest, ZoomTranscriptFileImportResult, ZoomTranscriptIngestResult,
    ZoomTranscriptObservationRequest, ZoomWebhookSubscriptionReconcileRequest,
    ZoomWebhookSubscriptionReconcileResult, ZoomWebhookSubscriptionRemoveRequest,
    ZoomWebhookSubscriptionRemoveResult, ZoomWebhookSubscriptionStatusRequest,
    ZoomWebhookSubscriptionStatusResult,
};
use crate::vault::errors::HostVaultError;
use crate::vault::models::VaultMode;

#[path = "zoom_support.rs"]
mod zoom_support;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore;
use zoom_support::*;

const ZOOM_SIGNATURE_HEADER: &str = "x-zm-signature";
const ZOOM_TIMESTAMP_HEADER: &str = "x-zm-request-timestamp";
const ZOOM_WEBHOOK_SIGNATURE_TOLERANCE_SECONDS: i64 = 300;
const ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_ENABLED_SETTING_KEY: &str =
    "privacy.zoom_remote_transcript_download_enabled";
const ZOOM_REMOTE_RECORDING_DOWNLOAD_ENABLED_SETTING_KEY: &str =
    "privacy.zoom_remote_recording_download_enabled";
const ZOOM_REMOTE_RECORDING_DOWNLOAD_NOT_ENABLED: &str =
    "zoom_remote_recording_download_not_enabled";
const ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_NOT_ENABLED: &str =
    "zoom_remote_transcript_download_not_enabled";
type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
pub(crate) struct ZoomAccountsQuery {
    #[serde(default)]
    pub(crate) include_removed: bool,
}

#[derive(Deserialize)]
pub(crate) struct ZoomRuntimeStatusQuery {
    pub(crate) account_id: String,
}

#[derive(Deserialize)]
pub(crate) struct ZoomRecordingImportsQuery {
    #[serde(default = "default_zoom_recording_imports_limit")]
    pub(crate) limit: i64,
}

#[derive(Deserialize)]
pub(crate) struct ZoomAuditEventsQuery {
    #[serde(default = "default_zoom_audit_events_limit")]
    pub(crate) limit: i64,
}

#[derive(Deserialize)]
pub(crate) struct ZoomWebhookSubscriptionStatusQuery {
    pub(crate) account_id: String,
    pub(crate) api_base_url: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ZoomWebhookQuery {
    pub(crate) account_id: String,
}

#[derive(Clone, Debug)]
struct ZoomWebhookTranscriptDownload {
    request: ZoomTranscriptFileImportRequest,
    download_url: String,
    download_token: Option<String>,
}

#[derive(Clone, Debug)]
struct ZoomWebhookRecordingMediaDownload {
    request: ZoomRecordingMediaDownloadRequest,
    download_token: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ZoomCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) capabilities: Vec<ZoomCapabilityStatus>,
    pub(crate) planned_features: Vec<&'static str>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ZoomCapabilityStatus {
    pub(crate) capability: &'static str,
    pub(crate) category: &'static str,
    pub(crate) status: &'static str,
    pub(crate) action_class: &'static str,
    pub(crate) confirmation_required: bool,
    pub(crate) reason: &'static str,
}

pub(crate) async fn get_zoom_capabilities() -> Result<Json<ZoomCapabilitiesResponse>, ApiError> {
    Ok(Json(ZoomCapabilitiesResponse {
        version: "1.0",
        runtime_mode: "fixture_plus_authorized_live_workers",
        capabilities: vec![
            ZoomCapabilityStatus {
                capability: "accounts.fixture",
                category: "accounts",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Fixture Zoom accounts can be registered for local validation.",
            },
            ZoomCapabilityStatus {
                capability: "accounts.live_blocked",
                category: "accounts",
                status: "degraded",
                action_class: "local_write",
                confirmation_required: true,
                reason: "Live Zoom account metadata and secret references can be registered, but provider execution is blocked.",
            },
            ZoomCapabilityStatus {
                capability: "auth.oauth_user",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Zoom OAuth user grants can be exchanged and stored through host-vault secret references.",
            },
            ZoomCapabilityStatus {
                capability: "auth.server_to_server",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Zoom Server-to-Server OAuth account credentials can be exchanged and stored through host-vault secret references.",
            },
            ZoomCapabilityStatus {
                capability: "auth.token_refresh",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Authorized Zoom OAuth and Server-to-Server credentials can be renewed through host-vault token bundle updates.",
            },
            ZoomCapabilityStatus {
                capability: "auth.token_maintenance",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Authorized Zoom accounts can be scanned and expiring token bundles renewed through the same host-vault refresh boundary.",
            },
            ZoomCapabilityStatus {
                capability: "auth.token_rotation_policy",
                category: "authorization",
                status: "available",
                action_class: "read",
                confirmation_required: false,
                reason: "Runtime status exposes the Zoom token rotation policy, refresh due state and failure blocker without exposing raw token material.",
            },
            ZoomCapabilityStatus {
                capability: "token_maintenance.scheduler",
                category: "runtime",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "The local backend scheduler can invoke token maintenance behind Signal Hub and HostVault gates.",
            },
            ZoomCapabilityStatus {
                capability: "runtime.status",
                category: "runtime",
                status: "available",
                action_class: "read",
                confirmation_required: false,
                reason: "Runtime/account lifecycle state is exposed without reading provider secrets.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.meetings",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Meeting observations are stored as provider call evidence and emitted as Zoom events.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.recordings",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Recording observations are event-sourced and sanitized before dispatch.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.transcripts",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Transcript observations are linked to provider call evidence for AI/consistency workflows.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.transcript_files",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Zoom transcript files can be imported from VTT, SRT or plain text into provider call transcript evidence.",
            },
            ZoomCapabilityStatus {
                capability: "provider_sync.recordings",
                category: "ingest",
                status: "available",
                action_class: "external_provider_read",
                confirmation_required: true,
                reason: "Authorized Zoom accounts can manually synchronize cloud recording metadata; provider-side recording media and transcript-like file downloads require owner-visible privacy opt-in settings.",
            },
            ZoomCapabilityStatus {
                capability: "recording_imports.remove",
                category: "retention",
                status: "available",
                action_class: "local_delete",
                confirmation_required: true,
                reason: "Imported Zoom recording blobs can be explicitly removed per account through the local retention control surface, with follow-up audit events.",
            },
            ZoomCapabilityStatus {
                capability: "retention.cleanup",
                category: "retention",
                status: "available",
                action_class: "local_delete",
                confirmation_required: true,
                reason: "Expired Zoom recording imports and transcript evidence can be pruned through the owner-visible retention control surface using stamped expiry intent.",
            },
            ZoomCapabilityStatus {
                capability: "retention.cleanup.scheduler",
                category: "runtime",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "The local backend scheduler can periodically prune expired Zoom recording imports and transcript evidence through the same retention boundary.",
            },
            ZoomCapabilityStatus {
                capability: "provider_sync.recordings.scheduler",
                category: "runtime",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "Started authorized Zoom runtimes can periodically synchronize recent cloud recording metadata through the same HostVault-backed provider sync boundary.",
            },
            ZoomCapabilityStatus {
                capability: "webhooks.verified",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: true,
                reason: "Account-scoped Zoom webhooks are accepted only after URL validation or HMAC signature verification.",
            },
            ZoomCapabilityStatus {
                capability: "webhooks.subscription_management",
                category: "ingest",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Authorized Zoom accounts can reconcile managed event subscriptions through the live provider API using app-owned access tokens.",
            },
            ZoomCapabilityStatus {
                capability: "webhooks.edge_proxy",
                category: "ingest",
                status: "available",
                action_class: "external_ingress",
                confirmation_required: true,
                reason: "The makosh-zoom-edge-proxy binary forwards raw public Zoom webhooks into the protected verified runtime bridge.",
            },
            ZoomCapabilityStatus {
                capability: "calendar_event_matching",
                category: "workflow",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "Zoom meeting observations can be projected into existing Calendar event relations through the downstream workflow boundary.",
            },
            ZoomCapabilityStatus {
                capability: "meeting_participant_identity_resolution",
                category: "workflow",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "Zoom meeting participants can feed conservative identity-candidate generation into the existing Review workflow.",
            },
        ],
        planned_features: vec![],
        unsupported_features: vec![
            "hidden_recording",
            "joining_meetings_as_a_bot_without_explicit_setup",
            "auto_dialing",
            "training_models_on_zoom_content_by_default",
        ],
    }))
}

pub(crate) async fn post_zoom_fixture_account(
    State(state): State<AppState>,
    Json(request): Json<ZoomAccountSetupRequest>,
) -> Result<Json<ZoomAccountSetupResponse>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let response = zoom_provider_runtime_service(&state)?
        .setup_fixture_account(&request)
        .await?;
    sync_zoom_signal_connection(&state, &response.account.account_id).await?;
    Ok(Json(response))
}

pub(crate) async fn post_zoom_account(
    State(state): State<AppState>,
    Json(request): Json<ZoomLiveAccountSetupRequest>,
) -> Result<Json<ZoomAccountSetupResponse>, ApiError> {
    let response = zoom_provider_runtime_service(&state)?
        .setup_live_blocked_account(&request)
        .await?;
    sync_zoom_signal_connection(&state, &response.account.account_id).await?;
    Ok(Json(response))
}

pub(crate) async fn post_zoom_oauth_start(
    State(state): State<AppState>,
    Json(request): Json<ZoomOAuthStartRequest>,
) -> Result<Json<ZoomOAuthStartResponse>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let pending = zoom_provider_runtime_service(&state)?
        .start_oauth(&request)
        .await?;
    let response = pending.response();
    let mut pending_map = state
        .account_setup
        .pending_zoom_oauth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    pending_map.insert(pending.setup_id.clone(), pending);
    Ok(Json(response))
}

pub(crate) async fn post_zoom_oauth_complete(
    State(state): State<AppState>,
    Json(request): Json<ZoomOAuthCompleteRequest>,
) -> Result<Json<ZoomAuthorizationResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    request.validate()?;
    let pending = {
        let mut pending_map = state
            .account_setup
            .pending_zoom_oauth
            .lock()
            .map_err(|_| ApiError::AccountSetupState)?;
        pending_map
            .remove(&request.setup_id)
            .ok_or(ApiError::AccountSetupPendingGrantNotFound)?
    };
    if pending.state != request.state {
        return Err(ApiError::AccountSetupStateMismatch);
    }
    let secret_store = zoom_secret_reference_store(&state)?;
    let result = zoom_provider_runtime_service(&state)?
        .complete_oauth(
            &secret_store,
            &state.vault,
            pending,
            &request.authorization_code,
            request.external_account_id.as_deref(),
        )
        .await?;
    let account = provider_account_or_not_found(&state, &result.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&result.token_secret_ref))
        .await?;
    Ok(Json(result))
}

pub(crate) async fn post_zoom_server_to_server_authorize(
    State(state): State<AppState>,
    Json(request): Json<ZoomServerToServerAuthorizeRequest>,
) -> Result<Json<ZoomAuthorizationResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    let result = zoom_provider_runtime_service(&state)?
        .authorize_server_to_server(&secret_store, &state.vault, &request)
        .await?;
    let account = provider_account_or_not_found(&state, &result.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&result.token_secret_ref))
        .await?;
    Ok(Json(result))
}

pub(crate) async fn post_zoom_oauth_refresh(
    State(state): State<AppState>,
    Json(request): Json<ZoomTokenRefreshRequest>,
) -> Result<Json<ZoomTokenRefreshResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    let result = zoom_provider_runtime_service(&state)?
        .refresh_token(&secret_store, &state.vault, &request)
        .await?;
    let account = provider_account_or_not_found(&state, &result.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&result.token_secret_ref))
        .await?;
    Ok(Json(result))
}

pub(crate) async fn post_zoom_oauth_maintenance(
    State(state): State<AppState>,
    Json(request): Json<ZoomTokenMaintenanceRequest>,
) -> Result<Json<ZoomTokenMaintenanceResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    let result = zoom_provider_runtime_service(&state)?
        .maintain_tokens(&secret_store, &state.vault, &request)
        .await?;
    for item in &result.items {
        if item.status == "failed" {
            continue;
        }
        if let Ok(account) = provider_account_or_not_found(&state, &item.account_id).await {
            sync_provider_account_signal_connection(&state, &account, None).await?;
        }
    }
    Ok(Json(result))
}

pub(crate) async fn post_zoom_provider_sync_recordings(
    State(state): State<AppState>,
    Json(request): Json<ZoomRecordingSyncRequest>,
) -> Result<Json<ZoomRecordingSyncResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    let allow_remote_recording_downloads = zoom_remote_recording_download_enabled(&state).await?;
    let allow_remote_transcript_downloads = zoom_remote_transcript_download_enabled(&state).await?;
    let result = zoom_provider_runtime_service(&state)?
        .sync_recordings(
            &secret_store,
            &state.vault,
            &request,
            allow_remote_recording_downloads,
            allow_remote_transcript_downloads,
        )
        .await?;
    Ok(Json(result))
}

pub(crate) async fn get_zoom_webhook_subscription_status(
    State(state): State<AppState>,
    Query(query): Query<ZoomWebhookSubscriptionStatusQuery>,
) -> Result<Json<ZoomWebhookSubscriptionStatusResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .webhook_subscription_status(
                &secret_store,
                &state.vault,
                &ZoomWebhookSubscriptionStatusRequest {
                    account_id: query.account_id,
                    api_base_url: query.api_base_url,
                },
            )
            .await?,
    ))
}

pub(crate) async fn post_zoom_webhook_subscription_reconcile(
    State(state): State<AppState>,
    Json(request): Json<ZoomWebhookSubscriptionReconcileRequest>,
) -> Result<Json<ZoomWebhookSubscriptionReconcileResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .reconcile_webhook_subscription(&secret_store, &state.vault, &request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_webhook_subscription_remove(
    State(state): State<AppState>,
    Json(request): Json<ZoomWebhookSubscriptionRemoveRequest>,
) -> Result<Json<ZoomWebhookSubscriptionRemoveResult>, ApiError> {
    require_zoom_unlocked_host_vault(&state)?;
    let secret_store = zoom_secret_reference_store(&state)?;
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .remove_webhook_subscription(&secret_store, &state.vault, &request)
            .await?,
    ))
}

pub(crate) async fn get_zoom_accounts(
    State(state): State<AppState>,
    Query(query): Query<ZoomAccountsQuery>,
) -> Result<Json<ZoomAccountListResponse>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .list_accounts(query.include_removed)
            .await?,
    ))
}

pub(crate) async fn get_zoom_runtime_status(
    State(state): State<AppState>,
    Query(query): Query<ZoomRuntimeStatusQuery>,
) -> Result<Json<ZoomRuntimeStatus>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .runtime_status(&query.account_id)
            .await?,
    ))
}

pub(crate) async fn get_zoom_account_runtime_status(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<ZoomRuntimeStatus>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .runtime_status(&account_id)
            .await?,
    ))
}

pub(crate) async fn get_zoom_recording_imports(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Query(query): Query<ZoomRecordingImportsQuery>,
) -> Result<Json<ZoomRecordingImportAuditResponse>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .list_recording_imports(&account_id, query.limit)
            .await?,
    ))
}

pub(crate) async fn post_zoom_recording_import_remove(
    State(state): State<AppState>,
    Path((account_id, attachment_id)): Path<(String, String)>,
    Json(request): Json<ZoomRecordingImportRemoveRequest>,
) -> Result<Json<ZoomRecordingImportRemoveResponse>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .remove_recording_import(&account_id, &attachment_id, &request)
            .await?,
    ))
}

pub(crate) async fn get_zoom_audit_events(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Query(query): Query<ZoomAuditEventsQuery>,
) -> Result<Json<ZoomAuditEventResponse>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .list_audit_events(&account_id, query.limit)
            .await?,
    ))
}

pub(crate) async fn post_zoom_retention_cleanup(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<ZoomRetentionCleanupRequest>,
) -> Result<Json<ZoomRetentionCleanupResponse>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .cleanup_retention(&account_id, &request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_runtime_start(
    State(state): State<AppState>,
    Json(request): Json<ZoomRuntimeStartRequest>,
) -> Result<Json<ZoomRuntimeStatus>, ApiError> {
    let status = zoom_provider_runtime_service(&state)?
        .start_runtime(&request)
        .await?;
    sync_zoom_signal_connection(&state, &status.account_id).await?;
    Ok(Json(status))
}

pub(crate) async fn post_zoom_runtime_stop(
    State(state): State<AppState>,
    Json(request): Json<ZoomRuntimeStopRequest>,
) -> Result<Json<ZoomRuntimeStatus>, ApiError> {
    let status = zoom_provider_runtime_service(&state)?
        .stop_runtime(&request)
        .await?;
    sync_zoom_signal_connection(&state, &status.account_id).await?;
    Ok(Json(status))
}

pub(crate) async fn post_zoom_runtime_remove(
    State(state): State<AppState>,
    Json(request): Json<ZoomRuntimeRemoveRequest>,
) -> Result<Json<ZoomRuntimeRemoveResponse>, ApiError> {
    let response = zoom_provider_runtime_service(&state)?
        .remove_runtime(&request)
        .await?;
    sync_zoom_signal_connection(&state, &response.account_id).await?;
    Ok(Json(response))
}

pub(crate) async fn post_zoom_runtime_bridge_meeting(
    State(state): State<AppState>,
    Json(request): Json<ZoomMeetingObservationRequest>,
) -> Result<Json<ZoomMeetingIngestResult>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .observe_meeting(&request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_runtime_bridge_recording(
    State(state): State<AppState>,
    Json(request): Json<ZoomRecordingObservationRequest>,
) -> Result<Json<ZoomRecordingIngestResult>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .observe_recording(&request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_runtime_bridge_transcript(
    State(state): State<AppState>,
    Json(request): Json<ZoomTranscriptObservationRequest>,
) -> Result<Json<ZoomTranscriptIngestResult>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .observe_transcript(&request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_runtime_bridge_transcript_file(
    State(state): State<AppState>,
    Json(request): Json<ZoomTranscriptFileImportRequest>,
) -> Result<Json<ZoomTranscriptFileImportResult>, ApiError> {
    Ok(Json(
        zoom_provider_runtime_service(&state)?
            .import_transcript_file(&request)
            .await?,
    ))
}

pub(crate) async fn post_zoom_runtime_bridge_webhook(
    State(state): State<AppState>,
    Query(query): Query<ZoomWebhookQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let account_id = validate_zoom_webhook_account_id(&query.account_id)?;
    let webhook_secret = read_zoom_webhook_secret(&state, &account_id).await?;
    let envelope = zoom_webhook_envelope(&body)?;
    let event = zoom_webhook_event(&envelope)?;

    if event == "endpoint.url_validation" {
        let plain_token = zoom_webhook_plain_token(&envelope)?;
        let encrypted_token = zoom_webhook_validation_token(&webhook_secret, &plain_token)?;
        return Ok(Json(json!({
            "plainToken": plain_token,
            "encryptedToken": encrypted_token,
        })));
    }

    verify_zoom_webhook_signature(&webhook_secret, &headers, &body)?;

    let service = zoom_provider_runtime_service(&state)?;
    if event.starts_with("meeting.") {
        let request = zoom_meeting_observation_from_webhook(&account_id, event, &envelope)?;
        let result = service.observe_meeting(&request).await?;
        return Ok(Json(json!({
            "account_id": account_id,
            "event": event,
            "status": "recorded",
            "meeting": result,
        })));
    }

    if event.starts_with("recording.") {
        let recording_requests =
            zoom_recording_observations_from_webhook(&account_id, event, &envelope)?;
        let recording_downloads =
            zoom_recording_media_downloads_from_recording_webhook(&account_id, event, &envelope)?;
        let transcript_downloads =
            zoom_transcript_downloads_from_recording_webhook(&account_id, event, &envelope)?;
        let allow_remote_recording_downloads =
            zoom_remote_recording_download_enabled(&state).await?;
        let allow_remote_transcript_downloads =
            zoom_remote_transcript_download_enabled(&state).await?;
        let mut recordings = Vec::with_capacity(recording_requests.len());
        for request in recording_requests {
            recordings.push(service.observe_recording(&request).await?);
        }
        let mut recording_imports = Vec::with_capacity(recording_downloads.len());
        for download in recording_downloads {
            if !allow_remote_recording_downloads {
                recording_imports.push(json!({
                    "status": "blocked",
                    "recording_id": download.request.recording.recording_id,
                    "error": ZOOM_REMOTE_RECORDING_DOWNLOAD_NOT_ENABLED,
                }));
                continue;
            }
            match service
                .import_recording_media_download(
                    &download.request,
                    download.download_token.as_deref(),
                )
                .await
            {
                Ok(result) => recording_imports.push(json!({
                    "status": "recorded",
                    "recording": result,
                })),
                Err(error) => recording_imports.push(json!({
                    "status": "failed",
                    "recording_id": download.request.recording.recording_id,
                    "error": error.to_string(),
                })),
            }
        }
        let mut transcript_imports = Vec::with_capacity(transcript_downloads.len());
        for download in transcript_downloads {
            if !allow_remote_transcript_downloads {
                transcript_imports.push(json!({
                    "status": "blocked",
                    "transcript_id": download.request.transcript_id,
                    "error": ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_NOT_ENABLED,
                }));
                continue;
            }
            match service
                .import_transcript_file_download(
                    &download.request,
                    &download.download_url,
                    download.download_token.as_deref(),
                )
                .await
            {
                Ok(result) => transcript_imports.push(json!({
                    "status": "recorded",
                    "transcript": result,
                })),
                Err(error) => transcript_imports.push(json!({
                    "status": "failed",
                    "transcript_id": download.request.transcript_id,
                    "error": error.to_string(),
                })),
            }
        }
        return Ok(Json(json!({
            "account_id": account_id,
            "event": event,
            "status": if recordings.is_empty() { "ignored" } else { "recorded" },
            "recordings": recordings,
            "recording_imports": recording_imports,
            "transcript_imports": transcript_imports,
        })));
    }

    Ok(Json(json!({
        "account_id": account_id,
        "event": event,
        "status": "ignored",
        "reason": "unsupported_zoom_webhook_event",
    })))
}
