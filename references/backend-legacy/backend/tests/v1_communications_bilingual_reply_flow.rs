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
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::settings::store::ApplicationSettingsStore;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "v1comms-bilingual-reply-flow-test-token";

#[tokio::test]
async fn v1_bilingual_reply_flow_returns_review_contract_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/messages/{}/bilingual-reply-flow",
                seeded.message_id
            ),
            json!({
                "reply_text_ru": "Спасибо, мы проверим контракт сегодня.",
                "tone": "business"
            }),
        ))
        .await
        .expect("bilingual reply flow response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["subject"], "Re: Contrato");
    assert_eq!(body["tone"], "business");
    assert_eq!(body["reply_language"], "ru");
    assert_eq!(body["send_ready"], false);
    assert_eq!(body["original"]["language"], "es");
    assert!(
        body["original"]["text"]
            .as_str()
            .expect("original text")
            .contains("Hola equipo")
    );
    assert_eq!(body["translation"]["target"], "ru");
    assert_eq!(body["translation"]["translated"], false);
    assert_eq!(body["translation"]["text"], Value::Null);
    assert_eq!(body["translation"]["model"], Value::Null);
    assert_eq!(body["translation"]["reason"], "no LLM configured");
    assert_eq!(body["reply"]["language"], "ru");
    assert_eq!(body["reply"]["tone"], "business");
    assert_eq!(
        body["reply"]["text"],
        "Спасибо, мы проверим контракт сегодня."
    );
    assert_eq!(body["back_translation"]["target"], "es");
    assert_eq!(body["back_translation"]["translated"], false);
    assert_eq!(body["back_translation"]["text"], Value::Null);
    assert_eq!(body["back_translation"]["reason"], "no LLM configured");
}

#[tokio::test]
async fn v1_bilingual_reply_flow_rejects_unsupported_tone_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/messages/{}/bilingual-reply-flow",
                seeded.message_id
            ),
            json!({
                "reply_text_ru": "Спасибо.",
                "tone": "casual"
            }),
        ))
        .await
        .expect("bilingual reply flow rejection response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"], "invalid_communication_query");
    assert_eq!(body["message"], "unsupported bilingual reply tone");
}

#[tokio::test]
async fn v1_bilingual_reply_flow_emits_signal_hub_ai_events_when_runtime_runs() {
    let context = TestContext::new().await;
    let seeded = seed_message(context.pool().clone()).await;
    let ollama_base_url = spawn_fake_ollama().await;
    configure_fake_ollama_setting(context.pool(), &ollama_base_url).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/messages/{}/bilingual-reply-flow",
                seeded.message_id
            ),
            json!({
                "reply_text_ru": "Спасибо, мы проверим контракт сегодня.",
                "tone": "business"
            }),
        ))
        .await
        .expect("bilingual reply flow response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["send_ready"], true);
    assert_eq!(body["translation"]["translated"], true);
    assert_eq!(body["back_translation"]["translated"], true);

    let signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.ai.bilingual_reply_inbound_translation.observed',
            'signal.accepted.ai.bilingual_reply_inbound_translation',
            'signal.raw.ai.bilingual_reply_back_translation.observed',
            'signal.accepted.ai.bilingual_reply_back_translation'
        )
          AND subject->>'message_id' = $1
        "#,
    )
    .bind(&seeded.message_id)
    .fetch_one(context.pool())
    .await
    .expect("bilingual reply flow signal count");
    assert_eq!(signal_count, 4);
}

struct SeededMessage {
    message_id: String,
}

async fn seed_message(pool: sqlx::PgPool) -> SeededMessage {
    let suffix = uid();
    let account_id = format!("acct-bilingual-reply-flow-{suffix}");
    let provider_record_id = format!("provider-bilingual-reply-flow-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Bilingual Reply Flow Gmail",
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
            format!("sha256:{:0>64}", "e"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": "Contrato",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Hola equipo, gracias por enviar el contrato. Saludos."
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;

    SeededMessage { message_id }
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
            axum::routing::post(|axum::Json(body): axum::Json<Value>| async move {
                let text = body["messages"]
                    .as_array()
                    .and_then(|messages| messages.last())
                    .and_then(|message| message["content"].as_str())
                    .unwrap_or_default();
                let content = if text.contains("Translate the following text to ru") {
                    "Спасибо, вот перевод входящего письма."
                } else {
                    "Thanks, we will review the contract today."
                };
                axum::Json(json!({
                    "model": "qwen3:4b",
                    "message": { "role": "assistant", "content": content },
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
