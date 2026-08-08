use chrono::{DateTime, Utc};
use makosh_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignalPolicyDecision {
    Allow,
    Rejected { reason: String },
    Paused { reason: String },
    Muted { reason: String },
}

pub struct SignalPolicyEvaluator {
    now: DateTime<Utc>,
}

impl SignalPolicyEvaluator {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self { now }
    }

    pub fn decide(
        &self,
        source_code: &str,
        connection_id: Option<&str>,
        event_type: &str,
        policies: &[SignalPolicy],
    ) -> SignalPolicyDecision {
        let matching: Vec<&SignalPolicy> = policies
            .iter()
            .filter(|policy| self.policy_applies(policy, source_code, connection_id, event_type))
            .collect();

        if let Some(policy) = matching
            .iter()
            .copied()
            .find(|policy| matches!(policy.mode, SignalPolicyMode::Disabled))
        {
            return SignalPolicyDecision::Rejected {
                reason: policy.reason.clone(),
            };
        }

        if let Some(policy) = matching
            .iter()
            .copied()
            .find(|policy| matches!(policy.mode, SignalPolicyMode::Paused))
        {
            return SignalPolicyDecision::Paused {
                reason: policy.reason.clone(),
            };
        }

        if let Some(policy) = matching
            .iter()
            .copied()
            .find(|policy| matches!(policy.mode, SignalPolicyMode::Muted))
        {
            return SignalPolicyDecision::Muted {
                reason: policy.reason.clone(),
            };
        }

        SignalPolicyDecision::Allow
    }

    fn policy_applies(
        &self,
        policy: &SignalPolicy,
        source_code: &str,
        connection_id: Option<&str>,
        event_type: &str,
    ) -> bool {
        if policy
            .expires_at
            .is_some_and(|expires_at| expires_at <= self.now)
        {
            return false;
        }

        if policy
            .source_code
            .as_deref()
            .is_some_and(|policy_source| policy_source != source_code)
        {
            return false;
        }

        if policy
            .connection_id
            .as_deref()
            .is_some_and(|policy_connection| Some(policy_connection) != connection_id)
        {
            return false;
        }

        if policy
            .event_pattern
            .as_deref()
            .is_some_and(|pattern| !event_type_matches(pattern, event_type))
        {
            return false;
        }

        match policy.scope {
            SignalPolicyScope::Global => true,
            SignalPolicyScope::Source => policy.source_code.is_some(),
            SignalPolicyScope::Connection => policy.connection_id.is_some(),
            SignalPolicyScope::EventPattern => policy.event_pattern.is_some(),
            SignalPolicyScope::Profile => true,
        }
    }
}

fn event_type_matches(pattern: &str, event_type: &str) -> bool {
    if pattern == event_type {
        return true;
    }

    let Some(prefix) = pattern.strip_suffix(".*") else {
        return false;
    };

    event_type.starts_with(prefix)
}
