use crate::support::*;
use makosh_communications_api::accounts::{
    CommunicationProviderKind, NewProviderAccount, NewProviderAccountSecretBinding,
    ProviderAccountSecretPurpose,
};

#[tokio::test]
async fn provider_credential_reader_resolves_bound_account_secret_against_postgres() {
    let Some(database) = connect_database("provider credential reader test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_binding_store =
        makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        );
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_credential_reader_{suffix}");
    let secret_ref = format!("secret:test:credential-reader:{suffix}");
    let mut resolver = InMemorySecretResolver::new();

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Gmail credential reader",
            format!("credential-reader-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::OauthToken,
            SecretStoreKind::TestDouble,
            "Gmail test credential",
        ))
        .await
        .expect("store secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &account_id,
            ProviderAccountSecretPurpose::OauthToken,
            &secret_ref,
        ))
        .await
        .expect("bind account secret");
    resolver
        .insert(&secret_ref, "test-only-gmail-runtime-value")
        .expect("insert in-memory runtime value");

    let reader = ProviderCredentialReader::new(secret_binding_store, secret_store, &resolver);
    let credential = reader
        .read(&account_id, ProviderAccountSecretPurpose::OauthToken)
        .await
        .expect("read provider credential");

    assert_eq!(credential.binding.account_id, account_id);
    assert_eq!(
        credential.binding.secret_purpose,
        ProviderAccountSecretPurpose::OauthToken
    );
    assert_eq!(credential.reference.secret_ref, secret_ref);
    assert_eq!(credential.reference.secret_kind, SecretKind::OauthToken);
    assert_eq!(
        credential.secret.expose_for_runtime(),
        "test-only-gmail-runtime-value"
    );
    assert!(!format!("{credential:?}").contains("test-only-gmail-runtime-value"));
}

#[tokio::test]
async fn provider_credential_reader_reports_missing_binding_against_postgres() {
    let Some(database) =
        connect_database("missing provider credential binding test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_binding_store =
        makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        );
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_missing_credential_binding_{suffix}");
    let resolver = InMemorySecretResolver::new();

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Icloud,
            "iCloud missing credential binding",
            format!("missing-credential-binding-{suffix}@icloud.com"),
        ))
        .await
        .expect("store provider account");

    let reader = ProviderCredentialReader::new(secret_binding_store, secret_store, &resolver);
    let error = reader
        .read(&account_id, ProviderAccountSecretPurpose::ImapPassword)
        .await
        .expect_err("missing credential binding should fail");

    match error {
        ProviderCredentialError::MissingBinding {
            account_id: error_account_id,
            secret_purpose,
        } => {
            assert_eq!(error_account_id, account_id);
            assert_eq!(secret_purpose, ProviderAccountSecretPurpose::ImapPassword);
        }
        other => panic!("unexpected provider credential error: {other:?}"),
    }
}

#[tokio::test]
async fn provider_credential_reader_propagates_resolver_failures_against_postgres() {
    let Some(database) =
        connect_database("provider credential resolver failure test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_binding_store =
        makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        );
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_resolver_failure_{suffix}");
    let secret_ref = format!("secret:os-keychain:resolver-failure:{suffix}");
    let mut resolver = InMemorySecretResolver::new();

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "IMAP resolver failure",
            format!("resolver-failure-{suffix}@example.net"),
        ))
        .await
        .expect("store provider account");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::Password,
            SecretStoreKind::OsKeychain,
            "IMAP keychain credential",
        ))
        .await
        .expect("store secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &account_id,
            ProviderAccountSecretPurpose::ImapPassword,
            &secret_ref,
        ))
        .await
        .expect("bind account secret");
    resolver
        .insert(&secret_ref, "test-only-imap-runtime-value")
        .expect("insert in-memory runtime value");

    let reader = ProviderCredentialReader::new(secret_binding_store, secret_store, &resolver);
    let error = reader
        .read(&account_id, ProviderAccountSecretPurpose::ImapPassword)
        .await
        .expect_err("unsupported resolver store kind should fail");

    match error {
        ProviderCredentialError::SecretResolution(SecretResolutionError::UnsupportedStoreKind(
            store_kind,
        )) => assert_eq!(store_kind, "os_keychain"),
        other => panic!("unexpected provider credential error: {other:?}"),
    }
}

#[tokio::test]
async fn provider_credential_reader_rejects_incompatible_secret_kind_against_postgres() {
    let Some(database) =
        connect_database("incompatible provider credential kind test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_binding_store =
        makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        );
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_incompatible_credential_kind_{suffix}");
    let secret_ref = format!("secret:test:incompatible-kind:{suffix}");
    let resolver = InMemorySecretResolver::new();

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Gmail incompatible credential kind",
            format!("incompatible-kind-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::Password,
            SecretStoreKind::TestDouble,
            "Wrong Gmail credential kind",
        ))
        .await
        .expect("store incompatible secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &account_id,
            ProviderAccountSecretPurpose::OauthToken,
            &secret_ref,
        ))
        .await
        .expect("bind incompatible account secret");

    let reader = ProviderCredentialReader::new(secret_binding_store, secret_store, &resolver);
    let error = reader
        .read(&account_id, ProviderAccountSecretPurpose::OauthToken)
        .await
        .expect_err("incompatible credential kind should fail");

    match error {
        ProviderCredentialError::IncompatibleSecretKind {
            secret_ref: error_secret_ref,
            secret_purpose,
            secret_kind,
        } => {
            assert_eq!(error_secret_ref, secret_ref);
            assert_eq!(secret_purpose, ProviderAccountSecretPurpose::OauthToken);
            assert_eq!(secret_kind, SecretKind::Password);
        }
        other => panic!("unexpected provider credential error: {other:?}"),
    }
}
