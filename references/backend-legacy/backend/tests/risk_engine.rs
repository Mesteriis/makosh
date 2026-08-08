use makosh_hub_backend::engines::risk::engine::RiskEngine;
use makosh_hub_backend::engines::risk::models::{RiskAttentionStatus, RiskSeverity, RiskSignal};

#[test]
fn risk_engine_derives_attention_status_from_unresolved_severity() {
    let no_risks = RiskEngine::derive_attention_status(&[]);
    assert_eq!(no_risks, RiskAttentionStatus::Healthy);
    assert_eq!(no_risks.as_persona_health_status(), "healthy");

    let needs_attention = RiskEngine::derive_attention_status(&[
        RiskSignal::resolved(RiskSeverity::Critical),
        RiskSignal::unresolved(RiskSeverity::Low),
        RiskSignal::unresolved(RiskSeverity::Medium),
    ]);
    assert_eq!(needs_attention, RiskAttentionStatus::NeedsAttention);
    assert_eq!(
        needs_attention.as_persona_health_status(),
        "needs_attention"
    );

    let at_risk = RiskEngine::derive_attention_status(&[
        RiskSignal::unresolved(RiskSeverity::Medium),
        RiskSignal::unresolved(RiskSeverity::High),
    ]);
    assert_eq!(at_risk, RiskAttentionStatus::AtRisk);
    assert_eq!(at_risk.as_persona_health_status(), "at_risk");
}

#[test]
fn risk_severity_rejects_unknown_compatibility_values() {
    let error = RiskSeverity::parse("urgent").expect_err("unknown severity must be rejected");

    assert_eq!(error.to_string(), "invalid risk severity `urgent`");
}

#[test]
fn risk_engine_builds_source_backed_persona_observation_draft() {
    let draft = RiskEngine::persona_observation(
        "person:v1:email:alice@example.com",
        "relationship_attention",
        "Open evidence-backed relationship risk requires owner review.",
        "high",
        "communication_messages:message-1",
    )
    .expect("source-backed risk observation should be valid");

    assert_eq!(draft.affected_entity_kind, "persona");
    assert_eq!(
        draft.affected_entity_id,
        "person:v1:email:alice@example.com"
    );
    assert_eq!(draft.risk_type, "relationship_attention");
    assert_eq!(
        draft.evidence,
        "Open evidence-backed relationship risk requires owner review."
    );
    assert_eq!(draft.source, "communication_messages:message-1");
    assert_eq!(draft.confidence, 0.5);
    assert_eq!(draft.severity, RiskSeverity::High);
    assert_eq!(draft.severity.as_str(), "high");
    assert_eq!(draft.suggested_handling_state, "review_now");
    assert_eq!(draft.review_state, "suggested");
}

#[test]
fn risk_engine_rejects_unsourced_persona_observation() {
    let error = RiskEngine::persona_observation(
        "person:v1:email:alice@example.com",
        "relationship_attention",
        "Open evidence-backed relationship risk requires owner review.",
        "high",
        " ",
    )
    .expect_err("risk observation source should be required");

    assert_eq!(
        error.to_string(),
        "risk observation source must not be empty"
    );
}
