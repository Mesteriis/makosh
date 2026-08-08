use axum::http::StatusCode;
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::platform::storage::database::Database;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_personas_app, get_request_with_token, json_body,
    post_request_with_token, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn identity_traces_create_list_and_attach_unattached_trace() {
    let Some(database_url) = super::support::live_database_url("identity traces API").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_personas_app(&database_url).await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    let create = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/identity-traces",
            json!({
                "identity_type": "message_participant",
                "identity_value": format!("message:v1:{suffix}:api-unattached"),
                "source": "communication_projection"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create trace response");
    assert_eq!(create.status(), StatusCode::OK);
    let create_body = json_body(create).await;
    assert_eq!(create_body["persona_id"], Value::Null);
    assert!(create_body.get("person_id").is_none());
    assert_eq!(create_body["identity_type"], "message_participant");
    assert_eq!(create_body["source"], "communication_projection");
    let identity_id = create_body["id"].as_str().expect("identity id").to_owned();

    let observation_row = sqlx::query(
        "SELECT kind.code AS kind_code, observation.origin_kind
         FROM observation_links link
         JOIN observations observation ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'personas'
           AND link.entity_kind = 'identity_trace'
           AND link.entity_id = $1
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&identity_id)
    .fetch_one(&pool)
    .await
    .expect("identity trace observation");
    assert_eq!(
        observation_row
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "PERSONA_RECORD_MUTATION"
    );
    assert_eq!(
        observation_row
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );

    let list = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/identity-traces?status=unattached",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("list traces response");
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = json_body(list).await;
    let items = list_body["items"].as_array().expect("items");
    assert!(items.iter().any(|item| item["id"] == identity_id
        && item["persona_id"] == Value::Null
        && item.get("person_id").is_none()
        && item["identity_type"] == "message_participant"));

    let person_store = PersonaProjectionStore::new(pool.clone());
    let person = person_store
        .upsert_email_persona(&format!("identity-trace-api-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let attach = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/identity-traces/{}/assignment",
                urlencoding_percent_encode(&identity_id)
            ),
            // Legacy input remains readable while the response stays Persona-native.
            json!({ "person_id": person.persona_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("attach trace response");
    assert_eq!(attach.status(), StatusCode::OK);
    let attach_body = json_body(attach).await;
    assert_eq!(attach_body["id"], identity_id);
    assert_eq!(attach_body["persona_id"], person.persona_id);
    assert!(attach_body.get("person_id").is_none());
    assert_eq!(attach_body["status"], "active");

    let assignment_observation_row = sqlx::query(
        "SELECT kind.code AS kind_code, observation.origin_kind
         FROM observation_links link
         JOIN observations observation ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'personas'
           AND link.entity_kind = 'identity_trace'
           AND link.entity_id = $1
           AND link.relationship_kind = 'trace_assignment'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&identity_id)
    .fetch_one(&pool)
    .await
    .expect("identity trace assignment observation");
    assert_eq!(
        assignment_observation_row
            .try_get::<String, _>("kind_code")
            .expect("assignment kind code"),
        "PERSONA_RECORD_MUTATION"
    );
    assert_eq!(
        assignment_observation_row
            .try_get::<String, _>("origin_kind")
            .expect("assignment origin kind"),
        "manual"
    );
}
