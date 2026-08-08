use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use tokio::net::TcpListener;

use makosh_hub_backend::integrations::ollama::client::{
    OllamaClient, config::OllamaClientConfig, error::OllamaError,
};

#[tokio::test]
async fn ollama_client_round_trips_chat_embed_tags_and_version() {
    let base_url = spawn_fake_ollama(FakeOllamaMode::Ok).await;
    let client = OllamaClient::new(
        OllamaClientConfig::new(base_url, "qwen3:4b", "qwen3-embedding:4b").with_timeout_seconds(5),
    )
    .expect("client");

    let version = client.version().await.expect("version");
    assert_eq!(version, "0.17.4");

    let tags = client.tags().await.expect("tags");
    assert!(tags.contains(&"qwen3:4b".to_owned()));
    assert!(tags.contains(&"qwen3-embedding:4b".to_owned()));

    let chat = client
        .chat("Return exactly: makosh-ai-ok")
        .await
        .expect("chat");
    assert_eq!(chat.content, "makosh-ai-ok");
    assert_eq!(chat.model, "qwen3:4b");

    let embedding = client
        .embed("Макошь memory retrieval")
        .await
        .expect("embed");
    assert_eq!(embedding.model, "qwen3-embedding:4b");
    assert_eq!(embedding.embedding.len(), 2560);
}

#[tokio::test]
async fn ollama_client_strips_qwen_thinking_blocks_from_chat_content() {
    let base_url = spawn_fake_ollama(FakeOllamaMode::ThinkingContent).await;
    let client = OllamaClient::new(
        OllamaClientConfig::new(base_url, "qwen3:4b", "qwen3-embedding:4b").with_timeout_seconds(5),
    )
    .expect("client");

    let chat = client.chat("answer from sources").await.expect("chat");
    assert_eq!(chat.content, "Final cited answer.");
}

#[tokio::test]
async fn ollama_client_uses_json_mode_only_for_structured_chat() {
    let base_url = spawn_fake_ollama(FakeOllamaMode::JsonFormatRequired).await;
    let client = OllamaClient::new(
        OllamaClientConfig::new(base_url, "qwen3:4b", "qwen3-embedding:4b").with_timeout_seconds(5),
    )
    .expect("client");

    let chat = client
        .chat_json_with_model("Return structured evidence", "qwen3:4b")
        .await
        .expect("structured chat");

    assert_eq!(chat.content, "makosh-ai-ok");
}

#[tokio::test]
async fn ollama_client_reports_missing_models_and_malformed_json() {
    let missing_url = spawn_fake_ollama(FakeOllamaMode::MissingModels).await;
    let client = OllamaClient::new(
        OllamaClientConfig::new(missing_url, "qwen3:4b", "qwen3-embedding:4b")
            .with_timeout_seconds(5),
    )
    .expect("client");
    let error = client
        .validate_required_models()
        .await
        .expect_err("missing models");
    assert!(matches!(error, OllamaError::MissingModel { .. }));

    let malformed_url = spawn_fake_ollama(FakeOllamaMode::MalformedJson).await;
    let client = OllamaClient::new(
        OllamaClientConfig::new(malformed_url, "qwen3:4b", "qwen3-embedding:4b")
            .with_timeout_seconds(5),
    )
    .expect("client");
    let error = client.chat("hello").await.expect_err("malformed response");
    assert!(matches!(error, OllamaError::Protocol(_)));
}

#[derive(Clone, Copy)]
enum FakeOllamaMode {
    Ok,
    ThinkingContent,
    JsonFormatRequired,
    MissingModels,
    MalformedJson,
}

async fn spawn_fake_ollama(mode: FakeOllamaMode) -> String {
    let app = Router::new()
        .route(
            "/api/version",
            get(|| async { Json(json!({ "version": "0.17.4" })) }),
        )
        .route(
            "/api/tags",
            get(move || async move {
                let models = match mode {
                    FakeOllamaMode::MissingModels => vec![json!({ "name": "llama3.2:3b" })],
                    _ => vec![
                        json!({ "name": "qwen3:4b" }),
                        json!({ "name": "qwen3-embedding:4b" }),
                    ],
                };
                Json(json!({ "models": models }))
            }),
        )
        .route(
            "/api/chat",
            post(move |Json(body): Json<Value>| async move {
                match mode {
                    FakeOllamaMode::JsonFormatRequired if body["format"] != "json" => (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": "JSON response format is required" })),
                    ),
                    FakeOllamaMode::MalformedJson => (
                        StatusCode::OK,
                        Json(json!({ "model": "qwen3:4b", "message": {} })),
                    ),
                    FakeOllamaMode::ThinkingContent => (
                        StatusCode::OK,
                        Json(json!({
                            "model": "qwen3:4b",
                            "message": {
                                "role": "assistant",
                                "content": "<think>private chain of thought</think>\nFinal cited answer."
                            },
                            "done": true,
                            "total_duration": 10_000_000u64,
                            "prompt_eval_count": 8u32,
                            "eval_count": 3u32
                        })),
                    ),
                    _ => (
                        StatusCode::OK,
                        Json(json!({
                            "model": "qwen3:4b",
                            "message": { "role": "assistant", "content": "makosh-ai-ok" },
                            "done": true,
                            "total_duration": 10_000_000u64,
                            "prompt_eval_count": 8u32,
                            "eval_count": 3u32
                        })),
                    ),
                }
            }),
        )
        .route(
            "/api/embed",
            post(move |Json(_body): Json<Value>| async move {
                let embedding = vec![0.001_f32; 2560];
                Json(json!({
                    "model": "qwen3-embedding:4b",
                    "embeddings": [embedding],
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 4u32
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
