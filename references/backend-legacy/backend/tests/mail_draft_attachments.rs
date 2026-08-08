use chrono::Utc;
use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use serde_json::json;

use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_hub_backend::domains::communications::drafts::{
    CommunicationDraftStore, DraftStatus, NewCommunicationDraft,
};
use makosh_hub_backend::domains::communications::outbox::delivery::{
    EmailOutboxDeliveryWorker, OutboxDeliveryError, OutboxEmailSender, OutboxSendReceipt,
};
use makosh_hub_backend::domains::communications::outbox::{
    CommunicationOutboxStatus, CommunicationOutboxStore, NewCommunicationOutboxItem,
};
use makosh_hub_backend::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use makosh_hub_backend::domains::communications::storage::models::{
    NewCommunicationAttachmentImport, NewCommunicationBlob,
};
use makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore;
use makosh_hub_backend::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;

#[derive(Clone)]
struct PermanentFailureSender;

impl OutboxEmailSender for PermanentFailureSender {
    fn send<'a>(
        &'a self,
        _item: &'a makosh_hub_backend::domains::communications::outbox::CommunicationOutboxItem,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<OutboxSendReceipt, OutboxDeliveryError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async {
            Err(OutboxDeliveryError::Permanent(
                "attachment quarantine blocked delivery".to_owned(),
            ))
        })
    }
}

#[tokio::test]
async fn draft_attachments_are_ordered_and_snapshotted_into_outbox() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let account_id = "mail-draft-attachment-account";
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Draft attachment account",
            "draft-attachments@example.test",
        ))
        .await
        .expect("provider account");

    let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let storage = CommunicationStorageStore::new(pool.clone());
    let mut attachment_ids = Vec::new();
    for (index, bytes) in [
        b"first attachment".as_slice(),
        b"second attachment".as_slice(),
    ]
    .into_iter()
    .enumerate()
    {
        let local_blob = blob_store.put_blob(bytes).await.expect("local blob");
        let blob = storage
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&local_blob).content_type("text/plain"),
            )
            .await
            .expect("blob metadata");
        let attachment_id = format!("draft-attachment-{index}");
        storage
            .upsert_imported_attachment(
                &NewCommunicationAttachmentImport::new(
                    &attachment_id,
                    &blob.blob_id,
                    "text/plain",
                    local_blob.size_bytes,
                    &local_blob.sha256,
                    "mail-draft-attachment-test",
                )
                .account_id(account_id)
                .channel_kind("mail")
                .filename(format!("attachment-{index}.txt")),
            )
            .await
            .expect("attachment import");
        attachment_ids.push(attachment_id);
    }

    let draft_store = CommunicationDraftStore::new(pool.clone());
    let draft = draft_store
        .upsert(&NewCommunicationDraft {
            draft_id: "draft-with-attachments".to_owned(),
            account_id: account_id.to_owned(),
            persona_id: None,
            to_recipients: vec!["recipient@example.test".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Durable attachments".to_owned(),
            body_text: "Attachment body".to_owned(),
            body_html: None,
            in_reply_to: None,
            references: Vec::new(),
            attachment_ids: Some(attachment_ids.clone()),
            status: DraftStatus::Draft,
            scheduled_send_at: None,
            metadata: json!({}),
        })
        .await
        .expect("draft with attachments");
    assert_eq!(draft.attachment_ids, attachment_ids);
    assert_eq!(draft.attachments.len(), 2);
    assert_eq!(draft.attachments[0].attachment_id, attachment_ids[0]);
    assert_eq!(
        draft.attachments[0].filename.as_deref(),
        Some("attachment-0.txt")
    );
    assert_eq!(draft.attachments[0].content_type, "text/plain");
    assert_eq!(draft.attachments[0].size_bytes, 16);
    assert_eq!(draft.attachments[0].scan_status, "not_scanned");

    let draft = draft_store
        .upsert(&NewCommunicationDraft {
            draft_id: draft.draft_id,
            account_id: draft.account_id,
            persona_id: draft.persona_id,
            to_recipients: draft.to_recipients,
            cc_recipients: draft.cc_recipients,
            bcc_recipients: draft.bcc_recipients,
            subject: "Autosaved subject".to_owned(),
            body_text: draft.body_text,
            body_html: draft.body_html,
            in_reply_to: draft.in_reply_to,
            references: draft.references,
            attachment_ids: None,
            status: draft.status,
            scheduled_send_at: draft.scheduled_send_at,
            metadata: draft.metadata,
        })
        .await
        .expect("autosave without attachment replacement");
    assert_eq!(draft.attachment_ids, attachment_ids);
    assert_eq!(draft.attachments.len(), 2);
    assert_eq!(draft.subject, "Autosaved subject");

    let outbox = CommunicationOutboxStore::new(pool.clone())
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: "outbox-with-attachments".to_owned(),
            account_id: account_id.to_owned(),
            draft_id: Some(draft.draft_id.clone()),
            to_recipients: draft.to_recipients.clone(),
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: draft.subject.clone(),
            body_text: draft.body_text.clone(),
            body_html: None,
            status: CommunicationOutboxStatus::Queued,
            scheduled_send_at: None,
            undo_deadline_at: Some(Utc::now() + chrono::Duration::seconds(30)),
            metadata: json!({}),
        })
        .await
        .expect("outbox item");

    let snapshotted_ids = sqlx::query_scalar::<_, String>(
        r#"
        SELECT attachment_id
        FROM communication_outbox_attachments
        WHERE outbox_id = $1
        ORDER BY sort_order
        "#,
    )
    .bind(&outbox.outbox_id)
    .fetch_all(&pool)
    .await
    .expect("outbox attachment snapshot");
    assert_eq!(snapshotted_ids, attachment_ids);
}

#[tokio::test]
async fn draft_rejects_attachment_from_another_account_atomically() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    for account_id in ["mail-draft-owner", "mail-draft-other"] {
        CommunicationProviderAccountStore::new(pool.clone())
            .upsert(&NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::Gmail,
                account_id,
                format!("{account_id}@example.test"),
            ))
            .await
            .expect("provider account");
    }
    let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let local_blob = blob_store
        .put_blob(b"account scoped attachment")
        .await
        .expect("local blob");
    let storage = CommunicationStorageStore::new(pool.clone());
    let blob = storage
        .upsert_blob(&NewCommunicationBlob::from_local_blob(&local_blob).content_type("text/plain"))
        .await
        .expect("blob metadata");
    storage
        .upsert_imported_attachment(
            &NewCommunicationAttachmentImport::new(
                "other-account-attachment",
                &blob.blob_id,
                "text/plain",
                local_blob.size_bytes,
                &local_blob.sha256,
                "mail-draft-attachment-test",
            )
            .account_id("mail-draft-other")
            .channel_kind("mail"),
        )
        .await
        .expect("attachment import");

    let result = CommunicationDraftStore::new(pool.clone())
        .upsert(&NewCommunicationDraft {
            draft_id: "cross-account-draft".to_owned(),
            account_id: "mail-draft-owner".to_owned(),
            persona_id: None,
            to_recipients: Vec::new(),
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: String::new(),
            body_text: String::new(),
            body_html: None,
            in_reply_to: None,
            references: Vec::new(),
            attachment_ids: Some(vec!["other-account-attachment".to_owned()]),
            status: DraftStatus::Draft,
            scheduled_send_at: None,
            metadata: json!({}),
        })
        .await;
    assert!(result.is_err());

    let draft_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM communication_drafts WHERE draft_id = 'cross-account-draft')",
    )
    .fetch_one(&pool)
    .await
    .expect("draft existence");
    assert!(!draft_exists);
}

#[tokio::test]
async fn permanent_attachment_failure_does_not_schedule_transport_retry() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let account_id = "mail-permanent-attachment-failure";
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Permanent attachment failure",
            "blocked@example.test",
        ))
        .await
        .expect("provider account");
    let now = Utc::now();
    let store = CommunicationOutboxStore::new(pool.clone());
    store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: "blocked-attachment-outbox".to_owned(),
            account_id: account_id.to_owned(),
            draft_id: None,
            to_recipients: vec!["recipient@example.test".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Blocked attachment".to_owned(),
            body_text: "Body".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Queued,
            scheduled_send_at: None,
            undo_deadline_at: None,
            metadata: json!({}),
        })
        .await
        .expect("outbox item");

    let report = EmailOutboxDeliveryWorker::new(store.clone(), PermanentFailureSender)
        .deliver_due(now, 10)
        .await
        .expect("delivery report");
    assert_eq!(report.claimed, 1);
    assert_eq!(report.failed, 1);
    assert_eq!(report.retried, 0);
    let item = store
        .get("blocked-attachment-outbox")
        .await
        .expect("outbox lookup")
        .expect("outbox item");
    assert_eq!(item.status, CommunicationOutboxStatus::Failed);
    assert_eq!(
        item.last_error.as_deref(),
        Some("attachment quarantine blocked delivery")
    );
}
