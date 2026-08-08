use makosh_hub_backend::engines::enrichment::engine::EnrichmentEngine;
use serde_json::json;

#[test]
fn enrichment_engine_builds_persona_favorite_preference_draft() {
    let draft =
        EnrichmentEngine::persona_favorite_preference("person:v1:email:alice@example.com", true)
            .expect("favorite state should create a preference draft");

    assert_eq!(draft.preference_type, "ui:favorite");
    assert_eq!(draft.value, "true");
    assert_eq!(
        draft.source,
        "personas.is_favorite:person:v1:email:alice@example.com"
    );
    assert_eq!(draft.confidence, 1.0);
}

#[test]
fn enrichment_engine_skips_persona_favorite_preference_when_disabled() {
    let draft =
        EnrichmentEngine::persona_favorite_preference("person:v1:email:alice@example.com", false);

    assert!(draft.is_none());
}

#[test]
fn enrichment_engine_builds_source_backed_persona_observation_candidate() {
    let draft = EnrichmentEngine::persona_observation_candidate(
        "person:v1:email:alice@example.com",
        "communication_messages:message-1",
        "prefers concise asynchronous summaries",
        json!({
            "field": "communication_style",
            "value": "concise asynchronous summaries"
        }),
        0.82,
    )
    .expect("source-backed candidate should be valid");

    assert_eq!(draft.entity_kind, "persona");
    assert_eq!(draft.entity_id, "person:v1:email:alice@example.com");
    assert_eq!(draft.source, "communication_messages:message-1");
    assert_eq!(
        draft.extracted_claim,
        "prefers concise asynchronous summaries"
    );
    assert_eq!(draft.confidence, 0.82);
    assert_eq!(draft.review_state, "pending");
    assert_eq!(draft.freshness, "current");
    assert!(!draft.conflict_marker);
    assert_eq!(draft.data["field"], "communication_style");
    assert_eq!(
        draft.data["_enrichment"]["affected_entity_id"],
        "person:v1:email:alice@example.com"
    );
    assert_eq!(
        draft.data["_enrichment"]["extracted_claim"],
        "prefers concise asynchronous summaries"
    );
}

#[test]
fn enrichment_engine_rejects_unsourced_persona_observation_candidate() {
    let error = EnrichmentEngine::persona_observation_candidate(
        "person:v1:email:alice@example.com",
        " ",
        "prefers concise asynchronous summaries",
        json!({"field": "communication_style"}),
        0.82,
    )
    .expect_err("candidate source should be required");

    assert_eq!(
        error.to_string(),
        "enrichment candidate source must not be empty"
    );
}
