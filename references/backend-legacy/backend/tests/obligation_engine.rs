use makosh_hub_backend::engines::obligation::{
    engine::ObligationEngine,
    models::{
        ObligationCandidateKind, ObligationEntityKind, ObligationEvidenceSourceKind,
        ObligationExtractionInput, ObligationReviewState,
    },
};

#[test]
fn obligation_engine_detects_owner_promise_from_communication() {
    let input = ObligationExtractionInput::communication(
        "message:proposal-commitment",
        "I will send the revised project proposal by Friday 5pm. Thanks.",
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
    )
    .beneficiary(ObligationEntityKind::Project, "project:v1:client-dossier");

    let result = ObligationEngine::detect_candidates(&input).expect("detect obligation candidates");

    assert_eq!(result.obligations.len(), 1);
    assert_eq!(result.follow_ups.len(), 1);
    assert_eq!(result.task_candidates.len(), 1);

    let candidate = &result.obligations[0];
    assert_eq!(candidate.kind, ObligationCandidateKind::Commitment);
    assert_eq!(candidate.statement, "send the revised project proposal");
    assert_eq!(
        candidate.quote,
        "I will send the revised project proposal by Friday 5pm."
    );
    assert_eq!(candidate.due_text.as_deref(), Some("Friday 5pm"));
    assert_eq!(candidate.condition, None);
    assert_eq!(candidate.confidence, 0.84);
    assert_eq!(candidate.review_state, ObligationReviewState::Suggested);
    assert_eq!(
        candidate.obligated_entity_kind,
        ObligationEntityKind::Persona
    );
    assert_eq!(
        candidate.obligated_entity_id,
        "person:v1:email:owner@example.com"
    );
    assert_eq!(
        candidate.beneficiary_entity_kind,
        Some(ObligationEntityKind::Project)
    );
    assert_eq!(
        candidate.beneficiary_entity_id.as_deref(),
        Some("project:v1:client-dossier")
    );
    assert_eq!(
        candidate.evidence_source_kind,
        ObligationEvidenceSourceKind::Communication
    );
    assert_eq!(candidate.evidence_source_id, "message:proposal-commitment");

    assert_eq!(
        result.task_candidates[0].statement,
        "send the revised project proposal"
    );
    assert_eq!(
        result.follow_ups[0].source_obligation_statement,
        "send the revised project proposal"
    );
}

#[test]
fn obligation_engine_detects_request_to_owner_without_autoconfirming() {
    let input = ObligationExtractionInput::communication(
        "message:agreement-request",
        "Please send the signed agreement before Monday morning.",
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
    );

    let result = ObligationEngine::detect_candidates(&input).expect("detect obligation candidates");

    assert_eq!(result.obligations.len(), 1);
    let candidate = &result.obligations[0];
    assert_eq!(candidate.kind, ObligationCandidateKind::Request);
    assert_eq!(candidate.statement, "send the signed agreement");
    assert_eq!(candidate.due_text.as_deref(), Some("Monday morning"));
    assert_eq!(candidate.review_state, ObligationReviewState::Suggested);
    assert_eq!(candidate.confidence, 0.76);
}

#[test]
fn obligation_engine_ignores_deadline_without_commitment_language() {
    let input = ObligationExtractionInput::communication(
        "message:office-hours",
        "The office closes by Friday 5pm. The report was already sent.",
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
    );

    let result = ObligationEngine::detect_candidates(&input).expect("detect obligation candidates");

    assert_eq!(result.obligations, Vec::new());
    assert_eq!(result.task_candidates, Vec::new());
    assert_eq!(result.follow_ups, Vec::new());
}

#[test]
fn obligation_engine_rejects_empty_source_evidence_before_detection() {
    let input = ObligationExtractionInput::communication(
        " ",
        "I will send the revised project proposal by Friday 5pm.",
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
    );

    let error =
        ObligationEngine::detect_candidates(&input).expect_err("empty source id must be rejected");

    assert_eq!(error.to_string(), "source_id must not be empty");
}
