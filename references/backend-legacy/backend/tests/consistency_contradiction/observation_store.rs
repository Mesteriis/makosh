use makosh_hub_backend::engines::consistency::{
    models::{
        ContradictionReviewState, ContradictionSeverity, ContradictionSourceKind,
        NewContradictionObservation,
    },
    store::ContradictionObservationStore,
};
use serde_json::json;

use super::support::{live_consistency_pool, unique_suffix};

#[tokio::test]
async fn contradiction_observation_store_upserts_reviewable_observation_against_postgres() {
    let Some(pool) = live_consistency_pool("contradiction observation").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool);
    let suffix = unique_suffix();
    let observation = NewContradictionObservation {
        old_source_kind: ContradictionSourceKind::Memory,
        old_source_id: format!("memory:budget:{suffix}"),
        new_source_kind: ContradictionSourceKind::Communication,
        new_source_id: format!("message:budget:{suffix}"),
        affected_entities: json!([
            {"entity_kind": "project", "entity_id": format!("project:v1:{suffix}")}
        ]),
        conflict_type: "direct_contradiction".to_owned(),
        old_claim: "budget=approved".to_owned(),
        new_claim: "budget=rejected".to_owned(),
        confidence: 0.88,
        severity: ContradictionSeverity::High,
        review_state: ContradictionReviewState::Suggested,
        metadata: json!({"detector": "structured_claim_test"}),
    };

    let first = store
        .upsert(&observation)
        .await
        .expect("first contradiction upsert");
    let second = store
        .upsert(&observation)
        .await
        .expect("idempotent contradiction upsert");

    assert_eq!(first.observation_id, second.observation_id);
    assert_eq!(first.review_state, ContradictionReviewState::Suggested);
    assert_eq!(first.severity, ContradictionSeverity::High);
    assert_eq!(first.confidence, 0.88);

    let open = store.list_open(20).await.expect("open contradictions");
    assert!(
        open.iter()
            .any(|item| item.observation_id == first.observation_id)
    );

    let reviewed = store
        .set_review_state(
            &first.observation_id,
            ContradictionReviewState::UserConfirmed,
            "test-reviewer",
            Some("confirmed contradiction"),
        )
        .await
        .expect("review contradiction");

    assert_eq!(
        reviewed.review_state,
        ContradictionReviewState::UserConfirmed
    );
    assert_eq!(reviewed.reviewed_by.as_deref(), Some("test-reviewer"));
    assert_eq!(
        reviewed.resolution.as_deref(),
        Some("confirmed contradiction")
    );
}

#[test]
fn contradiction_observation_rejects_invalid_confidence_before_database_write() {
    let observation = NewContradictionObservation {
        old_source_kind: ContradictionSourceKind::Memory,
        old_source_id: "memory:invalid".to_owned(),
        new_source_kind: ContradictionSourceKind::Communication,
        new_source_id: "message:invalid".to_owned(),
        affected_entities: json!([]),
        conflict_type: "direct_contradiction".to_owned(),
        old_claim: "status=active".to_owned(),
        new_claim: "status=archived".to_owned(),
        confidence: 1.2,
        severity: ContradictionSeverity::Medium,
        review_state: ContradictionReviewState::Suggested,
        metadata: json!({}),
    };

    let error = observation
        .validate()
        .expect_err("invalid confidence must be rejected");

    assert_eq!(
        error.to_string(),
        "confidence must be between 0.0 and 1.0: 1.2"
    );
}
