use axum::Json;
use axum::extract::{Path, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::app::error::types::ApiError;
use crate::app::signal_hub_support::run_signal_hub_health_check;
use crate::app::state::AppState;
use crate::application::signal_hub_replay::SignalHubReplayService;
use crate::domains::signal_hub::capabilities::SignalHubCapabilityService;
use crate::domains::signal_hub::connections::SignalHubConnectionService;
use crate::domains::signal_hub::controls::{
    SignalHubControlRequest, SignalHubControlResult, SignalHubControlService,
};
use crate::domains::signal_hub::fixture_source::{
    SignalFixtureEmission, SignalFixtureEmitRequest, SignalFixtureSource,
    SignalFixtureSourceService,
};
use crate::domains::signal_hub::profiles::SignalHubProfileService;
use crate::domains::signal_hub::replay_contracts::{
    SignalReplayRequest, SignalReplayRequestCreate,
};
use crate::domains::signal_hub::store::{
    FixtureRestoreReport, SignalCapability, SignalConnection, SignalConnectionCreate,
    SignalConnectionUpdate, SignalHealth, SignalHealthCheckRequest, SignalHubError, SignalHubStore,
    SignalProfileCreate, SignalProfilePolicy, SignalProfileSummary, SignalProfileUpdate,
    SignalRuntimeState, SignalRuntimeStateUpdate, SignalSource,
};
use crate::platform::settings::store::ApplicationSettingsStore;
use makosh_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};
use makosh_signal_hub_postgres::raw_signals::adapter::RawSignalStore;

#[derive(Serialize)]
pub(crate) struct SignalHubSourcesResponse {
    items: Vec<SignalSource>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubPoliciesResponse {
    items: Vec<SignalPolicyDto>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubProfilesResponse {
    items: Vec<SignalProfileSummary>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubConnectionsResponse {
    items: Vec<SignalConnection>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubCapabilitiesResponse {
    items: Vec<SignalCapability>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubConnectionResponse {
    item: SignalConnection,
}

#[derive(Serialize)]
pub(crate) struct SignalHubHealthResponse {
    items: Vec<SignalHealth>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubRuntimeStatesResponse {
    items: Vec<SignalRuntimeState>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubReplayRequestsResponse {
    items: Vec<SignalReplayRequest>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubFixtureEmissionResponse {
    item: SignalFixtureEmission,
}

#[derive(Serialize)]
pub(crate) struct SignalHubFixtureSourcesResponse {
    items: Vec<SignalFixtureSource>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubPolicyRequest {
    scope: String,
    source_code: Option<String>,
    connection_id: Option<String>,
    event_pattern: Option<String>,
    mode: String,
    reason: String,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub(crate) struct SignalHubCreatePolicyResponse {
    id: String,
}

#[derive(Serialize)]
pub(crate) struct SignalHubControlResponse {
    source_code: Option<String>,
    connection_id: Option<String>,
    event_pattern: Option<String>,
    policy_id: Option<String>,
    cleared_count: u64,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubRuntimeStateRequest {
    source_code: String,
    runtime_kind: String,
    state: String,
    #[serde(default = "empty_json_object")]
    metadata: serde_json::Value,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubHealthCheckBody {
    source_code: String,
    connection_id: Option<String>,
    runtime_kind: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubControlBody {
    scope: String,
    source_code: Option<String>,
    connection_id: Option<String>,
    event_pattern: Option<String>,
    reason: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubReplayRequestBody {
    source_code: Option<String>,
    connection_id: Option<String>,
    event_pattern: Option<String>,
    from_position: Option<i64>,
    to_position: Option<i64>,
    from_time: Option<DateTime<Utc>>,
    to_time: Option<DateTime<Utc>>,
    target_consumer: Option<String>,
    target_projection: Option<String>,
    #[serde(default = "empty_json_object")]
    metadata: serde_json::Value,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubConnectionCreateRequest {
    source_code: String,
    display_name: String,
    status: String,
    profile: Option<String>,
    #[serde(default = "empty_json_object")]
    settings: serde_json::Value,
    secret_ref: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubConnectionUpdateRequest {
    display_name: Option<String>,
    status: Option<String>,
    profile: Option<String>,
    settings: Option<serde_json::Value>,
    secret_ref: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubProfileCreateRequest {
    code: String,
    display_name: String,
    description: String,
    #[serde(default)]
    source_policies: Vec<SignalProfilePolicy>,
}

#[derive(Deserialize)]
pub(crate) struct SignalHubProfileUpdateRequest {
    display_name: Option<String>,
    description: Option<String>,
    source_policies: Option<Vec<SignalProfilePolicy>>,
}

#[derive(Serialize)]
struct SignalPolicyDto {
    scope: String,
    source_code: Option<String>,
    connection_id: Option<String>,
    event_pattern: Option<String>,
    mode: String,
    reason: String,
    expires_at: Option<DateTime<Utc>>,
}

pub(crate) async fn get_signal_hub_sources(
    State(state): State<AppState>,
) -> Result<Json<SignalHubSourcesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalHubStore::new(pool).list_sources().await?;

    Ok(Json(SignalHubSourcesResponse { items }))
}

pub(crate) async fn get_signal_hub_source(
    State(state): State<AppState>,
    Path(source_code): Path<String>,
) -> Result<Json<SignalSource>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubStore::new(pool).get_source(&source_code).await?;

    Ok(Json(item))
}

#[derive(Deserialize)]
pub(crate) struct SignalHubCapabilitiesQuery {
    source_code: Option<String>,
    connection_id: Option<String>,
}

pub(crate) async fn get_signal_hub_capabilities(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SignalHubCapabilitiesQuery>,
) -> Result<Json<SignalHubCapabilitiesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalHubCapabilityService::new(SignalHubStore::new(pool))
        .list_capabilities(query.source_code.as_deref(), query.connection_id.as_deref())
        .await?;

    Ok(Json(SignalHubCapabilitiesResponse { items }))
}

pub(crate) async fn post_signal_hub_restore_system_fixture(
    State(state): State<AppState>,
) -> Result<Json<FixtureRestoreReport>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let report = SignalHubStore::new(pool).restore_system_sources().await?;

    Ok(Json(report))
}

pub(crate) async fn get_signal_hub_fixture_sources(
    State(state): State<AppState>,
) -> Result<Json<SignalHubFixtureSourcesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalFixtureSourceService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .list_fixture_sources()?;

    Ok(Json(SignalHubFixtureSourcesResponse { items }))
}

pub(crate) async fn get_signal_hub_profiles(
    State(state): State<AppState>,
) -> Result<Json<SignalHubProfilesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalHubProfileService::new(
        SignalHubStore::new(pool.clone()),
        ApplicationSettingsStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .list_profiles()
    .await?;

    Ok(Json(SignalHubProfilesResponse { items }))
}

pub(crate) async fn post_signal_hub_profile(
    State(state): State<AppState>,
    Json(body): Json<SignalHubProfileCreateRequest>,
) -> Result<Json<SignalProfileSummary>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubProfileService::new(
        SignalHubStore::new(pool.clone()),
        ApplicationSettingsStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .create_profile(&SignalProfileCreate {
        code: body.code,
        display_name: body.display_name,
        description: body.description,
        source_policies: body.source_policies,
    })
    .await?;

    Ok(Json(item))
}

pub(crate) async fn post_signal_hub_apply_profile(
    State(state): State<AppState>,
    Path(profile_code): Path<String>,
) -> Result<Json<SignalProfileSummary>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubProfileService::new(
        SignalHubStore::new(pool.clone()),
        ApplicationSettingsStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .apply_profile(&profile_code)
    .await?;

    Ok(Json(item))
}

pub(crate) async fn patch_signal_hub_profile(
    State(state): State<AppState>,
    Path(profile_code): Path<String>,
    Json(body): Json<SignalHubProfileUpdateRequest>,
) -> Result<Json<SignalProfileSummary>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubProfileService::new(
        SignalHubStore::new(pool.clone()),
        ApplicationSettingsStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .update_profile(&SignalProfileUpdate {
        code: profile_code,
        display_name: body.display_name,
        description: body.description,
        source_policies: body.source_policies,
    })
    .await?;

    Ok(Json(item))
}

pub(crate) async fn delete_signal_hub_profile(
    State(state): State<AppState>,
    Path(profile_code): Path<String>,
) -> Result<Json<SignalProfileSummary>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubProfileService::new(
        SignalHubStore::new(pool.clone()),
        ApplicationSettingsStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .remove_profile(&profile_code)
    .await?;

    Ok(Json(item))
}

pub(crate) async fn post_signal_hub_enable_source(
    State(state): State<AppState>,
    Path(source_code): Path<String>,
) -> Result<Json<SignalHubControlResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubControlService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .enable_source(&source_code, None)
    .await?;

    Ok(Json(control_response(item)))
}

pub(crate) async fn post_signal_hub_disable_source(
    State(state): State<AppState>,
    Path(source_code): Path<String>,
) -> Result<Json<SignalHubControlResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubControlService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .disable_source(&source_code, None)
    .await?;

    Ok(Json(control_response(item)))
}

pub(crate) async fn get_signal_hub_connections(
    State(state): State<AppState>,
) -> Result<Json<SignalHubConnectionsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalHubStore::new(pool).list_connections().await?;

    Ok(Json(SignalHubConnectionsResponse { items }))
}

pub(crate) async fn post_signal_hub_connection(
    State(state): State<AppState>,
    Json(request): Json<SignalHubConnectionCreateRequest>,
) -> Result<Json<SignalHubConnectionResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubConnectionService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .create_connection(&SignalConnectionCreate {
        source_code: request.source_code,
        display_name: request.display_name,
        status: request.status,
        profile: request.profile,
        settings: request.settings,
        secret_ref: request.secret_ref,
    })
    .await?;

    Ok(Json(SignalHubConnectionResponse { item }))
}

pub(crate) async fn patch_signal_hub_connection(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
    Json(request): Json<SignalHubConnectionUpdateRequest>,
) -> Result<Json<SignalHubConnectionResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubConnectionService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .update_connection(&SignalConnectionUpdate {
        id: connection_id,
        display_name: request.display_name,
        status: request.status,
        profile: request.profile,
        settings: request.settings,
        secret_ref: request.secret_ref,
    })
    .await?;

    Ok(Json(SignalHubConnectionResponse { item }))
}

pub(crate) async fn delete_signal_hub_connection(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
) -> Result<Json<SignalHubConnectionResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubConnectionService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .remove_connection(&connection_id)
    .await?;

    Ok(Json(SignalHubConnectionResponse { item }))
}

pub(crate) async fn get_signal_hub_health(
    State(state): State<AppState>,
) -> Result<Json<SignalHubHealthResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalHubStore::new(pool).list_health().await?;

    Ok(Json(SignalHubHealthResponse { items }))
}

pub(crate) async fn post_signal_hub_health_check(
    State(state): State<AppState>,
    Json(request): Json<SignalHubHealthCheckBody>,
) -> Result<Json<SignalHealth>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = run_signal_hub_health_check(
        &state.config,
        pool,
        &SignalHealthCheckRequest {
            source_code: request.source_code,
            connection_id: request.connection_id.and_then(non_empty_string),
            runtime_kind: request.runtime_kind.and_then(non_empty_string),
        },
    )
    .await?;

    Ok(Json(item))
}

pub(crate) async fn get_signal_hub_runtime_states(
    State(state): State<AppState>,
) -> Result<Json<SignalHubRuntimeStatesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalHubStore::new(pool).list_runtime_states().await?;

    Ok(Json(SignalHubRuntimeStatesResponse { items }))
}

pub(crate) async fn post_signal_hub_runtime_state(
    State(state): State<AppState>,
    Json(request): Json<SignalHubRuntimeStateRequest>,
) -> Result<Json<SignalRuntimeState>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let runtime = SignalHubStore::new(pool)
        .set_runtime_state(&SignalRuntimeStateUpdate {
            source_code: request.source_code,
            runtime_kind: request.runtime_kind,
            state: request.state,
            metadata: request.metadata,
        })
        .await?;

    Ok(Json(runtime))
}

pub(crate) async fn get_signal_hub_replay_requests(
    State(state): State<AppState>,
) -> Result<Json<SignalHubReplayRequestsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = SignalHubStore::new(pool).list_replay_requests().await?;

    Ok(Json(SignalHubReplayRequestsResponse { items }))
}

pub(crate) async fn post_signal_hub_replay_request(
    State(state): State<AppState>,
    Json(request): Json<SignalHubReplayRequestBody>,
) -> Result<Json<SignalReplayRequest>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let replay_request = SignalHubReplayService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .request_replay(&SignalReplayRequestCreate {
        source_code: request.source_code.and_then(non_empty_string),
        connection_id: request.connection_id.and_then(non_empty_string),
        event_pattern: request.event_pattern.and_then(non_empty_string),
        from_position: request.from_position,
        to_position: request.to_position,
        from_time: request.from_time,
        to_time: request.to_time,
        target_consumer: request.target_consumer.and_then(non_empty_string),
        target_projection: request.target_projection.and_then(non_empty_string),
        requested_by: "makosh-frontend".to_owned(),
        metadata: request.metadata,
    })
    .await?;

    Ok(Json(replay_request))
}

pub(crate) async fn post_signal_hub_emit_fixture_signal(
    State(state): State<AppState>,
    Path(fixture_id): Path<String>,
) -> Result<Json<SignalHubFixtureEmissionResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalFixtureSourceService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .emit_fixture(&SignalFixtureEmitRequest { fixture_id })
    .await?;

    Ok(Json(SignalHubFixtureEmissionResponse { item }))
}

pub(crate) async fn get_signal_hub_policies(
    State(state): State<AppState>,
) -> Result<Json<SignalHubPoliciesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = RawSignalStore::new(pool)
        .list_active_policies()
        .await
        .map_err(SignalHubError::from)?
        .into_iter()
        .map(policy_to_dto)
        .collect();

    Ok(Json(SignalHubPoliciesResponse { items }))
}

pub(crate) async fn post_signal_hub_policy(
    State(state): State<AppState>,
    Json(request): Json<SignalHubPolicyRequest>,
) -> Result<Json<SignalHubCreatePolicyResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let policy = policy_from_request(request)?;
    let id = SignalHubStore::new(pool).create_policy(&policy).await?;

    Ok(Json(SignalHubCreatePolicyResponse { id: id.to_string() }))
}

pub(crate) async fn post_signal_hub_mute_signals(
    State(state): State<AppState>,
    Json(request): Json<SignalHubControlBody>,
) -> Result<Json<SignalHubControlResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubControlService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .mute_signals(&control_request_from_body(request)?)
    .await?;

    Ok(Json(control_response(item)))
}

pub(crate) async fn post_signal_hub_unmute_signals(
    State(state): State<AppState>,
    Json(request): Json<SignalHubControlBody>,
) -> Result<Json<SignalHubControlResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubControlService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .unmute_signals(&control_request_from_body(request)?)
    .await?;

    Ok(Json(control_response(item)))
}

pub(crate) async fn post_signal_hub_pause_signals(
    State(state): State<AppState>,
    Json(request): Json<SignalHubControlBody>,
) -> Result<Json<SignalHubControlResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubControlService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .pause_signals(&control_request_from_body(request)?)
    .await?;

    Ok(Json(control_response(item)))
}

pub(crate) async fn post_signal_hub_resume_signals(
    State(state): State<AppState>,
    Json(request): Json<SignalHubControlBody>,
) -> Result<Json<SignalHubControlResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = SignalHubControlService::new(
        SignalHubStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool),
    )
    .resume_signals(&control_request_from_body(request)?)
    .await?;

    Ok(Json(control_response(item)))
}

fn policy_from_request(request: SignalHubPolicyRequest) -> Result<SignalPolicy, ApiError> {
    let scope = SignalPolicyScope::parse(request.scope.trim()).ok_or_else(|| {
        ApiError::SignalHub(SignalHubError::InvalidPolicyScope(request.scope.clone()))
    })?;
    let mode = SignalPolicyMode::parse(request.mode.trim()).ok_or_else(|| {
        ApiError::SignalHub(SignalHubError::InvalidPolicyMode(request.mode.clone()))
    })?;

    Ok(SignalPolicy {
        scope,
        source_code: request.source_code.and_then(non_empty_string),
        connection_id: request.connection_id.and_then(non_empty_string),
        event_pattern: request.event_pattern.and_then(non_empty_string),
        mode,
        reason: non_empty_string(request.reason).unwrap_or_else(|| "user policy".to_owned()),
        expires_at: request.expires_at,
    })
}

fn policy_to_dto(policy: SignalPolicy) -> SignalPolicyDto {
    SignalPolicyDto {
        scope: policy.scope.as_str().to_owned(),
        source_code: policy.source_code,
        connection_id: policy.connection_id,
        event_pattern: policy.event_pattern,
        mode: policy.mode.as_str().to_owned(),
        reason: policy.reason,
        expires_at: policy.expires_at,
    }
}

fn control_request_from_body(
    request: SignalHubControlBody,
) -> Result<SignalHubControlRequest, ApiError> {
    let scope = SignalPolicyScope::parse(request.scope.trim()).ok_or_else(|| {
        ApiError::SignalHub(SignalHubError::InvalidPolicyScope(request.scope.clone()))
    })?;
    Ok(SignalHubControlRequest {
        scope,
        source_code: request.source_code.and_then(non_empty_string),
        connection_id: request.connection_id.and_then(non_empty_string),
        event_pattern: request.event_pattern.and_then(non_empty_string),
        reason: request
            .reason
            .and_then(non_empty_string)
            .unwrap_or_else(|| "owner control".to_owned()),
    })
}

fn control_response(item: SignalHubControlResult) -> SignalHubControlResponse {
    SignalHubControlResponse {
        source_code: item.source_code,
        connection_id: item.connection_id,
        event_pattern: item.event_pattern,
        policy_id: item.policy_id,
        cleared_count: item.cleared_count,
    }
}

fn non_empty_string(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn empty_json_object() -> serde_json::Value {
    serde_json::json!({})
}
