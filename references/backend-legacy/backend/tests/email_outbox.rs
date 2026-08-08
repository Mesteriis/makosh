use chrono::{Duration, Utc};
use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use makosh_communications_api::email::{EmailSendError, OutgoingEmail, SendResult, SmtpConfig};
use serde_json::json;
use sqlx::Row;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::outbox::delivery::{
    EmailOutboxDeliveryWorker, OutboxDeliveryError, OutboxEmailSender, OutboxRetryPolicy,
    OutboxSendReceipt,
};
use makosh_hub_backend::domains::communications::outbox::smtp_sender::SmtpOutboxEmailSender;
use makosh_hub_backend::domains::communications::outbox::{
    CommunicationOutboxItem, CommunicationOutboxStatus, CommunicationOutboxStore,
    NewCommunicationOutboxItem,
};
use makosh_hub_backend::platform::communications::SmtpTransport;

use makosh_hub_backend::platform::secrets::models::{
    NewSecretReference, ResolvedSecret, SecretKind, SecretStoreKind,
};
use makosh_hub_backend::platform::secrets::resolver::InMemorySecretResolver;
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn outbox_claim_due_waits_for_schedule_and_undo_deadline_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-store-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Outbox Store IMAP",
            format!("outbox-store-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let store = CommunicationOutboxStore::new(pool);
    let now = Utc::now();
    let outbox_id = format!("outbox-store-{suffix}");
    store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: outbox_id.clone(),
            account_id,
            draft_id: None,
            to_recipients: vec!["recipient@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Claim after undo".to_owned(),
            body_text: "Do not send before undo window closes.".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Queued,
            scheduled_send_at: Some(now - Duration::minutes(1)),
            undo_deadline_at: Some(now + Duration::seconds(30)),
            metadata: json!({ "source": "test" }),
        })
        .await
        .expect("enqueue outbox item");

    let premature = store.claim_due(now, 10).await.expect("premature claim");
    assert!(premature.is_empty());

    let claimed = store
        .claim_due(now + Duration::seconds(31), 10)
        .await
        .expect("claim after undo window");
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].outbox_id, outbox_id);
    assert_eq!(claimed[0].status, CommunicationOutboxStatus::Sending);
    assert_eq!(claimed[0].send_attempts, 1);
    assert!(claimed[0].claimed_at.is_some());
}

#[tokio::test]
async fn outbox_delivery_worker_marks_sent_and_appends_event_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-delivery-{suffix}");
    seed_provider_account(pool.clone(), &account_id, suffix).await;
    let store = CommunicationOutboxStore::new(pool.clone());
    let now = Utc::now();
    let outbox_id = format!("outbox-delivery-{suffix}");
    enqueue_due_item(&store, &account_id, &outbox_id, now).await;

    let worker = EmailOutboxDeliveryWorker::new(
        store.clone(),
        StaticSuccessSender {
            provider_message_id: "provider-message-1".to_owned(),
            accepted_recipients: vec!["recipient@example.com".to_owned()],
        },
    );
    let report = worker
        .deliver_due(now + Duration::seconds(1), 10)
        .await
        .expect("deliver due outbox");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.sent, 1);
    assert_eq!(report.failed, 0);
    assert_eq!(report.retried, 0);
    let sent_items = store
        .list(Some(&account_id), Some(CommunicationOutboxStatus::Sent), 10)
        .await
        .expect("list sent items");
    assert_eq!(sent_items.len(), 1);
    assert_eq!(sent_items[0].outbox_id, outbox_id);
    assert_eq!(
        sent_items[0].provider_message_id.as_deref(),
        Some("provider-message-1")
    );
    assert!(sent_items[0].sent_at.is_some());
    assert!(sent_items[0].last_error.is_none());

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'mail.outbox.sent' AND subject->>'id' = $1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("sent event count");
    assert_eq!(event_count, 1);
    let sent_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("sent observation link");
    let sent_metadata: serde_json::Value = sent_link.try_get("metadata").expect("sent metadata");
    assert_eq!(sent_metadata["status"], "sent");
    let sent_observation_id: String = sent_link
        .try_get("observation_id")
        .expect("sent observation id");
    let sent_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&sent_observation_id)
    .fetch_one(&pool)
    .await
    .expect("sent observation");
    let sent_origin_kind: String = sent_observation
        .try_get("origin_kind")
        .expect("sent origin kind");
    let sent_payload: serde_json::Value =
        sent_observation.try_get("payload").expect("sent payload");
    assert_eq!(sent_origin_kind, "local_runtime");
    assert_eq!(sent_payload["operation"], "outbox_mark_sent");
}

#[tokio::test]
async fn outbox_delivery_worker_marks_failed_and_appends_event_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-failure-{suffix}");
    seed_provider_account(pool.clone(), &account_id, suffix).await;
    let store = CommunicationOutboxStore::new(pool.clone());
    let now = Utc::now();
    let outbox_id = format!("outbox-failure-{suffix}");
    enqueue_due_item(&store, &account_id, &outbox_id, now).await;

    let worker = EmailOutboxDeliveryWorker::with_retry_policy(
        store.clone(),
        StaticFailureSender {
            message: "SMTP unavailable".to_owned(),
        },
        OutboxRetryPolicy::disabled(),
    );
    let report = worker
        .deliver_due(now + Duration::seconds(1), 10)
        .await
        .expect("deliver due outbox");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.sent, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.retried, 0);
    let failed_items = store
        .list(
            Some(&account_id),
            Some(CommunicationOutboxStatus::Failed),
            10,
        )
        .await
        .expect("list failed items");
    assert_eq!(failed_items.len(), 1);
    assert_eq!(failed_items[0].outbox_id, outbox_id);
    assert_eq!(
        failed_items[0].last_error.as_deref(),
        Some("SMTP unavailable")
    );
    assert!(failed_items[0].provider_message_id.is_none());
    assert!(failed_items[0].sent_at.is_none());

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'mail.outbox.failed' AND subject->>'id' = $1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("failed event count");
    assert_eq!(event_count, 1);
    let failed_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("failed observation link");
    let failed_metadata: serde_json::Value =
        failed_link.try_get("metadata").expect("failed metadata");
    assert_eq!(failed_metadata["status"], "failed");
}

#[tokio::test]
async fn outbox_delivery_worker_schedules_retry_with_backoff_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-retry-{suffix}");
    seed_provider_account(pool.clone(), &account_id, suffix).await;
    let store = CommunicationOutboxStore::new(pool.clone());
    let now = Utc::now();
    let delivery_started_at = now + Duration::seconds(1);
    let outbox_id = format!("outbox-retry-{suffix}");
    enqueue_due_item(&store, &account_id, &outbox_id, now).await;

    let worker = EmailOutboxDeliveryWorker::with_retry_policy(
        store.clone(),
        StaticFailureSender {
            message: "SMTP unavailable".to_owned(),
        },
        OutboxRetryPolicy::new(3, Duration::seconds(60), Duration::minutes(10)),
    );
    let report = worker
        .deliver_due(delivery_started_at, 10)
        .await
        .expect("deliver due outbox");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.sent, 0);
    assert_eq!(report.failed, 0);
    assert_eq!(report.retried, 1);
    let retry_items = store
        .list(
            Some(&account_id),
            Some(CommunicationOutboxStatus::Scheduled),
            10,
        )
        .await
        .expect("list retry items");
    assert_eq!(retry_items.len(), 1);
    assert_eq!(retry_items[0].outbox_id, outbox_id);
    assert_eq!(retry_items[0].send_attempts, 1);
    assert_eq!(
        retry_items[0].scheduled_send_at,
        Some(delivery_started_at + Duration::seconds(60))
    );
    assert_eq!(
        retry_items[0].last_error.as_deref(),
        Some("SMTP unavailable")
    );
    assert!(retry_items[0].provider_message_id.is_none());
    assert!(retry_items[0].sent_at.is_none());

    let premature_claim = store
        .claim_due(delivery_started_at + Duration::seconds(59), 10)
        .await
        .expect("premature retry claim");
    assert!(premature_claim.is_empty());
    let due_retry = store
        .claim_due(delivery_started_at + Duration::seconds(60), 10)
        .await
        .expect("due retry claim");
    assert_eq!(due_retry.len(), 1);
    assert_eq!(due_retry[0].send_attempts, 2);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'mail.outbox.retry_scheduled' AND subject->>'id' = $1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("retry scheduled event count");
    assert_eq!(event_count, 1);
    let retry_links = sqlx::query(
        "SELECT metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at ASC",
    )
    .bind(&outbox_id)
    .fetch_all(&pool)
    .await
    .expect("retry observation links");
    let retry_statuses: Vec<String> = retry_links
        .iter()
        .map(|row| {
            row.try_get::<serde_json::Value, _>("metadata")
                .expect("retry metadata")["status"]
                .as_str()
                .expect("retry status")
                .to_owned()
        })
        .collect();
    assert!(retry_statuses.iter().any(|status| status == "scheduled"));
    assert!(retry_statuses.iter().any(|status| status == "sending"));
    let claim_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("outbox transition count");
    assert!(claim_count >= 3);
}

#[tokio::test]
async fn outbox_list_page_uses_cursor_pagination_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-page-{suffix}");
    seed_provider_account(pool.clone(), &account_id, suffix).await;
    let store = CommunicationOutboxStore::new(pool);
    let now = Utc::now();
    let older_id = format!("outbox-page-older-{suffix}");
    let newer_id = format!("outbox-page-newer-{suffix}");

    enqueue_due_item(&store, &account_id, &older_id, now - Duration::minutes(5)).await;
    enqueue_due_item(&store, &account_id, &newer_id, now).await;

    let first_page = store
        .list_page(Some(&account_id), None, None, 1)
        .await
        .expect("first outbox page");
    assert_eq!(first_page.items.len(), 1);
    assert_eq!(first_page.items[0].outbox_id, newer_id);
    assert!(first_page.has_more);
    assert!(first_page.next_cursor.is_some());

    let second_page = store
        .list_page(
            Some(&account_id),
            None,
            first_page.next_cursor.as_deref(),
            1,
        )
        .await
        .expect("second outbox page");
    assert_eq!(second_page.items.len(), 1);
    assert_eq!(second_page.items[0].outbox_id, older_id);
    assert!(!second_page.has_more);
    assert!(second_page.next_cursor.is_none());
}

#[tokio::test]
async fn smtp_outbox_sender_resolves_account_scoped_smtp_credentials_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-smtp-{suffix}");
    let secret_ref = format!("secret:outbox-smtp:{suffix}");
    seed_smtp_provider_account(pool.clone(), &account_id, &secret_ref, suffix).await;
    let mut resolver = InMemorySecretResolver::new();
    resolver
        .insert(&secret_ref, "smtp-test-password")
        .expect("insert smtp test credential");
    let transport = RecordingSmtpTransport::default();
    let sender = SmtpOutboxEmailSender::new(pool.clone(), resolver, transport.clone());
    let store = CommunicationOutboxStore::new(pool);
    let now = Utc::now();
    let outbox_id = format!("outbox-smtp-{suffix}");
    let item = enqueue_due_item(&store, &account_id, &outbox_id, now).await;

    let receipt = sender.send(&item).await.expect("send outbox item via SMTP");

    assert_eq!(receipt.provider_message_id, "smtp-message-1");
    assert_eq!(receipt.accepted_recipients, vec!["recipient@example.com"]);
    let calls = transport.calls.lock().expect("smtp transport calls");
    assert_eq!(calls.len(), 1);
    let call = &calls[0];
    assert_eq!(call.config.host, "smtp.example.com");
    assert_eq!(call.config.port, 2525);
    assert_eq!(call.config.username, "smtp-login@example.com");
    assert!(!call.config.tls);
    assert!(call.config.starttls);
    assert_eq!(call.password, "smtp-test-password");
    assert_eq!(
        call.email.from,
        format!("outbox-delivery-{suffix}@example.com")
    );
    assert_eq!(call.email.to, vec!["recipient@example.com"]);
    assert_eq!(call.email.subject, "Delivery worker");
    assert_eq!(call.email.body_text, "Deliver this due outbox item.");
}

#[tokio::test]
async fn smtp_outbox_sender_uses_icloud_defaults_and_imap_credential_when_smtp_binding_missing_against_postgres()
 {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-icloud-{suffix}");
    let secret_ref = format!("secret:outbox-icloud:{suffix}");
    seed_icloud_provider_account_with_imap_secret(pool.clone(), &account_id, &secret_ref, suffix)
        .await;
    let mut resolver = InMemorySecretResolver::new();
    resolver
        .insert(&secret_ref, "icloud-app-password")
        .expect("insert icloud test credential");
    let transport = RecordingSmtpTransport::default();
    let sender = SmtpOutboxEmailSender::new(pool.clone(), resolver, transport.clone());
    let store = CommunicationOutboxStore::new(pool);
    let outbox_id = format!("outbox-icloud-{suffix}");
    let item = enqueue_due_item(&store, &account_id, &outbox_id, Utc::now()).await;

    let receipt = sender
        .send(&item)
        .await
        .expect("send outbox item via iCloud SMTP");

    assert_eq!(receipt.provider_message_id, "smtp-message-1");
    assert_eq!(receipt.accepted_recipients, vec!["recipient@example.com"]);
    let calls = transport.calls.lock().expect("smtp transport calls");
    assert_eq!(calls.len(), 1);
    let call = &calls[0];
    assert_eq!(call.config.host, "smtp.mail.me.com");
    assert_eq!(call.config.port, 587);
    assert_eq!(
        call.config.username,
        format!("icloud-delivery-{suffix}@example.com")
    );
    assert!(call.config.tls);
    assert!(call.config.starttls);
    assert_eq!(call.password, "icloud-app-password");
}

#[tokio::test]
async fn smtp_outbox_sender_rejects_missing_smtp_config_without_transport_call() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = Utc::now()
        .timestamp_nanos_opt()
        .expect("current timestamp nanos");
    let account_id = format!("acct-outbox-smtp-missing-{suffix}");
    seed_provider_account(pool.clone(), &account_id, suffix).await;
    let transport = RecordingSmtpTransport::default();
    let sender = SmtpOutboxEmailSender::new(
        pool.clone(),
        InMemorySecretResolver::new(),
        transport.clone(),
    );
    let store = CommunicationOutboxStore::new(pool);
    let outbox_id = format!("outbox-smtp-missing-{suffix}");
    let item = enqueue_due_item(&store, &account_id, &outbox_id, Utc::now()).await;

    let error = sender
        .send(&item)
        .await
        .expect_err("missing SMTP config should fail before transport");

    assert_eq!(
        error.public_message(),
        "SMTP config is unavailable for this account"
    );
    assert!(
        transport
            .calls
            .lock()
            .expect("smtp transport calls")
            .is_empty()
    );
}

async fn seed_provider_account(pool: sqlx::PgPool, account_id: &str, suffix: i64) {
    CommunicationIngestionStore::new(pool)
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Imap,
            "Outbox Delivery IMAP",
            format!("outbox-delivery-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");
}

async fn seed_smtp_provider_account(
    pool: sqlx::PgPool,
    account_id: &str,
    secret_ref: &str,
    suffix: i64,
) {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::Imap,
                "Outbox SMTP IMAP",
                format!("outbox-delivery-{suffix}@example.com"),
            )
            .config(json!({
                "smtp_host": "smtp.example.com",
                "smtp_port": 2525,
                "smtp_tls": false,
                "smtp_starttls": true,
                "smtp_username": "smtp-login@example.com"
            })),
        )
        .await
        .expect("store SMTP provider account");
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(&NewSecretReference::new(
            secret_ref,
            SecretKind::Password,
            SecretStoreKind::TestDouble,
            "SMTP test credential",
        ))
        .await
        .expect("store SMTP secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            account_id,
            ProviderAccountSecretPurpose::SmtpPassword,
            secret_ref,
        ))
        .await
        .expect("bind SMTP secret reference");
}

async fn seed_icloud_provider_account_with_imap_secret(
    pool: sqlx::PgPool,
    account_id: &str,
    secret_ref: &str,
    suffix: i64,
) {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::Icloud,
                "Outbox iCloud",
                format!("icloud-delivery-{suffix}@example.com"),
            )
            .config(json!({})),
        )
        .await
        .expect("store iCloud provider account");
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(&NewSecretReference::new(
            secret_ref,
            SecretKind::Password,
            SecretStoreKind::TestDouble,
            "iCloud test credential",
        ))
        .await
        .expect("store iCloud secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            account_id,
            ProviderAccountSecretPurpose::ImapPassword,
            secret_ref,
        ))
        .await
        .expect("bind IMAP secret reference");
}

async fn enqueue_due_item(
    store: &CommunicationOutboxStore,
    account_id: &str,
    outbox_id: &str,
    now: chrono::DateTime<Utc>,
) -> CommunicationOutboxItem {
    store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: outbox_id.to_owned(),
            account_id: account_id.to_owned(),
            draft_id: None,
            to_recipients: vec!["recipient@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Delivery worker".to_owned(),
            body_text: "Deliver this due outbox item.".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Queued,
            scheduled_send_at: Some(now - Duration::minutes(1)),
            undo_deadline_at: Some(now - Duration::seconds(1)),
            metadata: json!({ "source": "test" }),
        })
        .await
        .expect("enqueue outbox item")
}

struct StaticSuccessSender {
    provider_message_id: String,
    accepted_recipients: Vec<String>,
}

impl OutboxEmailSender for StaticSuccessSender {
    fn send<'a>(
        &'a self,
        _item: &'a CommunicationOutboxItem,
    ) -> Pin<Box<dyn Future<Output = Result<OutboxSendReceipt, OutboxDeliveryError>> + Send + 'a>>
    {
        Box::pin(async move {
            Ok(OutboxSendReceipt {
                provider_message_id: self.provider_message_id.clone(),
                accepted_recipients: self.accepted_recipients.clone(),
            })
        })
    }
}

struct StaticFailureSender {
    message: String,
}

#[derive(Clone, Debug)]
struct RecordedSmtpCall {
    config: SmtpConfig,
    password: String,
    email: OutgoingEmail,
}

#[derive(Clone, Default)]
struct RecordingSmtpTransport {
    calls: Arc<Mutex<Vec<RecordedSmtpCall>>>,
}

impl SmtpTransport for RecordingSmtpTransport {
    fn send<'a>(
        &'a self,
        config: &'a SmtpConfig,
        password: &'a ResolvedSecret,
        email: &'a OutgoingEmail,
    ) -> Pin<Box<dyn Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>> {
        Box::pin(async move {
            self.calls
                .lock()
                .expect("record SMTP call")
                .push(RecordedSmtpCall {
                    config: config.clone(),
                    password: password.expose_for_runtime().to_owned(),
                    email: email.clone(),
                });
            Ok(SendResult {
                message_id: "smtp-message-1".to_owned(),
                accepted_recipients: email.to.clone(),
            })
        })
    }
}

impl OutboxEmailSender for StaticFailureSender {
    fn send<'a>(
        &'a self,
        _item: &'a CommunicationOutboxItem,
    ) -> Pin<Box<dyn Future<Output = Result<OutboxSendReceipt, OutboxDeliveryError>> + Send + 'a>>
    {
        Box::pin(async move { Err(OutboxDeliveryError::Transport(self.message.clone())) })
    }
}
