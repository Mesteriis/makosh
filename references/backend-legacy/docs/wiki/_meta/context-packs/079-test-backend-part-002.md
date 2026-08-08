# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `079-test-backend-part-002`
- Group / Группа: `backend`
- Role / Роль: `test`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/backend-tests.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/tests/ai_control_center.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/ai_control_center.rs`
- Size bytes / Размер в байтах: `27552`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::ai::control_center::{
    AiControlCenterError, AiControlCenterStore, AiModelRouteUpdateRequest,
    AiProviderConsentRequest, AiProviderCreateRequest,
};
use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "ai-control-center-test-token";

fn cfg() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
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
            &makosh_hub_backend::ai::control_center::AiPromptTestRequest {
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
    assert_invalid_requ
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/ai_smoke.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/ai_smoke.rs`
- Size bytes / Размер в байтах: `1876`
- Included characters / Включено символов: `1876`
- Truncated / Обрезано: `no`

```rust
use std::env;

use makosh_hub_backend::integrations::ollama::client::{OllamaClient, OllamaClientConfig};

#[tokio::test]
async fn live_ollama_qwen3_runtime_smoke() {
    let Some(base_url) = env::var("MAKOSH_OLLAMA_BASE_URL").ok() else {
        eprintln!("skipping live Ollama smoke test: MAKOSH_OLLAMA_BASE_URL is not set");
        return;
    };
    let chat_model = env::var("MAKOSH_OLLAMA_CHAT_MODEL").unwrap_or_else(|_| "qwen3:4b".to_owned());
    let embed_model =
        env::var("MAKOSH_OLLAMA_EMBED_MODEL").unwrap_or_else(|_| "qwen3-embedding:4b".to_owned());
    let timeout_seconds = env::var("MAKOSH_OLLAMA_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(120);

    let client = OllamaClient::new(
        OllamaClientConfig::new(base_url, chat_model.clone(), embed_model.clone())
            .with_timeout_seconds(timeout_seconds),
    )
    .expect("Ollama client");

    let version = client.version().await.expect("Ollama version");
    assert!(!version.trim().is_empty());

    let tags = client.tags().await.expect("Ollama tags");
    assert!(
        tags.iter().any(|model| model == &chat_model),
        "missing chat model {chat_model}; available models: {tags:?}"
    );
    assert!(
        tags.iter().any(|model| model == &embed_model),
        "missing embedding model {embed_model}; available models: {tags:?}"
    );

    let chat = client
        .chat("Return exactly this token and nothing else: makosh-ai-smoke-ok")
        .await
        .expect("Ollama chat");
    assert!(
        chat.content.contains("makosh-ai-smoke-ok"),
        "unexpected chat response: {}",
        chat.content
    );

    let embedding = client
        .embed("Макошь V3 AI semantic retrieval smoke")
        .await
        .expect("Ollama embed");
    assert_eq!(embedding.embedding.len(), 2560);
}
```

### `backend/tests/calendar.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar.rs`
- Size bytes / Размер в байтах: `350`
- Included characters / Включено символов: `350`
- Truncated / Обрезано: `no`

```rust
#[path = "calendar/account_event.rs"]
mod account_event;
#[path = "calendar/event_context.rs"]
mod event_context;
#[path = "calendar/intelligence_sync.rs"]
mod intelligence_sync;
#[path = "calendar/meeting_outcomes.rs"]
mod meeting_outcomes;
#[path = "calendar/scheduling_rules.rs"]
mod scheduling_rules;
#[path = "calendar/support.rs"]
mod support;
```

### `backend/tests/calendar/account_event.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar/account_event.rs`
- Size bytes / Размер в байтах: `6685`
- Included characters / Включено символов: `6685`
- Truncated / Обрезано: `no`

```rust
use chrono::{Duration, Utc};
use makosh_hub_backend::domains::calendar::events::{
    CalendarAccountStore, CalendarAccountUpdate, CalendarEventListQuery, CalendarEventStore,
    CalendarEventUpdate, CalendarSourceStore, NewCalendarEvent,
};

use super::support::{live_pool, unique_suffix};

#[tokio::test]
async fn calendar_account_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = CalendarAccountStore::new(pool);
    let suffix = unique_suffix();

    let acct = store
        .create("local", &format!("Test Account {suffix}"), None)
        .await
        .expect("create account");
    assert_eq!(acct.provider, "local");
    assert!(acct.account_id.starts_with("cal:v1:"));

    let fetched = store
        .get(&acct.account_id)
        .await
        .expect("get account")
        .expect("account exists");
    assert_eq!(fetched.account_name, acct.account_name);

    let update = CalendarAccountUpdate {
        account_name: Some(format!("Updated {suffix}")),
        ..Default::default()
    };
    let updated = store
        .update(&acct.account_id, &update)
        .await
        .expect("update account");
    assert_eq!(updated.account_name, format!("Updated {suffix}"));

    let list = store.list(Some("local")).await.expect("list accounts");
    assert!(list.iter().any(|a| a.account_id == acct.account_id));

    store
        .delete(&acct.account_id)
        .await
        .expect("delete account");
    assert!(
        store
            .get(&acct.account_id)
            .await
            .expect("get deleted")
            .is_none()
    );
}

#[tokio::test]
async fn calendar_source_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let src_store = CalendarSourceStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Src Test {suffix}"), None)
        .await
        .expect("create account");
    let src = src_store
        .create(
            &acct.account_id,
            "Work Calendar",
            Some("gcal-123"),
            Some("#4285f4"),
            Some("Europe/Madrid"),
        )
        .await
        .expect("create source");
    assert!(src.source_id.starts_with("src:v1:"));
    assert_eq!(src.name, "Work Calendar");
    assert_eq!(src.color.as_deref(), Some("#4285f4"));

    let list = src_store
        .list_by_account(&acct.account_id)
        .await
        .expect("list sources");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Work Calendar");
}

#[tokio::test]
async fn calendar_event_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Event Test {suffix}"), None)
        .await
        .expect("create account");
    let now = Utc::now();
    let req = NewCalendarEvent {
        title: format!("Test Event {suffix}"),
        description: Some("Test description".into()),
        start_at: now,
        end_at: now + Duration::hours(1),
        account_id: Some(acct.account_id.clone()),
        event_type: Some("meeting".into()),
        ..Default::default()
    };

    let event = event_store.create(&req).await.expect("create event");
    assert!(event.event_id.starts_with("evt:v1:"));
    assert!(event.observation_id.starts_with("observation:v1:"));
    assert_eq!(event.title, format!("Test Event {suffix}"));
    assert_eq!(event.status, "scheduled");

    let observation_kind: Option<String> = sqlx::query_scalar(
        r#"
        SELECT kind.code
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&event.observation_id)
    .fetch_optional(&pool)
    .await
    .expect("calendar event observation");
    assert_eq!(observation_kind.as_deref(), Some("CALENDAR_EVENT"));

    let fetched = event_store
        .get(&event.event_id)
        .await
        .expect("get event")
        .expect("event exists");
    assert_eq!(fetched.event_type.as_deref(), Some("meeting"));

    let update = CalendarEventUpdate {
        title: Some(format!("Updated {suffix}")),
        ..Default::default()
    };
    let updated = event_store
        .update(&event.event_id, &update)
        .await
        .expect("update event");
    assert_eq!(updated.title, format!("Updated {suffix}"));

    let list = event_store
        .list(&CalendarEventListQuery {
            from: Some(now - Duration::hours(1)),
            to: Some(now + Duration::hours(2)),
            limit: Some(50),
            ..Default::default()
        })
        .await
        .expect("list events");
    assert!(list.iter().any(|e| e.event_id == event.event_id));

    event_store
        .delete(&event.event_id)
        .await
        .expect("delete event");
    assert!(
        event_store
            .get(&event.event_id)
            .await
            .expect("get deleted")
            .is_none()
    );
}

#[tokio::test]
async fn event_reschedule_and_status_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Status Test {suffix}"), None)
        .await
        .expect("create account");
    let now = Utc::now();
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Reschedule {suffix}"),
            start_at: now,
            end_at: now + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let new_start = now + Duration::hours(2);
    let rescheduled = event_store
        .reschedule(&event.event_id, new_start, new_start + Duration::hours(1))
        .await
        .expect("reschedule");
    assert_eq!(rescheduled.status, "rescheduled");

    event_store
        .set_status(&event.event_id, "cancelled")
        .await
        .expect("cancel");
    let cancelled = event_store
        .get(&event.event_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(cancelled.status, "cancelled");
}
```

### `backend/tests/calendar/event_context.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar/event_context.rs`
- Size bytes / Размер в байтах: `5982`
- Included characters / Включено символов: `5982`
- Truncated / Обрезано: `no`

```rust
use chrono::{Duration, Utc};
use makosh_hub_backend::domains::calendar::core::{
    ContextPackInput, EventAgendaStore, EventChecklistStore, EventContextPackStore,
    EventParticipantStore, EventRelationStore,
};
use makosh_hub_backend::domains::calendar::events::{
    CalendarAccountStore, CalendarEventStore, NewCalendarEvent,
};
use serde_json::json;

use super::support::{live_pool, unique_suffix};

#[tokio::test]
async fn event_participants_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let part_store = EventParticipantStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Part Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Participants {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let participant = part_store
        .add(
            &event.event_id,
            &format!("john-{suffix}@test.com"),
            Some("John"),
            Some("required"),
            None,
            None,
        )
        .await
        .expect("add participant");
    assert_eq!(participant.role, "required");
    assert_eq!(participant.email, format!("john-{suffix}@test.com"));

    let list = part_store
        .list(&event.event_id)
        .await
        .expect("list participants");
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn event_relations_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let rel_store = EventRelationStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Rel Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Relations {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let rel = rel_store
        .link(&event.event_id, "project", "proj-1", "related_to")
        .await
        .expect("link");
    assert_eq!(rel.entity_type, "project");

    let list = rel_store
        .list(&event.event_id)
        .await
        .expect("list relations");
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn event_context_pack_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let ctx_store = EventContextPackStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Ctx Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Context {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let input = ContextPackInput {
        summary: Some("Test summary".into()),
        documents: json!([]),
        tasks: json!([]),
        open_questions: json!(["Q1"]),
        risks: json!(["No agenda"]),
        suggested_agenda: json!([]),
        suggested_actions: json!([]),
        ..Default::default()
    };
    let pack = ctx_store
        .upsert(&event.event_id, &input)
        .await
        .expect("upsert context pack");
    assert_eq!(pack.summary.as_deref(), Some("Test summary"));

    let fetched = ctx_store
        .get(&event.event_id)
        .await
        .expect("get pack")
        .expect("pack exists");
    assert_eq!(fetched.summary.as_deref(), Some("Test summary"));
}

#[tokio::test]
async fn event_agenda_and_checklist_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let agenda_store = EventAgendaStore::new(pool.clone());
    let cl_store = EventChecklistStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Agenda Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Agenda {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");

    let agenda = agenda_store
        .set(&event.event_id, json!(["Item 1", "Item 2"]), "manual")
        .await
        .expect("set agenda");
    assert_eq!(agenda.source, "manual");

    let checklist = cl_store
        .set(
            &event.event_id,
            json!([{"text": "Prepare docs", "done": false}]),
            "manual",
        )
        .await
        .expect("set checklist");
    assert_eq!(checklist.source, "manual");

    let fetched_agenda = agenda_store
        .get(&event.event_id)
        .await
        .expect("get agenda")
        .expect("exists");
    let items = fetched_agenda.items.as_array().expect("array");
    assert_eq!(items.len(), 2);
}
```

### `backend/tests/calendar/intelligence_sync.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar/intelligence_sync.rs`
- Size bytes / Размер в байтах: `5342`
- Included characters / Включено символов: `5342`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::calendar::brain::CalendarBrainService;
use makosh_hub_backend::domains::calendar::core::{
    EventAgendaStore, EventChecklistStore, EventContextPackStore, EventParticipantStore,
    EventRelationStore,
};
use makosh_hub_backend::domains::calendar::events::CalendarEventStore;
use makosh_hub_backend::domains::calendar::health::CalendarWatchtowerService;
use makosh_hub_backend::domains::calendar::intelligence::CalendarIntelligenceService;
use makosh_hub_backend::domains::calendar::rules::CalendarRuleStore;
use makosh_hub_backend::domains::calendar::scheduling::{DeadlineStore, FocusBlockStore};

use super::support::{disconnected_pool, live_pool};

#[test]
fn intelligence_classify_event() {
    assert_eq!(
        CalendarIntelligenceService::classify_event("Weekly sync meeting", 3, 60),
        "meeting"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Tax deadline AEAT", 1, 0),
        "deadline"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Focus: deep work", 1, 120),
        "focus"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Flight to Madrid", 1, 180),
        "travel"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Coffee", 1, 15),
        "personal"
    );
    assert_eq!(
        CalendarIntelligenceService::classify_event("Sprint planning", 4, 90),
        "planning"
    );
}

#[tokio::test]
async fn intelligence_calculate_importance() {
    let base = CalendarIntelligenceService::calculate_importance("Coffee", 1, false, false);
    assert!(base > 0.0 && base < 1.0);

    let urgent = CalendarIntelligenceService::calculate_importance(
        "URGENT: client escalation",
        3,
        true,
        true,
    );
    assert!(urgent > base);
    assert!(urgent <= 1.0);
}

#[tokio::test]
async fn intelligence_calculate_readiness() {
    let full = CalendarIntelligenceService::calculate_readiness(true, true, true, true, true);
    assert_eq!(full, 1.0);

    let none = CalendarIntelligenceService::calculate_readiness(false, false, false, false, false);
    assert_eq!(none, 0.0);

    let partial = CalendarIntelligenceService::calculate_readiness(true, false, true, false, true);
    assert!(partial > 0.0 && partial < 1.0);
}

#[tokio::test]
async fn intelligence_detect_risks() {
    let none = CalendarIntelligenceService::detect_risks(true, true, true, true, false);
    assert!(none.is_empty());

    let missing = CalendarIntelligenceService::detect_risks(false, false, false, false, true);
    assert_eq!(missing.len(), 5);
    assert!(missing.contains(&"No agenda prepared".to_string()));
}

#[tokio::test]
async fn health_services_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };

    let brief = CalendarWatchtowerService::weekly_brief(&pool).await;
    assert!(brief.is_ok());

    let prep = CalendarWatchtowerService::events_needing_preparation(&pool).await;
    assert!(prep.is_ok());

    let no_outcomes = CalendarWatchtowerService::events_without_outcomes(&pool).await;
    assert!(no_outcomes.is_ok());

    let load = CalendarWatchtowerService::meeting_load_analysis(&pool).await;
    assert!(load.is_ok());
}

#[tokio::test]
async fn brain_services_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };

    let overview = CalendarBrainService::answer(&pool, "show me this week").await;
    assert!(overview.is_ok());

    let search = CalendarBrainService::search_events(&pool, "meeting").await;
    assert!(search.is_ok());
}

#[test]
fn sync_ics_export() {
    let ics = makosh_hub_backend::domains::calendar::sync::export_event_ics(
        "Test Meeting",
        Some("Description"),
        Some("Office"),
        "20260101T100000",
        "20260101T110000",
        Some("Europe/Madrid"),
    );
    assert!(ics.contains("BEGIN:VCALENDAR"));
    assert!(ics.contains("SUMMARY:Test Meeting"));
    assert!(ics.contains("DTSTART"));
    assert!(ics.contains("DTEND"));
}

#[test]
fn sync_markdown_export() {
    let md = makosh_hub_backend::domains::calendar::sync::export_event_md(
        "Test Meeting",
        Some("Description"),
        Some("Office"),
        "2026-01-01T10:00:00+01:00",
        "2026-01-01T11:00:00+01:00",
        &["John".into(), "Jane".into()],
    );
    assert!(md.contains("# Test Meeting"));
    assert!(md.contains("John"));
    assert!(md.contains("Jane"));
    assert!(md.contains("Office"));
}

#[tokio::test]
async fn disconnected_pool_creates_store() {
    let pool = disconnected_pool();
    let _store = CalendarEventStore::new(pool);
}

#[tokio::test]
async fn disconnected_pool_core_stores() {
    let pool = disconnected_pool();
    let _p = EventParticipantStore::new(pool.clone());
    let _r = EventRelationStore::new(pool.clone());
    let _c = EventContextPackStore::new(pool.clone());
    let _a = EventAgendaStore::new(pool.clone());
    let _cl = EventChecklistStore::new(pool);
}

#[tokio::test]
async fn disconnected_pool_scheduling_stores() {
    let pool = disconnected_pool();
    let _d = DeadlineStore::new(pool.clone());
    let _f = FocusBlockStore::new(pool);
}

#[tokio::test]
async fn disconnected_pool_rules_store() {
    let _store = CalendarRuleStore::new(disconnected_pool());
}
```

### `backend/tests/calendar/meeting_outcomes.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar/meeting_outcomes.rs`
- Size bytes / Размер в байтах: `10249`
- Included characters / Включено символов: `10249`
- Truncated / Обрезано: `no`

```rust
use chrono::{Duration, Utc};
use makosh_hub_backend::application::CalendarMeetingOutcomeApplicationService;
use makosh_hub_backend::domains::calendar::events::{
    CalendarAccountStore, CalendarEventStore, NewCalendarEvent,
};
use makosh_hub_backend::domains::calendar::meetings::{MeetingNoteStore, MeetingOutcomeStore};
use makosh_hub_backend::domains::decisions::{
    DecisionEntityKind, DecisionReviewState, DecisionStore,
};
use makosh_hub_backend::domains::obligations::{
    ObligationEntityKind, ObligationReviewState, ObligationStore,
};

use super::support::{live_pool, unique_suffix};

#[tokio::test]
async fn meeting_notes_and_outcomes_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let note_store = MeetingNoteStore::new(pool.clone());
    let outcome_store = MeetingOutcomeStore::new(pool);
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Meeting Test {suffix}"), None)
        .await
        .expect("create");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Meeting {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            ..Default::default()
        })
        .await
        .expect("create event");
    assert!(event.observation_id.starts_with("observation:v1:"));

    let note = note_store
        .create(
            &event.event_id,
            "# Meeting Notes\n\nDiscussed scope.",
            Some("markdown"),
            Some("manual"),
        )
        .await
        .expect("create note");
    assert!(note.content.contains("Meeting Notes"));

    let outcome = outcome_store
        .add(
            &event.event_id,
            "decision",
            "Use Rust",
            Some("Decided to use Rust for backend"),
            None,
            None,
            Some("manual"),
        )
        .await
        .expect("add outcome");
    assert_eq!(outcome.outcome_type, "decision");
    assert_eq!(outcome.source, "manual");

    let notes = note_store.list(&event.event_id).await.expect("list notes");
    assert_eq!(notes.len(), 1);

    let outcomes = outcome_store
        .list(&event.event_id)
        .await
        .expect("list outcomes");
    assert_eq!(outcomes.len(), 1);
}

#[tokio::test]
async fn meeting_outcome_decision_creates_suggested_decision_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let outcome_service = CalendarMeetingOutcomeApplicationService::new(pool.clone());
    let decision_store = DecisionStore::new(pool.clone());
    let suffix = unique_suffix();

    let acct = acct_store
        .create("local", &format!("Decision Outcome Test {suffix}"), None)
        .await
        .expect("create account");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Decision outcome meeting {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            event_type: Some("meeting".into()),
            ..Default::default()
        })
        .await
        .expect("create event");

    let outcome = outcome_service
        .add_manual(
            &event.event_id,
            "decision",
            &format!("Adopt meeting outcome adapter {suffix}"),
            Some("We decided to persist meeting decisions as reviewable domain Decisions."),
            None,
            None,
        )
        .await
        .expect("add decision outcome");
    let linked_decision_id = outcome
        .linked_entity_id
        .as_deref()
        .expect("decision outcome should link to suggested Decision");

    let decisions = decision_store
        .list_for_entity(DecisionEntityKind::Event, &event.event_id, 10)
        .await
        .expect("event decisions");
    let decision = decisions
        .iter()
        .find(|item| item.decision_id == linked_decision_id)
        .expect("suggested Decision linked to meeting outcome");

    assert_eq!(decision.title, outcome.title);
    assert_eq!(decision.review_state, DecisionReviewState::Suggested);
    assert_eq!(
        decision.rationale,
        "We decided to persist meeting decisions as reviewable domain Decisions."
    );

    let evidence: (String, String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, observation_id, quote FROM decision_evidence WHERE decision_id = $1",
    )
    .bind(linked_decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision evidence");
    assert_eq!(evidence.0, "event");
    assert_eq!(evidence.1, event.event_id);
    assert_eq!(evidence.2.as_deref(), Some(event.observation_id.as_str()));
    assert_eq!(
        evidence.3.as_deref(),
        Some("We decided to persist meeting decisions as reviewable domain Decisions.")
    );

    let review_item: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT
            review_item.review_item_id,
            review_item.item_kind,
            review_item.metadata->>'mirrored_from',
            review_item.metadata->>'decision_id'
        FROM review_items review_item
        JOIN review_item_evidence evidence
          ON evidence.review_item_id = review_item.review_item_id
        WHERE evidence.observation_id = $1
          AND review_item.item_kind = 'potential_decision'
          AND review_item.metadata->>'decision_id' = $2
        "#,
    )
    .bind(&event.observation_id)
    .bind(linked_decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision review mirror");
    assert_eq!(review_item.1, "potential_decision");
    assert_eq!(review_item.2, "decisions");
    assert_eq!(review_item.3, linked_decision_id);
}

#[tokio::test]
async fn meeting_outcome_promise_creates_suggested_obligation_and_review_item_without_task_link_against_postgres()
 {
    let Some(pool) = live_pool().await else {
        return;
    };
    let acct_store = CalendarAccountStore::new(pool.clone());
    let event_store = CalendarEventStore::new(pool.clone());
    let outcome_service = CalendarMeetingOutcomeApplicationService::new(pool.clone());
    let obligation_store = ObligationStore::new(pool.clone());
    let suffix = unique_suffix();
    let owner_person_id = format!("person:v1:email:meeting-promise-{suffix}@example.com");
    let due_at = Utc::now() + Duration::days(3);

    let acct = acct_store
        .create("local", &format!("Promise Outcome Test {suffix}"), None)
        .await
        .expect("create account");
    let event = event_store
        .create(&NewCalendarEvent {
            title: format!("Promise outcome meeting {suffix}"),
            start_at: Utc::now(),
            end_at: Utc::now() + Duration::hours(1),
            account_id: Some(acct.account_id),
            event_type: Some("meeting".into()),
            ..Default::default()
        })
        .await
        .expect("create event");

    let outcome = outcome_service
        .add_manual(
            &event.event_id,
            "promise",
            &format!("Send meeting follow-up package {suffix}"),
            Some("Alex promised to send the follow-up package after the meeting."),
            Some(&owner_person_id),
            Some(due_at),
        )
        .await
        .expect("add promise outcome");
    let linked_obligation_id = outcome
        .linked_entity_id
        .as_deref()
        .expect("promise outcome should link to suggested Obligation");

    let obligations = obligation_store
        .list_for_entity(ObligationEntityKind::Persona, &owner_person_id, 10)
        .await
        .expect("owner obligations");
    let obligation = obligations
        .iter()
        .find(|item| item.obligation_id == linked_obligation_id)
        .expect("suggested Obligation linked to meeting outcome");

    assert_eq!(obligation.statement, outcome.title);
    assert_eq!(obligation.review_state, ObligationReviewState::Suggested);
    assert_eq!(
        obligation.due_at.map(|value| value.timestamp_micros()),
        Some(due_at.timestamp_micros())
    );

    let evidence: (String, String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, observation_id, quote FROM obligation_evidence WHERE obligation_id = $1",
    )
    .bind(linked_obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation evidence");
    assert_eq!(evidence.0, "event");
    assert_eq!(evidence.1, event.event_id);
    assert_eq!(evidence.2.as_deref(), Some(event.observation_id.as_str()));
    assert_eq!(
        evidence.3.as_deref(),
        Some("Alex promised to send the follow-up package after the meeting.")
    );

    let review_item: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT
            review_item.review_item_id,
            review_item.item_kind,
            review_item.metadata->>'mirrored_from',
            review_item.metadata->>'obligation_id'
        FROM review_items review_item
        JOIN review_item_evidence evidence
          ON evidence.review_item_id = review_item.review_item_id
        WHERE evidence.observation_id = $1
          AND review_item.item_kind = 'potential_obligation'
          AND review_item.metadata->>'obligation_id' = $2
        "#,
    )
    .bind(&event.observation_id)
    .bind(linked_obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation review mirror");
    assert_eq!(review_item.1, "potential_obligation");
    assert_eq!(review_item.2, "obligations");
    assert_eq!(review_item.3, linked_obligation_id);

    let task_link_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM obligation_task_links WHERE obligation_id = $1")
            .bind(linked_obligation_id)
            .fetch_one(&pool)
            .await
            .expect("task link count");
    assert_eq!(task_link_count, 0);
}
```

### `backend/tests/calendar/scheduling_rules.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar/scheduling_rules.rs`
- Size bytes / Размер в байтах: `2597`
- Included characters / Включено символов: `2597`
- Truncated / Обрезано: `no`

```rust
use chrono::{Duration, Utc};
use makosh_hub_backend::domains::calendar::rules::CalendarRuleStore;
use makosh_hub_backend::domains::calendar::scheduling::{DeadlineStore, FocusBlockStore};
use serde_json::json;

use super::support::{live_pool, unique_suffix};

#[tokio::test]
async fn deadlines_and_focus_blocks_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let deadline_store = DeadlineStore::new(pool.clone());
    let fb_store = FocusBlockStore::new(pool);
    let suffix = unique_suffix();
    let now = Utc::now();

    let deadline = deadline_store
        .create(
            &format!("Deadline {suffix}"),
            now + Duration::days(7),
            Some("high"),
            None,
            None,
        )
        .await
        .expect("create deadline");
    assert_eq!(deadline.severity, "high");
    assert_eq!(deadline.status, "active");

    let deadlines = deadline_store.list(None, 50).await.expect("list deadlines");
    assert!(
        deadlines
            .iter()
            .any(|item| item.title == format!("Deadline {suffix}"))
    );

    let focus_block = fb_store
        .create(
            &format!("Focus {suffix}"),
            now,
            now + Duration::hours(2),
            Some("Deep work"),
            None,
            Some("high"),
        )
        .await
        .expect("create focus block");
    assert_eq!(focus_block.protection_level, "high");

    let blocks = fb_store
        .list(None, None, 50)
        .await
        .expect("list focus blocks");
    assert!(blocks.iter().any(|b| b.title == format!("Focus {suffix}")));
}

#[tokio::test]
async fn calendar_rules_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = CalendarRuleStore::new(pool);
    let suffix = unique_suffix();

    let rule = store
        .create(
            &format!("Rule {suffix}"),
            Some("Auto-prepare meetings with clients"),
            json!({"trigger": "event_type=meeting", "action": "generate_brief"}),
            Some("suggest_only"),
        )
        .await
        .expect("create rule");
    assert!(rule.rule_id.starts_with("rule:v1:"));
    assert_eq!(rule.approval_mode, "suggest_only");

    let list = store.list().await.expect("list rules");
    assert!(list.iter().any(|r| r.rule_id == rule.rule_id));

    store.delete(&rule.rule_id).await.expect("delete rule");
    let list_after_delete = store.list().await.expect("list after delete");
    assert!(!list_after_delete.iter().any(|r| r.rule_id == rule.rule_id));
}
```

### `backend/tests/calendar/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar/support.rs`
- Size bytes / Размер в байтах: `854`
- Included characters / Включено символов: `854`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use makosh_hub_backend::platform::storage::Database;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub async fn live_pool() -> Option<PgPool> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(database.pool().expect("configured pool").clone())
}

pub fn disconnected_pool() -> PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool")
}
```

### `backend/tests/calendar_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar_api.rs`
- Size bytes / Размер в байтах: `294`
- Included characters / Включено символов: `294`
- Truncated / Обрезано: `no`

```rust
#[path = "calendar_api/accounts.rs"]
mod accounts;
#[path = "calendar_api/auth.rs"]
mod auth;
#[path = "calendar_api/event_details.rs"]
mod event_details;
#[path = "calendar_api/events.rs"]
mod events;
#[path = "calendar_api/misc.rs"]
mod misc;
#[path = "calendar_api/support.rs"]
mod support;
```

### `backend/tests/calendar_api/accounts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar_api/accounts.rs`
- Size bytes / Размер в байтах: `6283`
- Included characters / Включено символов: `6283`
- Truncated / Обрезано: `no`

```rust
use testkit::context::TestContext;

use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, delete_request_with_token, get_request_with_token, json_body,
    post_request_with_token, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};
use makosh_hub_backend::platform::storage::Database;

#[tokio::test]
async fn calendar_accounts_crud_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;
    let acct_name = format!("API Cal Acct {suffix}");

    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": &acct_name, "email": format!("cal-{suffix}@example.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let created = json_body(response).await;
    let account_id = created["account_id"]
        .as_str()
        .expect("account_id")
        .to_owned();
    assert_eq!(created["provider"], json!("google"));
    let created_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_account'
          AND entity_id = $1
          AND relationship_kind = 'create'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("calendar account create observation");

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
            ),
            json!({"account_name": format!("Updated {acct_name}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let updated = json_body(response).await;
    assert_eq!(
        updated["account_name"],
        json!(format!("Updated {acct_name}"))
    );
    let updated_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_account'
          AND entity_id = $1
          AND relationship_kind = 'update'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("calendar account update observation");
    assert_ne!(updated_observation_id, created_observation_id);

    let response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let deleted = json_body(response).await;
    assert_eq!(deleted["deleted"], json!(true));
    let deleted_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_account'
          AND entity_id = $1
          AND relationship_kind = 'delete'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("calendar account delete observation");
    assert_ne!(deleted_observation_id, updated_observation_id);
    let delete_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&deleted_observation_id)
    .fetch_one(&pool)
    .await
    .expect("calendar account delete observation row");
    assert_eq!(
        delete_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );
    assert_eq!(
        delete_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "CALENDAR_ACCOUNT_MUTATION"
    );
}

#[tokio::test]
async fn calendar_accounts_list_returns_items() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;

    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("List Acct {suffix}"), "email": format!("list-{suffix}@example.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/accounts",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    assert!(!body["items"].as_array().expect("items").is_empty());
}
```

### `backend/tests/calendar_api/auth.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar_api/auth.rs`
- Size bytes / Размер в байтах: `1003`
- Included characters / Включено символов: `1003`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router;

use super::support::{config_with_api_token, get_request, json_body};

#[tokio::test]
async fn calendar_accounts_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request("/api/v1/calendar/accounts"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({"error": "invalid_api_secret", "message": "missing or invalid x-makosh-secret header"})
    );
}

#[tokio::test]
async fn calendar_events_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request("/api/v1/calendar/events"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
```

### `backend/tests/calendar_api/event_details.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar_api/event_details.rs`
- Size bytes / Размер в байтах: `22255`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use testkit::context::TestContext;

use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, create_cal_event, get_request_with_token, json_body,
    post_request_with_token, unique_suffix, urlencoding_percent_encode,
};
use makosh_hub_backend::platform::storage::Database;

async fn event_get_endpoint_returns_non_server_error(path_suffix: &str) -> Option<Value> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return None;
    };

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/{}",
                urlencoding_percent_encode(&event_id),
                path_suffix
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    Some(json_body(response).await)
}

#[tokio::test]
async fn calendar_event_relations_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("relations").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_context_pack_returns_ok() {
    event_get_endpoint_returns_non_server_error("context-pack").await;
}

#[tokio::test]
async fn calendar_event_agenda_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("agenda").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_checklist_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("checklist").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_risks_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("risks").await else {
        return;
    };
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn calendar_event_meeting_notes_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("notes").await else {
        return;
    };
    assert!(body["items"].is_array() || body.is_object());
}

#[tokio::test]
async fn calendar_event_meeting_outcomes_list_returns_empty() {
    let Some(body) = event_get_endpoint_returns_non_server_error("outcomes").await else {
        return;
    };
    assert!(body["items"].is_array() || body.is_object());
}

#[tokio::test]
async fn calendar_event_recording_list_returns_ok() {
    event_get_endpoint_returns_non_server_error("recording").await;
}

#[tokio::test]
async fn calendar_event_transcript_returns_ok() {
    event_get_endpoint_returns_non_server_error("transcript").await;
}

#[tokio::test]
async fn calendar_event_brief_returns_ok() {
    event_get_endpoint_returns_non_server_error("brief").await;
}

#[tokio::test]
async fn calendar_event_export_returns_text() {
    event_get_endpoint_returns_non_server_error("export").await;
}

#[tokio::test]
async fn calendar_event_reminders_list_returns_empty() {
    event_get_endpoint_returns_non_server_error("reminders").await;
}

macro_rules! cal_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_context = TestContext::new().await;
            let database_url = test_context.connection_string();
            let suffix = unique_suffix();
            let app = build_cal_app(&database_url).await;
            let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
                eprintln!("skip: no event");
                return;
            };
            let response = app
                .oneshot(post_request_with_token(
                    &format!(
                        "/api/v1/calendar/events/{}/{}",
                        urlencoding_percent_encode(&event_id),
                        $path_suffix
                    ),
                    $body,
                    LOCAL_API_TOKEN,
                ))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                response.status()
            );
        }
    };
}

cal_post_test!(
    cal_event_post_relation,
    "relations",
    json!({"related_event_id": "event:fake", "relation_type": "follows"})
);
cal_post_test!(
    cal_event_post_context_pack,
    "context-pack",
    json!({"summary": "Test context"})
);
cal_post_test!(
    cal_event_post_agenda,
    "agenda",
    json!({"item": "Test agenda item", "order_index": 1})
);
cal_post_test!(
    cal_event_post_checklist,
    "checklist",
    json!({"item": "Test checklist", "done": false})
);
cal_post_test!(
    cal_event_post_meeting_note,
    "notes",
    json!({"content": "Test note", "note_type": "action_item"})
);
cal_post_test!(
    cal_event_post_meeting_outcome,
    "outcomes",
    json!({"outcome": "Test outcome", "decision": false})
);
cal_post_test!(
    cal_event_post_follow_up,
    "follow-up",
    json!({"action": "Send follow-up", "due_by": "2027-12-01T00:00:00Z"})
);
cal_post_test!(
    cal_event_post_recording,
    "recording",
    json!({"url": "https://example.com/rec", "format": "mp4"})
);
cal_post_test!(
    cal_event_post_generate_agenda,
    "generate-agenda",
    json!({"participant_count": 3, "duration_minutes": 60})
);
cal_post_test!(
    cal_event_post_reminder,
    "reminders",
    json!({"minutes_before": 15, "method": "notification"})
);

#[tokio::test]
async fn cal_event_follow_up_status() {
    event_get_endpoint_returns_non_server_error("follow-up-status").await;
}

#[tokio::test]
async fn cal_event_reminder_toggle() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: no event");
        return;
    };
    let create_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reminders",
                urlencoding_percent_encode(&event_id)
            ),
            json!({
                "reminder_type": "time_based",
                "minutes_before": 15,
                "message": "Prepare for the meeting"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create reminder response");
    assert_eq!(create_response.status(), axum::http::StatusCode::OK);
    let reminder_id = json_body(create_response).await["id"]
        .as_str()
        .expect("reminder id")
        .to_owned();

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reminders/{}/toggle",
                urlencoding_percent_encode(&event_id),
                reminder_id
            ),
            json!({"active": false}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "reminder toggle={}",
        response.status()
    );

    let pool = Database::connect(Some(&database_url))
        .await
        .expect("database")
        .pool()
        .expect("pool")
        .clone();
    let source: String =
        sqlx::query_scalar("SELECT source FROM calendar_reminders WHERE id::text = $1")
            .bind(&reminder_id)
            .fetch_one(&pool)
            .await
            .expect("reminder source");
    assert!(source.starts_with("observation:"));

    let observation_id = source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(observation_id)
            .fetch_one(&pool)
            .await
            .expect("toggle observation");
    assert_eq!(origin_kind, "manual");

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'calendar'
           AND entity_kind = 'event_reminder'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&reminder_id)
    .fetch_one(&pool)
    .await
    .expect("reminder observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn calendar_manual_event_materials_capture_observations_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: no event");
        return;
    };
    let pool = Database::connect(Some(&database_url))
        .await
        .expect("database")
        .pool()
        .expect("pool")
        .clone();
    let encoded_event_id = urlencoding_percent_encode(&event_id);

    let agenda_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/calendar/events/{encoded_event_id}/agenda"),
            json!({
                "items": ["Kickoff", "Scope review"],
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("agenda response");
    assert_eq!(agenda_response.status(), axum::http::StatusCode::OK);
    let agenda_id = json_body(agenda_response).await["id"]
        .as_str()
        .expect("agenda id")
        .to_owned();
    let agenda_source: String =
        sqlx::query_scalar("SELECT source FROM event_agendas WHERE id::text = $1")
            .bind(&agenda_id)
            .fetch_one(&pool)
            .await
            .expect("agenda source");
    assert!(agenda_source.starts_with("observation:"));

    let checklist_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/calendar/events/{encoded_event_id}/checklist"),
            json!({
                "items": [{"text": "Prepare deck", "done": false}],
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("checklist response");
    assert_eq!(checklist_response.status(), axum::http::StatusCode::OK);
    let checklist_id = json_body(checklist_response).await["id"]
        .as_str()
        .expect("checklist id")
        .to_owned();
    let checklist_source: String =
        sqlx::query_scalar("SELECT source FROM event_checklists WHERE id::text = $1")
            .bind(&checklist_id)
            .fetch_one(&pool)
            .await
            .expect("checklist source");
    assert!(checklist_source.starts_with("observation:"));

    let note_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/calendar/events/{encoded_event_id}/notes"),
            json!({
                "content": "Discussed migration sequencing.",
                "format": "markdown",
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("note response");
    assert_eq!(note_response.status(), axum::http::StatusCode::OK);
    let note_id = json_body(note_response).await["id"]
        .as_str()
        .expect("note id")
        .to_owned();
    let note_source: String =
        sqlx::query_scalar("SELECT source FROM meeting_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/calendar_api/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar_api/events.rs`
- Size bytes / Размер в байтах: `18664`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use testkit::context::TestContext;

use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, create_cal_event, delete_request_with_token,
    get_request_with_token, json_body, post_request_with_token, put_request_with_token,
    unique_suffix, urlencoding_percent_encode,
};
use makosh_hub_backend::platform::storage::Database;

#[tokio::test]
async fn calendar_events_crud_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let fetched = json_body(response).await;
    assert_eq!(fetched["event_id"], json!(event_id));

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
            ),
            json!({"title": format!("Updated Event {suffix}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let updated = json_body(response).await;
    assert_eq!(updated["title"], json!(format!("Updated Event {suffix}")));

    let response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

#[tokio::test]
async fn calendar_events_list_returns_items() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    create_cal_event(&app, suffix).await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/events",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    let _items = body["items"].as_array().expect("items");
}

#[tokio::test]
async fn calendar_event_reschedule() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let new_start = Utc::now() + Duration::hours(3);
    let new_end = Utc::now() + Duration::hours(4);

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reschedule",
                urlencoding_percent_encode(&event_id)
            ),
            json!({"start_at": new_start.to_rfc3339(), "end_at": new_end.to_rfc3339()}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
}

#[tokio::test]
async fn calendar_event_cancel() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/cancel",
                urlencoding_percent_encode(&event_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    assert_eq!(body["cancelled"], json!(true));
}

#[tokio::test]
async fn calendar_event_participants_crud() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/participants",
                urlencoding_percent_encode(&event_id)
            ),
            json!({
                "email": format!("participant-{suffix}@example.com"),
                "display_name": format!("Participant {suffix}"),
                "role": "required"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let created = json_body(response).await;
    let participant_id = created["id"].as_str().expect("participant id").to_owned();

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/participants",
                urlencoding_percent_encode(&event_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    assert!(!body["items"].as_array().expect("items").is_empty());

    let pool = Database::connect(Some(&database_url))
        .await
        .expect("database")
        .pool()
        .expect("pool")
        .clone();
    let participant_source: String =
        sqlx::query_scalar("SELECT source FROM event_participants WHERE id::text = $1")
            .bind(&participant_id)
            .fetch_one(&pool)
            .await
            .expect("participant source");
    assert!(participant_source.starts_with("observation:"));

    let observation_id = participant_source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(observation_id)
            .fetch_one(&pool)
            .await
            .expect("participant observation");
    assert_eq!(origin_kind, "manual");

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'calendar'
           AND entity_kind = 'event_participant'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&participant_id)
    .fetch_one(&pool)
    .await
    .expect("participant observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn calendar_event_manual_lifecycle_captures_append_only_observations_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;
    let Some((_, event_id)) = create_cal_event(&app, suffix).await else {
        eprintln!("skip: event create failed");
        return;
    };
    let pool = Database::connect(Some(&database_url))
        .await
        .expect("database")
        .pool()
        .expect("pool")
        .clone();

    let created_row =
        sqlx::query("SELECT observation_id, status FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("created event row");
    let created_observation_id: String = created_row
        .try_get("observation_id")
        .expect("created observation id");
    let created_status: String = created_row.try_get("status").expect("created status");
    assert_eq!(created_status, "confirmed");
    let created_kind: String = sqlx::query_scalar(
        r#"
        SELECT kind.code
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&created_observation_id)
    .fetch_one(&pool)
    .await
    .expect("created observation kind");
    let created_origin: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&created_observation_id)
            .fetch_one(&pool)
            .await
            .expect("created observation origin");
    assert_eq!(created_kind, "CALENDAR_EVENT");
    assert_eq!(created_origin, "manual");

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}",
                urlencoding_percent_encode(&event_id)
            ),
            json!({"title": format!("Updated Event {suffix}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("update response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let updated_row =
        sqlx::query("SELECT observation_id, title FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("updated event row");
    let updated_observation_id: String = updated_row
        .try_get("observation_id")
        .expect("updated observation id");
    assert_ne!(updated_observation_id, created_observation_id);

    let new_start = Utc::now() + Duration::hours(3);
    let new_end = Utc::now() + Duration::hours(4);
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/events/{}/reschedule",
                urlencoding_percent_encode(&event_id)
            ),
            json!({"start_at": new_start.to_rfc3339(), "end_at": new_end.to_rfc3339()}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reschedule response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let rescheduled_row =
        sqlx::query("SELECT observation_id, status FROM calendar_events WHERE event_id = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("rescheduled event row");
    let rescheduled_observation_id: String = rescheduled_row
        .try_get("observation_id")
        .expect("rescheduled observation id");
    let rescheduled_status: String = rescheduled_row
        .try_get("status")
        .expect("rescheduled status");
    assert_ne!(rescheduled_observation_id, updated_observation_id);
    assert_eq!(rescheduled_status, "resche
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/calendar_api/misc.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar_api/misc.rs`
- Size bytes / Размер в байтах: `18564`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use testkit::context::TestContext;

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, delete_request_with_token, get_request_with_token, json_body,
    post_request_with_token, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};
use makosh_hub_backend::platform::storage::Database;

async fn get_calendar_endpoint_returns_non_server_error(path: &str) {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(path, LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "{path} status={}",
        response.status()
    );
}

#[tokio::test]
async fn calendar_deadlines_list_returns_empty() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/deadlines").await;
}

#[tokio::test]
async fn calendar_focus_blocks_list_returns_empty() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/focus-blocks").await;
}

#[tokio::test]
async fn calendar_watchtower_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/watchtower").await;
}

#[tokio::test]
async fn calendar_health_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/health").await;
}

#[tokio::test]
async fn calendar_weekly_brief_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/weekly-brief").await;
}

#[tokio::test]
async fn calendar_search_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/search?q=meeting").await;
}

#[tokio::test]
async fn calendar_rules_list_returns_empty() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/rules").await;
}

#[tokio::test]
async fn calendar_analytics_distribution_returns_ok() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics/distribution").await;
}

#[tokio::test]
async fn calendar_sources_list() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;
    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("SrcAcct{suffix}"), "email": format!("src-{suffix}@x.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: acct create failed");
        return;
    }
    let account_id = json_body(response).await["account_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    if account_id.is_empty() {
        eprintln!("skip: no account_id");
        return;
    }

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sources",
                urlencoding_percent_encode(&account_id)
            ),
            json!({"name": format!("Src{suffix}"), "color": "#ff0000", "timezone": "UTC"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "src create={}",
        response.status()
    );
    let body = json_body(response).await;
    let source_id = body["source_id"].as_str().expect("source_id");
    let observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_source'
          AND entity_id = $1
          AND relationship_kind = 'create'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(source_id)
    .fetch_one(&pool)
    .await
    .expect("calendar source observation link");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("calendar source observation");
    assert_eq!(origin_kind, "manual");

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}/sources",
                urlencoding_percent_encode(&account_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "src list={}",
        response.status()
    );
}

#[tokio::test]
async fn cal_post_deadline() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;
    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/calendar/deadlines",
            json!({"title": "Test Deadline", "due_at": "2027-12-31T23:59:59Z", "severity": "high"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "deadline post={}",
        response.status()
    );
    let body = json_body(response).await;
    let deadline_id = body["id"].as_str().expect("deadline id");
    let observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'deadline_event'
          AND entity_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(deadline_id)
    .fetch_one(&pool)
    .await
    .expect("deadline observation link");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("deadline observation");
    assert_eq!(origin_kind, "manual");
}

#[tokio::test]
async fn cal_post_focus_block() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;
    let start_at = Utc::now() + Duration::hours(2);
    let end_at = start_at + Duration::minutes(90);
    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/calendar/focus-blocks",
            json!({
                "title": "Focus Block",
                "start_at": start_at.to_rfc3339(),
                "end_at": end_at.to_rfc3339(),
                "purpose": "Deep work"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "focus block post={}",
        response.status()
    );
    let body = json_body(response).await;
    let focus_block_id = body["id"].as_str().expect("focus block id");
    let observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'focus_block'
          AND entity_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(focus_block_id)
    .fetch_one(&pool)
    .await
    .expect("focus block observation link");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("focus block observation");
    assert_eq!(origin_kind, "manual");
}

#[tokio::test]
async fn cal_post_smart_schedule() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let app = build_cal_app(&database_url).await;
    let response = app.oneshot(post_request_with_token(
        "/api/v1/calendar/smart-schedule",
        json!({"task_title": "Schedule me", "duration_minutes": 60, "deadline": "2027-12-31T23:59:59Z"}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "smart schedule={}",
        response.status()
    );
}

#[tokio::test]
async fn cal_analytics() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics").await;
}

#[tokio::test]
async fn cal_focus_balance() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics/focus-balance")
        .await;
}

#[tokio::test]
async fn cal_back_to_back() {
    get_calendar_endpoint_returns_non_server_error("/api/v1/calendar/analytics/back-to-back").await;
}

#[tokio::test]
async fn cal_rules_crud() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/calendar/rules",
            json!({
                "name": format!("Rule{suffix}"),
                "description": "Color busy blocks",
                "dsl": {"color": "#00ff00"},
                "approval_mode": "suggest_only"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: rule create failed");
        return;
    }
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_default();
    let value: Value = serde_json::from_slice(&bytes).unwrap_or_default();
    let rule_id = value["rule_id"].as_str().unwrap_or("").to_owned();
    if rule_id.is_empty() {
        return;
    }
    let created_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_rule'
          AND entity_id = $1
          AND relationship_kind = 'create'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&rule_id)
    .fetch_one(&pool)
    .await
    .expect("calendar rule create observation");

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/rules/{}",
                urlencoding_percent_encode(&rule_id)
            ),
            json!({"name": format!("Updated{suffix}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "rule update={}",
        response.status()
    );
    let updated_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_rule'
          AND entity_id = $1
          AND relationship_kind = 'update'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&rule_id)
    .fetch_one(&pool)
    .await
    .expect("calendar rule update observation");
    assert_ne!(updated_observation_id, created_observation_id);

    let response = app

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/calendar_api/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar_api/support.rs`
- Size bytes / Размер в байтах: `4129`
- Included characters / Включено символов: `4129`
- Truncated / Обрезано: `no`

```rust
#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, header};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;

pub const LOCAL_API_TOKEN: &str = "cal-api-test-token";

pub fn config_with_api_token() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

pub fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

pub fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub fn post_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn put_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn delete_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

pub fn urlencoding_percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

pub async fn build_cal_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url),
        database,
    )
}

pub async fn create_cal_event(app: &axum::Router, suffix: u128) -> Option<(String, String)> {
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/calendar/accounts",
            json!({
                "provider": "google",
                "account_name": format!("Evt Acct {suffix}"),
                "email": format!("evt-{suffix}@example.com")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        return None;
    }
    let account_id = json_body(response).await["account_id"].as_str()?.to_owned();

    let now = Utc::now();
    let start = now + Duration::hours(1);
    let end = now + Duration::hours(2);

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/calendar/events",
            json!({
                "account_id": &account_id,
                "title": format!("Test Event {suffix}"),
                "start_at": start.to_rfc3339(),
                "end_at": end.to_rfc3339(),
                "status": "confirmed",
                "event_type": "meeting",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        return None;
    }
    let body = json_body(response).await;
    let event_id = body["event_id"].as_str()?;
    Some((account_id, event_id.to_owned()))
}
```

### `backend/tests/calendar_api_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar_api_architecture.rs`
- Size bytes / Размер в байтах: `1726`
- Included characters / Включено символов: `1726`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn calendar_api_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_calendar_api_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "calendar API test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_calendar_api_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_calendar_api_violations(&path, violations);
            continue;
        }
        if !is_calendar_api_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_calendar_api_test_file(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value.starts_with("calendar_api"))
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/calendar_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calendar_architecture.rs`
- Size bytes / Размер в байтах: `1932`
- Included characters / Включено символов: `1932`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn calendar_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_calendar_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "calendar test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_calendar_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_calendar_test_violations(&path, violations);
            continue;
        }
        if !is_calendar_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_calendar_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "calendar.rs" || file_name == "calendar_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "calendar")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/calls_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/calls_api.rs`
- Size bytes / Размер в байтах: `3143`
- Included characters / Включено символов: `3143`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;

const TOKEN: &str = "calls-test-token";

fn cfg() -> AppConfig {
    testkit::app::config_with_secret(TOKEN)
}

fn get(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("req")
}

fn post(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("req")
}

async fn body(response: axum::response::Response) -> Value {
    let b = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&b).expect("json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("t")
        .as_nanos()
}

async fn app(db: &str) -> axum::Router {
    let database = Database::connect(Some(db)).await.expect("db");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(TOKEN, db),
        database,
    )
}

#[tokio::test]
async fn calls_reject_no_secret() {
    let r = build_router(cfg());
    let resp = r.oneshot(get("/api/v1/calls", "")).await.expect("r");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn calls_list_ok() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let a = app(&db).await;
    let resp = a.oneshot(get("/api/v1/calls", TOKEN)).await.expect("r");
    assert!(!resp.status().is_server_error(), "status={}", resp.status());
    assert!(body(resp).await["items"].is_array());
}

#[tokio::test]
async fn call_create_ok() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = app(&db).await;
    let resp = a.oneshot(post("/api/v1/calls", json!({
        "call_type": "telegram", "chat_id": format!("c{s}"), "direction": "inbound",
        "state": "completed", "initiated_at": chrono::Utc::now().to_rfc3339(), "duration_seconds": 120
    }), TOKEN)).await.expect("r");
    assert!(!resp.status().is_server_error(), "status={}", resp.status());
}

#[tokio::test]
async fn call_transcript_404() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = app(&db).await;
    let resp = a
        .oneshot(get(
            &format!("/api/v1/calls/call:nonexistent-{s}/transcript"),
            TOKEN,
        ))
        .await
        .expect("r");
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}
```

### `backend/tests/characterization_communication.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/characterization_communication.rs`
- Size bytes / Размер в байтах: `6412`
- Included characters / Включено символов: `6336`
- Truncated / Обрезано: `no`

```rust
//! Characterization tests for Communication API.
//!
//! Captures CURRENT behavior before alignment refactoring (Phase 3+).
//! Do NOT change existing behavior — only add tests.
//!
//! These live tests use the shared testcontainers pgvector fixture with
//! per-test migrated databases.

use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::Value;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;

const TOKEN: &str = "char-comm-test-token";

fn cfg(db: &str) -> AppConfig {
    testkit::app::config_with_secret_and_database_url(TOKEN, db)
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", TOKEN)
        .body(Body::empty())
        .expect("req")
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", TOKEN)
        .body(Body::from(body.to_string()))
        .expect("req")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

async fn build_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(cfg(database_url), database)
}

async fn live_app(_test_name: &str) -> Option<axum::Router> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    Some(build_app(&database_url).await)
}

// ── AC3: Communication API characterization ─────────────────────────────────

/// AC3 characterisation: GET /api/v1/communications/messages returns 200.
#[tokio::test]
async fn char_communications_messages_list_returns_ok() {
    let Some(app) = live_app("communications messages list").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/communications/messages"))
        .await
        .expect("response");

    assert!(
        !response.status().is_server_error(),
        "GET /api/v1/communications/messages must not return 5xx, got {:?}",
        response.status()
    );

    let body = json_body(response).await;
    // Characterize response shape — should contain items array
    assert!(
        body.get("items").is_some() || body.is_array(),
        "Response should contain items array or be an array, got keys: {:?}",
        body.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
}

/// AC3 characterisation: GET /api/v1/communications/search returns 200.
#[tokio::test]
async fn char_communications_search_returns_ok() {
    let Some(app) = live_app("communications search").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/communications/search?q=test"))
        .await
        .expect("response");

    assert!(
        !response.status().is_server_error(),
        "GET /api/v1/communications/search must not return 5xx, got {:?}",
        response.status()
    );
}

/// AC3 characterisation: GET /api/v1/communications/threads returns 200.
#[tokio::test]
async fn char_communications_threads_list_returns_ok() {
    let Some(app) = live_app("communications threads list").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/communications/threads"))
        .await
        .expect("response");

    assert!(
        !response.status().is_server_error(),
        "GET /api/v1/communications/threads must not return 5xx, got {:?}",
        response.status()
    );
}

/// AC3 characterisation: GET /api/v1/communications/messages/states returns 200.
#[tokio::test]
async fn char_communications_message_states_returns_ok() {
    let Some(app) = live_app("communications message states").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/communications/messages/states"))
        .await
        .expect("response");

    assert!(
        !response.status().is_server_error(),
        "GET /api/v1/communications/messages/states must not return 5xx, got {:?}",
        response.status()
    );
}

/// AC3 characterisation: GET /api/v1/communications/drafts returns 200.
#[tokio::test]
async fn char_communications_drafts_list_returns_ok() {
    let Some(app) = live_app("communications drafts list").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/communications/drafts"))
        .await
        .expect("response");

    assert!(
        !response.status().is_server_error(),
        "GET /api/v1/communications/drafts must not return 5xx, got {:?}",
        response.status()
    );
}

/// AC3 characterisation: GET /api/v1/communications by specific message ID returns 200 or 404.
#[tokio::test]
async fn char_communication_message_by_id_returns_ok_or_404() {
    let Some(app) = live_app("communication message by id").await else {
        return;
    };

    // Non-existent message — expect 404
    let response = app
        .oneshot(get("/api/v1/communications/messages/rec:nonexistent"))
        .await
        .expect("response");

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "non-existent message should return 404"
    );
}

/// AC3 characterisation: POST to workflow-actions endpoint.
#[tokio::test]
async fn char_workflow_actions_endpoint_accepts_valid_body() {
    let Some(app) = live_app("workflow actions endpoint").await else {
        return;
    };

    let response = app
        .oneshot(post(
            "/api/v1/workflow-actions",
            serde_json::json!({
                "action": "archive",
                "message_ids": []
            }),
        ))
        .await
        .expect("response");

    // Accept either 200 (empty archive succeeds) or 4xx (validation)
    assert!(
        !response.status().is_server_error(),
        "POST /api/v1/workflow-actions must not return 5xx, got {:?}",
        response.status()
    );
}
```

### `backend/tests/characterization_person.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/characterization_person.rs`
- Size bytes / Размер в байтах: `6439`
- Included characters / Включено символов: `6349`
- Truncated / Обрезано: `no`

```rust
//! Characterization tests for Person / Persona API.
//!
//! Captures CURRENT behavior before alignment refactoring (Phase 2+).
//! Do NOT change existing behavior — only add tests.
//!
//! These live tests use the shared testcontainers pgvector fixture with
//! per-test migrated databases.

use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;

const TOKEN: &str = "char-person-test-token";

fn cfg(db: &str) -> AppConfig {
    testkit::app::config_with_secret_and_database_url(TOKEN, db)
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", TOKEN)
        .body(Body::empty())
        .expect("req")
}

fn put(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", TOKEN)
        .body(Body::from(body.to_string()))
        .expect("req")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

async fn build_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(cfg(database_url), database)
}

async fn live_app(_test_name: &str) -> Option<axum::Router> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    Some(build_app(&database_url).await)
}

// ── AC2: Person API characterization ────────────────────────────────────────

/// GAP-2 characterisation: GET /api/v1/persons returns 200 with items array.
#[tokio::test]
async fn char_persons_list_returns_ok() {
    let Some(app) = live_app("persons list").await else {
        return;
    };

    let response = app.oneshot(get("/api/v1/persons")).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert!(
        body.get("items").is_some(),
        "GET /api/v1/persons must return 'items' array"
    );

    // Characterize current pagination structure
    if let Some(items) = body["items"].as_array() {
        assert!(
            items.len() <= 50,
            "default persons limit should be <= 50, got {:?}",
            items.len()
        );
    }
}

/// GAP-2 characterisation: GET /api/v1/personas returns persona-native schema.
#[tokio::test]
async fn char_personas_list_returns_ok() {
    let Some(app) = live_app("personas list").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/personas"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert!(
        body.get("items").is_some(),
        "GET /api/v1/personas must return 'items' array"
    );
}

/// GAP-2 characterisation: both /api/v1/persons and /api/v1/personas coexist.
#[tokio::test]
async fn char_persons_and_personas_both_exist() {
    let Some(app) = live_app("persons and personas coexistence").await else {
        return;
    };

    let persons_resp = app
        .clone()
        .oneshot(get("/api/v1/persons"))
        .await
        .expect("persons response");
    assert_eq!(persons_resp.status(), StatusCode::OK);

    let personas_resp = app
        .clone()
        .oneshot(get("/api/v1/personas"))
        .await
        .expect("personas response");
    assert_eq!(personas_resp.status(), StatusCode::OK);
}

/// GAP-2 characterisation: GET /api/v1/persons/owner returns the owner persona envelope.
#[tokio::test]
async fn char_owner_persona_returns_ok() {
    let Some(app) = live_app("owner persona").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/persons/owner"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let owner = body
        .get("owner_persona")
        .expect("Owner response should contain owner_persona envelope");
    assert!(
        owner.is_null() || (owner.get("person_id").is_some() && owner.get("is_self").is_some()),
        "Owner persona should be null or contain person fields: {body:?}",
    );
}

/// GAP-2 characterisation: GET /api/v1/persons/search requires a 'q' param.
#[tokio::test]
async fn char_person_search_requires_query() {
    let Some(app) = live_app("person search query validation").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/persons/search"))
        .await
        .expect("response");
    // Expect 400 for empty search
    assert!(
        response.status().is_client_error(),
        "search without query should return 4xx, got {:?}",
        response.status()
    );
}

/// GAP-2 characterisation: GET /api/v1/persons/{id} returns person by ID.
#[tokio::test]
async fn char_person_by_id_returns_ok_or_404() {
    let Some(app) = live_app("person by id").await else {
        return;
    };

    // Using a non-existent person ID — expect 404
    let response = app
        .oneshot(get("/api/v1/persons/person:nonexistent"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// GAP-2 characterisation: PUT /api/v1/personas/{persona_id} updates persona.
#[tokio::test]
async fn char_persona_update_accepts_valid_body() {
    let Some(app) = live_app("persona update").await else {
        return;
    };

    // Non-existent persona — expect 404 or appropriate error
    let response = app
        .oneshot(put(
            "/api/v1/personas/persona:nonexistent",
            json!({"name": "Updated Name"}),
        ))
        .await
        .expect("response");
    assert!(
        response.status().is_client_error(),
        "updating non-existent persona should return 4xx, got {:?}",
        response.status()
    );
}
```

### `backend/tests/communication_ingestion.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/communication_ingestion.rs`
- Size bytes / Размер в байтах: `348`
- Included characters / Включено символов: `348`
- Truncated / Обрезано: `no`

```rust
#[path = "communication_ingestion/contracts.rs"]
mod contracts;
#[path = "communication_ingestion/credential_reader.rs"]
mod credential_reader;
#[path = "communication_ingestion/raw_records.rs"]
mod raw_records;
#[path = "communication_ingestion/secret_bindings.rs"]
mod secret_bindings;
#[path = "communication_ingestion/support.rs"]
mod support;
```

### `backend/tests/communication_ingestion/contracts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/communication_ingestion/contracts.rs`
- Size bytes / Размер в байтах: `5180`
- Included characters / Включено символов: `5180`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;

#[test]
fn email_provider_kind_supports_gmail_icloud_and_raw_imap() {
    assert_eq!(
        EmailProviderKind::try_from("gmail").expect("gmail provider kind"),
        EmailProviderKind::Gmail
    );
    assert_eq!(
        EmailProviderKind::try_from("icloud").expect("icloud provider kind"),
        EmailProviderKind::Icloud
    );
    assert_eq!(
        EmailProviderKind::try_from("imap").expect("imap provider kind"),
        EmailProviderKind::Imap
    );
    assert!(EmailProviderKind::try_from("exchange").is_err());
}

#[test]
fn provider_account_secret_purpose_accepts_expected_secret_kinds() {
    assert!(ProviderAccountSecretPurpose::OauthToken.accepts_secret_kind(SecretKind::OauthToken));
    assert!(!ProviderAccountSecretPurpose::OauthToken.accepts_secret_kind(SecretKind::Password));
    assert!(!ProviderAccountSecretPurpose::OauthToken.accepts_secret_kind(SecretKind::AppPassword));

    assert!(ProviderAccountSecretPurpose::ImapPassword.accepts_secret_kind(SecretKind::Password));
    assert!(
        ProviderAccountSecretPurpose::ImapPassword.accepts_secret_kind(SecretKind::AppPassword)
    );
    assert!(
        !ProviderAccountSecretPurpose::ImapPassword.accepts_secret_kind(SecretKind::OauthToken)
    );

    assert!(ProviderAccountSecretPurpose::SmtpPassword.accepts_secret_kind(SecretKind::Password));
    assert!(
        ProviderAccountSecretPurpose::SmtpPassword.accepts_secret_kind(SecretKind::AppPassword)
    );
    assert!(
        !ProviderAccountSecretPurpose::SmtpPassword.accepts_secret_kind(SecretKind::OauthToken)
    );
}

#[tokio::test]
async fn communication_ingestion_registers_email_provider_accounts_against_postgres() {
    let Some(database) = connect_database("communication ingestion test fixture database").await
    else {
        return;
    };

    let store = CommunicationIngestionStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();

    let accounts = [
        NewProviderAccount::new(
            format!("acct_gmail_{suffix}"),
            EmailProviderKind::Gmail,
            "Gmail primary",
            format!("gmail-user-{suffix}@example.com"),
        )
        .config(json!({"auth": "oauth", "api": "gmail"})),
        NewProviderAccount::new(
            format!("acct_icloud_{suffix}"),
            EmailProviderKind::Icloud,
            "iCloud Mail",
            format!("icloud-user-{suffix}@icloud.com"),
        )
        .config(json!({"auth": "app_password", "transport": "imap"})),
        NewProviderAccount::new(
            format!("acct_imap_{suffix}"),
            EmailProviderKind::Imap,
            "Generic IMAP",
            format!("imap-user-{suffix}@example.net"),
        )
        .config(json!({"host": "imap.example.net", "port": 993, "tls": true})),
    ];

    for account in accounts {
        let stored = store
            .upsert_provider_account(&account)
            .await
            .expect("store provider account");

        assert_eq!(stored.account_id, account.account_id);
        assert_eq!(stored.provider_kind, account.provider_kind);
        assert_eq!(stored.external_account_id, account.external_account_id);
        assert_eq!(stored.config, account.config);
    }
}

#[tokio::test]
async fn communication_ingestion_tracks_checkpoints_against_postgres() {
    let Some(database) = connect_database("communication checkpoint test fixture database").await
    else {
        return;
    };

    let store = CommunicationIngestionStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_checkpoint_{suffix}");

    store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Icloud,
            "iCloud checkpoint test",
            format!("checkpoint-{suffix}@icloud.com"),
        ))
        .await
        .expect("store provider account");

    let saved = store
        .save_checkpoint(&NewIngestionCheckpoint::new(
            &account_id,
            "imap:INBOX",
            json!({
                "provider": "icloud",
                "mailbox": "INBOX",
                "uid_validity": 42,
                "last_seen_uid": 1001
            }),
        ))
        .await
        .expect("save checkpoint");

    assert_eq!(saved.account_id, account_id);
    assert_eq!(saved.stream_id, "imap:INBOX");
    assert_eq!(saved.checkpoint["last_seen_uid"], 1001);

    let updated = store
        .save_checkpoint(&NewIngestionCheckpoint::new(
            &saved.account_id,
            &saved.stream_id,
            json!({
                "provider": "icloud",
                "mailbox": "INBOX",
                "uid_validity": 42,
                "last_seen_uid": 1002
            }),
        ))
        .await
        .expect("update checkpoint");

    assert_eq!(updated.checkpoint["last_seen_uid"], 1002);

    let loaded = store
        .checkpoint(&saved.account_id, &saved.stream_id)
        .await
        .expect("load checkpoint")
        .expect("checkpoint exists");
    assert_eq!(loaded.checkpoint["last_seen_uid"], 1002);
}
```
