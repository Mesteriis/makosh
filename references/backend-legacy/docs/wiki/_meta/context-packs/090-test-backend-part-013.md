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

- Chunk ID / ID чанка: `090-test-backend-part-013`
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

### `backend/tests/telegram_members_sync_private.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_members_sync_private.rs`
- Size bytes / Размер в байтах: `9250`
- Included characters / Включено символов: `9250`
- Truncated / Обрезано: `no`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-members-private-sync-secret";

#[tokio::test]
async fn telegram_private_members_sync_uses_tdlib_chat_metadata_and_records_audit() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-private-members-{suffix}");
    let provider_chat_id = format!("private-chat-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::TelegramUser,
                format!("Telegram Private Members {suffix}"),
                "telegram:777".to_owned(),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("private-message-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Alice",
            "sender_id": "user:888",
            "sender_display_name": "Alice",
            "text": "hello",
            "import_batch_id": format!("telegram-private-members-seed-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let telegram_chat_id = first_chat_id(app.clone(), &account_id).await;
    sqlx::query(
        r#"
        UPDATE telegram_chats
        SET metadata = metadata || $2::jsonb
        WHERE telegram_chat_id = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .bind(json!({
        "tdlib_private_user_id": "888",
        "tdlib_chat_type": "chatTypePrivate"
    }))
    .execute(&pool)
    .await
    .expect("update chat metadata");

    let sync_response = app
        .clone()
        .oneshot(json_post(
            &format!(
                "/api/v1/integrations/telegram/provider-sync/conversations/{telegram_chat_id}/members"
            ),
            json!({}),
        ))
        .await
        .expect("sync members response");
    assert_eq!(sync_response.status(), StatusCode::OK);
    let sync_body = json_body(sync_response).await;
    assert_eq!(sync_body["telegram_chat_id"], json!(telegram_chat_id));
    assert_eq!(sync_body["synced_count"], json!(1));
    assert_eq!(sync_body["items"][0]["source"], json!("tdlib"));
    assert_eq!(
        sync_body["items"][0]["provider_member_id"],
        json!("user:888")
    );
    assert_eq!(sync_body["items"][0]["sender_display_name"], json!("Alice"));
    assert_eq!(sync_body["items"][0]["role"], json!("member"));
    assert_eq!(sync_body["items"][0]["status"], json!("member"));
    assert_eq!(
        sync_body["items"][0]["permissions"]["observed_via"],
        json!("tdlib.chat.metadata")
    );

    let members_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?limit=10"
        )))
        .await
        .expect("members response");
    assert_eq!(members_response.status(), StatusCode::OK);
    let members_body = json_body(members_response).await;
    assert_eq!(members_body["items"][0]["source"], json!("tdlib"));
    assert_eq!(
        members_body["items"][0]["provider_member_id"],
        json!("user:888")
    );

    let audit_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.participants.sync'
          AND actor_id = 'makosh-frontend'
          AND target_id = $1
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("participants sync audit metadata");
    assert_eq!(audit_metadata["action_class"], json!("read"));
    assert_eq!(
        audit_metadata["capability"],
        json!("telegram.participants.sync")
    );
    assert_eq!(audit_metadata["decision"], json!("allowed"));
    assert_eq!(
        audit_metadata["reason"],
        json!("explicit_user_confirmation")
    );
    assert_eq!(audit_metadata["account_id"], json!(account_id));
    assert_eq!(audit_metadata["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(audit_metadata["synced_count"], json!("1"));

    let participant_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'telegram.participant.updated'
          AND subject->>'kind' = 'telegram_chat_participant'
          AND subject->>'telegram_chat_id' = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("participant update event count");
    assert_eq!(participant_event_count, 1);

    let started_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'telegram.sync.started'
          AND payload->>'scope' = 'members'
          AND payload->>'provider_chat_id' = $2
          AND subject->>'id' = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("members sync started count");
    assert_eq!(started_count, 1);

    let progress_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'telegram.sync.progress'
          AND payload->>'scope' = 'members'
          AND payload->>'provider_chat_id' = $2
          AND payload->>'status' = 'completed'
          AND subject->>'id' = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("members sync progress count");
    assert_eq!(progress_count, 1);

    let completed_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'telegram.sync.completed'
          AND payload->>'scope' = 'members'
          AND payload->>'provider_chat_id' = $2
          AND payload->>'status' = 'completed'
          AND subject->>'id' = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("members sync completed count");
    assert_eq!(completed_count, 1);
}

async fn first_chat_id<S>(app: S, account_id: &str) -> String
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/conversations?account_id={account_id}&limit=10"
        )))
        .await
        .expect("chat list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned()
}

async fn post_ok<S>(app: S, path: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post(path, body))
        .await
        .expect("post response");
    assert_eq!(response.status(), StatusCode::OK);
}

fn get(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::empty())
        .expect("request")
}

fn json_post(path: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1_000_000)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

fn unique_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
        .to_string()
}
```

### `backend/tests/telegram_message_lifecycle_capability_gates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_message_lifecycle_capability_gates.rs`
- Size bytes / Размер в байтах: `16208`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, delete_request_with_token, get_request_with_token,
    ingest_fixture_telegram_message, json_body, json_post_request_with_actor, unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn removed_account_blocks_message_lifecycle_and_reaction_routes_before_side_effects() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-message-gates-{suffix}");
    let provider_chat_id = format!("message-gates-chat-{suffix}");
    let provider_message_id = format!("{provider_chat_id}:42");
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
                "display_name": "Telegram Message Capability Gates",
                "external_account_id": format!("tg-message-gates-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &provider_chat_id,
        &provider_message_id,
        "Message lifecycle actions should obey capability gates.",
        "2026-06-06T12:00:00Z",
    )
    .await;

    let account_delete_response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!("/api/v1/integrations/telegram/accounts/{account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("remove account response");
    assert_eq!(account_delete_response.status(), StatusCode::OK);

    let detail_before = message_detail(app.clone(), &message_id).await;
    let initial_metadata = detail_before["item"]["metadata"].clone();

    let edit_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/edit"),
            json!({
                "command_id": format!("edit-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "new_text": "edited after account removal"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("edit response");
    assert_eq!(edit_response.status(), StatusCode::BAD_REQUEST);

    let delete_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/messages/{message_id}/delete"
            ),
            json!({
                "command_id": format!("delete-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reason_class": "deleted_by_owner",
                "actor_class": "owner",
                "is_provider_delete": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete response");
    assert_eq!(delete_response.status(), StatusCode::BAD_REQUEST);

    let restore_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/restore-visibility"),
            json!({
                "command_id": format!("restore-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reason": "undo"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("restore response");
    assert_eq!(restore_response.status(), StatusCode::BAD_REQUEST);

    let pin_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/pin"),
            json!({
                "command_id": format!("pin-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "is_pinned": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("pin response");
    assert_eq!(pin_response.status(), StatusCode::BAD_REQUEST);

    let add_reaction_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/messages/{message_id}/reactions"),
            json!({
                "command_id": format!("react-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reaction_emoji": "👍",
                "sender_id": "owner",
                "sender_display_name": "Owner"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("add reaction response");
    assert_eq!(add_reaction_response.status(), StatusCode::BAD_REQUEST);

    let remove_reaction_response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/communications/messages/{message_id}/reactions?account_id={account_id}&provider_chat_id={provider_chat_id}&provider_message_id={provider_message_id}&reaction_emoji=%F0%9F%91%8D&sender_id=owner&sender_display_name=Owner&command_id=unreact-{suffix}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("remove reaction response");
    assert_eq!(remove_reaction_response.status(), StatusCode::BAD_REQUEST);

    let command_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_provider_write_commands WHERE account_id = $1 AND command_kind = ANY($2)",
    )
    .bind(&account_id)
    .bind(vec![
        "edit",
        "delete",
        "restore_visibility",
        "pin",
        "react",
        "unreact",
    ])
    .fetch_one(&pool)
    .await
    .expect("message lifecycle command count");
    assert_eq!(command_count, 0);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE target_id = $1 AND operation = ANY($2)",
    )
    .bind(&message_id)
    .bind(vec![
        "telegram.message.edit",
        "telegram.message.delete",
        "telegram.message.restore_visibility",
        "telegram.message.pin",
        "telegram.message.reaction",
    ])
    .fetch_one(&pool)
    .await
    .expect("message lifecycle audit count");
    assert_eq!(audit_count, 0);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE (subject->>'id' = $1 OR payload->>'message_id' = $1) AND event_type = ANY($2)",
    )
    .bind(&message_id)
    .bind(vec![
        "telegram.message.updated",
        "telegram.message.deleted",
        "telegram.message.visibility_restored",
        "telegram.reaction.changed",
        "telegram.command.status_changed",
    ])
    .fetch_one(&pool)
    .await
    .expect("message lifecycle event count");
    assert_eq!(event_count, 0);

    let version_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_message_versions WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("version count");
    assert_eq!(version_count, 0);

    let tombstone_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_message_tombstones WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("tombstone count");
    assert_eq!(tombstone_count, 0);

    let reaction_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_message_reactions WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("reaction count");
    assert_eq!(reaction_count, 0);

    let detail_after = message_detail(app, &message_id).await;
    let final_metadata = &detail_after["item"]["metadata"];
    assert_eq!(final_metadata["pinned"], initial_metadata["pinned"]);
    assert_eq!(final_metadata["is_pinned"], initial_metadata["is_pinned"]);
}

#[tokio::test]
async fn message_lifecycle_status_events_include_command_identity_for_realtime_command_inserts() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-message-status-{suffix}");
    let provider_chat_id = format!("message-status-chat-{suffix}");
    let provider_message_id = format!("{provider_chat_id}:42");
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
                "display_name": "Telegram Message Status Events",
                "external_account_id": format!("tg-message-status-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &provider_chat_id,
        &provider_message_id,
        "Message lifecycle realtime command payload should stay self-describing.",
        "2026-06-06T12:00:00Z",
    )
    .await;

    let edit_command_id = format!("edit-status-{suffix}");
    let edit_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/edit"),
            json!({
                "command_id": edit_command_id,
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "new_text": "edited from command status event test"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("edit response");
    assert_eq!(edit_response.status(), StatusCode::OK);

    let edit_diff: Value = sqlx::query_scalar(
        r#"
        SELECT raw_diff_payload
        FROM telegram_message_versions
        WHERE message_id = $1
        ORDER BY version_number DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_message_links.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_message_links.rs`
- Size bytes / Размер в байтах: `6811`
- Included characters / Включено символов: `6811`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_hub_backend::domains::communications::messages::ProviderChannelMessageStore;
use makosh_hub_backend::integrations::telegram::client::{
    NewTelegramChat, TelegramChatKind, TelegramStore, TelegramSyncState,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-message-link-test-secret";

#[tokio::test]
async fn telegram_message_ingestion_projects_public_message_link_without_erasing_chat_username() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-link-{suffix}");
    let chat_id = format!("100{suffix}");
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
            "display_name": "Telegram Link Projection",
            "external_account_id": format!("tg-link-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/link-{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let store = telegram_store(&pool);
    let public_chat = store
        .upsert_chat(&NewTelegramChat {
            account_id: account_id.clone(),
            provider_chat_id: chat_id.clone(),
            chat_kind: TelegramChatKind::Channel,
            title: "Public Link Channel".to_owned(),
            username: Some("МакошьPublicChannel".to_owned()),
            sync_state: TelegramSyncState::Synced,
            last_message_at: None,
            metadata: json!({"runtime": "tdlib"}),
        })
        .await
        .expect("public chat");

    let result = assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id.clone(),
            "provider_chat_id": chat_id.clone(),
            "provider_message_id": format!("{chat_id}:4242"),
            "chat_kind": "channel",
            "chat_title": public_chat.title.clone(),
            "sender_id": "sender-link",
            "sender_display_name": "Link Sender",
            "text": "Public channel message with stable provider permalink.",
            "import_batch_id": format!("telegram-link-fixture-{suffix}"),
            "occurred_at": "2026-06-19T10:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let chats_after_ingest = store
        .list_chats(Some(&account_id), 10)
        .await
        .expect("chat lookup");
    let chat_after_ingest = chats_after_ingest
        .iter()
        .find(|chat| chat.provider_chat_id == chat_id)
        .expect("chat row");
    assert_eq!(
        chat_after_ingest.username.as_deref(),
        Some("МакошьPublicChannel")
    );
    let chat_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'chat'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&public_chat.telegram_chat_id)
    .fetch_all(&pool)
    .await
    .expect("chat observations");
    assert!(
        chat_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_CHAT"
                && row.get::<String, _>("relationship_kind") == "upsert"
                && row.get::<Value, _>("payload")["username"] == json!("МакошьPublicChannel")
        }),
        "chat upsert observation must exist"
    );

    let message = store
        .message_by_id(result["message_id"].as_str().expect("message_id"))
        .await
        .expect("message lookup")
        .expect("projected message");
    assert_eq!(
        message.metadata["message_link"],
        json!("https://t.me/МакошьPublicChannel/4242")
    );
    assert_eq!(message.metadata["message_link_kind"], json!("public_t_me"));
}

async fn assert_ok<S>(app: S, path: &str, body: Value) -> Value
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post_request(path, body, LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("json response")
}

fn json_post_request(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn unique_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    format!("{now}")
}

fn telegram_store(pool: &sqlx::PgPool) -> TelegramStore {
    TelegramStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(ProviderChannelMessageStore::new(pool.clone())),
        Arc::new(
            makosh_hub_backend::domains::communications::core::CommunicationIngestionStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            makosh_hub_backend::platform::communications::EventStoreProviderMessageObservationEventPort::new(
                pool.clone(),
            ),
        ),
    )
}
```

### `backend/tests/telegram_message_mark_read_capability_gates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_message_mark_read_capability_gates.rs`
- Size bytes / Размер в байтах: `5739`
- Included characters / Включено символов: `5739`
- Truncated / Обрезано: `no`

```rust
mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, json_post_request_with_actor,
    unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn fixture_account_blocks_message_mark_read_before_side_effects() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-message-read-{suffix}");
    let provider_chat_id = format!("message-read-chat-{suffix}");
    let provider_message_id = format!("{provider_chat_id}:9001");
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
            "display_name": "Telegram Message Read",
            "external_account_id": format!("tg-message-read-{suffix}"),
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
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_kind": "private",
                "chat_title": "Message Read Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Maria Petrova",
                "text": "Mark this message as read through provider state.",
                "import_batch_id": format!("telegram-message-read-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_result = json_body(message_response).await;
    let message_id = message_result["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

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
    let initial_unread_count = chats_body["items"][0]["metadata"]["unread_count"].clone();
    let initial_mention_count = chats_body["items"][0]["metadata"]["mention_count"].clone();

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/messages/{message_id}/mark-read"
            ),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message mark-read response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let command_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_provider_write_commands WHERE account_id = $1 AND command_kind = 'mark_read'",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("mark-read command count");
    assert_eq!(command_count, 0);

    let command_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.command.status_changed' AND payload->>'command_kind' = 'mark_read' AND payload->>'message_id' = $1",
    )
    .bind(&provider_message_id)
    .fetch_one(&pool)
    .await
    .expect("message mark-read command event count");
    assert_eq!(command_event_count, 0);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE operation = 'telegram.message.mark_read' AND target_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message mark-read audit count");
    assert_eq!(audit_count, 0);

    let chat_row: Value = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT jsonb_build_object(
            'last_read_inbox_provider_message_id', metadata->>'last_read_inbox_provider_message_id',
            'unread_count', metadata->'unread_count',
            'mention_count', metadata->'mention_count'
        )
        FROM telegram_chats
        WHERE telegram_chat_id = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("chat metadata after blocked mark-read");
    assert_eq!(chat_row["last_read_inbox_provider_message_id"], Value::Null);
    assert_eq!(chat_row["unread_count"], initial_unread_count);
    assert_eq!(chat_row["mention_count"], initial_mention_count);
}
```

### `backend/tests/telegram_message_realtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_message_realtime.rs`
- Size bytes / Размер в байтах: `19398`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::Arc;

use chrono::Utc;
use serde_json::json;
use sqlx::Row;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderAccountStore, CommunicationProviderKind,
    CommunicationProviderSecretBindingStore, NewProviderAccount,
};
use makosh_hub_backend::domains::communications::messages::{
    ProviderChannelMessageStore, consume_accepted_signal_event,
};
use makosh_hub_backend::domains::signal_hub::dispatch_telegram_raw_signal;
use makosh_hub_backend::integrations::telegram::client::lifecycle::{
    self, reconcile_delete_commands_from_provider_state,
    reconcile_edit_commands_from_provider_state,
    reconcile_message_pin_commands_from_provider_state, record_provider_delete_observation,
    record_provider_edit_observation,
};
use makosh_hub_backend::integrations::telegram::client::{
    NewTelegramMessage, TelegramChatKind, TelegramDeliveryState, TelegramMessage, TelegramStore,
};
use testkit::context::TestContext;

#[tokio::test]
async fn telegram_provider_delete_observation_is_idempotent_and_reconciles_delete_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "message-delete", "telegram:delete").await;
    let store = telegram_store(&pool);
    let provider_chat_id = "-100message-delete";
    let provider_message_id = format!("{provider_chat_id}:42");

    let message = ingest_projected_fixture_message(
        &pool,
        &store,
        NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Delete Test".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "hello".to_owned(),
            import_batch_id: "telegram-realtime-test".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        },
    )
    .await;

    lifecycle::insert_command(
        &pool,
        "tcmd_delete_observed",
        &account_id,
        "delete",
        "delete-observed",
        provider_chat_id,
        Some(&provider_message_id),
        "available",
        "destructive",
        "confirmed",
        "makosh-frontend",
        json!({"reason_class": "deleted_by_owner", "is_provider_delete": true}),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
        json!({"source": "test"}),
    )
    .await
    .expect("insert delete command");

    let first_tombstone = record_provider_delete_observation(
        &pool,
        &message,
        Utc::now(),
        "updateDeleteMessages",
        true,
        false,
    )
    .await
    .expect("first tombstone");
    let second_tombstone = record_provider_delete_observation(
        &pool,
        &message,
        Utc::now(),
        "updateDeleteMessages",
        true,
        false,
    )
    .await
    .expect("second tombstone");

    assert_eq!(first_tombstone.tombstone_id, second_tombstone.tombstone_id);
    assert_eq!(first_tombstone.reason_class, "deleted_by_provider");
    assert_eq!(first_tombstone.actor_class, "provider");
    assert!(!first_tombstone.is_local_visible);

    let reconciled = reconcile_delete_commands_from_provider_state(
        &pool,
        &account_id,
        provider_chat_id,
        &provider_message_id,
        Utc::now(),
        "tdlib.updateDeleteMessages",
    )
    .await
    .expect("reconcile delete commands");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, "tcmd_delete_observed");
    assert_eq!(reconciled[0].status, "completed");
    assert_eq!(reconciled[0].reconciliation_status, "observed");

    let tombstones = lifecycle::list_tombstones(&pool, &message.message_id)
        .await
        .expect("list tombstones");
    assert_eq!(tombstones.len(), 1);
    let tombstone_id = tombstones[0].tombstone_id.clone();
    let tombstone_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'message_tombstone'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&tombstone_id)
    .fetch_all(&pool)
    .await
    .expect("tombstone observations");
    assert!(
        tombstone_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_MESSAGE_TOMBSTONE"
                && row.get::<String, _>("relationship_kind") == "provider_delete"
                && row.get::<serde_json::Value, _>("payload")["reason_class"]
                    == json!("deleted_by_provider")
        }),
        "provider_delete tombstone observation must exist"
    );
}

#[tokio::test]
async fn telegram_provider_edit_observation_is_idempotent_and_reconciles_edit_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "message-edit", "telegram:edit").await;
    let store = telegram_store(&pool);
    let provider_chat_id = "-100message-edit";
    let provider_message_id = format!("{provider_chat_id}:42");

    let message = ingest_projected_fixture_message(
        &pool,
        &store,
        NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Edit Test".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "before".to_owned(),
            import_batch_id: "telegram-realtime-test".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        },
    )
    .await;

    lifecycle::insert_command(
        &pool,
        "tcmd_edit_observed",
        &account_id,
        "edit",
        "edit-observed",
        provider_chat_id,
        Some(&provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({"new_text": "after"}),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
        json!({"source": "test"}),
    )
    .await
    .expect("insert edit command");

    let first_version = record_provider_edit_observation(
        &pool,
        &message,
        "after",
        Utc::now(),
        "updateMessageContent",
        json!({"previous_text": "before", "new_text": "after"}),
        json!({"provider": "telegram", "runtime": "tdlib"}),
    )
    .await
    .expect("first version");
    let second_version = record_provider_edit_observation(
        &pool,
        &message,
        "after",
        first_version.edit_timestamp,
        "updateMessageContent",
        json!({"previous_text": "before", "new_text": "after"}),
        json!({"provider": "telegram", "runtime": "tdlib"}),
    )
    .await
    .expect("second version");

    assert_eq!(first_version.version_id, second_version.version_id);
    assert_eq!(first_version.body_text.as_deref(), Some("after"));
    assert_eq!(
        first_version.source_event.as_deref(),
        Some("updateMessageContent")
    );

    let reconciled = reconcile_edit_commands_from_provider_state(
        &pool,
        &account_id,
        provider_chat_id,
        &provider_message_id,
        "after",
        Utc::now(),
        "tdlib.updateMessageContent",
    )
    .await
    .expect("reconcile edit commands");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, "tcmd_edit_observed");
    assert_eq!(reconciled[0].status, "completed");
    assert_eq!(reconciled[0].reconciliation_status, "observed");
    let version_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'message_version'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&first_version.version_id)
    .fetch_all(&pool)
    .await
    .expect("message version observations");
    assert!(
        version_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_MESSAGE_VERSION"
                && row.get::<String, _>("relationship_kind") == "insert"
                && row.get::<serde_json::Value, _>("payload")["version_number"] == json!(1)
        }),
        "message version observation must exist"
    );
}

#[tokio::test]
async fn telegram_provider_edit_observation_marks_mismatched_edit_command_failed() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id =
        create_telegram_account(&pool, "message-edit-mismatch", "telegram:edit-mismatch").await;
    let store = telegram_store(&pool);
    let provider_chat_id = "-100message-edit-mismatch";
    let provider_message_id = format!("{provider_chat_id}:42");

    store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Edit Mismatch Test".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "before".to_owned(),
            import_batch_id: "telegram-realtime-test".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    lifecycle::insert_command(
        &pool,
        "tcmd_edit_mismatch",
        &account_id,
        "edit",
        "edit-mismatch",
        provider_chat_id,
        Some(&provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({"new_text": "expected provider body"}),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
        json!({"source": "test"}),
    )
    .await
    .expect("insert edit command");

    let reconciled = reconcile_edit_commands_from_provider_state(
        &pool,
        &account_id,
        provider_chat_id,
        &provider_message_id,
        "different provider body",
        Utc::now(),
        "tdlib.updateMessageContent",
    )
    .await
    .expect("reconcile mismatched edit commands");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, "tcmd_edit_mismatch");
    assert_eq!(reconciled[0].status, "failed");
    assert_eq!(reconciled[0].reconciliation_status, "mismatch");
    assert_eq!(
        reconciled[0].last_error.as_deref(),
        Some("Provider observed a different message body than requested")
    );
    assert_eq!(
        reconciled[0].provider_state["expected_body_text"],
        json!("expec
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_messages_basic.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_messages_basic.rs`
- Size bytes / Размер в байтах: `17732`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
mod telegram_support;

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::consume_accepted_signal_event;
use makosh_hub_backend::domains::signal_hub::dispatch_telegram_raw_signal;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, json_post_request_with_actor,
    json_post_request_with_explicit_actor_header, unique_suffix,
};
use testkit::context::TestContext;
#[tokio::test]
async fn telegram_manual_send_records_sent_message_and_redacted_provider_write_audit() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-send-{suffix}");
    let chat_id = format!("send-chat-{suffix}");
    let command_id = format!("manual-send-{suffix}");
    let message_text = "Manual Telegram reply from Макошь.";
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
            "display_name": "Telegram Send",
            "external_account_id": format!("tg-send-{suffix}"),
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
            "provider_chat_id": chat_id,
            "provider_message_id": format!("incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Manual Send Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Can Макошь reply here?",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let send_response = app
        .clone()
        .oneshot(json_post_request_with_explicit_actor_header(
            "/api/v1/integrations/telegram/provider-commands/messages/send",
            json!({
                "command_id": command_id,
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "text": message_text
            }),
            LOCAL_API_TOKEN,
            "legacy-telegram-test-actor",
        ))
        .await
        .expect("send response");
    assert_eq!(send_response.status(), StatusCode::OK);
    let send_body = json_body(send_response).await;
    assert_eq!(send_body["account_id"], json!(account_id));
    assert_eq!(send_body["provider_chat_id"], json!(chat_id));
    assert_eq!(send_body["delivery_state"], json!("sent"));
    assert_eq!(send_body["status"], json!("sent"));
    assert_eq!(send_body["runtime_kind"], json!("fixture"));
    assert!(
        send_body["message_id"]
            .as_str()
            .expect("message id")
            .starts_with("message:v4:telegram:")
    );
    assert!(
        send_body["rendered_preview_hash"]
            .as_str()
            .expect("preview hash")
            .starts_with("sha256:")
    );

    let messages_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages?account_id={account_id}&conversation_id={chat_id}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("messages response");
    assert_eq!(messages_response.status(), StatusCode::OK);
    let messages_body = json_body(messages_response).await;
    let sent_message = messages_body["items"]
        .as_array()
        .expect("messages")
        .iter()
        .find(|message| message["delivery_state"] == "sent")
        .expect("sent message");
    assert_eq!(sent_message["body_text_preview"], json!(message_text));
    assert_eq!(sent_message["sender_display_name"], json!("Макошь"));

    let audit_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.message.send'
          AND actor_id = $1
          AND target_id = $2
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind("makosh-frontend")
    .bind(send_body["message_id"].as_str().expect("message id"))
    .fetch_one(&pool)
    .await
    .expect("manual send audit metadata");
    assert_eq!(audit_metadata["action_class"], json!("provider_write"));
    assert_eq!(audit_metadata["capability"], json!("telegram.message.send"));
    assert_eq!(audit_metadata["decision"], json!("allowed"));
    assert_eq!(
        audit_metadata["reason"],
        json!("explicit_user_confirmation")
    );
    assert_eq!(audit_metadata["confirmation_required"], json!(false));
    assert_eq!(audit_metadata["account_id"], json!(account_id));
    assert_eq!(audit_metadata["provider_chat_id"], json!(chat_id));
    assert_eq!(
        audit_metadata["rendered_preview_hash"],
        send_body["rendered_preview_hash"]
    );
    assert!(audit_metadata.get("text").is_none());
    assert!(audit_metadata.get("message_text").is_none());
    assert!(audit_metadata.get("rendered_text").is_none());

    let neutral_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/conversations/{chat_id}/messages"),
            json!({
                "account_id": account_id,
                "text": "Provider-neutral Telegram send from Макошь."
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("provider-neutral send response");
    assert_eq!(neutral_response.status(), StatusCode::OK);
    let neutral_body = json_body(neutral_response).await;
    assert!(
        neutral_body["message_id"]
            .as_str()
            .expect("neutral message id")
            .starts_with("message:v4:telegram:")
    );
    assert!(
        !neutral_body["raw_record_id"]
            .as_str()
            .expect("neutral raw record id")
            .is_empty()
    );
    assert_eq!(neutral_body["conversation_id"], json!(chat_id));
    assert_eq!(neutral_body["channel_kind"], json!("telegram"));
    assert_eq!(neutral_body["provider"], json!("telegram"));
    assert_eq!(neutral_body["status"], json!("sent"));
}

#[tokio::test]
async fn telegram_raw_message_endpoint_returns_sanitized_source_evidence() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-raw-{suffix}");
    let chat_id = format!("raw-chat-{suffix}");
    let provider_message_id = format!("raw-message-{suffix}");
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
            "display_name": "Telegram Raw Evidence",
            "external_account_id": format!("tg-raw-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let raw_record_id = format!("telegram-raw-record:{suffix}");
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "telegram_message",
                &provider_message_id,
                format!("sha256:raw-{suffix}"),
                format!("telegram-raw-fixture-{suffix}"),
                json!({
                    "provider_chat_id": chat_id,
                    "chat_title": "Raw Evidence Chat",
                    "sender_id": format!("sender-{suffix}"),
                    "sender_display_name": "Raw Sender",
                    "text": "Raw evidence should stay visible.",
                    "delivery_state": "received",
                    "tdlib_raw": {
                        "@type": "message",
                        "id": 42,
                        "nested": {
                            "api_hash": "telegram-api-hash",
                            "token": "telegram-token"
                        }
                    }
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "runtime": "tdlib",
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "password": "provider-password"
            })),
        )
        .await
        .expect("raw source");
    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &raw_record)
        .await
        .expect("dispatch raw telegram signal")
        .expect("accepted telegram signal");
    let projected = consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted telegram signal")
        .expect("projected telegram message");
    let message_id = projected.message_id;

    let raw_response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{message_id}/raw-evidence"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("raw evidence response");
    assert_eq!(raw_response.status(), StatusCode::OK);
    let raw_body = json_body(raw_response).await;
    assert_eq!(
        raw_body["raw_record"]["raw_record_id"],
        json!(raw_record_id)
    );
    assert_eq!(
        raw_body["raw_record"]["provider_record_id"],
        json!(provider_message_id)
    );
    assert_eq!(
        raw_body["raw_record"]["payload"]["text"],
        json!("Raw evidence should stay visible.")
    );
    assert_eq!(
        raw_body["raw_record"]["payload"]["tdlib_raw"]["@type"],
        json!("message")
    );
    assert_eq!(
        raw_body["raw_record"]["payload"]["tdlib_raw"]["nested"]["api_hash"],
        json!("[redacted]")
    );
    assert_eq!(
        raw_body["raw_record"]["payload"]["tdlib_raw"]["nested"]["token"],
        json!("[redacted]")
    );
    assert_eq!(
        raw_body["raw_record"]["provenance"]["password"],
        json!("[redacted]")
    );
}

#[tokio::test]
async fn telegram_fixture_sync_chats_returns_account_chat_metadata() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-sync-chats-{suffix}");
    let chat_id = format!("sync-chat-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_u
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_outbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_outbox.rs`
- Size bytes / Размер в байтах: `5665`
- Included characters / Включено символов: `5665`
- Truncated / Обрезано: `no`

```rust
use chrono::{Duration, Utc};
use serde_json::json;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
};
use makosh_hub_backend::integrations::telegram::client::lifecycle;
use testkit::context::TestContext;

#[tokio::test]
async fn telegram_outbox_claims_due_command_and_unlocks_while_awaiting_provider() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "claim-await").await;
    let command_id = "tcmd_claim_awaiting_provider";

    insert_edit_command(&pool, &account_id, command_id).await;

    let claimed = lifecycle::claim_due_commands_for_execution(&pool, &account_id, Utc::now(), 10)
        .await
        .expect("claim due commands");

    assert_eq!(claimed.len(), 1);
    let command = &claimed[0];
    assert_eq!(command.command_id, command_id);
    assert_eq!(command.status, "executing");
    assert_eq!(command.retry_count, 1);
    assert!(command.last_attempt_at.is_some());
    assert_eq!(command.locked_by.as_deref(), Some("telegram-outbox-worker"));
    assert_eq!(command.reconciliation_status, "awaiting_provider");

    lifecycle::mark_command_awaiting_provider(
        &pool,
        command_id,
        Utc::now(),
        json!({"dispatch": "accepted"}),
    )
    .await
    .expect("mark awaiting provider");

    let stored = lifecycle::list_commands(&pool, &account_id, 10)
        .await
        .expect("list commands")
        .into_iter()
        .find(|item| item.command_id == command_id)
        .expect("stored command");

    assert_eq!(stored.status, "executing");
    assert_eq!(stored.reconciliation_status, "awaiting_provider");
    assert!(stored.locked_at.is_none());
    assert!(stored.locked_by.is_none());
}

#[tokio::test]
async fn telegram_outbox_recovers_stale_locked_execution_for_retry() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "stale-retry").await;
    let command_id = "tcmd_stale_retry";

    insert_edit_command(&pool, &account_id, command_id).await;
    let claimed = lifecycle::claim_due_commands_for_execution(&pool, &account_id, Utc::now(), 10)
        .await
        .expect("claim due commands");
    assert_eq!(claimed.len(), 1);

    let stale_locked_at = Utc::now() - Duration::minutes(10);
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET locked_at = $2
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .bind(stale_locked_at)
    .execute(&pool)
    .await
    .expect("backdate lock");

    let recovered = lifecycle::recover_stale_executing_commands(
        &pool,
        Utc::now(),
        Utc::now() - Duration::minutes(2),
    )
    .await
    .expect("recover stale commands");

    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].command_id, command_id);
    assert_eq!(recovered[0].status, "retrying");
    assert!(recovered[0].next_attempt_at.is_some());
    assert!(recovered[0].locked_at.is_none());
    assert!(recovered[0].locked_by.is_none());
}

#[tokio::test]
async fn telegram_outbox_dead_letter_can_be_manually_retried() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "manual-retry").await;
    let command_id = "tcmd_manual_retry";

    insert_edit_command(&pool, &account_id, command_id).await;
    lifecycle::dead_letter_command(
        &pool,
        command_id,
        Utc::now(),
        "Unsupported provider write command",
    )
    .await
    .expect("dead letter command");

    let dead_lettered = lifecycle::list_commands(&pool, &account_id, 10)
        .await
        .expect("list commands")
        .into_iter()
        .find(|item| item.command_id == command_id)
        .expect("stored command");
    assert_eq!(dead_lettered.status, "dead_letter");
    assert!(dead_lettered.dead_lettered_at.is_some());

    let retried = lifecycle::manual_retry_command(&pool, command_id, Utc::now())
        .await
        .expect("manual retry")
        .expect("eligible command");

    assert_eq!(retried.status, "retrying");
    assert_eq!(retried.retry_count, 0);
    assert!(retried.next_attempt_at.is_some());
    assert!(retried.dead_lettered_at.is_none());
    assert_eq!(retried.reconciliation_status, "not_observed");
}

async fn create_telegram_account(pool: &sqlx::PgPool, suffix: &str) -> String {
    let account_id = format!("telegram-outbox-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::TelegramUser,
                format!("Telegram Outbox {suffix}"),
                format!("tg-outbox-{suffix}"),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");
    account_id
}

async fn insert_edit_command(pool: &sqlx::PgPool, account_id: &str, command_id: &str) {
    lifecycle::insert_command(
        pool,
        command_id,
        account_id,
        "edit",
        command_id,
        "-100telegram-outbox",
        Some("-100telegram-outbox:42"),
        "available",
        "provider_write",
        "not_required",
        "makosh-frontend",
        json!({"new_text": "edited text"}),
        json!({"provider_chat_id": "-100telegram-outbox", "provider_message_id": "-100telegram-outbox:42"}),
        json!({"source": "test"}),
    )
    .await
    .expect("insert command");
}
```

### `backend/tests/telegram_participant_capability_gates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_participant_capability_gates.rs`
- Size bytes / Размер в байтах: `7592`
- Included characters / Включено символов: `7592`
- Truncated / Обрезано: `no`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-participant-capability-gates-secret";

#[tokio::test]
async fn fixture_account_blocks_members_sync_before_audit_or_events() {
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
            "tdlib_data_path": "docker/data/telegram/test-participant-capability-gates",
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
    let response = app
        .clone()
        .oneshot(json_post(
            &format!(
                "/api/v1/integrations/telegram/provider-sync/conversations/{telegram_chat_id}/members"
            ),
            json!({}),
        ))
        .await
        .expect("members sync response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let _body = json_body(response).await;

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE operation = 'telegram.participants.sync' AND target_id = $1",
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("audit count");
    assert_eq!(audit_count, 0);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type LIKE 'telegram.sync.%' AND subject->>'id' = $1",
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("event count");
    assert_eq!(event_count, 0);
}

#[tokio::test]
async fn fixture_account_blocks_join_and_leave_before_command_enqueue() {
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
            "tdlib_data_path": "docker/data/telegram/test-participant-capability-join-leave",
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
    let join_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/telegram/provider-commands/conversations/join",
            json!({
                "account_id": "acct-1",
                "provider_chat_id": "provider-chat-1"
            }),
        ))
        .await
        .expect("join response");
    assert_eq!(join_response.status(), StatusCode::BAD_REQUEST);

    let leave_response = app
        .clone()
        .oneshot(json_post(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/leave"
            ),
            json!({
                "account_id": "acct-1",
                "provider_chat_id": "provider-chat-1"
            }),
        ))
        .await
        .expect("leave response");
    assert_eq!(leave_response.status(), StatusCode::BAD_REQUEST);

    let command_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_provider_write_commands WHERE account_id = 'acct-1' AND command_kind IN ('join', 'leave')",
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

### `backend/tests/telegram_participant_reconciliation_absence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_participant_reconciliation_absence.rs`
- Size bytes / Размер в байтах: `5645`
- Included characters / Включено символов: `5645`
- Truncated / Обрезано: `no`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::{Row, query};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::participants::reconcile_leave_commands_from_exhaustive_absence;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-participant-absence-test-secret";

#[tokio::test]
async fn telegram_exhaustive_roster_absence_reconciles_self_leave_command() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database.clone(),
    );
    let pool = database.pool().expect("configured pool").clone();
    let observed_at = Utc::now();

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": "acct-1",
            "provider_kind": "telegram_user",
            "display_name": "Telegram Exhaustive Leave",
            "external_account_id": "telegram:12345",
            "tdlib_data_path": "docker/data/telegram/test-exhaustive-absence",
            "transcription_enabled": false
        }),
    )
    .await;

    query(
        r#"
        INSERT INTO telegram_provider_write_commands (
            command_id,
            account_id,
            command_kind,
            idempotency_key,
            provider_chat_id,
            provider_message_id,
            target_ref,
            payload,
            capability_state,
            action_class,
            confirmation_decision,
            status,
            retry_count,
            max_retries,
            result_payload,
            audit_metadata,
            actor_id,
            happened_at,
            reconciliation_status
        )
        VALUES (
            'cmd-leave-exhaustive-1',
            'acct-1',
            'leave',
            'idem-leave-exhaustive-1',
            'provider-chat-1',
            NULL,
            '{}'::jsonb,
            '{}'::jsonb,
            'available',
            'provider_write',
            'confirmed',
            'executing',
            0,
            3,
            '{}'::jsonb,
            '{}'::jsonb,
            'makosh-frontend',
            NOW() - INTERVAL '1 minute',
            'awaiting_provider'
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert leave command");

    let commands = reconcile_leave_commands_from_exhaustive_absence(
        &pool,
        "acct-1",
        "provider-chat-1",
        "user:12345",
        observed_at,
        "tdlib.getSupergroupMembers.exhaustive_absence",
    )
    .await
    .expect("reconcile exhaustive absence");

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command_id, "cmd-leave-exhaustive-1");
    assert_eq!(commands[0].status, "completed");
    assert_eq!(commands[0].reconciliation_status, "observed");
    assert_eq!(
        commands[0].provider_state["membership_state"],
        "absent_exhaustive"
    );
    assert_eq!(commands[0].provider_state["status"], Value::Null);
    assert_eq!(commands[0].provider_state["role"], Value::Null);
    assert_eq!(
        commands[0].provider_state["observed_via"],
        "tdlib.getSupergroupMembers.exhaustive_absence"
    );

    let row = query(
        r#"
        SELECT status, reconciliation_status, provider_state, result_payload, completed_at
        FROM telegram_provider_write_commands
        WHERE command_id = 'cmd-leave-exhaustive-1'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("reconciled leave command row");
    let provider_state: Value = row.try_get("provider_state").expect("provider state");
    let result_payload: Value = row.try_get("result_payload").expect("result payload");
    assert_eq!(
        row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        row.try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert_eq!(provider_state["membership_state"], "absent_exhaustive");
    assert_eq!(
        result_payload["source"],
        "tdlib.getSupergroupMembers.exhaustive_absence"
    );
    assert_eq!(result_payload["provider_member_id"], "user:12345");
    assert!(
        row.try_get::<Option<chrono::DateTime<Utc>>, _>("completed_at")
            .expect("completed at")
            .is_some()
    );
}

async fn post_ok<S>(app: S, uri: &str, body: Value) -> Value
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-Макошь-Secret", LOCAL_API_TOKEN)
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json body")
}
```

### `backend/tests/telegram_participant_reconciliation_sources.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_participant_reconciliation_sources.rs`
- Size bytes / Размер в байтах: `5858`
- Included characters / Включено символов: `5858`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::commands::insert_command;
use makosh_hub_backend::integrations::telegram::client::participants::reconcile_join_commands_from_provider_roster_with_source;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-participant-reconcile-source-secret";

#[tokio::test]
async fn telegram_basic_group_roster_reconciliation_records_observed_source() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-basic-group-reconcile-{suffix}");
    let provider_chat_id = format!("basic-group-reconcile-chat-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Basic Group Reconcile",
            "external_account_id": "telegram:12345",
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("basic-group-reconcile-message-{suffix}"),
            "chat_kind": "group",
            "chat_title": "Basic Group Reconcile Room",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": format!("telegram-basic-group-reconcile-seed-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let command_id = format!("cmd-basic-group-reconcile-{suffix}");
    insert_command(
        &pool,
        &command_id,
        &account_id,
        "join",
        &format!("join:manual:{suffix}"),
        &provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "membership_state": "present",
        }),
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed join command");

    let observed_at = sqlx::query_scalar("SELECT now()")
        .fetch_one(&pool)
        .await
        .expect("observed at");
    let commands = reconcile_join_commands_from_provider_roster_with_source(
        &pool,
        &account_id,
        &provider_chat_id,
        "user:12345",
        observed_at,
        "tdlib.getBasicGroupFullInfo",
    )
    .await
    .expect("reconciled join commands");

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command_id, command_id);
    assert_eq!(commands[0].status, "completed");
    assert_eq!(commands[0].reconciliation_status, "observed");
    assert_eq!(
        commands[0].provider_state["observed_via"],
        "tdlib.getBasicGroupFullInfo"
    );
    assert_eq!(
        commands[0].result_payload["source"],
        "tdlib.getBasicGroupFullInfo"
    );

    let row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, provider_state, result_payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled command row");
    let provider_state: Value = row.try_get("provider_state").expect("provider state");
    let result_payload: Value = row.try_get("result_payload").expect("result payload");
    assert_eq!(
        row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        row.try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert_eq!(
        provider_state["observed_via"],
        "tdlib.getBasicGroupFullInfo"
    );
    assert_eq!(provider_state["membership_state"], "present");
    assert_eq!(result_payload["source"], "tdlib.getBasicGroupFullInfo");
}

async fn post_ok<S>(app: S, path: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post(path, body))
        .await
        .expect("post response");
    assert_eq!(response.status(), StatusCode::OK);
}

fn json_post(path: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
        .to_string()
}
```

### `backend/tests/telegram_participants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_participants.rs`
- Size bytes / Размер в байтах: `21771`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
};
use makosh_hub_backend::integrations::telegram::client::participants::{
    reconcile_join_commands_from_provider_roster, reconcile_leave_commands_from_provider_roster,
    telegram_self_provider_member_id,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-participants-test-secret";

#[tokio::test]
async fn telegram_members_route_prefers_provider_roster_over_message_heuristic() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-participants-{suffix}");
    let provider_chat_id = format!("participants-chat-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Participants",
            "external_account_id": format!("tg-participants-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    for (index, (sender_id, sender_display_name)) in [
        (format!("sender-a-{suffix}"), "Alice"),
        (format!("sender-b-{suffix}"), "Bob"),
        (format!("sender-a-{suffix}"), "Alice"),
    ]
    .into_iter()
    .enumerate()
    {
        post_ok(
            app.clone(),
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("message-{suffix}-{index}"),
                "chat_kind": "group",
                "chat_title": "Provider Roster Room",
                "sender_id": sender_id,
                "sender_display_name": sender_display_name,
                "text": format!("message {index}"),
                "import_batch_id": format!("telegram-participants-seed-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
        )
        .await;
    }

    let telegram_chat_id = first_chat_id(app.clone(), &account_id).await;
    let fallback_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?limit=10"
        )))
        .await
        .expect("fallback members response");
    assert_eq!(fallback_response.status(), StatusCode::OK);
    let fallback_body = json_body(fallback_response).await;
    assert_eq!(fallback_body["items"].as_array().expect("items").len(), 2);
    assert_eq!(fallback_body["items"][0]["source"], "message_heuristic");
    assert_eq!(fallback_body["items"][0]["sender_display_name"], "Alice");
    assert_eq!(fallback_body["items"][0]["message_count"], 2);

    insert_provider_participant(
        &pool,
        &telegram_chat_id,
        &account_id,
        &provider_chat_id,
        &suffix,
    )
    .await;

    let provider_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?query=owner&role=owner&limit=10"
        )))
        .await
        .expect("provider members response");
    assert_eq!(provider_response.status(), StatusCode::OK);
    let provider_body = json_body(provider_response).await;
    let items = provider_body["items"].as_array().expect("provider items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["source"], "tdlib");
    assert_eq!(items[0]["provider_member_id"], "user:42");
    assert_eq!(items[0]["sender_id"], "user:42");
    assert_eq!(items[0]["sender_display_name"], "Owner User");
    assert_eq!(items[0]["role"], "owner");
    assert_eq!(items[0]["status"], "creator");
    assert_eq!(items[0]["is_admin"], true);
    assert_eq!(items[0]["is_owner"], true);
    assert_eq!(items[0]["permissions"]["can_invite_users"], true);
    assert_eq!(items[0]["message_count"], 0);
}

#[tokio::test]
async fn telegram_join_leave_routes_enqueue_provider_write_commands() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-join-leave-{suffix}");
    let provider_chat_id = format!("join-leave-chat-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    seed_tdlib_qr_account(
        &pool,
        &account_id,
        "Telegram Join Leave",
        &format!("tg-join-leave-{suffix}"),
    )
    .await;
    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("join-leave-message-{suffix}"),
            "chat_kind": "group",
            "chat_title": "Join Leave Room",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": format!("telegram-join-leave-seed-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let telegram_chat_id = first_chat_id(app.clone(), &account_id).await;
    let join_body = command_response(
        app.clone(),
        "/api/v1/integrations/telegram/provider-commands/conversations/join",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id
        }),
    )
    .await;
    assert_eq!(join_body["action"], "join");
    assert_eq!(join_body["status"], "queued");
    assert_eq!(join_body["telegram_chat_id"], Value::Null);

    let leave_body = command_response(
        app.clone(),
        &format!(
            "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/leave"
        ),
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id
        }),
    )
    .await;
    assert_eq!(leave_body["action"], "leave");
    assert_eq!(leave_body["status"], "queued");
    assert_eq!(leave_body["telegram_chat_id"], json!(telegram_chat_id));

    assert_command_row(
        &pool,
        join_body["command_id"].as_str().expect("join command id"),
        "join",
        &provider_chat_id,
        None,
    )
    .await;
    assert_command_row(
        &pool,
        leave_body["command_id"].as_str().expect("leave command id"),
        "leave",
        &provider_chat_id,
        Some(&telegram_chat_id),
    )
    .await;
}

#[tokio::test]
async fn telegram_roster_sync_reconciles_join_only_after_self_member_is_observed() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-join-reconcile-{suffix}");
    let provider_chat_id = format!("join-reconcile-chat-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    seed_tdlib_qr_account(
        &pool,
        &account_id,
        "Telegram Join Reconcile",
        "telegram:12345",
    )
    .await;
    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("join-reconcile-message-{suffix}"),
            "chat_kind": "group",
            "chat_title": "Join Reconcile Room",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": format!("telegram-join-reconcile-seed-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let join_body = command_response(
        app.clone(),
        "/api/v1/integrations/telegram/provider-commands/conversations/join",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id
        }),
    )
    .await;
    let command_id = join_body["command_id"].as_str().expect("command id");
    assert_eq!(
        telegram_self_provider_member_id("telegram:12345").as_deref(),
        Some("user:12345")
    );
    assert_eq!(
        telegram_self_provider_member_id(&format!("fixture-{suffix}")),
        None
    );

    let observed_at = sqlx::query_scalar("SELECT now()")
        .fetch_one(&pool)
        .await
        .expect("observed at");
    let commands = reconcile_join_commands_from_provider_roster(
        &pool,
        &account_id,
        &provider_chat_id,
        "user:12345",
        observed_at,
    )
    .await
    .expect("reconciled join commands");

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command_id, command_id);
    assert_eq!(commands[0].status, "completed");
    assert_eq!(commands[0].reconciliation_status, "observed");
    assert_eq!(
        commands[0].provider_state["observed_via"],
        "tdlib.getSupergroupMembers"
    );
    assert_eq!(commands[0].provider_state["membership_state"], "present");
    assert_eq!(
        commands[0].provider_state["provider_member_id"],
        "user:12345"
    );
    assert!(commands[0].provider_observed_at.is_some());
    assert!(commands[0].completed_at.is_some());

    let row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, provider_state, result_payload, completed_at
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled command row");
    let provider_state: Value = row.try_get("provider_state").expect("provider state");
    let result_payload: Value = row.try_get("result_payload").expect("result payload");
    assert_eq!(
        row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        row.try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert_eq!(provider_state["membership_state"], "present");
    assert_eq!(result_payload["source"], "tdlib.getSupergroupMembers");
    assert!(
        row.try_get::<Option<chrono::DateTime<Utc>>, _>("completed_at")
            .expect("completed at")
            .is_some()
    );
}

#[tokio::test]
async fn telegram_roster_sync_reconc
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_qr_login.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_qr_login.rs`
- Size bytes / Размер в байтах: `7603`
- Included characters / Включено символов: `7603`
- Truncated / Обрезано: `no`

```rust
mod telegram_support;

use std::env;

use axum::http::StatusCode;
use serde_json::json;
use testkit::context::TestContext;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, delete_request_with_token, get_request_with_token, json_body,
    json_post_request_with_actor,
};

#[tokio::test]
async fn telegram_qr_login_start_reports_tdlib_runtime_unavailable() {
    let app = build_router_with_database(
        testkit::app::config_with_secret(LOCAL_API_TOKEN)
            .with_test_tdjson_path("/tmp/makosh-test-missing-libtdjson.dylib"),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/login/qr/start",
            json!({
                "account_id": "telegram-qr",
                "display_name": "Telegram QR",
                "external_account_id": "qr-login:telegram-qr",
                "api_id": 12345,
                "api_hash": "telegram-api-hash",
                "session_encryption_key": "telegram-session-key",
                "tdlib_data_path": "docker/data/telegram/telegram-qr",
                "transcription_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR login response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_tdlib_runtime_unavailable"));
}

#[tokio::test]
async fn telegram_qr_login_start_uses_configured_app_credentials_when_payload_omits_them() {
    let app = build_router_with_database(
        testkit::app::config_with_secret(LOCAL_API_TOKEN)
            .with_test_tdjson_path("/tmp/makosh-test-missing-libtdjson.dylib")
            .with_test_telegram_app_credentials(12345, "telegram-api-hash"),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/login/qr/start",
            json!({
                "account_id": "telegram-qr-configured",
                "display_name": "Telegram QR Configured",
                "external_account_id": "qr-login:telegram-qr-configured",
                "session_encryption_key": "telegram-session-key",
                "tdlib_data_path": "docker/data/telegram/telegram-qr-configured",
                "transcription_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR login response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_tdlib_runtime_unavailable"));
}

#[tokio::test]
async fn telegram_live_smoke_syncs_configured_account_when_explicitly_enabled() {
    if env::var("MAKOSH_TELEGRAM_LIVE_SMOKE").ok().as_deref() != Some("1") {
        eprintln!("skipping live Telegram TDLib smoke test: MAKOSH_TELEGRAM_LIVE_SMOKE is not 1");
        return;
    }

    let account_id = env::var("MAKOSH_TELEGRAM_LIVE_ACCOUNT_ID")
        .expect("MAKOSH_TELEGRAM_LIVE_ACCOUNT_ID must be set");
    let provider_chat_id =
        env::var("MAKOSH_TELEGRAM_LIVE_CHAT_ID").expect("MAKOSH_TELEGRAM_LIVE_CHAT_ID must be set");
    let tdjson_path = env::var("MAKOSH_TDJSON_PATH").expect("MAKOSH_TDJSON_PATH must be set");
    let telegram_api_id = env::var("MAKOSH_TELEGRAM_API_ID")
        .expect("MAKOSH_TELEGRAM_API_ID must be set")
        .parse::<i64>()
        .expect("MAKOSH_TELEGRAM_API_ID must be a positive integer");
    let telegram_api_hash =
        env::var("MAKOSH_TELEGRAM_API_HASH").expect("MAKOSH_TELEGRAM_API_HASH must be set");
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = test_context.database();
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url)
            .with_test_tdjson_path(tdjson_path)
            .with_test_telegram_app_credentials(telegram_api_id, telegram_api_hash),
        database,
    );

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
    assert_eq!(start_body["runtime_kind"], json!("tdlib_qr_authorized"));
    assert_eq!(start_body["status"], json!("running"));

    let history_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/provider-sync/history",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "limit": 25
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("history sync response");
    assert_eq!(history_response.status(), StatusCode::OK);
    let history_body = json_body(history_response).await;
    assert_eq!(history_body["account_id"], json!(account_id));
    assert_eq!(history_body["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(history_body["runtime_kind"], json!("tdlib_qr_authorized"));
    assert_eq!(history_body["status"], json!("synced"));
}

#[tokio::test]
async fn telegram_qr_login_status_unknown_setup_returns_json_not_found() {
    let app = build_router_with_database(
        testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/integrations/telegram/login/qr/missing-setup",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR status response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_qr_login_not_found"));
}

#[tokio::test]
async fn telegram_qr_login_password_unknown_setup_returns_json_not_found() {
    let app = build_router_with_database(
        testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/login/qr/missing-setup/password",
            json!({ "password": "test-password" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR password response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_qr_login_not_found"));
}

#[tokio::test]
async fn telegram_qr_login_cancel_unknown_setup_returns_json_not_found() {
    let app = build_router_with_database(
        testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let response = app
        .oneshot(delete_request_with_token(
            "/api/v1/integrations/telegram/login/qr/missing-setup",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR cancel response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_qr_login_not_found"));
}
```

### `backend/tests/telegram_reactions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_reactions.rs`
- Size bytes / Размер в байтах: `4649`
- Included characters / Включено символов: `4634`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
};
use makosh_hub_backend::integrations::telegram::client::lifecycle;
use makosh_hub_backend::integrations::telegram::client::reconcile_reaction_commands_from_provider_reactions;
use testkit::context::TestContext;

#[tokio::test]
async fn telegram_provider_reactions_reconcile_react_and_unreact_commands() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = create_telegram_account(&pool, "reaction-reconcile", "telegram:123").await;
    let provider_chat_id = "-100reaction-reconcile";
    let provider_message_id = "-100reaction-reconcile:42";

    insert_reaction_command(
        &pool,
        &account_id,
        "tcmd_react_observed",
        "react",
        provider_chat_id,
        provider_message_id,
        "👍",
    )
    .await;
    insert_reaction_command(
        &pool,
        &account_id,
        "tcmd_unreact_observed",
        "unreact",
        provider_chat_id,
        provider_message_id,
        "🔥",
    )
    .await;
    insert_reaction_command(
        &pool,
        &account_id,
        "tcmd_react_still_pending",
        "react",
        provider_chat_id,
        provider_message_id,
        "😎",
    )
    .await;

    let reconciled = reconcile_reaction_commands_from_provider_reactions(
        &pool,
        &account_id,
        provider_chat_id,
        provider_message_id,
        &["👍".to_owned()],
        Utc::now(),
        "tdlib.interaction_info.reactions",
    )
    .await
    .expect("reconcile reaction commands");

    assert_eq!(reconciled.len(), 3);
    assert!(
        reconciled
            .iter()
            .any(|command| command.command_id == "tcmd_react_observed"
                && command.status == "completed"
                && command.reconciliation_status == "observed")
    );
    assert!(
        reconciled
            .iter()
            .any(|command| command.command_id == "tcmd_unreact_observed"
                && command.status == "completed"
                && command.reconciliation_status == "observed")
    );
    assert!(
        reconciled
            .iter()
            .any(|command| command.command_id == "tcmd_react_still_pending"
                && command.status == "failed"
                && command.reconciliation_status == "mismatch")
    );

    let commands = lifecycle::list_commands(&pool, &account_id, 10)
        .await
        .expect("list commands");
    let pending = commands
        .iter()
        .find(|command| command.command_id == "tcmd_react_still_pending")
        .expect("pending command");
    assert_eq!(pending.status, "failed");
    assert_eq!(pending.reconciliation_status, "mismatch");
    assert_eq!(
        pending.last_error.as_deref(),
        Some("Provider observed a different reaction state than requested")
    );
    assert_eq!(pending.provider_state["reaction_emoji"], json!("😎"));
    assert_eq!(pending.provider_state["expected_is_chosen"], json!(true));
    assert_eq!(pending.provider_state["observed_is_chosen"], json!(false));
}

async fn create_telegram_account(
    pool: &sqlx::PgPool,
    suffix: &str,
    external_account_id: &str,
) -> String {
    let account_id = format!("telegram-reactions-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::TelegramUser,
                format!("Telegram Reactions {suffix}"),
                external_account_id.to_owned(),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");
    account_id
}

async fn insert_reaction_command(
    pool: &sqlx::PgPool,
    account_id: &str,
    command_id: &str,
    command_kind: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    reaction_emoji: &str,
) {
    lifecycle::insert_command(
        pool,
        command_id,
        account_id,
        command_kind,
        command_id,
        provider_chat_id,
        Some(provider_message_id),
        "available",
        "provider_write",
        "not_required",
        "makosh-frontend",
        json!({"reaction_emoji": reaction_emoji}),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
        }),
        json!({"source": "test"}),
    )
    .await
    .expect("insert reaction command");
}
```

### `backend/tests/telegram_reference_idempotency.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_reference_idempotency.rs`
- Size bytes / Размер в байтах: `5500`
- Included characters / Включено символов: `5500`
- Truncated / Обрезано: `no`

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
    LOCAL_API_TOKEN, assert_ok, json_body, json_post_request_with_actor, unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn telegram_reference_inserts_return_existing_rows_on_conflict() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-reference-idempotent-{suffix}");
    let chat_id = format!("reference-idempotent-chat-{suffix}");
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
            "display_name": "Telegram Reference Idempotency",
            "external_account_id": format!("tg-reference-idempotent-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let root_message_id = create_reference_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("reference-idempotent-root-{suffix}"),
        "Root Sender",
        "Root body",
        &suffix,
    )
    .await;
    let reply_message_id = create_reference_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("reference-idempotent-reply-{suffix}"),
        "Reply Sender",
        "Reply body",
        &suffix,
    )
    .await;
    let forward_message_id = create_reference_message(
        app,
        &account_id,
        &chat_id,
        &format!("reference-idempotent-forward-{suffix}"),
        "Forward Sender",
        "Forward body",
        &suffix,
    )
    .await;

    let pool = ctx.pool();
    let first_reply = lifecycle::insert_reply_ref(
        pool,
        &reply_message_id,
        &root_message_id,
        &account_id,
        &chat_id,
        &format!("reference-idempotent-reply-{suffix}"),
        &format!("reference-idempotent-root-{suffix}"),
        false,
    )
    .await
    .expect("first reply ref");
    let second_reply = lifecycle::insert_reply_ref(
        pool,
        &reply_message_id,
        &root_message_id,
        &account_id,
        &chat_id,
        &format!("reference-idempotent-reply-{suffix}"),
        &format!("reference-idempotent-root-{suffix}"),
        false,
    )
    .await
    .expect("second reply ref returns existing row");
    assert_eq!(second_reply.reply_ref_id, first_reply.reply_ref_id);

    let forward_date = chrono::DateTime::parse_from_rfc3339("2026-06-05T11:00:00Z")
        .expect("timestamp")
        .with_timezone(&Utc);
    let first_forward = lifecycle::insert_forward_ref(
        pool,
        &forward_message_id,
        &account_id,
        &chat_id,
        &format!("reference-idempotent-forward-{suffix}"),
        Some("origin-chat-idempotent"),
        Some("origin-message-idempotent"),
        Some("origin-sender-idempotent"),
        Some("Original Author"),
        Some(forward_date),
    )
    .await
    .expect("first forward ref");
    let second_forward = lifecycle::insert_forward_ref(
        pool,
        &forward_message_id,
        &account_id,
        &chat_id,
        &format!("reference-idempotent-forward-{suffix}"),
        Some("origin-chat-idempotent"),
        Some("origin-message-idempotent"),
        Some("origin-sender-idempotent"),
        Some("Original Author"),
        Some(forward_date),
    )
    .await
    .expect("second forward ref returns existing row");
    assert_eq!(second_forward.forward_ref_id, first_forward.forward_ref_id);
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
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_kind": "group",
                "chat_title": "Reference Idempotency Room",
                "sender_id": format!("sender-{sender_display_name}-{suffix}"),
                "sender_display_name": sender_display_name,
                "text": text,
                "import_batch_id": format!("telegram-reference-idempotent-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reference message response");
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await["message_id"]
        .as_str()
        .expect("message id")
        .to_owned()
}
```
