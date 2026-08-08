use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use sqlx::Row;

use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::email_fixture_pipeline::{
    EmailFixturePipelineRequest, project_fixture_email_messages,
};

#[tokio::test]
async fn fixture_email_pipeline_imports_projects_persons_and_graph_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("acct_fixture_pipeline_{suffix}");
    let fixture_json = json!([
        {
            "provider_record_id": format!("fixture-pipeline-{suffix}"),
            "subject": "Pipeline import",
            "from": "sender@example.invalid",
            "to": ["recipient@example.invalid"],
            "sent_at": "2026-06-04T10:00:00Z",
            "body_text": "Pipeline body",
            "source_fingerprint": format!("sha256:pipeline-{suffix}")
        }
    ])
    .to_string();

    let provider_accounts =
        makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
            pool.clone(),
        );
    let communication_evidence = CommunicationIngestionStore::new(pool.clone());
    let report = project_fixture_email_messages(
        pool,
        &provider_accounts,
        &communication_evidence,
        &EmailFixturePipelineRequest::new(
            &account_id,
            "iCloud fixture pipeline",
            "redacted@example.invalid",
            CommunicationProviderKind::Icloud,
            format!("batch_pipeline_{suffix}"),
            fixture_json,
        ),
    )
    .await
    .expect("project fixture pipeline");

    assert_eq!(report.imported_records, 1);
    assert_eq!(report.projected_messages, 1);
    assert_eq!(report.upserted_personas, 2);
    assert!(!report.graph_summary.is_empty);
    assert!(report.total_graph_nodes >= 4);
    assert!(report.total_graph_edges >= 3);

    let accepted_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'signal.accepted.mail.message'
          AND source ->> 'account_id' = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(test_context.pool())
    .await
    .expect("accepted mail signal count");
    assert_eq!(accepted_signal_count, 1);

    let observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM communication_messages
        WHERE account_id = $1
        ORDER BY projected_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(test_context.pool())
    .await
    .expect("message observation id");
    let trace_rows = sqlx::query(
        r#"
        SELECT event_id, event_type, causation_id, correlation_id
        FROM event_log
        WHERE correlation_id = $1
          AND event_type IN (
              'observation.captured.v1',
              'signal.raw.mail.message.observed',
              'signal.accepted.mail.message',
              'communication.message.recorded'
          )
        ORDER BY position ASC
        "#,
    )
    .bind(&observation_id)
    .fetch_all(test_context.pool())
    .await
    .expect("mail trace rows");
    assert_eq!(trace_rows.len(), 4);
    let observation_event_id = format!("event:v1:observation-captured:{observation_id}");
    let raw_event_id: String = trace_rows[1].try_get("event_id").expect("raw event id");
    let accepted_event_id: String = trace_rows[2]
        .try_get("event_id")
        .expect("accepted event id");
    assert_eq!(
        trace_rows[0]
            .try_get::<String, _>("event_id")
            .expect("observation event id"),
        observation_event_id
    );
    assert_eq!(
        trace_rows[1]
            .try_get::<Option<String>, _>("causation_id")
            .expect("raw causation")
            .as_deref(),
        Some(observation_event_id.as_str())
    );
    assert_eq!(
        trace_rows[2]
            .try_get::<Option<String>, _>("causation_id")
            .expect("accepted causation")
            .as_deref(),
        Some(raw_event_id.as_str())
    );
    assert_eq!(
        trace_rows[3]
            .try_get::<Option<String>, _>("causation_id")
            .expect("communication causation")
            .as_deref(),
        Some(accepted_event_id.as_str())
    );
    assert!(trace_rows.iter().all(|row| {
        row.try_get::<Option<String>, _>("correlation_id")
            .expect("trace correlation")
            .as_deref()
            == Some(observation_id.as_str())
    }));
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
