use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::email_sync::EmailSyncAdapterConfig;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::platform::communications::email_sync::{
    EmailSyncPlanError, plan_email_sync,
};
use makosh_hub_backend::platform::communications::email_sync::{
    IMAP_ALL_MAILBOXES, email_sync_plan_selects_all_imap_mailboxes, email_sync_plan_stream_ids,
};
use makosh_hub_backend::platform::storage::database::Database;

#[tokio::test]
async fn email_sync_plan_selects_provider_specific_credentials_and_streams_against_postgres() {
    let Some((store, suffix)) = live_sync_context("email sync provider plans").await else {
        return;
    };

    let gmail = store
        .upsert_provider_account(
            &NewProviderAccount::new(
                format!("acct_sync_gmail_{suffix}"),
                CommunicationProviderKind::Gmail,
                "Gmail sync",
                format!("gmail-sync-{suffix}@example.com"),
            )
            .config(json!({"history_stream_id": "gmail:history:primary"})),
        )
        .await
        .expect("store gmail account");
    let icloud = store
        .upsert_provider_account(
            &NewProviderAccount::new(
                format!("acct_sync_icloud_{suffix}"),
                CommunicationProviderKind::Icloud,
                "iCloud sync",
                format!("icloud-sync-{suffix}@icloud.com"),
            )
            .config(json!({
                "host": "imap.mail.me.com",
                "port": 993,
                "tls": true,
                "mailbox": "Archive"
            })),
        )
        .await
        .expect("store icloud account");
    let imap = store
        .upsert_provider_account(
            &NewProviderAccount::new(
                format!("acct_sync_imap_{suffix}"),
                CommunicationProviderKind::Imap,
                "IMAP sync",
                format!("imap-sync-{suffix}@example.net"),
            )
            .config(json!({
                "host": "imap.example.net",
                "port": 1993,
                "tls": true
            })),
        )
        .await
        .expect("store imap account");

    let gmail_plan = plan_email_sync(&gmail).expect("gmail sync plan");
    assert_eq!(
        gmail_plan.credential_purpose,
        ProviderAccountSecretPurpose::OauthToken
    );
    assert_eq!(gmail_plan.stream_id, "gmail:history:primary");
    assert_eq!(
        gmail_plan.adapter_config,
        EmailSyncAdapterConfig::Gmail {
            history_stream_id: "gmail:history:primary".to_owned(),
        }
    );

    let icloud_plan = plan_email_sync(&icloud).expect("icloud sync plan");
    assert_eq!(
        icloud_plan.credential_purpose,
        ProviderAccountSecretPurpose::ImapPassword
    );
    assert_eq!(icloud_plan.stream_id, "imap:Archive");
    assert_eq!(
        icloud_plan.adapter_config,
        EmailSyncAdapterConfig::Imap {
            host: "imap.mail.me.com".to_owned(),
            port: 993,
            tls: true,
            mailboxes: vec!["Archive".to_owned()],
        }
    );

    let imap_plan = plan_email_sync(&imap).expect("imap sync plan");
    assert_eq!(
        imap_plan.credential_purpose,
        ProviderAccountSecretPurpose::ImapPassword
    );
    assert_eq!(imap_plan.stream_id, "imap:INBOX");
    assert_eq!(
        imap_plan.adapter_config,
        EmailSyncAdapterConfig::Imap {
            host: "imap.example.net".to_owned(),
            port: 1993,
            tls: true,
            mailboxes: vec!["INBOX".to_owned()],
        }
    );
}

#[test]
fn email_sync_plan_supports_multiple_imap_mailboxes() {
    let account = NewProviderAccount::new(
        "acct_multi_mailbox_imap",
        CommunicationProviderKind::Icloud,
        "Multi mailbox iCloud",
        "multi-mailbox@example.net",
    )
    .config(json!({
        "host": "imap.mail.me.com",
        "port": 993,
        "tls": true,
        "mailbox": "INBOX",
        "mailboxes": ["INBOX", "Junk", "Archive", "Junk"]
    }))
    .into_test_provider_account();

    let plan = plan_email_sync(&account).expect("multi-mailbox IMAP plan");

    assert_eq!(plan.stream_id, "imap:INBOX");
    assert_eq!(
        plan.adapter_config,
        EmailSyncAdapterConfig::Imap {
            host: "imap.mail.me.com".to_owned(),
            port: 993,
            tls: true,
            mailboxes: vec!["INBOX".to_owned(), "Junk".to_owned(), "Archive".to_owned()],
        }
    );
}

#[test]
fn email_sync_plan_can_select_all_imap_mailboxes() {
    let account = NewProviderAccount::new(
        "acct_all_mailboxes_imap",
        CommunicationProviderKind::Icloud,
        "All mailboxes iCloud",
        "all-mailboxes@example.net",
    )
    .config(json!({
        "host": "imap.mail.me.com",
        "port": 993,
        "tls": true,
        "sync_all_mailboxes": true,
        "mailboxes": ["INBOX"]
    }))
    .into_test_provider_account();

    let plan = plan_email_sync(&account).expect("all mailbox IMAP plan");

    assert_eq!(plan.stream_id, "imap:*");
    assert!(email_sync_plan_selects_all_imap_mailboxes(&plan));
    assert!(email_sync_plan_stream_ids(&plan).is_empty());
    assert_eq!(
        plan.adapter_config,
        EmailSyncAdapterConfig::Imap {
            host: "imap.mail.me.com".to_owned(),
            port: 993,
            tls: true,
            mailboxes: vec![IMAP_ALL_MAILBOXES.to_owned()],
        }
    );
}

#[tokio::test]
async fn email_sync_plan_keeps_multiple_accounts_isolated_against_postgres() {
    let Some((store, suffix)) = live_sync_context("multi-account email sync planning").await else {
        return;
    };

    let first = store
        .upsert_provider_account(
            &NewProviderAccount::new(
                format!("acct_sync_multi_gmail_a_{suffix}"),
                CommunicationProviderKind::Gmail,
                "Gmail work sync",
                format!("gmail-work-sync-{suffix}@example.com"),
            )
            .config(json!({"history_stream_id": "gmail:history:work"})),
        )
        .await
        .expect("store first gmail account");
    let second = store
        .upsert_provider_account(
            &NewProviderAccount::new(
                format!("acct_sync_multi_gmail_b_{suffix}"),
                CommunicationProviderKind::Gmail,
                "Gmail personal sync",
                format!("gmail-personal-sync-{suffix}@example.com"),
            )
            .config(json!({"history_stream_id": "gmail:history:personal"})),
        )
        .await
        .expect("store second gmail account");

    let first_plan = plan_email_sync(&first).expect("first gmail plan");
    let second_plan = plan_email_sync(&second).expect("second gmail plan");

    assert_ne!(first_plan.account_id, second_plan.account_id);
    assert_eq!(first_plan.stream_id, "gmail:history:work");
    assert_eq!(second_plan.stream_id, "gmail:history:personal");
}

#[test]
fn email_sync_plan_rejects_invalid_imap_config() {
    let cases = [
        (
            "host",
            NewProviderAccount::new(
                "acct_invalid_imap_host",
                CommunicationProviderKind::Imap,
                "Invalid IMAP host",
                "invalid-imap@example.net",
            )
            .config(json!({"host": " ", "port": 993, "tls": true})),
        ),
        (
            "port",
            NewProviderAccount::new(
                "acct_invalid_imap_port",
                CommunicationProviderKind::Imap,
                "Invalid IMAP port",
                "invalid-imap-port@example.net",
            )
            .config(json!({"host": "imap.example.net", "port": 0, "tls": true})),
        ),
        (
            "tls",
            NewProviderAccount::new(
                "acct_invalid_imap_tls",
                CommunicationProviderKind::Imap,
                "Invalid IMAP TLS",
                "invalid-imap-tls@example.net",
            )
            .config(json!({"host": "imap.example.net", "port": 993, "tls": "yes"})),
        ),
        (
            "mailbox",
            NewProviderAccount::new(
                "acct_invalid_imap_mailbox",
                CommunicationProviderKind::Imap,
                "Invalid IMAP mailbox",
                "invalid-imap-mailbox@example.net",
            )
            .config(json!({"host": "imap.example.net", "port": 993, "tls": true, "mailbox": "Inbox\nArchive"})),
        ),
    ];

    for (field_name, account) in cases {
        let account = account.into_test_provider_account();
        let error = plan_email_sync(&account).expect_err("invalid IMAP config must fail");

        assert!(
            matches!(
                error,
                EmailSyncPlanError::InvalidProviderConfig { field, .. } if field == field_name
            ),
            "expected invalid field {field_name}, got {error:?}"
        );
    }
}

#[test]
fn email_sync_plan_rejects_secret_like_account_config() {
    let cases = [
        (
            "oauth_token",
            NewProviderAccount::new(
                "acct_secret_config",
                CommunicationProviderKind::Gmail,
                "Gmail unsafe config",
                "unsafe-config@example.com",
            )
            .config(json!({
                "oauth_token": "must-not-be-here",
                "history_stream_id": "gmail:history"
            })),
        ),
        (
            "adapter.oauth_token",
            NewProviderAccount::new(
                "acct_nested_secret_config",
                CommunicationProviderKind::Gmail,
                "Gmail nested unsafe config",
                "nested-unsafe-config@example.com",
            )
            .config(json!({
                "adapter": {
                    "oauth_token": "must-not-be-here"
                },
                "history_stream_id": "gmail:history"
            })),
        ),
    ];

    for (expected_key, account) in cases {
        let account = account.into_test_provider_account();
        let error = plan_email_sync(&account).expect_err("secret-like config must fail");

        assert!(
            matches!(
                error,
                EmailSyncPlanError::SecretLikeConfigKey { ref key } if key == expected_key
            ),
            "expected secret-like key {expected_key}, got {error:?}"
        );
    }
}

#[test]
fn email_sync_plan_uses_delimiter_safe_imap_stream_id() {
    let account = NewProviderAccount::new(
        "acct_imap_delimiter_mailbox",
        CommunicationProviderKind::Imap,
        "Delimiter mailbox",
        "delimiter-mailbox@example.net",
    )
    .config(json!({
        "host": "imap.example.net",
        "port": 993,
        "tls": true,
        "mailbox": "Projects:2026%Q2"
    }))
    .into_test_provider_account();

    let plan = plan_email_sync(&account).expect("delimiter-safe IMAP plan");

    assert_eq!(plan.stream_id, "imap:Projects%3A2026%25Q2");
    assert_eq!(
        plan.adapter_config,
        EmailSyncAdapterConfig::Imap {
            host: "imap.example.net".to_owned(),
            port: 993,
            tls: true,
            mailboxes: vec!["Projects:2026%Q2".to_owned()],
        }
    );
}

async fn live_sync_context(_test_name: &str) -> Option<(CommunicationIngestionStore, u128)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = CommunicationIngestionStore::new(database.pool().expect("configured pool").clone());

    Some((store, unique_suffix()))
}

trait IntoTestProviderAccount {
    fn into_test_provider_account(self) -> makosh_communications_api::accounts::ProviderAccount;
}

impl IntoTestProviderAccount for NewProviderAccount {
    fn into_test_provider_account(self) -> makosh_communications_api::accounts::ProviderAccount {
        let now = chrono::Utc::now();

        makosh_communications_api::accounts::ProviderAccount {
            account_id: self.account_id,
            provider_kind: self.provider_kind,
            display_name: self.display_name,
            external_account_id: self.external_account_id,
            config: self.config,
            created_at: now,
            updated_at: now,
        }
    }
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
