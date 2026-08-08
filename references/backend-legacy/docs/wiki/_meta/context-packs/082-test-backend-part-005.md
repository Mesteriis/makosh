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

- Chunk ID / ID чанка: `082-test-backend-part-005`
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

### `backend/tests/email_account_setup/gmail_service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_account_setup/gmail_service.rs`
- Size bytes / Размер в байтах: `7361`
- Included characters / Включено символов: `7361`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use std::sync::Arc;

use makosh_hub_backend::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore, EmailProviderKind,
    ProviderAccountSecretPurpose,
};
use makosh_hub_backend::integrations::mail::accounts::{
    EmailAccountSetupService, GmailOAuthSetupRequest,
};
use makosh_hub_backend::platform::secrets::{
    DatabaseEncryptedSecretVault, NewSecretReference, ResolvedSecret, SecretKind, SecretResolver,
    SecretStoreKind,
};

use super::support::{MockTokenServer, live_setup_context, secret_reference};

#[tokio::test]
async fn gmail_oauth_setup_builds_pkce_url_and_persists_token_bundle_against_postgres() {
    let Some((database, communication_store, secret_store, suffix)) =
        live_setup_context("gmail oauth account setup").await
    else {
        return;
    };
    let token_server = MockTokenServer::start();
    let vault = DatabaseEncryptedSecretVault::new(
        database.pool().expect("configured pool").clone(),
        ResolvedSecret::new("gmail oauth vault key").expect("vault key"),
    );

    let service = EmailAccountSetupService::new(
        database.pool().expect("configured pool").clone(),
        secret_store.clone(),
        vault.clone(),
        Arc::new(CommunicationProviderAccountStore::new(
            database.pool().expect("configured pool").clone(),
        )),
        Arc::new(CommunicationProviderSecretBindingStore::new(
            database.pool().expect("configured pool").clone(),
        )),
    );
    let pending = service
        .start_gmail_oauth(
            GmailOAuthSetupRequest::new(
                format!("acct_gmail_setup_{suffix}"),
                "Gmail setup",
                format!("gmail-setup-{suffix}@example.com"),
                "desktop-client-id",
                "http://127.0.0.1:18088/oauth/callback",
            )
            .authorization_endpoint(format!("{}/authorize", token_server.base_url()))
            .token_endpoint(format!("{}/token", token_server.base_url())),
        )
        .expect("start gmail oauth setup");

    assert!(pending.authorization_url.contains("code_challenge="));
    assert!(
        pending
            .authorization_url
            .contains("code_challenge_method=S256")
    );
    assert!(pending.authorization_url.contains("access_type=offline"));
    assert!(pending.authorization_url.contains("prompt=consent"));
    assert!(pending.authorization_url.contains("gmail.readonly"));
    assert!(pending.authorization_url.contains("gmail.send"));
    assert!(!pending.authorization_url.contains(&pending.code_verifier));

    let completed = service
        .complete_gmail_oauth(pending.clone(), "authorization-code")
        .await
        .expect("complete gmail oauth setup");

    assert_eq!(completed.account_id, pending.account_id);
    assert_eq!(completed.secret_kind, SecretKind::OauthToken);
    assert_eq!(
        completed.store_kind,
        SecretStoreKind::DatabaseEncryptedVault
    );

    let account = communication_store
        .provider_account(&pending.account_id)
        .await
        .expect("load provider account")
        .expect("provider account exists");
    assert_eq!(account.provider_kind, EmailProviderKind::Gmail);
    assert_eq!(account.config["auth"], "oauth");
    assert_eq!(account.config["api"], "gmail");
    assert_eq!(account.config["oauth_client_id"], "desktop-client-id");
    assert!(account.config.get("access_token").is_none());
    assert!(account.config.get("refresh_token").is_none());

    let binding = communication_store
        .provider_account_secret_binding(
            &pending.account_id,
            ProviderAccountSecretPurpose::OauthToken,
        )
        .await
        .expect("load binding")
        .expect("binding exists");
    assert_eq!(binding.secret_ref, completed.secret_ref);

    let reference = secret_store
        .secret_reference(&completed.secret_ref)
        .await
        .expect("load secret reference")
        .expect("secret reference exists");
    assert_eq!(
        reference.store_kind,
        SecretStoreKind::DatabaseEncryptedVault
    );
    assert_eq!(reference.secret_kind, SecretKind::OauthToken);

    let token_bundle = vault
        .resolve(&reference)
        .await
        .expect("resolve token bundle");
    let token_bundle: Value =
        serde_json::from_str(token_bundle.expose_for_runtime()).expect("token bundle json");
    assert_eq!(token_bundle["access_token"], "gmail-access-token");
    assert_eq!(token_bundle["refresh_token"], "gmail-refresh-token");
    assert_eq!(token_bundle["client_id"], "desktop-client-id");

    let requests = token_server.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].body.contains("grant_type=authorization_code"));
    assert!(requests[0].body.contains("code=authorization-code"));
    assert!(requests[0].body.contains("code_verifier="));

    drop(database);
}

#[tokio::test]
async fn gmail_oauth_refresh_returns_runtime_access_token_and_updates_vault() {
    let Some((database, _communication_store, secret_store, suffix)) =
        live_setup_context("gmail oauth refresh").await
    else {
        return;
    };
    let token_server = MockTokenServer::start();
    let vault = DatabaseEncryptedSecretVault::new(
        database.pool().expect("configured pool").clone(),
        ResolvedSecret::new("refresh vault key").expect("vault key"),
    );
    let secret_ref = format!("secret:gmail:oauth:refresh-test:{suffix}");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::OauthToken,
            SecretStoreKind::DatabaseEncryptedVault,
            "Gmail refresh credential",
        ))
        .await
        .expect("store refresh secret reference");
    vault
        .store_secret(
            &secret_ref,
            &json!({
                "token_url": format!("{}/token", token_server.base_url()),
                "client_id": "desktop-client-id",
                "access_token": "expired-access-token",
                "refresh_token": "gmail-refresh-token",
                "expires_at": "2000-01-01T00:00:00Z"
            })
            .to_string(),
        )
        .await
        .expect("store expired token bundle");

    let service = EmailAccountSetupService::new_for_vault_only(vault.clone());
    let access_token = service
        .refresh_gmail_access_token(&secret_ref)
        .await
        .expect("refresh gmail access token");

    assert_eq!(
        access_token.expose_for_runtime(),
        "gmail-refreshed-access-token"
    );

    let refreshed = vault
        .resolve(&secret_reference(&secret_ref))
        .await
        .expect("resolve refreshed token bundle");
    let refreshed: Value =
        serde_json::from_str(refreshed.expose_for_runtime()).expect("refreshed token bundle json");
    assert_eq!(refreshed["access_token"], "gmail-refreshed-access-token");
    assert_eq!(refreshed["refresh_token"], "gmail-refresh-token");

    let requests = token_server.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].body.contains("grant_type=refresh_token"));
    assert!(
        requests[0]
            .body
            .contains("refresh_token=gmail-refresh-token")
    );

    drop(database);
}
```

### `backend/tests/email_account_setup/imap_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_account_setup/imap_api.rs`
- Size bytes / Размер в байтах: `17260`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::Arc;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::calendar::events::CalendarAccountStore;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderAccountStore,
    CommunicationProviderSecretBindingStore, EmailProviderKind, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::integrations::mail::accounts::{
    EmailAccountSetupService, ImapAccountSetupRequest,
};
use makosh_hub_backend::platform::secrets::{
    DatabaseEncryptedSecretVault, ResolvedSecret, SecretKind, SecretReferenceStore, SecretResolver,
    SecretStoreKind,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::vault::{HostVault, HostVaultConfig};
use testkit::context::TestContext;

use super::support::{
    LOCAL_API_TOKEN, json_body, json_request_with_token_and_actor, live_setup_context,
    unlock_test_vault,
};

#[tokio::test]
async fn imap_account_setup_stores_encrypted_secret_in_database_against_postgres() {
    let Some((database, communication_store, secret_store, suffix)) =
        live_setup_context("imap account setup").await
    else {
        return;
    };
    let vault = DatabaseEncryptedSecretVault::new(
        database.pool().expect("configured pool").clone(),
        ResolvedSecret::new("imap vault key").expect("vault key"),
    );
    let service = EmailAccountSetupService::new(
        database.pool().expect("configured pool").clone(),
        secret_store.clone(),
        vault.clone(),
        Arc::new(CommunicationProviderAccountStore::new(
            database.pool().expect("configured pool").clone(),
        )),
        Arc::new(CommunicationProviderSecretBindingStore::new(
            database.pool().expect("configured pool").clone(),
        )),
    );

    let account_id = format!("acct_imap_setup_{suffix}");
    let completed = service
        .setup_imap_account(
            ImapAccountSetupRequest::new(
                &account_id,
                EmailProviderKind::Icloud,
                "iCloud setup",
                format!("icloud-setup-{suffix}@icloud.com"),
                "imap.mail.me.com",
                993,
                true,
                "INBOX",
                format!("icloud-setup-{suffix}@icloud.com"),
                "icloud-app-password",
            )
            .secret_kind(SecretKind::AppPassword),
        )
        .await
        .expect("setup imap account");

    let account = communication_store
        .provider_account(&account_id)
        .await
        .expect("load provider account")
        .expect("provider account exists");
    assert_eq!(account.provider_kind, EmailProviderKind::Icloud);
    assert_eq!(account.config["host"], "imap.mail.me.com");
    assert_eq!(account.config["port"], 993);
    assert_eq!(account.config["tls"], true);
    assert_eq!(account.config["mailbox"], "INBOX");
    assert_eq!(
        account.config["username"],
        format!("icloud-setup-{suffix}@icloud.com")
    );
    assert!(account.config.get("password").is_none());
    assert!(account.config.get("app_password").is_none());

    let reference = secret_store
        .secret_reference(&completed.secret_ref)
        .await
        .expect("load secret reference")
        .expect("secret reference exists");
    assert_eq!(
        reference.store_kind,
        SecretStoreKind::DatabaseEncryptedVault
    );
    assert_eq!(reference.secret_kind, SecretKind::AppPassword);
    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve imap password")
            .expose_for_runtime(),
        "icloud-app-password"
    );

    drop(database);
}

#[tokio::test]
async fn icloud_account_setup_api_creates_calendar_account_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config"),
        database.clone(),
    );
    unlock_test_vault(app.clone()).await;

    let account_id = "icloud-primary";
    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "icloud",
                "display_name": "Primary iCloud",
                "external_account_id": "user@icloud.com",
                "host": "imap.mail.me.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "user@icloud.com",
                "password": "icloud-app-password",
                "secret_kind": "app_password"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account_id"], account_id);

    let pool = database.pool().expect("configured pool").clone();
    let account = CommunicationIngestionStore::new(pool.clone())
        .provider_account(account_id)
        .await
        .expect("load provider account")
        .expect("provider account");
    assert_eq!(account.provider_kind, EmailProviderKind::Icloud);
    assert_eq!(
        account.config["connected_services"],
        json!(["mail", "calendar", "contacts"])
    );
    let provider_account_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'vault'
           AND entity_kind = 'communication_provider_account'
           AND entity_id = $1
           AND relationship_kind = 'upsert'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .expect("provider account observation link");
    let provider_account_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&provider_account_observation_id)
    .fetch_one(&pool)
    .await
    .expect("provider account observation");
    assert_eq!(
        provider_account_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "local_runtime"
    );
    assert_eq!(
        provider_account_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_ACCOUNT"
    );
    assert_eq!(account.config["smtp_host"], "smtp.mail.me.com");
    assert_eq!(account.config["smtp_port"], 587);
    assert_eq!(account.config["smtp_tls"], true);
    assert_eq!(account.config["smtp_starttls"], true);
    assert_eq!(account.config["smtp_username"], "user@icloud.com");
    assert!(account.config.get("password").is_none());
    assert!(account.config.get("smtp_password").is_none());
    let signal_connection = sqlx::query(
        r#"
        SELECT source_code, status, settings, secret_ref
        FROM signal_connections
        WHERE source_code = 'mail'
          AND settings->>'account_id' = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .expect("mail signal connection");
    let signal_settings: serde_json::Value = signal_connection
        .try_get("settings")
        .expect("signal settings");
    assert_eq!(
        signal_connection
            .try_get::<String, _>("source_code")
            .expect("signal source"),
        "mail"
    );
    assert_eq!(
        signal_connection
            .try_get::<String, _>("status")
            .expect("signal status"),
        "connected"
    );
    assert_eq!(signal_settings["account_id"], json!(account_id));
    assert_eq!(signal_settings["provider_kind"], json!("icloud"));
    assert_eq!(
        signal_connection
            .try_get::<String, _>("secret_ref")
            .expect("signal secret ref"),
        "secret:provider-account:icloud-primary:imap_password"
    );

    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool.clone());
    let imap_binding = communication_store
        .provider_account_secret_binding(account_id, ProviderAccountSecretPurpose::ImapPassword)
        .await
        .expect("load imap binding")
        .expect("imap binding");
    let smtp_binding = communication_store
        .provider_account_secret_binding(account_id, ProviderAccountSecretPurpose::SmtpPassword)
        .await
        .expect("load smtp binding")
        .expect("smtp binding");
    assert_eq!(
        imap_binding.secret_ref,
        "secret:provider-account:icloud-primary:imap_password"
    );
    assert_eq!(
        smtp_binding.secret_ref,
        "secret:provider-account:icloud-primary:smtp_password"
    );
    let binding_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'vault'
           AND entity_kind = 'communication_provider_secret_binding'
           AND entity_id = $1
           AND relationship_kind = 'bind'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(format!(
        "{}:{}",
        account_id,
        ProviderAccountSecretPurpose::ImapPassword.as_str()
    ))
    .fetch_one(&pool)
    .await
    .expect("provider secret binding observation link");
    let binding_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&binding_observation_id)
    .fetch_one(&pool)
    .await
    .expect("provider secret binding observation");
    assert_eq!(
        binding_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "local_runtime"
    );
    assert_eq!(
        binding_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_SECRET_BINDING"
    );

    let smtp_reference = secret_store
        .secret_reference(&smtp_binding.secret_ref)
        .await
        .expect("load smtp secret reference")
        .expect("smtp secret reference");
    assert_eq!(smtp_reference.store_kind, SecretStoreKind::HostVault);
    assert_eq!(smtp_reference.secret_kind, SecretKind::AppPassword);
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    assert_eq!(
        vault
            .resolve(&smtp_reference)
            .await
            .expect("resolve smtp password")

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/email_account_setup/send_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_account_setup/send_api.rs`
- Size bytes / Размер в байтах: `15035`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

use super::support::{
    LOCAL_API_TOKEN, MockSmtpServer, json_body, json_request_with_token_and_actor,
    unlock_test_vault,
};

#[tokio::test]
async fn imap_send_api_queues_outbox_without_direct_smtp_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let smtp_server = MockSmtpServer::start();
    let account_id = "imap-send-primary";
    let setup_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "imap",
                "display_name": "IMAP Send",
                "external_account_id": "sender@example.com",
                "host": "imap.example.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "sender@example.com",
                "password": "smtp-app-password",
                "secret_kind": "password",
                "smtp_host": smtp_server.addr().ip().to_string(),
                "smtp_port": smtp_server.addr().port(),
                "smtp_tls": false,
                "smtp_starttls": false,
                "smtp_username": "sender@example.com"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("setup response");
    assert_eq!(setup_response.status(), StatusCode::OK);

    let send_response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "cc": ["copy@example.com"],
                "subject": "SMTP send test",
                "body_text": "Message body from Макошь test.",
                "confirmed_provider_write": true
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("send response");
    assert_eq!(send_response.status(), StatusCode::OK);
    let send_body = json_body(send_response).await;
    assert_eq!(send_body["transport"], "outbox");
    assert_eq!(send_body["status"], "queued");
    assert_eq!(
        send_body["accepted_recipients"],
        json!(["recipient@example.com", "copy@example.com"])
    );
    let outbox_id = send_body["outbox_id"].as_str().expect("outbox id");
    assert_eq!(send_body["message_id"], json!(outbox_id));
    let pool = ctx.pool().clone();
    let outbox = sqlx::query(
        "SELECT status, to_participants, cc_participants, bcc_participants, subject
         FROM communication_outbox
         WHERE outbox_id = $1",
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("outbox item");
    let to_participants: serde_json::Value =
        outbox.try_get("to_participants").expect("to participants");
    let cc_participants: serde_json::Value =
        outbox.try_get("cc_participants").expect("cc participants");
    let bcc_participants: serde_json::Value = outbox
        .try_get("bcc_participants")
        .expect("bcc participants");
    let outbox_status: String = outbox.try_get("status").expect("outbox status");
    let outbox_subject: String = outbox.try_get("subject").expect("outbox subject");
    assert_eq!(outbox_status, "queued");
    assert_eq!(outbox_subject, "SMTP send test");
    assert_eq!(to_participants, json!(["recipient@example.com"]));
    assert_eq!(cc_participants, json!(["copy@example.com"]));
    assert_eq!(bcc_participants, json!([]));

    let link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("outbox observation link");
    let link_metadata: serde_json::Value = link.try_get("metadata").expect("link metadata");
    assert_eq!(link_metadata["operation"], "outbox_enqueue");
    assert_eq!(link_metadata["status"], "queued");

    let commands = smtp_server.commands();
    assert!(commands.iter().all(|line| line != "AUTH LOGIN"));
    assert!(commands.iter().all(|line| !line.starts_with("MAIL FROM")));
    assert!(commands.iter().all(|line| !line.starts_with("RCPT TO")));
    assert!(commands.iter().all(|line| line != "DATA"));
}

#[tokio::test]
async fn gmail_send_api_queues_outbox_without_direct_gmail_client_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let account_id = "gmail-send-disabled";
    CommunicationIngestionStore::new(pool)
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                EmailProviderKind::Gmail,
                "Gmail Send Disabled",
                "sender@gmail.com",
            )
            .config(json!({
                "auth": "oauth",
                "api": "gmail"
            })),
        )
        .await
        .expect("store gmail account");

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "subject": "Gmail send disabled",
                "body_text": "Gmail provider-side send is not enabled.",
                "confirmed_provider_write": true
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("send response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["transport"], "outbox");
    assert_eq!(body["status"], "queued");
    assert!(body["outbox_id"].as_str().is_some());
}

#[tokio::test]
async fn imap_send_api_queues_without_smtp_password_binding_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let smtp_server = MockSmtpServer::start();
    let account_id = "imap-send-missing-smtp-password";
    let setup_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "imap",
                "display_name": "IMAP Missing SMTP Password",
                "external_account_id": "sender@example.com",
                "host": "imap.example.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "sender@example.com",
                "password": "smtp-app-password",
                "secret_kind": "password",
                "smtp_host": smtp_server.addr().ip().to_string(),
                "smtp_port": smtp_server.addr().port(),
                "smtp_tls": false,
                "smtp_starttls": false,
                "smtp_username": "sender@example.com"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("setup response");
    assert_eq!(setup_response.status(), StatusCode::OK);

    sqlx::query(
        "DELETE FROM communication_provider_account_secret_refs WHERE account_id = $1 AND secret_purpose = 'smtp_password'",
    )
    .bind(account_id)
    .execute(&pool)
    .await
    .expect("delete smtp binding");

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "subject": "Missing SMTP binding",
                "body_text": "This must not reach SMTP.",
                "confirmed_provider_write": true
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("send response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["transport"], "outbox");
    assert_eq!(body["status"], "queued");
    assert!(body["outbox_id"].as_str().is_some());
    assert!(
        smtp_server
            .commands()
            .iter()
            .all(|line| !line.starts_with("MAIL FROM"))
    );
}

#[tokio::test]
async fn imap_send_api_does_not_send_when_audit_record_fails_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("va
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/email_account_setup/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_account_setup/support.rs`
- Size bytes / Размер в байтах: `15334`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
#![allow(dead_code)]

use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tokio::time::{Duration, sleep};
use tower::ServiceExt;

use makosh_hub_backend::domains::calendar::events::CalendarAccountStore;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::platform::secrets::{SecretKind, SecretReferenceStore, SecretStoreKind};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::vault::HostVault;

pub const LOCAL_API_TOKEN: &str = "account-setup-test-token";

#[derive(Clone, Debug)]
pub struct TokenRequest {
    pub body: String,
}

pub struct MockTokenServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<TokenRequest>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockTokenServer {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind token server");
        let addr = listener.local_addr().expect("token server addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for _ in 0..2 {
                let Ok((mut stream, _)) = listener.accept() else {
                    break;
                };
                let request = read_http_request(&mut stream);
                if request.body.is_empty() {
                    break;
                }
                let body = if request.body.contains("grant_type=refresh_token") {
                    json!({
                        "access_token": "gmail-refreshed-access-token",
                        "expires_in": 3600,
                        "token_type": "Bearer"
                    })
                    .to_string()
                } else {
                    json!({
                        "access_token": "gmail-access-token",
                        "refresh_token": "gmail-refresh-token",
                        "expires_in": 3600,
                        "token_type": "Bearer",
                        "scope": "https://www.googleapis.com/auth/gmail.readonly"
                    })
                    .to_string()
                };
                requests_for_thread
                    .lock()
                    .expect("requests lock")
                    .push(request);
                write_http_response(&mut stream, &body);
            }
        });

        Self {
            addr,
            requests,
            handle: Some(handle),
        }
    }

    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn requests(&self) -> Vec<TokenRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

impl Drop for MockTokenServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            handle.join().expect("token server join");
        }
    }
}

pub struct MockSmtpServer {
    addr: SocketAddr,
    commands: Arc<Mutex<Vec<String>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockSmtpServer {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind SMTP server");
        let addr = listener.local_addr().expect("SMTP server addr");
        let commands = Arc::new(Mutex::new(Vec::new()));
        let commands_for_thread = Arc::clone(&commands);
        let handle = thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .expect("set SMTP read timeout");
            write!(stream, "220 mock.smtp.local ESMTP\r\n").expect("write greeting");

            let mut reader = BufReader::new(stream.try_clone().expect("clone SMTP stream"));
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => break,
                    Err(error) => panic!("read SMTP line: {error}"),
                }
                let command = line.trim_end().to_owned();
                commands_for_thread
                    .lock()
                    .expect("SMTP commands lock")
                    .push(command.clone());
                if command.starts_with("EHLO") {
                    write!(stream, "250-mock.smtp.local\r\n250 AUTH LOGIN\r\n")
                        .expect("write EHLO response");
                } else if command == "AUTH LOGIN" {
                    write!(stream, "334 VXNlcm5hbWU6\r\n").expect("write username prompt");
                } else if command == "c2VuZGVyQGV4YW1wbGUuY29t" {
                    write!(stream, "334 UGFzc3dvcmQ6\r\n").expect("write password prompt");
                } else if command == "c210cC1hcHAtcGFzc3dvcmQ=" {
                    write!(stream, "235 Authentication successful\r\n").expect("write auth ok");
                } else if command.starts_with("MAIL FROM") || command.starts_with("RCPT TO") {
                    write!(stream, "250 OK\r\n").expect("write envelope ok");
                } else if command == "DATA" {
                    write!(stream, "354 End data with <CR><LF>.<CR><LF>\r\n")
                        .expect("write DATA response");
                    loop {
                        let mut data_line = String::new();
                        if reader
                            .read_line(&mut data_line)
                            .expect("read SMTP data line")
                            == 0
                        {
                            return;
                        }
                        let data_line = data_line.trim_end().to_owned();
                        commands_for_thread
                            .lock()
                            .expect("SMTP commands lock")
                            .push(data_line.clone());
                        if data_line == "." {
                            break;
                        }
                    }
                    write!(stream, "250 mock-message-id queued\r\n").expect("write queued");
                } else if command == "QUIT" {
                    write!(stream, "221 Bye\r\n").expect("write bye");
                    break;
                } else {
                    write!(stream, "250 OK\r\n").expect("write default ok");
                }
            }
        });

        Self {
            addr,
            commands,
            handle: Some(handle),
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn commands(&self) -> Vec<String> {
        self.commands.lock().expect("SMTP commands lock").clone()
    }
}

impl Drop for MockSmtpServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            handle.join().expect("SMTP server join");
        }
    }
}

fn read_http_request(stream: &mut TcpStream) -> TokenRequest {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("set read timeout");
    let mut reader = BufReader::new(stream);
    let mut content_length = 0usize;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read request line");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse().expect("content length");
        }
    }

    let mut body = vec![0_u8; content_length];
    use std::io::Read;
    reader.read_exact(&mut body).expect("read request body");

    TokenRequest {
        body: String::from_utf8(body).expect("utf8 body"),
    }
}

fn write_http_response(stream: &mut TcpStream, body: &str) {
    let result = write!(
        stream,
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    if let Err(error) = result {
        assert_eq!(
            error.kind(),
            ErrorKind::BrokenPipe,
            "write response: {error}"
        );
    }
}

pub async fn live_setup_context(
    _test_name: &str,
) -> Option<(
    Database,
    CommunicationIngestionStore,
    SecretReferenceStore,
    u128,
)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool);

    Some((database, communication_store, secret_store, unique_suffix()))
}

pub fn secret_reference(
    secret_ref: &str,
) -> makosh_hub_backend::platform::secrets::SecretReference {
    let now = chrono::Utc::now();

    makosh_hub_backend::platform::secrets::SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind: SecretKind::OauthToken,
        store_kind: SecretStoreKind::DatabaseEncryptedVault,
        label: "Gmail OAuth".to_owned(),
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    }
}

pub fn json_request_with_token_and_actor(
    uri: &str,
    body: Value,
    token: &str,
    _actor_id: &str,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

pub async fn unlock_test_vault<S>(app: S)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let entropy_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);

    let create_response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/vault/create",
            json!({}),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);
}

pub async fn wait_for_provider_account(
    communication_store: &CommunicationIngestionStore,
    account_id: &str,
) -> makosh_hub_backend::domains::communications::core::ProviderAccount {
    for _ in 0..50 {
        if let Some(account) = communication_store
            .provider_account(account_id)
            .await
            .expect("load provider account")
        {
            return account;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("provider account {account_id} was not reconciled");
}

pub async fn wait_for_secret_reference(

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/email_account_setup/vault_reconciliation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_account_setup/vault_reconciliation.rs`
- Size bytes / Размер в байтах: `11109`
- Included characters / Включено символов: `11109`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::calendar::events::CalendarAccountStore;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::platform::secrets::{SecretKind, SecretReferenceStore, SecretResolver};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::vault::{HostVault, HostVaultConfig, SecretEntryContext};
use testkit::context::TestContext;

use super::support::{
    LOCAL_API_TOKEN, json_request_with_token_and_actor, unlock_test_vault,
    wait_for_calendar_account, wait_for_manifest_metadata_key, wait_for_provider_account,
    wait_for_provider_account_secret_binding, wait_for_secret_reference,
};

#[tokio::test]
async fn startup_reconciles_icloud_account_from_host_vault_manifest_after_postgres_metadata_wipe() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let app = build_router_with_database(config.clone(), database.clone());
    unlock_test_vault(app.clone()).await;

    let account_id = "icloud-recover";
    let secret_ref = "secret:provider-account:icloud-recover:imap_password";
    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "icloud",
                "display_name": "Recovered iCloud",
                "external_account_id": "recover@icloud.com",
                "host": "imap.mail.me.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "recover@icloud.com",
                "password": "icloud-app-password",
                "secret_kind": "app_password"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let pool = database.pool().expect("configured pool").clone();
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home.clone(),
        dev_mode: true,
        dev_key_path: dev_key_path.clone(),
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    vault
        .upsert_account_secret_manifest_entry(
            secret_ref,
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id,
                purpose: ProviderAccountSecretPurpose::ImapPassword.as_str(),
                secret_kind: SecretKind::AppPassword.as_str(),
                label: "IMAP password",
                metadata: &json!({
                    "provider": "icloud",
                    "account_id": account_id
                }),
            },
        )
        .expect("write sparse manifest entry");

    let _enriching_app = build_router_with_database(config.clone(), database.clone());
    wait_for_manifest_metadata_key(&vault, secret_ref, "display_name").await;
    wait_for_provider_account_secret_binding(
        &CommunicationIngestionStore::new(pool.clone()),
        account_id,
        ProviderAccountSecretPurpose::ImapPassword,
    )
    .await;

    sqlx::query("DELETE FROM calendar_accounts WHERE account_id = $1")
        .bind(format!("icloud-calendar:{account_id}"))
        .execute(&pool)
        .await
        .expect("delete calendar metadata");
    sqlx::query("DELETE FROM communication_provider_account_secret_refs WHERE account_id = $1")
        .bind(account_id)
        .execute(&pool)
        .await
        .expect("delete secret binding");
    sqlx::query("DELETE FROM communication_provider_accounts WHERE account_id = $1")
        .bind(account_id)
        .execute(&pool)
        .await
        .expect("delete provider account");
    sqlx::query("DELETE FROM secret_references WHERE secret_ref = $1")
        .bind(secret_ref)
        .execute(&pool)
        .await
        .expect("delete secret reference");

    assert!(
        CommunicationIngestionStore::new(pool.clone())
            .provider_account(account_id)
            .await
            .expect("load deleted account")
            .is_none()
    );

    let restarted_database = Database::connect(Some(&database_url))
        .await
        .expect("restarted database connection");
    let _restarted_app = build_router_with_database(config, restarted_database.clone());
    let restarted_pool = restarted_database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(restarted_pool.clone());
    let secret_store = SecretReferenceStore::new(restarted_pool.clone());

    let account = wait_for_provider_account(&communication_store, account_id).await;
    assert_eq!(account.provider_kind, EmailProviderKind::Icloud);
    assert_eq!(account.display_name, "Recovered iCloud");
    assert_eq!(account.external_account_id, "recover@icloud.com");
    assert_eq!(
        account.config["connected_services"],
        json!(["mail", "calendar", "contacts"])
    );
    let provider_account_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'vault'
           AND entity_kind = 'communication_provider_account'
           AND entity_id = $1
           AND relationship_kind = 'upsert'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(account_id)
    .fetch_one(&restarted_pool)
    .await
    .expect("provider account observation link");
    let provider_account_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&provider_account_observation_id)
    .fetch_one(&restarted_pool)
    .await
    .expect("provider account observation");
    assert_eq!(
        provider_account_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "vault_source"
    );
    assert_eq!(
        provider_account_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_ACCOUNT"
    );

    let reference = wait_for_secret_reference(&secret_store, secret_ref).await;
    assert_eq!(reference.store_kind.as_str(), "host_vault");
    assert_eq!(reference.secret_kind, SecretKind::AppPassword);

    let binding = wait_for_provider_account_secret_binding(
        &communication_store,
        account_id,
        ProviderAccountSecretPurpose::ImapPassword,
    )
    .await;
    assert_eq!(binding.secret_ref, secret_ref);
    let binding_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'vault'
           AND entity_kind = 'communication_provider_secret_binding'
           AND entity_id = $1
           AND relationship_kind = 'bind'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(format!(
        "{}:{}",
        account_id,
        ProviderAccountSecretPurpose::ImapPassword.as_str()
    ))
    .fetch_one(&restarted_pool)
    .await
    .expect("provider secret binding observation link");
    let binding_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&binding_observation_id)
    .fetch_one(&restarted_pool)
    .await
    .expect("provider secret binding observation");
    assert_eq!(
        binding_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "vault_source"
    );
    assert_eq!(
        binding_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_SECRET_BINDING"
    );

    let calendar_store = CalendarAccountStore::new(restarted_pool.clone());
    let calendar_account =
        wait_for_calendar_account(&calendar_store, &format!("icloud-calendar:{account_id}")).await;
    assert_eq!(calendar_account.provider, "apple");
    assert_eq!(
        calendar_account.email.as_deref(),
        Some("recover@icloud.com")
    );
    assert_eq!(
        calendar_account.credentials_reference.as_deref(),
        Some(secret_ref)
    );
    let calendar_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'calendar'
           AND entity_kind = 'calendar_account'
           AND entity_id = $1
           AND relationship_kind = 'linked_provider_upsert'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(format!("icloud-calendar:{account_id}"))
    .fetch_one(&restarted_pool)
    .await
    .expect("calendar account observation link");
    let calendar_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&calendar_observation_id)
    .fetch_one(&restarted_pool)
    .await
    .expect("calendar account observation");
    assert_eq!(
        calendar_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "vault_source"
    );
    assert_eq!(
        calendar_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "CALENDAR_ACCOUNT_LINK"
    );

    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve restored secret")
            .expose_for_runtime(),
        "icloud-app-password"
    );
}
```

### `backend/tests/email_account_setup_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_account_setup_architecture.rs`
- Size bytes / Размер в байтах: `1782`
- Included characters / Включено символов: `1782`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn email_account_setup_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_email_account_setup_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "email account setup test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_email_account_setup_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_email_account_setup_violations(&path, violations);
            continue;
        }
        if !is_email_account_setup_test_file(&path) {
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

fn is_email_account_setup_test_file(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value.starts_with("email_account_setup"))
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/email_fixture_export.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_fixture_export.rs`
- Size bytes / Размер в байтах: `3907`
- Included characters / Включено символов: `3907`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{TimeZone, Utc};
use serde_json::json;

use makosh_hub_backend::domains::communications::core::EmailProviderKind;
use makosh_hub_backend::domains::communications::fixtures::export::{
    EmailFixtureExportOptions, export_fixture_messages_from_sync_batch,
};
use makosh_hub_backend::integrations::mail::sync::{
    EmailSyncBatch, FetchedCommunicationSourceMessage,
};

#[test]
fn imap_raw_message_exports_redacted_fixture_without_personal_content() {
    let raw = concat!(
        "From: Alice Example <alice@company.test>\r\n",
        "To: Bob Example <bob@company.test>\r\n",
        "Subject: Confidential roadmap\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Real customer and roadmap details."
    );
    let batch = sync_batch_with_raw_message(raw);

    let fixtures =
        export_fixture_messages_from_sync_batch(&batch, EmailFixtureExportOptions::default())
            .expect("export fixture");

    assert_eq!(fixtures.len(), 1);
    let fixture = &fixtures[0];
    assert_eq!(fixture.provider_record_id, "43");
    assert_eq!(
        fixture.sent_at,
        Utc.with_ymd_and_hms(2026, 6, 4, 12, 0, 0).single()
    );
    assert!(fixture.subject.starts_with("Redacted subject "));
    assert!(fixture.from.ends_with("@example.invalid"));
    assert_eq!(fixture.to.len(), 1);
    assert!(fixture.to[0].ends_with("@example.invalid"));
    assert!(fixture.body_text.contains("Redacted body fixture"));

    let fixture_json = serde_json::to_string(&fixtures).expect("fixture JSON");
    assert!(!fixture_json.contains("alice@company.test"));
    assert!(!fixture_json.contains("bob@company.test"));
    assert!(!fixture_json.contains("Confidential roadmap"));
    assert!(!fixture_json.contains("Real customer"));
}

#[test]
fn imap_multipart_quoted_printable_message_exports_redacted_fixture() {
    let raw = concat!(
        "From: Sender <sender@example.test>\r\n",
        "To: Team <team@example.test>\r\n",
        "Subject: =?UTF-8?Q?Q2_update?=\r\n",
        "Content-Type: multipart/alternative; boundary=\"boundary-1\"\r\n",
        "\r\n",
        "--boundary-1\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n",
        "\r\n",
        "Hello=2C team=21\r\n",
        "--boundary-1\r\n",
        "Content-Type: text/html; charset=utf-8\r\n",
        "\r\n",
        "<p>Hello, team!</p>\r\n",
        "--boundary-1--\r\n"
    );
    let batch = sync_batch_with_raw_message(raw);

    let fixtures =
        export_fixture_messages_from_sync_batch(&batch, EmailFixtureExportOptions::default())
            .expect("export fixture");

    assert_eq!(fixtures.len(), 1);
    assert!(fixtures[0].subject.starts_with("Redacted subject "));
    assert!(fixtures[0].body_text.contains("original_chars=12"));
}

fn sync_batch_with_raw_message(raw: &str) -> EmailSyncBatch {
    EmailSyncBatch {
        provider_kind: EmailProviderKind::Icloud,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({
            "provider": "imap",
            "mailbox": "INBOX",
            "uid_validity": 999,
            "last_seen_uid": 43
        })),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: "43".to_owned(),
            source_fingerprint: "sha256:test-message".to_owned(),
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 4, 12, 0, 0).single(),
            payload: json!({
                "provider": "icloud",
                "transport": "imap",
                "mailbox": "INBOX",
                "uid": 43,
                "uid_validity": 999,
                "raw_rfc822_base64": BASE64_STANDARD.encode(raw.as_bytes()),
                "rfc822_size": raw.len()
            }),
        }],
    }
}
```

### `backend/tests/email_fixture_pipeline.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_fixture_pipeline.rs`
- Size bytes / Размер в байтах: `4762`
- Included characters / Включено символов: `4762`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use serde_json::json;
use sqlx::Row;

use makosh_hub_backend::domains::communications::core::EmailProviderKind;
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::email_fixture_pipeline::{
    EmailFixturePipelineRequest, project_fixture_email_messages,
};

#[tokio::test]
async fn fixture_email_pipeline_imports_projects_persons_and_graph_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("acct_fixture_pipeline_{suffix}");
    let fixture_json = json!([
        {
            "provider_record_id": format!("fixture-pipeline-{suffix}"),
            "subject": "Pipeline import",
            "from": "sender@example.invalid",
            "to": ["recipient@example.invalid"],
            "sent_at": "2026-06-04T10:00:00Z",
            "body_text": "Pipeline body",
            "source_fingerprint": format!("sha256:pipeline-{suffix}")
        }
    ])
    .to_string();

    let report = project_fixture_email_messages(
        pool,
        &EmailFixturePipelineRequest::new(
            &account_id,
            "iCloud fixture pipeline",
            "redacted@example.invalid",
            EmailProviderKind::Icloud,
            format!("batch_pipeline_{suffix}"),
            fixture_json,
        ),
    )
    .await
    .expect("project fixture pipeline");

    assert_eq!(report.imported_records, 1);
    assert_eq!(report.projected_messages, 1);
    assert_eq!(report.upserted_persons, 2);
    assert!(!report.graph_summary.is_empty);
    assert!(report.total_graph_nodes >= 4);
    assert!(report.total_graph_edges >= 3);

    let accepted_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'signal.accepted.mail.message'
          AND source ->> 'account_id' = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(test_context.pool())
    .await
    .expect("accepted mail signal count");
    assert_eq!(accepted_signal_count, 1);

    let observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM communication_messages
        WHERE account_id = $1
        ORDER BY projected_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(test_context.pool())
    .await
    .expect("message observation id");
    let trace_rows = sqlx::query(
        r#"
        SELECT event_id, event_type, causation_id, correlation_id
        FROM event_log
        WHERE correlation_id = $1
          AND event_type IN (
              'observation.captured.v1',
              'signal.raw.mail.message.observed',
              'signal.accepted.mail.message',
              'communication.message.recorded'
          )
        ORDER BY position ASC
        "#,
    )
    .bind(&observation_id)
    .fetch_all(test_context.pool())
    .await
    .expect("mail trace rows");
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
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
```

### `backend/tests/email_import.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_import.rs`
- Size bytes / Размер в байтах: `12657`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::{TimeZone, Utc};
use serde_json::json;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use makosh_hub_backend::domains::communications::import::{
    FixtureEmailImportRequest, import_fixture_email_messages,
    import_fixture_email_messages_with_records,
};
use makosh_hub_backend::domains::communications::sources::{
    FixtureCommunicationSourceMessage, parse_fixture_email_messages,
};
use makosh_hub_backend::platform::storage::Database;

#[test]
fn fixture_email_source_parses_account_scoped_messages() {
    let input = json!([
        {
            "provider_record_id": "gmail-msg-1",
            "subject": "Budget review",
            "from": "alice@example.com",
            "to": ["bob@example.com"],
            "sent_at": "2026-06-04T10:00:00Z",
            "body_text": "Please review the Q2 budget.",
            "source_fingerprint": "sha256:gmail-msg-1"
        }
    ])
    .to_string();

    let messages = parse_fixture_email_messages(&input).expect("parse fixture messages");

    assert_eq!(
        messages,
        vec![FixtureCommunicationSourceMessage {
            provider_record_id: "gmail-msg-1".to_owned(),
            subject: "Budget review".to_owned(),
            from: "alice@example.com".to_owned(),
            to: vec!["bob@example.com".to_owned()],
            sent_at: Utc.with_ymd_and_hms(2026, 6, 4, 10, 0, 0).single(),
            body_text: "Please review the Q2 budget.".to_owned(),
            source_fingerprint: "sha256:gmail-msg-1".to_owned(),
        }]
    );
}

#[tokio::test]
async fn fixture_email_import_records_raw_messages_idempotently_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_fixture_import_{suffix}");
    let fixture_json = format!(
        r#"[{{"provider_record_id":"fixture-msg-{suffix}","subject":"Fixture import","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:00:00Z","body_text":"Fixture body","source_fingerprint":"sha256:fixture-msg-{suffix}"}}]"#
    );

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Fixture Gmail",
            format!("fixture-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let first = import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(&account_id, format!("batch_{suffix}"), &fixture_json),
    )
    .await
    .expect("first import");
    let second = import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &account_id,
            format!("batch_retry_{suffix}"),
            &fixture_json,
        ),
    )
    .await
    .expect("second import");

    assert_eq!(first.inserted_or_existing_records, 1);
    assert_eq!(second.inserted_or_existing_records, 1);

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM communication_raw_records WHERE account_id = $1 AND provider_record_id = $2",
    )
    .bind(&account_id)
    .bind(format!("fixture-msg-{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("raw record count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn fixture_email_import_records_delimiter_bearing_identities_distinctly_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();

    let same_account_id = format!("acct_fixture_identity_same_{suffix}");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &same_account_id,
            EmailProviderKind::Gmail,
            "Fixture identity same account",
            format!("fixture-identity-same-{suffix}@example.com"),
        ))
        .await
        .expect("store same-account provider account");

    let same_account_fixture_json = format!(
        r#"[
            {{"provider_record_id":"thread:{suffix}:message","subject":"Delimiter import A","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:00:00Z","body_text":"Fixture body A","source_fingerprint":"sha256:same-a-{suffix}"}},
            {{"provider_record_id":"thread::{suffix}:message","subject":"Delimiter import B","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:01:00Z","body_text":"Fixture body B","source_fingerprint":"sha256:same-b-{suffix}"}}
        ]"#
    );

    let same_account_report = import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &same_account_id,
            format!("batch_same_identity_{suffix}"),
            same_account_fixture_json,
        ),
    )
    .await
    .expect("same-account delimiter import");
    assert_eq!(same_account_report.inserted_or_existing_records, 2);

    let same_account_raw_record_ids = sqlx::query_scalar::<_, String>(
        r#"
        SELECT raw_record_id
        FROM communication_raw_records
        WHERE account_id = $1
        ORDER BY provider_record_id
        "#,
    )
    .bind(&same_account_id)
    .fetch_all(&pool)
    .await
    .expect("same-account raw record IDs");
    assert_eq!(same_account_raw_record_ids.len(), 2);
    assert_ne!(
        same_account_raw_record_ids[0],
        same_account_raw_record_ids[1]
    );

    let ambiguous_account_id = format!("acct_fixture_identity_{suffix}");
    let ambiguous_left_account_id = format!("{ambiguous_account_id}:left");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &ambiguous_account_id,
            EmailProviderKind::Gmail,
            "Fixture identity base account",
            format!("fixture-identity-base-{suffix}@example.com"),
        ))
        .await
        .expect("store base provider account");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &ambiguous_left_account_id,
            EmailProviderKind::Gmail,
            "Fixture identity left account",
            format!("fixture-identity-left-{suffix}@example.com"),
        ))
        .await
        .expect("store left provider account");

    let base_fixture_json = format!(
        r#"[{{"provider_record_id":"left:right","subject":"Ambiguous import base","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:02:00Z","body_text":"Fixture body base","source_fingerprint":"sha256:base-{suffix}"}}]"#
    );
    let left_fixture_json = format!(
        r#"[{{"provider_record_id":"right","subject":"Ambiguous import left","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:03:00Z","body_text":"Fixture body left","source_fingerprint":"sha256:left-{suffix}"}}]"#
    );

    import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &ambiguous_account_id,
            format!("batch_base_identity_{suffix}"),
            base_fixture_json,
        ),
    )
    .await
    .expect("base ambiguous import");
    import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &ambiguous_left_account_id,
            format!("batch_left_identity_{suffix}"),
            left_fixture_json,
        ),
    )
    .await
    .expect("left ambiguous import");

    let ambiguous_raw_record_ids = sqlx::query_scalar::<_, String>(
        r#"
        SELECT raw_record_id
        FROM communication_raw_records
        WHERE account_id IN ($1, $2)
        ORDER BY account_id, provider_record_id
        "#,
    )
    .bind(&ambiguous_account_id)
    .bind(&ambiguous_left_account_id)
    .fetch_all(&pool)
    .await
    .expect("ambiguous raw record IDs");
    assert_eq!(ambiguous_raw_record_ids.len(), 2);
    assert_ne!(ambiguous_raw_record_ids[0], ambiguous_raw_record_ids[1]);
}

#[tokio::test]
async fn fixture_email_import_preserves_missing_sent_at_as_null_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_fixture_missing_sent_at_{suffix}");
    let provider_record_id = format!("fixture-missing-sent-at-{suffix}");
    let fixture_json = format!(
        r#"[{{"provider_record_id":"{provider_record_id}","subject":"Missing sent_at import","from":"alice@example.com","to":["bob@example.com"],"body_text":"Fixture body without sent_at","source_fingerprint":"sha256:missing-sent-at-{suffix}"}}]"#
    );

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Fixture missing sent_at",
            format!("fixture-missing-sent-at-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &account_id,
            format!("batch_missing_sent_at_{suffix}"),
            fixture_json,
        ),
    )
    .await
    .expect("missing sent_at import");

    let occurred_at = sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
        r#"
        SELECT occurred_at
        FROM communication_raw_records
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("raw record occurred_at");
    assert!(occurred_at.is_none());
}

#[tokio::test]
async fn fixture_email_import_returns_raw_records_for_projection_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_fixture_records_{suffix}");
    let provider_record_id = format!("fixture-records-{suffix}");
    let fixture_json = format!(
        r#"[{{"provider_record_id":"{provider_record_id}","subject":"Record import","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:00:00Z","body_text":"Fixture body","source_fingerprint":"sha256:records-{suffix}"}}]"#
    );

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Fixture records",
            format!("fixture-records-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let report = import_fixture_email_messages_with_records(
        &communication_store,
        &FixtureEmailImportReque
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/email_outbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_outbox.rs`
- Size bytes / Размер в байтах: `22043`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::Row;
use testkit::context::TestContext;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::domains::communications::outbox::{
    CommunicationOutboxItem, CommunicationOutboxStatus, CommunicationOutboxStore,
    EmailOutboxDeliveryWorker, NewCommunicationOutboxItem, OutboxDeliveryError, OutboxEmailSender,
    OutboxRetryPolicy, OutboxSendReceipt, SmtpOutboxEmailSender, SmtpTransport,
};
use makosh_hub_backend::integrations::mail::send::{
    EmailSendError, OutgoingEmail, SendResult, SmtpConfig,
};
use makosh_hub_backend::platform::secrets::{
    InMemorySecretResolver, NewSecretReference, ResolvedSecret, SecretKind, SecretReferenceStore,
    SecretStoreKind,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn outbox_claim_due_waits_for_schedule_and_undo_deadline_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-store-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Outbox Store IMAP",
            format!("outbox-store-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let store = CommunicationOutboxStore::new(pool);
    let now = Utc::now();
    let outbox_id = format!("outbox-store-{suffix}");
    store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: outbox_id.clone(),
            account_id,
            draft_id: None,
            to_recipients: vec!["recipient@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Claim after undo".to_owned(),
            body_text: "Do not send before undo window closes.".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Queued,
            scheduled_send_at: Some(now - Duration::minutes(1)),
            undo_deadline_at: Some(now + Duration::seconds(30)),
            metadata: json!({ "source": "test" }),
        })
        .await
        .expect("enqueue outbox item");

    let premature = store.claim_due(now, 10).await.expect("premature claim");
    assert!(premature.is_empty());

    let claimed = store
        .claim_due(now + Duration::seconds(31), 10)
        .await
        .expect("claim after undo window");
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].outbox_id, outbox_id);
    assert_eq!(claimed[0].status, CommunicationOutboxStatus::Sending);
    assert_eq!(claimed[0].send_attempts, 1);
    assert!(claimed[0].claimed_at.is_some());
}

#[tokio::test]
async fn outbox_delivery_worker_marks_sent_and_appends_event_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-delivery-{suffix}");
    seed_provider_account(pool.clone(), &account_id, suffix).await;
    let store = CommunicationOutboxStore::new(pool.clone());
    let now = Utc::now();
    let outbox_id = format!("outbox-delivery-{suffix}");
    enqueue_due_item(&store, &account_id, &outbox_id, now).await;

    let worker = EmailOutboxDeliveryWorker::new(
        store.clone(),
        StaticSuccessSender {
            provider_message_id: "provider-message-1".to_owned(),
            accepted_recipients: vec!["recipient@example.com".to_owned()],
        },
    );
    let report = worker
        .deliver_due(now + Duration::seconds(1), 10)
        .await
        .expect("deliver due outbox");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.sent, 1);
    assert_eq!(report.failed, 0);
    assert_eq!(report.retried, 0);
    let sent_items = store
        .list(Some(&account_id), Some(CommunicationOutboxStatus::Sent), 10)
        .await
        .expect("list sent items");
    assert_eq!(sent_items.len(), 1);
    assert_eq!(sent_items[0].outbox_id, outbox_id);
    assert_eq!(
        sent_items[0].provider_message_id.as_deref(),
        Some("provider-message-1")
    );
    assert!(sent_items[0].sent_at.is_some());
    assert!(sent_items[0].last_error.is_none());

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'mail.outbox.sent' AND subject->>'id' = $1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("sent event count");
    assert_eq!(event_count, 1);
    let sent_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("sent observation link");
    let sent_metadata: serde_json::Value = sent_link.try_get("metadata").expect("sent metadata");
    assert_eq!(sent_metadata["status"], "sent");
    let sent_observation_id: String = sent_link
        .try_get("observation_id")
        .expect("sent observation id");
    let sent_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&sent_observation_id)
    .fetch_one(&pool)
    .await
    .expect("sent observation");
    let sent_origin_kind: String = sent_observation
        .try_get("origin_kind")
        .expect("sent origin kind");
    let sent_payload: serde_json::Value =
        sent_observation.try_get("payload").expect("sent payload");
    assert_eq!(sent_origin_kind, "local_runtime");
    assert_eq!(sent_payload["operation"], "outbox_mark_sent");
}

#[tokio::test]
async fn outbox_delivery_worker_marks_failed_and_appends_event_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-failure-{suffix}");
    seed_provider_account(pool.clone(), &account_id, suffix).await;
    let store = CommunicationOutboxStore::new(pool.clone());
    let now = Utc::now();
    let outbox_id = format!("outbox-failure-{suffix}");
    enqueue_due_item(&store, &account_id, &outbox_id, now).await;

    let worker = EmailOutboxDeliveryWorker::with_retry_policy(
        store.clone(),
        StaticFailureSender {
            message: "SMTP unavailable".to_owned(),
        },
        OutboxRetryPolicy::disabled(),
    );
    let report = worker
        .deliver_due(now + Duration::seconds(1), 10)
        .await
        .expect("deliver due outbox");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.sent, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.retried, 0);
    let failed_items = store
        .list(
            Some(&account_id),
            Some(CommunicationOutboxStatus::Failed),
            10,
        )
        .await
        .expect("list failed items");
    assert_eq!(failed_items.len(), 1);
    assert_eq!(failed_items[0].outbox_id, outbox_id);
    assert_eq!(
        failed_items[0].last_error.as_deref(),
        Some("SMTP unavailable")
    );
    assert!(failed_items[0].provider_message_id.is_none());
    assert!(failed_items[0].sent_at.is_none());

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'mail.outbox.failed' AND subject->>'id' = $1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("failed event count");
    assert_eq!(event_count, 1);
    let failed_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("failed observation link");
    let failed_metadata: serde_json::Value =
        failed_link.try_get("metadata").expect("failed metadata");
    assert_eq!(failed_metadata["status"], "failed");
}

#[tokio::test]
async fn outbox_delivery_worker_schedules_retry_with_backoff_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-retry-{suffix}");
    seed_provider_account(pool.clone(), &account_id, suffix).await;
    let store = CommunicationOutboxStore::new(pool.clone());
    let now = Utc::now();
    let delivery_started_at = now + Duration::seconds(1);
    let outbox_id = format!("outbox-retry-{suffix}");
    enqueue_due_item(&store, &account_id, &outbox_id, now).await;

    let worker = EmailOutboxDeliveryWorker::with_retry_policy(
        store.clone(),
        StaticFailureSender {
            message: "SMTP unavailable".to_owned(),
        },
        OutboxRetryPolicy::new(3, Duration::seconds(60), Duration::minutes(10)),
    );
    let report = worker
        .deliver_due(delivery_started_at, 10)
        .await
        .expect("deliver due outbox");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.sent, 0);
    assert_eq!(report.failed, 0);
    assert_eq!(report.retried, 1);
    let retry_items = store
        .list(
            Some(&account_id),
            Some(CommunicationOutboxStatus::Scheduled),
            10,
        )
        .await
        .expect("list retry items");
    assert_eq!(retry_items.len(), 1);
    assert_eq!(retry_items[0].outbox_id, outbox_id);
    assert_eq!(retry_items[0].send_attempts, 1);
    assert_eq!(
        retry_items[0].scheduled_send_at,
        Some(delivery_started_at + Duration::seconds(60))
    );
    assert_eq!(
        retry_items[0].last_error.as_deref(),
        Some("SMTP unavailable")
    );
    assert!(retry_items[0].provider_message_id.is_none());
    assert!(retry_items[0].sent_at.is_none());

    let premature_claim = store
        .claim_due(delivery_started_at + Duration::seconds(59), 10)
        .await
        .expect("premature retry claim");
    assert!(premature_claim.is_empty());
    let due_retry = store
        .claim_due(delivery_started_at + Duration::seconds(60), 10)
        .await
        .expect("due retry claim");
    assert_eq!(due_retry.len(), 1);
    assert_eq!(due_retry[0].send_attempts, 2);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'mail.outbox.retry_scheduled' AND subject->>'id' = $1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("retry scheduled event count");
    assert_eq!(event_count, 1);
    let retry_links = sqlx::query(
        "SELECT metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at ASC",
    )
    .bind(&outbox_id)
    .fetch_all(&pool)
    .await
    .expect("retry observation links");
    let retry_statuses: Vec<String> = retry_links
        .iter()
        .map(|row| {
            row.try_get::<serde_json::Value, _>("metadata")
                .expect("retry metadata")["status"]
                .as_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/email_provider_network.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_provider_network.rs`
- Size bytes / Размер в байтах: `20644`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::{TimeZone, Utc};
use serde_json::json;
use sqlx::Row;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use makosh_hub_backend::domains::communications::storage::{
    CommunicationStorageStore, LocalCommunicationBlobStore,
};
use makosh_hub_backend::integrations::mail::gmail::client::{
    GmailApiClient, GmailFetchOptions, ImapFetchOptions, ImapNetworkClient,
};
use makosh_hub_backend::integrations::mail::sync::{
    EmailSyncBatch, FetchedCommunicationSourceMessage,
};
use makosh_hub_backend::platform::secrets::ResolvedSecret;
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::email_sync_pipeline::{
    record_email_sync_batch, record_email_sync_batch_with_mail_blobs,
};

#[tokio::test]
async fn gmail_api_client_fetches_raw_messages_with_bearer_token() {
    let server = MockGmailServer::start();
    let token = ResolvedSecret::new("gmail-access-token").expect("token");
    let client = GmailApiClient::new(server.base_url()).user_id("me");

    let batch = client
        .fetch_raw_messages(&token, &GmailFetchOptions::new(2).query("is:unread"))
        .await
        .expect("fetch gmail messages");

    assert_eq!(batch.provider_kind, EmailProviderKind::Gmail);
    assert_eq!(batch.stream_id, "gmail:history");
    assert_eq!(
        batch.checkpoint,
        Some(json!({
            "provider": "gmail",
            "history_id": "12345",
            "next_page_token": "next-page",
            "page_kind": "messages"
        }))
    );
    assert_eq!(batch.messages.len(), 1);

    let message = &batch.messages[0];
    assert_eq!(message.provider_record_id, "gmail-msg-1");
    assert_eq!(
        message.occurred_at,
        Utc.timestamp_millis_opt(1_770_000_000_000).single()
    );
    assert!(message.source_fingerprint.starts_with("sha256:"));
    assert_eq!(message.payload["provider"], "gmail");
    assert_eq!(message.payload["thread_id"], "thread-1");
    assert_eq!(
        message.payload["raw_base64url"],
        "U3ViamVjdDogR21haWwNCg0KSGVsbG8"
    );

    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    assert!(
        requests[0]
            .request_line
            .starts_with("GET /gmail/v1/users/me/messages?")
    );
    assert!(requests[0].request_line.contains("maxResults=2"));
    assert!(requests[0].request_line.contains("includeSpamTrash=true"));
    assert!(requests[0].request_line.contains("q=is%3Aunread"));
    assert_eq!(
        requests[0].authorization.as_deref(),
        Some("Bearer gmail-access-token")
    );
    assert_eq!(
        requests[1].request_line,
        "GET /gmail/v1/users/me/messages/gmail-msg-1?format=raw HTTP/1.1"
    );
    assert_eq!(
        requests[1].authorization.as_deref(),
        Some("Bearer gmail-access-token")
    );
}

#[tokio::test]
async fn imap_network_client_fetches_raw_messages_by_uid_without_mutating_mailbox() {
    let server = MockImapServer::start();
    let password = ResolvedSecret::new("imap-password").expect("password");
    let client = ImapNetworkClient::new();
    let options = ImapFetchOptions::new(
        "127.0.0.1",
        server.addr().port(),
        false,
        "Archive",
        "imap-user@example.net",
    )
    .last_seen_uid(42)
    .max_messages(10);

    let batch = client
        .fetch_raw_messages(&password, &options)
        .await
        .expect("fetch IMAP messages");

    assert_eq!(batch.provider_kind, EmailProviderKind::Imap);
    assert_eq!(batch.stream_id, "imap:Archive");
    assert_eq!(
        batch.checkpoint,
        Some(json!({
            "provider": "imap",
            "mailbox": "Archive",
            "uid_validity": 999,
            "last_seen_uid": 43
        }))
    );
    assert_eq!(batch.messages.len(), 1);
    assert_eq!(batch.messages[0].provider_record_id, "43");
    assert_eq!(batch.messages[0].payload["provider"], "imap");
    assert_eq!(batch.messages[0].payload["mailbox"], "Archive");
    assert_eq!(
        batch.messages[0].payload["raw_rfc822_base64"],
        "U3ViamVjdDogSU1BUA0KDQpIZWxsbw=="
    );

    let commands = server.commands();
    assert!(
        commands
            .iter()
            .any(|command| command.contains("LOGIN") && command.contains("imap-user@example.net"))
    );
    assert!(
        commands
            .iter()
            .any(|command| command.contains("EXAMINE") && command.contains("Archive"))
    );
    assert!(
        commands
            .iter()
            .any(|command| command.contains("UID SEARCH UID 43:*"))
    );
    assert!(commands.iter().any(|command| {
        command.contains("UID FETCH 43 (UID BODY.PEEK[] RFC822.SIZE INTERNALDATE)")
    }));
    for prohibited_command in ["SELECT", "STORE", "EXPUNGE", "COPY", "MOVE", "DELETE"] {
        assert!(
            !commands
                .iter()
                .any(|command| command.to_ascii_uppercase().contains(prohibited_command)),
            "IMAP client must not send mutating command `{prohibited_command}`: {commands:?}"
        );
    }
}

#[tokio::test]
async fn email_sync_records_provider_network_batch_against_postgres() {
    let Some((database, store, suffix)) = live_sync_context("provider network batch").await else {
        return;
    };

    let account_id = format!("acct_network_batch_{suffix}");
    store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Network Gmail",
            format!("network-batch-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let batch = EmailSyncBatch {
        provider_kind: EmailProviderKind::Gmail,
        stream_id: "gmail:history".to_owned(),
        checkpoint: Some(json!({"provider": "gmail", "history_id": "12345"})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: format!("gmail-network-message-{suffix}"),
            source_fingerprint: format!("sha256:gmail-network-message-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_000_000).single(),
            payload: json!({
                "provider": "gmail",
                "id": format!("gmail-network-message-{suffix}"),
                "raw_base64url": "U3ViamVjdA"
            }),
        }],
    };

    let report = record_email_sync_batch(
        &store,
        &account_id,
        &format!("provider-network-batch-{suffix}"),
        &batch,
    )
    .await
    .expect("record provider network batch");

    assert_eq!(report.inserted_or_existing_records, 1);
    assert!(report.checkpoint_saved);

    let pool = database.pool().expect("configured pool");
    let raw_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM communication_raw_records
        WHERE account_id = $1
          AND provider_record_id = $2
          AND payload ->> 'provider' = 'gmail'
        "#,
    )
    .bind(&account_id)
    .bind(&batch.messages[0].provider_record_id)
    .fetch_one(pool)
    .await
    .expect("count raw record");
    assert_eq!(raw_count, 1);

    let checkpoint = store
        .checkpoint(&account_id, &batch.stream_id)
        .await
        .expect("load checkpoint")
        .expect("checkpoint exists");
    assert_eq!(checkpoint.checkpoint["history_id"], "12345");
}

#[tokio::test]
async fn email_sync_records_provider_batches_with_mail_blobs_against_postgres() {
    let Some((database, store, suffix)) = live_sync_context("provider blob batch").await else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let mail_store = CommunicationStorageStore::new(pool.clone());
    let blob_root = tempfile::tempdir().expect("mail blob root");
    let blob_store = LocalCommunicationBlobStore::new(blob_root.path());
    let gmail_account_id = format!("acct_blob_gmail_{suffix}");
    let imap_account_id = format!("acct_blob_imap_{suffix}");

    store
        .upsert_provider_account(&NewProviderAccount::new(
            &gmail_account_id,
            EmailProviderKind::Gmail,
            "Blob Gmail",
            format!("blob-gmail-{suffix}@example.com"),
        ))
        .await
        .expect("store gmail provider account");
    store
        .upsert_provider_account(&NewProviderAccount::new(
            &imap_account_id,
            EmailProviderKind::Imap,
            "Blob IMAP",
            format!("blob-imap-{suffix}@example.net"),
        ))
        .await
        .expect("store imap provider account");

    let gmail_batch = EmailSyncBatch {
        provider_kind: EmailProviderKind::Gmail,
        stream_id: "gmail:history".to_owned(),
        checkpoint: Some(json!({"provider": "gmail", "history_id": "blob-123"})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: format!("gmail-blob-message-{suffix}"),
            source_fingerprint: format!("sha256:gmail-blob-message-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_000_000).single(),
            payload: json!({
                "provider": "gmail",
                "id": format!("gmail-blob-message-{suffix}"),
                "raw_base64url": "U3ViamVjdDogR21haWwNCg0KSGVsbG8"
            }),
        }],
    };
    let imap_batch = EmailSyncBatch {
        provider_kind: EmailProviderKind::Imap,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({"provider": "imap", "last_seen_uid": 77})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: format!("imap-blob-message-{suffix}"),
            source_fingerprint: format!("sha256:imap-blob-message-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_100_000).single(),
            payload: json!({
                "provider": "imap",
                "uid": 77,
                "raw_rfc822_base64": "U3ViamVjdDogSU1BUA0KDQpIZWxsbw=="
            }),
        }],
    };

    let gmail_report = record_email_sync_batch_with_mail_blobs(
        &store,
        &mail_store,
        &blob_store,
        &gmail_account_id,
        &format!("provider-blob-gmail-{suffix}"),
        &gmail_batch,
    )
    .await
    .expect("record gmail provider blob batch");
    let imap_report = record_email_sync_batch_with_mail_blobs(
        &store,
        &mail_store,
        &blob_store,
        &imap_account_id,
        &format!("provider-blob-imap-{suffix}"),
        &imap_batch,
    )
    .await
    .expect("record imap provider blob batch");

    assert_eq!(gmail_report.inserted_or_existing_records, 1);
    assert_eq!(gmail_report.blobs_upserted, 1);
    assert!(gmail_report.checkpoint_saved);
    assert_eq!(imap_report.inserted_or_existing_records, 1);
    assert_eq!(imap_report.blobs_upserted, 1);
    assert!(imap_report.checkpoint_saved);

    let rows = sqlx::query(
        r#"
        SELECT account_id, payload
        FROM communication_raw_records
        WHERE account_id IN ($1, $2)
        ORDER BY account_id
        "#,
    )
    .bind(&gmail_account_id)
    .bind(&imap_account_id)
    .fetch_all(&pool)
    .await
    .expect("raw records");
    assert_eq!(rows.len(), 2);

    for row in rows {
        let payload: serde_json::Value = row.try_get("payload").expect("payload");
        assert!(payload.get("raw_base64url").is_none());
        assert!(payload.get("raw_rfc822_base64").is_none());
        assert!(
            payload["raw_blob_id"]
                .as_str()
                .expect("raw_blob_id")
                .starts_with("blob:v1:sha256:")
        );
        assert_eq!(payload["raw_blob_storage_kind"], "local_fs");
        let storage_path = payload["raw_blob_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/email_rfc822.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_rfc822.rs`
- Size bytes / Размер в байтах: `7142`
- Included characters / Включено символов: `7093`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::integrations::mail::rfc822::{
    ParsedEmailAttachmentDisposition, parse_rfc822_message,
};

#[test]
fn rfc822_parser_extracts_nested_multipart_attachments_for_current_basic_slice() {
    let raw = concat!(
        "Subject: Nested attachments\r\n",
        "From: Sender <sender@example.invalid>\r\n",
        "To: Recipient <recipient@example.invalid>\r\n",
        "Content-Type: multipart/mixed; boundary=\"outer-boundary\"\r\n",
        "\r\n",
        "--outer-boundary\r\n",
        "Content-Type: multipart/alternative; boundary=\"alt-boundary\"\r\n",
        "\r\n",
        "--alt-boundary\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n",
        "\r\n",
        "Nested=20plain=20body.\r\n",
        "--alt-boundary\r\n",
        "Content-Type: text/html; charset=utf-8\r\n",
        "\r\n",
        "<p>Nested HTML body.</p>\r\n",
        "--alt-boundary--\r\n",
        "--outer-boundary\r\n",
        "Content-Type: application/pdf; name*=UTF-8''report%20Q2.pdf\r\n",
        "Content-Disposition: attachment; filename*=UTF-8''report%20Q2.pdf\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "JVBERi0xLjQ=\r\n",
        "--outer-boundary\r\n",
        "Content-Type: text/plain; name=\"notes.txt\"\r\n",
        "Content-Disposition: inline; filename=\"notes.txt\"\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n",
        "\r\n",
        "note=20body=0Asecond=20line\r\n",
        "--outer-boundary--\r\n"
    );

    let parsed = parse_rfc822_message(raw.as_bytes()).expect("parse nested multipart message");

    assert_eq!(parsed.subject, "Nested attachments");
    assert_eq!(parsed.body_text, "Nested plain body.");
    assert_eq!(
        parsed.body_html.as_deref(),
        Some("<p>Nested HTML body.</p>")
    );
    assert_eq!(parsed.attachments.len(), 2);

    let pdf = &parsed.attachments[0];
    assert_eq!(pdf.provider_attachment_id, "part-1");
    assert_eq!(pdf.filename.as_deref(), Some("report Q2.pdf"));
    assert_eq!(pdf.content_type, "application/pdf");
    assert_eq!(
        pdf.disposition,
        ParsedEmailAttachmentDisposition::Attachment
    );
    assert_eq!(pdf.body_bytes, b"%PDF-1.4");

    let notes = &parsed.attachments[1];
    assert_eq!(notes.provider_attachment_id, "part-2");
    assert_eq!(notes.filename.as_deref(), Some("notes.txt"));
    assert_eq!(notes.content_type, "text/plain");
    assert_eq!(notes.disposition, ParsedEmailAttachmentDisposition::Inline);
    assert_eq!(notes.body_bytes, b"note body\nsecond line");
}

#[test]
fn rfc822_parser_preserves_html_links_for_rich_mail_rendering() {
    let raw = concat!(
        "Subject: Rich links\r\n",
        "From: Fever <hello@example.invalid>\r\n",
        "To: User <user@example.invalid>\r\n",
        "Content-Type: text/html; charset=utf-8\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n",
        "\r\n",
        "<p>Footer</p><a href=3D\"https://click.example.invalid/privacy?qs=3Dabc\">Privacy policy</a>",
        "<a href=3D\"https://click.example.invalid/contact?qs=3Dabc\">Contact us</a>",
        "<a href=3D\"https://click.example.invalid/unsub?qs=3Dabc\">Unsubscribe</a>\r\n"
    );

    let parsed = parse_rfc822_message(raw.as_bytes()).expect("parse rich html message");

    assert!(parsed.body_text.contains("Privacy policy"));
    let html = parsed.body_html.as_deref().expect("body html");
    assert!(html.contains("href=\"https://click.example.invalid/privacy?qs=abc\""));
    assert!(html.contains(">Privacy policy</a>"));
    assert!(html.contains(">Contact us</a>"));
    assert!(html.contains(">Unsubscribe</a>"));
}

#[test]
fn rfc822_parser_preserves_source_headers_with_folded_values() {
    let raw = concat!(
        "Subject: Folded headers\r\n",
        "From: Sender <sender@example.invalid>\r\n",
        "To: Recipient <recipient@example.invalid>\r\n",
        "X-Макошь-Trace: first line\r\n",
        "\tcontinued line\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Body.\r\n"
    );

    let parsed = parse_rfc822_message(raw.as_bytes()).expect("parse folded header message");

    assert!(parsed.headers.contains(&(
        "X-Макошь-Trace".to_owned(),
        "first line continued line".to_owned()
    )));
    assert!(parsed.headers.contains(&(
        "Content-Type".to_owned(),
        "text/plain; charset=utf-8".to_owned()
    )));
}

#[test]
fn rfc822_parser_extracts_rfc2231_continued_attachment_filenames() {
    let raw = concat!(
        "Subject: Continued filename\r\n",
        "From: Sender <sender@example.invalid>\r\n",
        "To: Recipient <recipient@example.invalid>\r\n",
        "Content-Type: multipart/mixed; boundary=\"continued-boundary\"\r\n",
        "\r\n",
        "--continued-boundary\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Body.\r\n",
        "--continued-boundary\r\n",
        "Content-Type: application/pdf;\r\n",
        " name*0*=UTF-8''quarterly%20;\r\n",
        " name*1*=%D1%84%D0%B0%D0%B9%D0%BB;\r\n",
        " name*2=.pdf\r\n",
        "Content-Disposition: attachment;\r\n",
        " filename*0*=UTF-8''quarterly%20;\r\n",
        " filename*1*=%D1%84%D0%B0%D0%B9%D0%BB;\r\n",
        " filename*2=.pdf\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "JVBERi0xLjQ=\r\n",
        "--continued-boundary--\r\n"
    );

    let parsed = parse_rfc822_message(raw.as_bytes()).expect("parse continued filename message");

    assert_eq!(parsed.attachments.len(), 1);
    let attachment = &parsed.attachments[0];
    assert_eq!(attachment.filename.as_deref(), Some("quarterly файл.pdf"));
    assert_eq!(attachment.content_type, "application/pdf");
    assert_eq!(attachment.body_bytes, b"%PDF-1.4");
}

#[test]
fn rfc822_parser_decodes_legacy_cyrillic_message_bytes() {
    let mut raw = Vec::new();
    raw.extend_from_slice(b"Subject: ");
    raw.extend_from_slice(&[
        0xd2, 0xe5, 0xf1, 0xf2, 0xee, 0xe2, 0xee, 0xe5, 0x20, 0xef, 0xe8, 0xf1, 0xfc, 0xec, 0xee,
    ]);
    raw.extend_from_slice(b"\r\nFrom: ");
    raw.extend_from_slice(&[
        0xc8, 0xe2, 0xe0, 0xed, 0x20, 0xcf, 0xe5, 0xf2, 0xf0, 0xee, 0xe2,
    ]);
    raw.extend_from_slice(b" <ivan@example.invalid>\r\n");
    raw.extend_from_slice(b"To: Recipient <recipient@example.invalid>\r\n");
    raw.extend_from_slice(b"Content-Type: text/plain; charset=windows-1251\r\n");
    raw.extend_from_slice(b"\r\n");
    raw.extend_from_slice(&[
        0xcf, 0xf0, 0xe8, 0xe2, 0xe5, 0xf2, 0x2c, 0x20, 0xfd, 0xf2, 0xee, 0x20, 0xf1, 0xf2, 0xe0,
        0xf0, 0xee, 0xe5, 0x20, 0xef, 0xe8, 0xf1, 0xfc, 0xec, 0xee, 0x2e,
    ]);

    let parsed = parse_rfc822_message(&raw).expect("parse legacy cyrillic message");

    assert_eq!(parsed.subject, "Тестовое письмо");
    assert_eq!(parsed.from, "Иван Петров <ivan@example.invalid>");
    assert_eq!(parsed.body_text, "Привет, это старое письмо.");
    assert!(!parsed.body_text.contains('\u{fffd}'));
}
```

### `backend/tests/email_sync.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_sync.rs`
- Size bytes / Размер в байтах: `10600`
- Included characters / Включено символов: `10600`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use serde_json::json;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
    ProviderAccountSecretPurpose,
};
use makosh_hub_backend::integrations::mail::sync::{
    EmailSyncAdapterConfig, EmailSyncPlanError, plan_email_sync,
};
use makosh_hub_backend::platform::storage::Database;

#[tokio::test]
async fn email_sync_plan_selects_provider_specific_credentials_and_streams_against_postgres() {
    let Some((store, suffix)) = live_sync_context("email sync provider plans").await else {
        return;
    };

    let gmail = store
        .upsert_provider_account(
            &NewProviderAccount::new(
                format!("acct_sync_gmail_{suffix}"),
                EmailProviderKind::Gmail,
                "Gmail sync",
                format!("gmail-sync-{suffix}@example.com"),
            )
            .config(json!({"history_stream_id": "gmail:history:primary"})),
        )
        .await
        .expect("store gmail account");
    let icloud = store
        .upsert_provider_account(
            &NewProviderAccount::new(
                format!("acct_sync_icloud_{suffix}"),
                EmailProviderKind::Icloud,
                "iCloud sync",
                format!("icloud-sync-{suffix}@icloud.com"),
            )
            .config(json!({
                "host": "imap.mail.me.com",
                "port": 993,
                "tls": true,
                "mailbox": "Archive"
            })),
        )
        .await
        .expect("store icloud account");
    let imap = store
        .upsert_provider_account(
            &NewProviderAccount::new(
                format!("acct_sync_imap_{suffix}"),
                EmailProviderKind::Imap,
                "IMAP sync",
                format!("imap-sync-{suffix}@example.net"),
            )
            .config(json!({
                "host": "imap.example.net",
                "port": 1993,
                "tls": true
            })),
        )
        .await
        .expect("store imap account");

    let gmail_plan = plan_email_sync(&gmail).expect("gmail sync plan");
    assert_eq!(
        gmail_plan.credential_purpose,
        ProviderAccountSecretPurpose::OauthToken
    );
    assert_eq!(gmail_plan.stream_id, "gmail:history:primary");
    assert_eq!(
        gmail_plan.adapter_config,
        EmailSyncAdapterConfig::Gmail {
            history_stream_id: "gmail:history:primary".to_owned(),
        }
    );

    let icloud_plan = plan_email_sync(&icloud).expect("icloud sync plan");
    assert_eq!(
        icloud_plan.credential_purpose,
        ProviderAccountSecretPurpose::ImapPassword
    );
    assert_eq!(icloud_plan.stream_id, "imap:Archive");
    assert_eq!(
        icloud_plan.adapter_config,
        EmailSyncAdapterConfig::Imap {
            host: "imap.mail.me.com".to_owned(),
            port: 993,
            tls: true,
            mailbox: "Archive".to_owned(),
        }
    );

    let imap_plan = plan_email_sync(&imap).expect("imap sync plan");
    assert_eq!(
        imap_plan.credential_purpose,
        ProviderAccountSecretPurpose::ImapPassword
    );
    assert_eq!(imap_plan.stream_id, "imap:INBOX");
    assert_eq!(
        imap_plan.adapter_config,
        EmailSyncAdapterConfig::Imap {
            host: "imap.example.net".to_owned(),
            port: 1993,
            tls: true,
            mailbox: "INBOX".to_owned(),
        }
    );
}

#[tokio::test]
async fn email_sync_plan_keeps_multiple_accounts_isolated_against_postgres() {
    let Some((store, suffix)) = live_sync_context("multi-account email sync planning").await else {
        return;
    };

    let first = store
        .upsert_provider_account(
            &NewProviderAccount::new(
                format!("acct_sync_multi_gmail_a_{suffix}"),
                EmailProviderKind::Gmail,
                "Gmail work sync",
                format!("gmail-work-sync-{suffix}@example.com"),
            )
            .config(json!({"history_stream_id": "gmail:history:work"})),
        )
        .await
        .expect("store first gmail account");
    let second = store
        .upsert_provider_account(
            &NewProviderAccount::new(
                format!("acct_sync_multi_gmail_b_{suffix}"),
                EmailProviderKind::Gmail,
                "Gmail personal sync",
                format!("gmail-personal-sync-{suffix}@example.com"),
            )
            .config(json!({"history_stream_id": "gmail:history:personal"})),
        )
        .await
        .expect("store second gmail account");

    let first_plan = plan_email_sync(&first).expect("first gmail plan");
    let second_plan = plan_email_sync(&second).expect("second gmail plan");

    assert_ne!(first_plan.account_id, second_plan.account_id);
    assert_eq!(first_plan.stream_id, "gmail:history:work");
    assert_eq!(second_plan.stream_id, "gmail:history:personal");
}

#[test]
fn email_sync_plan_rejects_invalid_imap_config() {
    let cases = [
        (
            "host",
            NewProviderAccount::new(
                "acct_invalid_imap_host",
                EmailProviderKind::Imap,
                "Invalid IMAP host",
                "invalid-imap@example.net",
            )
            .config(json!({"host": " ", "port": 993, "tls": true})),
        ),
        (
            "port",
            NewProviderAccount::new(
                "acct_invalid_imap_port",
                EmailProviderKind::Imap,
                "Invalid IMAP port",
                "invalid-imap-port@example.net",
            )
            .config(json!({"host": "imap.example.net", "port": 0, "tls": true})),
        ),
        (
            "tls",
            NewProviderAccount::new(
                "acct_invalid_imap_tls",
                EmailProviderKind::Imap,
                "Invalid IMAP TLS",
                "invalid-imap-tls@example.net",
            )
            .config(json!({"host": "imap.example.net", "port": 993, "tls": "yes"})),
        ),
        (
            "mailbox",
            NewProviderAccount::new(
                "acct_invalid_imap_mailbox",
                EmailProviderKind::Imap,
                "Invalid IMAP mailbox",
                "invalid-imap-mailbox@example.net",
            )
            .config(json!({"host": "imap.example.net", "port": 993, "tls": true, "mailbox": "Inbox\nArchive"})),
        ),
    ];

    for (field_name, account) in cases {
        let account = account.into_test_provider_account();
        let error = plan_email_sync(&account).expect_err("invalid IMAP config must fail");

        assert!(
            matches!(
                error,
                EmailSyncPlanError::InvalidProviderConfig { field, .. } if field == field_name
            ),
            "expected invalid field {field_name}, got {error:?}"
        );
    }
}

#[test]
fn email_sync_plan_rejects_secret_like_account_config() {
    let cases = [
        (
            "oauth_token",
            NewProviderAccount::new(
                "acct_secret_config",
                EmailProviderKind::Gmail,
                "Gmail unsafe config",
                "unsafe-config@example.com",
            )
            .config(json!({
                "oauth_token": "must-not-be-here",
                "history_stream_id": "gmail:history"
            })),
        ),
        (
            "adapter.oauth_token",
            NewProviderAccount::new(
                "acct_nested_secret_config",
                EmailProviderKind::Gmail,
                "Gmail nested unsafe config",
                "nested-unsafe-config@example.com",
            )
            .config(json!({
                "adapter": {
                    "oauth_token": "must-not-be-here"
                },
                "history_stream_id": "gmail:history"
            })),
        ),
    ];

    for (expected_key, account) in cases {
        let account = account.into_test_provider_account();
        let error = plan_email_sync(&account).expect_err("secret-like config must fail");

        assert!(
            matches!(
                error,
                EmailSyncPlanError::SecretLikeConfigKey { ref key } if key == expected_key
            ),
            "expected secret-like key {expected_key}, got {error:?}"
        );
    }
}

#[test]
fn email_sync_plan_uses_delimiter_safe_imap_stream_id() {
    let account = NewProviderAccount::new(
        "acct_imap_delimiter_mailbox",
        EmailProviderKind::Imap,
        "Delimiter mailbox",
        "delimiter-mailbox@example.net",
    )
    .config(json!({
        "host": "imap.example.net",
        "port": 993,
        "tls": true,
        "mailbox": "Projects:2026%Q2"
    }))
    .into_test_provider_account();

    let plan = plan_email_sync(&account).expect("delimiter-safe IMAP plan");

    assert_eq!(plan.stream_id, "imap:Projects%3A2026%25Q2");
    assert_eq!(
        plan.adapter_config,
        EmailSyncAdapterConfig::Imap {
            host: "imap.example.net".to_owned(),
            port: 993,
            tls: true,
            mailbox: "Projects:2026%Q2".to_owned(),
        }
    );
}

async fn live_sync_context(_test_name: &str) -> Option<(CommunicationIngestionStore, u128)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = CommunicationIngestionStore::new(database.pool().expect("configured pool").clone());

    Some((store, unique_suffix()))
}

trait IntoTestProviderAccount {
    fn into_test_provider_account(
        self,
    ) -> makosh_hub_backend::domains::communications::core::ProviderAccount;
}

impl IntoTestProviderAccount for NewProviderAccount {
    fn into_test_provider_account(
        self,
    ) -> makosh_hub_backend::domains::communications::core::ProviderAccount {
        let now = chrono::Utc::now();

        makosh_hub_backend::domains::communications::core::ProviderAccount {
            account_id: self.account_id,
            provider_kind: self.provider_kind,
            display_name: self.display_name,
            external_account_id: self.external_account_id,
            config: self.config,
            created_at: now,
            updated_at: now,
        }
    }
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
```
