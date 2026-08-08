use makosh_hub_backend::engines::trust::{engine::TrustEngine, models::TrustSignalKind};

#[test]
fn trust_engine_maps_persona_compatibility_score_to_relationship_signal() {
    let signal = TrustEngine::persona_compatibility_score_signal(82);

    assert_eq!(signal.kind, TrustSignalKind::PersonaCompatibilityScore);
    assert_eq!(signal.relationship_type, "trusts");
    assert_eq!(signal.trust_score, 0.82);
    assert_eq!(signal.strength_score, 0.5);
    assert_eq!(signal.confidence, 1.0);
    assert_eq!(
        signal.explanation,
        "compatibility personas.trust_score signal"
    );
}

#[test]
fn trust_engine_clamps_legacy_persona_scores_to_relationship_range() {
    let low = TrustEngine::persona_compatibility_score_signal(-20);
    let high = TrustEngine::persona_compatibility_score_signal(135);

    assert_eq!(low.trust_score, 0.0);
    assert_eq!(high.trust_score, 1.0);
}

#[test]
fn trust_engine_builds_source_reliability_signal_for_review() {
    let signal = TrustEngine::source_reliability_signal(
        "persona_enrichment:persona:v1:human:alice:trust_score",
        "trust_score=82",
        0.82,
    )
    .expect("source-backed trust signal should be valid");

    assert_eq!(signal.kind, TrustSignalKind::SourceReliability);
    assert_eq!(
        signal.affected_source,
        "persona_enrichment:persona:v1:human:alice:trust_score"
    );
    assert_eq!(signal.evidence, "trust_score=82");
    assert_eq!(signal.confidence, 0.82);
    assert_eq!(signal.direction.as_str(), "positive");
    assert_eq!(signal.explanation, "source reliability signal for review");
}

#[test]
fn trust_engine_rejects_unsourced_source_reliability_signal() {
    let error = TrustEngine::source_reliability_signal(" ", "trust_score=82", 0.82)
        .expect_err("source reliability signal source should be required");

    assert_eq!(
        error.to_string(),
        "trust signal affected source must not be empty"
    );
}
