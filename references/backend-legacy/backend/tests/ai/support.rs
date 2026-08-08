use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::net::SocketAddr;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use axum::body::{Body, to_bytes};
pub(crate) use axum::http::{Request, StatusCode, header};
pub(crate) use axum::routing::{get, post};
pub(crate) use axum::{Json, Router};
pub(crate) use makosh_communications_postgres::store::CommunicationIngestionStore;
pub(crate) use makosh_hub_backend::ai::control_center::models::{
    AiModelAvailabilityUpdateRequest, AiModelRouteUpdateRequest,
};
pub(crate) use makosh_hub_backend::ai::control_center::store::AiControlCenterStore;
pub(crate) use makosh_hub_backend::ai::core::runs::{AiAgentRun, AiRunStore};
pub(crate) use makosh_hub_backend::ai::core::semantic::models::{
    NewSemanticEmbedding, SemanticSourceKind,
};
pub(crate) use makosh_hub_backend::ai::core::semantic::store::SemanticEmbeddingStore;
pub(crate) use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
pub(crate) use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
pub(crate) use makosh_hub_backend::domains::documents::core::models::NewDocumentImport;
pub(crate) use makosh_hub_backend::domains::documents::core::store::DocumentImportStore;
pub(crate) use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
pub(crate) use makosh_hub_backend::domains::projects::core::models::NewProject;
pub(crate) use makosh_hub_backend::domains::projects::core::store::ProjectStore;

pub(crate) use makosh_hub_backend::platform::config::app_config::AppConfig;
pub(crate) use makosh_hub_backend::platform::settings::store::ApplicationSettingsStore;
pub(crate) use makosh_hub_backend::platform::storage::database::Database;
pub(crate) use serde_json::{Value, json};
pub(crate) use sqlx::Row;
pub(crate) use sqlx::postgres::PgPool;
pub(crate) use tokio::net::TcpListener;
pub(crate) use tower::ServiceExt;

pub(crate) const LOCAL_API_TOKEN: &str = "ai-api-test-token";
pub(crate) static AI_RUNTIME_TEST_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

pub(crate) async fn spawn_fake_ollama() -> String {
    let app = Router::new()
        .route(
            "/api/version",
            get(|| async { Json(json!({ "version": "0.17.4" })) }),
        )
        .route(
            "/api/tags",
            get(|| async {
                Json(json!({
                    "models": [
                        { "name": "qwen3:4b" },
                        { "name": "qwen3-embedding:4b" }
                    ]
                }))
            }),
        )
        .route(
            "/api/embed",
            post(|Json(_body): Json<Value>| async {
                Json(json!({
                    "model": "qwen3-embedding:4b",
                    "embeddings": [unit_embedding(0)],
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 8u32
                }))
            }),
        )
        .route(
            "/api/chat",
            post(|Json(body): Json<Value>| async move {
                let text = body["messages"]
                    .as_array()
                    .and_then(|messages| messages.last())
                    .and_then(|message| message["content"].as_str())
                    .unwrap_or_default();
                let content = if text.contains("Return JSON task candidates") {
                    r#"[{"source_kind":"message","source_id":"__first__","title":"Review the V3 implementation checklist","evidence_excerpt":"Please review the V3 implementation checklist.","confidence":0.82}]"#
                } else if text.contains("meeting briefing") {
                    "Discuss V3 risks and validation evidence."
                } else {
                    "Макошь V3 is source-backed."
                };

                Json(json!({
                    "model": "qwen3:4b",
                    "message": { "role": "assistant", "content": content },
                    "done": true,
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 16u32,
                    "eval_count": 8u32
                }))
            }),
        );

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("listener");
    let address = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fake ollama");
    });

    format!("http://{address}")
}

pub(crate) async fn configure_fake_ollama_setting(pool: &PgPool, ollama_base_url: &str) {
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

pub(crate) fn unit_embedding(active_index: usize) -> Vec<f32> {
    let mut embedding = vec![0.0; 2560];
    embedding[active_index] = 1.0;
    embedding
}

pub(crate) async fn seed_message(
    pool: &PgPool,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let account_id = format!("ai-account-{suffix}");

    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id.clone(),
                CommunicationProviderKind::Imap,
                format!("AI Account {suffix}"),
                format!("ai-external-{suffix}"),
            )
            .config(json!({
                "host": "imap.example.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": format!("ai-{suffix}@example.com")
            })),
        )
        .await
        .expect("provider account");

    let raw_record = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_ai_{suffix}_{provider_record_id}"),
                account_id,
                "email_message",
                provider_record_id,
                format!("fingerprint-ai-{suffix}-{provider_record_id}"),
                format!("batch-ai-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body
                }),
            )
            .provenance(json!({"source":"ai_test"})),
        )
        .await
        .expect("raw record");

    let projected = project_raw_email_message(&message_store, &raw_record)
        .await
        .expect("project raw message");

    projected.message_id
}

pub(crate) async fn seed_document(
    pool: &PgPool,
    fingerprint: &str,
    title: &str,
    text: &str,
) -> String {
    DocumentImportStore::new(pool.clone())
        .import_document(&NewDocumentImport::markdown(fingerprint, title, text))
        .await
        .expect("document")
        .document_id
}

pub(crate) fn config_with_api_token() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

pub(crate) async fn wait_for_run_status(
    pool: &PgPool,
    run_id: &str,
    expected_status: &str,
) -> AiAgentRun {
    for _ in 0..80 {
        if let Some(run) = AiRunStore::new(pool.clone())
            .get_run(run_id)
            .await
            .expect("load run")
            && run.status == expected_status
        {
            return run;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for run {run_id} to reach status {expected_status}");
}

pub(crate) async fn wait_for_event_types(
    pool: &PgPool,
    correlation_id: &str,
    run_id: &str,
    event_types: &[&str],
) -> i64 {
    let expected = i64::try_from(event_types.len()).expect("event type count");
    let event_types = event_types
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();

    for _ in 0..80 {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT count(DISTINCT event_type)::bigint
            FROM event_log
            WHERE correlation_id = $1
              AND subject->>'run_id' = $2
              AND event_type = ANY($3)
            "#,
        )
        .bind(correlation_id)
        .bind(run_id)
        .bind(&event_types)
        .fetch_one(pool)
        .await
        .expect("event type count");
        if count >= expected {
            return count;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    panic!(
        "timed out waiting for events {:?} for run {run_id} correlation {correlation_id}",
        event_types
    );
}

pub(crate) fn get_request(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn get_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn json_post_request_with_actor(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos()
}
