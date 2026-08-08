use crate::support::*;
use makosh_communications_api::accounts::{
    CommunicationProviderKind, NewProviderAccount, NewProviderAccountSecretBinding,
    ProviderAccountSecretPurpose,
};

#[tokio::test]
async fn communication_ingestion_binds_provider_accounts_to_secret_refs_against_postgres() {
    let Some(database) =
        connect_database("communication secret binding test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let gmail_account_id = format!("acct_gmail_secret_{suffix}");
    let icloud_account_id = format!("acct_icloud_secret_{suffix}");
    let imap_account_id = format!("acct_imap_secret_{suffix}");

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &gmail_account_id,
            CommunicationProviderKind::Gmail,
            "Gmail secret binding",
            format!("gmail-secret-{suffix}@example.com"),
        ))
        .await
        .expect("store gmail account");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &icloud_account_id,
            CommunicationProviderKind::Icloud,
            "iCloud secret binding",
            format!("icloud-secret-{suffix}@icloud.com"),
        ))
        .await
        .expect("store icloud account");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &imap_account_id,
            CommunicationProviderKind::Imap,
            "IMAP secret binding",
            format!("imap-secret-{suffix}@example.net"),
        ))
        .await
        .expect("store imap account");

    let gmail_secret_ref = format!("secret:gmail:oauth:{suffix}");
    let icloud_secret_ref = format!("secret:icloud:app-password:{suffix}");
    let imap_secret_ref = format!("secret:imap:password:{suffix}");

    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &gmail_secret_ref,
            SecretKind::OauthToken,
            SecretStoreKind::OsKeychain,
            "Gmail OAuth credential",
        ))
        .await
        .expect("store gmail secret reference");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &icloud_secret_ref,
            SecretKind::AppPassword,
            SecretStoreKind::OsKeychain,
            "iCloud app-specific password",
        ))
        .await
        .expect("store icloud secret reference");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &imap_secret_ref,
            SecretKind::Password,
            SecretStoreKind::OsKeychain,
            "Generic IMAP password",
        ))
        .await
        .expect("store imap secret reference");

    let gmail_binding = communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &gmail_account_id,
            ProviderAccountSecretPurpose::OauthToken,
            &gmail_secret_ref,
        ))
        .await
        .expect("bind gmail oauth secret");
    let icloud_binding = communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &icloud_account_id,
            ProviderAccountSecretPurpose::ImapPassword,
            &icloud_secret_ref,
        ))
        .await
        .expect("bind icloud imap secret");
    let imap_binding = communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &imap_account_id,
            ProviderAccountSecretPurpose::ImapPassword,
            &imap_secret_ref,
        ))
        .await
        .expect("bind generic imap secret");

    assert_eq!(gmail_binding.secret_ref, gmail_secret_ref);
    assert_eq!(icloud_binding.secret_ref, icloud_secret_ref);
    assert_eq!(imap_binding.secret_ref, imap_secret_ref);

    let gmail_bindings = communication_store
        .provider_account_secret_bindings(&gmail_account_id)
        .await
        .expect("load gmail secret bindings");
    assert_eq!(gmail_bindings, vec![gmail_binding]);
}

#[tokio::test]
async fn communication_ingestion_scopes_secret_refs_by_provider_account_against_postgres() {
    let Some(database) =
        connect_database("multi-account secret binding test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let accounts = [
        (
            format!("acct_multi_gmail_a_{suffix}"),
            CommunicationProviderKind::Gmail,
            "Gmail work",
            format!("gmail-work-{suffix}@example.com"),
            ProviderAccountSecretPurpose::OauthToken,
            SecretKind::OauthToken,
            format!("secret:test:gmail:work:{suffix}"),
            "fake-gmail-work-runtime-secret",
        ),
        (
            format!("acct_multi_gmail_b_{suffix}"),
            CommunicationProviderKind::Gmail,
            "Gmail personal",
            format!("gmail-personal-{suffix}@example.com"),
            ProviderAccountSecretPurpose::OauthToken,
            SecretKind::OauthToken,
            format!("secret:test:gmail:personal:{suffix}"),
            "fake-gmail-personal-runtime-secret",
        ),
        (
            format!("acct_multi_icloud_a_{suffix}"),
            CommunicationProviderKind::Icloud,
            "iCloud work",
            format!("icloud-work-{suffix}@icloud.com"),
            ProviderAccountSecretPurpose::ImapPassword,
            SecretKind::AppPassword,
            format!("secret:test:icloud:work:{suffix}"),
            "fake-icloud-work-runtime-secret",
        ),
        (
            format!("acct_multi_icloud_b_{suffix}"),
            CommunicationProviderKind::Icloud,
            "iCloud personal",
            format!("icloud-personal-{suffix}@icloud.com"),
            ProviderAccountSecretPurpose::ImapPassword,
            SecretKind::AppPassword,
            format!("secret:test:icloud:personal:{suffix}"),
            "fake-icloud-personal-runtime-secret",
        ),
    ];
    let mut resolver = InMemorySecretResolver::new();

    for (
        account_id,
        provider_kind,
        display_name,
        external_account_id,
        secret_purpose,
        secret_kind,
        secret_ref,
        runtime_value,
    ) in &accounts
    {
        communication_store
            .upsert_provider_account(&NewProviderAccount::new(
                account_id.as_str(),
                *provider_kind,
                *display_name,
                external_account_id.as_str(),
            ))
            .await
            .expect("store provider account");
        secret_store
            .upsert_secret_reference(&NewSecretReference::new(
                secret_ref.as_str(),
                *secret_kind,
                SecretStoreKind::TestDouble,
                format!("{display_name} credential"),
            ))
            .await
            .expect("store secret reference");
        communication_store
            .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
                account_id.as_str(),
                *secret_purpose,
                secret_ref.as_str(),
            ))
            .await
            .expect("bind account secret");
        resolver
            .insert(secret_ref.as_str(), *runtime_value)
            .expect("insert in-memory runtime secret");
    }

    for (
        account_id,
        _provider_kind,
        _display_name,
        _external_account_id,
        secret_purpose,
        _secret_kind,
        secret_ref,
        runtime_value,
    ) in &accounts
    {
        let binding = communication_store
            .provider_account_secret_binding(account_id.as_str(), *secret_purpose)
            .await
            .expect("load account secret binding")
            .expect("account secret binding exists");
        assert_eq!(binding.account_id, *account_id);
        assert_eq!(binding.secret_ref, *secret_ref);

        let reference = secret_store
            .secret_reference(&binding.secret_ref)
            .await
            .expect("load secret reference")
            .expect("secret reference exists");
        let resolved = resolver
            .resolve(&reference)
            .await
            .expect("resolve account-scoped secret");
        assert_eq!(resolved.expose_for_runtime(), *runtime_value);
    }

    let first_gmail_binding = communication_store
        .provider_account_secret_binding(&accounts[0].0, ProviderAccountSecretPurpose::OauthToken)
        .await
        .expect("load first gmail binding")
        .expect("first gmail binding exists");
    let second_gmail_binding = communication_store
        .provider_account_secret_binding(&accounts[1].0, ProviderAccountSecretPurpose::OauthToken)
        .await
        .expect("load second gmail binding")
        .expect("second gmail binding exists");
    assert_ne!(
        first_gmail_binding.secret_ref,
        second_gmail_binding.secret_ref
    );

    let first_icloud_binding = communication_store
        .provider_account_secret_binding(&accounts[2].0, ProviderAccountSecretPurpose::ImapPassword)
        .await
        .expect("load first icloud binding")
        .expect("first icloud binding exists");
    let second_icloud_binding = communication_store
        .provider_account_secret_binding(&accounts[3].0, ProviderAccountSecretPurpose::ImapPassword)
        .await
        .expect("load second icloud binding")
        .expect("second icloud binding exists");
    assert_ne!(
        first_icloud_binding.secret_ref,
        second_icloud_binding.secret_ref
    );
}
