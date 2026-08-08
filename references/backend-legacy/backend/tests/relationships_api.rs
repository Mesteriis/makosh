use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::relationships::{
    models::{
        NewRelationship, NewRelationshipEvidence, Relationship, RelationshipEvidenceSourceKind,
        RelationshipReviewState,
    },
    store::RelationshipStore,
};
use makosh_hub_backend::platform::graph::{GraphNodeKind, node_id};
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "relationships-api-test-token";

#[tokio::test]
async fn relationships_list_returns_entity_scoped_relationships() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_persona_relationship(&pool, suffix).await;
    let source_entity_id = &stored.source_entity_id;

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/relationships?entity_kind=persona&entity_id={source_entity_id}&limit=10"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let item = items
        .iter()
        .find(|item| item["relationship_id"] == json!(stored.relationship_id))
        .expect("seeded relationship");

    assert_eq!(item["source_entity_kind"], "persona");
    assert_eq!(item["source_entity_id"], stored.source_entity_id);
    assert_eq!(item["target_entity_kind"], "persona");
    assert_eq!(item["target_entity_id"], stored.target_entity_id);
    assert_eq!(item["relationship_type"], "collaborates_with");
    assert_eq!(item["review_state"], "suggested");
}

#[tokio::test]
async fn relationships_list_returns_global_suggested_review_items() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let suggested = seed_persona_relationship_with_state(
        &pool,
        suffix,
        "global_review_suggested",
        RelationshipReviewState::Suggested,
    )
    .await;
    let confirmed = seed_persona_relationship_with_state(
        &pool,
        suffix + 1,
        "global_review_confirmed",
        RelationshipReviewState::UserConfirmed,
    )
    .await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/relationships?review_state=suggested&limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(
        items
            .iter()
            .any(|item| item["relationship_id"] == json!(suggested.relationship_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["relationship_id"] != json!(confirmed.relationship_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["review_state"] == json!("suggested"))
    );
}

#[tokio::test]
async fn put_relationship_review_updates_relationship_and_graph_projection() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_persona_relationship(&pool, suffix).await;
    let relationship_id = path_segment(&stored.relationship_id);

    let response = app
        .oneshot(json_put_request(
            &format!("/api/v1/relationships/{relationship_id}/review"),
            json!({
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["relationship_id"], stored.relationship_id);
    assert_eq!(body["review_state"], "user_confirmed");
    assert_eq!(body["trust_score"], json!(0.72));
    assert_eq!(body["strength_score"], json!(0.66));

    let stored_review_state: String =
        sqlx::query_scalar("SELECT review_state FROM relationships WHERE relationship_id = $1")
            .bind(&stored.relationship_id)
            .fetch_one(&pool)
            .await
            .expect("relationship review state");
    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'relationships'
           AND entity_kind = 'relationship'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&stored.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: Value = link_row.try_get("metadata").expect("link metadata");
    let graph_row = sqlx::query(
        r#"
        SELECT review_state, properties
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND valid_to IS NULL
        "#,
    )
    .bind(node_id(GraphNodeKind::Persona, &stored.source_entity_id))
    .bind(node_id(GraphNodeKind::Persona, &stored.target_entity_id))
    .fetch_one(&pool)
    .await
    .expect("relationship graph edge");
    let graph_review_state: String = graph_row.try_get("review_state").expect("graph review");
    let graph_properties: Value = graph_row.try_get("properties").expect("graph properties");

    assert_eq!(stored_review_state, "user_confirmed");
    assert_eq!(metadata["review_state"], "user_confirmed");
    assert_eq!(graph_review_state, "user_confirmed");
    assert_eq!(
        graph_properties["relationship_id"],
        json!(stored.relationship_id)
    );

    let observation_row =
        sqlx::query("SELECT origin_kind, payload FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("relationship observation");
    let origin_kind: String = observation_row.try_get("origin_kind").expect("origin kind");
    let payload: Value = observation_row.try_get("payload").expect("payload");
    assert_eq!(origin_kind, "manual");
    assert_eq!(payload["relationship_id"], json!(stored.relationship_id));
    assert_eq!(payload["review_state"], "user_confirmed");

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT status, target_entity_kind, target_entity_id
        FROM review_items
        WHERE metadata->>'relationship_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship review item");
    assert_eq!(review_item.0, "promoted");
    assert_eq!(review_item.1, "relationship");
    assert_eq!(review_item.2, stored.relationship_id);
}

async fn app_and_pool(database_url: &str) -> (axum::Router, PgPool) {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url,
        ),
        database,
    );

    (app, pool)
}

async fn seed_persona_relationship(pool: &PgPool, suffix: u128) -> Relationship {
    seed_persona_relationship_with_state(
        pool,
        suffix,
        "collaborates_with",
        RelationshipReviewState::Suggested,
    )
    .await
}

async fn seed_persona_relationship_with_state(
    pool: &PgPool,
    suffix: u128,
    relationship_type: &str,
    review_state: RelationshipReviewState,
) -> Relationship {
    let person_store = PersonaProjectionStore::new(pool.clone());
    let source = person_store
        .upsert_email_persona(&format!("relationship-api-source-{suffix}@example.com"))
        .await
        .expect("source persona");
    let target = person_store
        .upsert_email_persona(&format!("relationship-api-target-{suffix}@example.com"))
        .await
        .expect("target persona");
    let relationship = NewRelationship::between_personas(
        &source.persona_id,
        &target.persona_id,
        relationship_type,
        0.72,
        0.66,
        0.88,
        review_state,
    )
    .metadata(json!({"source": "relationships_api_test"}));
    let evidence = NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Communication,
        format!("message:relationship-api:{suffix}"),
    )
    .excerpt("They agreed to collaborate on the relationship API.")
    .metadata(json!({"source": "relationships_api_test"}));

    RelationshipStore::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await
        .expect("seed relationship")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
