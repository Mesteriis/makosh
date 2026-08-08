use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use chrono::Utc;

use makosh_hub_backend::platform::secrets::errors::SecretResolutionError;
use makosh_hub_backend::platform::secrets::models::{
    NewSecretReference, SecretKind, SecretReference, SecretStoreKind,
};
use makosh_hub_backend::platform::secrets::resolver::{InMemorySecretResolver, SecretResolver};
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::platform::storage::database::Database;

#[test]
fn secret_reference_enums_reject_unsupported_values() {
    assert_eq!(
        SecretKind::try_from("oauth_token").expect("oauth token kind"),
        SecretKind::OauthToken
    );
    assert_eq!(
        SecretKind::try_from("app_password").expect("app password kind"),
        SecretKind::AppPassword
    );
    assert_eq!(
        SecretStoreKind::try_from("os_keychain").expect("os keychain store kind"),
        SecretStoreKind::OsKeychain
    );
    assert_eq!(
        SecretStoreKind::try_from("database_encrypted_vault")
            .expect("database encrypted vault store kind"),
        SecretStoreKind::DatabaseEncryptedVault
    );
    assert!(SecretKind::try_from("plain_text").is_err());
    assert!(SecretStoreKind::try_from("postgres").is_err());
}

#[tokio::test]
async fn in_memory_secret_resolver_resolves_test_double_references_without_debug_leaking_value() {
    let mut resolver = InMemorySecretResolver::new();
    resolver
        .insert("secret:test:oauth", "fake-runtime-secret")
        .expect("insert in-memory secret");

    let reference = test_secret_reference(
        "secret:test:oauth",
        SecretKind::OauthToken,
        SecretStoreKind::TestDouble,
    );
    let resolved = resolver
        .resolve(&reference)
        .await
        .expect("resolve test secret");

    assert_eq!(resolved.expose_for_runtime(), "fake-runtime-secret");
    assert!(!format!("{resolved:?}").contains("fake-runtime-secret"));
}

#[tokio::test]
async fn in_memory_secret_resolver_reports_missing_test_double_references() {
    let resolver = InMemorySecretResolver::new();
    let reference = test_secret_reference(
        "secret:test:missing",
        SecretKind::Password,
        SecretStoreKind::TestDouble,
    );

    let error = resolver
        .resolve(&reference)
        .await
        .expect_err("missing in-memory secret should fail");

    assert_eq!(
        error,
        SecretResolutionError::MissingSecret {
            secret_ref: "secret:test:missing".to_owned()
        }
    );
}

#[tokio::test]
async fn in_memory_secret_resolver_rejects_non_test_double_store_kinds() {
    let mut resolver = InMemorySecretResolver::new();
    resolver
        .insert("secret:os:keychain", "fake-runtime-secret")
        .expect("insert in-memory secret");
    let reference = test_secret_reference(
        "secret:os:keychain",
        SecretKind::Password,
        SecretStoreKind::OsKeychain,
    );

    let error = resolver
        .resolve(&reference)
        .await
        .expect_err("non-test store kind should fail");

    assert_eq!(
        error,
        SecretResolutionError::UnsupportedStoreKind("os_keychain".to_owned())
    );
}

#[test]
fn resolved_secret_rejects_empty_values() {
    let error = InMemorySecretResolver::new()
        .insert("secret:test:empty", " ")
        .expect_err("empty secret value should fail");

    assert_eq!(error, SecretResolutionError::EmptySecretValue);
}

#[tokio::test]
async fn secret_references_store_only_metadata_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = SecretReferenceStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();
    let secret_ref = format!("secret:gmail:oauth:{suffix}");

    let stored = store
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret_ref,
                SecretKind::OauthToken,
                SecretStoreKind::OsKeychain,
                "Gmail OAuth credential",
            )
            .metadata(json!({
                "service": "makosh",
                "account": format!("gmail-user-{suffix}@example.com")
            })),
        )
        .await
        .expect("store secret reference");

    assert_eq!(stored.secret_ref, secret_ref);
    assert_eq!(stored.secret_kind, SecretKind::OauthToken);
    assert_eq!(stored.store_kind, SecretStoreKind::OsKeychain);
    assert_eq!(stored.metadata["service"], "makosh");

    let loaded = store
        .secret_reference(&stored.secret_ref)
        .await
        .expect("load secret reference")
        .expect("secret reference exists");
    assert_eq!(loaded, stored);
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn test_secret_reference(
    secret_ref: &str,
    secret_kind: SecretKind,
    store_kind: SecretStoreKind,
) -> SecretReference {
    let now = Utc::now();

    SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind,
        store_kind,
        label: "Test secret reference".to_owned(),
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    }
}
