use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::personas::core::interaction_contexts::{
    NewPersonaInteractionContext, PersonaInteractionContextStore,
};
use makosh_hub_backend::domains::personas::enrichment::store::PersonaEnrichmentStore;
use makosh_hub_backend::domains::personas::enrichment_engine::EnrichmentResultStore;
use makosh_hub_backend::domains::personas::health::PersonaHealthStore;
use makosh_hub_backend::domains::personas::memory::facts::PersonaFactStore;
use serde_json::json;

use super::support::{live_personas_pool, unique_suffix};

#[tokio::test]
async fn persona_interaction_context_upsert_and_delete_materializes_interaction_preferences_against_postgres()
 {
    let Some(pool) = live_personas_pool("persona interaction context preference adapter").await
    else {
        return;
    };
    let person_store = PersonaProjectionStore::new(pool.clone());
    let persona_store = PersonaInteractionContextStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_persona(&format!("persona-context-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let persona_id = format!("interaction-context-{suffix}");

    let context = persona_store
        .upsert(&NewPersonaInteractionContext {
            interaction_context_id: persona_id.clone(),
            source_persona_id: person.persona_id.clone(),
            name: "Work Context".to_owned(),
            context: Some("Professional replies for project updates".to_owned()),
            default_tone: Some("concise".to_owned()),
            default_language: Some("en".to_owned()),
            preferred_channel: Some("email".to_owned()),
        })
        .await
        .expect("upsert interaction context");

    let source = format!("persona_interaction_contexts:{persona_id}");
    let preferences: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT preference_type, value, source
        FROM persona_preferences
        WHERE persona_id = $1 AND source = $2
        ORDER BY preference_type
        "#,
    )
    .bind(&person.persona_id)
    .bind(&source)
    .fetch_all(&pool)
    .await
    .expect("interaction context preferences");

    assert_eq!(
        preferences,
        vec![
            (
                format!("interaction_context:{persona_id}:context"),
                "Professional replies for project updates".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:default_language"),
                "en".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:default_tone"),
                "concise".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:name"),
                "Work Context".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:preferred_channel"),
                "email".to_owned(),
                source.clone(),
            ),
        ]
    );

    let deleted = persona_store
        .delete(&context.interaction_context_id)
        .await
        .expect("delete interaction context");
    assert!(deleted);

    let remaining_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM persona_preferences WHERE persona_id = $1 AND source = $2",
    )
    .bind(&person.persona_id)
    .bind(&source)
    .fetch_one(&pool)
    .await
    .expect("remaining interaction context preference count");
    assert_eq!(remaining_count, 0);
}

#[tokio::test]
async fn persona_notes_materialize_memory_card_against_postgres() {
    let Some(pool) = live_personas_pool("person notes memory adapter").await else {
        return;
    };
    let person_store = PersonaProjectionStore::new(pool.clone());
    let enrichment_store = PersonaEnrichmentStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_persona(&format!("notes-memory-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let source = format!("personas.notes:{}", person.persona_id);

    enrichment_store
        .set_notes(
            &person.persona_id,
            "Remember that this Persona prefers concise written summaries.",
        )
        .await
        .expect("set notes");

    let root_notes: String = sqlx::query_scalar("SELECT notes FROM personas WHERE persona_id = $1")
        .bind(&person.persona_id)
        .fetch_one(&pool)
        .await
        .expect("root compatibility notes");
    assert_eq!(
        root_notes,
        "Remember that this Persona prefers concise written summaries."
    );

    let memory_card: (String, String, String, i16) = sqlx::query_as(
        r#"
        SELECT title, description, source, importance
        FROM persona_memory_cards
        WHERE persona_id = $1 AND source = $2
        "#,
    )
    .bind(&person.persona_id)
    .bind(&source)
    .fetch_one(&pool)
    .await
    .expect("notes memory card");
    assert_eq!(memory_card.0, "Compatibility notes");
    assert_eq!(
        memory_card.1,
        "Remember that this Persona prefers concise written summaries."
    );
    assert_eq!(memory_card.2, source);
    assert_eq!(memory_card.3, 5);
}

#[tokio::test]
async fn persona_fact_upsert_uses_memory_engine_source_backed_draft_against_postgres() {
    let Some(pool) = live_personas_pool("person fact memory engine adapter").await else {
        return;
    };
    let person_store = PersonaProjectionStore::new(pool.clone());
    let fact_store = PersonaFactStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_persona(&format!("fact-memory-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let fact = fact_store
        .upsert(
            &person.persona_id,
            " interest ",
            " local-first systems ",
            " communication_messages:message-1 ",
            0.84,
        )
        .await
        .expect("upsert fact");

    assert_eq!(fact.persona_id, person.persona_id);
    assert_eq!(fact.fact_type, "interest");
    assert_eq!(fact.value, "local-first systems");
    assert_eq!(fact.source, "communication_messages:message-1");
    assert!((fact.confidence - 0.84).abs() < 0.0001);
    assert!(fact.is_active);

    let stored_fact: (String, String, String, f64) = sqlx::query_as(
        r#"
        SELECT fact_type, value, source, confidence::float8 AS confidence
        FROM persona_facts
        WHERE id::text = $1
        "#,
    )
    .bind(&fact.id)
    .fetch_one(&pool)
    .await
    .expect("stored fact");
    assert_eq!(stored_fact.0, "interest");
    assert_eq!(stored_fact.1, "local-first systems");
    assert_eq!(stored_fact.2, "communication_messages:message-1");
    assert!((stored_fact.3 - 0.84).abs() < 0.0001);
}

#[tokio::test]
async fn persona_favorite_toggle_materializes_ui_preference_against_postgres() {
    let Some(pool) = live_personas_pool("person favorite preference adapter").await else {
        return;
    };
    let person_store = PersonaProjectionStore::new(pool.clone());
    let enrichment_store = PersonaEnrichmentStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_persona(&format!("favorite-preference-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let source = format!("personas.is_favorite:{}", person.persona_id);

    let is_favorite = enrichment_store
        .toggle_favorite(&person.persona_id)
        .await
        .expect("toggle favorite on");
    assert!(is_favorite);

    let preference: (String, String, String, f32) = sqlx::query_as(
        r#"
        SELECT preference_type, value, source, confidence
        FROM persona_preferences
        WHERE persona_id = $1 AND preference_type = 'ui:favorite'
        "#,
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("favorite UI preference");
    assert_eq!(preference.0, "ui:favorite");
    assert_eq!(preference.1, "true");
    assert_eq!(preference.2, source);
    assert_eq!(preference.3, 1.0);

    let is_favorite = enrichment_store
        .toggle_favorite(&person.persona_id)
        .await
        .expect("toggle favorite off");
    assert!(!is_favorite);

    let preference_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM persona_preferences WHERE persona_id = $1 AND preference_type = 'ui:favorite'",
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("remaining favorite preference count");
    assert_eq!(preference_count, 0);
}

#[tokio::test]
async fn persona_enrichment_result_upsert_materializes_pending_source_backed_candidate_against_postgres()
 {
    let Some(pool) = live_personas_pool("person enrichment result candidate").await else {
        return;
    };
    let person_store = PersonaProjectionStore::new(pool.clone());
    let enrichment_result_store = EnrichmentResultStore::new(pool);
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_persona(&format!("enrichment-candidate-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let result = enrichment_result_store
        .upsert(
            &person.persona_id,
            "communication_messages:message-1",
            json!({
                "field": "communication_style",
                "value": "concise asynchronous summaries",
                "extracted_claim": "prefers concise asynchronous summaries"
            }),
            0.82,
        )
        .await
        .expect("upsert enrichment result");

    assert_eq!(result.persona_id, person.persona_id);
    assert_eq!(result.source, "communication_messages:message-1");
    assert!((result.confidence - 0.82).abs() < 0.0001);
    assert_eq!(result.status, "pending");
    assert_eq!(result.data["field"], "communication_style");
    assert_eq!(
        result.data["_enrichment"]["affected_entity_kind"],
        "persona"
    );
    assert_eq!(
        result.data["_enrichment"]["affected_entity_id"],
        person.persona_id
    );
    assert_eq!(
        result.data["_enrichment"]["extracted_claim"],
        "prefers concise asynchronous summaries"
    );
    assert_eq!(result.data["_enrichment"]["review_state"], "pending");
    assert_eq!(result.data["_enrichment"]["freshness"], "current");
    assert_eq!(result.data["_enrichment"]["conflict_marker"], false);
}

#[tokio::test]
async fn persona_watchlist_toggle_materializes_ui_preference_against_postgres() {
    let Some(pool) = live_personas_pool("person watchlist preference adapter").await else {
        return;
    };
    let person_store = PersonaProjectionStore::new(pool.clone());
    let health_store = PersonaHealthStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_persona(&format!("watchlist-preference-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let source = format!("personas.watchlist:{}", person.persona_id);

    let watchlist = health_store
        .toggle_watchlist(&person.persona_id)
        .await
        .expect("toggle watchlist on");
    assert!(watchlist);

    let preference: (String, String, String, f32) = sqlx::query_as(
        r#"
        SELECT preference_type, value, source, confidence
        FROM persona_preferences
        WHERE persona_id = $1 AND preference_type = 'ui:watchlist'
        "#,
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("watchlist UI preference");
    assert_eq!(preference.0, "ui:watchlist");
    assert_eq!(preference.1, "true");
    assert_eq!(preference.2, source);
    assert_eq!(preference.3, 1.0);

    let watchlist = health_store
        .toggle_watchlist(&person.persona_id)
        .await
        .expect("toggle watchlist off");
    assert!(!watchlist);

    let preference_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM persona_preferences WHERE persona_id = $1 AND preference_type = 'ui:watchlist'",
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("remaining watchlist preference count");
    assert_eq!(preference_count, 0);
}
