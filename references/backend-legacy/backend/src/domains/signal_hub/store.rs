use chrono::{DateTime, Utc};
use makosh_events_api::{EventEnvelope, EventEnvelopeError};
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;
use uuid::Uuid;

use super::fixtures::{
    SystemProfileFixture, SystemSourceFixture, system_profile_fixtures, system_source_fixtures,
};
use crate::platform::settings::errors::SettingsError;
use makosh_events_postgres::errors::EventStoreError;
use makosh_signal_hub_api::policies::{SignalPolicyMode, SignalPolicyScope};
use makosh_signal_hub_api::raw_signals::{
    RawSignalPersistenceError, RawSignalPersistenceErrorKind,
};

mod connections;
mod health;
mod policy_mutations;
mod replay_queue;
mod runtime_states;
mod sources;
mod validation;

use validation::{
    is_unique_violation, parse_required_uuid, validate_non_empty, validate_profile_policies,
};

#[derive(Debug, Error)]
pub enum SignalHubError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Envelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Settings(#[from] SettingsError),

    #[error(transparent)]
    RawSignalPersistence(#[from] RawSignalPersistenceError),

    #[error("invalid raw signal event type: {0}")]
    InvalidRawSignalEventType(String),

    #[error("signal source_code is missing")]
    MissingSourceCode,

    #[error("invalid signal policy scope: {0}")]
    InvalidPolicyScope(String),

    #[error("invalid signal policy mode: {0}")]
    InvalidPolicyMode(String),

    #[error("invalid signal connection id: {0}")]
    InvalidConnectionId(String),

    #[error("invalid signal connection status: {0}")]
    InvalidConnectionStatus(String),

    #[error("signal source not found: {0}")]
    SourceNotFound(String),

    #[error("signal source does not support connections: {0}")]
    SourceDoesNotSupportConnections(String),

    #[error("signal connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("invalid signal runtime state: {0}")]
    InvalidRuntimeState(String),

    #[error("invalid signal runtime id: {0}")]
    InvalidRuntimeId(String),

    #[error("invalid signal health id: {0}")]
    InvalidHealthId(String),

    #[error("invalid signal replay request: {0}")]
    InvalidReplayRequest(String),

    #[error("signal fixture not found: {0}")]
    FixtureNotFound(String),

    #[error("invalid signal fixture catalog: {0}")]
    InvalidFixtureCatalog(String),

    #[error("signal profile not found: {0}")]
    ProfileNotFound(String),

    #[error("invalid signal profile definition: {0}")]
    InvalidProfileDefinition(String),

    #[error("system signal profile is immutable: {0}")]
    SystemProfileImmutable(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

impl SignalHubError {
    pub fn is_invalid_request(&self) -> bool {
        match self {
            Self::RawSignalPersistence(error) => matches!(
                error.kind,
                RawSignalPersistenceErrorKind::InvalidConnectionId
                    | RawSignalPersistenceErrorKind::InvalidPolicyScope
                    | RawSignalPersistenceErrorKind::InvalidPolicyMode
            ),
            Self::InvalidRawSignalEventType(_)
            | Self::MissingSourceCode
            | Self::InvalidPolicyScope(_)
            | Self::InvalidPolicyMode(_)
            | Self::InvalidConnectionId(_)
            | Self::InvalidConnectionStatus(_)
            | Self::InvalidRuntimeState(_)
            | Self::InvalidRuntimeId(_)
            | Self::InvalidHealthId(_)
            | Self::InvalidReplayRequest(_)
            | Self::InvalidFixtureCatalog(_)
            | Self::InvalidProfileDefinition(_)
            | Self::SystemProfileImmutable(_)
            | Self::EmptyField(_) => true,
            _ => false,
        }
    }

    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::SourceNotFound(_)
                | Self::ConnectionNotFound(_)
                | Self::FixtureNotFound(_)
                | Self::ProfileNotFound(_)
        )
    }

    pub fn is_failed_precondition(&self) -> bool {
        matches!(self, Self::SourceDoesNotSupportConnections(_))
    }
}

#[derive(Clone)]
pub struct SignalHubStore {
    pool: PgPool,
}

impl SignalHubStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn restore_system_sources(&self) -> Result<FixtureRestoreReport, SignalHubError> {
        let mut report = FixtureRestoreReport::default();

        for fixture in system_source_fixtures() {
            let existing = self.source_by_code(fixture.code).await?;
            match existing {
                Some(source) if source_matches_fixture(&source, fixture) => {}
                Some(source) => {
                    sqlx::query(
                        r#"
                        UPDATE signal_sources
                        SET
                            display_name = $2,
                            category = $3,
                            source_kind = $4,
                            default_enabled = $5,
                            supports_connections = $6,
                            supports_runtime = $7,
                            supports_replay = $8,
                            supports_pause = $9,
                            supports_mute = $10,
                            capability_schema_version = 1,
                            updated_at = now()
                        WHERE id = $1
                        "#,
                    )
                    .bind(source.id)
                    .bind(fixture.display_name)
                    .bind(fixture.category)
                    .bind(fixture.source_kind)
                    .bind(fixture.default_enabled)
                    .bind(fixture.supports_connections)
                    .bind(fixture.supports_runtime)
                    .bind(fixture.supports_replay)
                    .bind(fixture.supports_pause)
                    .bind(fixture.supports_mute)
                    .execute(&self.pool)
                    .await?;
                    report.sources_repaired += 1;
                }
                None => {
                    match sqlx::query(
                        r#"
                        INSERT INTO signal_sources (
                            id,
                            code,
                            display_name,
                            category,
                            source_kind,
                            default_enabled,
                            supports_connections,
                            supports_runtime,
                            supports_replay,
                            supports_pause,
                            supports_mute,
                            capability_schema_version
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 1)
                        "#,
                    )
                    .bind(Uuid::now_v7())
                    .bind(fixture.code)
                    .bind(fixture.display_name)
                    .bind(fixture.category)
                    .bind(fixture.source_kind)
                    .bind(fixture.default_enabled)
                    .bind(fixture.supports_connections)
                    .bind(fixture.supports_runtime)
                    .bind(fixture.supports_replay)
                    .bind(fixture.supports_pause)
                    .bind(fixture.supports_mute)
                    .execute(&self.pool)
                    .await
                    {
                        Ok(_) => {
                            report.sources_created += 1;
                        }
                        Err(error) if is_unique_violation(&error) => {
                            if let Some(source) = self.source_by_code(fixture.code).await?
                                && !source_matches_fixture(&source, fixture)
                            {
                                sqlx::query(
                                    r#"
                                        UPDATE signal_sources
                                        SET
                                            display_name = $2,
                                            category = $3,
                                            source_kind = $4,
                                            default_enabled = $5,
                                            supports_connections = $6,
                                            supports_runtime = $7,
                                            supports_replay = $8,
                                            supports_pause = $9,
                                            supports_mute = $10,
                                            capability_schema_version = 1,
                                            updated_at = now()
                                        WHERE id = $1
                                        "#,
                                )
                                .bind(source.id)
                                .bind(fixture.display_name)
                                .bind(fixture.category)
                                .bind(fixture.source_kind)
                                .bind(fixture.default_enabled)
                                .bind(fixture.supports_connections)
                                .bind(fixture.supports_runtime)
                                .bind(fixture.supports_replay)
                                .bind(fixture.supports_pause)
                                .bind(fixture.supports_mute)
                                .execute(&self.pool)
                                .await?;
                                report.sources_repaired += 1;
                            }
                        }
                        Err(error) => return Err(error.into()),
                    }
                }
            }
        }

        for fixture in system_profile_fixtures() {
            let existing = self.profile_by_code(fixture.code).await?;
            match existing {
                Some(profile) if profile_matches_fixture(&profile, fixture) => {}
                Some(profile) => {
                    sqlx::query(
                        r#"
                        UPDATE signal_profiles
                        SET
                            display_name = $2,
                            description = $3,
                            source_policies = $4,
                            is_system = $5,
                            updated_at = now()
                        WHERE id = $1
                        "#,
                    )
                    .bind(parse_required_uuid(&profile.id)?)
                    .bind(fixture.display_name)
                    .bind(fixture.description)
                    .bind(profile_policies_json(fixture)?)
                    .bind(fixture.is_system)
                    .execute(&self.pool)
                    .await?;
                    report.profiles_repaired += 1;
                }
                None => {
                    match sqlx::query(
                        r#"
                        INSERT INTO signal_profiles (
                            id,
                            code,
                            display_name,
                            description,
                            source_policies,
                            is_system
                        )
                        VALUES ($1, $2, $3, $4, $5, $6)
                        "#,
                    )
                    .bind(Uuid::now_v7())
                    .bind(fixture.code)
                    .bind(fixture.display_name)
                    .bind(fixture.description)
                    .bind(profile_policies_json(fixture)?)
                    .bind(fixture.is_system)
                    .execute(&self.pool)
                    .await
                    {
                        Ok(_) => {
                            report.profiles_created += 1;
                        }
                        Err(error) if is_unique_violation(&error) => {
                            if let Some(profile) = self.profile_by_code(fixture.code).await?
                                && !profile_matches_fixture(&profile, fixture)
                            {
                                sqlx::query(
                                    r#"
                                    UPDATE signal_profiles
                                    SET
                                        display_name = $2,
                                        description = $3,
                                        source_policies = $4,
                                        is_system = $5,
                                        updated_at = now()
                                    WHERE id = $1
                                    "#,
                                )
                                .bind(parse_required_uuid(&profile.id)?)
                                .bind(fixture.display_name)
                                .bind(fixture.description)
                                .bind(profile_policies_json(fixture)?)
                                .bind(fixture.is_system)
                                .execute(&self.pool)
                                .await?;
                                report.profiles_repaired += 1;
                            }
                        }
                        Err(error) => return Err(error.into()),
                    }
                }
            }
        }

        Ok(report)
    }

    pub async fn list_profiles(&self) -> Result<Vec<SignalProfile>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                code,
                display_name,
                description,
                source_policies,
                is_system,
                created_at,
                updated_at
            FROM signal_profiles
            ORDER BY is_system DESC, code ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_profile).collect()
    }

    pub async fn profile_by_code(
        &self,
        profile_code: &str,
    ) -> Result<Option<SignalProfile>, SignalHubError> {
        let profile_code = validate_non_empty("profile_code", profile_code)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                code,
                display_name,
                description,
                source_policies,
                is_system,
                created_at,
                updated_at
            FROM signal_profiles
            WHERE code = $1
            "#,
        )
        .bind(profile_code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_profile).transpose()
    }

    pub async fn list_capabilities(
        &self,
        source_code: Option<&str>,
        connection_id: Option<&str>,
    ) -> Result<Vec<SignalCapability>, SignalHubError> {
        let source_code = source_code
            .map(|value| validate_non_empty("source_code", value))
            .transpose()?;
        let connection_id = connection_id.map(parse_required_uuid).transpose()?;

        let rows = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                capability,
                state,
                reason,
                requires_confirmation,
                action_class,
                updated_at
            FROM signal_capabilities
            WHERE ($1::text IS NULL OR source_code = $1)
              AND (
                ($2::uuid IS NULL AND connection_id IS NULL)
                OR connection_id = $2
                OR $2::uuid IS NULL
              )
            ORDER BY source_code ASC, capability ASC
            "#,
        )
        .bind(source_code)
        .bind(connection_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_capability).collect()
    }

    pub async fn replace_source_capabilities(
        &self,
        source_code: &str,
        connection_id: Option<&str>,
        capabilities: &[SignalCapabilityUpsert],
    ) -> Result<(), SignalHubError> {
        let source_code = validate_non_empty("source_code", source_code)?;
        let connection_id = connection_id.map(parse_required_uuid).transpose()?;

        sqlx::query(
            r#"
            DELETE FROM signal_capabilities
            WHERE source_code = $1
              AND (
                ($2::uuid IS NULL AND connection_id IS NULL)
                OR connection_id = $2
              )
            "#,
        )
        .bind(&source_code)
        .bind(connection_id)
        .execute(&self.pool)
        .await?;

        for capability in capabilities {
            let capability_name = validate_non_empty("capability", &capability.capability)?;
            let state = validate_non_empty("state", &capability.state)?;
            let action_class = validate_non_empty("action_class", &capability.action_class)?;
            let reason = capability
                .reason
                .as_deref()
                .map(|value| validate_non_empty("reason", value))
                .transpose()?;

            sqlx::query(
                r#"
                INSERT INTO signal_capabilities (
                    id,
                    source_code,
                    connection_id,
                    capability,
                    state,
                    reason,
                    requires_confirmation,
                    action_class,
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(&source_code)
            .bind(connection_id)
            .bind(capability_name)
            .bind(state)
            .bind(reason)
            .bind(capability.requires_confirmation)
            .bind(action_class)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn create_profile(
        &self,
        request: &SignalProfileCreate,
    ) -> Result<SignalProfile, SignalHubError> {
        let code = validate_non_empty("code", &request.code)?;
        let display_name = validate_non_empty("display_name", &request.display_name)?;
        let description = validate_non_empty("description", &request.description)?;
        let source_policies = validate_profile_policies(&request.source_policies)?;

        if self.profile_by_code(&code).await?.is_some() {
            return Err(SignalHubError::InvalidProfileDefinition(format!(
                "profile code already exists: {code}"
            )));
        }

        let id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO signal_profiles (
                id,
                code,
                display_name,
                description,
                source_policies,
                is_system
            )
            VALUES ($1, $2, $3, $4, $5, FALSE)
            "#,
        )
        .bind(id)
        .bind(&code)
        .bind(display_name)
        .bind(description)
        .bind(serde_json::to_value(&source_policies)?)
        .execute(&self.pool)
        .await?;

        self.profile_by_code(&code)
            .await?
            .ok_or_else(|| SignalHubError::ProfileNotFound(code))
    }

    pub async fn update_profile(
        &self,
        request: &SignalProfileUpdate,
    ) -> Result<SignalProfile, SignalHubError> {
        let code = validate_non_empty("code", &request.code)?;
        let current = self
            .profile_by_code(&code)
            .await?
            .ok_or_else(|| SignalHubError::ProfileNotFound(code.clone()))?;

        if current.is_system {
            return Err(SignalHubError::SystemProfileImmutable(code));
        }

        let display_name = request
            .display_name
            .as_deref()
            .map(|value| validate_non_empty("display_name", value))
            .transpose()?
            .unwrap_or(current.display_name.clone());
        let description = request
            .description
            .as_deref()
            .map(|value| validate_non_empty("description", value))
            .transpose()?
            .unwrap_or(current.description.clone());
        let source_policies = match request.source_policies.as_ref() {
            Some(policies) => validate_profile_policies(policies)?,
            None => current.source_policies.clone(),
        };

        sqlx::query(
            r#"
            UPDATE signal_profiles
            SET
                display_name = $2,
                description = $3,
                source_policies = $4,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(parse_required_uuid(&current.id)?)
        .bind(display_name)
        .bind(description)
        .bind(serde_json::to_value(&source_policies)?)
        .execute(&self.pool)
        .await?;

        self.profile_by_code(&current.code)
            .await?
            .ok_or_else(|| SignalHubError::ProfileNotFound(current.code))
    }

    pub async fn delete_profile(
        &self,
        profile_code: &str,
    ) -> Result<SignalProfile, SignalHubError> {
        let code = validate_non_empty("profile_code", profile_code)?;
        let current = self
            .profile_by_code(&code)
            .await?
            .ok_or_else(|| SignalHubError::ProfileNotFound(code.clone()))?;

        if current.is_system {
            return Err(SignalHubError::SystemProfileImmutable(code));
        }

        sqlx::query(
            r#"
            DELETE FROM signal_profiles
            WHERE id = $1
            "#,
        )
        .bind(parse_required_uuid(&current.id)?)
        .execute(&self.pool)
        .await?;

        Ok(current)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FixtureRestoreReport {
    pub sources_created: usize,
    pub sources_repaired: usize,
    pub profiles_created: usize,
    pub profiles_repaired: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalSource {
    pub id: Uuid,
    pub code: String,
    pub display_name: String,
    pub category: String,
    pub source_kind: String,
    pub default_enabled: bool,
    pub supports_connections: bool,
    pub supports_runtime: bool,
    pub supports_replay: bool,
    pub supports_pause: bool,
    pub supports_mute: bool,
    pub capability_schema_version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalConnection {
    pub id: String,
    pub source_code: String,
    pub display_name: String,
    pub status: String,
    pub profile: Option<String>,
    pub settings: Value,
    pub secret_ref: Option<String>,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub last_signal_at: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalProfile {
    pub id: String,
    pub code: String,
    pub display_name: String,
    pub description: String,
    pub source_policies: Vec<SignalProfilePolicy>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalProfileSummary {
    pub id: String,
    pub code: String,
    pub display_name: String,
    pub description: String,
    pub policy_count: usize,
    pub source_policies: Vec<SignalProfilePolicy>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
pub struct SignalProfilePolicy {
    pub scope: SignalPolicyScope,
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub mode: SignalPolicyMode,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalProfileCreate {
    pub code: String,
    pub display_name: String,
    pub description: String,
    pub source_policies: Vec<SignalProfilePolicy>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalProfileUpdate {
    pub code: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub source_policies: Option<Vec<SignalProfilePolicy>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalCapability {
    pub id: String,
    pub source_code: String,
    pub connection_id: Option<String>,
    pub capability: String,
    pub state: String,
    pub reason: Option<String>,
    pub requires_confirmation: bool,
    pub action_class: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalCapabilityUpsert {
    pub source_code: String,
    pub connection_id: Option<String>,
    pub capability: String,
    pub state: String,
    pub reason: Option<String>,
    pub requires_confirmation: bool,
    pub action_class: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalConnectionCreate {
    pub source_code: String,
    pub display_name: String,
    pub status: String,
    pub profile: Option<String>,
    pub settings: Value,
    pub secret_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalConnectionUpdate {
    pub id: String,
    pub display_name: Option<String>,
    pub status: Option<String>,
    pub profile: Option<String>,
    pub settings: Option<Value>,
    pub secret_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalHealth {
    pub id: String,
    pub source_code: String,
    pub connection_id: Option<String>,
    pub level: String,
    pub summary: String,
    pub last_ok_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub failure_count: i32,
    pub consecutive_failure_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub evidence: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalHealthCheckRequest {
    pub source_code: String,
    pub connection_id: Option<String>,
    pub runtime_kind: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalHealthSnapshotWrite {
    pub level: String,
    pub summary: String,
    pub last_ok_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub failure_count: i32,
    pub consecutive_failure_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub evidence: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalRuntimeState {
    pub id: String,
    pub source_code: String,
    pub connection_id: Option<String>,
    pub runtime_kind: String,
    pub state: String,
    pub last_started_at: Option<DateTime<Utc>>,
    pub last_stopped_at: Option<DateTime<Utc>>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub last_error_at: Option<DateTime<Utc>>,
    pub last_error_code: Option<String>,
    pub last_error_message_redacted: Option<String>,
    pub metadata: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalRuntimeStateUpdate {
    pub source_code: String,
    pub runtime_kind: String,
    pub state: String,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PausedSignalEvent {
    pub id: String,
    pub event_id: String,
    pub source_code: String,
    pub raw_event_type: String,
    pub event: EventEnvelope,
    pub paused_at: DateTime<Utc>,
}

fn row_to_profile(row: PgRow) -> Result<SignalProfile, SignalHubError> {
    Ok(SignalProfile {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        code: row.try_get("code")?,
        display_name: row.try_get("display_name")?,
        description: row.try_get("description")?,
        source_policies: serde_json::from_value(row.try_get("source_policies")?)?,
        is_system: row.try_get("is_system")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_capability(row: PgRow) -> Result<SignalCapability, SignalHubError> {
    let connection_id = row.try_get::<Option<Uuid>, _>("connection_id")?;
    Ok(SignalCapability {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        source_code: row.try_get("source_code")?,
        connection_id: connection_id.map(|value| value.to_string()),
        capability: row.try_get("capability")?,
        state: row.try_get("state")?,
        reason: row.try_get("reason")?,
        requires_confirmation: row.try_get("requires_confirmation")?,
        action_class: row.try_get("action_class")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_runtime_state(row: PgRow) -> Result<SignalRuntimeState, SignalHubError> {
    let connection_id = row.try_get::<Option<Uuid>, _>("connection_id")?;
    Ok(SignalRuntimeState {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        source_code: row.try_get("source_code")?,
        connection_id: connection_id.map(|value| value.to_string()),
        runtime_kind: row.try_get("runtime_kind")?,
        state: row.try_get("state")?,
        last_started_at: row.try_get("last_started_at")?,
        last_stopped_at: row.try_get("last_stopped_at")?,
        last_heartbeat_at: row.try_get("last_heartbeat_at")?,
        last_error_at: row.try_get("last_error_at")?,
        last_error_code: row.try_get("last_error_code")?,
        last_error_message_redacted: row.try_get("last_error_message_redacted")?,
        metadata: row.try_get("metadata")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn source_matches_fixture(source: &SignalSource, fixture: &SystemSourceFixture) -> bool {
    source.display_name == fixture.display_name
        && source.category == fixture.category
        && source.source_kind == fixture.source_kind
        && source.default_enabled == fixture.default_enabled
        && source.supports_connections == fixture.supports_connections
        && source.supports_runtime == fixture.supports_runtime
        && source.supports_replay == fixture.supports_replay
        && source.supports_pause == fixture.supports_pause
        && source.supports_mute == fixture.supports_mute
        && source.capability_schema_version == 1
}

fn profile_matches_fixture(profile: &SignalProfile, fixture: &SystemProfileFixture) -> bool {
    match profile_policies_json(fixture) {
        Ok(policies) => {
            profile.display_name == fixture.display_name
                && profile.description == fixture.description
                && profile.is_system == fixture.is_system
                && serde_json::to_value(&profile.source_policies).ok() == Some(policies)
        }
        Err(_) => false,
    }
}

fn profile_policies_json(fixture: &SystemProfileFixture) -> Result<Value, SignalHubError> {
    let policies = fixture
        .policies
        .iter()
        .map(|policy| SignalProfilePolicy {
            scope: policy.scope.clone(),
            source_code: policy.source_code.map(ToOwned::to_owned),
            connection_id: None,
            event_pattern: policy.event_pattern.map(ToOwned::to_owned),
            mode: policy.mode.clone(),
            reason: policy.reason.to_owned(),
        })
        .collect::<Vec<_>>();

    Ok(serde_json::to_value(policies)?)
}

pub(crate) fn event_type_pattern_matches(pattern: &str, event_type: &str) -> bool {
    if pattern == event_type {
        return true;
    }

    let Some(prefix) = pattern.strip_suffix(".*") else {
        return false;
    };

    event_type.starts_with(prefix)
}
