use axum::http::StatusCode;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use serde_json::json;
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

use super::support::{
    LOCAL_API_TOKEN, MockSmtpServer, json_body, json_request_with_token_and_actor,
    unlock_test_vault,
};

#[tokio::test]
async fn imap_send_api_queues_outbox_without_direct_smtp_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let pool = ctx.pool().clone();
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
    let canonical_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM communication_accounts WHERE account_id = $1)",
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .expect("canonical account exists query");
    assert!(canonical_exists);

    let send_response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "cc": ["copy@example.com"],
                "subject": "SMTP send test",
                "body_text": "Message body from Макошь test.",
                "draft_id": "draft-local-only",
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
    let outbox = sqlx::query(
        "SELECT status, draft_id, to_participants, cc_participants, bcc_participants, subject
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
    let outbox_draft_id: Option<String> = outbox.try_get("draft_id").expect("outbox draft id");
    let outbox_subject: String = outbox.try_get("subject").expect("outbox subject");
    assert_eq!(outbox_status, "queued");
    assert_eq!(outbox_draft_id, None);
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
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let account_id = "gmail-send-disabled";
    CommunicationIngestionStore::new(pool)
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::Gmail,
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
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
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
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let smtp_server = MockSmtpServer::start();
    let account_id = "imap-send-audit-fail";
    let setup_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "imap",
                "display_name": "IMAP Audit Failure",
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

    sqlx::query("DROP TABLE api_audit_log")
        .execute(&pool)
        .await
        .expect("drop audit table");

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "subject": "Audit fail closed",
                "body_text": "This must not reach SMTP.",
                "confirmed_provider_write": true
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("send response");
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        smtp_server
            .commands()
            .iter()
            .all(|line| !line.starts_with("MAIL FROM"))
    );
}
