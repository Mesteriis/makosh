use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::platform::storage::database::Database;

use super::support::{
    LOCAL_API_TOKEN, build_personas_app_with_database, json_body, post_request_with_token,
    put_request_with_token, run_persona_derived_evidence_consumer, unique_suffix,
    urlencoding_percent_encode,
};

#[tokio::test]
async fn persona_manual_memory_entrypoints_capture_observations_against_postgres() {
    let Some(database_url) =
        super::support::live_database_url("person manual memory observations").await
    else {
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&format!("manual-memory-{suffix}@example.com"))
        .await
        .expect("upsert person");
    let app = build_personas_app_with_database(&database_url, database);
    let encoded_person_id = urlencoding_percent_encode(&person.persona_id);

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!("/api/v1/personas/{encoded_person_id}/notes"),
            json!({"notes": "Manual persona notes from observation-backed API"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("notes response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let notes_card_source: String = sqlx::query_scalar(
        r#"
        SELECT source
        FROM persona_memory_cards
        WHERE persona_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("notes memory card source");
    assert!(notes_card_source.starts_with("observation:"));
    let notes_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'notes'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("notes observation link");

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/personas/{encoded_person_id}/facts"),
            json!({
                "fact_type": "preference",
                "value": "local-first architecture",
                "confidence": 0.91
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fact response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let fact_body = json_body(response).await;
    let fact_id = fact_body["id"].as_str().expect("fact id");
    let fact_source: String =
        sqlx::query_scalar("SELECT source FROM persona_facts WHERE id::text = $1")
            .bind(fact_id)
            .fetch_one(&pool)
            .await
            .expect("fact source");
    assert!(fact_source.starts_with("observation:"));

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/personas/{encoded_person_id}/memory-cards"),
            json!({
                "title": "NAS shortlist",
                "description": "Shortlisted Synology and QNAP options",
                "importance": 7
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("memory card response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let card_body = json_body(response).await;
    let card_id = card_body["id"].as_str().expect("card id");
    let card_source: String =
        sqlx::query_scalar("SELECT source FROM persona_memory_cards WHERE id::text = $1")
            .bind(card_id)
            .fetch_one(&pool)
            .await
            .expect("memory card source");
    assert!(card_source.starts_with("observation:"));

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/personas/{encoded_person_id}/preferences"),
            json!({
                "preference_type": "timezone",
                "value": "Europe/Madrid"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("preference response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let pref_body = json_body(response).await;
    let pref_id = pref_body["id"].as_str().expect("preference id");
    let pref_source: String =
        sqlx::query_scalar("SELECT source FROM persona_preferences WHERE id::text = $1")
            .bind(pref_id)
            .fetch_one(&pool)
            .await
            .expect("preference source");
    assert!(pref_source.starts_with("observation:"));

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/personas/{encoded_person_id}/fingerprint"),
            json!({"fingerprint_data": "manual-fingerprint-trigger"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fingerprint response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    run_persona_derived_evidence_consumer(pool.clone()).await;

    let fingerprint_observation_row = sqlx::query(
        "SELECT observation.observation_id, observation.origin_kind, kind.code AS kind_code
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'personas'
           AND link.entity_kind = 'persona'
           AND link.entity_id = $1
           AND link.relationship_kind = 'profile_enrichment'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("fingerprint observation row");
    assert_eq!(
        fingerprint_observation_row
            .try_get::<String, _>("origin_kind")
            .expect("fingerprint origin"),
        "manual"
    );
    assert_eq!(
        fingerprint_observation_row
            .try_get::<String, _>("kind_code")
            .expect("fingerprint kind"),
        "PERSONA_MUTATION"
    );

    let trust_signal_count: i64 = sqlx::query_scalar(
        "SELECT count(*)
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE kind.code = 'PERSONA_TRUST_SIGNAL'
           AND observation.payload->>'persona_id' = $1
           AND (observation.payload ? 'person_id') = false",
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("person trust signal count");
    assert!(trust_signal_count >= 1);

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/personas/{encoded_person_id}/favorite"),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("favorite response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let favorite_source: String = sqlx::query_scalar(
        "SELECT source FROM persona_preferences WHERE persona_id = $1 AND preference_type = 'ui:favorite'",
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("favorite source");
    assert!(favorite_source.starts_with("observation:"));
    let favorite_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'favorite_toggle'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("favorite observation link");

    let response = app
        .oneshot(post_request_with_token(
            &format!("/api/v1/personas/{encoded_person_id}/watchlist"),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("watchlist response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let watchlist_source: String = sqlx::query_scalar(
        "SELECT source FROM persona_preferences WHERE persona_id = $1 AND preference_type = 'ui:watchlist'",
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("watchlist source");
    assert!(watchlist_source.starts_with("observation:"));
    let watchlist_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'watchlist_toggle'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("watchlist observation link");

    for source in [
        notes_card_source.clone(),
        fact_source.clone(),
        card_source.clone(),
        pref_source.clone(),
        favorite_source.clone(),
        watchlist_source.clone(),
    ] {
        let observation_id = source
            .strip_prefix("observation:")
            .expect("observation source prefix");
        let row = sqlx::query(
            "SELECT observation.observation_id, observation.origin_kind, kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )
        .bind(observation_id)
        .fetch_one(&pool)
        .await
        .expect("stored observation");
        assert_eq!(
            row.try_get::<String, _>("origin_kind")
                .expect("origin kind"),
            "manual"
        );
    }

    for source in [fact_source, pref_source] {
        let observation_id = source
            .strip_prefix("observation:")
            .expect("observation source prefix");
        let kind_code: String = sqlx::query_scalar(
            "SELECT kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )
        .bind(observation_id)
        .fetch_one(&pool)
        .await
        .expect("person record mutation kind code");
        assert_eq!(kind_code, "PERSONA_RECORD_MUTATION");
    }

    for observation_id in [
        notes_observation_id,
        notes_card_source
            .strip_prefix("observation:")
            .expect("observation source prefix")
            .to_owned(),
        card_source
            .strip_prefix("observation:")
            .expect("observation source prefix")
            .to_owned(),
    ] {
        let kind_code: String = sqlx::query_scalar(
            "SELECT kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )
        .bind(observation_id)
        .fetch_one(&pool)
        .await
        .expect("person memory card kind code");
        assert_eq!(kind_code, "PERSONA_MEMORY_CARD");
    }

    for observation_id in [favorite_observation_id, watchlist_observation_id] {
        let kind_code: String = sqlx::query_scalar(
            "SELECT kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )
        .bind(observation_id)
        .fetch_one(&pool)
        .await
        .expect("favorite/watchlist kind code");
        assert_eq!(kind_code, "PERSONA_MUTATION");
    }
}
