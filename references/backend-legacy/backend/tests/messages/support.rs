#![allow(dead_code)]

use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};

use makosh_communications_api::evidence::{
    NewRawCommunicationRecord, StoredRawCommunicationRecord,
};

use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgPoolOptions};

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;

use makosh_hub_backend::platform::storage::database::Database;

pub async fn live_projection_context(
    _test_name: &str,
) -> Option<(
    TestContext,
    PgPool,
    CommunicationIngestionStore,
    MessageProjectionStore,
)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some((
        test_context,
        pool.clone(),
        CommunicationIngestionStore::new(pool.clone()),
        MessageProjectionStore::new(pool),
    ))
}

pub async fn store_provider_account(
    store: &CommunicationIngestionStore,
    account_id: &str,
    display_name: &str,
    external_account_id: String,
) {
    store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            display_name,
            external_account_id,
        ))
        .await
        .expect("store provider account");
}

pub async fn record_raw_email_message(
    store: &CommunicationIngestionStore,
    account_id: &str,
    raw_record_id: &str,
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> StoredRawCommunicationRecord {
    store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                raw_record_id,
                account_id,
                "email_message",
                provider_record_id,
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "subject": subject,
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": body_text
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record raw message")
}

pub fn disconnected_message_store() -> MessageProjectionStore {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    MessageProjectionStore::new(pool)
}

pub fn stored_raw_record_with_payload(payload: Value) -> StoredRawCommunicationRecord {
    let suffix = unique_suffix();

    StoredRawCommunicationRecord {
        raw_record_id: format!("raw_payload_validation_{suffix}"),
        observation_id: format!("observation:v1:raw-payload-validation-{suffix}"),
        account_id: format!("acct_payload_validation_{suffix}"),
        record_kind: "email_message".to_owned(),
        provider_record_id: format!("provider-payload-validation-{suffix}"),
        source_fingerprint: format!("sha256:payload-validation-{suffix}"),
        import_batch_id: format!("batch_payload_validation_{suffix}"),
        occurred_at: Some(Utc::now()),
        captured_at: Utc::now(),
        payload,
        provenance: json!({"source":"test"}),
    }
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
