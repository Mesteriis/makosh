use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use uuid::Uuid;

use super::store::{SignalConnection, SignalHubError, SignalHubStore, SignalSource};
use makosh_events_postgres::store::EventStore;
use makosh_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalHubControlRequest {
    pub scope: SignalPolicyScope,
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalHubControlResult {
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub policy_id: Option<String>,
    pub cleared_count: u64,
}

#[derive(Clone)]
pub struct SignalHubControlService {
    signal_store: SignalHubStore,
    event_store: EventStore,
}

impl SignalHubControlService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        Self {
            signal_store,
            event_store,
        }
    }

    pub async fn disable_source(
        &self,
        source_code: &str,
        reason: Option<&str>,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        let source = self.signal_store.get_source(source_code).await?;
        let policy = SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some(source.code.clone()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Disabled,
            reason: normalize_reason(reason, "source disabled"),
            expires_at: None,
        };
        let policy_id = self.signal_store.create_policy(&policy).await?;
        self.reconcile_source_runtime_state(&source.code).await?;
        self.append_control_events("signal.source.disabled", &policy, Some(&source), None, 0)
            .await?;

        Ok(SignalHubControlResult {
            source_code: policy.source_code,
            connection_id: None,
            event_pattern: None,
            policy_id: Some(policy_id.to_string()),
            cleared_count: 0,
        })
    }

    pub async fn enable_source(
        &self,
        source_code: &str,
        reason: Option<&str>,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        let source = self.signal_store.get_source(source_code).await?;
        let policy = SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some(source.code.clone()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Disabled,
            reason: normalize_reason(reason, "source enabled"),
            expires_at: None,
        };
        let cleared_count = self
            .signal_store
            .expire_matching_policies(&policy, &[SignalPolicyMode::Disabled])
            .await?;
        self.reconcile_source_runtime_state(&source.code).await?;
        self.append_control_events(
            "signal.source.enabled",
            &policy,
            Some(&source),
            None,
            cleared_count,
        )
        .await?;

        Ok(SignalHubControlResult {
            source_code: policy.source_code,
            connection_id: None,
            event_pattern: None,
            policy_id: None,
            cleared_count,
        })
    }

    pub async fn mute_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.create_scoped_policy(request, SignalPolicyMode::Muted, "signal.source.muted")
            .await
    }

    pub async fn disable_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.create_scoped_policy(
            request,
            SignalPolicyMode::Disabled,
            "signal.signals.disabled",
        )
        .await
    }

    pub async fn enable_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.clear_scoped_policy(
            request,
            SignalPolicyMode::Disabled,
            "signal.signals.enabled",
        )
        .await
    }

    pub async fn unmute_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.clear_scoped_policy(request, SignalPolicyMode::Muted, "signal.source.unmuted")
            .await
    }

    pub async fn pause_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.create_scoped_policy(request, SignalPolicyMode::Paused, "signal.source.paused")
            .await
    }

    pub async fn resume_signals(
        &self,
        request: &SignalHubControlRequest,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        self.clear_scoped_policy(request, SignalPolicyMode::Paused, "signal.source.resumed")
            .await
    }

    async fn create_scoped_policy(
        &self,
        request: &SignalHubControlRequest,
        mode: SignalPolicyMode,
        event_type: &str,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        let resolved = self.resolve_request(request, Some(mode.clone())).await?;
        let policy_id = self.signal_store.create_policy(&resolved.policy).await?;
        if matches!(resolved.policy.scope, SignalPolicyScope::Source)
            && let Some(source_code) = resolved.policy.source_code.as_deref()
        {
            self.reconcile_source_runtime_state(source_code).await?;
        }
        self.append_control_events(
            event_type,
            &resolved.policy,
            resolved.source.as_ref(),
            resolved.connection.as_ref(),
            0,
        )
        .await?;

        Ok(SignalHubControlResult {
            source_code: resolved.policy.source_code,
            connection_id: resolved.policy.connection_id,
            event_pattern: resolved.policy.event_pattern,
            policy_id: Some(policy_id.to_string()),
            cleared_count: 0,
        })
    }

    async fn clear_scoped_policy(
        &self,
        request: &SignalHubControlRequest,
        mode: SignalPolicyMode,
        event_type: &str,
    ) -> Result<SignalHubControlResult, SignalHubError> {
        let resolved = self.resolve_request(request, Some(mode.clone())).await?;
        let cleared_count = self
            .signal_store
            .expire_matching_policies(&resolved.policy, &[mode])
            .await?;
        if matches!(resolved.policy.scope, SignalPolicyScope::Source)
            && let Some(source_code) = resolved.policy.source_code.as_deref()
        {
            self.reconcile_source_runtime_state(source_code).await?;
        }
        self.append_control_events(
            event_type,
            &resolved.policy,
            resolved.source.as_ref(),
            resolved.connection.as_ref(),
            cleared_count,
        )
        .await?;

        Ok(SignalHubControlResult {
            source_code: resolved.policy.source_code,
            connection_id: resolved.policy.connection_id,
            event_pattern: resolved.policy.event_pattern,
            policy_id: None,
            cleared_count,
        })
    }

    async fn reconcile_source_runtime_state(
        &self,
        source_code: &str,
    ) -> Result<(), SignalHubError> {
        let state = crate::platform::events::runtime::source_runtime_state_from_policies(
            self.signal_store.pool(),
            source_code,
        )
        .await?;
        self.signal_store
            .set_source_runtime_state(source_code, state)
            .await?;
        Ok(())
    }

    async fn resolve_request(
        &self,
        request: &SignalHubControlRequest,
        mode: Option<SignalPolicyMode>,
    ) -> Result<ResolvedControlRequest, SignalHubError> {
        let reason = normalize_reason(Some(&request.reason), "owner control");
        let source_code = request
            .source_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let connection_id = request
            .connection_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let event_pattern = request
            .event_pattern
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        let source = match request.scope {
            SignalPolicyScope::Source => Some(
                self.signal_store
                    .get_source(
                        source_code
                            .as_deref()
                            .ok_or(SignalHubError::EmptyField("source_code"))?,
                    )
                    .await?,
            ),
            _ => match source_code.as_deref() {
                Some(code) => Some(self.signal_store.get_source(code).await?),
                None => None,
            },
        };

        let connection = match request.scope {
            SignalPolicyScope::Connection => Some(
                self.signal_store
                    .get_connection(
                        connection_id
                            .as_deref()
                            .ok_or(SignalHubError::EmptyField("connection_id"))?,
                    )
                    .await?,
            ),
            _ => match connection_id.as_deref() {
                Some(id) => Some(self.signal_store.get_connection(id).await?),
                None => None,
            },
        };

        if matches!(request.scope, SignalPolicyScope::EventPattern) && event_pattern.is_none() {
            return Err(SignalHubError::EmptyField("event_pattern"));
        }

        if matches!(request.scope, SignalPolicyScope::Profile) {
            return Err(SignalHubError::InvalidPolicyScope("profile".to_owned()));
        }

        let normalized_source_code = match (&source, &connection) {
            (Some(source), _) => Some(source.code.clone()),
            (None, Some(connection)) => Some(connection.source_code.clone()),
            (None, None) => None,
        };
        if let (Some(source), Some(connection)) = (&source, &connection)
            && connection.source_code != source.code
        {
            return Err(SignalHubError::InvalidConnectionId(connection.id.clone()));
        }

        Ok(ResolvedControlRequest {
            policy: SignalPolicy {
                scope: request.scope.clone(),
                source_code: normalized_source_code,
                connection_id,
                event_pattern,
                mode: mode.unwrap_or(SignalPolicyMode::Enabled),
                reason,
                expires_at: None,
            },
            source,
            connection,
        })
    }

    async fn append_control_events(
        &self,
        control_event_type: &str,
        policy: &SignalPolicy,
        _source: Option<&SignalSource>,
        connection: Option<&SignalConnection>,
        cleared_count: u64,
    ) -> Result<(), SignalHubError> {
        let now = Utc::now();
        let control_event_id = Uuid::now_v7();
        let policy_event_id = Uuid::now_v7();
        let control_source_id = format!("signal_hub_control:{control_event_id}");
        let policy_source_id = format!("signal_hub_control:{policy_event_id}");
        let source_code = policy
            .source_code
            .as_deref()
            .or_else(|| connection.map(|item| item.source_code.as_str()))
            .unwrap_or("system");
        let entity_id = connection
            .map(|item| item.id.as_str())
            .or(policy.event_pattern.as_deref())
            .unwrap_or(source_code);
        let payload = json!({
            "scope": policy.scope.as_str(),
            "source_code": policy.source_code,
            "connection_id": policy.connection_id,
            "event_pattern": policy.event_pattern,
            "mode": policy.mode.as_str(),
            "reason": policy.reason,
            "cleared_count": cleared_count,
        });
        let provenance = json!({
            "source": "signal_hub_control_service",
            "managed_via": "connectrpc",
        });

        let control_event = NewEventEnvelope::builder(
            format!(
                "evt_{}_{}",
                control_event_type.replace('.', "_"),
                control_event_id
            ),
            control_event_type,
            now,
            json!({
                "kind": "signal_source",
                "source_code": source_code,
                "source_id": control_source_id,
            }),
            json!({
                "kind": "signal_policy",
                "entity_id": entity_id,
                "scope": policy.scope.as_str(),
            }),
        )
        .payload(payload.clone())
        .provenance(provenance.clone())
        .build()?;
        self.event_store.append_for_dispatch(&control_event).await?;

        let policy_event = NewEventEnvelope::builder(
            format!("evt_signal_policy_changed_{policy_event_id}"),
            "signal.policy.changed",
            now,
            json!({
                "kind": "signal_source",
                "source_code": source_code,
                "source_id": policy_source_id,
            }),
            json!({
                "kind": "signal_policy",
                "entity_id": entity_id,
                "scope": policy.scope.as_str(),
            }),
        )
        .payload(payload)
        .provenance(provenance)
        .build()?;
        self.event_store.append_for_dispatch(&policy_event).await?;

        Ok(())
    }
}

struct ResolvedControlRequest {
    policy: SignalPolicy,
    source: Option<SignalSource>,
    connection: Option<SignalConnection>,
}

fn normalize_reason(reason: Option<&str>, fallback: &str) -> String {
    reason
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_owned()
}
