use crate::support::*;
use makosh_communications_api::accounts::{
    CommunicationProviderKind, NewProviderAccount, ProviderAccountSecretPurpose,
};

#[test]
fn email_provider_kind_supports_gmail_icloud_and_raw_imap() {
    assert_eq!(
        CommunicationProviderKind::try_from("gmail").expect("gmail provider kind"),
        CommunicationProviderKind::Gmail
    );
    assert_eq!(
        CommunicationProviderKind::try_from("icloud").expect("icloud provider kind"),
        CommunicationProviderKind::Icloud
    );
    assert_eq!(
        CommunicationProviderKind::try_from("imap").expect("imap provider kind"),
        CommunicationProviderKind::Imap
    );
    assert!(CommunicationProviderKind::try_from("exchange").is_err());
}

#[test]
fn provider_account_secret_purpose_accepts_expected_secret_kinds() {
    assert!(ProviderAccountSecretPurpose::OauthToken.accepts_secret_kind(SecretKind::OauthToken));
    assert!(!ProviderAccountSecretPurpose::OauthToken.accepts_secret_kind(SecretKind::Password));
    assert!(!ProviderAccountSecretPurpose::OauthToken.accepts_secret_kind(SecretKind::AppPassword));

    assert!(ProviderAccountSecretPurpose::ImapPassword.accepts_secret_kind(SecretKind::Password));
    assert!(
        ProviderAccountSecretPurpose::ImapPassword.accepts_secret_kind(SecretKind::AppPassword)
    );
    assert!(
        !ProviderAccountSecretPurpose::ImapPassword.accepts_secret_kind(SecretKind::OauthToken)
    );

    assert!(ProviderAccountSecretPurpose::SmtpPassword.accepts_secret_kind(SecretKind::Password));
    assert!(
        ProviderAccountSecretPurpose::SmtpPassword.accepts_secret_kind(SecretKind::AppPassword)
    );
    assert!(
        !ProviderAccountSecretPurpose::SmtpPassword.accepts_secret_kind(SecretKind::OauthToken)
    );
}

#[tokio::test]
async fn communication_ingestion_registers_email_provider_accounts_against_postgres() {
    let Some(database) = connect_database("communication ingestion test fixture database").await
    else {
        return;
    };

    let store = CommunicationIngestionStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();

    let accounts = [
        NewProviderAccount::new(
            format!("acct_gmail_{suffix}"),
            CommunicationProviderKind::Gmail,
            "Gmail primary",
            format!("gmail-user-{suffix}@example.com"),
        )
        .config(json!({"auth": "oauth", "api": "gmail"})),
        NewProviderAccount::new(
            format!("acct_icloud_{suffix}"),
            CommunicationProviderKind::Icloud,
            "iCloud Mail",
            format!("icloud-user-{suffix}@icloud.com"),
        )
        .config(json!({"auth": "app_password", "transport": "imap"})),
        NewProviderAccount::new(
            format!("acct_imap_{suffix}"),
            CommunicationProviderKind::Imap,
            "Generic IMAP",
            format!("imap-user-{suffix}@example.net"),
        )
        .config(json!({"host": "imap.example.net", "port": 993, "tls": true})),
    ];

    for account in accounts {
        let stored = store
            .upsert_provider_account(&account)
            .await
            .expect("store provider account");

        assert_eq!(stored.account_id, account.account_id);
        assert_eq!(stored.provider_kind, account.provider_kind);
        assert_eq!(stored.external_account_id, account.external_account_id);
        assert_eq!(stored.config, account.config);
    }
}

#[tokio::test]
async fn communication_ingestion_tracks_checkpoints_against_postgres() {
    let Some(database) = connect_database("communication checkpoint test fixture database").await
    else {
        return;
    };

    let store = CommunicationIngestionStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_checkpoint_{suffix}");

    store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Icloud,
            "iCloud checkpoint test",
            format!("checkpoint-{suffix}@icloud.com"),
        ))
        .await
        .expect("store provider account");

    let saved = store
        .save_checkpoint(&NewIngestionCheckpoint::new(
            &account_id,
            "imap:INBOX",
            json!({
                "provider": "icloud",
                "mailbox": "INBOX",
                "uid_validity": 42,
                "last_seen_uid": 1001
            }),
        ))
        .await
        .expect("save checkpoint");

    assert_eq!(saved.account_id, account_id);
    assert_eq!(saved.stream_id, "imap:INBOX");
    assert_eq!(saved.checkpoint["last_seen_uid"], 1001);

    let updated = store
        .save_checkpoint(&NewIngestionCheckpoint::new(
            &saved.account_id,
            &saved.stream_id,
            json!({
                "provider": "icloud",
                "mailbox": "INBOX",
                "uid_validity": 42,
                "last_seen_uid": 1002
            }),
        ))
        .await
        .expect("update checkpoint");

    assert_eq!(updated.checkpoint["last_seen_uid"], 1002);

    let loaded = store
        .checkpoint(&saved.account_id, &saved.stream_id)
        .await
        .expect("load checkpoint")
        .expect("checkpoint exists");
    assert_eq!(loaded.checkpoint["last_seen_uid"], 1002);
}
