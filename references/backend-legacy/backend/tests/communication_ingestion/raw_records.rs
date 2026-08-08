use crate::support::*;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use sqlx::Row;

#[tokio::test]
async fn communication_ingestion_records_raw_sources_idempotently_against_postgres() {
    let Some(database) = connect_database("communication raw source test fixture database").await
    else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_raw_{suffix}");
    let provider_record_id = format!("gmail-message-{suffix}");

    store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Gmail raw source test",
            format!("raw-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let first = store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{suffix}"),
                format!("batch_{suffix}"),
                json!({"id": provider_record_id, "provider": "gmail"}),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source": "gmail-api"})),
        )
        .await
        .expect("record raw source");

    let retry = store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                first.raw_record_id.clone(),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{suffix}"),
                format!("batch_{suffix}"),
                json!({"id": provider_record_id, "provider": "gmail"}),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source": "retry"})),
        )
        .await
        .expect("retry raw source");
    assert_eq!(retry.raw_record_id, first.raw_record_id);
    assert_eq!(retry.observation_id, first.observation_id);
    assert_eq!(retry.payload, first.payload);

    let provider_update = store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_duplicate_{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:different-{suffix}"),
                format!("batch_{suffix}"),
                json!({"id": provider_record_id, "provider": "gmail", "changed": true}),
            )
            .provenance(json!({"source": "retry"})),
        )
        .await
        .expect("record provider update snapshot");

    assert_ne!(provider_update.raw_record_id, first.raw_record_id);
    assert_eq!(
        provider_update.raw_record_id,
        format!("raw_duplicate_{suffix}")
    );
    assert_eq!(provider_update.observation_id, first.observation_id);
    assert_eq!(provider_update.payload["changed"], json!(true));

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_raw_records
        WHERE account_id = $1
          AND record_kind = 'email_message'
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("raw record count");
    assert_eq!(count, 2);

    let observation = sqlx::query(
        r#"
        SELECT
            observation.observation_id,
            kind.code AS kind_code,
            observation.source_ref,
            observation.provenance
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&first.observation_id)
    .fetch_one(&pool)
    .await
    .expect("canonical raw record observation");

    let kind_code: String = observation.try_get("kind_code").expect("kind code");
    let source_ref: String = observation.try_get("source_ref").expect("source ref");
    let provenance: serde_json::Value = observation.try_get("provenance").expect("provenance");
    assert_eq!(kind_code, "COMMUNICATION_MESSAGE");
    assert_eq!(
        source_ref,
        format!("communication://{account_id}/email_message/{provider_record_id}")
    );
    assert_eq!(provenance["communication_raw_record"], json!(true));
    assert_eq!(provenance["raw_record_id"], json!(first.raw_record_id));

    let observation_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM observations
        WHERE source_ref = $1
        "#,
    )
    .bind(&source_ref)
    .fetch_one(&pool)
    .await
    .expect("raw record observation count");
    assert_eq!(observation_count, 1);

    let mutation = sqlx::query(
        "UPDATE communication_raw_records SET payload = '{}'::jsonb WHERE raw_record_id = $1",
    )
    .bind(&first.raw_record_id)
    .execute(&pool)
    .await;
    assert!(
        mutation.is_err(),
        "raw provider records must be append-only"
    );
}
