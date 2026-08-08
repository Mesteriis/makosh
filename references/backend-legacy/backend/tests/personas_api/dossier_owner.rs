use axum::http::StatusCode;
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::platform::storage::database::Database;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_personas_app, build_personas_app_with_database, get_request_with_token,
    json_body, post_request_with_token, put_request_with_token, unique_suffix,
    urlencoding_percent_encode,
};

#[tokio::test]
async fn persona_dossier_get_persists_snapshot_and_review_state_against_postgres() {
    let Some(database_url) = super::support::live_database_url("dossier snapshot API").await else {
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = PersonaProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let persona = store
        .upsert_email_persona(&format!("dossier-snapshot-{suffix}@example.com"))
        .await
        .expect("upsert dossier persona");

    let app = build_personas_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/personas/{}/dossier",
                urlencoding_percent_encode(&persona.persona_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dossier response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let snapshot_id = body["dossier_snapshot_id"]
        .as_str()
        .expect("dossier snapshot id")
        .to_owned();
    assert_eq!(body["review_state"], "suggested");
    assert_eq!(body["persona"]["persona_id"], persona.persona_id);
    assert!(body.get("person").is_none());
    assert!(body["persona"].get("person_id").is_none());

    let row = sqlx::query(
        r#"
        SELECT persona_id, review_state, dossier
        FROM persona_dossier_snapshots
        WHERE dossier_snapshot_id = $1
        "#,
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("stored dossier snapshot");
    assert_eq!(
        row.try_get::<String, _>("persona_id").expect("persona id"),
        persona.persona_id
    );
    assert_eq!(
        row.try_get::<String, _>("review_state")
            .expect("review state"),
        "suggested"
    );
    let stored_dossier = row.try_get::<Value, _>("dossier").expect("dossier json");
    assert_eq!(stored_dossier["persona"]["persona_id"], persona.persona_id);
    assert!(stored_dossier.get("person").is_none());
    assert!(stored_dossier["persona"].get("person_id").is_none());

    let dossier_refresh_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'dossier_snapshot'
           AND entity_id = $1
           AND relationship_kind = 'dossier_refresh'",
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("dossier refresh observation link count");
    assert_eq!(dossier_refresh_link_count, 1);

    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/personas/{}/dossier/review",
                urlencoding_percent_encode(&persona.persona_id)
            ),
            json!({ "review_state": "user_confirmed" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dossier review response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["dossier_snapshot_id"], snapshot_id);
    assert_eq!(body["review_state"], "user_confirmed");
    assert!(body["reviewed_at"].is_string());

    let dossier_review_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'dossier_snapshot'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'",
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("dossier review observation link count");
    assert_eq!(dossier_review_link_count, 1);
}

#[tokio::test]
async fn persona_investigate_captures_observation_and_links_snapshot_against_postgres() {
    let Some(database_url) = super::support::live_database_url("persona investigate API").await
    else {
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = PersonaProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = store
        .upsert_email_persona(&format!("investigate-snapshot-{suffix}@example.com"))
        .await
        .expect("upsert investigate persona");

    let app = build_personas_app_with_database(&database_url, database);

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/personas/{}/investigate",
                urlencoding_percent_encode(&person.persona_id)
            ),
            json!({ "query": "refresh dossier" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("investigate response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let snapshot_id = body["dossier_snapshot_id"]
        .as_str()
        .expect("dossier snapshot id")
        .to_owned();

    let investigation_observation: (String, String) = sqlx::query_as(
        "SELECT link.observation_id, kind.code AS kind_code
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'personas'
           AND link.entity_kind = 'dossier_snapshot'
           AND link.entity_id = $1
           AND link.relationship_kind = 'dossier_refresh'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("investigate observation");
    assert!(!investigation_observation.0.is_empty());
    assert_eq!(investigation_observation.1, "PERSONA_MUTATION");
}

#[tokio::test]
async fn persona_detail_not_found_returns_404() {
    let Some(database_url) = super::support::live_database_url("person detail").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_personas_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/personas/person:nonexistent-{suffix}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn persona_owner_get_and_put_uses_owner_persona_against_postgres() {
    let Some(database_url) = super::support::live_database_url("owner persona API").await else {
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    sqlx::query("UPDATE personas SET is_self = false WHERE is_self = true")
        .execute(&pool)
        .await
        .expect("clear existing owner persona");
    let store = PersonaProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_persona(&format!("owner-api-{suffix}@example.com"))
        .await
        .expect("upsert owner candidate");
    let other = store
        .upsert_email_persona(&format!("not-owner-api-{suffix}@example.com"))
        .await
        .expect("upsert non-owner candidate");

    let app = build_personas_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/personas/owner",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("initial owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert!(body["owner_persona"].is_null());

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            "/api/v1/personas/owner",
            json!({ "persona_id": owner.persona_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("set owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["owner_persona"]["persona_id"], owner.persona_id);
    assert!(body["owner_persona"].get("person_id").is_none());
    assert_eq!(body["owner_persona"]["is_self"], true);
    assert_eq!(body["owner_persona"]["persona_type"], "human");

    let owner_assignment_observation: (String, String) = sqlx::query_as(
        "SELECT link.observation_id, kind.code AS kind_code
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'personas'
           AND link.entity_kind = 'persona'
           AND link.entity_id = $1
           AND link.relationship_kind = 'owner_assignment'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&owner.persona_id)
    .fetch_one(&pool)
    .await
    .expect("owner assignment observation");
    assert!(!owner_assignment_observation.0.is_empty());
    assert_eq!(owner_assignment_observation.1, "PERSONA_MUTATION");

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/personas/owner",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["owner_persona"]["persona_id"], owner.persona_id);
    assert_ne!(body["owner_persona"]["persona_id"], other.persona_id);
    assert!(body["owner_persona"].get("person_id").is_none());
}
