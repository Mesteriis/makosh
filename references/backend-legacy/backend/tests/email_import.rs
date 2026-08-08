use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use serde_json::json;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::import::{
    FixtureEmailImportRequest, import_fixture_email_messages,
    import_fixture_email_messages_with_records,
};
use makosh_hub_backend::domains::communications::sources::{
    FixtureCommunicationSourceMessage, parse_fixture_email_messages,
};

use makosh_hub_backend::platform::storage::database::Database;

#[test]
fn fixture_email_source_parses_account_scoped_messages() {
    let input = json!([
        {
            "provider_record_id": "gmail-msg-1",
            "subject": "Budget review",
            "from": "alice@example.com",
            "to": ["bob@example.com"],
            "sent_at": "2026-06-04T10:00:00Z",
            "body_text": "Please review the Q2 budget.",
            "source_fingerprint": "sha256:gmail-msg-1"
        }
    ])
    .to_string();

    let messages = parse_fixture_email_messages(&input).expect("parse fixture messages");

    assert_eq!(
        messages,
        vec![FixtureCommunicationSourceMessage {
            provider_record_id: "gmail-msg-1".to_owned(),
            subject: "Budget review".to_owned(),
            from: "alice@example.com".to_owned(),
            to: vec!["bob@example.com".to_owned()],
            sent_at: Utc.with_ymd_and_hms(2026, 6, 4, 10, 0, 0).single(),
            body_text: "Please review the Q2 budget.".to_owned(),
            source_fingerprint: "sha256:gmail-msg-1".to_owned(),
        }]
    );
}

#[tokio::test]
async fn fixture_email_import_records_raw_messages_idempotently_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_fixture_import_{suffix}");
    let fixture_json = format!(
        r#"[{{"provider_record_id":"fixture-msg-{suffix}","subject":"Fixture import","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:00:00Z","body_text":"Fixture body","source_fingerprint":"sha256:fixture-msg-{suffix}"}}]"#
    );

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Fixture Gmail",
            format!("fixture-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let first = import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(&account_id, format!("batch_{suffix}"), &fixture_json),
    )
    .await
    .expect("first import");
    let second = import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &account_id,
            format!("batch_retry_{suffix}"),
            &fixture_json,
        ),
    )
    .await
    .expect("second import");

    assert_eq!(first.inserted_or_existing_records, 1);
    assert_eq!(second.inserted_or_existing_records, 1);

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM communication_raw_records WHERE account_id = $1 AND provider_record_id = $2",
    )
    .bind(&account_id)
    .bind(format!("fixture-msg-{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("raw record count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn fixture_email_import_records_delimiter_bearing_identities_distinctly_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();

    let same_account_id = format!("acct_fixture_identity_same_{suffix}");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &same_account_id,
            CommunicationProviderKind::Gmail,
            "Fixture identity same account",
            format!("fixture-identity-same-{suffix}@example.com"),
        ))
        .await
        .expect("store same-account provider account");

    let same_account_fixture_json = format!(
        r#"[
            {{"provider_record_id":"thread:{suffix}:message","subject":"Delimiter import A","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:00:00Z","body_text":"Fixture body A","source_fingerprint":"sha256:same-a-{suffix}"}},
            {{"provider_record_id":"thread::{suffix}:message","subject":"Delimiter import B","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:01:00Z","body_text":"Fixture body B","source_fingerprint":"sha256:same-b-{suffix}"}}
        ]"#
    );

    let same_account_report = import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &same_account_id,
            format!("batch_same_identity_{suffix}"),
            same_account_fixture_json,
        ),
    )
    .await
    .expect("same-account delimiter import");
    assert_eq!(same_account_report.inserted_or_existing_records, 2);

    let same_account_raw_record_ids = sqlx::query_scalar::<_, String>(
        r#"
        SELECT raw_record_id
        FROM communication_raw_records
        WHERE account_id = $1
        ORDER BY provider_record_id
        "#,
    )
    .bind(&same_account_id)
    .fetch_all(&pool)
    .await
    .expect("same-account raw record IDs");
    assert_eq!(same_account_raw_record_ids.len(), 2);
    assert_ne!(
        same_account_raw_record_ids[0],
        same_account_raw_record_ids[1]
    );

    let ambiguous_account_id = format!("acct_fixture_identity_{suffix}");
    let ambiguous_left_account_id = format!("{ambiguous_account_id}:left");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &ambiguous_account_id,
            CommunicationProviderKind::Gmail,
            "Fixture identity base account",
            format!("fixture-identity-base-{suffix}@example.com"),
        ))
        .await
        .expect("store base provider account");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &ambiguous_left_account_id,
            CommunicationProviderKind::Gmail,
            "Fixture identity left account",
            format!("fixture-identity-left-{suffix}@example.com"),
        ))
        .await
        .expect("store left provider account");

    let base_fixture_json = format!(
        r#"[{{"provider_record_id":"left:right","subject":"Ambiguous import base","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:02:00Z","body_text":"Fixture body base","source_fingerprint":"sha256:base-{suffix}"}}]"#
    );
    let left_fixture_json = format!(
        r#"[{{"provider_record_id":"right","subject":"Ambiguous import left","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:03:00Z","body_text":"Fixture body left","source_fingerprint":"sha256:left-{suffix}"}}]"#
    );

    import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &ambiguous_account_id,
            format!("batch_base_identity_{suffix}"),
            base_fixture_json,
        ),
    )
    .await
    .expect("base ambiguous import");
    import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &ambiguous_left_account_id,
            format!("batch_left_identity_{suffix}"),
            left_fixture_json,
        ),
    )
    .await
    .expect("left ambiguous import");

    let ambiguous_raw_record_ids = sqlx::query_scalar::<_, String>(
        r#"
        SELECT raw_record_id
        FROM communication_raw_records
        WHERE account_id IN ($1, $2)
        ORDER BY account_id, provider_record_id
        "#,
    )
    .bind(&ambiguous_account_id)
    .bind(&ambiguous_left_account_id)
    .fetch_all(&pool)
    .await
    .expect("ambiguous raw record IDs");
    assert_eq!(ambiguous_raw_record_ids.len(), 2);
    assert_ne!(ambiguous_raw_record_ids[0], ambiguous_raw_record_ids[1]);
}

#[tokio::test]
async fn fixture_email_import_preserves_missing_sent_at_as_null_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_fixture_missing_sent_at_{suffix}");
    let provider_record_id = format!("fixture-missing-sent-at-{suffix}");
    let fixture_json = format!(
        r#"[{{"provider_record_id":"{provider_record_id}","subject":"Missing sent_at import","from":"alice@example.com","to":["bob@example.com"],"body_text":"Fixture body without sent_at","source_fingerprint":"sha256:missing-sent-at-{suffix}"}}]"#
    );

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Fixture missing sent_at",
            format!("fixture-missing-sent-at-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &account_id,
            format!("batch_missing_sent_at_{suffix}"),
            fixture_json,
        ),
    )
    .await
    .expect("missing sent_at import");

    let occurred_at = sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
        r#"
        SELECT occurred_at
        FROM communication_raw_records
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("raw record occurred_at");
    assert!(occurred_at.is_none());
}

#[tokio::test]
async fn fixture_email_import_returns_raw_records_for_projection_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_fixture_records_{suffix}");
    let provider_record_id = format!("fixture-records-{suffix}");
    let fixture_json = format!(
        r#"[{{"provider_record_id":"{provider_record_id}","subject":"Record import","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:00:00Z","body_text":"Fixture body","source_fingerprint":"sha256:records-{suffix}"}}]"#
    );

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Fixture records",
            format!("fixture-records-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let report = import_fixture_email_messages_with_records(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &account_id,
            format!("batch_records_{suffix}"),
            fixture_json,
        ),
    )
    .await
    .expect("fixture import with records");

    assert_eq!(report.inserted_or_existing_records, 1);
    assert_eq!(report.raw_records.len(), 1);
    assert_eq!(report.raw_records[0].account_id, account_id);
    assert_eq!(report.raw_records[0].provider_record_id, provider_record_id);
    assert_eq!(report.raw_records[0].payload["subject"], "Record import");
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
