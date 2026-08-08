use axum::http::StatusCode;
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::platform::storage::database::Database;
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_personas_app, build_personas_app_with_database, get_request_with_token,
    json_body, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn personas_list_returns_ok() {
    let Some(database_url) = super::support::live_database_url("personas list").await else {
        return;
    };
    let app = build_personas_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token("/api/v1/personas", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn personas_routes_return_persona_native_schema_against_postgres() {
    let Some(database_url) = super::support::live_database_url("personas route").await else {
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
    let store = PersonaProjectionStore::new(pool);
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_persona(&format!("persona-native-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    store
        .set_owner_persona(&owner.persona_id)
        .await
        .expect("set owner persona");

    let app = build_personas_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/personas?limit=20",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("personas list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items array");
    assert!(
        items
            .iter()
            .any(|item| item["persona_id"] == owner.persona_id && item["is_self"] == true),
        "personas list should include owner Persona: {body}"
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.persona_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona detail response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["persona_id"], owner.persona_id);
    assert_eq!(body["persona_type"], "human");
    assert_eq!(body["is_self"], true);
    assert_eq!(body["identity"]["display_name"], owner.display_name);
    assert_eq!(
        body["identity"]["email_address"],
        owner
            .email_address
            .as_deref()
            .expect("owner fixture has email")
    );
    assert_eq!(
        body["communication"]["primary_email"],
        owner
            .email_address
            .as_deref()
            .expect("owner fixture has email")
    );
    assert!(
        body.get("compatibility").is_none(),
        "native Personas API response must not advertise legacy routes: {body}"
    );
}

#[tokio::test]
async fn personas_put_updates_compatibility_projection_against_postgres() {
    let Some(database_url) = super::support::live_database_url("personas write route").await else {
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
        .upsert_email_persona(&format!("persona-native-write-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    let previous_owner = store
        .upsert_email_persona(&format!("persona-native-write-prev-{suffix}@example.com"))
        .await
        .expect("upsert previous owner persona");
    store
        .set_owner_persona(&previous_owner.persona_id)
        .await
        .expect("set previous owner persona");

    let app = build_personas_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.persona_id)
            ),
            json!({
                "identity": {
                    "display_name": "Owner Persona"
                },
                "is_self": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona update response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["persona_id"], owner.persona_id);
    assert_eq!(body["identity"]["display_name"], "Owner Persona");
    assert_eq!(body["is_self"], true);

    let row = sqlx::query(
        r#"
        SELECT display_name, is_self
        FROM personas
        WHERE persona_id = $1
        "#,
    )
    .bind(&owner.persona_id)
    .fetch_one(&pool)
    .await
    .expect("updated persona row");
    assert_eq!(
        row.try_get::<String, _>("display_name").unwrap(),
        "Owner Persona"
    );
    assert!(row.try_get::<bool, _>("is_self").unwrap());

    let persona_update_observation: (String, String) = sqlx::query_as(
        "SELECT link.observation_id, kind.code AS kind_code
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'personas'
           AND link.entity_kind = 'persona'
           AND link.entity_id = $1
           AND link.relationship_kind = 'persona_update'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&owner.persona_id)
    .fetch_one(&pool)
    .await
    .expect("persona update observation link");
    assert!(!persona_update_observation.0.is_empty());
    assert_eq!(persona_update_observation.1, "PERSONA_MUTATION");

    let previous_is_self: bool =
        sqlx::query_scalar("SELECT is_self FROM personas WHERE persona_id = $1")
            .bind(&previous_owner.persona_id)
            .fetch_one(&pool)
            .await
            .expect("previous owner row");
    assert!(!previous_is_self);

    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.persona_id)
            ),
            json!({ "is_self": false }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona unset owner response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn personas_profile_aliases_cover_owner_and_dossier_against_postgres() {
    let Some(database_url) = super::support::live_database_url("personas profile aliases").await
    else {
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
    let store = PersonaProjectionStore::new(pool);
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_persona(&format!("persona-alias-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");

    let app = build_personas_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            "/api/v1/personas/owner",
            json!({ "persona_id": owner.persona_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona owner update response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(
        body["owner_persona"]["persona_id"], owner.persona_id,
        "unexpected owner update body: {body}"
    );
    assert!(body["owner_persona"].get("person_id").is_none());
    assert_eq!(body["owner_persona"]["is_self"], true);

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/personas/owner",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["owner_persona"]["persona_id"], owner.persona_id);
    assert!(body["owner_persona"].get("person_id").is_none());

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/personas/{}/dossier",
                urlencoding_percent_encode(&owner.persona_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona dossier response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["persona_id"], owner.persona_id);
    assert!(body["dossier_snapshot_id"].as_str().is_some());
}
