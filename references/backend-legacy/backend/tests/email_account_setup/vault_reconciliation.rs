use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use serde_json::json;
use sqlx::Row;
use tempfile::tempdir;
use tokio::time::{Duration, sleep};
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::calendar::events::account_store::CalendarAccountStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::secrets::{
    models::SecretKind, resolver::SecretResolver, store::SecretReferenceStore,
};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::{
    HostVault,
    models::{EntropyEvent, HostVaultConfig, SecretEntryContext},
};

use super::support::{
    LOCAL_API_TOKEN, json_body, json_request_with_token_and_actor, unlock_test_vault,
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
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
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
    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, axum::http::StatusCode::OK, "setup body: {body}");

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

    let mut wipe = pool.begin().await.expect("begin metadata wipe");
    sqlx::query(
        "SELECT account_id FROM communication_provider_accounts WHERE account_id = $1 FOR UPDATE",
    )
    .bind(account_id)
    .fetch_optional(&mut *wipe)
    .await
    .expect("lock provider account metadata");
    sqlx::query("DELETE FROM calendar_accounts WHERE account_id = $1")
        .bind(format!("icloud-calendar:{account_id}"))
        .execute(&mut *wipe)
        .await
        .expect("delete calendar metadata");
    sqlx::query("DELETE FROM communication_provider_account_secret_refs WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *wipe)
        .await
        .expect("delete secret binding");
    sqlx::query("DELETE FROM communication_address_book_sync_runs WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *wipe)
        .await
        .expect("delete address book sync metadata");
    sqlx::query("DELETE FROM communication_provider_accounts WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *wipe)
        .await
        .expect("delete provider account");
    sqlx::query("DELETE FROM secret_references WHERE secret_ref = $1")
        .bind(secret_ref)
        .execute(&mut *wipe)
        .await
        .expect("delete secret reference");
    wipe.commit().await.expect("commit metadata wipe");

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
    assert_eq!(account.provider_kind, CommunicationProviderKind::Icloud);
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

#[tokio::test]
async fn startup_reconciles_non_mail_provider_account_from_host_vault_manifest() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
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
    .expect("config");
    let app = build_router_with_database(config.clone(), database.clone());
    unlock_test_vault(app).await;

    let account_id = "zulip-recover";
    let secret_ref = "secret:provider-account:zulip-recover:zulip_api_key";
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    vault
        .store_secret(
            secret_ref,
            "zulip-api-key",
            SecretEntryContext {
                entry_kind: "provider_api_token",
                account_id,
                purpose: ProviderAccountSecretPurpose::ZulipApiKey.as_str(),
                secret_kind: SecretKind::ApiToken.as_str(),
                label: "Zulip API key",
                metadata: &json!({
                    "provider": CommunicationProviderKind::ZulipBot.as_str(),
                    "account_id": account_id,
                    "display_name": "Recovered Zulip",
                    "external_account_id": "bot@example.zulipchat.com",
                    "provider_account_config": {
                        "base_url": "https://example.zulipchat.com",
                        "runtime": "api"
                    }
                }),
            },
        )
        .expect("store zulip secret");

    let restarted_database = Database::connect(Some(&database_url))
        .await
        .expect("restarted database connection");
    let _restarted_app = build_router_with_database(config, restarted_database.clone());
    let restarted_pool = restarted_database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(restarted_pool.clone());
    let secret_store = SecretReferenceStore::new(restarted_pool);

    let account = wait_for_provider_account(&communication_store, account_id).await;
    assert_eq!(account.provider_kind, CommunicationProviderKind::ZulipBot);
    assert_eq!(account.display_name, "Recovered Zulip");
    assert_eq!(account.external_account_id, "bot@example.zulipchat.com");
    assert_eq!(
        account.config["base_url"],
        json!("https://example.zulipchat.com")
    );

    let reference = wait_for_secret_reference(&secret_store, secret_ref).await;
    assert_eq!(reference.secret_kind, SecretKind::ApiToken);
    assert_eq!(reference.store_kind.as_str(), "host_vault");
    let binding = wait_for_provider_account_secret_binding(
        &communication_store,
        account_id,
        ProviderAccountSecretPurpose::ZulipApiKey,
    )
    .await;
    assert_eq!(binding.secret_ref, secret_ref);
    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve restored zulip secret")
            .expose_for_runtime(),
        "zulip-api-key"
    );
}

#[tokio::test]
async fn startup_reconciles_legacy_gmail_manifest_without_provider_metadata() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
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
    .expect("config");
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home.clone(),
        dev_mode: true,
        dev_key_path: dev_key_path.clone(),
    })
    .expect("host vault");
    let entropy_events: Vec<EntropyEvent> = super::support::vault_entropy_events(2000)
        .into_iter()
        .map(|value| serde_json::from_value(value).expect("entropy event"))
        .collect();
    vault
        .collect_entropy(entropy_events)
        .expect("collect entropy");
    vault.create().expect("create host vault");

    let account_id = "mail-gmail-karelon-gmail-com";
    let secret_ref = "secret:provider-account:mail-gmail-karelon-gmail-com:oauth_token";
    vault
        .store_secret(
            secret_ref,
            "legacy-gmail-oauth-token",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: secret_ref,
                purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "OAuth credential",
                metadata: &json!({}),
            },
        )
        .expect("store legacy gmail secret");

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let _app = build_router_with_database(config, database.clone());
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool);

    let account = wait_for_provider_account(&communication_store, account_id).await;
    assert_eq!(account.provider_kind, CommunicationProviderKind::Gmail);
    assert_eq!(account.display_name, "Google Workspace");
    assert_eq!(account.external_account_id, "karelon@gmail.com");
    assert_eq!(account.config["auth"], json!("oauth"));
    assert_eq!(account.config["api"], json!("gmail"));

    let reference = wait_for_secret_reference(&secret_store, secret_ref).await;
    assert_eq!(reference.secret_kind, SecretKind::OauthToken);
    assert_eq!(reference.store_kind.as_str(), "host_vault");
    let binding = wait_for_provider_account_secret_binding(
        &communication_store,
        account_id,
        ProviderAccountSecretPurpose::OauthToken,
    )
    .await;
    assert_eq!(binding.secret_ref, secret_ref);
    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve restored legacy gmail secret")
            .expose_for_runtime(),
        "legacy-gmail-oauth-token"
    );
}

#[tokio::test]
async fn startup_reconciles_one_account_for_duplicate_provider_external_identity() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
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
    .expect("config");
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    let entropy_events: Vec<EntropyEvent> = super::support::vault_entropy_events(2000)
        .into_iter()
        .map(|value| serde_json::from_value(value).expect("entropy event"))
        .collect();
    vault
        .collect_entropy(entropy_events)
        .expect("collect entropy");
    vault.create().expect("create host vault");

    vault
        .store_secret(
            "secret:provider-account:gmail-duplicate-old:oauth_token",
            "old-gmail-oauth-token",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "gmail-duplicate-old",
                purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "Old Gmail OAuth credential",
                metadata: &json!({
                    "provider": "gmail",
                    "account_id": "gmail-duplicate-old",
                    "display_name": "Old Gmail",
                    "external_account_id": "duplicate@gmail.com",
                    "provider_account_config": {
                        "auth": "oauth",
                        "api": "gmail"
                    }
                }),
            },
        )
        .expect("store old duplicate gmail secret");
    vault
        .store_secret(
            "secret:provider-account:gmail-duplicate-new:oauth_token",
            "new-gmail-oauth-token",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "gmail-duplicate-new",
                purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "New Gmail OAuth credential",
                metadata: &json!({
                    "provider": "gmail",
                    "account_id": "gmail-duplicate-new",
                    "display_name": "New Gmail",
                    "external_account_id": "duplicate@gmail.com",
                    "provider_account_config": {
                        "auth": "oauth",
                        "api": "gmail"
                    }
                }),
            },
        )
        .expect("store new duplicate gmail secret");

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let _app = build_router_with_database(config, database.clone());
    let pool = database.pool().expect("configured pool").clone();

    let mut last_counts = (0_i64, 0_i64, 0_usize);
    for _ in 0..50 {
        let account_count: i64 = sqlx::query_scalar(
            "SELECT count(*)
             FROM communication_provider_accounts
             WHERE provider_kind = 'gmail'
               AND external_account_id = 'duplicate@gmail.com'",
        )
        .fetch_one(&pool)
        .await
        .expect("duplicate account count");
        let binding_count: i64 = sqlx::query_scalar(
            "SELECT count(*)
             FROM communication_provider_account_secret_refs refs
             JOIN communication_provider_accounts accounts
               ON accounts.account_id = refs.account_id
             WHERE accounts.provider_kind = 'gmail'
               AND accounts.external_account_id = 'duplicate@gmail.com'",
        )
        .fetch_one(&pool)
        .await
        .expect("duplicate binding count");
        let manifest_count = vault
            .account_secret_manifest()
            .expect("host vault manifest")
            .into_iter()
            .filter(|entry| {
                entry
                    .metadata
                    .get("provider")
                    .and_then(|value| value.as_str())
                    == Some("gmail")
                    && entry
                        .metadata
                        .get("external_account_id")
                        .and_then(|value| value.as_str())
                        == Some("duplicate@gmail.com")
            })
            .count();

        last_counts = (account_count, binding_count, manifest_count);
        if last_counts == (1, 1, 1) {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!(
        "duplicate Gmail vault entries were not fully reconciled: accounts={}, bindings={}, manifest_entries={}",
        last_counts.0, last_counts.1, last_counts.2
    );
}
