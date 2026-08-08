use makosh_backend_testkit::context::TestContext;
use makosh_backend_testkit::factories::persona::PersonaFactory;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::email_sync::{EmailSyncBatch, FetchedCommunicationSourceMessage};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use chrono::{TimeZone, Utc};
use serde_json::json;
use sqlx::Row;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::storage::port::LocalBlobPort;

use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::email_sync_pipeline::service::project_email_sync_batch_with_mail_blobs;

#[tokio::test]
async fn email_sync_pipeline_records_raw_blob_and_links_confirmed_message_participants_against_postgres()
 {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_sync_pipeline_{suffix}");
    let provider_record_id = format!("sync-pipeline-message-{suffix}");
    let sender_domain = format!("acme-{suffix}.test");
    let recipient_domain = format!("client-{suffix}.test");
    let sender_email = format!("sender-{suffix}@{sender_domain}");
    let recipient_email = format!("recipient-{suffix}@{recipient_domain}");
    let blob_root = tempfile::tempdir().expect("mail blob root");
    let blob_store = LocalBlobPort::new(blob_root.path());

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Sync pipeline IMAP",
            format!("sync-pipeline-{suffix}@example.net"),
        ))
        .await
        .expect("store provider account");
    PersonaFactory::new(&pool)
        .with_name("Confirmed Sender")
        .with_email(sender_email.clone())
        .create()
        .await
        .expect("confirmed sender person");

    let raw_rfc822 = format!(
        "Subject: Sync Pipeline\r\n\
         From: Sender <{sender_email}>\r\n\
         To: Recipient <{recipient_email}>\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         Cached message body.\r\n"
    );
    let raw_rfc822_base64 = base64::engine::general_purpose::STANDARD.encode(raw_rfc822);
    let batch = EmailSyncBatch {
        provider_kind: CommunicationProviderKind::Imap,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({"provider": "imap", "last_seen_uid": 88})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: provider_record_id.clone(),
            source_fingerprint: format!("sha256:sync-pipeline-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_000_000).single(),
            payload: json!({
                "provider": "imap",
                "uid": 88,
                "raw_rfc822_base64": raw_rfc822_base64
            }),
        }],
    };

    let report = project_email_sync_batch_with_mail_blobs(
        pool.clone(),
        &communication_store,
        &blob_store,
        &account_id,
        format!("sync-pipeline-batch-{suffix}"),
        &batch,
    )
    .await
    .expect("project email sync batch");

    assert_eq!(report.imported_records, 1);
    assert_eq!(report.raw_blobs_upserted, 1);
    assert_eq!(report.projected_messages, 1);
    assert_eq!(report.attachment_blobs_upserted, 0);
    assert_eq!(report.attachments_extracted, 0);
    assert_eq!(report.attachments_not_scanned, 0);
    assert_eq!(report.upserted_personas, 0);
    assert_eq!(report.upserted_persona_identities, 0);
    assert_eq!(report.upserted_message_participants, 1);
    assert_eq!(report.upserted_relationship_events, 1);
    assert_eq!(report.upserted_organizations, 1);
    assert_eq!(report.upserted_organization_persona_links, 1);

    let projected = sqlx::query(
        r#"
        SELECT message_id, observation_id, subject, sender, recipients, body_text
        FROM communication_messages
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("projected message");
    let message_id: String = projected.try_get("message_id").expect("message id");
    let observation_id: String = projected.try_get("observation_id").expect("observation id");
    let subject: String = projected.try_get("subject").expect("subject");
    let sender: String = projected.try_get("sender").expect("sender");
    let recipients: serde_json::Value = projected.try_get("recipients").expect("recipients");
    let body_text: String = projected.try_get("body_text").expect("body_text");
    assert_eq!(subject, "Sync Pipeline");
    assert_eq!(sender, format!("Sender <{sender_email}>"));
    assert_eq!(body_text, "Cached message body.");
    assert_eq!(
        recipients,
        json!([format!("Recipient <{recipient_email}>")])
    );

    let accepted_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'signal.accepted.mail.message'
          AND source ->> 'account_id' = $1
          AND subject ->> 'provider_record_id' = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("accepted mail signal count");
    assert_eq!(accepted_signal_count, 1);

    let identity_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM persona_identities
        WHERE identity_type = 'email'
          AND identity_value = ANY($1)
          AND source = 'email_sync'
          AND status = 'active'
        "#,
    )
    .bind(vec![sender_email.as_str(), recipient_email.as_str()])
    .fetch_one(&pool)
    .await
    .expect("person email identities");
    assert_eq!(identity_count, 0);

    let persona_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'personas'
          AND entity_kind = 'persona'
          AND relationship_kind = 'email_sync_projection'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("persona observation links");
    assert_eq!(persona_observation_link_count, 0);

    let identity_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'personas'
          AND entity_kind = 'identity'
          AND relationship_kind = 'email_sync_projection'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("identity observation links");
    assert_eq!(identity_observation_link_count, 0);

    let participant_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM communication_message_participants
        WHERE message_id = (
            SELECT message_id
            FROM communication_messages
            WHERE account_id = $1 AND provider_record_id = $2
        )
          AND email_address = $3
          AND role = 'sender'
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .bind(&sender_email)
    .fetch_one(&pool)
    .await
    .expect("message participants");
    assert_eq!(participant_count, 1);

    let participant_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'communications'
          AND entity_kind = 'message_participant'
          AND relationship_kind = 'email_sync_participant'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("message participant observation links");
    assert_eq!(participant_observation_link_count, 1);

    let relationship_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM relationship_events
        WHERE related_entity_kind = 'communication_message'
          AND related_entity_id = (
            SELECT message_id
            FROM communication_messages
            WHERE account_id = $1 AND provider_record_id = $2
          )
          AND event_type IN ('email_sent', 'email_received')
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("relationship events");
    assert_eq!(relationship_count, 1);

    let relationship_event_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'personas'
          AND entity_kind = 'relationship_event'
          AND relationship_kind = 'email_sync_relationship_event'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("relationship event observation links");
    assert_eq!(relationship_event_observation_link_count, 1);

    let organization_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM organization_persona_links link
        JOIN organization_domains domain ON domain.organization_id = link.organization_id
        JOIN persona_identities identity ON identity.persona_id = link.persona_id
        WHERE domain.domain = ANY($1)
          AND identity.identity_value = $2
        "#,
    )
    .bind(vec![sender_domain.as_str()])
    .bind(&sender_email)
    .fetch_one(&pool)
    .await
    .expect("organization-person links");
    assert_eq!(organization_link_count, 1);

    let organization_relationship_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM relationships relationship
        JOIN relationship_evidence evidence
          ON evidence.relationship_id = relationship.relationship_id
        JOIN organization_domains domain
          ON domain.organization_id = relationship.target_entity_id
        JOIN persona_identities identity
          ON identity.persona_id = relationship.source_entity_id
        WHERE relationship.source_entity_kind = 'persona'
          AND relationship.target_entity_kind = 'organization'
          AND relationship.relationship_type = 'member_of'
          AND relationship.review_state = 'system_accepted'
          AND relationship.metadata->>'compatibility_table' = 'organization_persona_links'
          AND relationship.metadata->>'source' = 'email_sync'
          AND evidence.source_kind = 'communication'
          AND evidence.source_id = $1
          AND domain.domain = $2
          AND identity.identity_value = $3
        "#,
    )
    .bind(&message_id)
    .bind(&sender_domain)
    .bind(&sender_email)
    .fetch_one(&pool)
    .await
    .expect("organization relationships");
    assert_eq!(organization_relationship_count, 1);

    let organization_projection_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'organizations'
          AND entity_kind = 'organization'
          AND relationship_kind = 'email_sync_projection'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("organization observation links");
    assert_eq!(organization_projection_observation_link_count, 1);

    let organization_domain_projection_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'organizations'
          AND entity_kind = 'organization_domain'
          AND relationship_kind = 'email_sync_projection'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("organization domain observation links");
    assert_eq!(organization_domain_projection_observation_link_count, 1);

    let organization_identity_projection_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'organizations'
          AND entity_kind = 'organization_identity'
          AND relationship_kind = 'email_sync_projection'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("organization identity observation links");
    assert_eq!(organization_identity_projection_observation_link_count, 1);
}

#[tokio::test]
async fn email_sync_pipeline_does_not_create_ai_candidates_directly_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_sync_candidates_{suffix}");
    let provider_record_id = format!("sync-candidates-message-{suffix}");
    let sender_email = format!("candidate-sender-{suffix}@example.net");
    let recipient_email = format!("candidate-recipient-{suffix}@example.net");
    let decision_title = format!("Use candidate refresh {suffix}");
    let decision_rationale = "communication ingestion must build context";
    let obligation_statement = format!("send the candidate refresh summary {suffix}");
    let blob_root = tempfile::tempdir().expect("mail blob root");
    let blob_store = LocalBlobPort::new(blob_root.path());

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Sync candidate IMAP",
            format!("sync-candidates-{suffix}@example.net"),
        ))
        .await
        .expect("store provider account");

    let raw_rfc822 = format!(
        "Subject: Candidate Pipeline\r\n\
         From: Sender <{sender_email}>\r\n\
         To: Recipient <{recipient_email}>\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         Decision: {decision_title} because {decision_rationale}.\r\n\
         I will {obligation_statement} by Friday 5pm.\r\n"
    );
    let raw_rfc822_base64 = base64::engine::general_purpose::STANDARD.encode(raw_rfc822);
    let batch = EmailSyncBatch {
        provider_kind: CommunicationProviderKind::Imap,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({"provider": "imap", "last_seen_uid": 90})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: provider_record_id.clone(),
            source_fingerprint: format!("sha256:sync-candidates-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_200_000).single(),
            payload: json!({
                "provider": "imap",
                "uid": 90,
                "raw_rfc822_base64": raw_rfc822_base64
            }),
        }],
    };

    let report = project_email_sync_batch_with_mail_blobs(
        pool.clone(),
        &communication_store,
        &blob_store,
        &account_id,
        format!("sync-candidates-batch-{suffix}"),
        &batch,
    )
    .await
    .expect("project email sync batch");

    assert_eq!(report.projected_messages, 1);
    assert_eq!(report.refreshed_decision_candidates, 0);
    assert_eq!(report.refreshed_knowledge_candidates, 0);
    assert_eq!(report.refreshed_task_candidates, 0);

    let message_id: String = sqlx::query_scalar(
        r#"
        SELECT message_id
        FROM communication_messages
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("projected message id");
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");

    let decision_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM decisions decision
        JOIN decision_impacted_entities impacted
          ON impacted.decision_id = decision.decision_id
        WHERE impacted.entity_kind = 'communication'
          AND impacted.entity_id = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("decision candidate count");
    assert_eq!(decision_count, 0);

    let mirrored_review_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM review_items
        WHERE review_item_id IN (
            SELECT review_item_id
            FROM review_item_evidence
            WHERE observation_id = $1
        )
          AND item_kind IN ('potential_decision', 'potential_task', 'knowledge_candidate')
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("mirrored review candidate count");
    assert_eq!(mirrored_review_count, 0);

    let task_candidate_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM task_candidates
        WHERE source_kind = 'observation'
          AND source_id = $1
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("task candidate count");
    assert_eq!(task_candidate_count, 0);

    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&message_id)
            .fetch_one(&pool)
            .await
            .expect("task count");
    let obligation_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM obligations WHERE statement = $1")
            .bind(&obligation_statement)
            .fetch_one(&pool)
            .await
            .expect("obligation count");
    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn email_sync_pipeline_extracts_attachment_metadata_with_initial_scan_status() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_sync_attachment_{suffix}");
    let provider_record_id = format!("sync-attachment-message-{suffix}");
    let blob_root = tempfile::tempdir().expect("mail blob root");
    let blob_store = LocalBlobPort::new(blob_root.path());

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Sync attachment IMAP",
            format!("sync-attachment-{suffix}@example.net"),
        ))
        .await
        .expect("store provider account");

    let raw_rfc822 = concat!(
        "Subject: Attachment Pipeline\r\n",
        "From: Sender <sender@example.invalid>\r\n",
        "To: Recipient <recipient@example.invalid>\r\n",
        "Content-Type: multipart/mixed; boundary=\"makosh-boundary\"\r\n",
        "\r\n",
        "--makosh-boundary\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "See attached cache fixture.\r\n",
        "--makosh-boundary\r\n",
        "Content-Type: text/plain; name=\"invoice.txt\"\r\n",
        "Content-Disposition: attachment; filename=\"invoice.txt\"\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "YXR0YWNobWVudCBieXRlcw==\r\n",
        "--makosh-boundary--\r\n"
    );
    let raw_rfc822_base64 = base64::engine::general_purpose::STANDARD.encode(raw_rfc822);
    let batch = EmailSyncBatch {
        provider_kind: CommunicationProviderKind::Imap,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({"provider": "imap", "last_seen_uid": 89})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: provider_record_id.clone(),
            source_fingerprint: format!("sha256:sync-attachment-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_100_000).single(),
            payload: json!({
                "provider": "imap",
                "uid": 89,
                "raw_rfc822_base64": raw_rfc822_base64
            }),
        }],
    };

    let report = project_email_sync_batch_with_mail_blobs(
        pool.clone(),
        &communication_store,
        &blob_store,
        &account_id,
        format!("sync-attachment-batch-{suffix}"),
        &batch,
    )
    .await
    .expect("project email sync batch");

    assert_eq!(report.imported_records, 1);
    assert_eq!(report.raw_blobs_upserted, 1);
    assert_eq!(report.projected_messages, 1);
    assert_eq!(report.attachment_blobs_upserted, 1);
    assert_eq!(report.attachments_extracted, 1);
    assert_eq!(report.attachments_not_scanned, 1);

    let attachment = sqlx::query(
        r#"
        SELECT
            a.filename,
            a.content_type,
            a.size_bytes,
            a.sha256,
            a.disposition,
            a.scan_status,
            a.scan_engine,
            a.scan_checked_at,
            a.scan_summary,
            a.scan_metadata,
            b.storage_kind,
            b.storage_path
        FROM communication_attachments a
        JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
        JOIN communication_messages m ON m.message_id = a.message_id
        WHERE m.account_id = $1
          AND m.provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("projected attachment metadata");

    let filename: Option<String> = attachment.try_get("filename").expect("filename");
    let content_type: String = attachment.try_get("content_type").expect("content_type");
    let size_bytes: i64 = attachment.try_get("size_bytes").expect("size_bytes");
    let sha256: String = attachment.try_get("sha256").expect("sha256");
    let disposition: String = attachment.try_get("disposition").expect("disposition");
    let scan_status: String = attachment.try_get("scan_status").expect("scan_status");
    let scan_engine: Option<String> = attachment.try_get("scan_engine").expect("scan_engine");
    let scan_checked_at: Option<chrono::DateTime<Utc>> = attachment
        .try_get("scan_checked_at")
        .expect("scan_checked_at");
    let scan_summary: Option<String> = attachment.try_get("scan_summary").expect("scan_summary");
    let scan_metadata: serde_json::Value =
        attachment.try_get("scan_metadata").expect("scan_metadata");
    let storage_kind: String = attachment.try_get("storage_kind").expect("storage_kind");
    let storage_path: String = attachment.try_get("storage_path").expect("storage_path");

    assert_eq!(filename.as_deref(), Some("invoice.txt"));
    assert_eq!(content_type, "text/plain");
    assert_eq!(size_bytes, 16);
    assert!(sha256.starts_with("sha256:"));
    assert_eq!(disposition, "attachment");
    assert_eq!(scan_status, "not_scanned");
    assert!(scan_engine.is_none());
    assert!(scan_checked_at.is_none());
    assert!(scan_summary.is_none());
    assert_eq!(scan_metadata, json!({}));
    assert_eq!(storage_kind, "local_fs");
    assert!(blob_root.path().join(storage_path).is_file());
}

#[tokio::test]
async fn email_sync_pipeline_marks_executable_attachment_payloads_malicious() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_sync_malicious_attachment_{suffix}");
    let provider_record_id = format!("sync-malicious-attachment-message-{suffix}");
    let blob_root = tempfile::tempdir().expect("mail blob root");
    let blob_store = LocalBlobPort::new(blob_root.path());

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Sync malicious attachment IMAP",
            format!("sync-malicious-attachment-{suffix}@example.net"),
        ))
        .await
        .expect("store provider account");

    let attachment_body =
        base64::engine::general_purpose::STANDARD.encode(b"MZ\x90\x00fake portable executable");
    let raw_rfc822 = format!(
        "Subject: Attachment Safety\r\n\
         From: Sender <sender@example.invalid>\r\n\
         To: Recipient <recipient@example.invalid>\r\n\
         Content-Type: multipart/mixed; boundary=\"makosh-boundary\"\r\n\
         \r\n\
         --makosh-boundary\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         See attached executable payload.\r\n\
         --makosh-boundary\r\n\
         Content-Type: application/pdf; name=\"invoice.pdf\"\r\n\
         Content-Disposition: attachment; filename=\"invoice.pdf\"\r\n\
         Content-Transfer-Encoding: base64\r\n\
         \r\n\
         {attachment_body}\r\n\
         --makosh-boundary--\r\n"
    );
    let raw_rfc822_base64 = base64::engine::general_purpose::STANDARD.encode(raw_rfc822);
    let batch = EmailSyncBatch {
        provider_kind: CommunicationProviderKind::Imap,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({"provider": "imap", "last_seen_uid": 90})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: provider_record_id.clone(),
            source_fingerprint: format!("sha256:sync-malicious-attachment-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_200_000).single(),
            payload: json!({
                "provider": "imap",
                "uid": 90,
                "raw_rfc822_base64": raw_rfc822_base64
            }),
        }],
    };

    let report = project_email_sync_batch_with_mail_blobs(
        pool.clone(),
        &communication_store,
        &blob_store,
        &account_id,
        format!("sync-malicious-attachment-batch-{suffix}"),
        &batch,
    )
    .await
    .expect("project email sync batch");

    assert_eq!(report.attachments_extracted, 1);
    assert_eq!(report.attachments_not_scanned, 0);

    let attachment = sqlx::query(
        r#"
        SELECT
            a.scan_status,
            a.scan_engine,
            a.scan_checked_at,
            a.scan_summary,
            a.scan_metadata
        FROM communication_attachments a
        JOIN communication_messages m ON m.message_id = a.message_id
        WHERE m.account_id = $1
          AND m.provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("projected attachment safety metadata");

    let scan_status: String = attachment.try_get("scan_status").expect("scan_status");
    let scan_engine: Option<String> = attachment.try_get("scan_engine").expect("scan_engine");
    let scan_checked_at: Option<chrono::DateTime<Utc>> = attachment
        .try_get("scan_checked_at")
        .expect("scan_checked_at");
    let scan_summary: Option<String> = attachment.try_get("scan_summary").expect("scan_summary");
    let scan_metadata: serde_json::Value =
        attachment.try_get("scan_metadata").expect("scan_metadata");

    assert_eq!(scan_status, "malicious");
    assert_eq!(scan_engine.as_deref(), Some("makosh_heuristic_v1"));
    assert!(scan_checked_at.is_some());
    assert_eq!(scan_summary.as_deref(), Some("Executable payload detected"));
    assert_eq!(scan_metadata["reasons"], json!(["executable_magic"]));
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
