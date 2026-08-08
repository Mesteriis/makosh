use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::platform::storage::database::Database;

use super::support::{
    LOCAL_API_TOKEN, build_personas_app_with_database, delete_request_with_token, json_body,
    post_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn persona_identity_post_and_delete() {
    let Some(database_url) =
        super::support::live_database_url("persona identity post and delete").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&format!("identity-delete-{suffix}@example.com"))
        .await
        .expect("upsert person");
    let app = build_personas_app_with_database(&database_url, database);
    let pid = person.persona_id.clone();
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/personas/{}/identities",
                urlencoding_percent_encode(&pid)
            ),
            json!({
                "identity_type": "email",
                "identity_value": format!("test-{suffix}@example.com"),
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "identity post={}",
        response.status()
    );
    let post_body = json_body(response).await;
    let identity_id = post_body["id"].as_str().expect("identity id").to_owned();

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/personas/{}/identities/{}",
                urlencoding_percent_encode(&pid),
                urlencoding_percent_encode(&identity_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "identity delete={}",
        response.status()
    );
    let delete_body = json_body(response).await;
    assert_eq!(delete_body["deleted"], json!(true));

    let delete_observation_row = sqlx::query(
        "SELECT kind.code AS kind_code, observation.origin_kind, link.metadata->>'deleted' AS deleted
         FROM observation_links link
         JOIN observations observation ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'personas'
           AND link.entity_kind = 'identity'
           AND link.entity_id = $1
           AND link.relationship_kind = 'identity_delete'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&identity_id)
    .fetch_one(&pool)
    .await
    .expect("identity delete observation");
    assert_eq!(
        delete_observation_row
            .try_get::<String, _>("kind_code")
            .expect("delete kind code"),
        "PERSONA_RECORD_MUTATION"
    );
    assert_eq!(
        delete_observation_row
            .try_get::<String, _>("origin_kind")
            .expect("delete origin kind"),
        "manual"
    );
    assert_eq!(
        delete_observation_row
            .try_get::<String, _>("deleted")
            .expect("delete metadata"),
        "true"
    );
}

#[tokio::test]
async fn persona_relationship_timeline_entrypoint_captures_observation_against_postgres() {
    let Some(database_url) =
        super::support::live_database_url("person relationship timeline observations").await
    else {
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&format!("relationship-event-{suffix}@example.com"))
        .await
        .expect("upsert person");
    let app = build_personas_app_with_database(&database_url, database);
    let encoded_person_id = urlencoding_percent_encode(&person.persona_id);

    let response = app
        .oneshot(post_request_with_token(
            &format!("/api/v1/personas/{encoded_person_id}/timeline"),
            json!({
                "event_type": "meeting",
                "title": format!("Meeting with {suffix}"),
                "description": "Manual relationship event should be observation-backed.",
                "occurred_at": "2027-01-01T00:00:00Z",
                "source": "manual",
                "related_entity_id": format!("evt:v1:{suffix}"),
                "related_entity_kind": "event"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("timeline response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let event_id = json_body(response).await["id"]
        .as_str()
        .expect("relationship event id")
        .to_owned();

    let source: String =
        sqlx::query_scalar("SELECT source FROM relationship_events WHERE id::text = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("relationship event source");
    assert!(source.starts_with("observation:"));

    let observation_id = source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(observation_id)
            .fetch_one(&pool)
            .await
            .expect("relationship event observation");
    assert_eq!(origin_kind, "manual");
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
    .expect("relationship event observation kind");
    assert_eq!(kind_code, "PERSONA_RECORD_MUTATION");

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'personas'
           AND entity_kind = 'relationship_event'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .expect("relationship event observation link count");
    assert_eq!(link_count, 1);
}
