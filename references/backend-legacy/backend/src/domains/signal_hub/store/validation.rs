use makosh_signal_hub_api::policies::SignalPolicyScope;
use serde_json::Value;
use uuid::Uuid;

use super::{SignalHubError, SignalProfilePolicy};

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<String, SignalHubError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(SignalHubError::EmptyField(field));
    }
    Ok(value.to_owned())
}

pub(super) fn validate_object(field: &'static str, value: &Value) -> Result<(), SignalHubError> {
    if value.is_object() {
        return Ok(());
    }
    Err(SignalHubError::EmptyField(field))
}

pub(super) fn parse_required_uuid(value: &str) -> Result<Uuid, SignalHubError> {
    Uuid::parse_str(value.trim()).map_err(|_| SignalHubError::InvalidConnectionId(value.to_owned()))
}

pub(super) fn validate_profile_policies(
    policies: &[SignalProfilePolicy],
) -> Result<Vec<SignalProfilePolicy>, SignalHubError> {
    policies
        .iter()
        .map(|policy| {
            if policy.scope == SignalPolicyScope::Profile {
                return Err(SignalHubError::InvalidProfileDefinition(
                    "profile-managed policies cannot use profile scope".to_owned(),
                ));
            }

            let reason = validate_non_empty("reason", &policy.reason)?;
            let source_code = policy
                .source_code
                .as_deref()
                .map(|value| validate_non_empty("source_code", value))
                .transpose()?;
            let connection_id = policy
                .connection_id
                .as_deref()
                .map(|value| {
                    let normalized = validate_non_empty("connection_id", value)?;
                    parse_required_uuid(&normalized)?;
                    Ok::<String, SignalHubError>(normalized)
                })
                .transpose()?;
            let event_pattern = policy
                .event_pattern
                .as_deref()
                .map(|value| validate_non_empty("event_pattern", value))
                .transpose()?;

            match policy.scope {
                SignalPolicyScope::Global => {}
                SignalPolicyScope::Source if source_code.is_some() => {}
                SignalPolicyScope::Connection
                    if source_code.is_some() && connection_id.is_some() => {}
                SignalPolicyScope::EventPattern if event_pattern.is_some() => {}
                SignalPolicyScope::Profile => unreachable!(),
                SignalPolicyScope::Source => {
                    return Err(SignalHubError::InvalidProfileDefinition(
                        "source profile policy requires source_code".to_owned(),
                    ));
                }
                SignalPolicyScope::Connection => {
                    return Err(SignalHubError::InvalidProfileDefinition(
                        "connection profile policy requires source_code and connection_id"
                            .to_owned(),
                    ));
                }
                SignalPolicyScope::EventPattern => {
                    return Err(SignalHubError::InvalidProfileDefinition(
                        "event_pattern profile policy requires event_pattern".to_owned(),
                    ));
                }
            }

            Ok(SignalProfilePolicy {
                scope: policy.scope.clone(),
                source_code,
                connection_id,
                event_pattern,
                mode: policy.mode.clone(),
                reason,
            })
        })
        .collect()
}

pub(super) fn runtime_state_value(value: &str) -> Result<&str, SignalHubError> {
    match value.trim() {
        "stopped" | "starting" | "running" | "reconnecting" | "paused" | "muted" | "stopping"
        | "error" => Ok(value.trim()),
        other => Err(SignalHubError::InvalidRuntimeState(other.to_owned())),
    }
}

pub(super) fn connection_status_value(value: &str) -> Result<&str, SignalHubError> {
    match value.trim() {
        "not_configured"
        | "connecting"
        | "awaiting_user_action"
        | "connected"
        | "degraded"
        | "disconnected"
        | "paused"
        | "muted"
        | "disabled"
        | "error"
        | "removed" => Ok(value.trim()),
        other => Err(SignalHubError::InvalidConnectionStatus(other.to_owned())),
    }
}

pub(super) fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db_error) if db_error.is_unique_violation())
}

pub(super) fn parse_optional_uuid(value: Option<&str>) -> Result<Option<Uuid>, SignalHubError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            Uuid::parse_str(value)
                .map_err(|_| SignalHubError::InvalidConnectionId(value.to_owned()))
        })
        .transpose()
}

pub(super) fn truncate_redacted_error(error: &str) -> String {
    let trimmed = error.trim();
    trimmed.chars().take(500).collect()
}
