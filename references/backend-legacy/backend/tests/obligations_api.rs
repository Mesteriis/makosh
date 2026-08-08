use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::obligations::models::{
    entity_kind::ObligationEntityKind, evidence::NewObligationEvidence, obligation::NewObligation,
    read_model::Obligation, source_kind::ObligationEvidenceSourceKind,
    states::ObligationReviewState,
};
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "obligations-api-test-token";

#[tokio::test]
async fn obligations_list_returns_entity_scoped_obligations() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let obligated_persona_id = format!("person:v1:email:obligation-api-{suffix}@example.com");
    let stored = seed_obligation(&pool, suffix, &obligated_persona_id).await;

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/obligations?entity_kind=persona&entity_id={obligated_persona_id}&limit=10"
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
        .find(|item| item["obligation_id"] == json!(stored.obligation_id))
        .expect("seeded obligation");

    assert_eq!(item["obligated_entity_kind"], "persona");
    assert_eq!(item["obligated_entity_id"], obligated_persona_id);
    assert_eq!(item["statement"], stored.statement);
    assert_eq!(item["review_state"], "suggested");
}

#[tokio::test]
async fn obligations_list_returns_global_suggested_review_items() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let suggested_persona_id =
        format!("person:v1:email:obligation-global-suggested-{suffix}@example.com");
    let confirmed_persona_id =
        format!("person:v1:email:obligation-global-confirmed-{suffix}@example.com");
    let suggested = seed_obligation_with_review_state(
        &pool,
        suffix,
        &suggested_persona_id,
        ObligationReviewState::Suggested,
    )
    .await;
    let confirmed = seed_obligation_with_review_state(
        &pool,
        suffix + 1,
        &confirmed_persona_id,
        ObligationReviewState::UserConfirmed,
    )
    .await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/obligations?review_state=suggested&limit=10",
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
            .any(|item| item["obligation_id"] == json!(suggested.obligation_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["obligation_id"] != json!(confirmed.obligation_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["review_state"] == json!("suggested"))
    );
}

#[tokio::test]
async fn put_obligation_review_updates_review_state_with_observation_trail() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let obligated_persona_id = format!("person:v1:email:obligation-review-{suffix}@example.com");
    let stored = seed_obligation(&pool, suffix, &obligated_persona_id).await;
    let obligation_id = path_segment(&stored.obligation_id);

    let response = app
        .oneshot(json_put_request(
            &format!("/api/v1/obligations/{obligation_id}/review"),
            json!({
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["obligation_id"], stored.obligation_id);
    assert_eq!(body["review_state"], "user_confirmed");

    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM obligations WHERE obligation_id = $1")
            .bind(&stored.obligation_id)
            .fetch_one(&pool)
            .await
            .expect("stored review state");
    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'obligations'
           AND entity_kind = 'obligation'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&stored.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: Value = link_row.try_get("metadata").expect("link metadata");
    let task_link_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM obligation_task_links WHERE obligation_id = $1")
            .bind(&stored.obligation_id)
            .fetch_one(&pool)
            .await
            .expect("task link count");

    assert_eq!(review_state, "user_confirmed");
    assert_eq!(metadata["review_state"], "user_confirmed");
    assert_eq!(task_link_count, 0);

    let observation_row =
        sqlx::query("SELECT origin_kind, payload FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("obligation observation");
    let origin_kind: String = observation_row.try_get("origin_kind").expect("origin kind");
    let payload: Value = observation_row.try_get("payload").expect("payload");
    assert_eq!(origin_kind, "manual");
    assert_eq!(payload["obligation_id"], json!(stored.obligation_id));
    assert_eq!(payload["review_state"], "user_confirmed");

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT status, target_entity_kind, target_entity_id
        FROM review_items
        WHERE metadata->>'obligation_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation review item");
    assert_eq!(review_item.0, "promoted");
    assert_eq!(review_item.1, "obligation");
    assert_eq!(review_item.2, stored.obligation_id);
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

async fn seed_obligation(pool: &PgPool, suffix: u128, obligated_persona_id: &str) -> Obligation {
    seed_obligation_with_review_state(
        pool,
        suffix,
        obligated_persona_id,
        ObligationReviewState::Suggested,
    )
    .await
}

async fn seed_obligation_with_review_state(
    pool: &PgPool,
    suffix: u128,
    obligated_persona_id: &str,
    review_state: ObligationReviewState,
) -> Obligation {
    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        obligated_persona_id,
        format!("Send obligation API evidence package {suffix}"),
        0.82,
        review_state,
    )
    .metadata(json!({"source": "obligations_api_test"}));
    let evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Communication,
        format!("message:obligation-api:{suffix}"),
    )
    .quote("I will send the obligation API evidence package.")
    .confidence(0.9)
    .metadata(json!({"source": "obligations_api_test"}));

    ObligationStore::new(pool.clone())
        .upsert_with_evidence(&obligation, &[evidence])
        .await
        .expect("seed obligation")
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
use makosh_hub_backend::domains::obligations::store::ObligationStore;
