use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use std::sync::Arc;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::calendar::events::account_store::CalendarAccountStore;
use makosh_hub_backend::integrations::mail::accounts::models::ImapAccountSetupRequest;
use makosh_hub_backend::integrations::mail::accounts::service::EmailAccountSetupService;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::secrets::{
    database_vault::DatabaseEncryptedSecretVault,
    models::{ResolvedSecret, SecretKind, SecretStoreKind},
    resolver::SecretResolver,
    store::SecretReferenceStore,
};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::{HostVault, models::HostVaultConfig};

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
                CommunicationProviderKind::Icloud,
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
    assert_eq!(account.provider_kind, CommunicationProviderKind::Icloud);
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
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
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
    assert_eq!(account.provider_kind, CommunicationProviderKind::Icloud);
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
            .expose_for_runtime(),
        "icloud-app-password"
    );

    let calendar_account_id = format!("icloud-calendar:{account_id}");
    let calendar_account = CalendarAccountStore::new(pool.clone())
        .get(&calendar_account_id)
        .await
        .expect("load calendar account")
        .expect("calendar account");
    assert_eq!(calendar_account.provider, "apple");
    assert_eq!(calendar_account.account_name, "Primary iCloud");
    assert_eq!(calendar_account.email.as_deref(), Some("user@icloud.com"));
    assert_eq!(
        calendar_account.credentials_reference.as_deref(),
        Some("secret:provider-account:icloud-primary:imap_password")
    );
    assert_eq!(calendar_account.capabilities["mail_account_id"], account_id);
    assert_eq!(calendar_account.capabilities["source_provider"], "icloud");
    assert_eq!(
        calendar_account.capabilities["connected_services"],
        json!(["calendar"])
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
    .bind(&calendar_account_id)
    .fetch_one(&pool)
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
    .fetch_one(&pool)
    .await
    .expect("calendar account observation");
    assert_eq!(
        calendar_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "local_runtime"
    );
    assert_eq!(
        calendar_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "CALENDAR_ACCOUNT_LINK"
    );
}

#[tokio::test]
async fn imap_account_setup_api_requires_configured_database() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/imap",
            json!({
                "account_id": "acct_no_vault",
                "provider_kind": "imap",
                "display_name": "No vault",
                "external_account_id": "no-vault@example.net",
                "host": "imap.example.net",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "no-vault@example.net",
                "password": "secret"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], "database_not_configured");
}

#[tokio::test]
async fn imap_account_setup_api_requires_initialized_host_vault_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let vault_dir = tempdir().expect("vault tempdir");
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
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

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/imap",
            json!({
                "account_id": "acct_no_vault_key",
                "provider_kind": "imap",
                "display_name": "No vault key",
                "external_account_id": "no-vault-key@example.net",
                "host": "imap.example.net",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "no-vault-key@example.net",
                "password": "secret"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], "host_vault_error");
    assert_eq!(body["message"], "host vault is not initialized");
}
