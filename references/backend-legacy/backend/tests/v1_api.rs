use axum::body::{Body, to_bytes};
use axum::http::{HeaderValue, Method, Request, StatusCode, header};
use chrono::Utc;
use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use makosh_hub_backend::domains::communications::storage::models::{
    CommunicationAttachmentDisposition, NewCommunicationAttachment, NewCommunicationBlob,
};
use makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore;

use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "test-token";

#[tokio::test]
async fn v1_status_returns_enabled_surfaces_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/status")
                .header("x-makosh-secret", HeaderValue::from_static("test-token"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");
    assert_eq!(value["version"], json!("1.0"));
    assert_eq!(value["surfaces"]["messages"], json!(true));
    assert_eq!(value["surfaces"]["personas"], json!(true));
    assert_eq!(value["surfaces"]["search"], json!(true));
    assert_eq!(value["surfaces"]["documents"], json!(true));
    assert_eq!(value["surfaces"]["account_setup"], json!(true));
}

#[tokio::test]
async fn v1_communications_message_detail_returns_attachment_metadata_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("acct_v1_communications_{suffix}");
    let provider_record_id = format!("v1-communications-message-{suffix}");
    let raw_record_id = format!("raw-v1-communications-{suffix}");
    let subject = format!("V1 communications API subject {suffix}");

    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let mail_store = CommunicationStorageStore::new(pool.clone());
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Icloud,
            "V1 Communications API iCloud",
            format!("v1-communications-{suffix}@example.invalid"),
        ))
        .await
        .expect("provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:raw-v1-communications-{suffix}"),
                format!("batch-v1-communications-{suffix}"),
                json!({
                    "subject": subject,
                    "from": "sender@example.invalid",
                    "to": ["recipient@example.invalid"],
                    "body_text": "The attachment metadata must be visible without reading the blob."
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source": "v1_communications_api_test"})),
        )
        .await
        .expect("raw record");
    let message = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    let blob_root = tempfile::tempdir().expect("blob root");
    let local_blob_store = LocalCommunicationBlobStore::new(blob_root.path());
    let local_blob = local_blob_store
        .put_blob(b"attachment bytes")
        .await
        .expect("write attachment blob");
    let blob = mail_store
        .upsert_blob(&NewCommunicationBlob::from_local_blob(&local_blob).content_type("text/plain"))
        .await
        .expect("blob metadata");
    mail_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message.message_id,
                &raw.raw_record_id,
                &blob.blob_id,
                "part-1",
                "text/plain",
                local_blob.size_bytes,
                &blob.sha256,
            )
            .filename("notes.txt")
            .disposition(CommunicationAttachmentDisposition::Attachment),
        )
        .await
        .expect("attachment metadata");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let list_response = app
        .clone()
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/communications/messages?limit=100",
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = json_body(list_response).await;
    let list_item = list_body["items"]
        .as_array()
        .expect("items array")
        .iter()
        .find(|item| item["message_id"] == message.message_id)
        .expect("seeded message in list");
    assert_eq!(list_item["subject"], json!(subject));
    assert_eq!(list_item["attachment_count"], json!(1));

    let detail_response = app
        .oneshot(get_request_with_token_and_actor(
            &format!("/api/v1/communications/messages/{}", message.message_id),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("detail response");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = json_body(detail_response).await;
    assert_eq!(
        detail_body["message"]["message_id"],
        json!(message.message_id)
    );
    assert_eq!(
        detail_body["message"]["body_text"],
        json!(message.body_text)
    );
    assert_eq!(
        detail_body["attachments"][0]["filename"],
        json!("notes.txt")
    );
    assert_eq!(
        detail_body["attachments"][0]["content_type"],
        json!("text/plain")
    );
    assert_eq!(
        detail_body["attachments"][0]["scan_status"],
        json!("not_scanned")
    );
    assert_eq!(
        detail_body["attachments"][0]["storage_kind"],
        json!("local_fs")
    );
    assert_eq!(
        detail_body["attachments"][0]["storage_path"],
        json!(local_blob.storage_path)
    );
}

#[tokio::test]
async fn v1_status_rejects_missing_local_api_secret_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/status"))
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
async fn v1_status_accepts_local_frontend_cors_preflight_before_auth() {
    let app = build_router(config_with_api_token());

    for origin in [
        "http://127.0.0.1:5174",
        "http://localhost:5173",
        "http://tauri.localhost",
        "tauri://localhost",
    ] {
        assert_local_cors_preflight(&app, origin, "GET", "/api/v1/status", "x-makosh-secret").await;
        assert_local_cors_preflight(
            &app,
            origin,
            "PATCH",
            "/api/v1/communications/messages/message-1",
            "x-makosh-secret",
        )
        .await;
        assert_local_cors_preflight(
            &app,
            origin,
            "POST",
            "/makosh.communications.v1.CommunicationsService/ListMessages",
            "connect-protocol-version,content-type,x-makosh-secret",
        )
        .await;
        assert_local_cors_preflight(
            &app,
            origin,
            "GET",
            "/api/realtime/v2/events?after_position=5",
            "last-event-id,x-makosh-secret",
        )
        .await;
    }
}

async fn assert_local_cors_preflight(
    app: &axum::Router,
    origin: &'static str,
    request_method: &'static str,
    uri: &'static str,
    request_headers: &'static str,
) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri(uri)
                .header(header::ORIGIN, HeaderValue::from_static(origin))
                .header(
                    header::ACCESS_CONTROL_REQUEST_METHOD,
                    HeaderValue::from_static(request_method),
                )
                .header(
                    header::ACCESS_CONTROL_REQUEST_HEADERS,
                    HeaderValue::from_static(request_headers),
                )
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(&HeaderValue::from_static(origin))
    );
    let allow_headers = response
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_HEADERS)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    for requested_header in request_headers.split(',') {
        assert!(
            allow_headers.contains(requested_header),
            "missing {requested_header} in {allow_headers}"
        );
    }
}

#[tokio::test]
async fn v1_status_rejects_invalid_local_api_secret_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/status",
            "wrong-token",
            "makosh-frontend",
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
async fn v1_status_accepts_secret_without_actor_header_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_without_actor(
            "/api/v1/status",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn v1_status_ignores_actor_header_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/status",
            LOCAL_API_TOKEN,
            "invalid actor",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn v1_status_returns_service_unavailable_after_auth_when_database_is_not_configured() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/status",
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
    assert!(body["message"].is_string());
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

fn get_request_with_token_without_actor(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token_and_actor(uri: &str, token: &str, _actor_id: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
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
