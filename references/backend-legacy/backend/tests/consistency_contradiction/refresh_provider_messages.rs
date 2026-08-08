use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::engines::consistency::{
    models::{ContradictionSeverity, ContradictionSourceKind},
    store::ContradictionObservationStore,
};
use serde_json::json;

use super::support::{
    live_consistency_pool, seed_telegram_message, seed_whatsapp_message, seed_zulip_message,
    unique_suffix,
};

#[tokio::test]
async fn contradiction_refresh_detects_telegram_message_claim_against_active_persona_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction telegram message refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-telegram-{suffix}@example.com");
    let sender_id = format!("telegram-sender-{suffix}");
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&email_address)
        .await
        .expect("person");
    sqlx::query(
        r#"
        INSERT INTO persona_identities (persona_id, identity_type, identity_value, source, confidence, status)
        VALUES ($1, 'telegram', $2, 'test', 1.0, 'active')
        "#,
    )
    .bind(&person.persona_id)
    .bind(&sender_id)
    .execute(&pool)
    .await
    .expect("telegram identity");
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
    let message_id = seed_telegram_message(&pool, suffix, &sender_id, "Location: Madrid").await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("telegram message claim should contradict active remembered person fact");

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
async fn contradiction_refresh_detects_whatsapp_message_claim_against_active_persona_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction WhatsApp message refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-whatsapp-{suffix}@example.com");
    let sender_id = format!("whatsapp-sender-{suffix}");
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&email_address)
        .await
        .expect("person");
    sqlx::query(
        r#"
        INSERT INTO persona_identities (persona_id, identity_type, identity_value, source, confidence, status)
        VALUES ($1, 'whatsapp', $2, 'test', 1.0, 'active')
        "#,
    )
    .bind(&person.persona_id)
    .bind(&sender_id)
    .execute(&pool)
    .await
    .expect("whatsapp identity");
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
    let message_id = seed_whatsapp_message(&pool, suffix, &sender_id, "Location: Madrid").await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("WhatsApp message claim should contradict active remembered person fact");

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
async fn contradiction_refresh_detects_zulip_message_claim_against_active_persona_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction Zulip message refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-zulip-{suffix}@example.com");
    let sender_email = format!("zulip-sender-{suffix}@example.com");
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&email_address)
        .await
        .expect("person");
    sqlx::query(
        r#"
        INSERT INTO persona_identities (persona_id, identity_type, identity_value, source, confidence, status)
        VALUES ($1, 'zulip', $2, 'test', 1.0, 'active')
        "#,
    )
    .bind(&person.persona_id)
    .bind(&sender_email)
    .execute(&pool)
    .await
    .expect("zulip identity");
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
    let message_id = seed_zulip_message(&pool, suffix, &sender_email, "Location: Madrid").await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("Zulip message claim should contradict active remembered person fact");

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
