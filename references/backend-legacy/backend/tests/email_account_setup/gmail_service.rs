use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use serde_json::{Value, json};

use std::sync::Arc;

use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_hub_backend::integrations::mail::accounts::models::GmailOAuthSetupRequest;
use makosh_hub_backend::integrations::mail::accounts::service::EmailAccountSetupService;

use makosh_hub_backend::platform::secrets::{
    database_vault::DatabaseEncryptedSecretVault,
    models::{NewSecretReference, ResolvedSecret, SecretKind, SecretStoreKind},
    resolver::SecretResolver,
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
    assert!(pending.authorization_url.contains("gmail.modify"));
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
    assert_eq!(account.provider_kind, CommunicationProviderKind::Gmail);
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
