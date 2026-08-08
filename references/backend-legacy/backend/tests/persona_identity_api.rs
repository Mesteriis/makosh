use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_events_postgres::consumers::EventConsumerConfig;
use makosh_events_postgres::consumers::EventConsumerRunner;
use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::personas::identity::store::PersonaIdentityReviewStore;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::review_inbox::{
    PERSONA_IDENTITY_REVIEW_INBOX_CONSUMER, project_persona_identity_review_event,
};

const LOCAL_API_TOKEN: &str = "person-identity-api-test-token";

#[tokio::test]
async fn identity_candidates_reject_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/identity-candidates"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn identity_candidates_returns_safe_candidate_payload() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let context = PersonaIdentityApiContext {
        person_store: PersonaProjectionStore::new(pool.clone()),
    };
    let shared_name = format!("Identity Api Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_persona(&format!("left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_persona(&format!("right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_personas(&pool, &left.persona_id, &right.persona_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonaIdentityReviewStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let candidate_id = identity_candidate_id_from_personas(&left.persona_id, &right.persona_id);
    promote_identity_candidate(&pool, &candidate_id)
        .await
        .expect("promote candidate");
    run_persona_identity_review_inbox_consumer(pool.clone()).await;

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/identity-candidates?limit=100",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(!items.is_empty());

    let item = items
        .iter()
        .find(|value| value["identity_candidate_id"] == json!(candidate_id))
        .expect("candidate payload");

    assert_eq!(item["candidate_kind"], "merge_personas");
    assert_eq!(item["review_state"], "suggested");
    assert_eq!(item["left_persona_id"], json!(left.persona_id));
    assert_eq!(item["right_persona_id"], json!(right.persona_id));
    assert!(item.get("left_person_id").is_none());
    assert!(item.get("right_person_id").is_none());
    assert!(item["evidence_summary"].is_string());
    assert!(item["confidence"].is_number());

    let review_item: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT
            review_item.review_item_id,
            review_item.item_kind,
            review_item.metadata->>'mirrored_from',
            review_item.metadata->>'identity_candidate_id'
        FROM review_items review_item
        WHERE review_item.metadata->>'identity_candidate_id' = $1
        ORDER BY review_item.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&candidate_id)
    .fetch_one(&pool)
    .await
    .expect("identity candidate review item");
    assert_eq!(review_item.1, "identity_candidate");
    assert_eq!(review_item.2, "identity_candidates");
    assert_eq!(review_item.3, candidate_id);

    let observation_kind: String = sqlx::query_scalar(
        r#"
        SELECT kind.code AS kind_code
        FROM review_item_evidence evidence
        JOIN observations observation
          ON observation.observation_id = evidence.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE evidence.review_item_id = $1
        ORDER BY evidence.created_at ASC
        LIMIT 1
        "#,
    )
    .bind(&review_item.0)
    .fetch_one(&pool)
    .await
    .expect("identity candidate review evidence observation kind");
    assert_eq!(observation_kind, "PERSONA_IDENTITY_CANDIDATE");
}

#[tokio::test]
async fn identity_candidates_returns_split_candidate_for_confirmed_merge() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let person_store = PersonaProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Api Split {suffix}");

    let left = person_store
        .upsert_email_persona(&format!("split-left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = person_store
        .upsert_email_persona(&format!("split-right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_personas(&pool, &left.persona_id, &right.persona_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonaIdentityReviewStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let merge_candidate_id =
        identity_candidate_id_from_personas(&left.persona_id, &right.persona_id);
    let command_id = format!("identity-api-split-confirm-{suffix}");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_put_request_with_actor(
            &format!("/api/v1/identity-candidates/{merge_candidate_id}/review"),
            json!({
                "command_id": command_id,
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    run_persona_identity_review_inbox_consumer(pool.clone()).await;

    let split_candidate_id =
        split_identity_candidate_id_from_personas(&left.persona_id, &right.persona_id);
    promote_identity_candidate(&pool, &split_candidate_id)
        .await
        .expect("promote split candidate");

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/identity-candidates?limit=100",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let split_item = items
        .iter()
        .find(|value| value["identity_candidate_id"] == json!(split_candidate_id))
        .expect("split candidate payload");

    assert_eq!(split_item["candidate_kind"], "split_persona");
    assert_eq!(split_item["review_state"], "suggested");
    let evidence_summary = split_item["evidence_summary"]
        .as_str()
        .expect("evidence summary");
    assert!(evidence_summary.starts_with("Previously confirmed merge can be split:"));
    assert!(evidence_summary.contains(&left.persona_id));
    assert!(evidence_summary.contains(&right.persona_id));
}

#[tokio::test]
async fn put_identity_candidate_review_confirms_candidate() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let person_store = PersonaProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Review Api {suffix}");

    let left = person_store
        .upsert_email_persona(&format!("review-left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = person_store
        .upsert_email_persona(&format!("review-right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_personas(&pool, &left.persona_id, &right.persona_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonaIdentityReviewStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let identity_candidate_id =
        identity_candidate_id_from_personas(&left.persona_id, &right.persona_id);
    let command_id = format!("identity-api-confirm-{suffix}");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(json_put_request_with_actor(
            &format!("/api/v1/identity-candidates/{identity_candidate_id}/review"),
            json!({
                "command_id": command_id,
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    run_persona_identity_review_inbox_consumer(pool.clone()).await;
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "identity_candidate_id": identity_candidate_id,
            "review_state": "user_confirmed",
            "event_id": format!("persona_identity_review:{command_id}"),
        })
    );

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT status, target_entity_kind, target_entity_id
        FROM review_items
        WHERE metadata->>'identity_candidate_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&identity_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("identity candidate review item");
    assert_eq!(review_item.0, "promoted");
    assert_eq!(review_item.1, "identity_candidate");
    assert_eq!(review_item.2, identity_candidate_id);
}

async fn run_persona_identity_review_inbox_consumer(pool: PgPool) {
    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig::new(PERSONA_IDENTITY_REVIEW_INBOX_CONSUMER),
    );

    for _ in 0..10 {
        let handler_pool = pool.clone();
        let report = runner
            .process_next_batch(|event| {
                project_persona_identity_review_event(handler_pool.clone(), event)
            })
            .await
            .expect("persona identity review inbox consumer");
        if report.processed == 0 {
            break;
        }
    }
}

#[tokio::test]
async fn persona_identity_returns_confirmed_links_for_person() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let person_store = PersonaProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Detail Api {suffix}");

    let left = person_store
        .upsert_email_persona(&format!("detail-left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = person_store
        .upsert_email_persona(&format!("detail-right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_personas(&pool, &left.persona_id, &right.persona_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonaIdentityReviewStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let identity_candidate_id =
        identity_candidate_id_from_personas(&left.persona_id, &right.persona_id);

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_put_request_with_actor(
            &format!("/api/v1/identity-candidates/{identity_candidate_id}/review"),
            json!({
                "command_id": format!("identity-detail-confirm-{suffix}"),
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/personas/{}/identity", left.persona_id),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(!items.is_empty());
    let item = items
        .iter()
        .find(|value| value["identity_candidate_id"] == json!(identity_candidate_id))
        .expect("confirmed identity candidate");
    assert_eq!(item["review_state"], "user_confirmed");
    assert_eq!(item["left_persona_id"], json!(left.persona_id));
    assert_eq!(item["right_persona_id"], json!(right.persona_id));
    assert!(item.get("left_person_id").is_none());
    assert!(item.get("right_person_id").is_none());
}

#[tokio::test]
async fn persona_identity_manual_create_paths_capture_observations_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let person_store = PersonaProjectionStore::new(pool.clone());
    let person = person_store
        .upsert_email_persona(&format!("identity-write-{suffix}@example.com"))
        .await
        .expect("upsert person");
    let encoded_person_id = urlencoding_percent_encode(&person.persona_id);

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let trace_response = app
        .clone()
        .oneshot(json_post_request_with_token(
            "/api/v1/identity-traces",
            json!({
                "identity_type": "telegram",
                "identity_value": format!("trace-{suffix}"),
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("trace response");
    assert_eq!(trace_response.status(), StatusCode::OK);
    let trace_id = json_body(trace_response).await["id"]
        .as_str()
        .expect("trace id")
        .to_owned();
    let trace_source: String =
        sqlx::query_scalar("SELECT source FROM persona_identities WHERE id::text = $1")
            .bind(&trace_id)
            .fetch_one(&pool)
            .await
            .expect("trace source");
    assert!(trace_source.starts_with("observation:"));

    let identity_response = app
        .oneshot(json_post_request_with_token(
            &format!("/api/v1/personas/{encoded_person_id}/identities"),
            json!({
                "identity_type": "linkedin",
                "identity_value": format!("person-{suffix}"),
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("identity response");
    assert_eq!(identity_response.status(), StatusCode::OK);
    let identity_id = json_body(identity_response).await["id"]
        .as_str()
        .expect("identity id")
        .to_owned();
    let identity_source: String =
        sqlx::query_scalar("SELECT source FROM persona_identities WHERE id::text = $1")
            .bind(&identity_id)
            .fetch_one(&pool)
            .await
            .expect("identity source");
    assert!(identity_source.starts_with("observation:"));

    for (source, entity_type, entity_id) in [
        (trace_source, "identity_trace", trace_id),
        (identity_source, "identity", identity_id),
    ] {
        let observation_id = source
            .strip_prefix("observation:")
            .expect("observation prefix");
        let row = sqlx::query(
            "SELECT observation_id, origin_kind FROM observations WHERE observation_id = $1",
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

        let link_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM observation_links
             WHERE observation_id = $1
               AND domain = 'personas'
               AND entity_kind = $2
               AND entity_id = $3",
        )
        .bind(observation_id)
        .bind(entity_type)
        .bind(entity_id)
        .fetch_one(&pool)
        .await
        .expect("observation link count");
        assert_eq!(link_count, 1);
    }
}

#[derive(Clone)]
struct PersonaIdentityApiContext {
    person_store: PersonaProjectionStore,
}

fn config_with_api_token() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request_with_actor(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

fn json_post_request_with_token(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
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

async fn seed_normalized_personas(
    pool: &PgPool,
    left_persona_id: &str,
    right_persona_id: &str,
    display_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE personas
        SET display_name = $1
        WHERE persona_id = $2 OR persona_id = $3
        "#,
    )
    .bind(display_name)
    .bind(left_persona_id)
    .bind(right_persona_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn promote_identity_candidate(
    pool: &PgPool,
    identity_candidate_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE persona_identity_candidates
        SET updated_at = clock_timestamp()
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .execute(pool)
    .await
    .map(|result| {
        assert_eq!(
            result.rows_affected(),
            1,
            "identity candidate should exist before list promotion"
        );
    })?;

    Ok(())
}

fn identity_candidate_id_from_personas(left_id: &str, right_id: &str) -> String {
    let (left_persona_id, right_persona_id) = ordered_persona_ids(left_id, right_id);
    format!("identity_candidate:v1:merge_personas:{left_persona_id}:{right_persona_id}")
}

fn split_identity_candidate_id_from_personas(left_id: &str, right_id: &str) -> String {
    let (left_persona_id, right_persona_id) = ordered_persona_ids(left_id, right_id);
    format!("identity_candidate:v1:split_persona:{left_persona_id}:{right_persona_id}")
}

fn ordered_persona_ids(left_id: &str, right_id: &str) -> (String, String) {
    if left_id <= right_id {
        (left_id.to_owned(), right_id.to_owned())
    } else {
        (right_id.to_owned(), left_id.to_owned())
    }
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn urlencoding_percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}
