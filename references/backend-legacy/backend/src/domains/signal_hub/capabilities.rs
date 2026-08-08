use chrono::Utc;
use makosh_signal_hub_postgres::raw_signals::adapter::RawSignalStore;

use super::store::{SignalCapability, SignalCapabilityUpsert, SignalHubError, SignalHubStore};
use makosh_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};

#[derive(Clone)]
pub struct SignalHubCapabilityService {
    store: SignalHubStore,
}

impl SignalHubCapabilityService {
    pub fn new(store: SignalHubStore) -> Self {
        Self { store }
    }

    pub async fn list_capabilities(
        &self,
        source_code: Option<&str>,
        connection_id: Option<&str>,
    ) -> Result<Vec<SignalCapability>, SignalHubError> {
        if let Some(source_code) = source_code {
            self.refresh_source_capabilities(source_code).await?;
        } else {
            for source in self.store.list_sources().await? {
                self.refresh_source_capabilities(&source.code).await?;
            }
        }

        self.store
            .list_capabilities(source_code, connection_id)
            .await
    }

    async fn refresh_source_capabilities(&self, source_code: &str) -> Result<(), SignalHubError> {
        let source = self.store.get_source(source_code).await?;
        let policies = RawSignalStore::new(self.store.pool().clone())
            .list_active_policies()
            .await?;
        let control_state = source_capability_control_state(&source.code, &policies);
        let mut capabilities = vec![SignalCapabilityUpsert {
            source_code: source.code.clone(),
            connection_id: None,
            capability: "signals.observe".to_owned(),
            state: control_state.state.to_owned(),
            reason: Some(control_state_reason(
                control_state,
                "source is registered in Signal Hub",
            )),
            requires_confirmation: false,
            action_class: "read".to_owned(),
        }];

        if source.supports_connections {
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: "connections.manage".to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(
                    control_state,
                    "source supports operator-managed connection records",
                )),
                requires_confirmation: false,
                action_class: "local_write".to_owned(),
            });
        }

        if source.supports_runtime {
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: "runtime.health_check".to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(
                    control_state,
                    "source runtime can report durable health state",
                )),
                requires_confirmation: false,
                action_class: "read".to_owned(),
            });
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: "runtime.pause".to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(
                    control_state,
                    "source runtime can be paused or resumed by Signal Hub",
                )),
                requires_confirmation: false,
                action_class: "local_write".to_owned(),
            });
        }

        if source.supports_mute {
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: "runtime.mute".to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(
                    control_state,
                    "source signal publication can be muted without stopping runtime",
                )),
                requires_confirmation: false,
                action_class: "local_write".to_owned(),
            });
        }

        if source.supports_replay {
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: "runtime.replay".to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(
                    control_state,
                    "source events can be replayed from durable Signal Hub history",
                )),
                requires_confirmation: false,
                action_class: "local_write".to_owned(),
            });
        }

        let source_specific = match source.code.as_str() {
            "browser" => Some(("browser.capture", "browser capture source is registered")),
            "filesystem" => Some((
                "files.observe",
                "filesystem observation source is registered",
            )),
            "voice" => Some(("voice.transcribe", "voice capture source is registered")),
            "fixture" => Some((
                "fixture.emit",
                "fixture source can emit deterministic test signals",
            )),
            "ai" => Some(("ai.enrich", "local AI signal source is registered")),
            _ => None,
        };

        if let Some((capability, reason)) = source_specific {
            capabilities.push(SignalCapabilityUpsert {
                source_code: source.code.clone(),
                connection_id: None,
                capability: capability.to_owned(),
                state: control_state.state.to_owned(),
                reason: Some(control_state_reason(control_state, reason)),
                requires_confirmation: false,
                action_class: "read".to_owned(),
            });
        }

        self.store
            .replace_source_capabilities(&source.code, None, &capabilities)
            .await
    }
}

#[derive(Clone, Copy)]
struct CapabilityControlState<'a> {
    state: &'a str,
    status_label: Option<&'a str>,
    reason: Option<&'a str>,
}

fn source_capability_control_state<'a>(
    source_code: &str,
    policies: &'a [SignalPolicy],
) -> CapabilityControlState<'a> {
    let now = Utc::now();
    let matching = policies
        .iter()
        .filter(|policy| {
            if policy
                .expires_at
                .is_some_and(|expires_at| expires_at <= now)
            {
                return false;
            }

            if policy.connection_id.is_some() || policy.event_pattern.is_some() {
                return false;
            }

            match policy.scope {
                SignalPolicyScope::Global => true,
                SignalPolicyScope::Source | SignalPolicyScope::Profile => policy
                    .source_code
                    .as_deref()
                    .is_some_and(|policy_source| policy_source == source_code),
                SignalPolicyScope::Connection | SignalPolicyScope::EventPattern => false,
            }
        })
        .collect::<Vec<_>>();

    if let Some(policy) = matching
        .iter()
        .copied()
        .find(|policy| matches!(policy.mode, SignalPolicyMode::Disabled))
    {
        return CapabilityControlState {
            state: "blocked",
            status_label: Some("disabled"),
            reason: Some(policy.reason.as_str()),
        };
    }

    if let Some(policy) = matching
        .iter()
        .copied()
        .find(|policy| matches!(policy.mode, SignalPolicyMode::Paused))
    {
        return CapabilityControlState {
            state: "degraded",
            status_label: Some("paused"),
            reason: Some(policy.reason.as_str()),
        };
    }

    if let Some(policy) = matching
        .iter()
        .copied()
        .find(|policy| matches!(policy.mode, SignalPolicyMode::Muted))
    {
        return CapabilityControlState {
            state: "degraded",
            status_label: Some("muted"),
            reason: Some(policy.reason.as_str()),
        };
    }

    CapabilityControlState {
        state: "available",
        status_label: None,
        reason: None,
    }
}

fn control_state_reason(control_state: CapabilityControlState<'_>, base_reason: &str) -> String {
    match (control_state.status_label, control_state.reason) {
        (Some(status), Some(reason)) => {
            format!("{base_reason}; source is currently {status} by policy: {reason}")
        }
        (Some(status), None) => format!("{base_reason}; source is currently {status} by policy"),
        (None, _) => base_reason.to_owned(),
    }
}
