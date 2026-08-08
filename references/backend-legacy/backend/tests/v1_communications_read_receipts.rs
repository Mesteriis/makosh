use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::outbox::delivery::OutboxSendReceipt;
use makosh_hub_backend::domains::communications::outbox::{
    CommunicationOutboxStatus, CommunicationOutboxStore, NewCommunicationOutboxItem,
};

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const T: &str = "v1comms-read-receipt-test-token";

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

#[tokio::test]
async fn v1_read_receipt_records_correlation_and_realtime_event_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-read-receipt-{suffix}");
    let provider_message_id = format!("provider-message-read-receipt-{suffix}");
    let outbox_id = seed_sent_outbox_item(pool.clone(), &account_id, &provider_message_id).await;
    let read_at = Utc::now();

    let r = router(&context.connection_string()).await;
    let response = r
        .oneshot(post(
            "/api/v1/communications/read-receipts",
            json!({
                "account_id": account_id,
                "provider_message_id": provider_message_id,
                "recipient": "reader@example.com",
                "read_at": read_at,
                "source_kind": "mdn",
                "provider_record_id": format!("mdn-{suffix}"),
                "metadata": {
                    "user_agent": "fixture-mdn"
                }
            }),
        ))
        .await
        .expect("read receipt response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["outbox_id"], outbox_id);
    assert_eq!(body["provider_message_id"], provider_message_id);
    assert_eq!(body["recipient"], "reader@example.com");
    assert_eq!(body["receipt_kind"], "read");
    assert_eq!(body["source_kind"], "mdn");

    let persisted = sqlx::query(
        r#"
        SELECT account_id, outbox_id, provider_message_id, recipient, receipt_kind, metadata
        FROM communication_read_receipts
        WHERE provider_record_id = $1
        "#,
    )
    .bind(format!("mdn-{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("persisted read receipt");
    assert_eq!(
        persisted.try_get::<String, _>("account_id").unwrap(),
        account_id
    );
    assert_eq!(
        persisted.try_get::<Option<String>, _>("outbox_id").unwrap(),
        Some(outbox_id.clone())
    );
    assert_eq!(
        persisted
            .try_get::<String, _>("provider_message_id")
            .unwrap(),
        provider_message_id
    );
    assert_eq!(
        persisted.try_get::<String, _>("recipient").unwrap(),
        "reader@example.com"
    );
    assert_eq!(
        persisted.try_get::<String, _>("receipt_kind").unwrap(),
        "read"
    );
    assert_eq!(
        persisted.try_get::<Value, _>("metadata").unwrap(),
        json!({"user_agent": "fixture-mdn"})
    );

    let event = sqlx::query(
        r#"
        SELECT subject, payload
        FROM event_log
        WHERE event_type = 'mail.read_receipt.recorded'
          AND subject->>'kind' = 'mail_read_receipt'
          AND subject->>'outbox_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("read receipt event");
    let subject = event.try_get::<Value, _>("subject").unwrap();
    let payload = event.try_get::<Value, _>("payload").unwrap();
    assert_eq!(subject["outbox_id"], outbox_id);
    assert_eq!(payload["account_id"], account_id);
    assert_eq!(payload["provider_message_id"], provider_message_id);
    assert_eq!(payload["receipt_kind"], "read");
    assert!(payload.get("recipient").is_none());
    assert!(payload.get("body_text").is_none());
    let receipt_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'read_receipt'
           AND relationship_kind = 'read_receipt_recorded'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("read receipt observation link");
    let receipt_observation_id: String = receipt_link
        .try_get("observation_id")
        .expect("read receipt observation id");
    let receipt_metadata: Value = receipt_link.try_get("metadata").expect("receipt metadata");
    assert_eq!(receipt_metadata["receipt_kind"], "read");
    let receipt_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&receipt_observation_id)
    .fetch_one(&pool)
    .await
    .expect("read receipt observation");
    let receipt_origin_kind: String = receipt_observation
        .try_get("origin_kind")
        .expect("receipt origin kind");
    let receipt_payload: Value = receipt_observation
        .try_get("payload")
        .expect("receipt payload");
    assert_eq!(receipt_origin_kind, "local_runtime");
    assert_eq!(receipt_payload["operation"], "read_receipt_recorded");
}

#[tokio::test]
async fn v1_outbox_list_includes_latest_read_receipt_summary_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-outbox-read-summary-{suffix}");
    let provider_message_id = format!("provider-message-outbox-read-summary-{suffix}");
    let outbox_id = seed_sent_outbox_item(pool.clone(), &account_id, &provider_message_id).await;
    let read_at = Utc::now();

    let r = router(&context.connection_string()).await;
    let receipt_response = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/read-receipts",
            json!({
                "account_id": account_id,
                "provider_message_id": provider_message_id,
                "recipient": "reader@example.com",
                "read_at": read_at,
                "source_kind": "mdn",
                "provider_record_id": format!("mdn-outbox-summary-{suffix}")
            }),
        ))
        .await
        .expect("read receipt response");
    assert_eq!(receipt_response.status(), StatusCode::OK);

    let list_response = r
        .oneshot(get(&format!(
            "/api/v1/communications/outbox?account_id={account_id}&status=sent"
        )))
        .await
        .expect("outbox list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let body = response_json(list_response).await;
    let items = body["items"].as_array().expect("outbox items");
    let item = items
        .iter()
        .find(|candidate| candidate["outbox_id"] == outbox_id)
        .expect("sent outbox item in response");

    let latest_read_receipt = &item["metadata"]["latest_read_receipt"];
    assert_eq!(latest_read_receipt["receipt_kind"], "read");
    assert_eq!(latest_read_receipt["source_kind"], "mdn");
    let listed_read_at = latest_read_receipt["read_at"]
        .as_str()
        .expect("latest read receipt read_at");
    assert_eq!(
        DateTime::parse_from_rfc3339(listed_read_at)
            .expect("parse listed read_at")
            .with_timezone(&Utc),
        read_at
    );
    assert!(latest_read_receipt.get("recipient").is_none());
    assert!(latest_read_receipt.get("provider_record_id").is_none());
}

#[tokio::test]
async fn v1_provider_delivery_event_records_delivery_status_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-provider-delivery-{suffix}");
    let provider_message_id = format!("provider-message-provider-delivery-{suffix}");
    let outbox_id = seed_sent_outbox_item(pool.clone(), &account_id, &provider_message_id).await;
    let occurred_at = Utc::now();

    let r = router(&context.connection_string()).await;
    let response = r
        .oneshot(post(
            "/api/v1/integrations/mail/provider-delivery-events",
            json!({
                "account_id": account_id,
                "provider_message_id": provider_message_id,
                "event_kind": "delivered",
                "occurred_at": occurred_at,
                "source_kind": "gmail_history",
                "provider_record_id": format!("gmail-history-delivered-{suffix}")
            }),
        ))
        .await
        .expect("provider delivery event response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["notification_kind"], "delivery_status");
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["outbox_id"], outbox_id);
    assert_eq!(body["provider_message_id"], provider_message_id);
    assert_eq!(body["delivery_status"], "delivered");
    assert_eq!(body["source_kind"], "gmail_history");

    let metadata: Value =
        sqlx::query_scalar("SELECT metadata FROM communication_outbox WHERE outbox_id = $1")
            .bind(&outbox_id)
            .fetch_one(&pool)
            .await
            .expect("outbox metadata");
    assert_eq!(metadata["delivery_status"]["delivery_status"], "delivered");
    assert_eq!(metadata["delivery_status"]["source_kind"], "gmail_history");
    let delivery_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'delivery_status_observed'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("delivery status observation link");
    let delivery_observation_id: String = delivery_link
        .try_get("observation_id")
        .expect("delivery observation id");
    let delivery_metadata: Value = delivery_link
        .try_get("metadata")
        .expect("delivery metadata");
    assert_eq!(delivery_metadata["delivery_status"], "delivered");
    let delivery_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&delivery_observation_id)
    .fetch_one(&pool)
    .await
    .expect("delivery observation");
    let delivery_origin_kind: String = delivery_observation
        .try_get("origin_kind")
        .expect("delivery origin kind");
    let delivery_payload: Value = delivery_observation
        .try_get("payload")
        .expect("delivery payload");
    assert_eq!(delivery_origin_kind, "local_runtime");
    assert_eq!(delivery_payload["operation"], "delivery_status_recorded");

    let raw_signal_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.mail.delivery_status.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("raw mail delivery signal count");
    assert_eq!(raw_signal_count, 1);

    let accepted_signal_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.mail.delivery_status'",
    )
    .fetch_one(&pool)
    .await
    .expect("accepted mail delivery signal count");
    assert_eq!(accepted_signal_count, 1);
}

#[tokio::test]
async fn v1_provider_delivery_event_records_read_receipt_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-provider-read-{suffix}");
    let provider_message_id = format!("provider-message-provider-read-{suffix}");
    let outbox_id = seed_sent_outbox_item(pool.clone(), &account_id, &provider_message_id).await;
    let occurred_at = Utc::now();

    let r = router(&context.connection_string()).await;
    let response = r
        .oneshot(post(
            "/api/v1/integrations/mail/provider-delivery-events",
            json!({
                "account_id": account_id,
                "provider_message_id": provider_message_id,
                "event_kind": "read",
                "recipient": "reader@example.com",
                "occurred_at": occurred_at,
                "source_kind": "gmail_history",
                "provider_record_id": format!("gmail-history-read-{suffix}")
            }),
        ))
        .await
        .expect("provider read event response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["notification_kind"], "read_receipt");
    assert_eq!(body["read_receipt"]["account_id"], account_id);
    assert_eq!(body["read_receipt"]["outbox_id"], outbox_id);
    assert_eq!(
        body["read_receipt"]["provider_message_id"],
        provider_message_id
    );
    assert_eq!(body["read_receipt"]["source_kind"], "gmail_history");
    assert_eq!(
        body["read_receipt"]["metadata"]["provider_event_kind"],
        "read"
    );
    let provider_message_link_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'provider_message'
           AND entity_id = $1
           AND relationship_kind = 'read_receipt_observed'",
    )
    .bind(&provider_message_id)
    .fetch_one(&pool)
    .await
    .expect("provider message read receipt links");
    assert!(provider_message_link_count >= 1);

    let raw_signal_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.mail.read_receipt.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("raw mail read receipt signal count");
    assert_eq!(raw_signal_count, 1);

    let accepted_signal_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.mail.read_receipt'",
    )
    .fetch_one(&pool)
    .await
    .expect("accepted mail read receipt signal count");
    assert_eq!(accepted_signal_count, 1);
}

#[tokio::test]
async fn v1_delivery_notification_parses_dsn_and_appends_delivery_status_event_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-dsn-{suffix}");
    let provider_message_id = format!("provider-message-dsn-{suffix}");
    let outbox_id = seed_sent_outbox_item(pool.clone(), &account_id, &provider_message_id).await;
    let received_at = Utc::now();

    let r = router(&context.connection_string()).await;
    let response = r
        .oneshot(post(
            "/api/v1/communications/delivery-notifications",
            json!({
                "account_id": account_id,
                "provider_record_id": format!("dsn-{suffix}"),
                "received_at": received_at,
                "raw_message": format!(
                    concat!(
                        "Content-Type: multipart/report; report-type=delivery-status\r\n",
                        "\r\n",
                        "Original-Message-ID: <{}>\r\n",
                        "Final-Recipient: rfc822; bounced@example.com\r\n",
                        "Action: failed\r\n",
                        "Status: 5.1.1\r\n",
                        "Diagnostic-Code: smtp; 550 mailbox unavailable\r\n"
                    ),
                    provider_message_id
                )
            }),
        ))
        .await
        .expect("delivery notification response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["notification_kind"], "delivery_status");
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["outbox_id"], outbox_id);
    assert_eq!(body["provider_message_id"], provider_message_id);
    assert_eq!(body["delivery_status"], "failed");
    assert_eq!(body["smtp_status"], "5.1.1");
    assert_eq!(body["source_kind"], "dsn");

    let outbox_metadata: Value =
        sqlx::query_scalar("SELECT metadata FROM communication_outbox WHERE outbox_id = $1")
            .bind(&outbox_id)
            .fetch_one(&pool)
            .await
            .expect("outbox metadata");
    assert_eq!(
        outbox_metadata["delivery_status"],
        json!({
            "delivery_status": "failed",
            "smtp_status": "5.1.1",
            "source_kind": "dsn",
            "provider_record_id": format!("dsn-{suffix}"),
            "recorded_at": received_at
        })
    );

    let event = sqlx::query(
        r#"
        SELECT subject, payload
        FROM event_log
        WHERE event_type = 'mail.outbox.delivery_status_changed'
          AND subject->>'id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("delivery status event");
    let subject = event.try_get::<Value, _>("subject").unwrap();
    let payload = event.try_get::<Value, _>("payload").unwrap();
    assert_eq!(subject["outbox_id"], outbox_id);
    assert_eq!(payload["delivery_status"], "failed");
    assert_eq!(payload["smtp_status"], "5.1.1");
    assert!(payload.get("recipient").is_none());
    assert!(payload.get("diagnostic_code").is_none());
    assert!(payload.get("raw_message").is_none());
    let failed_delivery_link = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'provider_message'
           AND entity_id = $1
           AND relationship_kind = 'delivery_status_observed'",
    )
    .bind(&provider_message_id)
    .fetch_one(&pool)
    .await
    .expect("provider message delivery status links");
    assert!(failed_delivery_link >= 1);

    let raw_signal_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.mail.delivery_status.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("dsn raw mail delivery signal count");
    assert_eq!(raw_signal_count, 1);

    let accepted_signal_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.mail.delivery_status'",
    )
    .fetch_one(&pool)
    .await
    .expect("dsn accepted mail delivery signal count");
    assert_eq!(accepted_signal_count, 1);
}

#[tokio::test]
async fn v1_delivery_notification_parses_mdn_into_read_receipt_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-mdn-parser-{suffix}");
    let provider_message_id = format!("provider-message-mdn-parser-{suffix}");
    let outbox_id = seed_sent_outbox_item(pool.clone(), &account_id, &provider_message_id).await;
    let received_at = Utc::now();

    let r = router(&context.connection_string()).await;
    let response = r
        .oneshot(post(
            "/api/v1/communications/delivery-notifications",
            json!({
                "account_id": account_id,
                "provider_record_id": format!("mdn-parser-{suffix}"),
                "received_at": received_at,
                "raw_message": format!(
                    concat!(
                        "Content-Type: multipart/report; report-type=disposition-notification\r\n",
                        "\r\n",
                        "Original-Message-ID: <{}>\r\n",
                        "Final-Recipient: rfc822; reader@example.com\r\n",
                        "Disposition: automatic-action/MDN-sent-automatically; displayed\r\n",
                        "Reporting-UA: fixture-mdn\r\n"
                    ),
                    provider_message_id
                )
            }),
        ))
        .await
        .expect("delivery notification response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["notification_kind"], "read_receipt");
    assert_eq!(body["read_receipt"]["account_id"], account_id);
    assert_eq!(body["read_receipt"]["outbox_id"], outbox_id);
    assert_eq!(
        body["read_receipt"]["provider_message_id"],
        provider_message_id
    );
    assert_eq!(body["read_receipt"]["recipient"], "reader@example.com");
    assert_eq!(body["read_receipt"]["receipt_kind"], "read");
    assert_eq!(body["read_receipt"]["source_kind"], "mdn");

    let read_receipt_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM communication_read_receipts WHERE account_id = $1 AND provider_record_id = $2",
    )
    .bind(&account_id)
    .bind(format!("mdn-parser-{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("read receipt count");
    assert_eq!(read_receipt_count, 1);
    let mdn_receipt_links = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'read_receipt_observed'",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("mdn outbox read receipt links");
    assert!(mdn_receipt_links >= 1);

    let raw_signal_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.mail.read_receipt.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("mdn raw mail read receipt signal count");
    assert_eq!(raw_signal_count, 1);

    let accepted_signal_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.mail.read_receipt'",
    )
    .fetch_one(&pool)
    .await
    .expect("mdn accepted mail read receipt signal count");
    assert_eq!(accepted_signal_count, 1);
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

async fn seed_sent_outbox_item(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_message_id: &str,
) -> String {
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Seed Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");

    let outbox_id = format!("outbox-read-receipt-{}", uid());
    let store = CommunicationOutboxStore::new(pool);
    store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: outbox_id.clone(),
            account_id: account_id.to_owned(),
            draft_id: None,
            to_recipients: vec!["reader@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Read receipt seed".to_owned(),
            body_text: "Private body not for receipt events".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Queued,
            scheduled_send_at: None,
            undo_deadline_at: None,
            metadata: json!({}),
        })
        .await
        .expect("enqueue outbox item");
    let claimed = store.claim_due(Utc::now(), 10).await.expect("claim due");
    assert_eq!(claimed.len(), 1);
    store
        .mark_sent(
            &outbox_id,
            Utc::now(),
            &OutboxSendReceipt {
                provider_message_id: provider_message_id.to_owned(),
                accepted_recipients: vec!["reader@example.com".to_owned()],
            },
        )
        .await
        .expect("mark sent")
        .expect("sent outbox item");

    outbox_id
}
