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

- Chunk ID / ID чанка: `091-test-backend-part-014`
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

### `backend/tests/telegram_references_members.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_references_members.rs`
- Size bytes / Размер в байтах: `21414`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
mod telegram_support;

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::json;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::lifecycle;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, json_post_request_with_actor,
    unique_suffix,
};
use testkit::context::TestContext;
#[tokio::test]
async fn telegram_reference_routes_return_enriched_message_summaries() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-reference-{suffix}");
    let chat_id = format!("reference-chat-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram References",
            "external_account_id": format!("tg-reference-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let root_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("reference-root-{suffix}"),
                "chat_kind": "group",
                "chat_title": "Reference Room",
                "sender_id": format!("sender-root-{suffix}"),
                "sender_display_name": "Root Sender",
                "text": "Root message for reply targets",
                "import_batch_id": format!("telegram-reference-root-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("root response");
    assert_eq!(root_response.status(), StatusCode::OK);
    let root_body = json_body(root_response).await;
    let root_message_id = root_body["message_id"]
        .as_str()
        .expect("root message id")
        .to_owned();

    let reply_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("reference-reply-{suffix}"),
                "chat_kind": "group",
                "chat_title": "Reference Room",
                "sender_id": format!("sender-reply-{suffix}"),
                "sender_display_name": "Reply Sender",
                "text": "Reply body should appear in chain",
                "import_batch_id": format!("telegram-reference-reply-{suffix}"),
                "occurred_at": "2026-06-06T12:01:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reply response");
    assert_eq!(reply_response.status(), StatusCode::OK);
    let reply_body = json_body(reply_response).await;
    let reply_message_id = reply_body["message_id"]
        .as_str()
        .expect("reply message id")
        .to_owned();

    let forward_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("reference-forward-{suffix}"),
                "chat_kind": "group",
                "chat_title": "Reference Room",
                "sender_id": format!("sender-forward-{suffix}"),
                "sender_display_name": "Forward Sender",
                "text": "Forward body should appear in summaries",
                "import_batch_id": format!("telegram-reference-forward-{suffix}"),
                "occurred_at": "2026-06-06T12:02:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("forward response");
    assert_eq!(forward_response.status(), StatusCode::OK);
    let forward_body = json_body(forward_response).await;
    let forward_message_id = forward_body["message_id"]
        .as_str()
        .expect("forward message id")
        .to_owned();
    let leaf_message_id = create_reference_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("reference-leaf-{suffix}"),
        "Leaf Sender",
        "Leaf reply should appear through bounded traversal",
        &suffix,
    )
    .await;
    let final_forward_message_id = create_reference_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("reference-final-forward-{suffix}"),
        "Final Forward Sender",
        "Final forward should appear through bounded traversal",
        &suffix,
    )
    .await;

    let pool = ctx.pool();
    lifecycle::insert_reply_ref(
        pool,
        &reply_message_id,
        &root_message_id,
        &account_id,
        &chat_id,
        &format!("reference-reply-{suffix}"),
        &format!("reference-root-{suffix}"),
        false,
    )
    .await
    .expect("insert reply ref");
    lifecycle::insert_reply_ref(
        pool,
        &leaf_message_id,
        &reply_message_id,
        &account_id,
        &chat_id,
        &format!("reference-leaf-{suffix}"),
        &format!("reference-reply-{suffix}"),
        false,
    )
    .await
    .expect("insert leaf reply ref");
    lifecycle::insert_reply_ref(
        pool,
        &root_message_id,
        &leaf_message_id,
        &account_id,
        &chat_id,
        &format!("reference-root-{suffix}"),
        &format!("reference-leaf-{suffix}"),
        false,
    )
    .await
    .expect("insert reply cycle ref");
    lifecycle::insert_forward_ref(
        pool,
        &forward_message_id,
        &account_id,
        &chat_id,
        &format!("reference-forward-{suffix}"),
        Some(&chat_id),
        Some(&format!("reference-root-{suffix}")),
        Some(&format!("sender-root-{suffix}")),
        Some("Root Sender"),
        Some(
            chrono::DateTime::parse_from_rfc3339("2026-06-05T11:00:00Z")
                .expect("timestamp")
                .with_timezone(&Utc),
        ),
    )
    .await
    .expect("insert forward ref");
    lifecycle::insert_forward_ref(
        pool,
        &final_forward_message_id,
        &account_id,
        &chat_id,
        &format!("reference-final-forward-{suffix}"),
        Some(&chat_id),
        Some(&format!("reference-forward-{suffix}")),
        Some(&format!("sender-forward-{suffix}")),
        Some("Forward Sender"),
        None,
    )
    .await
    .expect("insert final forward ref");
    lifecycle::insert_forward_ref(
        pool,
        &root_message_id,
        &account_id,
        &chat_id,
        &format!("reference-root-{suffix}"),
        Some(&chat_id),
        Some(&format!("reference-final-forward-{suffix}")),
        Some(&format!("sender-final-forward-{suffix}")),
        Some("Final Forward Sender"),
        None,
    )
    .await
    .expect("insert forward cycle ref");

    let reply_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{reply_message_id}/reply-chain"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reply chain response");
    assert_eq!(reply_chain_response.status(), StatusCode::OK);
    let reply_chain_body = json_body(reply_chain_response).await;
    assert_eq!(
        reply_chain_body["reply_to"][0]["target_message_summary"]["text"],
        json!("Root message for reply targets")
    );
    assert_eq!(
        reply_chain_body["reply_to"][0]["target_message_summary"]["sender_display_name"],
        json!("Root Sender")
    );

    let root_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{root_message_id}/reply-chain"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("root chain response");
    assert_eq!(root_chain_response.status(), StatusCode::OK);
    let root_chain_body = json_body(root_chain_response).await;
    assert_eq!(
        root_chain_body["replies"][0]["source_message_summary"]["text"],
        json!("Reply body should appear in chain")
    );
    assert_eq!(
        root_chain_body["replies"][0]["source_message_summary"]["sender_display_name"],
        json!("Reply Sender")
    );
    assert_eq!(root_chain_body["replies"].as_array().unwrap().len(), 2);
    assert_eq!(
        root_chain_body["replies"][1]["source_message_summary"]["text"],
        json!("Leaf reply should appear through bounded traversal")
    );

    let leaf_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{leaf_message_id}/reply-chain"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("leaf reply chain response");
    assert_eq!(leaf_chain_response.status(), StatusCode::OK);
    let leaf_chain_body = json_body(leaf_chain_response).await;
    assert_eq!(leaf_chain_body["reply_to"].as_array().unwrap().len(), 2);
    assert_eq!(
        leaf_chain_body["reply_to"][1]["target_message_summary"]["text"],
        json!("Root message for reply targets")
    );

    let forward_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{forward_message_id}/forward-chain"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("forward chain response");
    assert_eq!(forward_chain_response.status(), StatusCode::OK);
    let forward_chain_body = json_body(forward_chain_response).await;
    assert_eq!(
        forward_chain_body["forwards"][0]["source_message_summary"]["text"],
        json!("Forward body should appear in summaries")
    );
    assert_eq!(
        forward_chain_body["forwards"][0]["forward_origin_sender_name"],
        json!("Root Sender")
    );

    let final_forward_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{final_forward_message_id}/forward-chain"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("final forward chain response");
    assert_eq!(final_forward_chain_response.status(), StatusCode::OK);
    let final_forward_chain_body = json_body(final_forward_chain_response).await;
    assert_eq!(
        final_forward_chain_body["forwards"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        final_forward_chain_body["forwards"][1]["source_message_summary"]["text"],
        json!("Forward body should appear in summaries")
    );
}

async fn create_reference_message(
    app: axum::Router,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    sender_display_name: &str,
    text: &str,
    suffix: &str,
) -> String {
    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_ch
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_reply_forward_capability_gates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_reply_forward_capability_gates.rs`
- Size bytes / Размер в байтах: `5851`
- Included characters / Включено символов: `5851`
- Truncated / Обрезано: `no`

```rust
mod telegram_support;

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, ingest_fixture_telegram_message, json_body,
    json_post_request_with_actor, unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn fixture_account_blocks_reply_and_forward_before_side_effects() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-reply-forward-gates-{suffix}");
    let provider_chat_id = format!("reply-forward-chat-{suffix}");
    let reply_target_provider_message_id = format!("{provider_chat_id}:root");
    let forward_source_provider_message_id = format!("{provider_chat_id}:forward-source");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Reply Forward Gates",
            "external_account_id": format!("tg-reply-forward-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    let root_message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &provider_chat_id,
        &reply_target_provider_message_id,
        "Root message for reply gate coverage.",
        "2026-06-06T12:00:00Z",
    )
    .await;
    let _forward_message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &provider_chat_id,
        &forward_source_provider_message_id,
        "Source message for forward gate coverage.",
        "2026-06-06T12:01:00Z",
    )
    .await;

    let before_messages = message_count(app.clone(), &account_id, &provider_chat_id).await;

    let reply_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/messages/{root_message_id}/reply"
            ),
            json!({
                "command_id": format!("reply-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "reply_to_provider_message_id": reply_target_provider_message_id,
                "text": "Reply should be blocked in fixture mode by capability gate."
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reply response");
    assert_eq!(reply_response.status(), StatusCode::BAD_REQUEST);

    let forward_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/messages/{root_message_id}/forward"
            ),
            json!({
                "command_id": format!("forward-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "from_provider_chat_id": provider_chat_id,
                "from_provider_message_id": forward_source_provider_message_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("forward response");
    assert_eq!(forward_response.status(), StatusCode::BAD_REQUEST);

    let after_messages = message_count(app.clone(), &account_id, &provider_chat_id).await;
    assert_eq!(after_messages, before_messages);

    let send_audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE operation = 'telegram.message.send' AND metadata->>'account_id' = $1 AND metadata->>'provider_chat_id' = $2",
    )
    .bind(&account_id)
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("send audit count");
    assert_eq!(send_audit_count, 0);

    let command_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.command.status_changed' AND payload->>'provider_chat_id' = $1",
    )
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("command event count");
    assert_eq!(command_event_count, 0);

    let created_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.message.created' AND payload->>'provider_chat_id' = $1 AND subject->>'id' <> $2",
    )
    .bind(&provider_chat_id)
    .bind(&root_message_id)
    .fetch_one(&pool)
    .await
    .expect("created event count");
    assert_eq!(created_event_count, 1);
}

async fn message_count<S>(app: S, account_id: &str, provider_chat_id: &str) -> usize
where
    S: tower::Service<axum::http::Request<axum::body::Body>, Response = axum::response::Response>
        + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages?account_id={account_id}&conversation_id={provider_chat_id}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("messages response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["items"].as_array().expect("message items").len()
}
```

### `backend/tests/telegram_runtime_lifecycle.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_runtime_lifecycle.rs`
- Size bytes / Размер в байтах: `21773`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, account_item, delete_request_with_token, get_request_with_token, json_body,
    json_post_request_with_actor, unique_suffix,
};
use testkit::context::TestContext;
#[tokio::test]
async fn telegram_fixture_runtime_status_can_start_account_actor() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-runtime-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Runtime",
                "external_account_id": format!("tg-runtime-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let initial_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("initial runtime status");
    assert_eq!(initial_status.status(), StatusCode::OK);
    let initial_body = json_body(initial_status).await;
    assert_eq!(initial_body["account_id"], json!(account_id));
    assert_eq!(initial_body["provider_kind"], json!("telegram_user"));
    assert_eq!(initial_body["runtime_kind"], json!("fixture"));
    assert_eq!(initial_body["status"], json!("stopped"));
    assert_eq!(initial_body["live_send_available"], json!(false));
    assert_eq!(initial_body["fixture_runtime"], json!(true));
    assert_eq!(initial_body["telegram_api_id_configured"], json!(false));
    assert_eq!(initial_body["telegram_api_hash_configured"], json!(false));
    assert_eq!(
        initial_body["telegram_app_credentials_configured"],
        json!(false)
    );
    assert_eq!(initial_body["runtime_blockers"], json!([]));

    let start_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/runtime/start",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    assert_eq!(start_body["account_id"], json!(account_id));
    assert_eq!(start_body["status"], json!("running"));
    assert_eq!(start_body["runtime_kind"], json!("fixture"));

    let running_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("running runtime status");
    assert_eq!(running_status.status(), StatusCode::OK);
    let running_body = json_body(running_status).await;
    assert_eq!(running_body["status"], json!("running"));
    assert_eq!(running_body["last_error"], Value::Null);
    assert_eq!(running_body["runtime_blockers"], json!([]));

    let restart_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/runtime/restart",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime restart response");
    assert_eq!(restart_response.status(), StatusCode::OK);
    let restart_body = json_body(restart_response).await;
    assert_eq!(restart_body["account_id"], json!(account_id));
    assert_eq!(restart_body["runtime_kind"], json!("fixture"));
    assert_eq!(restart_body["status"], json!("running"));

    let stop_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/runtime/stop",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime stop response");
    assert_eq!(stop_response.status(), StatusCode::OK);
    let stop_body = json_body(stop_response).await;
    assert_eq!(stop_body["account_id"], json!(account_id));
    assert_eq!(stop_body["runtime_kind"], json!("fixture"));
    assert_eq!(stop_body["status"], json!("stopped"));
    assert_eq!(stop_body["live_send_available"], json!(false));

    let stopped_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("stopped runtime status");
    assert_eq!(stopped_status.status(), StatusCode::OK);
    let stopped_body = json_body(stopped_status).await;
    assert_eq!(stopped_body["status"], json!("stopped"));

    let audit_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.runtime.stop'
          AND target_id = $1
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(ctx.pool())
    .await
    .expect("runtime stop audit metadata");
    assert_eq!(audit_metadata["capability"], json!("telegram.runtime.stop"));
    assert_eq!(audit_metadata["action_class"], json!("local_write"));
    assert_eq!(audit_metadata["account_id"], json!(account_id));
    assert_eq!(audit_metadata["runtime_kind"], json!("fixture"));
    assert_eq!(audit_metadata["status"], json!("stopped"));

    let restart_audit_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.runtime.restart'
          AND target_id = $1
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(ctx.pool())
    .await
    .expect("runtime restart audit metadata");
    assert_eq!(
        restart_audit_metadata["capability"],
        json!("telegram.runtime.restart")
    );
    assert_eq!(restart_audit_metadata["action_class"], json!("local_write"));
    assert_eq!(restart_audit_metadata["account_id"], json!(account_id));
    assert_eq!(restart_audit_metadata["runtime_kind"], json!("fixture"));
    assert_eq!(restart_audit_metadata["status"], json!("running"));
}

#[tokio::test]
async fn telegram_runtime_status_reports_tdlib_diagnostics_for_qr_authorized_user_accounts() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-runtime-health-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_TDJSON_PATH",
                    "/tmp/makosh-test-missing-libtdjson-runtime-health.dylib",
                ),
                ("MAKOSH_TELEGRAM_API_ID", "12345"),
                ("MAKOSH_TELEGRAM_API_HASH", "telegram-api-hash"),
            ])
            .expect("config"),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Runtime Health",
                "external_account_id": format!("telegram:{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/runtime-health-{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let runtime_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime status response");
    assert_eq!(runtime_status.status(), StatusCode::OK);
    let body = json_body(runtime_status).await;
    assert_eq!(body["runtime_kind"], json!("tdlib_qr_authorized"));
    assert_eq!(
        body["tdjson_path"],
        json!("/tmp/makosh-test-missing-libtdjson-runtime-health.dylib")
    );
    assert_eq!(body["tdjson_runtime_available"], json!(false));
    assert_eq!(body["telegram_api_id_configured"], json!(true));
    assert_eq!(body["telegram_api_hash_configured"], json!(true));
    assert_eq!(body["telegram_app_credentials_configured"], json!(true));
    assert_eq!(body["live_send_available"], json!(false));
    assert!(
        body["tdjson_probe_error"]
            .as_str()
            .expect("tdjson probe error")
            .contains("unable to load libtdjson")
    );
    assert!(
        body["runtime_blockers"]
            .as_array()
            .expect("runtime blockers")
            .iter()
            .any(|value| value == "tdjson_runtime_unavailable")
    );
}

#[tokio::test]
async fn telegram_account_lifecycle_lists_logs_out_and_removes_without_deleting_evidence() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-lifecycle-{suffix}");
    let second_account_id = format!("telegram-lifecycle-second-{suffix}");
    let chat_id = format!("lifecycle-chat-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    for (id, display_name) in [
        (&account_id, "Telegram Lifecycle"),
        (&second_account_id, "Telegram Lifecycle Second"),
    ] {
        let response = app
            .clone()
            .oneshot(json_post_request_with_actor(
                "/api/v1/integrations/telegram/fixtures/accounts",
                json!({
                    "account_id": id,
                    "provider_kind": "telegram_user",
                    "display_name": display_name,
                    "external_account_id": format!("tg-{id}"),
                    "tdlib_data_path": format!("docker/data/telegram/{id}"),
                    "transcription_enabled": false
                }),
                LOCAL_API_TOKEN,
            ))
            .await
            .expect("account response");
        assert_eq!(response.status(), StatusCode::OK);
        let signal_connection = sqlx::query(
            r#"
            SELECT source_code, status, settings
            FROM signal_connections
            WHERE source_code = 'telegram'
              AND settings->>'account_id' = $1

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_search_pinning.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_search_pinning.rs`
- Size bytes / Размер в байтах: `20282`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, ingest_fixture_telegram_message, json_body,
    json_post_request_with_actor, unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn telegram_dialog_search_returns_projected_chat_matches() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-search-{suffix}");
    let matching_chat_id = format!("chat-alpha-{suffix}");
    let other_chat_id = format!("chat-beta-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Dialog Search",
            "external_account_id": format!("tg-dialog-search-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": matching_chat_id,
            "provider_message_id": format!("dialog-search-message-1-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Project Alpha Ops",
            "sender_id": format!("sender-alpha-{suffix}"),
            "sender_display_name": "Alpha Sender",
            "text": "Alpha conversation seed",
            "import_batch_id": format!("telegram-dialog-search-seed-1-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": other_chat_id,
            "provider_message_id": format!("dialog-search-message-2-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Beta Support",
            "sender_id": format!("sender-beta-{suffix}"),
            "sender_display_name": "Beta Sender",
            "text": "Beta conversation seed",
            "import_batch_id": format!("telegram-dialog-search-seed-2-{suffix}"),
            "occurred_at": "2026-06-06T12:05:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations/search?q=Alpha&account_id={account_id}&limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dialog search response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("dialog search items");
    assert_eq!(body["query"], json!("Alpha"));
    assert_eq!(body["total"], json!(1));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["provider_chat_id"], json!(matching_chat_id));
    assert_eq!(items[0]["title"], json!("Project Alpha Ops"));
}

#[tokio::test]
async fn telegram_media_search_filters_by_free_text_query() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-media-search-{suffix}");
    let chat_id = format!("chat-media-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Media Search",
            "external_account_id": format!("tg-media-search-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("media-search-message-1-{suffix}"),
                "chat_kind": "private",
                "chat_title": "Media Search Chat",
                "sender_id": format!("sender-media-{suffix}"),
                "sender_display_name": "Media Sender",
                "text": "invoice attachment",
                "import_batch_id": format!("telegram-media-search-seed-1-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

    sqlx::query(
        r#"
        UPDATE communication_messages
        SET message_metadata = $2::jsonb
        WHERE message_id = $1
        "#,
    )
    .bind(&message_id)
    .bind(json!({
        "attachments": [
            {
                "file_name": "invoice-2026.pdf",
                "kind": "document",
                "mime_type": "application/pdf",
                "size_bytes": 12345,
                "download_state": "downloaded",
                "attachment_id": "attachment-invoice-1",
                "tdlib_file_id": 4201,
                "local_path": "/tmp/makosh/invoice-2026.pdf"
            },
            {
                "file_name": "holiday-photo.jpg",
                "kind": "photo",
                "mime_type": "image/jpeg",
                "size_bytes": 45678,
                "download_state": "downloaded"
            }
        ]
    }))
    .execute(&pool)
    .await
    .expect("update message metadata");

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/search/media?q=invoice&account_id={account_id}&provider_chat_id={chat_id}&limit=20"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("media search response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("media search items");
    assert_eq!(body["query"], json!("invoice"));
    assert_eq!(body["source"], json!("projection"));
    assert_eq!(body["provider_search_attempted"], json!(false));
    assert_eq!(body["provider_search_error"], json!(null));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["file_name"], json!("invoice-2026.pdf"));
    assert_eq!(
        items[0]["provider_attachment_id"],
        json!("attachment-invoice-1")
    );
    assert_eq!(items[0]["tdlib_file_id"], json!(4201));
    assert_eq!(
        items[0]["local_path"],
        json!("/tmp/makosh/invoice-2026.pdf")
    );
}

#[tokio::test]
async fn telegram_pinned_messages_route_returns_projection_backed_items() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-pinned-messages-{suffix}");
    let chat_id = format!("chat-pinned-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Pinned Messages",
            "external_account_id": format!("tg-pinned-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let first_message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("pinned-message-1-{suffix}"),
        "Pinned root message",
        "2026-06-06T12:00:00Z",
    )
    .await;
    let second_message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("pinned-message-2-{suffix}"),
        "Newest pinned message",
        "2026-06-06T12:10:00Z",
    )
    .await;
    let unpinned_message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("pinned-message-3-{suffix}"),
        "Unpinned message",
        "2026-06-06T12:20:00Z",
    )
    .await;

    for message_id in [&first_message_id, &second_message_id] {
        sqlx::query(
            r#"
            UPDATE communication_messages
            SET message_metadata = $2::jsonb
            WHERE message_id = $1
            "#,
        )
        .bind(message_id)
        .bind(json!({ "is_pinned": true }))
        .execute(&pool)
        .await
        .expect("update pinned metadata");
    }
    sqlx::query(
        r#"
        UPDATE communication_messages
        SET message_metadata = $2::jsonb
        WHERE message_id = $1
        "#,
    )
    .bind(&unpinned_message_id)
    .bind(json!({ "is_pinned": false }))
    .execute(&pool)
    .await
    .expect("update unpinned metadata");

    let chats_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations?account_id={account_id}&limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chats response");
    assert_eq!(chats_response.status(), StatusCode::OK);
    let chats_body = json_body(chats_response).await;
    let telegram_chat_id = chats_body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned();

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/conversations/{telegram_chat_id}/pinned-messages?limit=10"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("pinned messages response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("pinned message items");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["message_id"], json!(second_message_id));
    assert_eq!(items[0]["text"], json!("Newest pinned message"));
    assert_eq!(items[1]["message_id"], json!(first_message_id));
    assert!(
        items

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_support/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_support/mod.rs`
- Size bytes / Размер в байтах: `5345`
- Included characters / Включено символов: `5345`
- Truncated / Обрезано: `no`

```rust
#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

pub const LOCAL_API_TOKEN: &str = "telegram-api-test-secret";

pub fn assert_capability_status(body: &Value, capability: &str, status: &str, closure_gate: bool) {
    let capabilities = body["capabilities"].as_array().expect("capabilities");
    let operation = match capability {
        "telegram_fixture_runtime" => "runtime.fixture",
        "automation_dry_run" => "automation.dry_run",
        "tdlib_live_runtime" => "runtime.tdlib_live",
        "automation_live_send" => "automation.live_send",
        "whisper_rs_speech_to_text" => "calls.transcription_live",
        other => other,
    };
    assert!(
        capabilities.iter().any(|item| {
            (item["capability"] == capability || item["operation"] == operation)
                && item["status"] == status
                && item["closure_gate"] == closure_gate
        }),
        "expected capability {capability}/{operation} to have status {status} and closure_gate {closure_gate}"
    );
}

pub async fn ingest_fixture_telegram_message<S>(
    app: S,
    account_id: &str,
    chat_id: &str,
    provider_message_id: &str,
    text: &str,
    occurred_at: &str,
) -> String
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "chat_kind": "private",
                "chat_title": "Pinned Message Chat",
                "sender_id": "sender-pinned",
                "sender_display_name": "Pinned Sender",
                "text": text,
                "import_batch_id": format!("telegram-pinned-seed-{provider_message_id}"),
                "occurred_at": occurred_at,
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fixture message response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["message_id"].as_str().expect("message id").to_owned()
}

pub async fn assert_ok<S>(app: S, path: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post_request_with_actor(path, body, LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

pub fn account_item<'a>(items: &'a [Value], account_id: &str) -> &'a Value {
    items
        .iter()
        .find(|item| item["account_id"] == json!(account_id))
        .unwrap_or_else(|| panic!("expected account `{account_id}` in account list"))
}

pub fn json_post_request_with_actor(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn json_post_request_with_explicit_actor_header(
    path: &str,
    body: Value,
    token: &str,
    actor_id: &str,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", token)
        .header("x-makosh-actor-id", actor_id)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn get_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub fn delete_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(path)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub fn vault_entropy_events(count: usize) -> Vec<Value> {
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
                "interval_ms": 5
            })
        })
        .collect()
}

pub async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

pub fn unique_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    format!("{now}")
}
```

### `backend/tests/telegram_topic_capability_gates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_topic_capability_gates.rs`
- Size bytes / Размер в байтах: `6083`
- Included characters / Включено символов: `6083`
- Truncated / Обрезано: `no`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::query;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-topic-capability-gates-secret";

#[tokio::test]
async fn fixture_account_allows_topic_list_but_blocks_topic_writes() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": "acct-1",
            "provider_kind": "telegram_user",
            "display_name": "Telegram Fixture User",
            "external_account_id": "telegram:12345",
            "tdlib_data_path": "docker/data/telegram/test-topic-capability-gates",
            "transcription_enabled": false
        }),
    )
    .await;
    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": "acct-1",
            "provider_chat_id": "provider-chat-1",
            "provider_message_id": "seed-message-1",
            "chat_kind": "group",
            "chat_title": "Capability Room",
            "sender_id": "sender-1",
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": "seed-batch-1",
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let telegram_chat_id = first_chat_id(app.clone()).await;
    query(
        r#"
        INSERT INTO telegram_topics (
            topic_id, telegram_chat_id, account_id, provider_topic_id, provider_chat_id,
            title, icon_emoji, is_pinned, is_closed, unread_count, metadata, created_at, updated_at
        )
        VALUES (
            'topic-1', $1, 'acct-1', 101, 'provider-chat-1',
            'General', NULL, false, false, 0, '{}'::jsonb, NOW(), NOW()
        )
        "#,
    )
    .bind(&telegram_chat_id)
    .execute(&pool)
    .await
    .expect("insert topic");

    let list_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/topics?limit=10"
        )))
        .await
        .expect("topics list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = json_body(list_response).await;
    assert_eq!(list_body["items"].as_array().expect("items").len(), 1);
    assert_eq!(list_body["items"][0]["topic_id"], "topic-1");

    let create_response = app
        .clone()
        .oneshot(json_post(
            &format!("/api/v1/communications/conversations/{telegram_chat_id}/topics"),
            json!({
                "account_id": "acct-1",
                "provider_chat_id": "provider-chat-1",
                "title": "New Topic",
                "command_id": "cmd-topic-create-1"
            }),
        ))
        .await
        .expect("topic create response");
    assert_eq!(create_response.status(), StatusCode::BAD_REQUEST);

    let close_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/telegram/provider-commands/topics/topic-1/close",
            json!({
                "account_id": "acct-1",
                "provider_chat_id": "provider-chat-1",
                "is_closed": true,
                "command_id": "cmd-topic-close-1"
            }),
        ))
        .await
        .expect("topic close response");
    assert_eq!(close_response.status(), StatusCode::BAD_REQUEST);

    let command_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_provider_write_commands WHERE account_id = 'acct-1' AND command_kind IN ('topic_create', 'topic_close', 'topic_reopen')",
    )
    .fetch_one(&pool)
    .await
    .expect("command count");
    assert_eq!(command_count, 0);
}

async fn first_chat_id<S>(app: S) -> String
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get(
            "/api/v1/communications/conversations?account_id=acct-1&limit=10",
        ))
        .await
        .expect("chat list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned()
}

async fn post_ok<S>(app: S, uri: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app.oneshot(json_post(uri, body)).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("X-Макошь-Secret", LOCAL_API_TOKEN)
        .body(Body::empty())
        .expect("request")
}

fn json_post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Макошь-Secret", LOCAL_API_TOKEN)
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json body")
}
```

### `backend/tests/timeline_engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/timeline_engine.rs`
- Size bytes / Размер в байтах: `21940`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::{TimeZone, Utc};
use makosh_hub_backend::engines::timeline::{TimelineEngine, TimelineEventDraft};
use makosh_hub_backend::platform::events::{
    EventEnvelope, EventStore, NewEventEnvelope, ProjectionCursorStore, StoredEventEnvelope,
};
use makosh_hub_backend::platform::storage::Database;
use serde_json::json;

#[test]
fn timeline_engine_bounds_entity_timeline_limits() {
    assert_eq!(TimelineEngine::bounded_entity_limit(0), 1);
    assert_eq!(TimelineEngine::bounded_entity_limit(25), 25);
    assert_eq!(TimelineEngine::bounded_entity_limit(250), 100);
}

#[test]
fn timeline_engine_rejects_unsourced_timeline_event() {
    let draft = TimelineEventDraft {
        entity_kind: "persona",
        entity_id: "persona:v1:human:alice",
        event_type: "first_message",
        title: "First message",
        occurred_at: Utc::now(),
        source: " ",
    };

    let error = TimelineEngine::validate_event(&draft).expect_err("event should be rejected");

    assert_eq!(error.to_string(), "timeline event source must not be empty");
}

#[test]
fn timeline_engine_accepts_source_backed_timeline_event() {
    let draft = TimelineEventDraft {
        entity_kind: "persona",
        entity_id: "persona:v1:human:alice",
        event_type: "first_message",
        title: "First message",
        occurred_at: Utc::now(),
        source: "communication_messages:message-1",
    };

    TimelineEngine::validate_event(&draft).expect("source-backed event should be valid");
}

#[test]
fn timeline_engine_builds_period_summary_for_source_backed_events() {
    let period_start = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();
    let events = vec![
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Message from Alice",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 3, 12, 0, 0).unwrap(),
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "project",
            entity_id: "project:makosh",
            event_type: "decision",
            title: "Decision accepted",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 5, 9, 0, 0).unwrap(),
            source: "decisions:decision-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Message from Alice before period",
            occurred_at: Utc.with_ymd_and_hms(2026, 5, 31, 23, 0, 0).unwrap(),
            source: "communication_messages:message-0",
        },
    ];

    let summary = TimelineEngine::period_summary(&events, period_start, period_end)
        .expect("period summary should be valid");

    assert_eq!(summary.period_start, period_start);
    assert_eq!(summary.period_end, period_end);
    assert_eq!(summary.total_events, 2);
    assert_eq!(summary.by_entity_kind.get("persona"), Some(&1));
    assert_eq!(summary.by_entity_kind.get("project"), Some(&1));
    assert_eq!(summary.by_event_type.get("message"), Some(&1));
    assert_eq!(summary.by_event_type.get("decision"), Some(&1));
}

#[test]
fn timeline_engine_rejects_invalid_period_summary_range() {
    let period_start = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();

    let error = TimelineEngine::period_summary(&[], period_start, period_end)
        .expect_err("period start must not be after period end");

    assert_eq!(
        error.to_string(),
        "timeline period start must not be after period end"
    );
}

#[test]
fn timeline_engine_builds_recency_signal_for_source_backed_entity_events() {
    let as_of = Utc.with_ymd_and_hms(2026, 6, 10, 12, 0, 0).unwrap();
    let last_event_at = Utc.with_ymd_and_hms(2026, 6, 8, 12, 0, 0).unwrap();
    let events = vec![
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Earlier message",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 2, 12, 0, 0).unwrap(),
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "decision",
            title: "Latest reviewed decision",
            occurred_at: last_event_at,
            source: "decisions:decision-1",
        },
        TimelineEventDraft {
            entity_kind: "project",
            entity_id: "project:makosh",
            event_type: "status_change",
            title: "Project update",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 9, 12, 0, 0).unwrap(),
            source: "projects:project-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "future_message",
            title: "Future message",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 11, 12, 0, 0).unwrap(),
            source: "communication_messages:message-2",
        },
    ];

    let signal =
        TimelineEngine::recency_signal(&events, "persona", "persona:v1:human:alice", as_of)
            .expect("recency signal should be valid");

    assert_eq!(signal.entity_kind, "persona");
    assert_eq!(signal.entity_id, "persona:v1:human:alice");
    assert_eq!(signal.last_event_at, Some(last_event_at));
    assert_eq!(signal.last_event_type.as_deref(), Some("decision"));
    assert_eq!(
        signal.last_event_source.as_deref(),
        Some("decisions:decision-1")
    );
    assert_eq!(signal.age_seconds, Some(172_800));
}

#[test]
fn timeline_engine_detects_source_backed_entity_timeline_gaps() {
    let period_start = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();
    let gap_start = Utc.with_ymd_and_hms(2026, 6, 2, 12, 0, 0).unwrap();
    let gap_end = Utc.with_ymd_and_hms(2026, 6, 10, 12, 0, 0).unwrap();
    let events = vec![
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Later message",
            occurred_at: gap_end,
            source: "communication_messages:message-2",
        },
        TimelineEventDraft {
            entity_kind: "project",
            entity_id: "project:makosh",
            event_type: "status_change",
            title: "Project update",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 6, 12, 0, 0).unwrap(),
            source: "projects:project-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Earlier message",
            occurred_at: gap_start,
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "decision",
            title: "Decision after gap",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 11, 12, 0, 0).unwrap(),
            source: "decisions:decision-1",
        },
    ];

    let gaps = TimelineEngine::timeline_gaps(
        &events,
        "persona",
        "persona:v1:human:alice",
        period_start,
        period_end,
        259_200,
    )
    .expect("timeline gaps should be valid");

    assert_eq!(gaps.len(), 1);
    let gap = &gaps[0];
    assert_eq!(gap.entity_kind, "persona");
    assert_eq!(gap.entity_id, "persona:v1:human:alice");
    assert_eq!(gap.gap_start, gap_start);
    assert_eq!(gap.gap_end, gap_end);
    assert_eq!(gap.gap_seconds, 691_200);
    assert_eq!(
        gap.previous_event_source.as_deref(),
        Some("communication_messages:message-1")
    );
    assert_eq!(
        gap.next_event_source.as_deref(),
        Some("communication_messages:message-2")
    );
}

#[test]
fn timeline_engine_rejects_invalid_gap_threshold() {
    let period_start = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();

    let error = TimelineEngine::timeline_gaps(
        &[],
        "persona",
        "persona:v1:human:alice",
        period_start,
        period_end,
        0,
    )
    .expect_err("timeline gap threshold must be positive");

    assert_eq!(
        error.to_string(),
        "timeline gap threshold must be greater than zero"
    );
}

#[test]
fn timeline_engine_builds_source_backed_change_diff_for_entity_snapshots() {
    let shared_event_at = Utc.with_ymd_and_hms(2026, 6, 2, 12, 0, 0).unwrap();
    let removed_event_at = Utc.with_ymd_and_hms(2026, 6, 4, 12, 0, 0).unwrap();
    let added_event_at = Utc.with_ymd_and_hms(2026, 6, 8, 12, 0, 0).unwrap();
    let previous_events = vec![
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Shared message",
            occurred_at: shared_event_at,
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "decision",
            title: "Removed decision",
            occurred_at: removed_event_at,
            source: "decisions:decision-1",
        },
    ];
    let current_events = vec![
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Shared message",
            occurred_at: shared_event_at,
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "project",
            entity_id: "project:makosh",
            event_type: "status_change",
            title: "Project update",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 7, 12, 0, 0).unwrap(),
            source: "projects:project-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "decision",
            title: "Added decision",
            occurred_at: added_event_at,
            source: "decisions:decision-2",
        },
    ];

    let diff = TimelineEngine::change_diff(
        &previous_events,
        &current_events,
        "persona",
        "persona:v1:human:alice",
    )
    .expect("change diff should be valid");

    assert_eq!(diff.entity_kind, "persona");
    assert_eq!(diff.entity_id, "persona:v1:human:alice");
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.removed.len(), 1);
    assert_eq!(diff.added[0].source, "decisions:decision-2");
    assert_eq!(diff.added[0].event_type, "decision");
    assert_eq!(diff.added[0].occurred_at, added_event_at);
    assert_eq!(diff.removed[0].source, "decisions:decision-1");
    assert_eq!(diff.removed[0].event_type, "decision");
    assert_eq!(diff.removed[0].occurred_at, removed_event_at);
}

#[test]
fn timeline_engine_builds_cross_domain_timeline_for_source_backed_events() {
    let period_start = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();
    let message_at = Utc.with_ymd_and_hms(2026, 6, 3, 12, 0, 0).unwrap();
    let decision_at = Utc.with_ymd_and_hms(2026, 6, 5, 9, 0, 0).unwrap();
    let events = vec![
        TimelineEventDraft {
            entity_kind: "project",
            entity_id: "project:makosh",

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/trust_engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/trust_engine.rs`
- Size bytes / Размер в байтах: `1931`
- Included characters / Включено символов: `1931`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::engines::trust::{TrustEngine, TrustSignalKind};

#[test]
fn trust_engine_maps_persona_compatibility_score_to_relationship_signal() {
    let signal = TrustEngine::persona_compatibility_score_signal(82);

    assert_eq!(signal.kind, TrustSignalKind::PersonaCompatibilityScore);
    assert_eq!(signal.relationship_type, "trusts");
    assert_eq!(signal.trust_score, 0.82);
    assert_eq!(signal.strength_score, 0.5);
    assert_eq!(signal.confidence, 1.0);
    assert_eq!(
        signal.explanation,
        "compatibility persons.trust_score signal"
    );
}

#[test]
fn trust_engine_clamps_legacy_persona_scores_to_relationship_range() {
    let low = TrustEngine::persona_compatibility_score_signal(-20);
    let high = TrustEngine::persona_compatibility_score_signal(135);

    assert_eq!(low.trust_score, 0.0);
    assert_eq!(high.trust_score, 1.0);
}

#[test]
fn trust_engine_builds_source_reliability_signal_for_review() {
    let signal = TrustEngine::source_reliability_signal(
        "person_enrichment:persona:v1:human:alice:trust_score",
        "trust_score=82",
        0.82,
    )
    .expect("source-backed trust signal should be valid");

    assert_eq!(signal.kind, TrustSignalKind::SourceReliability);
    assert_eq!(
        signal.affected_source,
        "person_enrichment:persona:v1:human:alice:trust_score"
    );
    assert_eq!(signal.evidence, "trust_score=82");
    assert_eq!(signal.confidence, 0.82);
    assert_eq!(signal.direction.as_str(), "positive");
    assert_eq!(signal.explanation, "source reliability signal for review");
}

#[test]
fn trust_engine_rejects_unsourced_source_reliability_signal() {
    let error = TrustEngine::source_reliability_signal(" ", "trust_score=82", 0.82)
        .expect_err("source reliability signal source should be required");

    assert_eq!(
        error.to_string(),
        "trust signal affected source must not be empty"
    );
}
```

### `backend/tests/v1_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_api.rs`
- Size bytes / Размер в байтах: `13129`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{HeaderValue, Method, Request, StatusCode, header};
use chrono::Utc;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;
use tower::ServiceExt;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::communications::storage::{
    CommunicationAttachmentDisposition, CommunicationStorageStore, LocalCommunicationBlobStore,
    NewCommunicationAttachment, NewCommunicationBlob,
};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "test-token";

#[tokio::test]
async fn v1_status_returns_enabled_surfaces_against_postgres() {
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
    assert_eq!(value["surfaces"]["persons"], json!(true));
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
            EmailProviderKind::Icloud,
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
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
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
        assert_local_cors_preflight(&app, origin, "GET", "/api/v1/status").await;
        assert_local_cors_preflight(
            &app,
            origin,
            "PATCH",
            "/api/v1/communications/messages/message-1",
        )
        .await;
    }
}

async fn assert_local_cors_preflight(
    app: &axum::Router,
    origin: &'static str,
    request_method: &'static str,
    uri: &'static str,
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
                    HeaderValue::from_static("x-makosh-secret"),
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
    assert!(bo
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/v1_communications_ai_state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_ai_state.rs`
- Size bytes / Размер в байтах: `7645`
- Included characters / Включено символов: `7645`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-ai-state-test-token";

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

fn put(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

#[tokio::test]
async fn v1_message_ai_state_transitions_are_durable_and_emit_event_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-ai-state-api-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-ai-state-api-{suffix}"),
        "AI state transition",
    )
    .await;

    let r = router(&context.connection_string()).await;
    let response = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages/{message_id}/ai-state"
        )))
        .await
        .expect("initial ai state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["ai_state"], "NEW");

    let response = r
        .clone()
        .oneshot(put(
            &format!("/api/v1/communications/messages/{message_id}/ai-state"),
            json!({"ai_state": "PROCESSING"}),
        ))
        .await
        .expect("transition ai state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["ai_state"], "PROCESSING");
    assert!(body["updated_at"].is_string());
    let observation = sqlx::query(
        r#"
        SELECT kind.code AS kind_code,
               observation.origin_kind,
               observation.payload,
               link.relationship_kind
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        JOIN observation_links link
          ON link.observation_id = observation.observation_id
        WHERE link.domain = 'communications'
          AND link.entity_kind = 'communication_message'
          AND link.entity_id = $1
          AND link.relationship_kind = 'ai_state_transition'
        ORDER BY observation.captured_at DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("ai state observation");
    assert_eq!(
        observation.try_get::<String, _>("kind_code").unwrap(),
        "COMMUNICATION_MESSAGE"
    );
    assert_eq!(
        observation.try_get::<String, _>("origin_kind").unwrap(),
        "manual"
    );
    let observation_payload = observation.try_get::<Value, _>("payload").unwrap();
    assert_eq!(observation_payload["message_id"], message_id);
    assert_eq!(observation_payload["previous_ai_state"], "NEW");
    assert_eq!(observation_payload["request"]["ai_state"], "PROCESSING");

    let persisted = sqlx::query(
        r#"
        SELECT ai_state, review_reason, last_error
        FROM communication_ai_states
        WHERE message_id = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("persisted ai state");
    assert_eq!(
        persisted.try_get::<String, _>("ai_state").unwrap(),
        "PROCESSING"
    );
    assert!(
        persisted
            .try_get::<Option<String>, _>("review_reason")
            .unwrap()
            .is_none()
    );
    assert!(
        persisted
            .try_get::<Option<String>, _>("last_error")
            .unwrap()
            .is_none()
    );

    let event = sqlx::query(
        r#"
        SELECT subject, payload
        FROM event_log
        WHERE event_type = 'mail.ai_state.changed'
          AND subject->>'kind' = 'mail_ai_state'
          AND subject->>'id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("ai state event");
    let subject = event.try_get::<Value, _>("subject").unwrap();
    let payload = event.try_get::<Value, _>("payload").unwrap();
    assert_eq!(subject["message_id"], message_id);
    assert_eq!(payload["ai_state"], "PROCESSING");
    assert_eq!(payload["previous_ai_state"], "NEW");
    assert!(payload.get("body_text").is_none());

    let response = r
        .oneshot(get(&format!(
            "/api/v1/communications/messages/{message_id}/ai-state"
        )))
        .await
        .expect("current ai state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["ai_state"], "PROCESSING");
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

async fn seed_projected_message(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            "Seed Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            account_id,
            "email_message",
            provider_record_id,
            format!("sha256:{provider_record_id}"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": subject,
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Private body that must not be emitted in AI state events"
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}
```

### `backend/tests/v1_communications_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_api.rs`
- Size bytes / Размер в байтах: `19872`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-test-token";

fn cfg() -> AppConfig {
    testkit::app::config_with_secret(T)
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("req")
}

fn pget(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}

fn pget_with_actor(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .header("x-makosh-actor-id", "makosh-frontend")
        .body(Body::from(body.to_string()))
        .expect("req")
}

fn pput(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("t")
        .as_nanos()
}

async fn router(db: &str) -> axum::Router {
    let database = Database::connect(Some(db)).await.expect("db");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, db),
        database,
    )
}

macro_rules! v1_read_test {
    ($name:ident, $path:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_context = TestContext::new().await;
            let db = test_context.connection_string();
            let r = router(&db).await;
            let resp = r.oneshot(get($path)).await.expect("r");
            assert!(
                !resp.status().is_server_error(),
                "{}: status={}",
                stringify!($name),
                resp.status()
            );
        }
    };
}

v1_read_test!(v1_messages_list, "/api/v1/communications/messages");
v1_read_test!(v1_message_states, "/api/v1/communications/messages/states");
v1_read_test!(v1_threads_list, "/api/v1/communications/threads");
v1_read_test!(
    v1_thread_messages,
    "/api/v1/communications/threads/messages?thread_id=thread%3Atest"
);
v1_read_test!(v1_search, "/api/v1/communications/search?q=test");
v1_read_test!(v1_personas_list, "/api/v1/communications/personas");
v1_read_test!(v1_drafts_list, "/api/v1/communications/drafts");
v1_read_test!(v1_invoices_list, "/api/v1/communications/finance/invoices");
v1_read_test!(
    v1_analytics_health,
    "/api/v1/communications/analytics/health"
);
v1_read_test!(
    v1_analytics_senders,
    "/api/v1/communications/analytics/senders"
);
v1_read_test!(v1_subscriptions, "/api/v1/communications/subscriptions");
v1_read_test!(
    v1_dup_attachments,
    "/api/v1/communications/attachments/duplicates"
);
v1_read_test!(v1_legal_list, "/api/v1/communications/legal");
v1_read_test!(v1_certs_list, "/api/v1/communications/certificates");
v1_read_test!(
    v1_certs_expiring,
    "/api/v1/communications/certificates/expiring"
);
v1_read_test!(v1_rich_templates, "/api/v1/communications/templates/rich");
v1_read_test!(v1_blockers, "/api/v1/communications/blockers");
v1_read_test!(
    v1_sync_status,
    "/api/v1/integrations/mail/accounts/sync-status"
);

// ── Write-like endpoints (may fail gracefully without data) ────────────────

macro_rules! v1_post_test {
    ($name:ident, $path:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_context = TestContext::new().await;
            let db = test_context.connection_string();
            let r = router(&db).await;
            let resp = r.oneshot(pget($path, $body)).await.expect("r");
            assert!(
                resp.status().is_success()
                    || resp.status() == StatusCode::NOT_FOUND
                    || resp.status() == StatusCode::BAD_REQUEST
                    || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
                "{}: status={}",
                stringify!($name),
                resp.status()
            );
        }
    };
}

v1_post_test!(
    v1_pin_msg,
    "/api/v1/communications/messages/msg:fake/pin",
    json!({})
);
v1_post_test!(
    v1_snooze_msg,
    "/api/v1/communications/messages/msg:fake/snooze",
    json!({"until": "2027-01-01T00:00:00Z"})
);
v1_post_test!(
    v1_mute_msg,
    "/api/v1/communications/messages/msg:fake/mute",
    json!({})
);
v1_post_test!(
    v1_label_msg,
    "/api/v1/communications/messages/msg:fake/labels",
    json!({"label": "important"})
);
v1_post_test!(
    v1_render_tpl,
    "/api/v1/communications/templates/rich/render",
    json!({"template": "Hello {{name}}", "context": {"name": "Test"}})
);

#[tokio::test]
async fn v1_sync_settings_default_update_and_manual_sync_status_against_postgres() {
    let context = TestContext::new().await;
    let db = context.connection_string();
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-sync-api-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Sync API IMAP",
            format!("sync-api-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let r = router(&db).await;
    let settings_path = format!("/api/v1/integrations/mail/accounts/{account_id}/sync-settings");
    let resp = r
        .clone()
        .oneshot(get(&settings_path))
        .await
        .expect("get settings");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read settings"),
    )
    .expect("settings json");
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["sync_enabled"], true);
    assert_eq!(body["batch_size"], 100);
    assert_eq!(body["poll_interval_seconds"], 300);

    let resp = r
        .clone()
        .oneshot(pput(
            &settings_path,
            json!({"sync_enabled": false, "batch_size": 7, "poll_interval_seconds": 600}),
        ))
        .await
        .expect("put settings");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read updated settings"),
    )
    .expect("updated settings json");
    assert_eq!(body["sync_enabled"], false);
    assert_eq!(body["batch_size"], 7);
    assert_eq!(body["poll_interval_seconds"], 600);

    let resp = r
        .clone()
        .oneshot(pget(
            &format!("/api/v1/integrations/mail/accounts/{account_id}/sync-now"),
            json!({}),
        ))
        .await
        .expect("sync now");
    let sync_now_status = resp.status();
    assert!(
        sync_now_status == StatusCode::OK || sync_now_status == StatusCode::BAD_REQUEST,
        "sync-now should return structured result or safe configuration error, got {sync_now_status:?}",
    );
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read sync-now"),
    )
    .expect("sync-now json");
    if sync_now_status == StatusCode::BAD_REQUEST {
        assert_eq!(body["error"], "invalid_communication_query");
        return;
    }
    assert_eq!(body["account_id"], account_id);
    assert!(body.get("status").is_some());
    assert!(body.get("phase").is_some());
    let run_id = body["run_id"].as_str().expect("sync run id");
    let sync_events = sqlx::query(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'kind' = 'mail_sync_run'
          AND subject->>'id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(&pool)
    .await
    .expect("sync events");
    let sync_event_types = sync_events
        .iter()
        .map(|row| row.get::<String, _>("event_type"))
        .collect::<Vec<_>>();
    assert_eq!(
        sync_event_types,
        vec!["mail.sync.started", "mail.sync.skipped"]
    );
    let skipped_payload = sync_events
        .last()
        .expect("skipped event")
        .get::<Value, _>("payload");
    assert_eq!(skipped_payload["account_id"], account_id);
    assert_eq!(skipped_payload["run_id"], run_id);
    assert_eq!(skipped_payload["status"], "skipped");
    let sync_observation_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT kind.code AS kind_code
        FROM observation_links link
        JOIN observations observation ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'communications'
          AND link.entity_kind = 'mail_sync_run'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(&pool)
    .await
    .expect("sync observations");
    assert_eq!(
        sync_observation_kinds,
        vec![
            "COMMUNICATION_MAIL_SYNC_RUN".to_owned(),
            "COMMUNICATION_MAIL_SYNC_RUN_STATUS".to_owned(),
        ]
    );
    let skipped_relationship_kind: String = sqlx::query_scalar(
        r#"
        SELECT relationship_kind
        FROM observation_links
        WHERE domain = 'communications'
          AND entity_kind = 'mail_sync_run'
          AND entity_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("skipped sync relationship kind");
    assert_eq!(skipped_relationship_kind, "skipped");

    let resp = r
        .oneshot(pget(
            &format!("/api/v1/integrations/mail/accounts/{account_id}/sync-full-resync"),
            json!({}),
        ))
        .await
        .expect("sync full resync");
    let full_resync_status = resp.status();
    assert!(
        full_resync_status == StatusCode::OK || full_resync_status == StatusCode::BAD_REQUEST,
        "sync-full-resync should return structured result or safe configuration error, got {full_resync_status:?}",
    );
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read sync-full-resync"),
    )
    .expect("sync-full-resync json");
    if full_resync_status == StatusCode::BAD_REQUEST {
        assert_eq!(body["error"], "invalid_communication_query");
        return;
    }
    assert_eq!(body["account_id"], account_id);
    assert!(body.get("status").is_some());
    assert!(body.get("phase").is_some());
}

#[tokio::test]
async fn v1_send_requires_explicit_provider_write_confirmation() {
    let ctx = TestContext::new().await;
    let r = router(&ctx.connection_string()).await;
    let resp = r
        .oneshot(pget_with_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": "icloud-primary",
                "to": ["recipient@example.com"],
                "subject": "Provider write guard",
                "body_text": "T
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/v1_communications_archive_inspection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_archive_inspection.rs`
- Size bytes / Размер в байтах: `6808`
- Included characters / Включено символов: `6808`
- Truncated / Обрезано: `no`

```rust
use std::io::{Cursor, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::communications::storage::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus, CommunicationAttachmentDisposition,
    CommunicationStorageStore, LocalCommunicationBlobStore, NewCommunicationAttachment,
    NewCommunicationBlob,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use testkit::context::TestContext;

const T: &str = "v1comms-archive-inspection-test-token";

#[tokio::test]
async fn v1_attachment_archive_inspection_reads_local_zip_blob_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_zip_attachment(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/archive-inspection",
            seeded.attachment_id
        )))
        .await
        .expect("archive inspection response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["filename"], "evidence.zip");
    assert_eq!(body["content_type"], "application/zip");
    assert_eq!(body["scan_status"], "not_scanned");
    assert_eq!(body["report"]["archive_kind"], "zip");
    assert_eq!(body["report"]["entry_count"], 2);
    assert_eq!(body["report"]["total_uncompressed_bytes"], 17);
    assert_eq!(body["report"]["has_nested_archive"], false);
    assert_eq!(
        body["report"]["entries"][0]["normalized_path"],
        "docs/readme.txt"
    );
    assert_eq!(
        body["report"]["entries"][1]["normalized_path"],
        "invoice.txt"
    );
}

struct SeededAttachment {
    attachment_id: String,
    message_id: String,
}

async fn seed_zip_attachment(pool: sqlx::PgPool) -> SeededAttachment {
    let suffix = uid();
    let account_id = format!("acct-archive-inspection-{suffix}");
    let provider_record_id = format!("provider-archive-inspection-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let storage_store = CommunicationStorageStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Archive Inspection Gmail",
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
                "subject": "Archive inspection",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Please inspect the attached archive metadata."
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;

    let zip_bytes = zip_bytes(&[
        ("docs/readme.txt", b"hello" as &[u8]),
        ("invoice.txt", b"invoice data" as &[u8]),
    ]);
    let local_blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let local_blob = local_blob_store
        .put_blob(&zip_bytes)
        .await
        .expect("write zip blob");
    let blob = storage_store
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&local_blob).content_type("application/zip"),
        )
        .await
        .expect("store zip blob metadata");
    let attachment = storage_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message_id,
                &raw.raw_record_id,
                blob.blob_id,
                "part-evidence-zip",
                "application/zip",
                local_blob.size_bytes,
                local_blob.sha256,
            )
            .filename("evidence.zip")
            .disposition(CommunicationAttachmentDisposition::Attachment)
            .scan_report(AttachmentSafetyScanReport {
                status: AttachmentSafetyScanStatus::NotScanned,
                engine: None,
                checked_at: None,
                summary: None,
                metadata: json!({}),
            }),
        )
        .await
        .expect("store zip attachment");

    SeededAttachment {
        attachment_id: attachment.attachment_id,
        message_id,
    }
}

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for (name, bytes) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }

    writer.finish().unwrap().into_inner()
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
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
```

### `backend/tests/v1_communications_attachment_preview.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/v1_communications_attachment_preview.rs`
- Size bytes / Размер в байтах: `9405`
- Included characters / Включено символов: `9405`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::communications::storage::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus, CommunicationAttachmentDisposition,
    CommunicationStorageStore, LocalCommunicationBlobStore, NewCommunicationAttachment,
    NewCommunicationBlob,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use testkit::context::TestContext;

const T: &str = "v1comms-attachment-preview-test-token";

#[tokio::test]
async fn v1_attachment_preview_reads_bounded_local_text_blob_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "notes.txt",
        "text/plain",
        AttachmentSafetyScanStatus::NotScanned,
        b"First line\nSecond line\n",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment preview response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["filename"], "notes.txt");
    assert_eq!(body["content_type"], "text/plain");
    assert_eq!(body["scan_status"], "not_scanned");
    assert_eq!(body["preview_kind"], "text");
    assert_eq!(body["text"], "First line\nSecond line\n");
    assert_eq!(body["truncated"], false);
    assert_eq!(body["byte_count"], 23);
    assert_eq!(body["max_preview_bytes"], 65536);
}

#[tokio::test]
async fn v1_attachment_preview_reads_bounded_local_image_blob_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "pixel.png",
        "image/png",
        AttachmentSafetyScanStatus::NotScanned,
        b"\x89PNG\r\n\x1a\n",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment image preview response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["filename"], "pixel.png");
    assert_eq!(body["content_type"], "image/png");
    assert_eq!(body["preview_kind"], "image");
    assert_eq!(body["text"], "");
    assert_eq!(body["data_url"], "data:image/png;base64,iVBORw0KGgo=");
    assert_eq!(body["truncated"], false);
    assert_eq!(body["byte_count"], 8);
}

#[tokio::test]
async fn v1_attachment_preview_reads_bounded_local_pdf_blob_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "spec.pdf",
        "application/pdf",
        AttachmentSafetyScanStatus::NotScanned,
        b"%PDF-1.4\n",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment pdf preview response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["filename"], "spec.pdf");
    assert_eq!(body["content_type"], "application/pdf");
    assert_eq!(body["preview_kind"], "pdf");
    assert_eq!(body["text"], "");
    assert_eq!(body["data_url"], "data:application/pdf;base64,JVBERi0xLjQK");
    assert_eq!(body["truncated"], false);
    assert_eq!(body["byte_count"], 9);
    assert_eq!(body["max_preview_bytes"], 16777216);
}

#[tokio::test]
async fn v1_attachment_preview_rejects_malicious_attachment_metadata() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "danger.txt",
        "text/plain",
        AttachmentSafetyScanStatus::Malicious,
        b"This text must not be exposed through preview.",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment preview rejection response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"], "invalid_communication_query");
    assert_eq!(
        body["message"],
        "attachment preview is blocked by attachment scan status"
    );
}

struct SeededAttachment {
    attachment_id: String,
    message_id: String,
}

async fn seed_text_attachment(
    pool: sqlx::PgPool,
    filename: &str,
    content_type: &str,
    scan_status: AttachmentSafetyScanStatus,
    bytes: &[u8],
) -> SeededAttachment {
    let suffix = uid();
    let account_id = format!("acct-attachment-preview-{suffix}");
    let provider_record_id = format!("provider-attachment-preview-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let storage_store = CommunicationStorageStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Attachment Preview Gmail",
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
            format!("sha256:{:0>64}", "f"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": "Attachment preview",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Preview the attached text metadata safely."
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
        .put_blob(bytes)
        .await
        .expect("write text blob");
    let blob = storage_store
        .upsert_blob(&NewCommunicationBlob::from_local_blob(&local_blob).content_type(content_type))
        .await
        .expect("store text blob metadata");
    let attachment = storage_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message_id,
                &raw.raw_record_id,
                blob.blob_id,
                format!("part-{filename}"),
                content_type,
                local_blob.size_bytes,
                local_blob.sha256,
            )
            .filename(filename)
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
        .expect("store text attachment");

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
        testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
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
```
