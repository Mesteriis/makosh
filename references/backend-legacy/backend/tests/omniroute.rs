use std::net::SocketAddr;

use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use makosh_hub_backend::integrations::omniroute::client::{
    OmniRouteClient, config::OmniRouteClientConfig, error::OmniRouteError,
};
use makosh_hub_backend::platform::secrets::models::ResolvedSecret;
use serde_json::{Value, json};
use tokio::net::TcpListener;

#[tokio::test]
async fn omniroute_client_round_trips_openai_compatible_models_chat_and_embeddings() {
    let base_url = spawn_fake_omniroute(FakeOmniRouteMode::Ok).await;
    let client = OmniRouteClient::new(
        OmniRouteClientConfig::new(
            base_url,
            "codex/gpt-5.5",
            "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
            ResolvedSecret::new("test-omniroute-key").expect("secret"),
        )
        .with_timeout_seconds(5),
    )
    .expect("client");

    let models = client.models().await.expect("models");
    assert!(models.contains(&"codex/gpt-5.5".to_owned()));
    assert!(models.contains(&"openai-compatible-chat-ollama-pve/qwen3-embedding:4b".to_owned()));
    client
        .validate_required_models()
        .await
        .expect("required models");

    let chat = client
        .chat("Return exactly: makosh-omniroute-ok")
        .await
        .expect("chat");
    assert_eq!(chat.content, "makosh-omniroute-ok");
    assert_eq!(chat.model, "codex/gpt-5.5");

    let embedding = client
        .embed("Макошь source-backed retrieval")
        .await
        .expect("embedding");
    assert_eq!(
        embedding.model,
        "openai-compatible-chat-ollama-pve/qwen3-embedding:4b"
    );
    assert_eq!(embedding.embedding.len(), 2560);
}

#[tokio::test]
async fn omniroute_client_reports_auth_missing_models_and_malformed_json() {
    let unauthorized_url = spawn_fake_omniroute(FakeOmniRouteMode::Unauthorized).await;
    let client = omniroute_client(unauthorized_url);
    let error = client.models().await.expect_err("unauthorized response");
    assert!(matches!(error, OmniRouteError::Endpoint { status: 401 }));

    let missing_url = spawn_fake_omniroute(FakeOmniRouteMode::MissingModels).await;
    let client = omniroute_client(missing_url);
    let error = client
        .validate_required_models()
        .await
        .expect_err("missing models");
    assert!(matches!(error, OmniRouteError::MissingModel { .. }));

    let malformed_url = spawn_fake_omniroute(FakeOmniRouteMode::MalformedJson).await;
    let client = omniroute_client(malformed_url);
    let error = client.chat("hello").await.expect_err("malformed response");
    assert!(matches!(error, OmniRouteError::Protocol(_)));
}

#[derive(Clone, Copy)]
enum FakeOmniRouteMode {
    Ok,
    Unauthorized,
    MissingModels,
    MalformedJson,
}

fn omniroute_client(base_url: String) -> OmniRouteClient {
    OmniRouteClient::new(
        OmniRouteClientConfig::new(
            base_url,
            "codex/gpt-5.5",
            "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
            ResolvedSecret::new("test-omniroute-key").expect("secret"),
        )
        .with_timeout_seconds(5),
    )
    .expect("client")
}

async fn spawn_fake_omniroute(mode: FakeOmniRouteMode) -> String {
    let app = Router::new()
        .route(
            "/v1/models",
            get(move |headers: HeaderMap| async move {
                if !authorized(&headers) || matches!(mode, FakeOmniRouteMode::Unauthorized) {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "unauthorized"})),
                    );
                }
                let models = match mode {
                    FakeOmniRouteMode::MissingModels => {
                        vec![json!({ "id": "openrouter/openrouter/free" })]
                    }
                    _ => vec![
                        json!({ "id": "codex/gpt-5.5" }),
                        json!({ "id": "openai-compatible-chat-ollama-pve/qwen3-embedding:4b" }),
                    ],
                };
                (
                    StatusCode::OK,
                    Json(json!({ "object": "list", "data": models })),
                )
            }),
        )
        .route(
            "/v1/chat/completions",
            post(
                move |headers: HeaderMap, Json(_body): Json<Value>| async move {
                    if !authorized(&headers) || matches!(mode, FakeOmniRouteMode::Unauthorized) {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"error": "unauthorized"})),
                        );
                    }
                    match mode {
                        FakeOmniRouteMode::MalformedJson => (
                            StatusCode::OK,
                            Json(json!({
                                "id": "chatcmpl_fake",
                                "model": "codex/gpt-5.5",
                                "choices": [{ "message": {} }]
                            })),
                        ),
                        _ => (
                            StatusCode::OK,
                            Json(json!({
                                "id": "chatcmpl_fake",
                                "model": "codex/gpt-5.5",
                                "choices": [
                                    {
                                        "index": 0,
                                        "message": {
                                            "role": "assistant",
                                            "content": "<think>hidden</think>\nmakosh-omniroute-ok"
                                        }
                                    }
                                ]
                            })),
                        ),
                    }
                },
            ),
        )
        .route(
            "/v1/embeddings",
            post(
                move |headers: HeaderMap, Json(_body): Json<Value>| async move {
                    if !authorized(&headers) || matches!(mode, FakeOmniRouteMode::Unauthorized) {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"error": "unauthorized"})),
                        );
                    }
                    (
                        StatusCode::OK,
                        Json(json!({
                            "model": "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
                            "data": [
                                {
                                    "index": 0,
                                    "embedding": vec![0.002_f32; 2560]
                                }
                            ]
                        })),
                    )
                },
            ),
        );

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("listener");
    let address = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fake omniroute");
    });

    format!("http://{address}/v1")
}

fn authorized(headers: &HeaderMap) -> bool {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        == Some("Bearer test-omniroute-key")
}
