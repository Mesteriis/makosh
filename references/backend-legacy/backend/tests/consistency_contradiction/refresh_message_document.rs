use makosh_hub_backend::domains::documents::core::{
    models::NewDocumentImport, store::DocumentImportStore,
};
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::engines::consistency::{
    models::{ContradictionSeverity, ContradictionSourceKind},
    store::ContradictionObservationStore,
};
use serde_json::json;

use super::support::{live_consistency_pool, seed_message, unique_suffix};

#[tokio::test]
async fn contradiction_refresh_detects_message_claim_against_active_persona_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let sender = format!("polygraph-{suffix}@example.com");
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&sender)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO persona_facts (persona_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let message_id = seed_message(
        &pool,
        suffix,
        &sender,
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-polygraph-{suffix}"),
        &format!("Location update {suffix}"),
        "Location: Madrid",
    )
    .await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("message claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.persona_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "communication"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM persona_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_natural_language_message_claim_against_active_persona_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction natural-language refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let sender = format!("polygraph-natural-language-{suffix}@example.com");
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&sender)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO persona_facts (persona_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let message_id = seed_message(
        &pool,
        suffix,
        &sender,
        &[format!("owner-natural-language-{suffix}@example.com")],
        &format!("provider-polygraph-natural-language-{suffix}"),
        &format!("Natural language location update {suffix}"),
        "Quick update: I am now in Madrid.",
    )
    .await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("natural-language message claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM persona_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_document_claim_against_active_persona_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction document refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-document-{suffix}@example.com");
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&email_address)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO persona_facts (persona_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let document_id = format!("document_polygraph_{suffix}");
    DocumentImportStore::new(pool.clone())
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            format!("Persona dossier {suffix}"),
            format!("# Persona dossier\nEmail: {email_address}\nLocation: Madrid"),
        ))
        .await
        .expect("document import");

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == document_id)
        .expect("document claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Document
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.persona_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "document"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM persona_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}
