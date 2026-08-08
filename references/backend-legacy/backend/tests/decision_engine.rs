use makosh_hub_backend::domains::decisions::extraction::engine::DecisionEngine;
use makosh_hub_backend::domains::decisions::extraction::errors::DecisionEngineError;
use makosh_hub_backend::domains::decisions::extraction::models::{
    DecisionCandidateKind, DecisionExtractionInput,
};
use serde_json::json;

#[test]
fn decision_engine_detects_explicit_communication_decision_candidate() {
    let input = DecisionExtractionInput::communication(
        "message:decision-engine",
        "Decision: Use local-first storage because private context must work offline.",
        DecisionEntityKind::Project,
        "project:v1:makosh",
    )
    .decided_by(
        DecisionEntityKind::Persona,
        "person:v1:email:owner@example.com",
    );

    let result = DecisionEngine::detect_candidates(&input).expect("detect decisions");

    assert_eq!(result.decisions.len(), 1);
    let candidate = &result.decisions[0];
    assert_eq!(candidate.kind, DecisionCandidateKind::ExplicitDecision);
    assert_eq!(candidate.title, "Use local-first storage");
    assert_eq!(candidate.rationale, "private context must work offline");
    assert_eq!(
        candidate.quote,
        "Decision: Use local-first storage because private context must work offline."
    );
    assert_eq!(
        candidate.decided_by_entity_kind,
        Some(DecisionEntityKind::Persona)
    );
    assert_eq!(
        candidate.decided_by_entity_id.as_deref(),
        Some("person:v1:email:owner@example.com")
    );
    assert_eq!(candidate.confidence, 0.83);
    assert_eq!(candidate.review_state, DecisionReviewState::Suggested);
    assert_eq!(
        candidate.evidence_source_kind,
        DecisionEvidenceSourceKind::Communication
    );
    assert_eq!(candidate.evidence_source_id, "message:decision-engine");
    assert_eq!(candidate.impacted_entities.len(), 1);
    assert_eq!(
        candidate.impacted_entities[0].entity_kind,
        DecisionEntityKind::Project
    );
    assert_eq!(
        candidate.impacted_entities[0].entity_id,
        "project:v1:makosh"
    );

    let (decision, evidence, impacted_entities) = candidate.to_decision_draft();

    assert_eq!(decision.title, "Use local-first storage");
    assert_eq!(decision.rationale, "private context must work offline");
    assert_eq!(decision.review_state, DecisionReviewState::Suggested);
    assert_eq!(decision.confidence, 0.83);
    assert_eq!(
        decision.metadata,
        json!({
            "engine": "decision",
            "candidate_kind": "explicit_decision"
        })
    );
    assert_eq!(
        evidence.source_kind,
        DecisionEvidenceSourceKind::Communication
    );
    assert_eq!(evidence.source_id, "message:decision-engine");
    assert_eq!(
        evidence.quote.as_deref(),
        Some("Decision: Use local-first storage because private context must work offline.")
    );
    assert_eq!(evidence.confidence, 0.83);
    assert_eq!(impacted_entities.len(), 1);
    assert_eq!(impacted_entities[0].impact_type, "decision_context");
}

#[test]
fn decision_engine_ignores_non_decision_evidence() {
    let input = DecisionExtractionInput::document(
        "document:status-note",
        "The team discussed storage options but no decision was made.",
        DecisionEntityKind::Project,
        "project:v1:makosh",
    );

    let result = DecisionEngine::detect_candidates(&input).expect("detect decisions");

    assert!(result.decisions.is_empty());
}

#[test]
fn decision_engine_rejects_empty_source_evidence_before_detection() {
    let input = DecisionExtractionInput::communication(
        "message:empty-decision",
        " ",
        DecisionEntityKind::Project,
        "project:v1:makosh",
    );

    let error = DecisionEngine::detect_candidates(&input)
        .expect_err("empty evidence text must be rejected");

    assert!(matches!(error, DecisionEngineError::EmptyField("text")));
}
use makosh_hub_backend::domains::decisions::models::entity_kind::DecisionEntityKind;
use makosh_hub_backend::domains::decisions::models::source_kind::DecisionEvidenceSourceKind;
use makosh_hub_backend::domains::decisions::models::states::DecisionReviewState;
