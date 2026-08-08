use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::documents::core::models::NewDocumentImport;
use makosh_hub_backend::domains::documents::core::store::DocumentImportStore;
use makosh_hub_backend::domains::projects::core::models::NewProject;
use makosh_hub_backend::domains::projects::core::store::ProjectStore;

use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "projects-api-test-token";

#[tokio::test]
async fn projects_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/projects"))
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
async fn project_detail_returns_live_project_payload() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let project_id = format!("project:v1:api:{suffix}");
    ProjectStore::new(pool.clone())
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("API Project {suffix}"),
                "Product Development",
                "API project detail test",
                "Alex Morgan",
                vec![format!("ApiProject{suffix}")],
            )
            .progress(64),
        )
        .await
        .expect("upsert API project");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/projects/{}",
                urlencoding_percent_encode(&project_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert_eq!(body["project"]["project_id"], json!(project_id));
    assert_eq!(body["project"]["progress_percent"], json!(64));
    assert_eq!(body["stats"]["message_count"], json!(0));
    assert!(body["timeline"].as_array().expect("timeline").is_empty());

    sqlx::query("DELETE FROM projects WHERE project_id = $1")
        .bind(&project_id)
        .execute(&pool)
        .await
        .expect("cleanup API project");
}

#[tokio::test]
async fn project_link_candidates_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request(
            "/api/v1/projects/project%3Alink-review-placeholder/link-candidates",
        ))
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
async fn project_link_candidates_return_safe_message_and_document_candidates() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let keyword = format!("LinkKeyword{suffix}");
    let project_id = format!("project:v1:link-candidates:{suffix}");

    ProjectStore::new(pool.clone())
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Link Candidates Project {suffix}"),
                "Product Development",
                "Project for link candidates API test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(37),
        )
        .await
        .expect("upsert link candidates project");

    let message_id = seed_message(
        &pool,
        suffix,
        &format!("reviewer-link-{suffix}@example.com"),
        &[format!("owner-link-{suffix}@example.com")],
        &format!("provider-link-candidates-message-{suffix}"),
        &format!("{keyword} message subject"),
        "Message body",
    )
    .await;
    let document_id = seed_document(
        &pool,
        format!("doc_link_candidates_{suffix}"),
        &format!("{keyword} architecture.md"),
        "# Architecture\n\nProject body.",
    )
    .await;

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/projects/{}/link-candidates",
                urlencoding_percent_encode(&project_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);

    let message_candidate = items
        .iter()
        .find(|item| item["target_kind"] == json!("message"))
        .expect("message candidate");
    let document_candidate = items
        .iter()
        .find(|item| item["target_kind"] == json!("document"))
        .expect("document candidate");

    assert_eq!(message_candidate["review_state"], json!("suggested"));
    assert_eq!(message_candidate["target_id"], json!(message_id));
    assert_eq!(message_candidate["project_id"], json!(project_id));
    assert_eq!(document_candidate["review_state"], json!("suggested"));
    assert_eq!(document_candidate["target_id"], json!(document_id));
    assert_eq!(document_candidate["project_id"], json!(project_id));
    assert!(message_candidate["evidence_excerpt"].is_string());
    assert!(document_candidate["evidence_excerpt"].is_string());

    assert_eq!(
        message_candidate["evidence_excerpt"],
        json!(format!("reviewer-link-{suffix}@example.com"))
    );

    let review_items: Vec<(String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT
            review_item_id,
            item_kind,
            metadata->>'mirrored_from',
            metadata->>'target_id'
        FROM review_items
        WHERE metadata->>'project_id' = $1
          AND item_kind = 'project_link_candidate'
        ORDER BY created_at ASC
        "#,
    )
    .bind(&project_id)
    .fetch_all(&pool)
    .await
    .expect("project link candidate review items");
    assert_eq!(review_items.len(), 2);
    assert!(
        review_items
            .iter()
            .all(|item| item.1 == "project_link_candidate")
    );
    assert!(
        review_items
            .iter()
            .all(|item| item.2 == "project_link_candidates")
    );
    assert!(review_items.iter().any(|item| item.3 == message_id));
    assert!(review_items.iter().any(|item| item.3 == document_id));
}

#[tokio::test]
async fn put_project_link_review_updates_review_state() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let project_id = format!("project:v1:link-review-api:{suffix}");
    ProjectStore::new(pool.clone())
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Link Review API {suffix}"),
                "Product Development",
                "Project for link review API test",
                "Alex Morgan",
                vec![format!("LinkReview{suffix}")],
            )
            .progress(66),
        )
        .await
        .expect("upsert link review project");

    let message_id = seed_message(
        &pool,
        suffix,
        &format!("reviewer-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-link-review-api-{suffix}"),
        &format!("LinkReview{suffix} Message"),
        "Link review body",
    )
    .await;

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let command_id = format!("link-review-confirm-{suffix}");
    let response = app
        .oneshot(json_put_request_with_token(
            &format!(
                "/api/v1/projects/{}/link-reviews",
                urlencoding_percent_encode(&project_id)
            ),
            json!({
                "command_id": command_id,
                "target_kind": "message",
                "target_id": message_id,
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;

    assert_eq!(
        body,
        json!({
            "project_id": project_id,
            "target_kind": "message",
            "target_id": message_id,
            "review_state": "user_confirmed",
            "event_id": format!("project_link_review:{command_id}"),
        })
    );

    let persisted_state: String = sqlx::query_scalar(
        "SELECT review_state FROM project_link_reviews WHERE project_id = $1 AND target_kind = 'message' AND target_id = $2",
    )
    .bind(&project_id)
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("review state");
    assert_eq!(persisted_state, "user_confirmed");

    let review_transition_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'projects'
           AND entity_kind = 'project_link_review'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'",
    )
    .bind(format!("project_link_review:{command_id}"))
    .fetch_one(&pool)
    .await
    .expect("project link review observation link count");
    assert_eq!(review_transition_link_count, 1);

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT status, target_entity_kind, target_entity_id
        FROM review_items
        WHERE metadata->>'project_id' = $1
          AND metadata->>'target_kind' = 'message'
          AND metadata->>'target_id' = $2
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&project_id)
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("project link review item");
    assert_eq!(review_item.0, "promoted");
    assert_eq!(review_item.1, "project_link_candidate");
    assert_eq!(review_item.2, format!("{project_id}:message:{message_id}"));
}

#[tokio::test]
async fn put_project_link_review_rejects_missing_target() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let project_id = format!("project:v1:link-review-missing-target:{suffix}");
    ProjectStore::new(pool)
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Link Review Missing Target {suffix}"),
                "Product Development",
                "Project for missing target review API test",
                "Alex Morgan",
                vec![format!("MissingTarget{suffix}")],
            )
            .progress(50),
        )
        .await
        .expect("upsert missing-target project");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(json_put_request_with_token(
            &format!(
                "/api/v1/projects/{}/link-reviews",
                urlencoding_percent_encode(&project_id)
            ),
            json!({
                "command_id": format!("link-review-missing-target-{suffix}"),
                "target_kind": "message",
                "target_id": format!("message_missing_{suffix}"),
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "project_link_target_not_found",
            "message": "project link target was not found"
        })
    );
}

#[derive(Clone)]
struct ProjectsApiContext {
    communication_store: CommunicationIngestionStore,
    message_store: MessageProjectionStore,
    document_store: DocumentImportStore,
}

fn live_projects_api_context(pool: &PgPool) -> ProjectsApiContext {
    ProjectsApiContext {
        communication_store: CommunicationIngestionStore::new(pool.clone()),
        message_store: MessageProjectionStore::new(pool.clone()),
        document_store: DocumentImportStore::new(pool.clone()),
    }
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

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn json_put_request_with_token(uri: &str, value: serde_json::Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn seed_message(
    pool: &PgPool,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let context = live_projects_api_context(pool);
    let account_id = format!("acct_link_review_{suffix}");
    context
        .communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Project Link API Gmail",
            format!("link-review-api-{suffix}@example.com"),
        ))
        .await
        .expect("store link review provider account");

    let raw_record_id = format!("raw_link_review_api_{suffix}_{provider_record_id}");
    let raw = context
        .communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:link-review-api:{suffix}:{provider_record_id}"),
                format!("batch-link-review-api_{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(chrono::Utc::now())
            .provenance(json!({"source":"projects_api_test"})),
        )
        .await
        .expect("record link review raw message");

    project_raw_email_message(&context.message_store, &raw)
        .await
        .expect("project link review message")
        .message_id
}

async fn seed_document(pool: &PgPool, document_id: String, title: &str, markdown: &str) -> String {
    live_projects_api_context(pool)
        .document_store
        .import_document(&NewDocumentImport::markdown(
            document_id.clone(),
            title,
            markdown,
        ))
        .await
        .expect("project link review document");

    document_id
}

fn urlencoding_percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
