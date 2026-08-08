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

- Chunk ID / ID чанка: `088-test-backend-part-011`
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

### `backend/tests/signal_hub.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/signal_hub.rs`
- Size bytes / Размер в байтах: `95434`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::{Duration, Utc};
use serde_json::json;
use testkit::context::TestContext;

use makosh_hub_backend::application::SignalHubReplayService;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionPort, CommunicationProviderAccountStore, CommunicationProviderKind,
    NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, consume_accepted_signal_event,
    project_accepted_signal_if_runtime_allows,
};
use makosh_hub_backend::domains::persons::core::PERSON_ROLE_ASSIGNED_EVENT_TYPE;
use makosh_hub_backend::domains::signal_hub::dispatch_telegram_raw_signal;
use makosh_hub_backend::domains::signal_hub::{
    SIGNAL_HUB_RAW_SIGNAL_CONSUMER, SignalConnectionCreate, SignalFixtureEmitRequest,
    SignalFixtureSourceService, SignalHealthCheckRequest, SignalHubConnectionService,
    SignalHubControlRequest, SignalHubControlService, SignalHubHealthService,
    SignalHubProfileService, SignalHubSignalService, SignalHubStore, SignalPolicy,
    SignalPolicyDecision, SignalPolicyEvaluator, SignalPolicyMode, SignalPolicyScope,
    SignalProcessingOutcome, SignalReplayRequestCreate, SignalRuntimeStateUpdate,
    process_signal_hub_raw_event,
};
use makosh_hub_backend::platform::events::{
    EventConsumerConfig, EventConsumerRunner, EventConsumerStore, EventDeadLetterReviewState,
    EventLogQuery, EventStore, EventStoreError, NewEventEnvelope, ProjectionCursorStore,
    runtime_allows_processing,
};
use makosh_hub_backend::platform::settings::ApplicationSettingsStore;
use makosh_hub_backend::workflows::person_derived_evidence::{
    PERSON_DERIVED_EVIDENCE_CONSUMER, project_person_derived_evidence_event,
};
use makosh_hub_backend::workflows::project_link_review_effects::{
    PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, PROJECT_LINK_REVIEW_EVENT_TYPE,
    project_link_review_effect_event,
};

#[tokio::test]
async fn signal_hub_restores_canonical_sources_idempotently() {
    let ctx = TestContext::new().await;
    let store = SignalHubStore::new(ctx.pool().clone());

    let first = store
        .restore_system_sources()
        .await
        .expect("first fixture restore");
    let second = store
        .restore_system_sources()
        .await
        .expect("second fixture restore");

    assert_eq!(first.sources_created, 14);
    assert_eq!(first.profiles_created, 4);
    assert_eq!(second.sources_created, 0);
    assert_eq!(second.sources_repaired, 0);
    assert_eq!(second.profiles_created, 0);
    assert_eq!(second.profiles_repaired, 0);

    let sources = store.list_sources().await.expect("list sources");
    let source_codes: Vec<_> = sources.iter().map(|source| source.code.as_str()).collect();

    assert_eq!(
        source_codes,
        vec![
            "ai",
            "browser",
            "calendar",
            "filesystem",
            "fixture",
            "github",
            "home_assistant",
            "mail",
            "rss",
            "system",
            "telegram",
            "voice",
            "whatsapp",
            "zoom",
        ]
    );

    let telegram = sources
        .iter()
        .find(|source| source.code == "telegram")
        .expect("telegram source exists");

    assert!(telegram.default_enabled);
    assert!(telegram.supports_connections);
    assert!(telegram.supports_runtime);
    assert!(telegram.supports_pause);
    assert!(telegram.supports_mute);

    let profiles = store.list_profiles().await.expect("list profiles");
    let profile_codes: Vec<_> = profiles
        .iter()
        .map(|profile| profile.code.as_str())
        .collect();
    assert_eq!(
        profile_codes,
        vec!["development", "maintenance", "production", "testing"]
    );
}

#[test]
fn signal_policy_evaluator_applies_reject_pause_mute_allow_order() {
    let now = Utc::now();
    let policies = vec![
        SignalPolicy {
            scope: SignalPolicyScope::Global,
            source_code: None,
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Muted,
            reason: "maintenance window".to_owned(),
            expires_at: None,
        },
        SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Paused,
            reason: "review telegram backlog".to_owned(),
            expires_at: None,
        },
        SignalPolicy {
            scope: SignalPolicyScope::EventPattern,
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: Some("signal.raw.telegram.message.observed".to_owned()),
            mode: SignalPolicyMode::Disabled,
            reason: "reject telegram messages".to_owned(),
            expires_at: None,
        },
        SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Disabled,
            reason: "expired source policy".to_owned(),
            expires_at: Some(now - Duration::minutes(5)),
        },
    ];

    let decision = SignalPolicyEvaluator::new(now).decide(
        "telegram",
        None,
        "signal.raw.telegram.message.observed",
        &policies,
    );

    assert_eq!(
        decision,
        SignalPolicyDecision::Rejected {
            reason: "reject telegram messages".to_owned()
        }
    );

    let decision = SignalPolicyEvaluator::new(now).decide(
        "telegram",
        None,
        "signal.raw.telegram.typing.observed",
        &policies,
    );

    assert_eq!(
        decision,
        SignalPolicyDecision::Paused {
            reason: "review telegram backlog".to_owned()
        }
    );

    let decision = SignalPolicyEvaluator::new(now).decide(
        "mail",
        None,
        "signal.raw.mail.message.observed",
        &policies,
    );

    assert_eq!(
        decision,
        SignalPolicyDecision::Muted {
            reason: "maintenance window".to_owned()
        }
    );
}

#[tokio::test]
async fn event_store_queries_signal_events_by_type_source_subject_correlation_and_time() {
    let ctx = TestContext::new().await;
    let store = EventStore::new(ctx.pool().clone());
    let occurred_at = Utc::now();

    let telegram = NewEventEnvelope::builder(
        format!(
            "evt_signal_query_telegram_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-message-1"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "message-1"
        }),
    )
    .payload(json!({"summary": "metadata only"}))
    .correlation_id("corr-signal-query")
    .build()
    .expect("valid telegram signal");

    let mail = NewEventEnvelope::builder(
        format!(
            "evt_signal_query_mail_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.mail.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "mail",
            "source_id": "mail-message-1"
        }),
        json!({
            "kind": "signal",
            "source_code": "mail",
            "entity_id": "message-2"
        }),
    )
    .payload(json!({"blob_ref": "mail/blob/ref"}))
    .correlation_id("corr-other-signal-query")
    .build()
    .expect("valid mail signal");

    store.append(&telegram).await.expect("append telegram");
    store.append(&mail).await.expect("append mail");

    let queried = store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.raw.telegram.message.observed")
                .source_code("telegram")
                .subject_kind("signal")
                .subject_entity_id("message-1")
                .correlation_id("corr-signal-query")
                .occurred_between(
                    occurred_at - Duration::seconds(1),
                    occurred_at + Duration::seconds(1),
                )
                .limit(10),
        )
        .await
        .expect("query signal events");

    assert_eq!(queried.len(), 1);
    assert_eq!(queried[0].event.event_id, telegram.event_id);
}

#[tokio::test]
async fn signal_hub_accepts_raw_signal_when_no_policy_blocks_it() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");
    let event_store = EventStore::new(ctx.pool().clone());
    let service = SignalHubSignalService::new(signal_store, event_store.clone());
    let occurred_at = Utc::now();
    let raw = NewEventEnvelope::builder(
        format!(
            "evt_raw_accept_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-message-accepted"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-message-accepted"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-message-accepted"}))
    .correlation_id("corr-raw-accept")
    .build()
    .expect("valid raw signal");

    event_store
        .append_for_dispatch(&raw)
        .await
        .expect("append raw");
    let raw_event = event_store
        .get_by_id(&raw.event_id)
        .await
        .expect("load raw")
        .expect("raw exists");

    let outcome = service
        .process_raw_signal(&raw_event)
        .await
        .expect("process raw signal");

    assert!(matches!(outcome, SignalProcessingOutcome::Accepted { .. }));

    let accepted = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.accepted.telegram.message")
                .source_code("telegram")
                .correlation_id("corr-raw-accept")
                .limit(10),
        )
        .await
        .expect("query accepted signal");

    assert_eq!(accepted.len(), 1);
    assert_eq!(
        accepted[0].event.causation_id.as_deref(),
        Some(raw.event_id.as_str())
    );
    assert_eq!(accepted[0].event.payload, raw.payload);
}

#[tokio::test]
async fn signal_hub_pause_policy_buffers_raw_signal_without_accepted_publication() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");
    signal_store
        .create_policy(&SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Paused,
            reason: "manual pause".to_owned(),
            expires_at: None,
        })
        .await
        .expect("create pause policy");

    let event_store = EventStore::new(ctx.pool().clone());
    let service = SignalHubSignalService::new(signal_store.clone(), event_store.clone());
    let occurred_at = Utc::now();
    let raw = NewEventEnvelope::builder(
        format!(
            "evt_raw_pause_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        jso
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/signal_hub_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/signal_hub_api.rs`
- Size bytes / Размер в байтах: `62066`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use chrono::Utc;
use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionPort, CommunicationProviderAccountStore, CommunicationProviderKind,
    NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, project_accepted_signal_if_runtime_allows,
};
use makosh_hub_backend::domains::signal_hub::dispatch_telegram_raw_signal;
use serde_json::Value;
use testkit::app::{TestApp, delete, get, patch_json, post_json};
use testkit::context::TestContext;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn signal_hub_api_restores_fixture_and_lists_sources() {
    let app = TestApp::new().await;
    let router = app.clone_router();

    let restore_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/fixtures/system/restore",
            serde_json::json!({}),
        ))
        .await
        .expect("restore request");

    assert_eq!(restore_response.status(), StatusCode::OK);

    let response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/sources"))
        .await
        .expect("sources request");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    let body: Value = serde_json::from_slice(&bytes).expect("json body");
    let codes: Vec<&str> = body["items"]
        .as_array()
        .expect("items array")
        .iter()
        .map(|item| item["code"].as_str().expect("source code"))
        .collect();

    assert_eq!(
        codes,
        vec![
            "ai",
            "browser",
            "calendar",
            "filesystem",
            "fixture",
            "github",
            "home_assistant",
            "mail",
            "rss",
            "system",
            "telegram",
            "voice",
            "whatsapp",
            "zoom",
        ]
    );

    let emit_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/fixtures/fixture_basic_message/emit",
            serde_json::json!({}),
        ))
        .await
        .expect("emit fixture request");
    assert_eq!(emit_response.status(), StatusCode::OK);
    let emit_body = to_bytes(emit_response.into_body(), usize::MAX)
        .await
        .expect("emit fixture body");
    let emit_json: Value = serde_json::from_slice(&emit_body).expect("emit fixture json");
    assert_eq!(emit_json["item"]["fixture_id"], "fixture_basic_message");
    assert_eq!(emit_json["item"]["source_code"], "fixture");

    let fixture_list_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/fixtures"))
        .await
        .expect("fixture list request");
    assert_eq!(fixture_list_response.status(), StatusCode::OK);
    let fixture_list_body = to_bytes(fixture_list_response.into_body(), usize::MAX)
        .await
        .expect("fixture list body");
    let fixture_list_json: Value =
        serde_json::from_slice(&fixture_list_body).expect("fixture list json");
    let fixture = fixture_list_json["items"]
        .as_array()
        .expect("fixture items")
        .iter()
        .find(|item| item["fixture_id"] == "fixture_basic_message")
        .expect("fixture basic message in list");
    assert_eq!(fixture["source_code"], "fixture");
    assert_eq!(fixture["event_type"], "signal.raw.fixture.message.observed");
    assert_eq!(fixture["summary"], "Fixture message");

    let profiles_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/profiles"))
        .await
        .expect("profiles request");
    assert_eq!(profiles_response.status(), StatusCode::OK);
    let profiles_body = to_bytes(profiles_response.into_body(), usize::MAX)
        .await
        .expect("profiles body");
    let profiles_json: Value = serde_json::from_slice(&profiles_body).expect("profiles json");
    assert!(
        profiles_json["items"]
            .as_array()
            .expect("profile items")
            .iter()
            .any(|item| item["code"] == "testing")
    );

    let apply_profile_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/profiles/testing/apply",
            serde_json::json!({}),
        ))
        .await
        .expect("apply profile request");
    assert_eq!(apply_profile_response.status(), StatusCode::OK);
    let apply_profile_body = to_bytes(apply_profile_response.into_body(), usize::MAX)
        .await
        .expect("apply profile body");
    let apply_profile_json: Value =
        serde_json::from_slice(&apply_profile_body).expect("apply profile json");
    assert_eq!(apply_profile_json["code"], "testing");
    assert_eq!(apply_profile_json["is_active"], true);

    let capabilities_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/capabilities?source_code=telegram"))
        .await
        .expect("capabilities request");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = to_bytes(capabilities_response.into_body(), usize::MAX)
        .await
        .expect("capabilities body");
    let capabilities_json: Value =
        serde_json::from_slice(&capabilities_body).expect("capabilities json");
    assert!(
        capabilities_json["items"]
            .as_array()
            .expect("capability items")
            .iter()
            .any(|item| item["capability"] == "runtime.replay")
    );
    assert!(
        capabilities_json["items"]
            .as_array()
            .expect("capability items")
            .iter()
            .all(|item| item["state"] == "degraded")
    );

    let disable_source_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/sources/telegram/disable",
            serde_json::json!({}),
        ))
        .await
        .expect("disable source request");
    assert_eq!(disable_source_response.status(), StatusCode::OK);

    let blocked_capabilities_response = router
        .clone()
        .oneshot(get("/api/v1/signal-hub/capabilities?source_code=telegram"))
        .await
        .expect("blocked capabilities request");
    assert_eq!(blocked_capabilities_response.status(), StatusCode::OK);
    let blocked_capabilities_body = to_bytes(blocked_capabilities_response.into_body(), usize::MAX)
        .await
        .expect("blocked capabilities body");
    let blocked_capabilities_json: Value =
        serde_json::from_slice(&blocked_capabilities_body).expect("blocked capabilities json");
    assert!(
        blocked_capabilities_json["items"]
            .as_array()
            .expect("blocked capability items")
            .iter()
            .any(|item| item["capability"] == "runtime.replay" && item["state"] == "blocked")
    );

    let create_profile_response = router
        .clone()
        .oneshot(post_json(
            "/api/v1/signal-hub/profiles",
            serde_json::json!({
                "code": "quiet_hours",
                "display_name": "Quiet Hours",
                "description": "Mute noisy overnight signals",
                "source_policies": [
                    {
                        "scope": "source",
                        "source_code": "telegram",
                        "mode": "muted",
                        "reason": "night mute"
                    }
                ]
            }),
        ))
        .await
        .expect("create profile request");
    assert_eq!(create_profile_response.status(), StatusCode::OK);
    let create_profile_body = to_bytes(create_profile_response.into_body(), usize::MAX)
        .await
        .expect("create profile body");
    let create_profile_json: Value =
        serde_json::from_slice(&create_profile_body).expect("create profile json");
    assert_eq!(create_profile_json["code"], "quiet_hours");
    assert_eq!(
        create_profile_json["source_policies"][0]["source_code"],
        "telegram"
    );

    let update_profile_response = router
        .clone()
        .oneshot(patch_json(
            "/api/v1/signal-hub/profiles/quiet_hours",
            serde_json::json!({
                "description": "Updated quiet profile",
                "source_policies": [
                    {
                        "scope": "event_pattern",
                        "event_pattern": "signal.raw.mail.*",
                        "mode": "paused",
                        "reason": "overnight mail pause"
                    }
                ]
            }),
        ))
        .await
        .expect("update profile request");
    assert_eq!(update_profile_response.status(), StatusCode::OK);
    let update_profile_body = to_bytes(update_profile_response.into_body(), usize::MAX)
        .await
        .expect("update profile body");
    let update_profile_json: Value =
        serde_json::from_slice(&update_profile_body).expect("update profile json");
    assert_eq!(update_profile_json["description"], "Updated quiet profile");
    assert_eq!(
        update_profile_json["source_policies"][0]["event_pattern"],
        "signal.raw.mail.*"
    );

    let delete_profile_response = router
        .clone()
        .oneshot(delete("/api/v1/signal-hub/profiles/quiet_hours"))
        .await
        .expect("delete profile request");
    assert_eq!(delete_profile_response.status(), StatusCode::OK);
    let delete_profile_body = to_bytes(delete_profile_response.into_body(), usize::MAX)
        .await
        .expect("delete profile body");
    let delete_profile_json: Value =
        serde_json::from_slice(&delete_profile_body).expect("delete profile json");
    assert_eq!(delete_profile_json["code"], "quiet_hours");

    let system_profile_update_response = router
        .clone()
        .oneshot(patch_json(
            "/api/v1/signal-hub/profiles/testing",
            serde_json::json!({
                "description": "should fail"
            }),
        ))
        .await
        .expect("system profile update request");
    assert_eq!(
        system_profile_update_response.status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn signal_hub_connect_api_requires_local_api_secret() {
    let app = TestApp::new().await;
    let router = app.clone_router();

    let forbidden_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/makosh.signal_hub.v1.SignalHubService/ListSources")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .expect("connect request without secret"),
        )
        .await
        .expect("connect response without secret");
    assert_eq!(forbidden_response.status(), StatusCode::FORBIDDEN);
    let forbidden_body = to_bytes(forbidden_response.into_body(), usize::MAX)
        .await
        .expect("forbidden body");
    let forbidden_json: Value =
        serde_json::from_slice(&forbidden_body).expect("forbidden json body");
    assert_eq!(forbidden_json["error"], "invalid_api_secret");

    let allowed_response = router
        .clone()
        .oneshot(post_json(
            "/makosh.signal_hub.v1.SignalHubService/ListSources",
            serde_json::json!({}),
        ))
        .await
        .expect("connect response with secret");
    assert_eq!(allowed_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn signal_hub_api_runs_ai_health_check_against_runtime_status() {
    let ctx = TestContext::new().await;
    let config = ctx
        .app_config("makosh-test-api-secret")
        .with_test_pairs([("MAKOSH_OLLAMA_BASE_URL", "http://127.0.0.1:9")])
        .expect("ai runtime test config");
    let router = build_router_with_da
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/snapshot_smoke.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/snapshot_smoke.rs`
- Size bytes / Размер в байтах: `687`
- Included characters / Включено символов: `687`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

#[test]
fn event_payload_snapshot_remains_stable() {
    let payload = json!({
        "event_type": "signal.accepted.telegram.message",
        "source": {
            "kind": "integration",
            "provider": "telegram",
            "source_id": "msg-42"
        },
        "subject": {
            "kind": "communication",
            "entity_id": "thread-7"
        },
        "metadata": {
            "priority": "normal",
            "channel": "telegram"
        }
    });

    insta::assert_snapshot!(
        "event_payload_snapshot_remains_stable",
        serde_json::to_string_pretty(&payload).expect("snapshot payload must serialize"),
    );
}
```

### `backend/tests/snapshots/snapshot_smoke__event_payload_snapshot_remains_stable.snap`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/snapshots/snapshot_smoke__event_payload_snapshot_remains_stable.snap`
- Size bytes / Размер в байтах: `450`
- Included characters / Включено символов: `450`
- Truncated / Обрезано: `no`

```text
---
source: backend/tests/snapshot_smoke.rs
expression: "serde_json::to_string_pretty(&payload).expect(\"snapshot payload must serialize\")"
---
{
  "event_type": "signal.accepted.telegram.message",
  "metadata": {
    "channel": "telegram",
    "priority": "normal"
  },
  "source": {
    "kind": "integration",
    "provider": "telegram",
    "source_id": "msg-42"
  },
  "subject": {
    "entity_id": "thread-7",
    "kind": "communication"
  }
}
```

### `backend/tests/storage.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/storage.rs`
- Size bytes / Размер в байтах: `2544`
- Included characters / Включено символов: `2544`
- Truncated / Обрезано: `no`

```rust
use testkit::context::TestContext;

use chrono::{DateTime, Utc};
use makosh_hub_backend::platform::storage::{Database, ReadinessStatus};

#[tokio::test]
async fn database_without_url_reports_not_configured() {
    let database = Database::connect(None).await.expect("disabled database");

    let readiness = database.readiness().await;

    assert_eq!(readiness.status(), ReadinessStatus::NotConfigured);
    assert!(!readiness.message().is_empty());
}

#[tokio::test]
async fn database_without_url_reports_migrations_not_configured() {
    let database = Database::connect(None).await.expect("disabled database");

    let readiness = database.migration_readiness().await;

    assert_eq!(readiness.status(), ReadinessStatus::NotConfigured);
    assert!(!readiness.message().is_empty());
}

#[tokio::test]
async fn migration_readiness_rejects_missing_latest_migration_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool");

    let migration: (i64, String, DateTime<Utc>, bool, Vec<u8>, i64) = sqlx::query_as(
        r#"
        SELECT version, description, installed_on, success, checksum, execution_time
        FROM _sqlx_migrations
        ORDER BY version DESC
        LIMIT 1
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("latest sqlx migration");

    sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
        .bind(migration.0)
        .execute(pool)
        .await
        .expect("delete latest sqlx migration record");

    let readiness = database.migration_readiness().await;

    sqlx::query(
        r#"
        INSERT INTO _sqlx_migrations (
            version,
            description,
            installed_on,
            success,
            checksum,
            execution_time
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(migration.0)
    .bind(migration.1)
    .bind(migration.2)
    .bind(migration.3)
    .bind(migration.4)
    .bind(migration.5)
    .execute(pool)
    .await
    .expect("restore latest sqlx migration record");

    assert!(
        migration.0 >= 4,
        "test requires actor identity migration to exist"
    );
    assert_eq!(readiness.status(), ReadinessStatus::Unavailable);
    assert_eq!(
        readiness.message(),
        "required database migrations are incomplete"
    );
}
```

### `backend/tests/task_candidates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/task_candidates.rs`
- Size bytes / Размер в байтах: `216`
- Included characters / Включено символов: `216`
- Truncated / Обрезано: `no`

```rust
#[path = "task_candidates/event_replay.rs"]
mod event_replay;
#[path = "task_candidates/refresh.rs"]
mod refresh;
#[path = "task_candidates/review.rs"]
mod review;
#[path = "task_candidates/support.rs"]
mod support;
```

### `backend/tests/task_candidates/event_replay.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/task_candidates/event_replay.rs`
- Size bytes / Размер в байтах: `3346`
- Included characters / Включено символов: `3346`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::tasks::candidates::TaskCandidateReviewState;

use super::support::{
    build_review_event, live_task_candidate_context, seed_message, unique_suffix,
};

#[tokio::test]
async fn task_candidate_review_event_rebuilds_state_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("event-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-event-{suffix}"),
        &format!("Task event {suffix}"),
        "Action: verify integration",
    )
    .await;

    let _ = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'observation'",
    )
    .bind(&message_observation_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate id");

    let confirm_event = build_review_event(
        &task_candidate_id,
        TaskCandidateReviewState::UserConfirmed,
        "event-reviewer",
        &format!("task-candidate-event-confirm-{suffix}"),
    );
    let reject_event = build_review_event(
        &task_candidate_id,
        TaskCandidateReviewState::UserRejected,
        "event-reviewer",
        &format!("task-candidate-event-reject-{suffix}"),
    );

    context
        .event_store
        .append(&confirm_event)
        .await
        .expect("append confirm event");
    context
        .event_store
        .append(&reject_event)
        .await
        .expect("append reject event");

    let confirm_event = context
        .event_store
        .get_by_id(&confirm_event.event_id)
        .await
        .expect("load confirm event")
        .expect("confirm event exists");
    context
        .store
        .apply_review_event(&confirm_event)
        .await
        .expect("apply confirm event");
    let reject_event = context
        .event_store
        .get_by_id(&reject_event.event_id)
        .await
        .expect("load reject event")
        .expect("reject event exists");
    context
        .store
        .apply_review_event(&reject_event)
        .await
        .expect("apply reject event");

    let state: String =
        sqlx::query_scalar("SELECT review_state FROM task_candidates WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("load state");
    assert_eq!(state, "user_rejected");

    let event_id: String =
        sqlx::query_scalar("SELECT event_id FROM task_candidates WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("load event id");
    assert_eq!(
        event_id,
        format!("task_candidate_review:task-candidate-event-reject-{suffix}")
    );
}
```

### `backend/tests/task_candidates/refresh.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/task_candidates/refresh.rs`
- Size bytes / Размер в байтах: `10326`
- Included characters / Включено символов: `10326`
- Truncated / Обрезано: `no`

```rust
use super::support::{live_task_candidate_context, seed_document, seed_message, unique_suffix};

#[tokio::test]
async fn task_candidate_refresh_creates_message_and_document_candidates_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("TaskCandidate{suffix}");

    let message_id = seed_message(
        &context,
        suffix,
        &format!("sender-{suffix}@example.com"),
        &[format!("recipient-{suffix}@example.com")],
        &format!("provider-task-candidate-msg-{suffix}"),
        &format!("{keyword} Update"),
        "Please action: schedule sync call",
    )
    .await;
    let document_id = seed_document(
        &context.pool,
        &format!("document_task_candidate_{suffix}"),
        &format!("{keyword} architecture"),
        "Follow up: draft document",
    )
    .await;
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");
    let document_observation_id: String =
        sqlx::query_scalar("SELECT observation_id FROM documents WHERE document_id = $1")
            .bind(&document_id)
            .fetch_one(&context.pool)
            .await
            .expect("document observation id");

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh candidates");
    assert!(refreshed >= 2);

    let message_rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT task_candidate_id, source_kind, review_state
        FROM task_candidates
        WHERE source_id = $1
        "#,
    )
    .bind(&message_observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("message candidate rows");
    assert_eq!(
        message_rows.len(),
        1,
        "should persist deterministic message candidate"
    );
    assert_eq!(message_rows[0].1, "observation");
    assert_eq!(message_rows[0].2, "suggested");
    let message_observation_id: Option<String> = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM task_candidates
        WHERE source_id = $1
          AND source_kind = 'observation'
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&context.pool)
    .await
    .expect("message candidate observation id");
    assert!(
        message_observation_id.is_some(),
        "message candidates must carry canonical observation evidence"
    );

    let document_rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT task_candidate_id, source_kind, review_state
        FROM task_candidates
        WHERE source_id = $1
        "#,
    )
    .bind(&document_observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("document candidate rows");
    assert_eq!(
        document_rows.len(),
        1,
        "should persist deterministic document candidate"
    );
    assert_eq!(document_rows[0].1, "observation");
}

#[tokio::test]
async fn task_candidate_refresh_uses_obligation_engine_for_message_commitments_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("send the redlined agreement {suffix}");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("commitment-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-obligation-{suffix}"),
        &format!("Obligation engine {suffix}"),
        &format!("I will {statement} by Friday 5pm."),
    )
    .await;

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    assert!(refreshed >= 1);
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");

    let rows: Vec<(String, String, Option<String>, f64, String)> = sqlx::query_as(
        r#"
        SELECT title, review_state, due_text, confidence, evidence_excerpt
        FROM task_candidates
        WHERE source_id = $1
          AND source_kind = 'observation'
        "#,
    )
    .bind(&message_observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("message candidate rows");

    assert_eq!(
        rows.len(),
        1,
        "commitment language should create one reviewable task candidate"
    );
    assert_eq!(rows[0].0, statement);
    assert_eq!(rows[0].1, "suggested");
    assert_eq!(rows[0].2.as_deref(), Some("Friday 5pm"));
    assert!(rows[0].3 > 0.7);
    assert_eq!(rows[0].4, format!("I will {statement} by Friday 5pm."));

    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&message_id)
            .fetch_one(&context.pool)
            .await
            .expect("task count");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE statement = $1")
            .bind(&statement)
            .fetch_one(&context.pool)
            .await
            .expect("accepted obligation count");

    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn task_candidate_refresh_uses_obligation_engine_for_document_commitments_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("send the document-backed commitment {suffix}");
    let document_id = seed_document(
        &context.pool,
        &format!("document_obligation_candidate_{suffix}"),
        &format!("Document obligation {suffix}"),
        &format!("I will {statement} by Friday 5pm."),
    )
    .await;

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    assert!(refreshed >= 1);
    let document_observation_id: String =
        sqlx::query_scalar("SELECT observation_id FROM documents WHERE document_id = $1")
            .bind(&document_id)
            .fetch_one(&context.pool)
            .await
            .expect("document observation id");

    let rows: Vec<(String, String, String, Option<String>, f64, String)> = sqlx::query_as(
        r#"
        SELECT title, review_state, candidate_kind, due_text, confidence, evidence_excerpt
        FROM task_candidates
        WHERE source_id = $1
          AND source_kind = 'observation'
          AND candidate_kind = 'obligation_task'
        "#,
    )
    .bind(&document_observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("document obligation candidate rows");

    assert_eq!(
        rows.len(),
        1,
        "document commitment language should create one reviewable obligation-derived task candidate"
    );
    assert_eq!(rows[0].0, statement);
    assert_eq!(rows[0].1, "suggested");
    assert_eq!(rows[0].2, "obligation_task");
    assert_eq!(rows[0].3.as_deref(), Some("Friday 5pm"));
    assert!(rows[0].4 > 0.7);
    assert_eq!(rows[0].5, format!("I will {statement} by Friday 5pm."));

    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&document_id)
            .fetch_one(&context.pool)
            .await
            .expect("task count");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE statement = $1")
            .bind(&statement)
            .fetch_one(&context.pool)
            .await
            .expect("accepted obligation count");

    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn task_candidate_refresh_updates_existing_source_title_candidate_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("source-title-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-source-title-{suffix}"),
        &format!("Source title conflict {suffix}"),
        "Action: Review This Item",
    )
    .await;
    let observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");

    sqlx::query(
        r#"
        INSERT INTO task_candidates (
            task_candidate_id,
            source_kind,
            source_id,
            observation_id,
            candidate_kind,
            candidate_metadata,
            title,
            confidence,
            review_state,
            evidence_excerpt
        )
        VALUES ($1, 'observation', $2, $3, 'task', '{}'::jsonb, $4, 0.5, 'suggested', $5)
        "#,
    )
    .bind(format!("task_candidate:v1:legacy-source-title:{suffix}"))
    .bind(&observation_id)
    .bind(&observation_id)
    .bind("action: review this item")
    .bind("legacy evidence")
    .execute(&context.pool)
    .await
    .expect("legacy candidate");

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh should update source/title candidate without duplicate-key failure");
    assert!(refreshed >= 1);

    let rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT task_candidate_id, title, evidence_excerpt
        FROM task_candidates
        WHERE source_kind = 'observation' AND source_id = $1
        "#,
    )
    .bind(&observation_id)
    .fetch_all(&context.pool)
    .await
    .expect("candidate rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1, "Action: Review This Item");
    assert_eq!(rows[0].2, "Action: Review This Item");
}
```

### `backend/tests/task_candidates/review.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/task_candidates/review.rs`
- Size bytes / Размер в байтах: `18268`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use makosh_hub_backend::domains::tasks::candidates::{
    TaskCandidateReviewCommand, TaskCandidateReviewState,
};
use makosh_hub_backend::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore,
};
use serde_json::json;
use sqlx::Row;

use super::support::{live_task_candidate_context, seed_message, unique_suffix};

#[tokio::test]
async fn task_candidate_review_confirm_creates_active_task_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("confirm-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-confirm-{suffix}"),
        &format!("Task confirm {suffix}"),
        "Action: review this item",
    )
    .await;

    let _ = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'observation'",
    )
    .bind(&message_observation_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate id");
    let candidate_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM task_candidates WHERE task_candidate_id = $1",
    )
    .bind(&task_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate observation id");

    let result = context
        .store
        .set_review_state(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-confirm-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm");
    assert_eq!(result.review_state, TaskCandidateReviewState::UserConfirmed);
    assert_eq!(result.task_candidate_id, task_candidate_id);

    let task_row: (String, String, String, String, String, String) = sqlx::query_as(
        r#"
        SELECT task_id, provenance_kind, provenance_id, source_kind, source_id, source_type
        FROM tasks
        WHERE task_candidate_id = $1
        "#,
    )
    .bind(&task_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("task row");
    assert_eq!(task_row.1, "observation");
    assert_eq!(task_row.2, candidate_observation_id);
    assert_eq!(task_row.3, "observation");
    assert_eq!(task_row.4, task_row.2);
    assert_eq!(task_row.5, "observation");

    let observation_row: (String, String) = sqlx::query_as(
        r#"
        SELECT source_ref, kind.code
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&task_row.2)
    .fetch_one(&context.pool)
    .await
    .expect("provenance observation");
    assert!(!observation_row.0.trim().is_empty());
    assert_eq!(observation_row.1, "COMMUNICATION_MESSAGE");
}

#[tokio::test]
async fn task_candidate_store_review_with_observation_materializes_transition_link_against_postgres()
 {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("confirm-link-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-confirm-link-{suffix}"),
        &format!("Task confirm link {suffix}"),
        "Action: review this link owner path",
    )
    .await;

    let _ = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'observation'",
    )
    .bind(&message_observation_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate id");
    let review_observation = ObservationStore::new(context.pool.clone())
        .capture(
            &NewObservation::new(
                "REVIEW_TRANSITION",
                ObservationOriginKind::Manual,
                chrono::Utc::now(),
                json!({
                    "task_candidate_id": task_candidate_id,
                    "operation": "task_candidate_review",
                }),
                format!("manual://task-candidate-review/{suffix}"),
            )
            .provenance(json!({
                "source": "task_candidates.review.test",
            })),
        )
        .await
        .expect("capture review observation");

    let result = context
        .store
        .set_review_state_with_observation(
            &TaskCandidateReviewCommand {
                command_id: format!("task-candidate-confirm-link-{suffix}"),
                task_candidate_id: task_candidate_id.clone(),
                review_state: TaskCandidateReviewState::UserConfirmed,
                actor_id: "tests-reviewer".to_owned(),
            },
            &review_observation.observation_id,
            json!({
                "source": "task_candidates.review.test",
            }),
        )
        .await
        .expect("confirm");
    assert_eq!(result.review_state, TaskCandidateReviewState::UserConfirmed);

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task_candidate'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&task_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("task candidate review transition link");
    let linked_observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: serde_json::Value = link_row.try_get("metadata").expect("metadata");
    assert_eq!(linked_observation_id, review_observation.observation_id);
    assert_eq!(metadata["review_state"], json!("user_confirmed"));
    assert_eq!(
        metadata["event_id"],
        json!(format!(
            "task_candidate_review:task-candidate-confirm-link-{suffix}"
        ))
    );
}

#[tokio::test]
async fn task_candidate_review_confirm_materializes_obligation_candidate_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("send the countersigned agreement {suffix}");
    let quote = format!("I will {statement} by Friday 5pm.");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("obligation-confirm-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-obligation-confirm-{suffix}"),
        &format!("Obligation confirm {suffix}"),
        &quote,
    )
    .await;

    let _ = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'observation'",
    )
    .bind(&message_observation_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate id");
    let candidate_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM task_candidates WHERE task_candidate_id = $1",
    )
    .bind(&task_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate observation id");

    let result = context
        .store
        .set_review_state(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-obligation-confirm-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm");
    assert_eq!(result.review_state, TaskCandidateReviewState::UserConfirmed);

    let task_id: String =
        sqlx::query_scalar("SELECT task_id FROM tasks WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("task id");

    let obligation_rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT
            o.obligation_id,
            o.review_state,
            o.obligated_entity_kind,
            o.obligated_entity_id,
            l.link_kind
        FROM obligations o
        JOIN obligation_task_links l
          ON l.obligation_id = o.obligation_id
        WHERE l.task_id = $1
        ORDER BY o.obligation_id
        "#,
    )
    .bind(&task_id)
    .fetch_all(&context.pool)
    .await
    .expect("linked obligation rows");
    assert_eq!(
        obligation_rows.len(),
        1,
        "confirming an obligation-derived candidate should create one linked obligation"
    );
    assert_eq!(obligation_rows[0].1, "user_confirmed");
    assert_eq!(obligation_rows[0].2, "persona");
    assert_eq!(obligation_rows[0].3, "persona:owner");
    assert_eq!(obligation_rows[0].4, "fulfillment_task");

    let evidence_row: (String, String, Option<String>, Option<String>) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, quote, observation_id
        FROM obligation_evidence
        WHERE obligation_id = $1
        "#,
    )
    .bind(&obligation_rows[0].0)
    .fetch_one(&context.pool)
    .await
    .expect("obligation evidence");
    assert_eq!(evidence_row.0, "observation");
    assert_eq!(evidence_row.1, candidate_observation_id);
    assert_eq!(evidence_row.2.as_deref(), Some(quote.as_str()));
    assert_eq!(evidence_row.3.as_deref(), Some(evidence_row.1.as_str()));
}

#[tokio::test]
async fn obligation_task_candidate_reset_demotes_obligation_review_state_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("send the vendor approval memo {suffix}");
    let quote = format!("I will {statement} by Friday 5pm.");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("obligation-reset-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-obligation-reset-{suffix}"),
        &format!("Obligation reset {suffix}"),
        &quote,
    )
    .await;

    let _ = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");
    let task_cand
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/task_candidates/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/task_candidates/support.rs`
- Size bytes / Размер в байтах: `4632`
- Included characters / Включено символов: `4632`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use makosh_hub_backend::domains::tasks::candidates::{
    TaskCandidateReviewState, TaskCandidateStore,
};
use makosh_hub_backend::platform::events::{EventStore, NewEventEnvelope};
use makosh_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::postgres::PgPool;

const TASK_CANDIDATE_REVIEW_EVENT_TYPE: &str = "task_candidate.review_state_changed";

pub(crate) struct TaskCandidateTestContext {
    pub(crate) pool: PgPool,
    pub(crate) store: TaskCandidateStore,
    pub(crate) event_store: EventStore,
}

pub(crate) async fn live_task_candidate_context() -> Option<TaskCandidateTestContext> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    task_candidate_context(&database_url).await
}

async fn task_candidate_context(database_url: &str) -> Option<TaskCandidateTestContext> {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some(TaskCandidateTestContext {
        store: TaskCandidateStore::new(pool.clone()),
        event_store: EventStore::new(pool.clone()),
        pool,
    })
}

pub(crate) async fn seed_message(
    context: &TaskCandidateTestContext,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_task_candidate_{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(context.pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Task Candidate Gmail",
            format!("task-candidate-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");

    let raw_record_id = format!("raw_task_candidate_{suffix}_{provider_record_id}");
    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:task-candidate:{suffix}:{provider_record_id}"),
                format!("batch-task-candidate-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"task_candidate_test"})),
        )
        .await
        .expect("raw message");

    let message_store = MessageProjectionStore::new(context.pool.clone());
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

pub(crate) async fn seed_document(
    pool: &PgPool,
    document_id: &str,
    title: &str,
    body: &str,
) -> String {
    let import = NewDocumentImport::markdown(document_id, title, body);
    DocumentImportStore::new(pool.clone())
        .import_document(&import)
        .await
        .expect("document import");
    document_id.to_owned()
}

pub(crate) fn build_review_event(
    task_candidate_id: &str,
    review_state: TaskCandidateReviewState,
    actor_id: &str,
    command_id: &str,
) -> NewEventEnvelope {
    NewEventEnvelope::builder(
        format!("task_candidate_review:{command_id}"),
        TASK_CANDIDATE_REVIEW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "task_candidate_review",
            "provider": "local_api",
            "source_id": command_id,
        }),
        json!({"kind": "task_candidate_review"}),
    )
    .actor(json!({"actor_id": actor_id}))
    .payload(json!({
        "task_candidate_id": task_candidate_id,
        "review_state": review_state.as_str(),
    }))
    .build()
    .expect("review event")
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
```

### `backend/tests/task_candidates_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/task_candidates_api.rs`
- Size bytes / Размер в байтах: `13893`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use makosh_hub_backend::domains::tasks::candidates::TaskCandidateStore;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "task-candidates-api-test-token";

#[tokio::test]
async fn task_candidates_reject_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/task-candidates"))
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
async fn task_candidates_returns_safe_candidate_payload() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let keyword = format!("TaskCandidatesApi{suffix}");

    let store = TaskCandidateStore::new(pool.clone());
    let message_id = seed_message(
        &pool,
        suffix,
        &format!("api-message-{suffix}@example.com"),
        &[format!("api-recipient-{suffix}@example.com")],
        &format!("provider-task-candidate-api-msg-{suffix}"),
        &format!("Task API {keyword}"),
        "Please follow up with the client",
    )
    .await;
    let document_id = seed_document(
        &pool,
        &format!("document_task_candidate_api_{suffix}"),
        &format!("{keyword} plan"),
        "Please review this task",
    )
    .await;
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let document_observation_id: String =
        sqlx::query_scalar("SELECT observation_id FROM documents WHERE document_id = $1")
            .bind(&document_id)
            .fetch_one(&pool)
            .await
            .expect("document observation id");
    let _ = store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh candidates");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/task-candidates?limit={}&", 100),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(!items.is_empty());

    let message_payload = items
        .iter()
        .find(|item| item["source_id"] == json!(message_observation_id))
        .expect("message payload");
    let document_payload = items
        .iter()
        .find(|item| item["source_id"] == json!(document_observation_id))
        .expect("document payload");
    assert_eq!(message_payload["source_kind"], "observation");
    assert_eq!(document_payload["source_kind"], "observation");
    assert_eq!(
        message_payload["observation_id"],
        json!(message_observation_id)
    );
    assert_eq!(
        document_payload["observation_id"],
        json!(document_observation_id)
    );
    assert!(message_payload["evidence_excerpt"].is_string());
    assert!(document_payload["evidence_excerpt"].is_string());
    assert!(message_payload.get("candidate_kind").is_none());
    assert!(message_payload.get("candidate_metadata").is_none());
    assert!(document_payload.get("candidate_kind").is_none());
    assert!(document_payload.get("candidate_metadata").is_none());
}

#[tokio::test]
async fn put_task_candidate_review_confirms_task_with_observation_trail() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let store = TaskCandidateStore::new(pool.clone());

    let message_id = seed_message(
        &pool,
        suffix,
        &format!("review-api-{suffix}@example.com"),
        &[format!("api-owner-{suffix}@example.com")],
        &format!("provider-task-candidate-review-api-{suffix}"),
        &format!("Task review API {suffix}"),
        "Action: process this ticket",
    )
    .await;
    let _ = store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh candidates");
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'observation'",
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("candidate id");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let command_id = format!("task-candidate-api-confirm-{suffix}");
    let response = app
        .oneshot(json_put_request_with_actor(
            &format!("/api/v1/task-candidates/{task_candidate_id}/review"),
            json!({
                "command_id": command_id,
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
            "task_candidate_id": task_candidate_id,
            "review_state": "user_confirmed",
            "event_id": format!("task_candidate_review:{command_id}"),
        })
    );

    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM task_candidates WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&pool)
            .await
            .expect("candidate review state");
    assert_eq!(review_state, "user_confirmed");

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task_candidate'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&task_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("task candidate observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: Value = link_row.try_get("metadata").expect("link metadata");
    assert_eq!(metadata["review_state"], "user_confirmed");
    assert_eq!(
        metadata["event_id"],
        json!(format!("task_candidate_review:{command_id}"))
    );

    let observation_row =
        sqlx::query("SELECT origin_kind, payload FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("task candidate observation");
    let origin_kind: String = observation_row.try_get("origin_kind").expect("origin kind");
    let payload: Value = observation_row.try_get("payload").expect("payload");
    assert_eq!(origin_kind, "manual");
    assert_eq!(payload["task_candidate_id"], json!(task_candidate_id));
    assert_eq!(payload["review_state"], "user_confirmed");

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT status, target_entity_kind, target_entity_id
        FROM review_items
        WHERE metadata->>'task_candidate_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&task_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("task candidate review item");
    assert_eq!(review_item.0, "promoted");
    assert_eq!(review_item.1, "task");
    assert!(!review_item.2.is_empty());
}

#[tokio::test]
async fn put_task_candidate_review_rejects_missing_candidate() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let response = app
        .oneshot(json_put_request_with_actor(
            "/api/v1/task-candidates/task_candidate:v1:missing%3Amissing%3Acandidate/review",
            json!({
                "command_id": "task-candidate-missing-review",
                "review_state": "user_rejected",
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
            "error": "task_candidate_not_found",
            "message": "task candidate was not found"
        })
    );
}

#[derive(Clone)]
struct TaskCandidatesApiContext {
    communication_store: CommunicationIngestionStore,
    message_store: MessageProjectionStore,
}

fn config_with_api_token() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
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

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
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
    let context = TaskCandidatesApiContext {
        communication_store: CommunicationIngestionStore::new(pool.clone()),
        message_store: MessageProjectionStore::new(pool.clone()),
    };
    le
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/task_candidates_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/task_candidates_architecture.rs`
- Size bytes / Размер в байтах: `1995`
- Included characters / Включено символов: `1995`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn task_candidate_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_task_candidate_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "task candidate test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_task_candidate_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_task_candidate_test_violations(&path, violations);
            continue;
        }
        if !is_task_candidate_test_file(&path) {
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

fn is_task_candidate_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "task_candidates.rs" || file_name == "task_candidates_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "task_candidates")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/tasks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/tasks.rs`
- Size bytes / Размер в байтах: `42866`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::{Duration, Utc};
use makosh_hub_backend::domains::decisions::{
    DecisionEvidenceSourceKind, DecisionReviewState, DecisionStore, NewDecision,
    NewDecisionEvidence,
};
use makosh_hub_backend::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationEvidenceSourceKind,
    ObligationReviewState, ObligationStore,
};
use makosh_hub_backend::domains::relationships::{
    RelationshipEntityKind, RelationshipEvidenceSourceKind, RelationshipReviewState,
    RelationshipStore,
};
use makosh_hub_backend::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxStore, ReviewItemKind,
};
use makosh_hub_backend::domains::tasks::api::{NewTask, TaskListQuery, TaskStore, TaskUpdate};
use makosh_hub_backend::domains::tasks::brain::TaskBrainService;
use makosh_hub_backend::domains::tasks::core::{
    TaskChecklistStore, TaskContextPackStore, TaskEvidenceStore, TaskProviderStore,
    TaskRelationStore, TaskSubtaskStore,
};
use makosh_hub_backend::domains::tasks::health::TaskWatchtowerService;
use makosh_hub_backend::domains::tasks::intelligence::TaskIntelligenceService;
use makosh_hub_backend::domains::tasks::rules::{TaskRuleStore, TaskTemplateStore};
use makosh_hub_backend::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore,
};
use makosh_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

async fn live_pool() -> Option<PgPool> {
    let test_context = TestContext::new().await;
    let url = test_context.connection_string();
    let db = Database::connect(Some(&url)).await.expect("connect");
    Some(db.pool().expect("pool").clone())
}

fn disconnected_pool() -> PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://x:x@127.0.0.1:1/db")
        .expect("lazy")
}

fn assert_float_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {expected}, got {actual}"
    );
}

// ── Task CRUD ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Test {suffix}"),
            description: Some("desc".into()),
            source_type: Some("manual".into()),
            makosh_status: Some("new".into()),
            priority_score: Some(0.8),
            ..Default::default()
        })
        .await
        .expect("create");
    assert!(task.task_id.starts_with("task:v1:"));
    assert_eq!(task.makosh_status, "new");
    assert_eq!(task.provenance_kind, "observation");
    assert!(task.provenance_id.starts_with("observation:v1:"));
    assert_eq!(task.source_kind, "observation");
    assert_eq!(task.source_type, "observation");
    assert_eq!(task.source_id, task.provenance_id);

    let fetched = store
        .get(&task.task_id)
        .await
        .expect("get")
        .expect("exists");
    assert_float_eq(fetched.priority_score.expect("priority score"), 0.8);

    let updated = store
        .update(
            &task.task_id,
            &TaskUpdate {
                makosh_status: Some("in_progress".into()),
                priority_score: Some(0.9),
                ..Default::default()
            },
        )
        .await
        .expect("update");
    assert_eq!(updated.makosh_status, "in_progress");
    assert_float_eq(updated.priority_score.expect("updated priority score"), 0.9);

    store
        .set_status(&task.task_id, "done")
        .await
        .expect("set status");
    let done = store
        .get(&task.task_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(done.makosh_status, "done");
    assert!(done.completed_at.is_some());

    store.archive(&task.task_id).await.expect("archive");
    let archived = store
        .get(&task.task_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(archived.makosh_status, "archived");
}

#[tokio::test]
async fn task_manual_creation_materializes_explicit_observation_provenance_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Manual provenance {suffix}"),
            description: Some("Task created directly from UI".to_owned()),
            ..Default::default()
        })
        .await
        .expect("create manual task");

    assert_eq!(task.provenance_kind, "observation");
    assert_eq!(task.source_kind, "observation");
    assert_eq!(task.source_type, "observation");
    assert_eq!(task.source_id, task.provenance_id);

    let row = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, observation.payload
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&task.provenance_id)
    .fetch_one(&pool)
    .await
    .expect("task provenance observation");
    let kind_code: String = row.try_get("kind_code").expect("kind code");
    let payload: serde_json::Value = row.try_get("payload").expect("payload");

    assert_eq!(kind_code, "TASK_MUTATION");
    assert_eq!(payload["task_title"], json!(task.title));
    assert_eq!(payload["captured_from"], json!("task_create"));
}

#[tokio::test]
async fn task_store_update_with_observation_materializes_task_link_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let observation_store = ObservationStore::new(pool.clone());
    let store = TaskStore::new(pool.clone());
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Task update source {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");
    let observation = observation_store
        .capture(
            &NewObservation::new(
                "TASK_MUTATION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "task_id": task.task_id,
                    "title": format!("Task update applied {suffix}"),
                }),
                format!("manual://tasks/update/{suffix}"),
            )
            .confidence(0.9),
        )
        .await
        .expect("capture observation");

    let updated = store
        .update_with_observation(
            &task.task_id,
            &TaskUpdate {
                title: Some(format!("Task update applied {suffix}")),
                ..Default::default()
            },
            &observation.observation_id,
            "task_update",
            json!({
                "operation": "update",
            }),
        )
        .await
        .expect("update");
    assert_eq!(updated.title, format!("Task update applied {suffix}"));

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'tasks'
           AND entity_kind = 'task'
           AND entity_id = $2
           AND relationship_kind = 'task_update'",
    )
    .bind(&observation.observation_id)
    .bind(&task.task_id)
    .fetch_one(&pool)
    .await
    .expect("observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn task_creation_rejects_missing_review_item_provenance_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();

    let result = store
        .create(&NewTask {
            title: format!("Orphan review task {suffix}"),
            provenance_kind: Some("review_item".to_owned()),
            provenance_id: Some("review_item:v1:missing".to_owned()),
            source_type: Some("manual".to_owned()),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_err(),
        "task must not be created without existing review item"
    );
}

#[tokio::test]
async fn task_creation_rejects_missing_observation_provenance_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();

    let result = store
        .create(&NewTask {
            title: format!("Orphan observation task {suffix}"),
            provenance_kind: Some("observation".to_owned()),
            provenance_id: Some("observation:v1:missing".to_owned()),
            source_type: Some("manual".to_owned()),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_err(),
        "task must not be created without existing observation"
    );
}

#[tokio::test]
async fn task_creation_from_explicit_observation_provenance_uses_observation_source_against_postgres()
 {
    let Some(pool) = live_pool().await else {
        return;
    };
    let observation_store = ObservationStore::new(pool.clone());
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();
    let observation = observation_store
        .capture(
            &NewObservation::new(
                "DOCUMENT",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "title": format!("Explicit observation source {suffix}"),
                    "body": "Task should inherit observation-backed source fields."
                }),
                format!("manual://task-provenance/{suffix}"),
            )
            .confidence(0.9),
        )
        .await
        .expect("capture observation");

    let task = store
        .create(&NewTask {
            title: format!("Observation provenance task {suffix}"),
            provenance_kind: Some("observation".to_owned()),
            provenance_id: Some(observation.observation_id.clone()),
            ..Default::default()
        })
        .await
        .expect("create task");

    assert_eq!(task.provenance_kind, "observation");
    assert_eq!(task.provenance_id, observation.observation_id);
    assert_eq!(task.source_kind, "observation");
    assert_eq!(task.source_type, "observation");
    assert_eq!(task.source_id, task.provenance_id);
}

#[tokio::test]
async fn task_creation_rejects_missing_decision_provenance_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();

    let result = store
        .create(&NewTask {
            title: format!("Orphan decision task {suffix}"),
            provenance_kind: Some("decision".to_owned()),
            provenance_id: Some("decision:v1:missing".to_owned()),
            source_type: Some("manual".to_owned()),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_err(),
        "task must not be created without existing decision"
    );
}

#[tokio::test]
async fn task_creation_rejects_missing_obligation_provenance_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();

    let result = store
        .create(&NewTask {
            title: format!("Orphan obligation task {suffix}"),
            provenance_kind: Some("obligation".to_owned()),
            provenance_id: Some("obliga
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/tasks_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/tasks_api.rs`
- Size bytes / Размер в байтах: `218`
- Included characters / Включено символов: `218`
- Truncated / Обрезано: `no`

```rust
#[path = "tasks_api/auth.rs"]
mod auth;
#[path = "tasks_api/crud.rs"]
mod crud;
#[path = "tasks_api/mutations.rs"]
mod mutations;
#[path = "tasks_api/reads.rs"]
mod reads;
#[path = "tasks_api/support.rs"]
mod support;
```

### `backend/tests/tasks_api/auth.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/tasks_api/auth.rs`
- Size bytes / Размер в байтах: `500`
- Included characters / Включено символов: `500`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;

#[tokio::test]
async fn tasks_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request("/api/v1/tasks"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({"error": "invalid_api_secret", "message": "missing or invalid x-makosh-secret header"})
    );
}
```

### `backend/tests/tasks_api/crud.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/tasks_api/crud.rs`
- Size bytes / Размер в байтах: `21541`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use crate::support::*;
use makosh_hub_backend::domains::decisions::{
    DecisionEvidenceSourceKind, DecisionReviewState, DecisionStore, NewDecision,
    NewDecisionEvidence,
};
use sqlx::Row;

#[tokio::test]
async fn tasks_crud_against_postgres() {
    let Some(database_url) = test_database_url("tasks CRUD test").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/tasks",
            json!({"title": format!("CRUD Task {suffix}"), "description": "CRUD test", "status": "active"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: task create failed");
        return;
    }
    let created = json_body(response).await;
    let Some(task_id) = created["task_id"].as_str().map(|value| value.to_owned()) else {
        eprintln!("skip: no task_id");
        return;
    };
    assert_eq!(created["title"], json!(format!("CRUD Task {suffix}")));

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/tasks/{}", urlencoding_percent_encode(&task_id)),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let fetched = json_body(response).await;
    assert_eq!(fetched["task_id"], json!(task_id));

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!("/api/v1/tasks/{}", urlencoding_percent_encode(&task_id)),
            json!({"title": format!("Updated Task {suffix}"), "priority": "high"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let updated = json_body(response).await;
    assert_eq!(updated["title"], json!(format!("Updated Task {suffix}")));

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();
    let update_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task'
           AND entity_id = $1
           AND relationship_kind = 'task_update'",
    )
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task update observation link count");
    assert_eq!(update_link_count, 1);

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/archive",
                urlencoding_percent_encode(&task_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn tasks_list_returns_items() {
    let Some(database_url) = test_database_url("tasks list test").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    create_task(&app, suffix).await;

    let response = app
        .oneshot(get_request_with_token("/api/v1/tasks", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let _items = body["items"].as_array().expect("items");
}

#[tokio::test]
async fn task_status_transition() {
    let Some(database_url) = test_database_url("task status test").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/status",
                urlencoding_percent_encode(&task_id)
            ),
            json!({"status": "completed"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();
    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task'
           AND entity_id = $1
           AND relationship_kind = 'status_update'",
    )
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task status observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn task_analyze_runtime_path_captures_observation_against_postgres() {
    let Some(database_url) = test_database_url("task analyze observation api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/analyze",
                urlencoding_percent_encode(&task_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();
    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task'
           AND entity_id = $1
           AND relationship_kind = 'analysis_update'",
    )
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task analyze observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn task_creation_rejects_unknown_review_item_reference_in_api() {
    let Some(database_url) = test_database_url("task create invalid provenance api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/tasks",
            json!({
                "title": format!("API invalid provenance task {suffix}"),
                "provenance_kind": "review_item",
                "provenance_id": "review_item:v1:does-not-exist",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("invalid_task_query"));
    assert_eq!(
        body["message"],
        json!("task provenance reference does not exist")
    );
}

#[tokio::test]
async fn task_creation_rejects_decision_without_observation_evidence_in_api() {
    let Some(database_url) = test_database_url("task create decision provenance api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();
    let decision = DecisionStore::new(pool)
        .upsert_with_evidence(
            &NewDecision::new(
                format!("API decision evidence gap {suffix}"),
                "Decision exists but has no observation evidence.",
                0.78,
                DecisionReviewState::Suggested,
            ),
            &[NewDecisionEvidence::new(
                DecisionEvidenceSourceKind::Event,
                format!("event:api-task-provenance:{suffix}"),
            )
            .quote("Decision evidence recorded outside canonical observations.")],
            &[],
        )
        .await
        .expect("create decision");

    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/tasks",
            json!({
                "title": format!("API provenance evidence gap {suffix}"),
                "provenance_kind": "decision",
                "provenance_id": decision.decision_id,
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("invalid_task_query"));
    assert_eq!(
        body["message"],
        json!("task provenance reference has no observation evidence")
    );
}

#[tokio::test]
async fn task_checklist_manual_create_path_captures_observation_against_postgres() {
    let Some(database_url) = test_database_url("task checklist observation api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/checklist",
                urlencoding_percent_encode(&task_id)
            ),
            json!({
                "items": [{"text": "Prepare migration", "done": false}],
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let checklist_id = body["id"].as_str().expect("checklist id").to_owned();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();

    let checklist_source: String =
        sqlx::query_scalar("SELECT source FROM task_checklists WHERE id::text = $1")
            .bind(&checklist_id)
            .fetch_one(&pool)
            .await
            .expect("checklist source");
    assert!(checklist_source.starts_with("observation:"));

    let observation_id = checklist_source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let row = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
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
    assert_eq!(
        row.try_get::<String, _>("kind_code").expect("kind code"),
        "TASK_MUTATION"
    );

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'tasks'
           AND entity_kind = 'task_checklist'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&checklist_id)
    .fetch_one(&pool)
    .await
    .expect("observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn task_evidence_manual_create_path_captures_observation_against_postgres() {
    let Some(database_url) = test_database_url("task evidence observation api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };


```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/tasks_api/mutations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/tasks_api/mutations.rs`
- Size bytes / Размер в байтах: `4525`
- Included characters / Включено символов: `4525`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;

#[tokio::test]
async fn task_rule_create_and_delete() {
    let Some(database_url) = test_database_url("task rule create/delete test").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(_task_id) = create_task(&app, suffix).await else {
        eprintln!("skip");
        return;
    };

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/tasks/rules",
            json!({"name": format!("Rule{suffix}"), "rule_type": "auto_priority", "config": json!({"default": "medium"})}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: rule create failed");
        return;
    }
    let rule_id = json_body(response).await["rule_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    if rule_id.is_empty() {
        return;
    }

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/tasks/rules/{}",
                urlencoding_percent_encode(&rule_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "rule delete={}",
        response.status()
    );
}

macro_rules! task_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = test_database_url(stringify!($name)).await else {
                return;
            };
            let suffix = unique_suffix();
            let app = build_tasks_app(&database_url).await;
            let Some(task_id) = create_task(&app, suffix).await else {
                eprintln!("skip: no task");
                return;
            };
            let response = app
                .oneshot(post_request_with_token(
                    &format!(
                        "/api/v1/tasks/{}/{}",
                        urlencoding_percent_encode(&task_id),
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

task_post_test!(
    task_post_context_pack,
    "context-pack",
    json!({"summary": "Test context"})
);
task_post_test!(
    task_post_evidence,
    "evidence",
    json!({"source": "email", "reference_id": "msg:test", "note": "Test evidence"})
);
task_post_test!(
    task_post_relation,
    "relations",
    json!({"related_task_id": "task:fake", "relation_type": "blocks"})
);
task_post_test!(
    task_post_checklist,
    "checklist",
    json!({"item": "Test item", "done": false})
);
task_post_test!(
    task_post_subtask,
    "subtasks",
    json!({"child_task_id": "task:fake", "sort_order": 0})
);

#[tokio::test]
async fn task_post_provider() {
    let Some(database_url) = test_database_url("task provider post test").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/tasks/providers",
            json!({"name": format!("Provider{suffix}"), "provider_type": "jira", "config": json!({"url": "https://example.com"})}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "provider post={}",
        response.status()
    );
}

#[tokio::test]
async fn task_candidate_review() {
    let Some(database_url) = test_database_url("task candidate review test").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let response = app
        .oneshot(put_request_with_token(
            &format!("/api/v1/task-candidates/tc:fake-{suffix}/review"),
            json!({"review_state": "declined", "reason": "Not relevant"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "candidate review={}",
        response.status()
    );
}
```

### `backend/tests/tasks_api/reads.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/tasks_api/reads.rs`
- Size bytes / Размер в байтах: `2891`
- Included characters / Включено символов: `2891`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;

async fn create_task_or_skip(app: &Router, suffix: u128) -> Option<String> {
    let task_id = create_task(app, suffix).await;
    if task_id.is_none() {
        eprintln!("skip: task create failed");
    }
    task_id
}

macro_rules! task_get_requires_task {
    ($name:ident, $path_suffix:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = test_database_url(stringify!($name)).await else {
                return;
            };
            let suffix = unique_suffix();
            let app = build_tasks_app(&database_url).await;
            let Some(task_id) = create_task_or_skip(&app, suffix).await else {
                return;
            };

            let response = app
                .oneshot(get_request_with_token(
                    &format!(
                        "/api/v1/tasks/{}/{}",
                        urlencoding_percent_encode(&task_id),
                        $path_suffix
                    ),
                    LOCAL_API_TOKEN,
                ))
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::OK);
        }
    };
}

macro_rules! task_get_simple {
    ($name:ident, $path:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = test_database_url(stringify!($name)).await else {
                return;
            };
            let app = build_tasks_app(&database_url).await;
            let response = app
                .oneshot(get_request_with_token($path, LOCAL_API_TOKEN))
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::OK);
        }
    };
}

task_get_requires_task!(task_context_pack_returns_ok, "context-pack");
task_get_requires_task!(task_evidence_list_returns_empty, "evidence");
task_get_requires_task!(task_relations_list_returns_empty, "relations");
task_get_requires_task!(task_checklist_list_returns_empty, "checklist");
task_get_requires_task!(task_subtasks_list_returns_empty, "subtasks");
task_get_requires_task!(task_export_returns_text, "export");
task_get_requires_task!(task_external_returns_ok, "external");

task_get_simple!(task_providers_list_returns_ok, "/api/v1/tasks/providers");
task_get_simple!(task_search_returns_ok, "/api/v1/tasks/search?q=test");
task_get_simple!(task_daily_brief_returns_ok, "/api/v1/tasks/daily-brief");
task_get_simple!(task_rules_list_returns_empty, "/api/v1/tasks/rules");
task_get_simple!(task_templates_list_returns_ok, "/api/v1/tasks/templates");
task_get_simple!(task_watchtower_returns_ok, "/api/v1/tasks/watchtower");
task_get_simple!(task_health_returns_ok, "/api/v1/tasks/health");
task_get_simple!(task_analytics_returns_ok, "/api/v1/tasks/analytics");
task_get_simple!(task_candidates_list_returns_ok, "/api/v1/task-candidates");
```

### `backend/tests/tasks_api/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/tasks_api/support.rs`
- Size bytes / Размер в байтах: `3735`
- Included characters / Включено символов: `3735`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

pub(crate) use axum::Router;
pub(crate) use axum::body::{Body, to_bytes};
pub(crate) use axum::http::{Method, Request, StatusCode, header};
pub(crate) use makosh_hub_backend::app::{build_router, build_router_with_database};
pub(crate) use makosh_hub_backend::platform::config::AppConfig;
pub(crate) use makosh_hub_backend::platform::storage::Database;
pub(crate) use serde_json::{Value, json};
pub(crate) use tower::ServiceExt;

pub(crate) const LOCAL_API_TOKEN: &str = "tasks-api-test-token";

pub(crate) fn config_with_api_token() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

pub(crate) async fn test_database_url(test_name: &str) -> Option<String> {
    let _ = test_name;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    Some(database_url)
}

pub(crate) fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn post_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) fn put_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) fn delete_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub(crate) async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

pub(crate) fn urlencoding_percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

pub(crate) async fn build_tasks_app(database_url: &str) -> Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url),
        database,
    )
}

pub(crate) async fn create_task(app: &Router, suffix: u128) -> Option<String> {
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/tasks",
            json!({
                "title": format!("API Task {suffix}"),
                "description": "Task for API testing",
                "status": "active",
                "priority": "medium",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        return None;
    }
    json_body(response).await["task_id"]
        .as_str()
        .map(|value| value.to_owned())
}
```

### `backend/tests/tasks_api_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/tasks_api_architecture.rs`
- Size bytes / Размер в байтах: `1942`
- Included characters / Включено символов: `1942`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn tasks_api_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_tasks_api_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "tasks api test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_tasks_api_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_tasks_api_test_violations(&path, violations);
            continue;
        }
        if !is_tasks_api_test_file(&path) {
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

fn is_tasks_api_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "tasks_api.rs" || file_name == "tasks_api_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "tasks_api")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/telegram.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram.rs`
- Size bytes / Размер в байтах: `1044`
- Included characters / Включено символов: `1044`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::communications::core::{
    CommunicationProviderKind, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::platform::secrets::SecretKind;

#[test]
fn telegram_provider_and_secret_kinds_are_account_scoped() {
    assert_eq!(
        CommunicationProviderKind::try_from("telegram_user").expect("telegram user"),
        CommunicationProviderKind::TelegramUser
    );
    assert_eq!(
        CommunicationProviderKind::try_from("telegram_bot").expect("telegram bot"),
        CommunicationProviderKind::TelegramBot
    );
    assert!(
        ProviderAccountSecretPurpose::TelegramApiHash.accepts_secret_kind(SecretKind::ApiToken)
    );
    assert!(
        ProviderAccountSecretPurpose::TelegramBotToken.accepts_secret_kind(SecretKind::ApiToken)
    );
    assert!(
        ProviderAccountSecretPurpose::TelegramSessionKey
            .accepts_secret_kind(SecretKind::PrivateKey)
    );
    assert!(
        !ProviderAccountSecretPurpose::TelegramBotToken.accepts_secret_kind(SecretKind::Password)
    );
}
```
