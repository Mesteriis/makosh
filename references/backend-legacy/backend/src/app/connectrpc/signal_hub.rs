use std::sync::Arc;

use chrono::{DateTime, Utc};
use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, Router as ConnectRouter, ServiceRequest,
    ServiceResult,
};
use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::app::signal_hub_support::run_signal_hub_health_check;
use crate::application::signal_hub_replay::SignalHubReplayService;
use crate::domains::signal_hub::capabilities::SignalHubCapabilityService;
use crate::domains::signal_hub::connections::SignalHubConnectionService;
use crate::domains::signal_hub::controls::{SignalHubControlRequest, SignalHubControlService};
use crate::domains::signal_hub::fixture_source::{
    SignalFixtureEmitRequest, SignalFixtureSource, SignalFixtureSourceService,
};
use crate::domains::signal_hub::profiles::SignalHubProfileService;
use crate::domains::signal_hub::replay_contracts::{
    SignalReplayRequest, SignalReplayRequestCreate,
};
use crate::domains::signal_hub::store::{
    SignalCapability, SignalConnection, SignalConnectionCreate, SignalConnectionUpdate,
    SignalHealth, SignalHealthCheckRequest as DomainSignalHealthCheckRequest, SignalHubError,
    SignalHubStore, SignalProfileCreate, SignalProfilePolicy, SignalProfileSummary,
    SignalProfileUpdate, SignalRuntimeState, SignalRuntimeStateUpdate, SignalSource,
};
use crate::platform::config::app_config::AppConfig;
use crate::platform::settings::store::ApplicationSettingsStore;
use makosh_connectrpc_contracts::makosh::signal_hub::v1::{
    ApplyProfileRequest, ApplyProfileResponse, CreateConnectionRequest, CreateConnectionResponse,
    CreatePolicyRequest, CreatePolicyResponse, CreateProfileRequest, CreateProfileResponse,
    DisableSignalsRequest, DisableSignalsResponse, DisableSourceRequest, DisableSourceResponse,
    EmitFixtureSignalRequest, EmitFixtureSignalResponse, EnableSignalsRequest,
    EnableSignalsResponse, EnableSourceRequest, EnableSourceResponse, GetSourceRequest,
    GetSourceResponse, ListCapabilitiesRequest, ListCapabilitiesResponse, ListConnectionsRequest,
    ListConnectionsResponse, ListFixtureSourcesRequest, ListFixtureSourcesResponse,
    ListHealthRequest, ListHealthResponse, ListPoliciesRequest, ListPoliciesResponse,
    ListProfilesRequest, ListProfilesResponse, ListReplayRequestsRequest,
    ListReplayRequestsResponse, ListRuntimeStatesRequest, ListRuntimeStatesResponse,
    ListSourcesRequest, ListSourcesResponse, MuteSignalsRequest, MuteSignalsResponse,
    PauseSignalsRequest, PauseSignalsResponse, RemoveConnectionRequest, RemoveConnectionResponse,
    RemoveProfileRequest, RemoveProfileResponse, RequestReplayRequest, RequestReplayResponse,
    RestoreSystemFixtureRequest, RestoreSystemFixtureResponse, ResumeSignalsRequest,
    ResumeSignalsResponse, RunHealthCheckRequest, RunHealthCheckResponse,
    SignalCapability as ProtoSignalCapability, SignalConnection as ProtoSignalConnection,
    SignalFixtureSource as ProtoSignalFixtureSource, SignalHealth as ProtoSignalHealth,
    SignalHubService, SignalHubServiceExt, SignalPolicy as ProtoSignalPolicy,
    SignalProfile as ProtoSignalProfile, SignalProfilePolicy as ProtoSignalProfilePolicy,
    SignalReplayRequest as ProtoSignalReplayRequest, SignalRuntimeState as ProtoSignalRuntimeState,
    SignalSource as ProtoSignalSource, UnmuteSignalsRequest, UnmuteSignalsResponse,
    UpdateConnectionRequest, UpdateConnectionResponse, UpdateProfileRequest, UpdateProfileResponse,
    UpdateRuntimeStateRequest, UpdateRuntimeStateResponse,
};

#[path = "signal_hub_service_impl.rs"]
#[allow(refining_impl_trait)]
mod signal_hub_service_impl;
use makosh_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};
use makosh_signal_hub_postgres::raw_signals::adapter::RawSignalStore;

pub(crate) fn register(
    router: ConnectRouter,
    pool: Option<PgPool>,
    config: AppConfig,
) -> ConnectRouter {
    let Some(pool) = pool else {
        return router;
    };

    Arc::new(SignalHubConnectService::new(pool, config)).register(router)
}

struct SignalHubConnectService {
    config: AppConfig,
    capability_service: SignalHubCapabilityService,
    fixture_service: SignalFixtureSourceService,
    connection_service: SignalHubConnectionService,
    control_service: SignalHubControlService,
    profile_service: SignalHubProfileService,
    replay_service: SignalHubReplayService,
    store: SignalHubStore,
}

impl SignalHubConnectService {
    fn new(pool: PgPool, config: AppConfig) -> Self {
        let store = SignalHubStore::new(pool.clone());
        Self {
            config,
            capability_service: SignalHubCapabilityService::new(store.clone()),
            fixture_service: SignalFixtureSourceService::new(
                store.clone(),
                makosh_events_postgres::store::EventStore::new(pool.clone()),
            ),
            connection_service: SignalHubConnectionService::new(
                store.clone(),
                makosh_events_postgres::store::EventStore::new(pool.clone()),
            ),
            control_service: SignalHubControlService::new(
                store.clone(),
                makosh_events_postgres::store::EventStore::new(pool.clone()),
            ),
            profile_service: SignalHubProfileService::new(
                store.clone(),
                ApplicationSettingsStore::new(pool.clone()),
                makosh_events_postgres::store::EventStore::new(pool.clone()),
            ),
            replay_service: SignalHubReplayService::new(
                store.clone(),
                makosh_events_postgres::store::EventStore::new(pool),
            ),
            store,
        }
    }
}

#[allow(refining_impl_trait)]
fn proto_source(source: SignalSource) -> ProtoSignalSource {
    ProtoSignalSource {
        id: source.id.to_string(),
        code: source.code,
        display_name: source.display_name,
        category: source.category,
        source_kind: source.source_kind,
        default_enabled: source.default_enabled,
        supports_connections: source.supports_connections,
        supports_runtime: source.supports_runtime,
        supports_replay: source.supports_replay,
        supports_pause: source.supports_pause,
        supports_mute: source.supports_mute,
        capability_schema_version: source.capability_schema_version,
        created_at: timestamp_to_string(source.created_at),
        updated_at: timestamp_to_string(source.updated_at),
        ..Default::default()
    }
}

fn proto_connection(connection: SignalConnection) -> ProtoSignalConnection {
    ProtoSignalConnection {
        id: connection.id.to_string(),
        source_code: connection.source_code,
        display_name: connection.display_name,
        status: connection.status,
        profile: connection.profile,
        secret_ref: connection.secret_ref,
        connected_at: connection.connected_at.map(timestamp_to_string),
        last_seen_at: connection.last_seen_at.map(timestamp_to_string),
        last_signal_at: connection.last_signal_at.map(timestamp_to_string),
        last_sync_at: connection.last_sync_at.map(timestamp_to_string),
        created_at: timestamp_to_string(connection.created_at),
        updated_at: timestamp_to_string(connection.updated_at),
        settings_json: connection.settings.to_string(),
        ..Default::default()
    }
}

fn proto_fixture_source(fixture: SignalFixtureSource) -> ProtoSignalFixtureSource {
    ProtoSignalFixtureSource {
        fixture_id: fixture.fixture_id,
        source_code: fixture.source_code,
        event_type: fixture.event_type,
        correlation_id: fixture.correlation_id,
        occurred_at: timestamp_to_string(fixture.occurred_at),
        summary: fixture.summary,
        ..Default::default()
    }
}

fn proto_profile(profile: SignalProfileSummary) -> ProtoSignalProfile {
    ProtoSignalProfile {
        id: profile.id,
        code: profile.code,
        display_name: profile.display_name,
        description: profile.description,
        policy_count: u32::try_from(profile.policy_count).unwrap_or(u32::MAX),
        is_system: profile.is_system,
        is_active: profile.is_active,
        created_at: timestamp_to_string(profile.created_at),
        updated_at: timestamp_to_string(profile.updated_at),
        source_policies: profile
            .source_policies
            .into_iter()
            .map(proto_profile_policy)
            .collect(),
        ..Default::default()
    }
}

fn proto_capability(capability: SignalCapability) -> ProtoSignalCapability {
    ProtoSignalCapability {
        id: capability.id,
        source_code: capability.source_code,
        connection_id: capability.connection_id,
        capability: capability.capability,
        state: capability.state,
        reason: capability.reason,
        requires_confirmation: capability.requires_confirmation,
        action_class: capability.action_class,
        updated_at: timestamp_to_string(capability.updated_at),
        ..Default::default()
    }
}

fn proto_profile_policy(policy: SignalProfilePolicy) -> ProtoSignalProfilePolicy {
    ProtoSignalProfilePolicy {
        scope: policy.scope.as_str().to_owned(),
        source_code: policy.source_code,
        connection_id: policy.connection_id,
        event_pattern: policy.event_pattern,
        mode: policy.mode.as_str().to_owned(),
        reason: policy.reason,
        ..Default::default()
    }
}

fn proto_health(health: SignalHealth) -> ProtoSignalHealth {
    ProtoSignalHealth {
        id: health.id.to_string(),
        source_code: health.source_code,
        connection_id: health.connection_id.map(|id| id.to_string()),
        level: health.level,
        summary: health.summary,
        last_ok_at: health.last_ok_at.map(timestamp_to_string),
        last_failure_at: health.last_failure_at.map(timestamp_to_string),
        failure_count: health.failure_count,
        consecutive_failure_count: health.consecutive_failure_count,
        next_retry_at: health.next_retry_at.map(timestamp_to_string),
        updated_at: timestamp_to_string(health.updated_at),
        evidence_json: health.evidence.to_string(),
        ..Default::default()
    }
}

fn proto_runtime_state(runtime: SignalRuntimeState) -> ProtoSignalRuntimeState {
    ProtoSignalRuntimeState {
        id: runtime.id.to_string(),
        source_code: runtime.source_code,
        connection_id: runtime.connection_id.map(|id| id.to_string()),
        runtime_kind: runtime.runtime_kind,
        state: runtime.state,
        metadata_json: runtime.metadata.to_string(),
        updated_at: timestamp_to_string(runtime.updated_at),
        last_started_at: runtime.last_started_at.map(timestamp_to_string),
        last_stopped_at: runtime.last_stopped_at.map(timestamp_to_string),
        last_heartbeat_at: runtime.last_heartbeat_at.map(timestamp_to_string),
        last_error_at: runtime.last_error_at.map(timestamp_to_string),
        last_error_code: runtime.last_error_code,
        last_error_message_redacted: runtime.last_error_message_redacted,
        ..Default::default()
    }
}

fn proto_policy(policy: SignalPolicy) -> ProtoSignalPolicy {
    ProtoSignalPolicy {
        scope: policy.scope.as_str().to_owned(),
        source_code: policy.source_code,
        connection_id: policy.connection_id,
        event_pattern: policy.event_pattern,
        mode: policy.mode.as_str().to_owned(),
        reason: policy.reason,
        expires_at: policy.expires_at.map(timestamp_to_string),
        ..Default::default()
    }
}

fn proto_replay_request(request: SignalReplayRequest) -> ProtoSignalReplayRequest {
    ProtoSignalReplayRequest {
        id: request.id,
        source_code: request.source_code,
        connection_id: request.connection_id,
        event_pattern: request.event_pattern,
        from_position: request.from_position,
        to_position: request.to_position,
        from_time: request.from_time.map(timestamp_to_string),
        to_time: request.to_time.map(timestamp_to_string),
        target_consumer: request.target_consumer,
        target_projection: request.target_projection,
        status: request.status,
        requested_by: request.requested_by,
        requested_at: timestamp_to_string(request.requested_at),
        started_at: request.started_at.map(timestamp_to_string),
        completed_at: request.completed_at.map(timestamp_to_string),
        last_error_redacted: request.last_error_redacted,
        replayed_count: request.replayed_count,
        metadata_json: request.metadata.to_string(),
        ..Default::default()
    }
}

fn timestamp_to_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

fn parse_metadata_json(value: &str) -> Result<Value, String> {
    if value.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }

    let parsed: Value = serde_json::from_str(value)
        .map_err(|error| format!("metadata_json must be valid JSON: {error}"))?;
    if !parsed.is_object() {
        return Err("metadata_json must be a JSON object".to_owned());
    }
    Ok(parsed)
}

fn parse_optional_timestamp(
    value: Option<&str>,
    field_name: &str,
) -> Result<Option<DateTime<Utc>>, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    DateTime::parse_from_rfc3339(value)
        .map(|parsed| Some(parsed.with_timezone(&Utc)))
        .map_err(|error| format!("{field_name} must be a valid RFC3339 timestamp: {error}"))
}

fn parse_policy_scope(value: &str) -> Result<SignalPolicyScope, String> {
    SignalPolicyScope::parse(value.trim())
        .ok_or_else(|| format!("invalid signal policy scope: {value}"))
}

fn parse_policy_mode(value: &str) -> Result<SignalPolicyMode, String> {
    SignalPolicyMode::parse(value.trim())
        .ok_or_else(|| format!("invalid signal policy mode: {value}"))
}

fn control_request(
    scope: &str,
    source_code: Option<&str>,
    connection_id: Option<&str>,
    event_pattern: Option<&str>,
    reason: Option<&str>,
) -> Result<SignalHubControlRequest, ConnectError> {
    Ok(SignalHubControlRequest {
        scope: parse_policy_scope(scope).map_err(invalid_argument_error)?,
        source_code: source_code
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        connection_id: connection_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        event_pattern: event_pattern
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        reason: reason
            .map(non_empty_reason)
            .unwrap_or_else(|| "owner control".to_owned()),
    })
}

fn non_empty_reason(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "user policy".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn signal_hub_connect_error(error: SignalHubError) -> ConnectError {
    if error.is_invalid_request() {
        return invalid_argument_error(error.to_string());
    }
    if error.is_not_found() {
        return ConnectError::new(ErrorCode::NotFound, error.to_string());
    }
    if error.is_failed_precondition() {
        return ConnectError::new(ErrorCode::FailedPrecondition, error.to_string());
    }

    match error {
        SignalHubError::Sqlx(_)
        | SignalHubError::EventStore(_)
        | SignalHubError::Envelope(_)
        | SignalHubError::Json(_)
        | SignalHubError::Toml(_)
        | SignalHubError::Settings(_)
        | SignalHubError::RawSignalPersistence(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
        SignalHubError::InvalidRawSignalEventType(_)
        | SignalHubError::MissingSourceCode
        | SignalHubError::InvalidPolicyScope(_)
        | SignalHubError::InvalidPolicyMode(_)
        | SignalHubError::InvalidConnectionId(_)
        | SignalHubError::InvalidConnectionStatus(_)
        | SignalHubError::SourceNotFound(_)
        | SignalHubError::SourceDoesNotSupportConnections(_)
        | SignalHubError::ConnectionNotFound(_)
        | SignalHubError::InvalidRuntimeState(_)
        | SignalHubError::InvalidRuntimeId(_)
        | SignalHubError::InvalidHealthId(_)
        | SignalHubError::InvalidReplayRequest(_)
        | SignalHubError::FixtureNotFound(_)
        | SignalHubError::InvalidFixtureCatalog(_)
        | SignalHubError::ProfileNotFound(_)
        | SignalHubError::InvalidProfileDefinition(_)
        | SignalHubError::SystemProfileImmutable(_)
        | SignalHubError::EmptyField(_) => unreachable!("classified above"),
    }
}

fn invalid_argument_error(message: impl Into<String>) -> ConnectError {
    ConnectError::new(ErrorCode::InvalidArgument, message.into())
}
