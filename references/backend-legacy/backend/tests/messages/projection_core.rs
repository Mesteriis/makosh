use chrono::Utc;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use makosh_communications_api::mail_resources::{
    MailProviderResourceKind, MailProviderSemanticRole,
};
use serde_json::json;

use makosh_hub_backend::domains::communications::bulk_actions::{
    BulkMessageAction, BulkMessageActionStore,
};
use makosh_hub_backend::domains::communications::folders::{
    CommunicationFolderStore, NewCommunicationFolder,
};
use makosh_hub_backend::domains::communications::messages::models::NewProjectedMessage;
use makosh_hub_backend::domains::communications::messages::projection::{
    project_raw_email_message, project_raw_email_message_from_blob,
};
use makosh_hub_backend::domains::communications::outbox::delivery::OutboxSendReceipt;
use makosh_hub_backend::domains::communications::outbox::{
    CommunicationOutboxStatus, CommunicationOutboxStore, NewCommunicationOutboxItem,
};
use makosh_hub_backend::domains::communications::provider_resources::{
    MailProviderResourceMappingUpdate, MailProviderResourceStore, NewMailProviderResource,
};
use makosh_hub_backend::domains::communications::storage::blob_store::LocalCommunicationBlobStore;

use super::support::{
    live_projection_context, record_raw_email_message, store_provider_account, unique_suffix,
};

#[tokio::test]
async fn message_projection_upserts_canonical_message_against_postgres() {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("message projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_projection_{suffix}");
    let raw_record_id = format!("raw_message_projection_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection Gmail",
        format!("projection-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &raw_record_id,
        &format!("provider-message-{suffix}"),
        "Projected subject",
        "Projected body",
    )
    .await;

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    assert_eq!(projected.account_id, account_id);
    assert_eq!(projected.observation_id, raw.observation_id);
    assert_eq!(
        projected.provider_record_id,
        format!("provider-message-{suffix}")
    );
    assert_eq!(projected.subject, "Projected subject");
    assert_eq!(projected.sender, "alice@example.com");
    assert_eq!(projected.recipients, vec!["bob@example.com".to_owned()]);
}

#[tokio::test]
async fn message_projection_preserves_local_star_until_gmail_observes_it_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("Gmail star reconciliation").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_star_reconciliation_{suffix}");
    let provider_record_id = format!("provider-star-reconciliation-{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Gmail star reconciliation",
        format!("star-reconciliation-{suffix}@example.com"),
    )
    .await;

    let initial_raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-star-initial-{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:star-initial-{suffix}"),
                format!("batch-star-initial-{suffix}"),
                json!({
                    "provider": "gmail",
                    "label_ids": ["INBOX"],
                    "subject": "Star reconciliation",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Star reconciliation body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"projection_core_test"})),
        )
        .await
        .expect("initial Gmail observation");
    let initial = project_raw_email_message(&message_store, &initial_raw)
        .await
        .expect("project initial Gmail observation");

    BulkMessageActionStore::new(pool.clone())
        .apply(vec![initial.message_id.clone()], BulkMessageAction::Star)
        .await
        .expect("record local star intent");
    let queued_command: (String, String, String) = sqlx::query_as(
        "SELECT provider_message_id, status, reconciliation_status FROM communication_provider_commands WHERE account_id = $1 AND command_kind = 'star'",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("queued star provider command");
    assert_eq!(queued_command.0, provider_record_id);
    assert_eq!(queued_command.1, "queued");
    assert_eq!(queued_command.2, "not_observed");

    let stale_provider_raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-star-stale-{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:star-stale-{suffix}"),
                format!("batch-star-stale-{suffix}"),
                json!({
                    "provider": "gmail",
                    "label_ids": ["INBOX"],
                    "subject": "Star reconciliation",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Star reconciliation body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"projection_core_test"})),
        )
        .await
        .expect("stale Gmail observation");
    let preserved = project_raw_email_message(&message_store, &stale_provider_raw)
        .await
        .expect("project stale Gmail observation");
    assert_eq!(preserved.message_metadata["starred"], json!(true));
    assert_eq!(
        preserved.message_metadata["starred_origin"],
        json!("local_user")
    );

    let acknowledged_provider_raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-star-acknowledged-{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:star-acknowledged-{suffix}"),
                format!("batch-star-acknowledged-{suffix}"),
                json!({
                    "provider": "gmail",
                    "label_ids": ["INBOX", "STARRED"],
                    "subject": "Star reconciliation",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Star reconciliation body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"projection_core_test"})),
        )
        .await
        .expect("acknowledged Gmail observation");
    let acknowledged = project_raw_email_message(&message_store, &acknowledged_provider_raw)
        .await
        .expect("project acknowledged Gmail observation");
    assert_eq!(acknowledged.message_metadata["starred"], json!(true));
    assert_eq!(
        acknowledged.message_metadata["starred_origin"],
        json!("provider_observed")
    );

    let reconciliation_status: String = sqlx::query_scalar(
        "SELECT reconciliation_status FROM communication_provider_commands WHERE account_id = $1 AND command_kind = 'star'",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("star provider command reconciliation status");
    assert_eq!(reconciliation_status, "observed");

    BulkMessageActionStore::new(pool.clone())
        .apply(
            vec![acknowledged.message_id.clone()],
            BulkMessageAction::Unstar,
        )
        .await
        .expect("record local unstar intent");
    let stale_unstar_raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-unstar-stale-{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:unstar-stale-{suffix}"),
                format!("batch-unstar-stale-{suffix}"),
                json!({
                    "provider": "gmail",
                    "label_ids": ["INBOX", "STARRED"],
                    "subject": "Star reconciliation",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Star reconciliation body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"projection_core_test"})),
        )
        .await
        .expect("stale Gmail unstar observation");
    let unstar_preserved = project_raw_email_message(&message_store, &stale_unstar_raw)
        .await
        .expect("project stale Gmail unstar observation");
    assert_eq!(unstar_preserved.message_metadata["starred"], json!(false));
    assert_eq!(
        unstar_preserved.message_metadata["starred_origin"],
        json!("local_user")
    );

    let unstar_acknowledged_raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-unstar-acknowledged-{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:unstar-acknowledged-{suffix}"),
                format!("batch-unstar-acknowledged-{suffix}"),
                json!({
                    "provider": "gmail",
                    "label_ids": ["INBOX"],
                    "subject": "Star reconciliation",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Star reconciliation body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"projection_core_test"})),
        )
        .await
        .expect("acknowledged Gmail unstar observation");
    let unstar_acknowledged = project_raw_email_message(&message_store, &unstar_acknowledged_raw)
        .await
        .expect("project acknowledged Gmail unstar observation");
    assert_eq!(
        unstar_acknowledged.message_metadata["starred"],
        json!(false)
    );
    assert_eq!(
        unstar_acknowledged.message_metadata["starred_origin"],
        json!("provider_observed")
    );

    let unstar_reconciliation_status: String = sqlx::query_scalar(
        "SELECT reconciliation_status FROM communication_provider_commands WHERE account_id = $1 AND command_kind = 'unstar'",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("unstar provider command reconciliation status");
    assert_eq!(unstar_reconciliation_status, "observed");
}

#[tokio::test]
async fn message_projection_links_gmail_sent_record_to_matching_outbox_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("Gmail sent outbox correlation").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_sent_correlation_{suffix}");
    let provider_record_id = format!("gmail-sent-message-{suffix}");
    let outbox_id = format!("outbox-sent-correlation-{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection Gmail sent correlation",
        format!("projection-sent-correlation-{suffix}@example.com"),
    )
    .await;

    let outbox_store = CommunicationOutboxStore::new(pool);
    outbox_store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: outbox_id.clone(),
            account_id: account_id.clone(),
            draft_id: None,
            to_recipients: vec!["bob@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Sent correlation subject".to_owned(),
            body_text: "Sent correlation body".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Queued,
            scheduled_send_at: None,
            undo_deadline_at: None,
            metadata: json!({}),
        })
        .await
        .expect("enqueue outbox item");
    outbox_store
        .claim_due(Utc::now(), 1)
        .await
        .expect("claim outbox item");
    outbox_store
        .mark_sent(
            &outbox_id,
            Utc::now(),
            &OutboxSendReceipt {
                provider_message_id: provider_record_id.clone(),
                accepted_recipients: vec!["bob@example.com".to_owned()],
            },
        )
        .await
        .expect("mark outbox item sent");

    let raw_record_id = format!("raw_message_sent_correlation_{suffix}");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "provider": "gmail",
                    "label_ids": ["SENT"],
                    "subject": "Sent correlation subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Sent correlation body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record Gmail sent raw message");

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project Gmail sent message");

    assert_eq!(projected.delivery_state, "sent");
    assert_eq!(projected.message_metadata["outbox_id"], json!(outbox_id));
}

#[tokio::test]
async fn message_projection_projects_gmail_label_folder_membership_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("Gmail label folder membership").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_gmail_folder_mapping_{suffix}");
    let provider_record_id = format!("gmail-folder-message-{suffix}");
    store_provider_account(
        &communication_store,
        &account_id,
        "Projection Gmail folder mapping",
        format!("projection-gmail-folder-mapping-{suffix}@example.com"),
    )
    .await;

    let folder = CommunicationFolderStore::new(pool.clone())
        .create(NewCommunicationFolder {
            folder_id: None,
            account_id: Some(account_id.clone()),
            name: "Provider projects".to_owned(),
            description: None,
            color: None,
            sort_order: None,
        })
        .await
        .expect("create local folder");
    let resource_store = MailProviderResourceStore::new(pool.clone());
    let resource = resource_store
        .upsert_discovered(&NewMailProviderResource::new(
            &account_id,
            MailProviderResourceKind::Label,
            "Label_ProviderProjects",
            "Provider projects",
        ))
        .await
        .expect("discover provider label");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_gmail_folder_present_{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:gmail-folder-present-{suffix}"),
                format!("batch_gmail_folder_present_{suffix}"),
                json!({
                    "provider": "gmail",
                    "label_ids": ["Label_ProviderProjects"],
                    "subject": "Provider folder subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Provider folder body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record labeled Gmail raw message");
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project labeled Gmail message");

    resource_store
        .set_manual_mapping(
            &resource.mapping_id,
            &MailProviderResourceMappingUpdate {
                semantic_role: None,
                local_folder_id: Some(folder.folder_id.clone()),
            },
        )
        .await
        .expect("bind provider label after projection");

    let membership_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM communication_folder_messages WHERE folder_id = $1 AND message_id = $2",
    )
    .bind(&folder.folder_id)
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("load derived folder membership");
    assert_eq!(membership_count, 1);

    let raw_without_label = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_gmail_folder_removed_{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:gmail-folder-removed-{suffix}"),
                format!("batch_gmail_folder_removed_{suffix}"),
                json!({
                    "provider": "gmail",
                    "label_ids": [],
                    "subject": "Provider folder subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Provider folder body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record unlabeled Gmail raw message");
    let reprojected = project_raw_email_message(&message_store, &raw_without_label)
        .await
        .expect("reproject Gmail message without label");
    assert_eq!(reprojected.message_metadata["label_ids"], json!([]));

    let membership_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM communication_folder_messages WHERE folder_id = $1 AND message_id = $2",
    )
    .bind(&folder.folder_id)
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("load reconciled folder membership");
    assert_eq!(membership_count, 0);
}

#[tokio::test]
async fn message_projection_projects_imap_mailbox_folder_membership_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("IMAP mailbox folder membership").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_imap_folder_mapping_{suffix}");
    let mailbox = "Projects/2026";
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Projection IMAP folder mapping",
            format!("projection-imap-folder-mapping-{suffix}@example.com"),
        ))
        .await
        .expect("store IMAP provider account");

    let folder = CommunicationFolderStore::new(pool.clone())
        .create(NewCommunicationFolder {
            folder_id: None,
            account_id: Some(account_id.clone()),
            name: "Projects".to_owned(),
            description: None,
            color: None,
            sort_order: None,
        })
        .await
        .expect("create local folder");
    let resource_store = MailProviderResourceStore::new(pool.clone());
    let resource = resource_store
        .upsert_discovered(&NewMailProviderResource::new(
            &account_id,
            MailProviderResourceKind::Folder,
            mailbox,
            "Projects",
        ))
        .await
        .expect("discover provider mailbox");
    resource_store
        .set_manual_mapping(
            &resource.mapping_id,
            &MailProviderResourceMappingUpdate {
                semantic_role: None,
                local_folder_id: Some(folder.folder_id.clone()),
            },
        )
        .await
        .expect("bind provider mailbox to local folder");

    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_imap_folder_present_{suffix}"),
                &account_id,
                "email_message",
                format!("imap:Projects%2F2026:{suffix}"),
                format!("sha256:imap-folder-present-{suffix}"),
                format!("batch_imap_folder_present_{suffix}"),
                json!({
                    "provider": "imap",
                    "transport": "imap",
                    "mailbox": mailbox,
                    "subject": "Provider folder subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Provider folder body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record IMAP raw message");
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project IMAP message");

    let membership_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM communication_folder_messages WHERE folder_id = $1 AND message_id = $2",
    )
    .bind(&folder.folder_id)
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("load derived IMAP folder membership");
    assert_eq!(membership_count, 1);
}

#[tokio::test]
async fn message_projection_scopes_imap_uid_to_uid_validity_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("IMAP UIDVALIDITY message identity").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_imap_uid_validity_{suffix}");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Projection IMAP UIDVALIDITY",
            format!("projection-imap-uid-validity-{suffix}@example.com"),
        ))
        .await
        .expect("store IMAP provider account");

    let raw_for_epoch = |raw_record_id: String, uid_validity: u32| {
        NewRawCommunicationRecord::new(
            raw_record_id,
            &account_id,
            "email_message",
            "42",
            format!("sha256:imap-uid-validity-{suffix}-{uid_validity}"),
            format!("batch_imap_uid_validity_{suffix}_{uid_validity}"),
            json!({
                "provider": "imap",
                "transport": "imap",
                "mailbox": "INBOX",
                "uid": 42,
                "uid_validity": uid_validity,
                "subject": "UIDVALIDITY subject",
                "from": "alice@example.com",
                "to": ["bob@example.com"],
                "body_text": "UIDVALIDITY body"
            }),
        )
        .occurred_at(Utc::now())
        .provenance(json!({"source":"fixture_email"}))
    };
    let first_raw = communication_store
        .record_raw_source(&raw_for_epoch(
            format!("raw_imap_uid_validity_old_{suffix}"),
            7,
        ))
        .await
        .expect("record first UIDVALIDITY epoch");
    let second_raw = communication_store
        .record_raw_source(&raw_for_epoch(
            format!("raw_imap_uid_validity_new_{suffix}"),
            8,
        ))
        .await
        .expect("record second UIDVALIDITY epoch");

    assert_ne!(first_raw.raw_record_id, second_raw.raw_record_id);
    assert_eq!(first_raw.payload["uid_validity"], json!(7));
    assert_eq!(second_raw.payload["uid_validity"], json!(8));

    let first = project_raw_email_message(&message_store, &first_raw)
        .await
        .expect("project first UIDVALIDITY epoch");
    let second = project_raw_email_message(&message_store, &second_raw)
        .await
        .expect("project second UIDVALIDITY epoch");

    assert_ne!(first.message_id, second.message_id);
    assert_eq!(first.provider_record_id, "imap:v2:imap:INBOX:7:42");
    assert_eq!(second.provider_record_id, "imap:v2:imap:INBOX:8:42");
    let message_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM communication_messages WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("count IMAP UIDVALIDITY messages");
    assert_eq!(message_count, 2);
}

#[tokio::test]
async fn provider_folder_mapping_reconciles_existing_imap_messages_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("late IMAP folder mapping").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_imap_late_folder_mapping_{suffix}");
    let mailbox = "Projects/2026";
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Projection IMAP late folder mapping",
            format!("projection-imap-late-folder-mapping-{suffix}@example.com"),
        ))
        .await
        .expect("store IMAP provider account");

    let resource_store = MailProviderResourceStore::new(pool.clone());
    let resource = resource_store
        .upsert_discovered(&NewMailProviderResource::new(
            &account_id,
            MailProviderResourceKind::Folder,
            mailbox,
            "Projects",
        ))
        .await
        .expect("discover provider mailbox");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_imap_late_folder_mapping_{suffix}"),
                &account_id,
                "email_message",
                format!("imap:Projects%2F2026:{suffix}"),
                format!("sha256:imap-late-folder-mapping-{suffix}"),
                format!("batch_imap_late_folder_mapping_{suffix}"),
                json!({
                    "provider": "imap",
                    "transport": "imap",
                    "mailbox": mailbox,
                    "subject": "Provider folder subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Provider folder body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record IMAP raw message");
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project unmapped IMAP message");

    let folder = CommunicationFolderStore::new(pool.clone())
        .create(NewCommunicationFolder {
            folder_id: None,
            account_id: Some(account_id.clone()),
            name: "Projects".to_owned(),
            description: None,
            color: None,
            sort_order: None,
        })
        .await
        .expect("create local folder");
    resource_store
        .set_manual_mapping(
            &resource.mapping_id,
            &MailProviderResourceMappingUpdate {
                semantic_role: None,
                local_folder_id: Some(folder.folder_id.clone()),
            },
        )
        .await
        .expect("bind provider mailbox after projection");

    let membership_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM communication_folder_messages WHERE folder_id = $1 AND message_id = $2",
    )
    .bind(&folder.folder_id)
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("load mapped folder membership");
    assert_eq!(membership_count, 1);

    resource_store
        .set_manual_mapping(
            &resource.mapping_id,
            &MailProviderResourceMappingUpdate {
                semantic_role: None,
                local_folder_id: None,
            },
        )
        .await
        .expect("unbind provider mailbox");

    let membership_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM communication_folder_messages WHERE folder_id = $1 AND message_id = $2",
    )
    .bind(&folder.folder_id)
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("load removed provider-derived membership");
    assert_eq!(membership_count, 0);
}

#[tokio::test]
async fn manual_folder_copy_keeps_membership_after_provider_mapping_is_removed_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("manual ownership over provider folder mapping").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_manual_folder_ownership_{suffix}");
    let mailbox = "Projects/2026";
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Projection IMAP manual folder ownership",
            format!("projection-imap-manual-folder-ownership-{suffix}@example.com"),
        ))
        .await
        .expect("store IMAP provider account");

    let folder_store = CommunicationFolderStore::new(pool.clone());
    let folder = folder_store
        .create(NewCommunicationFolder {
            folder_id: None,
            account_id: Some(account_id.clone()),
            name: "Projects".to_owned(),
            description: None,
            color: None,
            sort_order: None,
        })
        .await
        .expect("create local folder");
    let resource_store = MailProviderResourceStore::new(pool.clone());
    let resource = resource_store
        .upsert_discovered(&NewMailProviderResource::new(
            &account_id,
            MailProviderResourceKind::Folder,
            mailbox,
            "Projects",
        ))
        .await
        .expect("discover provider mailbox");
    resource_store
        .set_manual_mapping(
            &resource.mapping_id,
            &MailProviderResourceMappingUpdate {
                semantic_role: None,
                local_folder_id: Some(folder.folder_id.clone()),
            },
        )
        .await
        .expect("bind provider mailbox");

    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_imap_manual_folder_ownership_{suffix}"),
                &account_id,
                "email_message",
                format!("imap:Projects%2F2026:{suffix}"),
                format!("sha256:imap-manual-folder-ownership-{suffix}"),
                format!("batch_imap_manual_folder_ownership_{suffix}"),
                json!({
                    "provider": "imap",
                    "transport": "imap",
                    "mailbox": mailbox,
                    "subject": "Provider folder subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Provider folder body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record IMAP raw message");
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project mapped IMAP message");

    folder_store
        .copy_message(&folder.folder_id, &projected.message_id)
        .await
        .expect("copy manually into mapped local folder")
        .expect("message and folder exist");
    resource_store
        .set_manual_mapping(
            &resource.mapping_id,
            &MailProviderResourceMappingUpdate {
                semantic_role: None,
                local_folder_id: None,
            },
        )
        .await
        .expect("unbind provider mailbox");

    let membership_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM communication_folder_messages WHERE folder_id = $1 AND message_id = $2",
    )
    .bind(&folder.folder_id)
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("load manually owned folder membership");
    assert_eq!(membership_count, 1);
}

#[tokio::test]
async fn message_projection_marks_imap_sent_only_from_semantic_folder_mapping_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("IMAP sent folder mapping").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_imap_sent_mapping_{suffix}");
    let mailbox = "Sent Messages";

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Projection IMAP sent mapping",
            format!("projection-imap-sent-mapping-{suffix}@example.com"),
        ))
        .await
        .expect("store IMAP provider account");

    let raw_record_id = format!("raw_message_imap_sent_mapping_{suffix}");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                format!("imap:Sent%20Messages:{suffix}"),
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "provider": "imap",
                    "transport": "imap",
                    "mailbox": mailbox,
                    "subject": "IMAP sent mapping subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "IMAP sent mapping body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record IMAP sent raw message");

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project IMAP sent message");

    assert_eq!(projected.delivery_state, "received");

    MailProviderResourceStore::new(pool.clone())
        .upsert_discovered(
            &NewMailProviderResource::new(
                &account_id,
                MailProviderResourceKind::Folder,
                mailbox,
                mailbox,
            )
            .semantic_role(MailProviderSemanticRole::Sent),
        )
        .await
        .expect("store sent mailbox mapping");

    let delivery_state = sqlx::query_scalar::<_, String>(
        "SELECT delivery_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("load reconciled IMAP delivery state");

    assert_eq!(delivery_state, "sent");
}

#[tokio::test]
async fn message_projection_links_imap_sent_record_to_outbox_by_rfc822_message_id_against_postgres()
{
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("IMAP sent outbox correlation by Message-ID").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_imap_sent_outbox_correlation_{suffix}");
    let outbox_id = format!("outbox-imap-sent-correlation-{suffix}");
    let mailbox = "Sent Messages";
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Projection IMAP sent outbox correlation",
            format!("projection-imap-sent-correlation-{suffix}@example.com"),
        ))
        .await
        .expect("store IMAP provider account");

    let resource_store = MailProviderResourceStore::new(pool.clone());
    let resource = resource_store
        .upsert_discovered(
            &NewMailProviderResource::new(
                &account_id,
                MailProviderResourceKind::Folder,
                mailbox,
                "Sent",
            )
            .semantic_role(MailProviderSemanticRole::Sent),
        )
        .await
        .expect("discover sent mailbox");
    resource_store
        .set_manual_mapping(
            &resource.mapping_id,
            &MailProviderResourceMappingUpdate {
                semantic_role: Some(MailProviderSemanticRole::Sent),
                local_folder_id: None,
            },
        )
        .await
        .expect("confirm sent mailbox mapping");

    let outbox_store = CommunicationOutboxStore::new(pool.clone());
    let outbox = outbox_store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: outbox_id.clone(),
            account_id: account_id.clone(),
            draft_id: None,
            to_recipients: vec!["bob@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Sent correlation subject".to_owned(),
            body_text: "Sent correlation body".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Queued,
            scheduled_send_at: None,
            undo_deadline_at: None,
            metadata: json!({}),
        })
        .await
        .expect("enqueue outbox item");
    let rfc822_message_id = outbox.metadata["rfc822_message_id"]
        .as_str()
        .expect("outbox RFC822 Message-ID")
        .to_owned();
    outbox_store
        .claim_due(Utc::now(), 1)
        .await
        .expect("claim outbox item");
    outbox_store
        .mark_sent(
            &outbox_id,
            Utc::now(),
            &OutboxSendReceipt {
                provider_message_id: format!("smtp-accepted-{suffix}"),
                accepted_recipients: vec!["bob@example.com".to_owned()],
            },
        )
        .await
        .expect("mark outbox item sent");

    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_imap_sent_outbox_correlation_{suffix}"),
                &account_id,
                "email_message",
                format!("imap:v2:imap:Sent Messages:7:{suffix}"),
                format!("sha256:imap-sent-outbox-correlation-{suffix}"),
                format!("batch_imap_sent_outbox_correlation_{suffix}"),
                json!({
                    "provider": "imap",
                    "transport": "imap",
                    "mailbox": mailbox,
                    "uid": 42,
                    "uid_validity": 7,
                    "rfc822_message_id": rfc822_message_id,
                    "subject": "Sent correlation subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Sent correlation body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record IMAP sent raw message");
    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project IMAP sent message");

    assert_eq!(projected.delivery_state, "sent");
    assert_eq!(projected.message_metadata["outbox_id"], json!(outbox_id));
}

#[tokio::test]
async fn message_projection_extracts_canonical_fields_from_raw_blob_against_postgres() {
    let Some((_context, _, communication_store, message_store)) =
        live_projection_context("message raw blob projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_blob_projection_{suffix}");
    let raw_record_id = format!("raw_message_blob_projection_{suffix}");
    let provider_record_id = format!("provider-message-blob-{suffix}");
    let blob_root = tempfile::tempdir().expect("blob root");
    let blob_store = LocalCommunicationBlobStore::new(blob_root.path());
    let local_blob = blob_store
        .put_blob(
            b"Subject: Real MIME\r\nFrom: Alice <alice@example.com>\r\nTo: Bob <bob@example.com>\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\nHello=20from=20real=20mail.",
        )
        .await
        .expect("write raw mail blob");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection raw blob",
        format!("projection-raw-blob-{suffix}@example.com"),
    )
    .await;
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "provider": "imap",
                    "raw_blob_storage_kind": local_blob.storage_kind,
                    "raw_blob_storage_path": local_blob.storage_path,
                    "raw_blob_sha256": local_blob.sha256,
                    "raw_blob_size_bytes": local_blob.size_bytes
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"email_provider_sync"})),
        )
        .await
        .expect("record raw blob message");

    let projected = project_raw_email_message_from_blob(&message_store, &blob_store, &raw)
        .await
        .expect("project message from raw blob");

    assert_eq!(projected.account_id, account_id);
    assert_eq!(projected.observation_id, raw.observation_id);
    assert_eq!(projected.provider_record_id, provider_record_id);
    assert_eq!(projected.subject, "Real MIME");
    assert_eq!(projected.sender, "Alice <alice@example.com>");
    assert_eq!(
        projected.recipients,
        vec!["Bob <bob@example.com>".to_owned()]
    );
    assert_eq!(projected.body_text, "Hello from real mail.");
}

#[tokio::test]
async fn message_projection_distinguishes_delimiter_bearing_identities_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("delimiter-bearing message projection identities").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let base_account_id = format!("acct_message_identity_{suffix}");
    let left_account_id = format!("{base_account_id}:left");

    store_provider_account(
        &communication_store,
        &base_account_id,
        "Projection identity base",
        format!("projection-identity-base-{suffix}@example.com"),
    )
    .await;
    store_provider_account(
        &communication_store,
        &left_account_id,
        "Projection identity left",
        format!("projection-identity-left-{suffix}@example.com"),
    )
    .await;

    let base_raw = record_raw_email_message(
        &communication_store,
        &base_account_id,
        &format!("raw_message_identity_base_{suffix}"),
        "left:right",
        "Delimiter subject base",
        "Delimiter body base",
    )
    .await;
    let left_raw = record_raw_email_message(
        &communication_store,
        &left_account_id,
        &format!("raw_message_identity_left_{suffix}"),
        "right",
        "Delimiter subject left",
        "Delimiter body left",
    )
    .await;

    let base_projected = project_raw_email_message(&message_store, &base_raw)
        .await
        .expect("project base delimiter message");
    let left_projected = project_raw_email_message(&message_store, &left_raw)
        .await
        .expect("project left delimiter message");

    assert_ne!(base_projected.message_id, left_projected.message_id);
    assert_eq!(base_projected.account_id, base_account_id);
    assert_eq!(base_projected.provider_record_id, "left:right");
    assert_eq!(left_projected.account_id, left_account_id);
    assert_eq!(left_projected.provider_record_id, "right");

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_messages
        WHERE account_id IN ($1, $2)
        "#,
    )
    .bind(&base_projected.account_id)
    .bind(&left_projected.account_id)
    .fetch_one(&pool)
    .await
    .expect("projected delimiter message count");
    assert_eq!(count, 2);
}

#[tokio::test]
async fn message_projection_reprojects_same_raw_record_idempotently_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("idempotent message projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_idempotent_{suffix}");
    let provider_record_id = format!("provider-message-idempotent-{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection idempotent Gmail",
        format!("projection-idempotent-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_message_idempotent_{suffix}"),
        &provider_record_id,
        "Idempotent subject",
        "Idempotent body",
    )
    .await;

    let first = project_raw_email_message(&message_store, &raw)
        .await
        .expect("first message projection");
    let second = project_raw_email_message(&message_store, &raw)
        .await
        .expect("second message projection");

    assert_eq!(second.message_id, first.message_id);
    assert_eq!(second.raw_record_id, first.raw_record_id);
    assert_eq!(second.account_id, first.account_id);
    assert_eq!(second.provider_record_id, first.provider_record_id);

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_messages
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("idempotent projected message count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn message_projection_derives_message_id_for_direct_upsert_against_postgres() {
    let Some((_context, pool, communication_store, message_store)) =
        live_projection_context("direct message upsert identity derivation").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_message_direct_identity_{suffix}");
    let provider_record_id = format!("provider-message-direct-identity-{suffix}");
    let arbitrary_message_id = format!("not-canonical-message-id-{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Projection direct identity",
        format!("projection-direct-identity-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &format!("raw_message_direct_identity_{suffix}"),
        &provider_record_id,
        "Direct identity subject",
        "Direct identity body",
    )
    .await;
    let message = NewProjectedMessage {
        message_id: arbitrary_message_id.clone(),
        raw_record_id: raw.raw_record_id,
        account_id,
        provider_record_id,
        subject: "Direct identity subject".to_owned(),
        sender: "alice@example.com".to_owned(),
        recipients: vec!["bob@example.com".to_owned()],
        body_text: "Direct identity body".to_owned(),
        occurred_at: raw.occurred_at,
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some("alice@example.com".to_owned()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };

    let projected = message_store
        .upsert_message(&message)
        .await
        .expect("direct upsert derives canonical message ID");

    assert_ne!(projected.message_id, arbitrary_message_id);
    assert!(projected.message_id.starts_with("msg:v1:"));
    assert_eq!(projected.observation_id, raw.observation_id);

    let arbitrary_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM communication_messages WHERE message_id = $1",
    )
    .bind(&arbitrary_message_id)
    .fetch_one(&pool)
    .await
    .expect("arbitrary message ID count");
    assert_eq!(arbitrary_count, 0);
}

#[test]
fn email_projection_upsert_uses_projected_metadata_bindings() {
    let source = include_str!("../../src/domains/communications/messages/store/upsert.rs");

    assert!(source.contains(".bind(&message.delivery_state)"));
    assert!(source.contains(".bind(&message.message_metadata)"));
    assert!(!source.contains("'received',\n                '{}'::jsonb"));
}
