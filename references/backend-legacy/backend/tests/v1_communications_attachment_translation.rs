use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::ai::control_center::{
    models::{AiModelAvailabilityUpdateRequest, AiModelRouteUpdateRequest},
    store::AiControlCenterStore,
};
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::attachment_text_extraction::AttachmentTextExtractionService;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use makosh_hub_backend::domains::communications::storage::models::{
    CommunicationAttachmentDisposition, NewCommunicationAttachment, NewCommunicationBlob,
};
use makosh_hub_backend::domains::communications::storage::scanner::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus,
};
use makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore;
use makosh_hub_backend::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::settings::store::ApplicationSettingsStore;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "v1comms-attachment-translation-test-token";

#[tokio::test]
async fn v1_attachment_translation_uses_provided_extracted_text_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message_with_attachment(context.pool().clone()).await;
    extract_attachment_text(context.pool().clone(), &seeded.attachment_id).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/attachments/{}/translate",
                seeded.attachment_id
            ),
            json!({
                "target_language": "en",
                "source_text": "Hola equipo, adjunto el contrato para revisión."
            }),
        ))
        .await
        .expect("translation response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["filename"], "contrato.txt");
    assert_eq!(body["original_language"], "es");
    assert_eq!(body["translated"], false);
    assert_eq!(body["target"], "en");
    assert_eq!(body["text"], Value::Null);
    assert_eq!(body["model"], Value::Null);
    assert_eq!(body["reason"], "no LLM configured");
    assert_eq!(body["source"], "durable_extracted_text");
}

#[tokio::test]
async fn v1_attachment_translation_emits_signal_hub_ai_events_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message_with_attachment(context.pool().clone()).await;
    extract_attachment_text(context.pool().clone(), &seeded.attachment_id).await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(context.pool(), &ollama_base_url).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/attachments/{}/translate",
                seeded.attachment_id
            ),
            json!({
                "target_language": "en",
                "source_text": "Hola equipo, adjunto el contrato para revisión."
            }),
        ))
        .await
        .expect("translation response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["translated"], true);

    let signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.ai.attachment_translation.observed',
            'signal.accepted.ai.attachment_translation'
        )
          AND subject->>'attachment_id' = $1
        "#,
    )
    .bind(&seeded.attachment_id)
    .fetch_one(context.pool())
    .await
    .expect("attachment translation signal count");
    assert_eq!(signal_count, 2);
}

#[tokio::test]
async fn v1_attachment_translation_requires_durable_extracted_text_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message_with_attachment(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/attachments/{}/translate",
                seeded.attachment_id
            ),
            json!({ "target_language": "en", "source_text": "untrusted caller text" }),
        ))
        .await
        .expect("translation response");

    assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
}

#[tokio::test]
async fn v1_attachment_translation_rejects_quarantined_attachment_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message_with_attachment_scan_status(
        context.pool().clone(),
        AttachmentSafetyScanStatus::NotScanned,
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/attachments/{}/translate",
                seeded.attachment_id
            ),
            json!({
                "target_language": "en",
                "source_text": "Hola equipo, adjunto el contrato para revisión."
            }),
        ))
        .await
        .expect("translation response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn v1_attachment_translation_blocks_external_ai_without_extracted_text_egress_permission() {
    let context = TestContext::new().await;
    let seeded = seed_message_with_attachment(context.pool().clone()).await;
    extract_attachment_text(context.pool().clone(), &seeded.attachment_id).await;
    ApplicationSettingsStore::new(context.pool().clone())
        .update_setting_value("ai.provider", &json!("omniroute"), "makosh-frontend")
        .await
        .expect("configure external AI runtime");
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/attachments/{}/translate",
                seeded.attachment_id
            ),
            json!({
                "target_language": "en",
                "source_text": "Hola equipo, adjunto el contrato para revisión."
            }),
        ))
        .await
        .expect("translation response");

    assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
    let body = response_json(response).await;
    assert_eq!(body["error"], "failed_precondition");
    assert_eq!(
        body["message"],
        "external AI extracted_text egress is disabled for this mail account"
    );
}

struct SeededAttachment {
    attachment_id: String,
    message_id: String,
}

async fn extract_attachment_text(pool: sqlx::PgPool, attachment_id: &str) {
    AttachmentTextExtractionService::new(
        pool,
        LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT),
    )
    .extract(attachment_id)
    .await
    .expect("extract attachment text");
}

async fn seed_message_with_attachment(pool: sqlx::PgPool) -> SeededAttachment {
    seed_message_with_attachment_scan_status(pool, AttachmentSafetyScanStatus::Clean).await
}

async fn seed_message_with_attachment_scan_status(
    pool: sqlx::PgPool,
    scan_status: AttachmentSafetyScanStatus,
) -> SeededAttachment {
    let suffix = uid();
    let account_id = format!("acct-attachment-translation-{suffix}");
    let provider_record_id = format!("provider-attachment-translation-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let storage_store = CommunicationStorageStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Attachment Translation Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            &account_id,
            "email_message",
            &provider_record_id,
            format!("sha256:{:0>64}", "c"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": "Contrato",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Please review the attached contract."
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;
    let local_blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let local_blob = local_blob_store
        .put_blob(b"Hola equipo, adjunto el contrato para revision.")
        .await
        .expect("write attachment blob");
    let blob = storage_store
        .upsert_blob(&NewCommunicationBlob::from_local_blob(&local_blob).content_type("text/plain"))
        .await
        .expect("store blob");
    let attachment = storage_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message_id,
                &raw.raw_record_id,
                blob.blob_id,
                "part-contrato",
                "text/plain",
                local_blob.size_bytes,
                local_blob.sha256,
            )
            .filename("contrato.txt")
            .disposition(CommunicationAttachmentDisposition::Attachment)
            .scan_report(AttachmentSafetyScanReport {
                status: scan_status,
                engine: None,
                checked_at: None,
                summary: None,
                metadata: json!({}),
            }),
        )
        .await
        .expect("store attachment");

    SeededAttachment {
        attachment_id: attachment.attachment_id,
        message_id,
    }
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url,
        ),
        database,
    )
}

fn post(uri: &str, value: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn spawn_fake_ollama() -> String {
    let app = axum::Router::new()
        .route(
            "/api/version",
            axum::routing::get(|| async { axum::Json(json!({ "version": "0.17.4" })) }),
        )
        .route(
            "/api/tags",
            axum::routing::get(|| async {
                axum::Json(json!({
                    "models": [
                        { "name": "qwen3:4b" },
                        { "name": "qwen3-embedding:4b" }
                    ]
                }))
            }),
        )
        .route(
            "/api/chat",
            axum::routing::post(|axum::Json(_body): axum::Json<Value>| async move {
                axum::Json(json!({
                    "model": "qwen3:4b",
                    "message": { "role": "assistant", "content": "Translated content from fake Ollama." },
                    "done": true,
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 16u32,
                    "eval_count": 8u32
                }))
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fake ollama");
    });

    format!("http://{address}")
}

async fn configure_fake_ollama_setting(pool: &sqlx::PgPool, ollama_base_url: &str) {
    ApplicationSettingsStore::new(pool.clone())
        .update_setting_value(
            "ai.ollama_base_url",
            &json!(ollama_base_url),
            "makosh-frontend",
        )
        .await
        .expect("fake Ollama setting");

    let store = AiControlCenterStore::new(pool.clone());
    let provider_id = "provider:built_in:ollama";
    let chat_model = "qwen3:4b";
    let embedding_model = "qwen3-embedding:4b";

    store
        .update_model_availability(
            &AiModelAvailabilityUpdateRequest {
                provider_id: provider_id.to_owned(),
                model_key: chat_model.to_owned(),
                is_available: true,
            },
            "makosh-frontend",
        )
        .await
        .expect("fake Ollama chat model availability");

    store
        .update_model_availability(
            &AiModelAvailabilityUpdateRequest {
                provider_id: provider_id.to_owned(),
                model_key: embedding_model.to_owned(),
                is_available: true,
            },
            "makosh-frontend",
        )
        .await
        .expect("fake Ollama embedding model availability");

    for slot in [
        "default_chat",
        "reasoning",
        "summarization",
        "mail_intelligence",
        "reply_draft",
        "extraction",
        "meeting_prep",
    ] {
        store
            .put_model_route(
                slot,
                &AiModelRouteUpdateRequest {
                    provider_id: provider_id.to_owned(),
                    model_key: chat_model.to_owned(),
                },
            )
            .await
            .expect("fake Ollama model route");
    }

    store
        .put_model_route(
            "embeddings",
            &AiModelRouteUpdateRequest {
                provider_id: provider_id.to_owned(),
                model_key: embedding_model.to_owned(),
            },
        )
        .await
        .expect("fake Ollama embedding route");
}
