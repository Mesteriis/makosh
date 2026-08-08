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

- Chunk ID / ID чанка: `089-test-backend-part-012`
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

### `backend/tests/telegram_account_setup_capabilities.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_account_setup_capabilities.rs`
- Size bytes / Размер в байтах: `19645`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::secrets::{SecretKind, SecretReferenceStore, SecretStoreKind};
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_capability_status, assert_ok, get_request_with_token, json_body,
    json_post_request_with_actor, unique_suffix, vault_entropy_events,
};
use testkit::context::TestContext;
#[tokio::test]
async fn telegram_live_account_setup_stores_bot_token_in_host_vault() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-bot-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_dir.path().join("vault").to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    vault_dir
                        .path()
                        .join("dev")
                        .join("master.key")
                        .to_str()
                        .expect("dev key path"),
                ),
            ])
            .expect("config"),
        database,
    );

    let entropy_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);
    let create_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/vault/create",
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_bot",
                "display_name": "Telegram Bot",
                "external_account_id": format!("@makosh_bot_{suffix}"),
                "bot_token": "123456:telegram-bot-token",
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account_id"], json!(account_id));
    assert_eq!(body["provider_kind"], json!("telegram_bot"));
    assert_eq!(body["runtime"], json!("live_blocked"));
    assert_eq!(
        body["credential_bindings"][0]["secret_purpose"],
        json!("telegram_bot_token")
    );
    assert_eq!(
        body["credential_bindings"][0]["secret_kind"],
        json!("api_token")
    );
    assert_eq!(
        body["credential_bindings"][0]["store_kind"],
        json!("host_vault")
    );

    let account = sqlx::query(
        "SELECT provider_kind, config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider account");
    let provider_kind: String = account.try_get("provider_kind").expect("provider kind");
    let config: Value = account.try_get("config").expect("config");
    assert_eq!(provider_kind, "telegram_bot");
    assert_eq!(config["runtime"], json!("live_blocked"));
    assert_eq!(config["transcription_enabled"], json!(false));
    assert!(config.get("bot_token").is_none());
    assert!(config.get("api_hash").is_none());

    let secret_ref = body["credential_bindings"][0]["secret_ref"]
        .as_str()
        .expect("secret ref");
    let secret_store = SecretReferenceStore::new(pool.clone());
    let reference = secret_store
        .secret_reference(secret_ref)
        .await
        .expect("secret reference query")
        .expect("secret reference exists");
    assert_eq!(reference.secret_kind, SecretKind::ApiToken);
    assert_eq!(reference.store_kind, SecretStoreKind::HostVault);
    assert_eq!(reference.metadata["provider"], json!("telegram_bot"));
    assert_eq!(reference.metadata["account_id"], json!(account_id));

    let database_payload_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM encrypted_secret_vault_entries WHERE secret_ref = $1",
    )
    .bind(secret_ref)
    .fetch_one(&pool)
    .await
    .expect("database payload count");
    assert_eq!(database_payload_count, 0);
}
#[tokio::test]
async fn telegram_qr_authorized_account_setup_persists_metadata_without_host_vault_secret() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-user-{suffix}");
    let tdlib_data_path = format!("docker/data/telegram/{account_id}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_dir.path().join("vault").to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    vault_dir
                        .path()
                        .join("dev")
                        .join("master.key")
                        .to_str()
                        .expect("dev key path"),
                ),
                ("MAKOSH_TELEGRAM_API_ID", "12345"),
                ("MAKOSH_TELEGRAM_API_HASH", "telegram-api-hash"),
            ])
            .expect("config"),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "@second_account",
                "external_account_id": format!("telegram:{suffix}"),
                "tdlib_data_path": tdlib_data_path,
                "transcription_enabled": false,
                "qr_authorized": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account_id"], json!(account_id));
    assert_eq!(body["provider_kind"], json!("telegram_user"));
    assert_eq!(body["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(body["credential_bindings"], json!([]));

    let account = sqlx::query(
        "SELECT provider_kind, display_name, external_account_id, config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider account");
    let provider_kind: String = account.try_get("provider_kind").expect("provider kind");
    let display_name: String = account.try_get("display_name").expect("display name");
    let external_account_id: String = account
        .try_get("external_account_id")
        .expect("external account id");
    let config: Value = account.try_get("config").expect("config");
    assert_eq!(provider_kind, "telegram_user");
    assert_eq!(display_name, "@second_account");
    assert_eq!(external_account_id, format!("telegram:{suffix}"));
    assert_eq!(config["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(config["tdlib_data_path"], json!(tdlib_data_path));
    assert_eq!(config["transcription_enabled"], json!(false));
    assert!(config.get("api_hash").is_none());
    assert!(config.get("bot_token").is_none());

    let binding_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM communication_provider_account_secret_refs WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("binding count");
    assert_eq!(binding_count, 0);
}
#[tokio::test]
async fn telegram_finalized_qr_account_setup_infers_qr_authorized_runtime() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-user-inferred-{suffix}");
    let tdlib_data_path = format!("docker/data/telegram/{account_id}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_dir.path().join("vault").to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    vault_dir
                        .path()
                        .join("dev")
                        .join("master.key")
                        .to_str()
                        .expect("dev key path"),
                ),
                ("MAKOSH_TELEGRAM_API_ID", "12345"),
                ("MAKOSH_TELEGRAM_API_HASH", "telegram-api-hash"),
            ])
            .expect("config"),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "@inferred_qr",
                "external_account_id": format!("telegram:{suffix}"),
                "tdlib_data_path": tdlib_data_path,
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(body["credential_bindings"], json!([]));

    let config: Value = sqlx::query_scalar(
        "SELECT config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider account config");
    assert_eq!(config["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(config["tdlib_data_path"], json!(tdlib_data_path));
    assert!(config.get("api_hash").is_none());

    let binding_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM communication_provider_account_secret_refs WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("binding count");
    assert_eq!(binding_count, 0);
}
#[tokio::test]
async fn telegram_live_account_setup_api_require
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_architecture.rs`
- Size bytes / Размер в байтах: `2645`
- Included characters / Включено символов: `2645`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_IMPLEMENTATION_FILE_LINES: usize = 700;

#[test]
fn telegram_implementation_files_stay_below_architecture_line_limit() {
    let backend_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = backend_manifest_dir
        .parent()
        .expect("backend crate must live under repository root");
    let roots = [
        backend_manifest_dir.join("src/app/api_support"),
        backend_manifest_dir.join("src/integrations/telegram"),
        backend_manifest_dir.join("tests"),
        repo_root.join("frontend/src/integrations/telegram"),
    ];

    let mut violations = Vec::new();
    for root in roots {
        collect_line_limit_violations(&root, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "Telegram source/test files exceed {MAX_IMPLEMENTATION_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_line_limit_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root).unwrap_or_else(|error| {
        panic!("failed to read Telegram implementation directory {root:?}: {error}")
    });

    for entry in entries {
        let entry = entry.expect("failed to read Telegram implementation directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_line_limit_violations(&path, violations);
            continue;
        }
        if !is_implementation_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_IMPLEMENTATION_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_implementation_file(path: &Path) -> bool {
    let is_backend_test_file = path
        .components()
        .any(|component| component.as_os_str() == "tests");
    let is_telegram_test_support_file = path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value.starts_with("telegram"))
    });
    if path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .is_some_and(|file_name| file_name.starts_with("telegram") && file_name.ends_with(".rs"))
    {
        return true;
    }
    if is_backend_test_file {
        return is_telegram_test_support_file;
    }

    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("rs" | "ts" | "vue")
    )
}
```

### `backend/tests/telegram_commands_query_filters.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_commands_query_filters.rs`
- Size bytes / Размер в байтах: `3883`
- Included characters / Включено символов: `3883`
- Truncated / Обрезано: `no`

```rust
mod telegram_support;

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::lifecycle;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn telegram_commands_endpoint_filters_by_chat_and_kind() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-command-filter-{suffix}");
    let provider_chat_id = format!("command-filter-chat-{suffix}");
    let other_chat_id = format!("command-filter-other-{suffix}");

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
            "display_name": "Telegram Command Filter",
            "external_account_id": format!("tg-command-filter-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    insert_command(
        &pool,
        &account_id,
        "cmd-filter-mark-read",
        "mark_read",
        &provider_chat_id,
        &format!("{provider_chat_id}:1"),
    )
    .await;
    insert_command(
        &pool,
        &account_id,
        "cmd-filter-join",
        "join",
        &provider_chat_id,
        &format!("{provider_chat_id}:2"),
    )
    .await;
    insert_command(
        &pool,
        &account_id,
        "cmd-filter-other-chat",
        "mark_read",
        &other_chat_id,
        &format!("{other_chat_id}:1"),
    )
    .await;
    insert_command(
        &pool,
        &account_id,
        "cmd-filter-other-message",
        "mark_read",
        &provider_chat_id,
        &format!("{provider_chat_id}:99"),
    )
    .await;

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/integrations/telegram/commands?account_id={account_id}&provider_chat_id={provider_chat_id}&provider_message_id={provider_chat_id}:1&command_kinds=mark_read&limit=20"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("commands response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("command items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["command_id"], json!("cmd-filter-mark-read"));
    assert_eq!(items[0]["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(items[0]["command_kind"], json!("mark_read"));
}

async fn insert_command(
    pool: &sqlx::PgPool,
    account_id: &str,
    command_id: &str,
    command_kind: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
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
        "confirmed",
        "makosh-frontend",
        json!({"source": "telegram_commands_query_filters"}),
        json!({"provider_chat_id": provider_chat_id}),
        json!({"source": "test"}),
    )
    .await
    .expect("insert command");
}
```

### `backend/tests/telegram_core.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_core.rs`
- Size bytes / Размер в байтах: `24750`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
mod telegram_support;

use testkit::context::TestContext;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_capability_status, assert_ok, get_request_with_token, json_body,
    json_post_request_with_actor, unique_suffix,
};
#[tokio::test]
async fn telegram_fixture_message_ingestion_refreshes_decision_and_obligation_candidates_against_postgres()
 {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-candidate-{suffix}");
    let chat_id = format!("tg-candidate-chat-{suffix}");
    let decision_title = format!("Use Telegram evidence for shared memory {suffix}");
    let decision_rationale = "channel context must feed the same domain model";
    let obligation_statement = format!("send the Telegram alignment note {suffix}");
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
                "display_name": "Telegram Candidate Source",
                "external_account_id": format!("tg-candidate-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/candidate-{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("tg-candidate-message-{suffix}"),
                "chat_kind": "private",
                "chat_title": "Telegram Candidate Review",
                "sender_id": format!("telegram-candidate-sender-{suffix}"),
                "sender_display_name": "Telegram Candidate Sender",
                "text": format!(
                    "Decision: {decision_title} because {decision_rationale}. I will {obligation_statement} by Friday 5pm."
                ),
                "import_batch_id": format!("telegram-candidate-fixture-{suffix}"),
                "occurred_at": "2026-06-06T12:30:00Z",
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
    let observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let raw_signal_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.telegram.message.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("raw telegram signal count");
    let accepted_signal_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.telegram.message'",
    )
    .fetch_one(&pool)
    .await
    .expect("accepted telegram signal count");
    let legacy_integration_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type LIKE 'integration.telegram.%'",
    )
    .fetch_one(&pool)
    .await
    .expect("legacy telegram integration event count");
    assert_eq!(raw_signal_count, 1);
    assert_eq!(accepted_signal_count, 1);
    assert_eq!(legacy_integration_count, 0);
    let trace_rows = sqlx::query(
        r#"
        SELECT event_id, event_type, causation_id, correlation_id
        FROM event_log
        WHERE correlation_id = $1
          AND event_type IN (
              'observation.captured.v1',
              'signal.raw.telegram.message.observed',
              'signal.accepted.telegram.message',
              'communication.message.recorded'
          )
        ORDER BY position ASC
        "#,
    )
    .bind(&observation_id)
    .fetch_all(&pool)
    .await
    .expect("telegram trace rows");
    assert_eq!(trace_rows.len(), 4);
    let observation_event_id = format!("event:v1:observation-captured:{observation_id}");
    let raw_event_id: String = trace_rows[1].try_get("event_id").expect("raw event id");
    let accepted_event_id: String = trace_rows[2]
        .try_get("event_id")
        .expect("accepted event id");
    assert_eq!(
        trace_rows[0]
            .try_get::<String, _>("event_id")
            .expect("observation event id"),
        observation_event_id
    );
    assert_eq!(
        trace_rows[1]
            .try_get::<Option<String>, _>("causation_id")
            .expect("raw causation")
            .as_deref(),
        Some(observation_event_id.as_str())
    );
    assert_eq!(
        trace_rows[2]
            .try_get::<Option<String>, _>("causation_id")
            .expect("accepted causation")
            .as_deref(),
        Some(raw_event_id.as_str())
    );
    assert_eq!(
        trace_rows[3]
            .try_get::<Option<String>, _>("causation_id")
            .expect("communication causation")
            .as_deref(),
        Some(accepted_event_id.as_str())
    );
    assert!(trace_rows.iter().all(|row| {
        row.try_get::<Option<String>, _>("correlation_id")
            .expect("trace correlation")
            .as_deref()
            == Some(observation_id.as_str())
    }));

    let decision_row: (String, String, String, String, String) = sqlx::query_as(
        r#"
        SELECT d.title, d.rationale, d.review_state, e.source_kind, e.source_id
        FROM decisions d
        JOIN decision_evidence e ON e.decision_id = d.decision_id
        WHERE e.source_kind = 'communication'
          AND e.source_id = $1
          AND d.title = $2
        "#,
    )
    .bind(&message_id)
    .bind(&decision_title)
    .fetch_one(&pool)
    .await
    .expect("Telegram message should create a suggested Decision candidate");
    assert_eq!(decision_row.1, decision_rationale);
    assert_eq!(decision_row.2, "suggested");
    assert_eq!(decision_row.3, "communication");
    assert_eq!(decision_row.4, message_id);

    let task_candidate_row: (String, String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT title, review_state, candidate_kind, due_text
        FROM task_candidates
        WHERE source_kind = 'observation'
          AND source_id = $1
          AND candidate_kind = 'obligation_task'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("Telegram message should create an obligation-derived task candidate");
    assert_eq!(task_candidate_row.0, obligation_statement);
    assert_eq!(task_candidate_row.1, "suggested");
    assert_eq!(task_candidate_row.2, "obligation_task");
    assert_eq!(task_candidate_row.3.as_deref(), Some("Friday 5pm"));

    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&message_id)
            .fetch_one(&pool)
            .await
            .expect("task count");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE statement = $1")
            .bind(&obligation_statement)
            .fetch_one(&pool)
            .await
            .expect("accepted obligation count");
    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn telegram_api_exercises_policy_and_call_foundation() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-user-{suffix}");
    let chat_id = format!("tg-chat-{suffix}");
    let policy_id = format!("policy-telegram-{suffix}");
    let template_id = format!("template-telegram-{suffix}");
    let call_id = format!("call-telegram-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    let capabilities_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/integrations/telegram/capabilities",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("capabilities response");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = json_body(capabilities_response).await;
    assert_eq!(capabilities_body["runtime_mode"], json!("fixture"));
    assert_eq!(
        capabilities_body["telegram_app_credentials_configured"],
        json!(false)
    );
    assert_eq!(capabilities_body["qr_login_ready"], json!(false));
    assert_capability_status(
        &capabilities_body,
        "telegram_fixture_runtime",
        "available",
        true,
    );
    assert_capability_status(&capabilities_body, "automation_dry_run", "available", true);
    assert_capability_status(&capabilities_body, "tdlib_live_runtime", "blocked", true);
    assert_capability_status(&capabilities_body, "automation_live_send", "blocked", true);
    assert_capability_status(
        &capabilities_body,
        "whisper_rs_speech_to_text",
        "blocked",
        true,
    );
    assert_capability_status(&capabilities_body, "topics.list", "degraded", false);
    assert_capability_status(&capabilities_body, "topics.create", "blocked", true);
    assert_capability_status(&capabilities_body, "topics.close", "blocked", true);
    assert!(
        capabilities_body["unsupported_features"]
            .as_array()
            .expect("unsupported features")
            .iter()
            .any(|feature| feature == "hidden_recording")
    );
    assert!(
        capabilities_body["planned_features"]
            .as_array()
            .expect("planned features")
            .iter()
            .any(|feature| feature == "bot_runtime")
    );
    assert!(
        capabilities_body["planned_features"]
            .as_array()
            .expect("planned features")
            .iter()
            .any(|feature| feature == "ai_review_flows")
    );
    assert!(
        !capabilities_body["unsupported_features"]
            .as_array()
            .expect("unsupported features")
            .iter()
            .any(|feature| feature == "forum_topic_mutations")
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram User",
                "external_account_id": format!("tg-user-{suffix}"),
                "tdlib_data_path"
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_dialog_actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_dialog_actions.rs`
- Size bytes / Размер в байтах: `7542`
- Included characters / Включено символов: `7536`
- Truncated / Обрезано: `no`

```rust
mod telegram_support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, delete_request_with_token, get_request_with_token, json_body,
    json_post_request_with_actor, unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn telegram_restore_and_reaction_actions_record_durable_command_rows() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-lifecycle-actions-{suffix}");
    let chat_id = format!("lifecycle-chat-{suffix}");
    let provider_message_id = format!("lifecycle-message-{suffix}");
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
            "display_name": "Telegram Lifecycle Actions",
            "external_account_id": format!("tg-lifecycle-{suffix}"),
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
                "provider_message_id": provider_message_id,
                "chat_kind": "private",
                "chat_title": "Lifecycle Action Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Pavel Sidorov",
                "text": "Lifecycle actions should create durable command rows.",
                "import_batch_id": format!("telegram-lifecycle-fixture-{suffix}"),
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

    let restore_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/restore-visibility"),
            json!({
                "command_id": format!("restore-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "reason": "manual_restore"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("restore response");
    assert_eq!(restore_response.status(), StatusCode::OK);

    let add_reaction_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/messages/{message_id}/reactions"),
            json!({
                "command_id": format!("react-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "reaction_emoji": "👍",
                "sender_id": "owner",
                "sender_display_name": "Owner"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("add reaction response");
    assert_eq!(add_reaction_response.status(), StatusCode::OK);

    let remove_reaction_response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/communications/messages/{message_id}/reactions?account_id={account_id}&provider_chat_id={chat_id}&provider_message_id={provider_message_id}&reaction_emoji=%F0%9F%91%8D&sender_id=owner&sender_display_name=Owner&command_id=unreact-{suffix}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("remove reaction response");
    assert_eq!(remove_reaction_response.status(), StatusCode::OK);

    let commands_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/commands?account_id={account_id}&limit=20"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("commands response");
    assert_eq!(commands_response.status(), StatusCode::OK);
    let commands_body = json_body(commands_response).await;
    let items = commands_body["items"].as_array().expect("command items");
    let kinds: Vec<&str> = items
        .iter()
        .filter_map(|item| item["command_kind"].as_str())
        .collect();

    for expected_kind in ["restore_visibility", "react", "unreact"] {
        assert!(
            kinds.iter().any(|kind| kind == &expected_kind),
            "expected command row for {expected_kind}, got {kinds:?}"
        );
    }

    let reaction_rows = sqlx::query(
        r#"
        SELECT reaction_id, is_active
        FROM telegram_message_reactions
        WHERE message_id = $1
          AND sender_id = 'owner'
          AND reaction_emoji = '👍'
        "#,
    )
    .bind(&message_id)
    .fetch_all(&pool)
    .await
    .expect("reaction rows");
    assert_eq!(reaction_rows.len(), 1);
    let reaction_id = reaction_rows[0].get::<String, _>("reaction_id");
    assert!(!reaction_rows[0].get::<bool, _>("is_active"));

    let reaction_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'message_reaction'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&reaction_id)
    .fetch_all(&pool)
    .await
    .expect("reaction observations");
    assert!(
        reaction_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_MESSAGE_REACTION"
                && row.get::<String, _>("relationship_kind") == "local_add"
                && row.get::<serde_json::Value, _>("payload")["is_active"] == json!(true)
        }),
        "local_add reaction observation must exist"
    );
    assert!(
        reaction_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_MESSAGE_REACTION"
                && row.get::<String, _>("relationship_kind") == "local_remove"
                && row.get::<serde_json::Value, _>("payload")["is_active"] == json!(false)
        }),
        "local_remove reaction observation must exist"
    );
}
```

### `backend/tests/telegram_dialog_capability_gates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_dialog_capability_gates.rs`
- Size bytes / Размер в байтах: `9045`
- Included characters / Включено символов: `9045`
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
async fn fixture_account_blocks_dialog_actions_before_side_effects() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-gates-{suffix}");
    let provider_chat_id = format!("dialog-gates-chat-{suffix}");
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
            "display_name": "Telegram Dialog Capability Gates",
            "external_account_id": format!("tg-dialog-gates-{suffix}"),
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
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("{provider_chat_id}:1"),
            "chat_kind": "private",
            "chat_title": "Dialog Capability Gates Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Dialog actions must respect capability gates before side effects.",
            "import_batch_id": format!("telegram-dialog-gates-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

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

    let detail_before = chat_detail(app.clone(), &telegram_chat_id).await;
    let initial_metadata = detail_before["item"]["metadata"].clone();
    let read_target_message_id = format!("{provider_chat_id}:777");

    for (action, body) in [
        (
            "pin",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "unpin",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "archive",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "unarchive",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "mute",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "unmute",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "read",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "last_read_inbox_provider_message_id": read_target_message_id
            }),
        ),
        (
            "unread",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "folders/7",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "folders/7/remove",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "folders/reassign",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "target_provider_folder_ids": [7]
            }),
        ),
    ] {
        let response = app
            .clone()
            .oneshot(json_post_request_with_actor(
                &format!(
                    "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/{action}"
                ),
                body,
                LOCAL_API_TOKEN,
            ))
            .await
            .expect("dialog action response");
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "expected {action} to be capability-blocked"
        );
    }

    let command_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_provider_write_commands WHERE account_id = $1 AND command_kind = ANY($2)",
    )
    .bind(&account_id)
    .bind(vec![
        "pin",
        "unpin",
        "archive",
        "unarchive",
        "mute",
        "unmute",
        "mark_read",
        "mark_unread",
        "folder_add",
        "folder_remove",
    ])
    .fetch_one(&pool)
    .await
    .expect("dialog command count");
    assert_eq!(command_count, 0);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE target_id = $1 AND operation = ANY($2)",
    )
    .bind(&telegram_chat_id)
    .bind(vec![
        "telegram.chat.pin",
        "telegram.chat.unpin",
        "telegram.chat.archive",
        "telegram.chat.unarchive",
        "telegram.chat.mute",
        "telegram.chat.unmute",
        "telegram.chat.mark_read",
        "telegram.chat.mark_unread",
        "telegram.chat.folder_add",
        "telegram.chat.folder_remove",
        "telegram.chat.folder_reassign",
    ])
    .fetch_one(&pool)
    .await
    .expect("dialog audit count");
    assert_eq!(audit_count, 0);

    let command_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.command.status_changed' AND payload->>'telegram_chat_id' = $1",
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("dialog command event count");
    assert_eq!(command_event_count, 0);

    let chat_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = ANY($1) AND payload->>'telegram_chat_id' = $2",
    )
    .bind(vec![
        "telegram.chat.updated",
        "telegram.chat.pinned",
        "telegram.chat.archived",
        "telegram.chat.muted",
    ])
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("dialog chat event count");
    assert_eq!(chat_event_count, 0);

    let detail_after = chat_detail(app, &telegram_chat_id).await;
    let final_metadata = &detail_after["item"]["metadata"];
    assert_eq!(final_metadata["is_pinned"], initial_metadata["is_pinned"]);
    assert_eq!(
        final_metadata["is_archived"],
        initial_metadata["is_archived"]
    );
    assert_eq!(final_metadata["is_muted"], initial_metadata["is_muted"]);
    assert_eq!(
        final_metadata["last_read_inbox_provider_message_id"],
        initial_metadata["last_read_inbox_provider_message_id"]
    );
    assert_eq!(
        final_metadata["unread_count"],
        initial_metadata["unread_count"]
    );
    assert_eq!(
        final_metadata["mention_count"],
        initial_metadata["mention_count"]
    );
}

async fn chat_detail<S>(app: S, telegram_chat_id: &str) -> Value
where
    S: tower::Service<axum::http::Request<axum::body::Body>, Response = axum::response::Response>
        + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations/{telegram_chat_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chat detail response");
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}
```

### `backend/tests/telegram_dialog_read_reconciliation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_dialog_read_reconciliation.rs`
- Size bytes / Размер в байтах: `18137`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
mod telegram_support;

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::json;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::lifecycle::{
    insert_command, new_command_id,
};
use makosh_hub_backend::integrations::telegram::client::{
    reconcile_mark_read_commands_from_provider_state, reconcile_pin_commands_from_provider_state,
};
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn mark_read_reconciliation_completes_targeted_read_commands_from_chat_read_inbox() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-reconcile-{suffix}");
    let provider_chat_id = format!("dialog-reconcile-chat-{suffix}");
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
            "display_name": "Telegram Dialog Read Reconcile",
            "external_account_id": format!("tg-dialog-reconcile-{suffix}"),
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
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("{provider_chat_id}:700"),
            "chat_kind": "private",
            "chat_title": "Dialog Reconcile Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Irina Volkova",
            "text": "Targeted mark-read commands should reconcile from updateChatReadInbox state.",
            "import_batch_id": format!("telegram-dialog-reconcile-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

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

    let target_message_id = format!("{provider_chat_id}:777");
    let command_id = new_command_id();
    insert_command(
        &pool,
        &command_id,
        &account_id,
        "mark_read",
        &format!("mark_read:{telegram_chat_id}:manual"),
        &provider_chat_id,
        Some(&target_message_id),
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "last_read_inbox_provider_message_id": target_message_id,
        }),
        json!({
            "telegram_chat_id": telegram_chat_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": target_message_id,
        }),
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "last_read_inbox_provider_message_id": target_message_id,
        }),
    )
    .await
    .expect("mark_read command row");

    let reconciled = reconcile_mark_read_commands_from_provider_state(
        &pool,
        &account_id,
        &provider_chat_id,
        &format!("{provider_chat_id}:778"),
        Utc::now(),
        "tdlib.updateChatReadInbox",
    )
    .await
    .expect("mark read reconciliation");
    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, command_id);
    assert_eq!(reconciled[0].command_kind, "mark_read");
    assert_eq!(
        reconciled[0].provider_message_id.as_deref(),
        Some(target_message_id.as_str())
    );
    assert_eq!(reconciled[0].status, "completed");
    assert_eq!(reconciled[0].reconciliation_status, "observed");
}

#[tokio::test]
async fn dialog_pin_reconciliation_marks_mismatched_unpin_command_failed() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-pin-reconcile-{suffix}");
    let provider_chat_id = format!("dialog-pin-chat-{suffix}");
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
            "display_name": "Telegram Dialog Pin Reconcile",
            "external_account_id": format!("tg-dialog-pin-reconcile-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let command_id = new_command_id();
    insert_command(
        &pool,
        &command_id,
        &account_id,
        "unpin",
        &format!("unpin:{provider_chat_id}:manual"),
        &provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "is_pinned": false,
        }),
        json!({
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_dialog_read_reconciliation",
        }),
    )
    .await
    .expect("unpin command row");

    let reconciled = reconcile_pin_commands_from_provider_state(
        &pool,
        &account_id,
        &provider_chat_id,
        true,
        Utc::now(),
        "tdlib.updateChatPosition",
    )
    .await
    .expect("dialog pin reconciliation");
    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, command_id);
    assert_eq!(reconciled[0].command_kind, "unpin");
    assert_eq!(reconciled[0].status, "failed");
    assert_eq!(reconciled[0].reconciliation_status, "mismatch");
    assert_eq!(
        reconciled[0].last_error.as_deref(),
        Some("Provider observed a different dialog pin state than requested")
    );
    assert_eq!(
        reconciled[0].provider_state["expected_is_pinned"],
        json!(false)
    );
    assert_eq!(
        reconciled[0].provider_state["observed_is_pinned"],
        json!(true)
    );
}

#[tokio::test]
async fn dialog_archive_reconciliation_marks_mismatched_unarchive_command_failed() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-archive-reconcile-{suffix}");
    let provider_chat_id = format!("dialog-archive-chat-{suffix}");
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
            "display_name": "Telegram Dialog Archive Reconcile",
            "external_account_id": format!("tg-dialog-archive-reconcile-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    insert_command(
        &pool,
        &new_command_id(),
        &account_id,
        "unarchive",
        &format!("unarchive:{provider_chat_id}:manual"),
        &provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "is_archived": false,
        }),
        json!({
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_dialog_read_reconciliation",
        }),
    )
    .await
    .expect("unarchive command row");

    let reconciled = makosh_hub_backend::integrations::telegram::client::reconcile_archive_commands_from_provider_state(
        &pool,
        &account_id,
        &provider_chat_id,
        true,
        Utc::now(),
        "tdlib.updateChatPosition",
    )
    .await
    .expect("dialog archive reconciliation");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_kind, "unarchive");
    assert_eq!(reconciled[0].status, "failed");
    assert_eq!(reconciled[0].reconciliation_status, "mismatch");
    assert_eq!(
        reconciled[0].last_error.as_deref(),
        Some("Provider observed a different archive state than requested")
    );
    assert_eq!(
        reconciled[0].provider_state["expected_is_archived"],
        json!(false)
    );
    assert_eq!(
        reconciled[0].provider_state["observed_is_archived"],
        json!(true)
    );
}

#[tokio::test]
async fn dialog_mute_reconciliation_marks_mismatched_unmute_command_failed() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-mute-reconcile-{suffix}");
    let provider_chat_id = format!("dialog-mute-chat-{suffix}");
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
            "display_name": "Telegram Dialog Mute Reconcile",
            "external_account_id": format!("tg-dialog-mute-reconcile-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    insert_command(
        &pool,
        &new_command_id(),
        &account_id,
        "unmute",
        &format!("unmute:{provider_chat_id}:manual"),
        &provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "use_default_mute_for": true,
            "mute_for": 0,
        }),
        json!({
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_dialog_read_reconciliat
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_folder_actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_folder_actions.rs`
- Size bytes / Размер в байтах: `15412`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
mod telegram_support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{LOCAL_API_TOKEN, json_body, json_post_request_with_actor, unique_suffix};
use testkit::context::TestContext;

#[tokio::test]
async fn telegram_folder_add_action_records_provider_write_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-folder-action-{suffix}");
    let provider_chat_id = format!("folder-chat-{suffix}");
    let provider_folder_id = 7_i64;
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Folder Action",
                "external_account_id": format!("tg-folder-{suffix}"),
                "api_id": 1,
                "api_hash": "test-api-hash",
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "qr_authorized": true,
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("{provider_chat_id}:1"),
                "chat_kind": "private",
                "chat_title": "Folder Action Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Folder Owner",
                "text": "Folder action should create a durable provider-write command.",
                "import_batch_id": format!("telegram-folder-action-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);

    let chats_response = app
        .clone()
        .oneshot(telegram_support::get_request_with_token(
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
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}"),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("folder add response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["action"], json!("folder_add"));
    assert_eq!(body["status"], json!("queued"));

    let command_id = body["command_id"].as_str().expect("command id");
    let row = sqlx::query(
        r#"
        SELECT command_kind, provider_chat_id, payload, action_class, status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("folder command row");

    let command_kind: String = row.get("command_kind");
    let stored_provider_chat_id: String = row.get("provider_chat_id");
    let action_class: String = row.get("action_class");
    let status: String = row.get("status");
    let payload: serde_json::Value = row.get("payload");

    assert_eq!(command_kind, "folder_add");
    assert_eq!(stored_provider_chat_id, provider_chat_id);
    assert_eq!(action_class, "provider_write");
    assert_eq!(status, "queued");
    assert_eq!(payload["provider_folder_id"], json!(provider_folder_id));
}

#[tokio::test]
async fn telegram_folder_remove_action_records_provider_write_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-folder-remove-action-{suffix}");
    let provider_chat_id = format!("folder-remove-chat-{suffix}");
    let provider_folder_id = 11_i64;
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Folder Remove Action",
                "external_account_id": format!("tg-folder-remove-{suffix}"),
                "api_id": 1,
                "api_hash": "test-api-hash",
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "qr_authorized": true,
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("{provider_chat_id}:1"),
                "chat_kind": "private",
                "chat_title": "Folder Remove Action Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Folder Owner",
                "text": "Folder remove should create a durable provider-write command.",
                "import_batch_id": format!("telegram-folder-remove-action-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);

    let chats_response = app
        .clone()
        .oneshot(telegram_support::get_request_with_token(
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
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}/remove"
            ),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("folder remove response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["action"], json!("folder_remove"));
    assert_eq!(body["status"], json!("queued"));

    let command_id = body["command_id"].as_str().expect("command id");
    let row = sqlx::query(
        r#"
        SELECT command_kind, provider_chat_id, payload, action_class, status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("folder remove command row");

    let command_kind: String = row.get("command_kind");
    let stored_provider_chat_id: String = row.get("provider_chat_id");
    let action_class: String = row.get("action_class");
    let status: String = row.get("status");
    let payload: serde_json::Value = row.get("payload");

    assert_eq!(command_kind, "folder_remove");
    assert_eq!(stored_provider_chat_id, provider_chat_id);
    assert_eq!(action_class, "provider_write");
    assert_eq!(status, "queued");
    assert_eq!(payload["provider_folder_id"], json!(provider_folder_id));
}

#[tokio::test]
async fn telegram_folder_reassign_action_queues_add_and_remove_commands_from_current_membership() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-folder-reassign-action-{suffix}");
    let provider_chat_id = format!("folder-reassign-chat-{suffix}");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Folder Reassign Action",
                "external_account_id": format!("tg-folder-reassign-{suffix}"),
                "api_id": 1,
                "api_hash": "test-api-hash",
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "qr_authorized": true,
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("{provider_chat_id}:1"),
                "chat_kind": "private",
                "chat_title": "Folder Reassign Action Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Folder Owner",
                "text": "Folder reassignment should queue add/remove commands from current membership.",
                "import_batch_id": format!("telegram-folder-reassign-action-{suffix}"),
                "occurred_at": "2026-
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/telegram_manual_send_capability_gates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_manual_send_capability_gates.rs`
- Size bytes / Размер в байтах: `5429`
- Included characters / Включено символов: `5429`
- Truncated / Обрезано: `no`

```rust
mod telegram_support;

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, json_post_request_with_actor,
    unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn removed_account_blocks_manual_send_before_message_audit_and_events() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-send-gates-{suffix}");
    let provider_chat_id = format!("send-gates-chat-{suffix}");
    let command_id = format!("send-{suffix}");
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
            "display_name": "Telegram Send Gates",
            "external_account_id": format!("tg-send-gates-{suffix}"),
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
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Manual Send Gate Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Can Макошь still send after account removal?",
            "import_batch_id": format!("telegram-send-gates-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let before_count = message_count(app.clone(), &account_id, &provider_chat_id).await;

    let remove_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/integrations/telegram/accounts/{account_id}"
                ))
                .header("x-makosh-secret", LOCAL_API_TOKEN)
                .body(axum::body::Body::empty())
                .expect("delete request"),
        )
        .await
        .expect("remove account response");
    assert_eq!(remove_response.status(), StatusCode::OK);

    let send_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/provider-commands/messages/send",
            json!({
                "command_id": command_id,
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "text": "This send must be blocked by capability gate."
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("send response");
    assert_eq!(send_response.status(), StatusCode::BAD_REQUEST);

    let after_count = message_count(app.clone(), &account_id, &provider_chat_id).await;
    assert_eq!(after_count, before_count);

    let send_audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE operation = 'telegram.message.send' AND metadata->>'account_id' = $1 AND metadata->>'provider_chat_id' = $2",
    )
    .bind(&account_id)
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("send audit count");
    assert_eq!(send_audit_count, 0);

    let created_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.message.created' AND payload->>'provider_chat_id' = $1",
    )
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("created event count");
    assert_eq!(created_event_count, 1);

    let command_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.command.status_changed' AND payload->>'provider_chat_id' = $1",
    )
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("command event count");
    assert_eq!(command_event_count, 0);
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

### `backend/tests/telegram_media_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_media_projection.rs`
- Size bytes / Размер в байтах: `7869`
- Included characters / Включено символов: `7869`
- Truncated / Обрезано: `no`

```rust
mod telegram_support;

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
    NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::consume_accepted_signal_event;
use makosh_hub_backend::domains::signal_hub::dispatch_telegram_raw_signal;
use makosh_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, json_body, json_post_request_with_actor, unique_suffix,
};
use testkit::context::TestContext;
#[tokio::test]
async fn telegram_tdlib_projection_accepts_media_message_without_text() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("telegram-empty-media-{suffix}");
    let provider_chat_id = format!("-100{suffix}");
    let provider_message_id = format!("{provider_chat_id}:87403003904");

    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::TelegramUser,
                "Telegram Empty Media",
                format!("tg-empty-media-{suffix}"),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw:telegram-empty-media:{suffix}"),
                &account_id,
                "telegram_message",
                &provider_message_id,
                format!("sha256:{suffix}"),
                format!("telegram-tdlib-history:{account_id}:{provider_chat_id}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": "Media Channel",
                    "chat_kind": "channel",
                    "sender_id": format!("chat:{provider_chat_id}"),
                    "sender_display_name": "Media Channel",
                    "text": "",
                    "delivery_state": "received",
                    "tdlib_raw": {
                        "@type": "message",
                        "id": 87403003904_i64,
                        "chat_id": provider_chat_id,
                        "content": {"@type": "messagePhoto"}
                    }
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "runtime": "tdlib",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw source");

    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &raw)
        .await
        .expect("dispatch empty media telegram signal")
        .expect("accepted empty media telegram signal");
    let projected = consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted empty media signal")
        .expect("projected empty media message");

    assert_eq!(projected.provider_record_id, provider_message_id);
    assert_eq!(projected.conversation_id, Some(provider_chat_id));
    assert_eq!(projected.body_text, "");
    assert_eq!(
        projected.message_metadata["tdlib_raw"]["content"]["@type"],
        json!("messagePhoto")
    );
}

#[tokio::test]
async fn telegram_fixture_media_download_fails_closed_without_live_runtime() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-media-{suffix}");
    let chat_id = format!("media-chat-{suffix}");
    let provider_message_id = format!("media-message-{suffix}");
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
            "display_name": "Telegram Media",
            "external_account_id": format!("tg-media-{suffix}"),
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
            "provider_message_id": provider_message_id,
            "chat_kind": "private",
            "chat_title": "Media Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Document metadata only.",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let download_response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/provider-media/download",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "tdlib_file_id": 42,
                "provider_attachment_id": "tdlib-file:42",
                "filename": "document.pdf",
                "content_type": "application/pdf"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("download response");

    assert_eq!(download_response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(download_response).await;
    assert_eq!(body["error"], json!("invalid_telegram_request"));
    assert!(
        body["message"]
            .as_str()
            .expect("message")
            .contains("Telegram media downloads require an enabled TDLib actor")
    );

    let media_events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE event_type IN (
            'telegram.media.download.started',
            'telegram.media.download.failed'
        )
          AND payload->>'provider_message_id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(&provider_message_id)
    .fetch_all(&pool)
    .await
    .expect("media download lifecycle events");
    assert_eq!(media_events.len(), 2);
    assert_eq!(media_events[0].0, "telegram.media.download.started");
    assert_eq!(media_events[0].1["download_state"], json!("requested"));
    assert_eq!(media_events[0].1["tdlib_file_id"], json!(42));
    assert_eq!(media_events[1].0, "telegram.media.download.failed");
    assert_eq!(media_events[1].1["download_state"], json!("failed"));
    assert!(
        media_events[1].1["error"]
            .as_str()
            .expect("failed event error")
            .contains("Telegram media downloads require an enabled TDLib actor")
    );
}
```

### `backend/tests/telegram_media_upload.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_media_upload.rs`
- Size bytes / Размер в байтах: `11827`
- Included characters / Включено символов: `11827`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-media-upload-test-secret";

#[tokio::test]
async fn telegram_media_upload_imports_attachment_and_queues_provider_command() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-media-{suffix}");
    create_tdlib_account(&pool, &account_id, &suffix).await;
    let app = router(database, &database_url);

    let import_response = app
        .clone()
        .oneshot(post(
            "/api/v1/communications/attachments/import",
            json!({
                "account_id": account_id,
                "channel_kind": "telegram_user",
                "filename": "upload-note.txt",
                "content_type": "text/plain",
                "content_base64": "SGVybWVzIG1lZGlhIHVwbG9hZCBmaXh0dXJl",
                "metadata": {"source": "telegram_media_upload_test"}
            }),
        ))
        .await
        .expect("import response");
    assert_eq!(import_response.status(), StatusCode::OK);
    let imported = json_body(import_response).await;
    assert_eq!(imported["scan_status"], "not_scanned");
    let attachment_id = imported["attachment_id"]
        .as_str()
        .expect("attachment id")
        .to_owned();
    let blob_id = imported["blob_id"].as_str().expect("blob id").to_owned();
    let attachment_observation = sqlx::query(
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
          AND link.entity_kind = 'attachment_import'
          AND link.entity_id = $1
        "#,
    )
    .bind(&attachment_id)
    .fetch_one(&pool)
    .await
    .expect("attachment import observation");
    assert_eq!(
        attachment_observation.get::<String, _>("kind_code"),
        "COMMUNICATION_ATTACHMENT"
    );
    assert_eq!(
        attachment_observation.get::<String, _>("origin_kind"),
        "manual"
    );
    assert_eq!(
        attachment_observation.get::<String, _>("relationship_kind"),
        "attachment_import"
    );
    let attachment_payload = attachment_observation.get::<Value, _>("payload");
    assert_eq!(attachment_payload["attachment_id"], attachment_id);
    assert_eq!(attachment_payload["channel_kind"], "telegram_user");
    assert_eq!(attachment_payload["content_type"], "text/plain");
    assert_eq!(attachment_payload["filename"], "upload-note.txt");

    let command_id = format!("tcmd_media_upload_{suffix}");
    let upload_response = app
        .clone()
        .oneshot(post(
            "/api/v1/integrations/telegram/provider-media/upload",
            json!({
                "command_id": command_id.clone(),
                "account_id": account_id.clone(),
                "provider_chat_id": "123456789",
                "attachment_id": attachment_id.clone(),
                "media_type": "document",
                "caption": "caption must not be written to audit metadata"
            }),
        ))
        .await
        .expect("upload response");
    assert_eq!(upload_response.status(), StatusCode::OK);
    let uploaded = json_body(upload_response).await;
    assert_eq!(uploaded["status"], "queued");
    assert_eq!(uploaded["reconciliation_status"], "not_observed");
    assert_eq!(uploaded["blob_id"], blob_id);

    let duplicate_upload_response = app
        .clone()
        .oneshot(post(
            "/api/v1/integrations/telegram/provider-media/upload",
            json!({
                "command_id": format!("tcmd_media_upload_duplicate_{suffix}"),
                "account_id": account_id.clone(),
                "provider_chat_id": "123456789",
                "attachment_id": attachment_id.clone(),
                "media_type": "document",
                "caption": "caption must not be written to audit metadata"
            }),
        ))
        .await
        .expect("duplicate upload response");
    assert_eq!(duplicate_upload_response.status(), StatusCode::OK);
    let duplicate_uploaded = json_body(duplicate_upload_response).await;
    assert_eq!(duplicate_uploaded["command_id"], command_id);

    let command = sqlx::query(
        r#"
        SELECT command_kind, status, reconciliation_status, payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("queued command");
    assert_eq!(command.get::<String, _>("command_kind"), "send_media");
    assert_eq!(command.get::<String, _>("status"), "queued");
    assert_eq!(
        command.get::<String, _>("reconciliation_status"),
        "not_observed"
    );
    let payload = command.get::<Value, _>("payload");
    assert_eq!(payload["media_type"], "document");
    assert_eq!(payload["attachment_id"], attachment_id);
    assert_eq!(payload["blob_id"], blob_id);

    let command_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM telegram_provider_write_commands WHERE account_id = $1 AND command_kind = 'send_media'",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("command count");
    assert_eq!(command_count, 1);

    let audit = sqlx::query(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.media.upload'
          AND target_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("media upload audit");
    let audit_metadata = audit.get::<Value, _>("metadata");
    assert_eq!(audit_metadata["capability"], "telegram.media.upload");
    assert_eq!(audit_metadata.get("caption"), None);

    let started_events = event_count(&pool, "telegram.media.upload.started", &command_id).await;
    let status_events = event_count(&pool, "telegram.command.status_changed", &command_id).await;
    assert_eq!(started_events, 1);
    assert_eq!(status_events, 1);
    let started_status = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT payload->>'status'
        FROM event_log
        WHERE event_type = 'telegram.media.upload.started'
          AND subject->>'id' = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("started event status");
    assert_eq!(started_status.as_deref(), Some("queued"));
    let started_payload = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'telegram.media.upload.started'
          AND subject->>'id' = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("started event payload");
    assert_eq!(started_payload["command_kind"], "send_media");
    assert_eq!(started_payload["payload"]["attachment_id"], attachment_id);
    assert_eq!(started_payload["payload"]["filename"], "upload-note.txt");
    assert_eq!(started_payload["capability_state"], "available");
}

#[tokio::test]
async fn telegram_media_upload_rejects_malicious_import_before_outbox_insert() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-media-malicious-{suffix}");
    create_tdlib_account(&pool, &account_id, &suffix).await;
    let app = router(database, &database_url);

    let import_response = app
        .clone()
        .oneshot(post(
            "/api/v1/communications/attachments/import",
            json!({
                "account_id": account_id,
                "channel_kind": "telegram_user",
                "filename": "payload.exe",
                "content_type": "application/octet-stream",
                "content_base64": "TVqQAAAA"
            }),
        ))
        .await
        .expect("import response");
    assert_eq!(import_response.status(), StatusCode::OK);
    let imported = json_body(import_response).await;
    assert_eq!(imported["scan_status"], "malicious");

    let command_id = format!("tcmd_media_upload_reject_{suffix}");
    let upload_response = app
        .oneshot(post(
            "/api/v1/integrations/telegram/provider-media/upload",
            json!({
                "command_id": command_id.clone(),
                "account_id": account_id.clone(),
                "provider_chat_id": "123456789",
                "attachment_id": imported["attachment_id"].clone(),
                "media_type": "document"
            }),
        ))
        .await
        .expect("upload response");
    assert_eq!(upload_response.status(), StatusCode::BAD_REQUEST);

    let command_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM telegram_provider_write_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("command count");
    assert_eq!(command_count, 0);
}

async fn create_tdlib_account(pool: &sqlx::PgPool, account_id: &str, suffix: &str) {
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::TelegramUser,
                format!("Telegram Media {suffix}"),
                format!("tg-media-{suffix}"),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");
}

fn router(database: Database, database_url: &str) -> axum::Router {
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url),
        database,
    )
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

async fn event_count(pool: &sqlx::PgPool, event_type: &str, subject_id: &str) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM event_log
        WHERE event_type = $1
          AND subject->>'id' = $2
        "#,
    )
    .bind(event_type)
    .bind(subject_id)
    .fetch_one(pool)
    .await
    .expect("event count")
}

fn unique_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    format!("{now}")
}
```

### `backend/tests/telegram_members_admin_roster.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_members_admin_roster.rs`
- Size bytes / Размер в байтах: `5252`
- Included characters / Включено символов: `5252`
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

const LOCAL_API_TOKEN: &str = "telegram-members-admin-test-secret";

#[tokio::test]
async fn members_route_returns_admin_only_provider_roster_rows() {
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

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": "acct-1",
            "provider_kind": "telegram_user",
            "display_name": "Telegram Member Admin Roster",
            "external_account_id": "telegram:12345",
            "tdlib_data_path": "docker/data/telegram/test-members-admin-roster",
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
            "chat_title": "Admin Roster Room",
            "sender_id": "sender-1",
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": "seed-batch-1",
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(get(
            "/api/v1/communications/conversations?account_id=acct-1&limit=10",
        ))
        .await
        .expect("chat list response");
    let body = json_body(response).await;
    let telegram_chat_id = body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned();

    query(
        r#"
        INSERT INTO telegram_chat_participants (
            participant_id, telegram_chat_id, account_id, provider_chat_id, provider_member_id,
            display_name, username, role, status, is_admin, is_owner, permissions, raw_payload,
            source, observed_at, created_at, updated_at
        )
        VALUES
            ('participant-1', $1, 'acct-1', 'provider-chat-1', 'user:1', 'Recent Member', NULL, 'member', 'member', false, false, '{"observed_via":"tdlib.getSupergroupMembers"}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW()),
            ('participant-2', $1, 'acct-1', 'provider-chat-1', 'user:2', 'Admin Only', 'admin_only', 'admin', 'administrator', true, false, '{"observed_via":"tdlib.getSupergroupMembers.administrators","can_invite_users":true}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW())
        "#,
    )
    .bind(&telegram_chat_id)
    .execute(&pool)
    .await
    .expect("insert participants");

    let members = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?limit=10"
        )))
        .await
        .expect("members response");
    let body = json_body(members).await;
    let items = body["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["provider_member_id"], "user:2");
    assert_eq!(items[0]["role"], "admin");
    assert_eq!(items[0]["status"], "administrator");
    assert_eq!(items[0]["is_admin"], true);
    assert_eq!(
        items[0]["permissions"]["observed_via"],
        "tdlib.getSupergroupMembers.administrators"
    );
    assert_eq!(items[0]["permissions"]["can_invite_users"], true);
    assert_eq!(items[1]["provider_member_id"], "user:1");
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("X-Макошь-Secret", LOCAL_API_TOKEN)
        .body(Body::empty())
        .expect("request")
}

async fn post_ok<S>(app: S, uri: &str, body: Value)
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
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json body")
}
```

### `backend/tests/telegram_members_inactive_filter.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_members_inactive_filter.rs`
- Size bytes / Размер в байтах: `5127`
- Included characters / Включено символов: `5127`
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

const LOCAL_API_TOKEN: &str = "telegram-members-inactive-test-secret";

#[tokio::test]
async fn members_route_excludes_inactive_provider_roster_rows() {
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

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": "acct-1",
            "provider_kind": "telegram_user",
            "display_name": "Telegram Member Lifecycle",
            "external_account_id": "telegram:12345",
            "tdlib_data_path": "docker/data/telegram/test-members-inactive",
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
            "chat_title": "Roster Room",
            "sender_id": "sender-1",
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": "seed-batch-1",
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(get(
            "/api/v1/communications/conversations?account_id=acct-1&limit=10",
        ))
        .await
        .expect("chat list response");
    let body = json_body(response).await;
    let telegram_chat_id = body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned();

    query(
        r#"
        INSERT INTO telegram_chat_participants (
            participant_id, telegram_chat_id, account_id, provider_chat_id, provider_member_id,
            display_name, username, role, status, is_admin, is_owner, permissions, raw_payload,
            source, observed_at, created_at, updated_at
        )
        VALUES
            ('participant-1', $1, 'acct-1', 'provider-chat-1', 'user:1', 'Active User', NULL, 'member', 'member', false, false, '{}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW()),
            ('participant-2', $1, 'acct-1', 'provider-chat-1', 'user:2', 'Left User', NULL, 'left', 'left', false, false, '{}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW()),
            ('participant-3', $1, 'acct-1', 'provider-chat-1', 'user:3', 'Banned User', NULL, 'banned', 'banned', false, false, '{}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW()),
            ('participant-4', $1, 'acct-1', 'provider-chat-1', 'user:4', 'Absent User', NULL, 'member', 'absent_exhaustive', false, false, '{}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW())
        "#,
    )
    .bind(&telegram_chat_id)
    .execute(&pool)
    .await
    .expect("insert participants");

    let members = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?limit=10"
        )))
        .await
        .expect("members response");
    let body = json_body(members).await;
    let items = body["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["provider_member_id"], "user:1");
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("X-Макошь-Secret", LOCAL_API_TOKEN)
        .body(Body::empty())
        .expect("request")
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
    json_body(response).await
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json body")
}
```

### `backend/tests/telegram_members_sync_exhaustive_absence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/telegram_members_sync_exhaustive_absence.rs`
- Size bytes / Размер в байтах: `5499`
- Included characters / Включено символов: `5499`
- Truncated / Обрезано: `no`

```rust
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::query;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::mark_absent_members_from_exhaustive_roster;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-members-absence-test-secret";

#[tokio::test]
async fn members_route_hides_absent_exhaustive_participants_after_roster_reconciliation() {
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

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": "acct-1",
            "provider_kind": "telegram_user",
            "display_name": "Telegram Member Absence",
            "external_account_id": "telegram:12345",
            "tdlib_data_path": "docker/data/telegram/test-members-absence",
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
            "chat_title": "Roster Room",
            "sender_id": "sender-1",
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": "seed-batch-1",
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(get(
            "/api/v1/communications/conversations?account_id=acct-1&limit=10",
        ))
        .await
        .expect("chat list response");
    let body = json_body(response).await;
    let telegram_chat_id = body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned();

    query(
        r#"
        INSERT INTO telegram_chat_participants (
            participant_id, telegram_chat_id, account_id, provider_chat_id, provider_member_id,
            display_name, username, role, status, is_admin, is_owner, permissions, raw_payload,
            source, observed_at, created_at, updated_at
        )
        VALUES
            ('participant-1', $1, 'acct-1', 'provider-chat-1', 'user:1', 'User One', NULL, 'member', 'member', false, false, '{}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW()),
            ('participant-2', $1, 'acct-1', 'provider-chat-1', 'user:2', 'User Two', NULL, 'member', 'member', false, false, '{}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW())
        "#,
    )
    .bind(&telegram_chat_id)
    .execute(&pool)
    .await
    .expect("insert participants");

    let members_before = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?limit=10"
        )))
        .await
        .expect("members before response");
    let before_body = json_body(members_before).await;
    assert_eq!(before_body["items"].as_array().expect("items").len(), 2);

    mark_absent_members_from_exhaustive_roster(
        &pool,
        &telegram_chat_id,
        &[String::from("user:1")],
        "tdlib.getSupergroupMembers.exhaustive_absence",
    )
    .await
    .expect("mark absent members");

    let members_after = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?limit=10"
        )))
        .await
        .expect("members after response");
    let after_body = json_body(members_after).await;
    let items = after_body["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["provider_member_id"], "user:1");
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("X-Макошь-Secret", LOCAL_API_TOKEN)
        .body(Body::empty())
        .expect("request")
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
    json_body(response).await
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json body")
}
```
