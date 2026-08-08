use chrono::{Duration, Utc};
use makosh_hub_backend::domains::obligations::models::{
    entity_kind::ObligationEntityKind, states::ObligationStatus,
};
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::personas::core::roles::PersonaRoleStore;
use makosh_hub_backend::domains::personas::enrichment::store::PersonaEnrichmentStore;
use makosh_hub_backend::domains::personas::intelligence::CommunicationFingerprint;
use makosh_hub_backend::domains::personas::trust::promises::PersonaPromiseStore;

use super::support::{live_personas_pool, run_persona_derived_evidence_consumer, unique_suffix};

#[tokio::test]
async fn persona_role_assign_and_remove_materializes_relationship_against_postgres() {
    let Some(pool) = live_personas_pool("person role relationship adapter").await else {
        return;
    };
    let person_store = PersonaProjectionStore::new(pool.clone());
    let role_store = PersonaRoleStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_persona(&format!("role-adapter-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let _role = role_store
        .assign(
            &person.persona_id,
            "Technical Advisor",
            Some("persona:owner"),
        )
        .await
        .expect("assign person role");
    run_persona_derived_evidence_consumer(pool.clone()).await;

    let relationship: (
        String,
        String,
        String,
        String,
        String,
        String,
        f64,
        f64,
        f64,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT
            relationship_id,
            source_entity_kind,
            source_entity_id,
            target_entity_kind,
            target_entity_id AS entity_id,
            review_state,
            trust_score::float8 AS trust_score,
            strength_score::float8 AS strength_score,
            confidence::float8 AS confidence,
            metadata
        FROM relationships
        WHERE source_entity_kind = 'persona'
          AND source_entity_id = $1
          AND target_entity_kind = 'knowledge'
          AND target_entity_id = 'persona_role:technical_advisor'
          AND relationship_type = 'has_role'
        "#,
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("role relationship");

    assert_eq!(relationship.1, "persona");
    assert_eq!(relationship.2, person.persona_id);
    assert_eq!(relationship.3, "knowledge");
    assert_eq!(relationship.4, "persona_role:technical_advisor");
    assert_eq!(relationship.5, "user_confirmed");
    assert_eq!(relationship.6, 1.0);
    assert_eq!(relationship.7, 0.7);
    assert_eq!(relationship.8, 1.0);
    assert_eq!(relationship.9["compatibility_source"], "persona_roles");
    assert_eq!(relationship.9["role"], "Technical Advisor");
    assert_eq!(relationship.9["assigned_by"], "persona:owner");

    let evidence: (String, String, Option<String>, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, excerpt, metadata
        FROM relationship_evidence
        WHERE relationship_id = $1
        "#,
    )
    .bind(&relationship.0)
    .fetch_one(&pool)
    .await
    .expect("role relationship evidence");
    assert_eq!(evidence.0, "observation");
    assert!(!evidence.1.is_empty());
    assert_eq!(evidence.2.as_deref(), Some("Technical Advisor"));
    assert_eq!(evidence.3["compatibility_source"], "persona_roles");
    let role_observation_kind: String = sqlx::query_scalar(
        r#"
        SELECT kinds.code
        FROM observations observation
        JOIN observation_kind_definitions kinds
          ON kinds.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&evidence.1)
    .fetch_one(&pool)
    .await
    .expect("role observation kind");
    assert_eq!(role_observation_kind, "PERSONA_ROLE");
    let role_observation_payload: serde_json::Value =
        sqlx::query_scalar("SELECT payload FROM observations WHERE observation_id = $1")
            .bind(&evidence.1)
            .fetch_one(&pool)
            .await
            .expect("role observation payload");
    assert_eq!(role_observation_payload["persona_id"], person.persona_id);
    assert!(role_observation_payload.get("person_id").is_none());

    let removed = role_store
        .remove(&person.persona_id, "Technical Advisor")
        .await
        .expect("remove person role");
    assert!(removed);
    run_persona_derived_evidence_consumer(pool.clone()).await;

    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM relationships WHERE relationship_id = $1")
            .bind(&relationship.0)
            .fetch_one(&pool)
            .await
            .expect("demoted role relationship");
    assert_eq!(review_state, "user_rejected");
}

#[tokio::test]
async fn persona_enrichment_trust_score_materializes_owner_relationship_against_postgres() {
    let Some(pool) = live_personas_pool("person enrichment trust relationship adapter").await
    else {
        return;
    };
    let person_store = PersonaProjectionStore::new(pool.clone());
    let enrichment_store = PersonaEnrichmentStore::new(pool.clone());
    let suffix = unique_suffix();
    let owner = person_store
        .upsert_email_persona(&format!("trust-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    let target = person_store
        .upsert_email_persona(&format!("trust-target-{suffix}@example.com"))
        .await
        .expect("upsert target persona");
    person_store
        .set_owner_persona(&owner.persona_id)
        .await
        .expect("set owner persona");

    let fingerprint = CommunicationFingerprint {
        avg_message_length: Some(120),
        avg_response_hours: Some(3.5),
        frequent_topics: vec!["project".to_owned()],
        typical_tone: Some("friendly".to_owned()),
        detected_language: Some("en".to_owned()),
        writing_style: Some("balanced".to_owned()),
        preferred_time_of_day: None,
        trust_score: Some(82),
    };

    enrichment_store
        .enrich_person(&target.persona_id, &fingerprint)
        .await
        .expect("enrich target persona");
    run_persona_derived_evidence_consumer(pool.clone()).await;

    let relationship: (
        String,
        String,
        String,
        String,
        f64,
        f64,
        f64,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT
            relationship_id,
            source_entity_id,
            target_entity_id AS entity_id,
            review_state,
            trust_score::float8 AS trust_score,
            strength_score::float8 AS strength_score,
            confidence::float8 AS confidence,
            metadata
        FROM relationships
        WHERE source_entity_kind = 'persona'
          AND source_entity_id = $1
          AND target_entity_kind = 'persona'
          AND target_entity_id = $2
          AND relationship_type = 'trusts'
        "#,
    )
    .bind(&owner.persona_id)
    .bind(&target.persona_id)
    .fetch_one(&pool)
    .await
    .expect("owner trust relationship");

    assert_eq!(relationship.1, owner.persona_id);
    assert_eq!(relationship.2, target.persona_id);
    assert_eq!(relationship.3, "suggested");
    assert_eq!(relationship.4, 0.82);
    assert_eq!(relationship.5, 0.5);
    assert_eq!(relationship.6, 1.0);
    assert_eq!(
        relationship.7["compatibility_source"],
        "personas.trust_score"
    );
    assert_eq!(relationship.7["trust_score"], 82);

    let evidence: (String, String, Option<String>, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, excerpt, metadata
        FROM relationship_evidence
        WHERE relationship_id = $1
        "#,
    )
    .bind(&relationship.0)
    .fetch_one(&pool)
    .await
    .expect("trust relationship evidence");
    assert_eq!(evidence.0, "observation");
    assert!(!evidence.1.is_empty());
    assert_eq!(evidence.2.as_deref(), Some("trust_score=82"));
    assert_eq!(evidence.3["compatibility_source"], "personas.trust_score");
    assert_eq!(
        evidence.3["trust_source_reliability"]["signal_type"],
        "source_reliability"
    );
    assert_eq!(
        evidence.3["trust_source_reliability"]["affected_source"],
        format!("persona_enrichment:{}:trust_score", target.persona_id)
    );
    assert_eq!(
        evidence.3["trust_source_reliability"]["direction"],
        "positive"
    );
    assert_eq!(evidence.3["trust_source_reliability"]["confidence"], 0.82);
    let trust_observation_kind: String = sqlx::query_scalar(
        r#"
        SELECT kinds.code
        FROM observations observation
        JOIN observation_kind_definitions kinds
          ON kinds.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&evidence.1)
    .fetch_one(&pool)
    .await
    .expect("trust observation kind");
    assert_eq!(trust_observation_kind, "PERSONA_TRUST_SIGNAL");

    let review_item: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT
            review_item.review_item_id,
            review_item.item_kind,
            review_item.metadata->>'mirrored_from',
            review_item.metadata->>'relationship_id'
        FROM review_items review_item
        JOIN review_item_evidence evidence
          ON evidence.review_item_id = review_item.review_item_id
        WHERE evidence.observation_id = $1
          AND review_item.item_kind = 'potential_relationship'
          AND review_item.metadata->>'relationship_id' = $2
        "#,
    )
    .bind(&evidence.1)
    .bind(&relationship.0)
    .fetch_one(&pool)
    .await
    .expect("relationship review mirror");
    assert_eq!(review_item.1, "potential_relationship");
    assert_eq!(review_item.2, "relationships");
    assert_eq!(review_item.3, relationship.0);
}

#[tokio::test]
async fn persona_promise_create_materializes_user_confirmed_obligation_without_task_against_postgres()
 {
    let Some(pool) = live_personas_pool("person promise obligation adapter").await else {
        return;
    };
    let person_store = PersonaProjectionStore::new(pool.clone());
    let promise_store = PersonaPromiseStore::new(pool.clone());
    let obligation_store = ObligationStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_persona(&format!("promise-adapter-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let due_at = Utc::now() + Duration::days(5);
    let description = format!("Send the persona promise evidence package {suffix}");

    let promise = promise_store
        .create(&person.persona_id, &description, Some(due_at))
        .await
        .expect("create person promise");
    run_persona_derived_evidence_consumer(pool.clone()).await;

    let obligations = obligation_store
        .list_for_entity(ObligationEntityKind::Persona, &person.persona_id, 10)
        .await
        .expect("persona obligations");
    let obligation = obligations
        .iter()
        .find(|item| item.statement == description)
        .expect("person promise should create a durable Obligation");

    assert_eq!(
        obligation.review_state,
        ObligationReviewState::UserConfirmed
    );
    assert_eq!(obligation.status, ObligationStatus::Open);
    assert_eq!(
        obligation.due_at.map(|value| value.timestamp_micros()),
        Some(due_at.timestamp_micros())
    );
    assert_eq!(
        obligation.metadata["persona_promise_id"],
        serde_json::json!(promise.id)
    );

    let evidence: (String, String, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, quote FROM obligation_evidence WHERE obligation_id = $1",
    )
    .bind(&obligation.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation evidence");
    assert_eq!(evidence.0, "observation");
    assert!(!evidence.1.is_empty());
    assert_eq!(evidence.2.as_deref(), Some(description.as_str()));
    let promise_observation_kind: String = sqlx::query_scalar(
        r#"
        SELECT kinds.code
        FROM observations observation
        JOIN observation_kind_definitions kinds
          ON kinds.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&evidence.1)
    .fetch_one(&pool)
    .await
    .expect("promise observation kind");
    assert_eq!(promise_observation_kind, "PERSONA_PROMISE");

    let task_link_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM obligation_task_links WHERE obligation_id = $1")
            .bind(&obligation.obligation_id)
            .fetch_one(&pool)
            .await
            .expect("task link count");
    assert_eq!(task_link_count, 0);
}
use makosh_hub_backend::domains::obligations::models::states::ObligationReviewState;
use makosh_hub_backend::domains::obligations::store::ObligationStore;
