use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use serde_json::{Value, json};
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::calendar::events::account_store::CalendarAccountStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::secrets::{
    models::SecretStoreKind, resolver::SecretResolver, store::SecretReferenceStore,
};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::{HostVault, models::HostVaultConfig};

use super::support::{
    LOCAL_API_TOKEN, MockTokenServer, get_request, json_body, json_request_with_token_and_actor,
    text_body, unique_suffix, unlock_test_vault,
};

#[tokio::test]
async fn gmail_oauth_start_api_uses_configured_google_desktop_client_against_postgres() {
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
                (
                    "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
                    r#"{
                    "installed": {
                        "client_id": "desktop-client-id.apps.googleusercontent.com",
                        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                        "token_uri": "https://oauth2.googleapis.com/token",
                        "client_secret": "desktop-client-secret",
                        "redirect_uris": ["http://localhost"]
                    }
                }"#,
                ),
            ])
            .expect("config"),
        database.clone(),
    );
    unlock_test_vault(app.clone()).await;

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/gmail/oauth/start",
            json!({
                "account_id": "gmail-primary",
                "display_name": "Google Workspace",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let authorization_url = body["authorization_url"]
        .as_str()
        .expect("authorization url");
    assert!(authorization_url.starts_with("https://accounts.google.com/o/oauth2/auth?"));
    assert!(authorization_url.contains("client_id=desktop-client-id.apps.googleusercontent.com"));
    assert!(authorization_url.contains("gmail.modify"));
    assert!(authorization_url.contains("gmail.send"));
    assert!(authorization_url.contains("calendar.readonly"));
    assert!(authorization_url.contains("contacts.readonly"));

    drop(database);
}

#[tokio::test]
async fn gmail_oauth_start_api_requires_initialized_host_vault_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

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
            (
                "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
                r#"{
                    "installed": {
                        "client_id": "desktop-client-id.apps.googleusercontent.com",
                        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                        "token_uri": "https://oauth2.googleapis.com/token",
                        "client_secret": "desktop-client-secret",
                        "redirect_uris": ["http://localhost"]
                    }
                }"#,
            ),
        ])
        .expect("config"),
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    );

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/gmail/oauth/start",
            json!({
                "account_id": "gmail-primary",
                "display_name": "Google Workspace",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback"
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

#[tokio::test]
async fn gmail_oauth_start_api_reopens_initialized_host_vault_after_restart_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

    let initialized_app = build_router_with_database(
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
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    );
    unlock_test_vault(initialized_app).await;

    let restarted_app = build_router_with_database(
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
            (
                "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
                r#"{
                    "installed": {
                        "client_id": "desktop-client-id.apps.googleusercontent.com",
                        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                        "token_uri": "https://oauth2.googleapis.com/token",
                        "client_secret": "desktop-client-secret",
                        "redirect_uris": ["http://localhost"]
                    }
                }"#,
            ),
        ])
        .expect("config"),
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    );

    let response = restarted_app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/gmail/oauth/start",
            json!({
                "account_id": "gmail-primary",
                "display_name": "Google Workspace",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let authorization_url = body["authorization_url"]
        .as_str()
        .expect("authorization url");
    assert!(authorization_url.starts_with("https://accounts.google.com/o/oauth2/auth?"));
    assert!(authorization_url.contains("client_id=desktop-client-id.apps.googleusercontent.com"));
}

#[tokio::test]
async fn gmail_oauth_callback_completes_pending_grant_without_api_secret() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
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
        database,
    );
    unlock_test_vault(app.clone()).await;

    let token_server = MockTokenServer::start();
    let suffix = unique_suffix();
    let account_id = format!("gmail-callback-{suffix}");
    let start_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/gmail/oauth/start",
            json!({
                "account_id": account_id,
                "display_name": "Google Workspace",
                "client_id": "desktop-client-id",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback",
                "app_return_url": "http://127.0.0.1:5174/?makosh_oauth=gmail_connected",
                "authorization_endpoint": format!("{}/authorize", token_server.base_url()),
                "token_endpoint": format!("{}/token", token_server.base_url())
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    let state = start_body["state"].as_str().expect("state");

    let callback_response = app
        .oneshot(get_request(&format!(
            "/api/v1/integrations/mail/accounts/gmail/oauth/callback?code=authorization-code&state={state}"
        )))
        .await
        .expect("callback response");

    assert_eq!(callback_response.status(), StatusCode::OK);
    let callback_body = text_body(callback_response).await;
    assert!(callback_body.contains("Google mail connected"));
    assert!(callback_body.contains(&account_id));
    assert!(callback_body.contains("makosh:gmail-oauth-connected"));
    assert!(callback_body.contains("postMessage"));
    assert!(callback_body.contains("window.close"));
    assert!(callback_body.contains("makosh_oauth=gmail_connected"));
    assert!(!callback_body.contains("gmail-access-token"));
    assert!(!callback_body.contains("gmail-refresh-token"));

    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let account = communication_store
        .provider_account(&account_id)
        .await
        .expect("load provider account")
        .expect("provider account");
    assert_eq!(account.provider_kind, CommunicationProviderKind::Gmail);
    assert_eq!(account.external_account_id, account_id);
    assert!(account.config.get("access_token").is_none());
    assert!(account.config.get("refresh_token").is_none());
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
    .bind(&account_id)
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
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("gmail signal connection");
    let signal_settings: Value = signal_connection
        .try_get("settings")
        .expect("gmail signal settings");
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
    assert_eq!(signal_settings["provider_kind"], json!("gmail"));
    assert_eq!(signal_settings["external_account_id"], json!(account_id));
    assert_eq!(
        signal_connection
            .try_get::<String, _>("secret_ref")
            .expect("signal secret ref"),
        format!("secret:provider-account:{account_id}:oauth_token")
    );

    let calendar_account_id = format!("google-calendar:{account_id}");
    let calendar_account = CalendarAccountStore::new(pool.clone())
        .get(&calendar_account_id)
        .await
        .expect("load calendar account")
        .expect("calendar account");
    assert_eq!(calendar_account.provider, "google");
    assert_eq!(calendar_account.account_name, "Google Workspace");
    assert_eq!(calendar_account.email.as_deref(), Some(account_id.as_str()));
    assert_eq!(
        calendar_account.credentials_reference.as_deref(),
        Some(format!("secret:provider-account:{account_id}:oauth_token").as_str())
    );
    assert_eq!(calendar_account.capabilities["mail_account_id"], account_id);
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

    let binding = communication_store
        .provider_account_secret_binding(&account_id, ProviderAccountSecretPurpose::OauthToken)
        .await
        .expect("load binding")
        .expect("secret binding");
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
        ProviderAccountSecretPurpose::OauthToken.as_str()
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
    let secret_store = SecretReferenceStore::new(pool);
    let reference = secret_store
        .secret_reference(&binding.secret_ref)
        .await
        .expect("load secret reference")
        .expect("secret reference");
    assert_eq!(reference.store_kind, SecretStoreKind::HostVault);

    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock().expect("unlock host vault");
    let token_bundle = vault
        .resolve(&reference)
        .await
        .expect("resolve token bundle");
    let token_bundle: serde_json::Value =
        serde_json::from_str(token_bundle.expose_for_runtime()).expect("token bundle json");
    assert_eq!(token_bundle["access_token"], "gmail-access-token");
    assert_eq!(token_bundle["refresh_token"], "gmail-refresh-token");
}

#[tokio::test]
async fn gmail_oauth_callback_rejects_unknown_state_without_api_secret() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(
                    "/api/v1/integrations/mail/accounts/gmail/oauth/callback?code=authorization-code&state=oauth-state",
                )
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = text_body(response).await;
    assert!(body.contains("Google mail connection failed"));
    assert!(body.contains("expired"));
    assert!(!body.contains("authorization-code"));
}

#[tokio::test]
async fn gmail_oauth_callback_rejects_missing_code_without_leaking_secrets() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let response = app
        .oneshot(get_request(
            "/api/v1/integrations/mail/accounts/gmail/oauth/callback?state=oauth-state",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = text_body(response).await;
    assert!(body.contains("Google mail connection failed"));
    assert!(body.contains("authorization code"));
    assert!(!body.contains("access_token"));
    assert!(!body.contains("refresh_token"));
}

#[tokio::test]
async fn gmail_oauth_start_and_complete_still_require_api_secret() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let start_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/integrations/mail/accounts/gmail/oauth/start")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "account_id": "gmail-primary",
                        "display_name": "Google Workspace",
                        "client_id": "desktop-client-id",
                        "redirect_uri": "http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("start response");
    assert_eq!(start_response.status(), StatusCode::FORBIDDEN);

    let complete_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/integrations/mail/accounts/gmail/oauth/complete")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "setup_id": "setup",
                        "state": "state",
                        "authorization_code": "code"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("complete response");
    assert_eq!(complete_response.status(), StatusCode::FORBIDDEN);
}
