use serde_json::json;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::{HostVault, models::HostVaultConfig};

use super::support::{
    LOCAL_API_TOKEN, delete_request_with_token, json_body, json_request_with_token_and_actor,
    unlock_test_vault,
};

#[tokio::test]
async fn delete_mail_account_removes_unbound_host_vault_secret_and_reference() {
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
    let app = build_router_with_database(config, database.clone());
    unlock_test_vault(app.clone()).await;

    let account_id = "icloud-delete-vault";
    let secret_ref = "secret:provider-account:icloud-delete-vault:imap_password";
    let smtp_secret_ref = "secret:provider-account:icloud-delete-vault:smtp_password";
    let setup_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "icloud",
                "display_name": "Delete Vault iCloud",
                "external_account_id": "delete-vault@icloud.com",
                "host": "imap.mail.me.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "delete-vault@icloud.com",
                "password": "icloud-app-password",
                "secret_kind": "app_password"
            }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("setup response");
    assert_eq!(setup_response.status(), axum::http::StatusCode::OK);

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool.clone());
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");

    assert!(
        communication_store
            .provider_account(account_id)
            .await
            .expect("provider account before delete")
            .is_some()
    );
    assert!(
        secret_store
            .secret_reference(secret_ref)
            .await
            .expect("secret reference before delete")
            .is_some()
    );

    let delete_response = app
        .oneshot(delete_request_with_token(
            &format!("/api/v1/integrations/mail/accounts/{account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete response");
    assert_eq!(delete_response.status(), axum::http::StatusCode::OK);
    let body = json_body(delete_response).await;
    assert_eq!(body["deleted"], json!(true));
    assert_eq!(
        body["vault_deleted_secret_refs"],
        json!([secret_ref, smtp_secret_ref])
    );
    assert_eq!(body["retained_secret_refs"], json!([]));

    assert!(
        communication_store
            .provider_account(account_id)
            .await
            .expect("provider account after delete")
            .is_none()
    );
    assert!(
        secret_store
            .secret_reference(secret_ref)
            .await
            .expect("secret reference after delete")
            .is_none()
    );
    assert!(
        secret_store
            .secret_reference(smtp_secret_ref)
            .await
            .expect("smtp secret reference after delete")
            .is_none()
    );
    assert!(
        vault
            .account_secret_manifest()
            .expect("manifest after delete")
            .into_iter()
            .all(|entry| entry.secret_ref != secret_ref && entry.secret_ref != smtp_secret_ref)
    );
    assert!(vault.read_secret(secret_ref).is_err());
    assert!(vault.read_secret(smtp_secret_ref).is_err());
}
