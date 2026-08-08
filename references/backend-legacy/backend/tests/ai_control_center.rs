use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{Value, json};
use sqlx::Row;
use std::net::SocketAddr;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::ai::control_center::{
    errors::AiControlCenterError,
    models::{
        AiModelAvailabilityUpdateRequest, AiModelRouteUpdateRequest, AiPromptActivateRequest,
        AiPromptCreateRequest, AiPromptTestRequest, AiPromptVersionCreateRequest,
        AiProviderCommandKind, AiProviderConsentRequest, AiProviderCreateRequest,
        AiProviderPatchRequest,
    },
    store::AiControlCenterStore,
};
use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::secrets::models::{
    NewSecretReference, SecretKind, SecretStoreKind,
};
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::platform::storage::database::Database;
use tokio::net::TcpListener;

const LOCAL_API_TOKEN: &str = "ai-control-center-test-token";

fn cfg() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .header("x-makosh-actor-id", "makosh-frontend")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .header("x-makosh-actor-id", "makosh-frontend")
        .body(Body::empty())
        .expect("request")
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("response body"),
    )
    .expect("json response")
}

fn vault_entropy_events(count: usize) -> Vec<Value> {
    (0..count)
        .map(|index| {
            json!({
                "x": index % 997,
                "y": index % 577,
                "dx": (index % 11) as i64 - 5,
                "dy": (index % 13) as i64 - 6,
                "timestamp_ms": index * 5,
                "velocity": (index % 19) as f64 / 10.0,
                "acceleration": (index % 23) as f64 / 100.0,
                "interval_ms": 5.0
            })
        })
        .collect()
}

async fn spawn_fake_ollama() -> String {
    let app = Router::new().route(
        "/api/pull",
        post(|Json(body): Json<Value>| async move {
            let model = body
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            Json(json!({
                "status": format!("downloaded {model}")
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

async fn spawn_failing_fake_ollama() -> String {
    let app = Router::new().route(
        "/api/pull",
        post(|| async move {
            Json(json!({
                "error": "pull failed"
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

async fn spawn_syncable_fake_ollama() -> String {
    let app = Router::new()
        .route(
            "/api/tags",
            post(|| async move {
                Json(json!({
                    "models": [
                        {
                            "name": "qwen3:4b",
                            "model": "qwen3:4b",
                            "details": {"family": "qwen3"}
                        },
                        {
                            "name": "qwen3-embedding:4b",
                            "model": "qwen3-embedding:4b",
                            "details": {"family": "qwen3"}
                        }
                    ]
                }))
            }),
        )
        .route(
            "/api/tags",
            axum::routing::get(|| async move {
                Json(json!({
                    "models": [
                        {
                            "name": "qwen3:4b",
                            "model": "qwen3:4b",
                            "details": {"family": "qwen3"}
                        },
                        {
                            "name": "qwen3-embedding:4b",
                            "model": "qwen3-embedding:4b",
                            "details": {"family": "qwen3"}
                        }
                    ]
                }))
            }),
        )
        .route(
            "/api/show",
            post(|Json(body): Json<Value>| async move {
                let model = body
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let response = match model {
                    "qwen3-embedding:4b" => json!({
                        "capabilities": ["embedding"],
                        "model_info": {
                            "qwen3.embedding_length": 2560
                        }
                    }),
                    _ => json!({
                        "capabilities": ["completion"],
                        "model_info": {
                            "qwen3.context_length": 32768,
                            "qwen3.embedding_length": 2560
                        }
                    }),
                };
                Json(response)
            }),
        );

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("listener");
    let address = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("syncable fake ollama");
    });

    format!("http://{address}")
}

async fn model_download_event_types(
    ctx: &TestContext,
    provider_id: &str,
    model_key: &str,
) -> Vec<String> {
    let rows = sqlx::query(
        r#"
        SELECT event_type
        FROM event_log
        WHERE subject->>'kind' = 'ai_model_download'
          AND subject->>'provider_id' = $1
          AND subject->>'model_key' = $2
        ORDER BY position ASC
        "#,
    )
    .bind(provider_id)
    .bind(model_key)
    .fetch_all(ctx.pool())
    .await
    .expect("model download events");

    rows.into_iter()
        .map(|row| row.get::<String, _>("event_type"))
        .collect()
}

#[tokio::test]
async fn ai_settings_read_endpoints_exist_without_database() {
    let app = build_router(cfg());

    for path in [
        "/api/v1/ai/settings/overview",
        "/api/v1/ai/providers",
        "/api/v1/ai/models",
        "/api/v1/ai/prompts",
    ] {
        let response = app
            .clone()
            .oneshot(get_request(path))
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE, "{path}");
        let body = response_json(response).await;
        assert_eq!(body["error"], json!("database_not_configured"), "{path}");
    }
}

#[tokio::test]
async fn ai_settings_write_endpoints_exist_without_database() {
    let app = build_router(cfg());

    let requests = [
        json_request(
            Method::POST,
            "/api/v1/ai/providers",
            json!({
                "provider_kind": "api",
                "provider_key": "openai",
                "display_name": "OpenAI",
                "base_url": "https://api.openai.com/v1"
            }),
        ),
        json_request(
            Method::PATCH,
            "/api/v1/ai/providers/provider:missing",
            json!({"enabled": true}),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/providers/provider:missing/test",
            json!({}),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/providers/provider:missing/sync-models",
            json!({}),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/providers/provider:missing/consent",
            json!({"consented": true}),
        ),
        json_request(
            Method::PATCH,
            "/api/v1/ai/models/availability",
            json!({
                "provider_id": "provider:missing",
                "model_key": "model:missing",
                "is_available": true
            }),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/model-downloads",
            json!({
                "provider_id": "provider:missing",
                "model_key": "model:missing"
            }),
        ),
        json_request(
            Method::PUT,
            "/api/v1/ai/model-routes/default_chat",
            json!({
                "provider_id": "provider:missing",
                "model_key": "model:missing"
            }),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/prompts",
            json!({
                "prompt_id": "prompt:test",
                "name": "Test prompt",
                "entity_scope": "global",
                "capability_slot": "default_chat"
            }),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/prompts/prompt:test/versions",
            json!({
                "body_template": "Answer {{query}}",
                "variables": ["query"]
            }),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/prompts/prompt:test/activate",
            json!({"prompt_version_id": "prompt-version:test"}),
        ),
        json_request(
            Method::POST,
            "/api/v1/ai/prompts/prompt:test/test",
            json!({
                "prompt_version_id": "prompt-version:test",
                "provider_id": "provider:missing",
                "model_key": "model:missing",
                "variables": {"query": "hello"}
            }),
        ),
    ];

    for request in requests {
        let response = app.clone().oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = response_json(response).await;
        assert_eq!(body["error"], json!("database_not_configured"));
    }
}

#[tokio::test]
async fn model_availability_toggle_removes_disabled_routes() {
    let ctx = TestContext::new().await;
    let store = AiControlCenterStore::new(ctx.pool().clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:built-in:availability".to_owned()),
            provider_kind: "built_in".to_owned(),
            provider_key: "availability".to_owned(),
            display_name: "Ollama Availability".to_owned(),
            base_url: None,
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: None,
            api_key: None,
        })
        .await
        .expect("provider");

    store
        .put_model_route(
            "default_chat",
            &AiModelRouteUpdateRequest {
                provider_id: provider.provider_id.clone(),
                model_key: "custom/default".to_owned(),
            },
        )
        .await
        .expect("route model");

    let disabled = store
        .update_model_availability(
            &AiModelAvailabilityUpdateRequest {
                provider_id: provider.provider_id.clone(),
                model_key: "custom/default".to_owned(),
                is_available: false,
            },
            "test",
        )
        .await
        .expect("disable model");

    assert!(!disabled.is_available);
    assert!(
        !store
            .model_ready_for_private_context(&provider.provider_id, "custom/default")
            .await
            .expect("ready check")
    );
    assert!(
        store
            .route_for_slot("default_chat")
            .await
            .expect("route lookup")
            .is_none()
    );

    let enabled = store
        .update_model_availability(
            &AiModelAvailabilityUpdateRequest {
                provider_id: provider.provider_id.clone(),
                model_key: "custom/default".to_owned(),
                is_available: true,
            },
            "test",
        )
        .await
        .expect("enable model");

    assert!(enabled.is_available);
}

#[tokio::test]
async fn model_route_delete_leaves_slot_unassigned() {
    let ctx = TestContext::new().await;
    let store = AiControlCenterStore::new(ctx.pool().clone());
    store
        .update_model_availability(
            &AiModelAvailabilityUpdateRequest {
                provider_id: "provider:built_in:ollama".to_owned(),
                model_key: "qwen3:4b".to_owned(),
                is_available: true,
            },
            "test",
        )
        .await
        .expect("enable model");
    store
        .put_model_route(
            "default_chat",
            &AiModelRouteUpdateRequest {
                provider_id: "provider:built_in:ollama".to_owned(),
                model_key: "qwen3:4b".to_owned(),
            },
        )
        .await
        .expect("route model");

    store
        .delete_model_route("default_chat")
        .await
        .expect("delete route");

    assert!(
        store
            .route_for_slot("default_chat")
            .await
            .expect("route lookup")
            .is_none()
    );
}

#[tokio::test]
async fn curated_ollama_models_start_not_downloaded_and_unrouted() {
    let ctx = TestContext::new().await;
    let store = AiControlCenterStore::new(ctx.pool().clone());

    let chat_model = store
        .model("provider:built_in:ollama", "qwen3:4b")
        .await
        .expect("chat model lookup")
        .expect("chat model");
    let embedding_model = store
        .model("provider:built_in:ollama", "qwen3-embedding:4b")
        .await
        .expect("embedding model lookup")
        .expect("embedding model");

    assert!(!chat_model.is_available);
    assert!(!embedding_model.is_available);
    assert!(
        store
            .route_for_slot("default_chat")
            .await
            .expect("default_chat route lookup")
            .is_none()
    );
    assert!(
        store
            .route_for_slot("embeddings")
            .await
            .expect("embeddings route lookup")
            .is_none()
    );
}

#[tokio::test]
async fn ollama_model_sync_preserves_user_availability_state() {
    let ctx = TestContext::new().await;
    let store = AiControlCenterStore::new(ctx.pool().clone());
    let ollama_base_url = spawn_syncable_fake_ollama().await;

    store
        .update_provider(
            "provider:built_in:ollama",
            &AiProviderPatchRequest {
                display_name: None,
                base_url: Some(ollama_base_url),
                config: None,
                enabled: None,
                api_key: None,
            },
        )
        .await
        .expect("patch provider base url");

    store
        .update_model_availability(
            &AiModelAvailabilityUpdateRequest {
                provider_id: "provider:built_in:ollama".to_owned(),
                model_key: "qwen3:4b".to_owned(),
                is_available: true,
            },
            "test",
        )
        .await
        .expect("enable curated chat model");

    let provider = store
        .provider("provider:built_in:ollama")
        .await
        .expect("provider lookup")
        .expect("provider");
    let synced = store
        .sync_ollama_provider_models(&provider, "test")
        .await
        .expect("sync models");

    assert_eq!(synced, 2);
    assert!(
        store
            .model("provider:built_in:ollama", "qwen3:4b")
            .await
            .expect("chat model lookup")
            .expect("chat model")
            .is_available
    );
    assert!(
        !store
            .model("provider:built_in:ollama", "qwen3-embedding:4b")
            .await
            .expect("embedding model lookup")
            .expect("embedding model")
            .is_available
    );
}

#[tokio::test]
async fn ollama_model_download_marks_model_available_without_creating_route() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let ollama_base_url = spawn_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = AiControlCenterStore::new(pool);
    store
        .update_provider(
            "provider:built_in:ollama",
            &AiProviderPatchRequest {
                display_name: None,
                base_url: Some(ollama_base_url),
                config: None,
                enabled: None,
                api_key: None,
            },
        )
        .await
        .expect("patch provider base url");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );
    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/v1/ai/model-downloads",
            json!({
                "provider_id": "provider:built_in:ollama",
                "model_key": "qwen3:4b"
            }),
        ))
        .await
        .expect("response");
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    assert_eq!(body["provider_id"], json!("provider:built_in:ollama"));
    assert_eq!(body["model_key"], json!("qwen3:4b"));
    assert_eq!(body["is_available"], json!(true));

    let route = store
        .route_for_slot("default_chat")
        .await
        .expect("route lookup");
    assert!(route.is_none());

    assert_eq!(
        model_download_event_types(&ctx, "provider:built_in:ollama", "qwen3:4b").await,
        vec![
            "ai.hub.model_download.requested".to_owned(),
            "ai.hub.model_download.completed".to_owned(),
        ]
    );
}

#[tokio::test]
async fn ollama_model_download_failure_appends_failed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let ollama_base_url = spawn_failing_fake_ollama().await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = AiControlCenterStore::new(pool);
    store
        .update_provider(
            "provider:built_in:ollama",
            &AiProviderPatchRequest {
                display_name: None,
                base_url: Some(ollama_base_url),
                config: None,
                enabled: None,
                api_key: None,
            },
        )
        .await
        .expect("patch provider base url");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );
    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/v1/ai/model-downloads",
            json!({
                "provider_id": "provider:built_in:ollama",
                "model_key": "qwen3:4b"
            }),
        ))
        .await
        .expect("response");
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY, "body={body}");
    assert_eq!(
        model_download_event_types(&ctx, "provider:built_in:ollama", "qwen3:4b").await,
        vec![
            "ai.hub.model_download.requested".to_owned(),
            "ai.hub.model_download.failed".to_owned(),
        ]
    );
}

#[tokio::test]
async fn remote_api_provider_models_require_host_vault_secret_before_private_context_use() {
    let ctx = TestContext::new().await;
    let store = AiControlCenterStore::new(ctx.pool().clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:api:openai-readiness".to_owned()),
            provider_kind: "api".to_owned(),
            provider_key: "openai".to_owned(),
            display_name: "OpenAI Readiness".to_owned(),
            base_url: Some("https://api.openai.com/v1".to_owned()),
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: Some(true),
            api_key: Some("sk-not-persisted-by-store".to_owned()),
        })
        .await
        .expect("provider");

    let route_error = store
        .put_model_route(
            "default_chat",
            &AiModelRouteUpdateRequest {
                provider_id: provider.provider_id.clone(),
                model_key: "gpt-5.5".to_owned(),
            },
        )
        .await
        .expect_err("remote route requires host-vault secret binding");
    assert_invalid_request_contains(route_error, "host-vault API key");

    let prompt_error = store
        .test_prompt(
            "prompt:system:global:default_chat",
            &AiPromptTestRequest {
                prompt_version_id: "prompt-version:system:global:default_chat:v1".to_owned(),
                provider_id: provider.provider_id.clone(),
                model_key: "gpt-5.5".to_owned(),
                variables: json!({"query": "hello"}),
                source_refs: Some(vec![]),
                score: None,
                notes: None,
            },
            "makosh-frontend",
        )
        .await
        .expect_err("prompt preview selection requires provider readiness");
    assert_invalid_request_contains(prompt_error, "host-vault API key");
}

#[tokio::test]
async fn remote_api_provider_model_route_accepts_host_vault_secret_binding() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let store = AiControlCenterStore::new(pool.clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:api:openai-ready".to_owned()),
            provider_kind: "api".to_owned(),
            provider_key: "openai".to_owned(),
            display_name: "OpenAI Ready".to_owned(),
            base_url: Some("https://api.openai.com/v1".to_owned()),
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: Some(true),
            api_key: Some("sk-not-persisted-by-store".to_owned()),
        })
        .await
        .expect("provider");
    let secret_ref = format!("secret:ai-provider:{}:api_key", provider.provider_id);
    SecretReferenceStore::new(pool)
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret_ref,
                SecretKind::ApiToken,
                SecretStoreKind::HostVault,
                "AI provider API key",
            )
            .metadata(json!({
                "provider_id": provider.provider_id,
                "secret_purpose": "api_key"
            })),
        )
        .await
        .expect("secret reference");
    store
        .bind_api_key_secret(&provider.provider_id, &secret_ref)
        .await
        .expect("secret binding");

    let route = store
        .put_model_route(
            "default_chat",
            &AiModelRouteUpdateRequest {
                provider_id: provider.provider_id.clone(),
                model_key: "gpt-5.5".to_owned(),
            },
        )
        .await
        .expect("ready remote route");

    assert_eq!(route.provider_id, provider.provider_id);
    assert_eq!(route.model_key, "gpt-5.5");
}

#[tokio::test]
async fn non_api_provider_rejects_api_key_secret_binding() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let store = AiControlCenterStore::new(pool.clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:built-in:ollama-no-secret".to_owned()),
            provider_kind: "built_in".to_owned(),
            provider_key: "ollama-no-secret".to_owned(),
            display_name: "Ollama No Secret".to_owned(),
            base_url: None,
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: None,
            api_key: None,
        })
        .await
        .expect("provider");
    let secret_ref = format!("secret:ai-provider:{}:api_key", provider.provider_id);
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret_ref,
                SecretKind::ApiToken,
                SecretStoreKind::HostVault,
                "AI provider API key",
            )
            .metadata(json!({
                "provider_id": provider.provider_id,
                "secret_purpose": "api_key"
            })),
        )
        .await
        .expect("secret reference");

    let error = store
        .bind_api_key_secret(&provider.provider_id, &secret_ref)
        .await
        .expect_err("non-API providers must not accept API key bindings");
    assert_invalid_request_contains(error, "only be bound to API providers");

    let binding_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM ai_provider_secret_refs WHERE provider_id = $1")
            .bind(&provider.provider_id)
            .fetch_one(&pool)
            .await
            .expect("binding count");
    assert_eq!(binding_count, 0);
}

#[tokio::test]
async fn non_api_provider_consent_mutation_is_rejected() {
    let ctx = TestContext::new().await;
    let store = AiControlCenterStore::new(ctx.pool().clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:built-in:ollama-consent".to_owned()),
            provider_kind: "built_in".to_owned(),
            provider_key: "ollama-consent".to_owned(),
            display_name: "Ollama Consent".to_owned(),
            base_url: None,
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: None,
            api_key: None,
        })
        .await
        .expect("provider");

    let error = store
        .record_consent(
            &provider.provider_id,
            &AiProviderConsentRequest { consented: true },
        )
        .await
        .expect_err("non-API providers do not have remote-context consent");
    assert_invalid_request_contains(error, "only to API providers");

    let provider = store
        .provider(&provider.provider_id)
        .await
        .expect("provider lookup")
        .expect("provider remains");
    assert_eq!(provider.consent_state, "not_required");
}

#[tokio::test]
async fn api_provider_create_with_locked_host_vault_does_not_leave_provider_row() {
    let ctx = TestContext::new().await;
    let vault_home = tempfile::tempdir().expect("vault home");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let vault_home = vault_home.path().to_string_lossy().to_string();
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
        LOCAL_API_TOKEN,
        database_url.as_str(),
    )
    .with_test_pairs([("MAKOSH_VAULT_HOME", vault_home.as_str())])
    .expect("config");
    let app = build_router_with_database(config, database);
    let provider_id = "provider:api:locked-vault-create";

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/v1/ai/providers",
            json!({
                "provider_id": provider_id,
                "provider_kind": "api",
                "provider_key": "locked-vault-create",
                "display_name": "Locked Vault Create",
                "base_url": "https://api.example.invalid/v1",
                "remote_context_consent": true,
                "api_key": "sk-not-persisted"
            }),
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = response_json(response).await;
    assert_eq!(body["error"], json!("host_vault_error"));

    let provider_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM ai_provider_accounts WHERE provider_id = $1)",
    )
    .bind(provider_id)
    .fetch_one(ctx.pool())
    .await
    .expect("provider exists query");
    assert!(!provider_exists);
}

#[tokio::test]
async fn api_provider_create_with_api_key_marks_ready_and_binds_host_vault_secret() {
    let ctx = TestContext::new().await;
    let vault_home = tempfile::tempdir().expect("vault home");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let vault_home = vault_home.path().to_string_lossy().to_string();
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
        LOCAL_API_TOKEN,
        database_url.as_str(),
    )
    .with_test_pairs([("MAKOSH_VAULT_HOME", vault_home.as_str())])
    .expect("config");
    let app = build_router_with_database(config, database);
    let provider_id = "provider:api:omniroute-ready";

    let entropy_response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);
    let create_vault_response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/v1/vault/create",
            json!({}),
        ))
        .await
        .expect("vault create response");
    assert_eq!(create_vault_response.status(), StatusCode::OK);

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/v1/ai/providers",
            json!({
                "provider_id": provider_id,
                "provider_kind": "api",
                "provider_key": "omniroute",
                "display_name": "OmniRoute",
                "base_url": "https://ai.sh-inc.ru/v1",
                "remote_context_consent": true,
                "api_key": "sk-test-provider-secret"
            }),
        ))
        .await
        .expect("provider create response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["provider_id"], json!(provider_id));
    assert_eq!(body["status"], json!("ready"));
    assert_eq!(body["consent_state"], json!("granted"));
    assert!(
        !body.to_string().contains("sk-test-provider-secret"),
        "API response must not echo provider token material"
    );

    let provider_config: Value =
        sqlx::query_scalar("SELECT config FROM ai_provider_accounts WHERE provider_id = $1")
            .bind(provider_id)
            .fetch_one(ctx.pool())
            .await
            .expect("provider config");
    assert_eq!(
        provider_config["base_url"],
        json!("https://ai.sh-inc.ru/v1")
    );
    assert!(
        !provider_config
            .to_string()
            .contains("sk-test-provider-secret"),
        "provider config must stay non-secret"
    );

    let binding = sqlx::query(
        r#"
        SELECT refs.secret_ref, secrets.secret_kind, secrets.store_kind
        FROM ai_provider_secret_refs refs
        JOIN secret_references secrets ON secrets.secret_ref = refs.secret_ref
        WHERE refs.provider_id = $1 AND refs.secret_purpose = 'api_key'
        "#,
    )
    .bind(provider_id)
    .fetch_one(ctx.pool())
    .await
    .expect("host-vault secret binding");

    assert_eq!(
        binding.get::<String, _>("secret_ref"),
        format!("secret:ai-provider:{provider_id}:api_key")
    );
    assert_eq!(binding.get::<String, _>("secret_kind"), "api_token");
    assert_eq!(binding.get::<String, _>("store_kind"), "host_vault");
}

#[tokio::test]
async fn ai_control_center_mutations_record_observation_trail_against_postgres() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let store = AiControlCenterStore::new(pool.clone());
    let provider = store
        .create_provider(&AiProviderCreateRequest {
            provider_id: Some("provider:api:observation-trail".to_owned()),
            provider_kind: "api".to_owned(),
            provider_key: "openai".to_owned(),
            display_name: "OpenAI Trail".to_owned(),
            base_url: Some("https://api.openai.com/v1".to_owned()),
            command_preset: None,
            config: None,
            capabilities: None,
            enabled: Some(true),
            remote_context_consent: Some(false),
            api_key: None,
        })
        .await
        .expect("provider");
    let secret_ref = format!("secret:ai-provider:{}:api_key", provider.provider_id);
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret_ref,
                SecretKind::ApiToken,
                SecretStoreKind::HostVault,
                "AI provider API key",
            )
            .metadata(json!({
                "provider_id": provider.provider_id,
                "secret_purpose": "api_key"
            })),
        )
        .await
        .expect("secret reference");
    store
        .bind_api_key_secret(&provider.provider_id, &secret_ref)
        .await
        .expect("bind secret");
    store
        .update_provider(
            &provider.provider_id,
            &AiProviderPatchRequest {
                display_name: Some("OpenAI Trail Updated".to_owned()),
                base_url: Some("https://api.openai.com/v1".to_owned()),
                config: Some(json!({"region": "us"})),
                enabled: Some(true),
                api_key: None,
            },
        )
        .await
        .expect("update provider");
    store
        .record_consent(
            &provider.provider_id,
            &AiProviderConsentRequest { consented: true },
        )
        .await
        .expect("record consent");
    store
        .put_model_route(
            "default_chat",
            &AiModelRouteUpdateRequest {
                provider_id: provider.provider_id.clone(),
                model_key: "gpt-5.5".to_owned(),
            },
        )
        .await
        .expect("put model route");
    store
        .provider_command(&provider.provider_id, AiProviderCommandKind::SyncModels)
        .await
        .expect("sync models");
    let prompt = store
        .create_prompt(
            &AiPromptCreateRequest {
                prompt_id: Some("prompt:test:trail".to_owned()),
                name: "Trail prompt".to_owned(),
                entity_scope: "global".to_owned(),
                capability_slot: "default_chat".to_owned(),
                description: Some("Prompt observation trail".to_owned()),
                metadata: Some(json!({"team": "core"})),
            },
            "makosh-frontend",
        )
        .await
        .expect("create prompt");
    let version = store
        .create_prompt_version(
            &prompt.prompt_id,
            &AiPromptVersionCreateRequest {
                prompt_version_id: Some("prompt-version:test:trail".to_owned()),
                version_label: Some("v1".to_owned()),
                body_template: "Answer {{query}}".to_owned(),
                variables: vec!["query".to_owned()],
            },
            "makosh-frontend",
        )
        .await
        .expect("create prompt version");
    store
        .activate_prompt_version(
            &prompt.prompt_id,
            &AiPromptActivateRequest {
                prompt_version_id: version.prompt_version_id.clone(),
            },
            "makosh-frontend",
        )
        .await
        .expect("activate prompt version");
    let eval_run = store
        .test_prompt(
            &prompt.prompt_id,
            &AiPromptTestRequest {
                prompt_version_id: version.prompt_version_id.clone(),
                provider_id: provider.provider_id.clone(),
                model_key: "gpt-5.5".to_owned(),
                variables: json!({"query": "hello"}),
                source_refs: Some(vec![json!({"kind": "observation", "id": "obs:test"})]),
                score: Some(5),
                notes: Some("looks good".to_owned()),
            },
            "makosh-frontend",
        )
        .await
        .expect("test prompt");

    let provider_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'provider_account'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&provider.provider_id)
    .fetch_all(&pool)
    .await
    .expect("provider observation rows");
    assert!(
        provider_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROVIDER_ACCOUNT"
                && row.get::<String, _>("relationship_kind") == "create"
        }),
        "provider create observation must exist"
    );
    assert!(
        provider_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROVIDER_ACCOUNT"
                && row.get::<String, _>("relationship_kind") == "update"
                && row.get::<Value, _>("payload")["display_name"] == "OpenAI Trail Updated"
        }),
        "provider update observation must exist"
    );
    assert!(
        provider_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROVIDER_ACCOUNT"
                && row.get::<String, _>("relationship_kind") == "consent_recorded"
                && row.get::<Value, _>("payload")["consent_state"] == "granted"
        }),
        "provider consent observation must exist"
    );
    let model_catalog_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'model_catalog_item'
          AND link.entity_id LIKE $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(format!("{}:%", provider.provider_id))
    .fetch_all(&pool)
    .await
    .expect("model catalog observations");
    assert!(
        model_catalog_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_MODEL_CATALOG_ITEM"
                && row.get::<String, _>("relationship_kind") == "seed"
        }),
        "model catalog seed observation must exist"
    );

    let prompt_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'prompt_template'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&prompt.prompt_id)
    .fetch_all(&pool)
    .await
    .expect("prompt observations");
    assert!(
        prompt_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROMPT_TEMPLATE"
                && row.get::<String, _>("relationship_kind") == "create"
        }),
        "prompt create observation must exist"
    );
    assert!(
        prompt_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROMPT_TEMPLATE"
                && row.get::<String, _>("relationship_kind") == "activate"
                && row.get::<Value, _>("payload")["active_version_id"]
                    == Value::String(version.prompt_version_id.clone())
        }),
        "prompt activate observation must exist"
    );

    let version_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'prompt_template_version'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&version.prompt_version_id)
    .fetch_all(&pool)
    .await
    .expect("prompt version observations");
    assert!(
        version_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROMPT_TEMPLATE_VERSION"
                && row.get::<String, _>("relationship_kind") == "create"
        }),
        "prompt version create observation must exist"
    );
    assert!(
        version_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROMPT_TEMPLATE_VERSION"
                && row.get::<String, _>("relationship_kind") == "activate"
                && row.get::<Value, _>("payload")["status"] == "active"
        }),
        "prompt version activate observation must exist"
    );

    let eval_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'prompt_eval_run'
          AND link.entity_id = $1
        "#,
    )
    .bind(&eval_run.eval_run_id)
    .fetch_all(&pool)
    .await
    .expect("prompt eval observations");
    assert!(
        eval_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "AI_PROMPT_EVAL_RUN"
                && row.get::<String, _>("relationship_kind") == "test"
                && row.get::<Value, _>("payload")["provider_id"]
                    == Value::String(provider.provider_id.clone())
        }),
        "prompt eval observation must exist"
    );

    let binding_id = format!("{}:api_key", provider.provider_id);
    let binding_row = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'provider_secret_binding'
          AND link.entity_id = $1
        ORDER BY observation.captured_at DESC
        LIMIT 1
        "#,
    )
    .bind(&binding_id)
    .fetch_one(&pool)
    .await
    .expect("binding observation row");
    assert_eq!(
        binding_row.get::<String, _>("kind_code"),
        "AI_PROVIDER_SECRET_BINDING"
    );
    assert_eq!(binding_row.get::<String, _>("relationship_kind"), "bind");
    assert_eq!(
        binding_row.get::<Value, _>("payload")["secret_ref"],
        secret_ref
    );

    let route_row = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'ai'
          AND link.entity_kind = 'model_route'
          AND link.entity_id = 'default_chat'
        ORDER BY observation.captured_at DESC
        LIMIT 1
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("route observation row");
    assert_eq!(route_row.get::<String, _>("kind_code"), "AI_MODEL_ROUTE");
    assert_eq!(route_row.get::<String, _>("relationship_kind"), "put");
    assert_eq!(
        route_row.get::<Value, _>("payload")["provider_id"],
        provider.provider_id
    );
    assert_eq!(route_row.get::<Value, _>("payload")["model_key"], "gpt-5.5");
}

#[tokio::test]
async fn ai_prompt_create_canonicalizes_legacy_person_scope_to_persona_against_postgres() {
    let ctx = TestContext::new().await;
    let store = AiControlCenterStore::new(ctx.pool().clone());

    let prompt = store
        .create_prompt(
            &AiPromptCreateRequest {
                prompt_id: None,
                name: "Persona prompt".to_owned(),
                entity_scope: "person".to_owned(),
                capability_slot: "default_chat".to_owned(),
                description: None,
                metadata: Some(json!({})),
            },
            "makosh-frontend",
        )
        .await
        .expect("create prompt with legacy person entity scope");

    assert_eq!(prompt.entity_scope, "persona");
    assert!(
        prompt.prompt_id.starts_with("prompt:user:persona:"),
        "generated prompt id must use persona scope, got {}",
        prompt.prompt_id
    );
}

fn assert_invalid_request_contains(error: AiControlCenterError, expected: &str) {
    match error {
        AiControlCenterError::InvalidRequest(message) => assert!(
            message.contains(expected),
            "expected `{message}` to contain `{expected}`"
        ),
        other => panic!("expected invalid request, got {other:?}"),
    }
}
