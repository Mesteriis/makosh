use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use makosh_provider_telemost::models::{
    YandexTelemostCohostPage, YandexTelemostConference, YandexTelemostConferencePatchRequest,
    YandexTelemostCreateConferenceCommand,
};
use makosh_provider_telemost::protocol::{
    YANDEX_TELEMOST_API_BASE_URL, YANDEX_TELEMOST_PROVIDER_KIND_STR,
    sanitize_yandex_telemost_payload, validate_required, validate_telemost_join_url,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path as FsPath, PathBuf};
use uuid::Uuid;

use crate::app::api_support::stores::{domain_stores::*, integration_stores::*, settings_vault::*};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::calendar::ports::CalendarEventQueryPort;
use crate::domains::review::models::{NewReviewItem, NewReviewItemEvidence, ReviewItemKind};
use crate::domains::review::store::ReviewInboxStore;
use crate::integrations::yandex_telemost::client::errors::YandexTelemostError;
use crate::integrations::yandex_telemost::client::models::{
    YANDEX_TELEMOST_WEB_ORIGIN, YandexTelemostAccountListResponse,
    YandexTelemostAccountSetupRequest, YandexTelemostAccountSetupResponse,
    YandexTelemostCapabilityState, YandexTelemostConferenceOpenRequest,
    YandexTelemostConferenceWebviewManifest, YandexTelemostLocalRecordingManifest,
    YandexTelemostLocalRecordingPolicy, YandexTelemostRecordingBridgeRequest,
    YandexTelemostRecordingBridgeResponse, YandexTelemostRetentionCleanupRequest,
    YandexTelemostRetentionCleanupResponse, YandexTelemostRuntimeStatus,
    YandexTelemostSpeakerTimelinePolicy, YandexTelemostTranscriptBridgeRequest,
    YandexTelemostTranscriptBridgeResponse, webview_manifest_for_request,
    yandex_telemost_capabilities,
};
use crate::integrations::yandex_telemost::runtime_bridge::complete_yandex_telemost_transcript_bridge;

use crate::platform::events::bus::yandex_telemost_event_types;
use crate::platform::realtime_conversation::bundle::build_call_bundle_manifest;
use crate::platform::realtime_conversation::events::{
    REALTIME_CONVERSATION_AUDIO_CAPTURE_COMPLETED, REALTIME_CONVERSATION_CALL_BUNDLE_CREATED,
    REALTIME_CONVERSATION_RADAR_SIGNALS_DETECTED, REALTIME_CONVERSATION_SPEAKER_HINT_OBSERVED,
    REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED,
};
use crate::platform::realtime_conversation::models::{
    CallBundleManifest, RealtimeConversationProviderKind, SpeakerTimelineHint,
};
use crate::vault::models::VaultMode;
use crate::workflows::realtime_conversation_memory_pipeline::plan_memory_pipeline;
use crate::workflows::realtime_conversation_radar_projection::{
    RealtimeConversationRadarProjectionContext, call_bundle_radar_candidates,
};
#[path = "yandex_telemost_support.rs"]
mod yandex_telemost_support;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use yandex_telemost_support::*;

const REALTIME_CONVERSATION_RADAR_SIGNAL_OBSERVATION_KIND: &str =
    "REALTIME_CONVERSATION_RADAR_SIGNAL";

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct YandexTelemostAccountsQuery {
    #[serde(default)]
    pub(crate) include_removed: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct YandexTelemostRuntimeStatusQuery {
    pub(crate) account_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct YandexTelemostCohostsQuery {
    #[serde(default)]
    pub(crate) offset: Option<u32>,
    #[serde(default)]
    pub(crate) limit: Option<u16>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct YandexTelemostCapabilitiesResponse {
    pub(crate) provider_kind: &'static str,
    pub(crate) api_base_url: &'static str,
    pub(crate) web_origin: &'static str,
    pub(crate) capabilities: Vec<YandexTelemostCapabilityState>,
    pub(crate) recording_policy: YandexTelemostLocalRecordingManifest,
    pub(crate) speaker_timeline_policy: YandexTelemostSpeakerTimelinePolicy,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct YandexTelemostConferenceOperationResponse {
    pub(crate) account_id: String,
    pub(crate) conference: YandexTelemostConference,
    pub(crate) status: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct YandexTelemostRecordingIntentResponse {
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) join_url: String,
    pub(crate) consent_required: bool,
    pub(crate) source_of_truth: bool,
    pub(crate) local_recording: YandexTelemostLocalRecordingManifest,
    pub(crate) speaker_timeline: YandexTelemostSpeakerTimelinePolicy,
    pub(crate) tauri_commands: serde_json::Value,
}

pub(crate) async fn get_yandex_telemost_capabilities(
    State(state): State<AppState>,
) -> Result<Json<YandexTelemostCapabilitiesResponse>, ApiError> {
    let authorized = matches!(state.vault.status()?.state, VaultMode::Unlocked);
    Ok(Json(YandexTelemostCapabilitiesResponse {
        provider_kind: YANDEX_TELEMOST_PROVIDER_KIND_STR,
        api_base_url: YANDEX_TELEMOST_API_BASE_URL,
        web_origin: YANDEX_TELEMOST_WEB_ORIGIN,
        capabilities: yandex_telemost_capabilities(authorized),
        recording_policy: recording_policy_manifest(),
        speaker_timeline_policy: speaker_timeline_policy(),
    }))
}

pub(crate) async fn get_yandex_telemost_accounts(
    State(state): State<AppState>,
    Query(query): Query<YandexTelemostAccountsQuery>,
) -> Result<Json<YandexTelemostAccountListResponse>, ApiError> {
    Ok(Json(
        yandex_telemost_provider_runtime_store(&state)?
            .list_accounts(query.include_removed)
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_account(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostAccountSetupRequest>,
) -> Result<Json<YandexTelemostAccountSetupResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    Ok(Json(
        store
            .setup_account(&secret_store, &state.vault, &request)
            .await?,
    ))
}

pub(crate) async fn get_yandex_telemost_runtime_status(
    State(state): State<AppState>,
    Query(query): Query<YandexTelemostRuntimeStatusQuery>,
) -> Result<Json<YandexTelemostRuntimeStatus>, ApiError> {
    Ok(Json(
        yandex_telemost_provider_runtime_store(&state)?
            .runtime_status(&query.account_id)
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_retention_cleanup(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<YandexTelemostRetentionCleanupRequest>,
) -> Result<Json<YandexTelemostRetentionCleanupResponse>, ApiError> {
    Ok(Json(
        yandex_telemost_provider_runtime_service(&state)?
            .cleanup_retention(&account_id, &request)
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_conference(
    State(state): State<AppState>,
    Json(command): Json<YandexTelemostCreateConferenceCommand>,
) -> Result<Json<YandexTelemostConferenceOperationResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    let conference = store
        .create_conference(
            &secret_store,
            &state.vault,
            &command.account_id,
            &command.body,
        )
        .await?;
    Ok(Json(YandexTelemostConferenceOperationResponse {
        account_id: command.account_id,
        conference,
        status: "created",
    }))
}

pub(crate) async fn get_yandex_telemost_conference(
    State(state): State<AppState>,
    Path((account_id, conference_id)): Path<(String, String)>,
) -> Result<Json<YandexTelemostConferenceOperationResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    let conference = store
        .get_conference(&secret_store, &state.vault, &account_id, &conference_id)
        .await?;
    Ok(Json(YandexTelemostConferenceOperationResponse {
        account_id,
        conference,
        status: "observed",
    }))
}

pub(crate) async fn patch_yandex_telemost_conference(
    State(state): State<AppState>,
    Path((account_id, conference_id)): Path<(String, String)>,
    Json(request): Json<YandexTelemostConferencePatchRequest>,
) -> Result<Json<YandexTelemostConferenceOperationResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    let conference = store
        .update_conference(
            &secret_store,
            &state.vault,
            &account_id,
            &conference_id,
            &request,
        )
        .await?;
    Ok(Json(YandexTelemostConferenceOperationResponse {
        account_id,
        conference,
        status: "updated",
    }))
}

pub(crate) async fn get_yandex_telemost_cohosts(
    State(state): State<AppState>,
    Path((account_id, conference_id)): Path<(String, String)>,
    Query(query): Query<YandexTelemostCohostsQuery>,
) -> Result<Json<YandexTelemostCohostPage>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    Ok(Json(
        store
            .list_cohosts(
                &secret_store,
                &state.vault,
                &account_id,
                &conference_id,
                query.offset,
                query.limit,
            )
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_webview_manifest(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostConferenceOpenRequest>,
) -> Result<Json<YandexTelemostConferenceWebviewManifest>, ApiError> {
    validate_telemost_join_url(&request.join_url).map_err(YandexTelemostError::from)?;
    let window_label = telemost_window_label(&request.account_id, request.conference_id.as_deref());
    publish_yandex_telemost_companion_event(
        &state,
        yandex_telemost_event_types::WEBVIEW_OPEN_REQUESTED,
        "webview_open_requested",
        &request,
        json!({ "window_label": window_label.clone(), "owner_visible": true, "hidden_headless_mode": "forbidden" }),
    )
    .await?;
    Ok(Json(webview_manifest_for_request(
        &request,
        window_label,
        false,
        false,
    )))
}

pub(crate) async fn post_yandex_telemost_recording_intent(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostConferenceOpenRequest>,
) -> Result<Json<YandexTelemostRecordingIntentResponse>, ApiError> {
    validate_telemost_join_url(&request.join_url).map_err(YandexTelemostError::from)?;
    publish_yandex_telemost_companion_event(
        &state,
        yandex_telemost_event_types::LOCAL_RECORDING_REQUESTED,
        "local_recording_requested",
        &request,
        json!({
            "consent_required": true,
            "source_of_truth": false,
            "audio_format": "mp3",
            "speaker_timeline": "hint_not_truth"
        }),
    )
    .await?;
    Ok(Json(YandexTelemostRecordingIntentResponse {
        account_id: request.account_id,
        conference_id: request.conference_id,
        join_url: request.join_url,
        consent_required: true,
        source_of_truth: false,
        local_recording: recording_policy_manifest(),
        speaker_timeline: speaker_timeline_policy(),
        tauri_commands: json!({
            "open_webview": "open_yandex_telemost_companion",
            "prepare_audio_device": "yandex_telemost_prepare_audio_device",
            "start_recording": "yandex_telemost_recording_start",
            "stop_recording": "yandex_telemost_recording_stop",
            "append_speaker_hint": "yandex_telemost_speaker_timeline_append"
        }),
    }))
}

pub(crate) async fn post_yandex_telemost_runtime_bridge_recording(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostRecordingBridgeRequest>,
) -> Result<Json<YandexTelemostRecordingBridgeResponse>, ApiError> {
    validate_yandex_telemost_recording_bridge_request(&request)?;
    let matched_calendar_event_id = matched_telemost_calendar_event_id(&state, &request).await?;
    let retention_policy = yandex_telemost_local_recording_retention_policy(
        &state,
        recording_bridge_observed_at(&request),
    )
    .await?;
    let materialized = materialize_yandex_telemost_call_bundle(
        &request,
        matched_calendar_event_id,
        retention_policy,
    )?;
    publish_local_recording_completed_event(&state, &request, &materialized).await?;
    publish_realtime_conversation_bootstrap_events(&state, &request, &materialized).await?;
    mirror_radar_candidates_into_review(&state, &request, &materialized).await?;
    Ok(Json(YandexTelemostRecordingBridgeResponse {
        account_id: request.account_id,
        conference_id: request.conference_id,
        recording_session_id: request.recording_session_id,
        bundle_id: materialized.manifest.bundle_id.clone(),
        bundle_root: materialized.bundle_root.to_string_lossy().into_owned(),
        manifest_path: materialized.manifest_path.to_string_lossy().into_owned(),
        follow_up_events: materialized.pipeline_plan.follow_up_events.clone(),
        radar_signal_kinds: materialized
            .radar_candidates
            .iter()
            .map(|candidate| candidate.signal_kind.clone())
            .collect(),
    }))
}

pub(crate) async fn post_yandex_telemost_runtime_bridge_transcript(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostTranscriptBridgeRequest>,
) -> Result<Json<YandexTelemostTranscriptBridgeResponse>, ApiError> {
    let response = complete_yandex_telemost_transcript_bridge(
        &event_store(&state).map_err(yandex_telemost_event_store_access_error)?,
        Some(&state.event_bus),
        &request,
    )
    .await?;
    Ok(Json(response))
}
