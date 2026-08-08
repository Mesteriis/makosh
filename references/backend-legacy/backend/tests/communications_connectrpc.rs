use axum::body::to_bytes;
use axum::http::StatusCode;
use chrono::Utc;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::drafts::{
    CommunicationDraftStore, DraftStatus, NewCommunicationDraft,
};
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::communications::outbox::{
    CommunicationOutboxStatus, CommunicationOutboxStore, NewCommunicationOutboxItem,
};
use makosh_hub_backend::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use makosh_hub_backend::domains::communications::storage::models::{
    CommunicationAttachmentDisposition, NewCommunicationAttachment,
    NewCommunicationAttachmentImport, NewCommunicationBlob,
};
use makosh_hub_backend::domains::communications::storage::scanner::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus,
};
use makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore;

use makosh_backend_testkit::app::{TestApp, post_json};
use makosh_backend_testkit::composition::router_for_context;
use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn test_app() -> TestApp {
    let context = TestContext::new().await;
    let router = router_for_context(&context);
    TestApp::new(context, router)
}

#[tokio::test]
async fn communications_connect_api_requires_local_api_secret() {
    let app = test_app().await;
    let router = app.clone_router();

    let forbidden_response = router
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/makosh.communications.v1.CommunicationsService/ListMessages")
                .header("content-type", "application/json")
                .body(axum::body::Body::from("{}"))
                .expect("connect request without secret"),
        )
        .await
        .expect("connect response without secret");
    assert_eq!(forbidden_response.status(), StatusCode::FORBIDDEN);

    let allowed_response = router
        .oneshot(post_json(
            "/makosh.communications.v1.CommunicationsService/ListMessages",
            json!({}),
        ))
        .await
        .expect("connect response with secret");
    assert_eq!(allowed_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn communications_connect_api_exposes_provider_neutral_queries_and_send() {
    let app = test_app().await;
    let pool = app.context().pool().clone();
    let router = app.clone_router();
    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let draft_store = CommunicationDraftStore::new(pool.clone());
    let outbox_store = CommunicationOutboxStore::new(pool.clone());

    ingestion
        .upsert_provider_account(&NewProviderAccount::new(
            "acct-connectrpc-mail",
            CommunicationProviderKind::Gmail,
            "ConnectRPC Mail",
            "connectrpc@example.com",
        ))
        .await
        .expect("store provider account");
    let raw = ingestion
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                "raw-connectrpc-message",
                "acct-connectrpc-mail",
                "email_message",
                "provider-connectrpc-message",
                "sha256:connectrpc-message",
                "batch-connectrpc-message",
                json!({
                    "subject": "ConnectRPC Thread",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "ConnectRPC message body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source": "communications_connectrpc_test"})),
        )
        .await
        .expect("record raw message");
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project raw message");
    let raw_newsletter = ingestion
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                "raw-connectrpc-message-2",
                "acct-connectrpc-mail",
                "email_message",
                "provider-connectrpc-message-2",
                "sha256:connectrpc-message-2",
                "batch-connectrpc-message",
                json!({
                    "subject": "ConnectRPC Thread",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "ConnectRPC newsletter body with unsubscribe link"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source": "communications_connectrpc_test"})),
        )
        .await
        .expect("record second raw message");
    project_raw_email_message(&message_store, &raw_newsletter)
        .await
        .expect("project second raw message");
    let seeded_attachment = seed_connectrpc_attachment(
        &app.context().pool().clone(),
        &raw.raw_record_id,
        &projected.message_id,
        "connectrpc-note.txt",
        "text/plain",
        AttachmentSafetyScanStatus::Clean,
        b"Hola equipo\n",
    )
    .await;
    let seeded_pdf_attachment = seed_connectrpc_attachment(
        &app.context().pool().clone(),
        &raw.raw_record_id,
        &projected.message_id,
        "connectrpc-spec.pdf",
        "application/pdf",
        AttachmentSafetyScanStatus::Clean,
        b"%PDF-1.4\n",
    )
    .await;
    let draft_blob = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
        .put_blob(b"ConnectRPC draft attachment")
        .await
        .expect("draft attachment blob");
    let draft_blob_metadata = CommunicationStorageStore::new(pool.clone())
        .upsert_blob(&NewCommunicationBlob::from_local_blob(&draft_blob).content_type("text/plain"))
        .await
        .expect("draft attachment blob metadata");
    CommunicationStorageStore::new(pool.clone())
        .upsert_imported_attachment(
            &NewCommunicationAttachmentImport::new(
                "draft-connectrpc-attachment",
                draft_blob_metadata.blob_id,
                "text/plain",
                draft_blob.size_bytes,
                draft_blob.sha256,
                "communications-connectrpc-test",
            )
            .account_id("acct-connectrpc-mail")
            .channel_kind("mail")
            .filename("draft-note.txt")
            .scan_report(AttachmentSafetyScanReport {
                status: AttachmentSafetyScanStatus::Clean,
                engine: Some("test_scanner".to_owned()),
                checked_at: Some(Utc::now()),
                summary: Some("Synthetic fixture is clean".to_owned()),
                metadata: json!({"fixture": true}),
            }),
        )
        .await
        .expect("draft attachment import");

    draft_store
        .upsert(&NewCommunicationDraft {
            draft_id: "draft-connectrpc-1".to_owned(),
            account_id: "acct-connectrpc-mail".to_owned(),
            persona_id: None,
            to_recipients: vec!["draft@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "ConnectRPC Draft".to_owned(),
            body_text: "Draft body".to_owned(),
            body_html: None,
            in_reply_to: None,
            references: Vec::new(),
            attachment_ids: Some(vec!["draft-connectrpc-attachment".to_owned()]),
            status: DraftStatus::Draft,
            scheduled_send_at: None,
            metadata: json!({"origin":"connectrpc_test"}),
        })
        .await
        .expect("store draft");
    outbox_store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: "outbox-connectrpc-1".to_owned(),
            account_id: "acct-connectrpc-mail".to_owned(),
            draft_id: None,
            to_recipients: vec!["queued@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Queued ConnectRPC".to_owned(),
            body_text: "Queued body".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Queued,
            scheduled_send_at: None,
            undo_deadline_at: Some(Utc::now() + chrono::Duration::minutes(5)),
            metadata: json!({"seeded": true}),
        })
        .await
        .expect("store outbox");

    let list_messages = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListMessages",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "limit": 10
                }),
            ))
            .await
            .expect("list messages response"),
    )
    .await;
    assert!(list_messages["items"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["messageId"] == projected.message_id)
    }));
    assert!(
        list_messages["items"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item["workflowState"] == "new"))
    );

    let get_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetMessage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("get message response"),
    )
    .await;
    assert_eq!(get_message["item"]["subject"], "ConnectRPC Thread");
    assert_eq!(
        get_message["attachments"][0]["attachmentId"],
        seeded_attachment.attachment_id
    );

    let transitioned_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/TransitionMessageWorkflowState",
                json!({
                    "message_id": projected.message_id,
                    "workflow_state": "reviewed"
                }),
            ))
            .await
            .expect("transition workflow state response"),
    )
    .await;
    assert_eq!(transitioned_message["messageId"], projected.message_id);
    assert_eq!(transitioned_message["workflowState"], "reviewed");
    assert_eq!(transitioned_message["previousState"], "new");

    let trashed_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/TrashMessage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("trash message response"),
    )
    .await;
    assert_eq!(trashed_message["messageId"], projected.message_id);
    assert_eq!(trashed_message["localState"], "trash");

    let restored_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/RestoreMessage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("restore message response"),
    )
    .await;
    assert_eq!(restored_message["messageId"], projected.message_id);
    assert_eq!(restored_message["localState"], "active");

    let marked_read = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/MarkMessageRead",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("mark message read response"),
    )
    .await;
    assert_eq!(marked_read["messageId"], projected.message_id);
    assert_eq!(marked_read["markedRead"], true);
    assert_eq!(marked_read["workflowState"], "reviewed");

    let deleted_from_provider = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/DeleteMessageFromProvider",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("delete message from provider response"),
    )
    .await;
    assert_eq!(deleted_from_provider["messageId"], projected.message_id);
    assert_eq!(deleted_from_provider["deleted"], true);
    assert_eq!(deleted_from_provider["localState"], "trash");

    let restored_after_provider_delete = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/RestoreMessage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("restore message after provider delete response"),
    )
    .await;
    assert_eq!(restored_after_provider_delete["localState"], "active");

    let bulk_action = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/BulkMessageAction",
                json!({
                    "action": "trash",
                    "message_ids": [projected.message_id]
                }),
            ))
            .await
            .expect("bulk message action response"),
    )
    .await;
    assert_eq!(bulk_action["action"], "trash");
    assert_eq!(bulk_action["updatedCount"], 1);

    let restored_after_bulk = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/RestoreMessage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("restore message after bulk response"),
    )
    .await;
    assert_eq!(restored_after_bulk["localState"], "active");

    let pinned_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ToggleMessagePin",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("toggle pin response"),
    )
    .await;
    assert_eq!(pinned_message["messageId"], projected.message_id);
    assert_eq!(pinned_message["pinned"], true);

    let important_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ToggleMessageImportant",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("toggle important response"),
    )
    .await;
    assert_eq!(important_message["important"], true);

    let muted_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ToggleMessageMute",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("toggle mute response"),
    )
    .await;
    assert_eq!(muted_message["muted"], true);

    let snoozed_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/SnoozeMessage",
                json!({
                    "message_id": projected.message_id,
                    "until": "2026-06-30T10:00:00Z"
                }),
            ))
            .await
            .expect("snooze message response"),
    )
    .await;
    assert_eq!(snoozed_message["snoozed"], true);

    let labeled_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/AddMessageLabel",
                json!({
                    "message_id": projected.message_id,
                    "label": "follow-up"
                }),
            ))
            .await
            .expect("add label response"),
    )
    .await;
    assert_eq!(labeled_message["labeled"], true);

    let unlabeled_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/RemoveMessageLabel",
                json!({
                    "message_id": projected.message_id,
                    "label": "follow-up"
                }),
            ))
            .await
            .expect("remove label response"),
    )
    .await;
    assert_eq!(unlabeled_message["removed"], true);

    let workflow_state_counts = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListMessageWorkflowStateCounts",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "local_state": "active"
                }),
            ))
            .await
            .expect("list workflow state counts response"),
    )
    .await;
    assert!(
        workflow_state_counts["counts"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item["state"] == "reviewed"))
    );

    let subscriptions = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListSubscriptions",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "limit": 10
                }),
            ))
            .await
            .expect("list subscriptions response"),
    )
    .await;
    assert_eq!(subscriptions["items"][0]["sender"], "alice@example.com");
    assert_eq!(subscriptions["items"][0]["hasUnsubscribe"], true);

    let mailbox_health = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetMailboxHealth",
                json!({
                    "account_id": "acct-connectrpc-mail"
                }),
            ))
            .await
            .expect("get mailbox health response"),
    )
    .await;
    assert_eq!(mailbox_health["item"]["totalMessages"], "2");
    assert_eq!(mailbox_health["item"]["withAttachments"], "1");

    let top_senders = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListTopSenders",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "limit": 10
                }),
            ))
            .await
            .expect("list top senders response"),
    )
    .await;
    assert_eq!(top_senders["items"][0]["sender"], "alice@example.com");
    assert_eq!(top_senders["items"][0]["messageCount"], "2");

    let blockers = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListCommunicationBlockers",
                json!({}),
            ))
            .await
            .expect("list blockers response"),
    )
    .await;
    assert!(
        blockers["items"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );

    let search_messages = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/SearchMessages",
                json!({
                    "query": "ConnectRPC",
                    "limit": 10
                }),
            ))
            .await
            .expect("search messages response"),
    )
    .await;
    assert!(
        search_messages["results"].is_null() || search_messages["results"].as_array().is_some()
    );

    let detected_language = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/DetectMessageLanguage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("detect message language response"),
    )
    .await;
    assert_eq!(detected_language["language"], "en");

    let analyzed_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/AnalyzeMessage",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("analyze message response"),
    )
    .await;
    assert_eq!(analyzed_message["messageId"], projected.message_id);
    assert_eq!(analyzed_message["analyzed"], true);
    assert_eq!(analyzed_message["source"], "local_heuristic");
    assert!(analyzed_message["summaryContract"].is_object());
    assert!(analyzed_message["evidence"].is_array());

    let explained_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetMessageExplain",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("get message explain response"),
    )
    .await;
    assert!(explained_message["reasons"].is_array());

    let smart_cc = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetMessageSmartCc",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("get smart cc response"),
    )
    .await;
    assert!(smart_cc["suggestions"].is_null() || smart_cc["suggestions"].is_array());

    let exported_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetMessageExport",
                json!({
                    "message_id": projected.message_id,
                    "format": "json"
                }),
            ))
            .await
            .expect("get message export response"),
    )
    .await;
    assert_eq!(exported_message["contentType"], "application/json");
    assert!(exported_message["content"].as_str().is_some());
    assert!(exported_message["filename"].as_str().is_some());

    let auth_report = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetMessageAuth",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("get message auth response"),
    )
    .await;
    assert!(auth_report["auth"].is_object());
    assert!(auth_report["risk"].is_object());

    let signature = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetMessageSignature",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("get message signature response"),
    )
    .await;
    assert!(signature["hasSignature"].is_null() || signature["hasSignature"].is_boolean());

    let ai_reply = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GenerateAiReply",
                json!({
                    "message_id": projected.message_id,
                    "tone": "business",
                    "language": "en"
                }),
            ))
            .await
            .expect("generate ai reply response"),
    )
    .await;
    assert!(ai_reply["generated"].is_null() || ai_reply["generated"].is_boolean());
    if ai_reply["generated"].as_bool().unwrap_or(false) {
        assert!(ai_reply["body"].as_str().is_some());
    } else {
        assert!(ai_reply["reason"].as_str().is_some());
    }

    let ai_reply_variants = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GenerateAiReplyVariants",
                json!({
                    "message_id": projected.message_id,
                    "languages": ["en"],
                    "tones": ["professional"]
                }),
            ))
            .await
            .expect("generate ai reply variants response"),
    )
    .await;
    assert!(ai_reply_variants["variants"].is_null() || ai_reply_variants["variants"].is_array());

    let translated_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/TranslateMessage",
                json!({
                    "message_id": projected.message_id,
                    "target_language": "es"
                }),
            ))
            .await
            .expect("translate message response"),
    )
    .await;
    assert!(!translated_message["translated"].as_bool().unwrap_or(false));
    assert!(translated_message["reason"].as_str().is_some());

    let extracted_tasks = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ExtractMessageTasks",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("extract message tasks response"),
    )
    .await;
    assert!(extracted_tasks["tasks"].is_null() || extracted_tasks["tasks"].is_array());

    let extracted_notes = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ExtractMessageNotes",
                json!({
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("extract message notes response"),
    )
    .await;
    assert!(extracted_notes["notes"].is_null() || extracted_notes["notes"].is_array());

    let list_threads = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListThreads",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "limit": 10
                }),
            ))
            .await
            .expect("list threads response"),
    )
    .await;
    assert_eq!(list_threads["items"][0]["subject"], "ConnectRPC Thread");

    let thread_messages = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListThreadMessages",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "subject": "ConnectRPC Thread",
                    "limit": 10
                }),
            ))
            .await
            .expect("list thread messages response"),
    )
    .await;
    assert_eq!(
        thread_messages["items"][0]["messageId"],
        projected.message_id
    );

    let translated_thread = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/TranslateThread",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "subject": "ConnectRPC Thread",
                    "target_language": "en",
                    "limit": 10
                }),
            ))
            .await
            .expect("translate thread response"),
    )
    .await;
    assert_eq!(translated_thread["accountId"], "acct-connectrpc-mail");
    assert_eq!(translated_thread["subject"], "ConnectRPC Thread");
    assert_eq!(translated_thread["targetLanguage"], "en");
    assert_eq!(
        translated_thread["items"][0]["messageId"],
        projected.message_id
    );

    let attachment_search = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/SearchAttachments",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "query": "connectrpc-note",
                    "limit": 10
                }),
            ))
            .await
            .expect("search attachments response"),
    )
    .await;
    assert_eq!(
        attachment_search["items"][0]["attachmentId"],
        seeded_attachment.attachment_id
    );

    let attachment_preview = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetAttachmentPreview",
                json!({
                    "attachment_id": seeded_attachment.attachment_id
                }),
            ))
            .await
            .expect("attachment preview response"),
    )
    .await;
    assert_eq!(
        attachment_preview["attachmentId"],
        seeded_attachment.attachment_id
    );
    assert_eq!(attachment_preview["previewKind"], "text");
    assert_eq!(attachment_preview["text"], "Hola equipo\n");

    let attachment_extraction = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ExtractAttachmentText",
                json!({ "attachment_id": seeded_attachment.attachment_id }),
            ))
            .await
            .expect("attachment text extraction response"),
    )
    .await;
    assert_eq!(
        attachment_extraction["attachmentId"],
        seeded_attachment.attachment_id
    );
    assert_eq!(attachment_extraction["status"], "completed");
    assert_eq!(attachment_extraction["extractedSizeBytes"], "12");

    let extracted_attachment_text = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetAttachmentExtractedText",
                json!({ "attachment_id": seeded_attachment.attachment_id }),
            ))
            .await
            .expect("attachment extracted text response"),
    )
    .await;
    assert_eq!(
        extracted_attachment_text["attachmentId"],
        seeded_attachment.attachment_id
    );
    assert_eq!(extracted_attachment_text["text"], "Hola equipo\n");
    assert!(
        extracted_attachment_text["truncated"].is_null(),
        "ConnectJSON omits default false scalar fields"
    );
    assert_eq!(extracted_attachment_text["extractedSizeBytes"], "12");

    let pdf_attachment_preview = router
        .clone()
        .oneshot(post_json(
            "/makosh.communications.v1.CommunicationsService/GetAttachmentPreview",
            json!({
                "attachment_id": seeded_pdf_attachment.attachment_id
            }),
        ))
        .await
        .expect("attachment pdf preview response");
    assert_eq!(pdf_attachment_preview.status(), StatusCode::BAD_REQUEST);
    let preview_error = response_body_json(pdf_attachment_preview).await;
    assert_eq!(preview_error["code"], "failed_precondition");
    assert_eq!(
        preview_error["message"],
        "extract attachment text before preview"
    );

    store_connectrpc_completed_safe_preview(
        &pool,
        &seeded_pdf_attachment.attachment_id,
        b"\x89PNG\r\n\x1a\n",
    )
    .await;
    let pdf_attachment_preview = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/GetAttachmentPreview",
                json!({
                    "attachment_id": seeded_pdf_attachment.attachment_id
                }),
            ))
            .await
            .expect("derived attachment pdf preview response"),
    )
    .await;
    assert_eq!(
        pdf_attachment_preview["attachmentId"],
        seeded_pdf_attachment.attachment_id
    );
    assert_eq!(pdf_attachment_preview["previewKind"], "image");
    assert!(pdf_attachment_preview["text"].is_null());
    assert_eq!(
        pdf_attachment_preview["dataUrl"],
        "data:image/png;base64,iVBORw0KGgo="
    );

    let quarantined_attachment = seed_connectrpc_attachment(
        &app.context().pool().clone(),
        &raw.raw_record_id,
        &projected.message_id,
        "connectrpc-pending.txt",
        "text/plain",
        AttachmentSafetyScanStatus::NotScanned,
        b"Quarantined until ClamAV returns a clean verdict.",
    )
    .await;
    let quarantined_preview = router
        .clone()
        .oneshot(post_json(
            "/makosh.communications.v1.CommunicationsService/GetAttachmentPreview",
            json!({ "attachment_id": quarantined_attachment.attachment_id }),
        ))
        .await
        .expect("quarantined attachment preview response");
    assert_eq!(quarantined_preview.status(), StatusCode::BAD_REQUEST);

    let quarantined_translation = router
        .clone()
        .oneshot(post_json(
            "/makosh.communications.v1.CommunicationsService/TranslateAttachment",
            json!({
                "attachment_id": quarantined_attachment.attachment_id,
                "target_language": "en",
                "source_text": "Quarantined text must not reach translation."
            }),
        ))
        .await
        .expect("quarantined attachment translation response");
    assert_eq!(quarantined_translation.status(), StatusCode::BAD_REQUEST);

    let attachment_translation = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/TranslateAttachment",
                json!({
                    "attachment_id": seeded_attachment.attachment_id,
                    "target_language": "en",
                    "source_text": "Hola equipo"
                }),
            ))
            .await
            .expect("attachment translation response"),
    )
    .await;
    assert_eq!(
        attachment_translation["attachmentId"],
        seeded_attachment.attachment_id
    );
    assert_eq!(attachment_translation["source"], "durable_extracted_text");
    assert!(
        !attachment_translation["translated"]
            .as_bool()
            .unwrap_or(false)
    );
    assert_eq!(
        attachment_translation["reason"],
        "translation runtime unavailable"
    );

    let list_saved_searches = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListSavedSearches",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "page": { "limit": 10, "cursor": "" }
                }),
            ))
            .await
            .expect("list saved searches response"),
    )
    .await;
    assert!(list_saved_searches.is_object());

    let created_saved_search = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/CreateSavedSearch",
                json!({
                    "name": "ConnectRPC invoices",
                    "account_id": "acct-connectrpc-mail",
                    "query": "invoice",
                    "workflow_state": "needs_action",
                    "local_state": "active",
                    "channel_kind": "email",
                    "is_smart_folder": true,
                    "sort_order": 10
                }),
            ))
            .await
            .expect("create saved search response"),
    )
    .await;
    let saved_search_id = created_saved_search["item"]["savedSearchId"]
        .as_str()
        .expect("saved search id")
        .to_owned();
    assert_eq!(created_saved_search["item"]["name"], "ConnectRPC invoices");
    assert_eq!(
        created_saved_search["item"]["workflowState"],
        "needs_action"
    );

    let updated_saved_search = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/UpdateSavedSearch",
                json!({
                    "saved_search_id": saved_search_id,
                    "name": "ConnectRPC waiting invoices",
                    "workflow_state": "waiting",
                    "is_smart_folder": false
                }),
            ))
            .await
            .expect("update saved search response"),
    )
    .await;
    assert_eq!(
        updated_saved_search["item"]["name"],
        "ConnectRPC waiting invoices"
    );
    assert_eq!(updated_saved_search["item"]["workflowState"], "waiting");

    let deleted_saved_search = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/DeleteSavedSearch",
                json!({
                    "saved_search_id": saved_search_id
                }),
            ))
            .await
            .expect("delete saved search response"),
    )
    .await;
    assert_eq!(deleted_saved_search["deleted"], true);

    let listed_folders = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListFolders",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "page": { "limit": 10, "cursor": "" }
                }),
            ))
            .await
            .expect("list folders response"),
    )
    .await;
    assert!(listed_folders.is_object());

    let created_folder = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/CreateFolder",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "name": "ConnectRPC Clients",
                    "description": "Important clients",
                    "color": "#3b82f6",
                    "sort_order": 10
                }),
            ))
            .await
            .expect("create folder response"),
    )
    .await;
    let folder_id = created_folder["item"]["folderId"]
        .as_str()
        .expect("folder id")
        .to_owned();
    assert_eq!(created_folder["item"]["name"], "ConnectRPC Clients");

    let updated_folder = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/UpdateFolder",
                json!({
                    "folder_id": folder_id,
                    "name": "ConnectRPC VIP Clients",
                    "color": "#2563eb",
                    "sort_order": 20
                }),
            ))
            .await
            .expect("update folder response"),
    )
    .await;
    assert_eq!(updated_folder["item"]["name"], "ConnectRPC VIP Clients");

    let copied_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/CopyMessageToFolder",
                json!({
                    "folder_id": folder_id,
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("copy folder message response"),
    )
    .await;
    assert_eq!(copied_message["item"]["operation"], "copy");

    let folder_messages = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListFolderMessages",
                json!({
                    "folder_id": folder_id,
                    "page": { "limit": 10, "cursor": "" }
                }),
            ))
            .await
            .expect("list folder messages response"),
    )
    .await;
    assert_eq!(
        folder_messages["items"][0]["messageId"],
        projected.message_id
    );

    let moved_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/MoveMessageToFolder",
                json!({
                    "folder_id": folder_id,
                    "message_id": projected.message_id
                }),
            ))
            .await
            .expect("move folder message response"),
    )
    .await;
    assert_eq!(moved_message["item"]["operation"], "move");

    let deleted_folder = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/DeleteFolder",
                json!({
                    "folder_id": folder_id
                }),
            ))
            .await
            .expect("delete folder response"),
    )
    .await;
    assert_eq!(deleted_folder["deleted"], true);

    let list_drafts = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListDrafts",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "page": { "limit": 10, "cursor": "" }
                }),
            ))
            .await
            .expect("list drafts response"),
    )
    .await;
    assert_eq!(list_drafts["items"][0]["draftId"], "draft-connectrpc-1");
    assert_eq!(
        list_drafts["items"][0]["attachmentIds"],
        json!(["draft-connectrpc-attachment"])
    );
    assert_eq!(
        list_drafts["items"][0]["attachments"][0]["filename"],
        "draft-note.txt"
    );
    assert_eq!(
        list_drafts["items"][0]["attachments"][0]["scanStatus"],
        "clean"
    );

    let created_draft = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/CreateDraft",
                json!({
                    "draft_id": "draft-connectrpc-2",
                    "account_id": "acct-connectrpc-mail",
                    "to_recipients": ["create@example.com"],
                    "subject": "ConnectRPC Created Draft",
                    "body_text": "Created via ConnectRPC",
                    "status": "draft",
                    "metadata_json": "{\"origin\":\"communications_connectrpc_test\"}"
                }),
            ))
            .await
            .expect("create draft response"),
    )
    .await;
    assert_eq!(created_draft["item"]["draftId"], "draft-connectrpc-2");
    assert_eq!(
        created_draft["item"]["metadataJson"],
        "{\"origin\":\"communications_connectrpc_test\"}"
    );

    let deleted_draft = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/DeleteDraft",
                json!({
                    "draft_id": "draft-connectrpc-2"
                }),
            ))
            .await
            .expect("delete draft response"),
    )
    .await;
    assert_eq!(deleted_draft["deleted"], true);

    let list_outbox = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/ListOutbox",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "page": { "limit": 10, "cursor": "" }
                }),
            ))
            .await
            .expect("list outbox response"),
    )
    .await;
    assert_eq!(list_outbox["items"][0]["outboxId"], "outbox-connectrpc-1");

    let undone_outbox = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/UndoOutboxItem",
                json!({
                    "outbox_id": "outbox-connectrpc-1"
                }),
            ))
            .await
            .expect("undo outbox response"),
    )
    .await;
    assert_eq!(undone_outbox["item"]["outboxId"], "outbox-connectrpc-1");
    assert_eq!(undone_outbox["item"]["status"], "canceled");

    let send_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/SendMessage",
                json!({
                    "account_id": "acct-connectrpc-mail",
                    "to_recipients": ["receiver@example.com"],
                    "subject": "ConnectRPC Send",
                    "body_text": "Queued via ConnectRPC",
                    "draft_id": "draft-connectrpc-1",
                    "undo_send_seconds": "300",
                    "confirmed_provider_write": true,
                    "metadata_json": "{\"source\":\"connectrpc\"}"
                }),
            ))
            .await
            .expect("send message response"),
    )
    .await;
    assert!(
        send_message["messageId"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert!(
        send_message["outboxId"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert_eq!(send_message["item"]["accountId"], "acct-connectrpc-mail");
    assert_eq!(send_message["item"]["status"], "queued");
    assert_eq!(send_message["transport"], "outbox");
    assert_eq!(send_message["status"], "queued");
    assert_eq!(
        send_message["acceptedRecipients"]
            .as_array()
            .expect("accepted recipients")
            .len(),
        1
    );
    let redirect_message = response_json(
        router
            .clone()
            .oneshot(post_json(
                "/makosh.communications.v1.CommunicationsService/RedirectMessage",
                json!({
                    "message_id": projected.message_id,
                    "to_recipients": ["redirect@example.com"],
                    "cc_recipients": ["copy@example.com"],
                    "confirmed_provider_write": true
                }),
            ))
            .await
            .expect("redirect message response"),
    )
    .await;
    assert!(
        redirect_message["outboxId"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert_eq!(redirect_message["transport"], "outbox");
    assert_eq!(redirect_message["status"], "queued");
    assert_eq!(
        redirect_message["acceptedRecipients"],
        json!(["redirect@example.com"])
    );
    let metadata_json = send_message["item"]["metadataJson"]
        .as_str()
        .expect("metadata json string");
    let metadata: Value = serde_json::from_str(metadata_json).expect("metadata json");
    assert_eq!(metadata["source"], "connectrpc");
    assert_eq!(metadata["from"], "connectrpc@example.com");
    assert_eq!(send_message["item"]["draftId"], "draft-connectrpc-1");
    assert!(send_message["item"]["undoDeadlineAt"].is_string());
}

async fn response_json(response: axum::response::Response) -> Value {
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("json body")
}

async fn response_body_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("json body")
}

struct SeededAttachment {
    attachment_id: String,
}

async fn seed_connectrpc_attachment(
    pool: &sqlx::PgPool,
    raw_record_id: &str,
    message_id: &str,
    filename: &str,
    content_type: &str,
    scan_status: AttachmentSafetyScanStatus,
    bytes: &[u8],
) -> SeededAttachment {
    let storage_store = CommunicationStorageStore::new(pool.clone());
    let local_blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let local_blob = local_blob_store
        .put_blob(bytes)
        .await
        .expect("write connectrpc attachment blob");
    let blob = storage_store
        .upsert_blob(&NewCommunicationBlob::from_local_blob(&local_blob).content_type(content_type))
        .await
        .expect("store connectrpc attachment blob");
    let attachment = storage_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                message_id,
                raw_record_id,
                blob.blob_id,
                format!("part-connectrpc-attachment-{filename}"),
                content_type,
                local_blob.size_bytes,
                local_blob.sha256,
            )
            .filename(filename)
            .disposition(CommunicationAttachmentDisposition::Attachment)
            .scan_report(AttachmentSafetyScanReport {
                status: scan_status,
                engine: None,
                checked_at: None,
                summary: None,
                metadata: json!({}),
            }),
        )
        .await
        .expect("store connectrpc attachment");

    SeededAttachment {
        attachment_id: attachment.attachment_id,
    }
}

async fn store_connectrpc_completed_safe_preview(
    pool: &sqlx::PgPool,
    attachment_id: &str,
    bytes: &[u8],
) {
    let source_sha256: String =
        sqlx::query_scalar("SELECT sha256 FROM communication_attachments WHERE attachment_id = $1")
            .bind(attachment_id)
            .fetch_one(pool)
            .await
            .expect("connectrpc attachment source hash");
    let local_blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let preview_blob = local_blob_store
        .put_blob(bytes)
        .await
        .expect("write connectrpc safe preview blob");
    let stored_blob = CommunicationStorageStore::new(pool.clone())
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&preview_blob).content_type("image/png"),
        )
        .await
        .expect("store connectrpc safe preview blob metadata");

    sqlx::query(
        r#"
        INSERT INTO communication_attachment_safe_previews (
            attachment_id, status, renderer, source_sha256, preview_blob_id,
            preview_content_type, preview_size_bytes, rendered_at
        ) VALUES ($1, 'completed', 'makosh.attachment-extractor.pdf_preview.v1', $2, $3, 'image/png', $4, now())
        "#,
    )
    .bind(attachment_id)
    .bind(source_sha256)
    .bind(stored_blob.blob_id)
    .bind(preview_blob.size_bytes)
    .execute(pool)
    .await
    .expect("store connectrpc completed safe preview");
}
