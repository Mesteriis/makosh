use chrono::Utc;
use makosh_backend_testkit::containers::zulip::{ProvisionedZulipRealm, ZulipServer};
use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use std::env;
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use tokio::time::{Duration, Instant, sleep};

const LIVE_WAIT_LOG_INTERVAL: Duration = Duration::from_secs(5);

use makosh_communications_api::commands::{
    CommunicationProviderCommand, NewCommunicationProviderCommand,
};
use makosh_communications_api::evidence::NewIngestionCheckpoint;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_events_api::EventLogQuery;
use makosh_events_api::StoredEventEnvelope;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::application::zulip_attachment_download::ZulipAttachmentDownloadWorker;
use makosh_hub_backend::application::zulip_command_executor::ZulipCommandWorker;
use makosh_hub_backend::application::zulip_event_ingest::ZulipEventIngestWorker;
use makosh_hub_backend::application::zulip_provider_observation_reconciliation::reconcile_zulip_provider_observation_event;
use makosh_hub_backend::domains::communications::messages::models::ProjectedMessage;
use makosh_hub_backend::domains::communications::messages::provider_observation_projection::consume_accepted_signal_event;
use makosh_hub_backend::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use makosh_hub_backend::domains::communications::storage::models::{
    ImportedCommunicationAttachment, NewCommunicationAttachmentImport, NewCommunicationBlob,
};
use makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore;
use makosh_hub_backend::domains::signal_hub::store::SignalHubStore;
use makosh_hub_backend::domains::signal_hub::zulip::dispatch_zulip_raw_signal;
use makosh_provider_orchestration::observation_to_raw_communication_record;

use makosh_hub_backend::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use makosh_hub_backend::platform::events::bus::InMemoryEventBus;
use makosh_hub_backend::platform::secrets::models::{
    NewSecretReference, SecretKind, SecretStoreKind,
};
use makosh_hub_backend::platform::secrets::resolver::InMemorySecretResolver;
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::workflows::review_inbox::refresh_message_task_candidates_into_review;
use makosh_provider_zulip::client::{
    ZulipApiClient, ZulipClientConfig, ZulipReactionRequest, ZulipUpdateMessageRequest,
};
use makosh_provider_zulip::event_mapper::{
    ZulipEventMappingContext, map_zulip_event_to_observation,
};
use makosh_provider_zulip::models::ZulipEvent;

#[tokio::test]
#[ignore = "starts the real Zulip Docker compose stack; set MAKOSH_ZULIP_TESTCONTAINERS=1"]
async fn zulip_testcontainers_server_exercises_provider_surface() {
    if !ZulipServer::enabled() {
        eprintln!("skipping: MAKOSH_ZULIP_TESTCONTAINERS=1 is not set");
        return;
    }

    let server = ZulipServer::start()
        .await
        .expect("Zulip testcontainer stack should start");
    let realm = server
        .provision_test_realm()
        .await
        .expect("Zulip test realm should be provisioned");
    let bot_client = ZulipApiClient::new(
        ZulipClientConfig::new(&realm.base_url, &realm.bot_email, &realm.bot_api_key)
            .expect("valid bot Zulip config"),
    );

    let queue = bot_client
        .register_event_queue(&["message", "reaction", "update_message", "delete_message"])
        .await
        .expect("register Zulip event queue");
    assert_eq!(queue.result, "success");
    let mut cursor = ZulipEventCursor::new(queue.queue_id, queue.last_event_id);
    let trace = LiveМакошьTrace::new(&realm).await;

    let stream_message = bot_client
        .send_stream_message(
            &realm.stream_name,
            "live-provider-surface",
            "Надо подготовить Zulip факты до завтра.",
        )
        .await
        .expect("send Zulip stream message");
    let stream_message_id = stream_message.id.expect("stream message id");

    let stream_event = cursor
        .wait_for_message_event(&bot_client, stream_message_id)
        .await
        .expect("stream message should appear in Zulip event queue");
    let stream_projection = trace.record_event(stream_event).await;
    assert_eq!(stream_projection.channel_kind, "zulip");
    assert_eq!(
        stream_projection.subject,
        "makosh-lab / live-provider-surface"
    );
    trace.assert_task_review_candidate(&stream_projection).await;

    let direct_message = bot_client
        .send_direct_message_to_user_ids(
            &[realm.human_user_id],
            "Direct Zulip check from Макошь bot",
        )
        .await
        .expect("send Zulip direct message");
    let direct_message_id = direct_message.id.expect("direct message id");
    let direct_event = cursor
        .wait_for_message_event(&bot_client, direct_message_id)
        .await
        .expect("direct message should appear in Zulip event queue");
    let direct_projection = trace.record_event(direct_event).await;
    assert_eq!(direct_projection.channel_kind, "zulip");
    assert!(
        direct_projection.subject.starts_with("Direct /"),
        "direct message should preserve direct conversation shape, got {}",
        direct_projection.subject
    );

    let upload = bot_client
        .upload_file_bytes("makosh-fact.txt", b"zulip live fixture attachment".to_vec())
        .await
        .expect("upload Zulip file");
    assert!(
        upload.uri.starts_with("/user_uploads/"),
        "unexpected Zulip upload URI: {}",
        upload.uri
    );
    let downloaded = bot_client
        .download_user_upload(&upload.uri)
        .await
        .expect("download Zulip user upload");
    assert_eq!(downloaded.bytes, b"zulip live fixture attachment");

    let attachment_message = bot_client
        .send_stream_message(
            &realm.stream_name,
            "live-provider-attachment",
            &format!("[makosh-fact.txt]({})", upload.uri),
        )
        .await
        .expect("send Zulip attachment message");
    let attachment_message_id = attachment_message.id.expect("attachment message id");
    let attachment_event = cursor
        .wait_for_message_event(&bot_client, attachment_message_id)
        .await
        .expect("attachment message should appear in Zulip event queue");
    let attachment_projection = trace.record_event(attachment_event).await;
    assert_eq!(
        attachment_projection
            .message_metadata
            .get("attachment_state")
            .and_then(|state| state.get("bytes_state"))
            .and_then(Value::as_str),
        Some("not_transferred")
    );
    assert!(
        attachment_projection
            .message_metadata
            .get("attachments")
            .and_then(Value::as_array)
            .is_some_and(|attachments| !attachments.is_empty()),
        "attachment message should retain Zulip attachment metadata"
    );

    let reaction = ZulipReactionRequest::new("thumbs_up");
    bot_client
        .add_reaction(stream_message_id, &reaction)
        .await
        .expect("add Zulip reaction");
    let reaction_add_event = cursor
        .wait_for_event(&bot_client, "reaction add", |event| {
            event.event_type == "reaction"
                && event_i64(event, "message_id") == Some(stream_message_id)
                && event_str(event, "op") == Some("add")
        })
        .await
        .expect("reaction add should appear in Zulip event queue");
    let reaction_add_projection = trace.record_event(reaction_add_event).await;
    assert_eq!(
        reaction_add_projection.message_id,
        stream_projection.message_id
    );
    trace.assert_reaction_state(&stream_projection, true).await;

    bot_client
        .remove_reaction(stream_message_id, &reaction)
        .await
        .expect("remove Zulip reaction");
    let reaction_remove_event = cursor
        .wait_for_event(&bot_client, "reaction remove", |event| {
            event.event_type == "reaction"
                && event_i64(event, "message_id") == Some(stream_message_id)
                && event_str(event, "op") == Some("remove")
        })
        .await
        .expect("reaction remove should appear in Zulip event queue");
    let reaction_remove_projection = trace.record_event(reaction_remove_event).await;
    assert_eq!(
        reaction_remove_projection.message_id,
        stream_projection.message_id
    );
    trace.assert_reaction_state(&stream_projection, false).await;

    let updated_content = "Надо подготовить Zulip факты до завтра. Обновлено live-тестом.";
    bot_client
        .update_message(
            stream_message_id,
            &ZulipUpdateMessageRequest::new().content(updated_content),
        )
        .await
        .expect("edit Zulip message");
    let update_event = cursor
        .wait_for_event(&bot_client, "message update", |event| {
            event.event_type == "update_message"
                && event_i64(event, "message_id") == Some(stream_message_id)
        })
        .await
        .expect("message update should appear in Zulip event queue");
    let update_projection = trace.record_event(update_event).await;
    assert_eq!(update_projection.message_id, stream_projection.message_id);
    assert_eq!(update_projection.body_text, updated_content);
    trace.assert_version_recorded(&stream_projection).await;

    bot_client
        .delete_message(stream_message_id)
        .await
        .expect("delete own Zulip message");
    let delete_event = cursor
        .wait_for_event(&bot_client, "message delete", |event| {
            event.event_type == "delete_message"
                && event_i64(event, "message_id") == Some(stream_message_id)
        })
        .await
        .expect("message delete should appear in Zulip event queue");
    let delete_projection = trace.record_event(delete_event).await;
    assert_eq!(delete_projection.message_id, stream_projection.message_id);
    trace.assert_tombstone_recorded(&stream_projection).await;

    trace
        .assert_backend_workers_round_trip_real_zulip_command(&realm)
        .await;
    trace
        .assert_event_ingest_reregisters_bad_live_queue(&realm)
        .await;
    write_lab_backend_evidence_report(&realm);
}

struct LiveМакошьTrace {
    _ctx: TestContext,
    pool: PgPool,
    account_id: String,
    base_url: String,
    resolver: InMemorySecretResolver,
}

impl LiveМакошьTrace {
    async fn new(realm: &ProvisionedZulipRealm) -> Self {
        eprintln!("[zulip-live] preparing Макошь trace database");
        let ctx = TestContext::new().await;
        let pool = ctx.pool().clone();
        let account_id = "zulip-live-account".to_owned();
        let mut resolver = InMemorySecretResolver::new();

        SignalHubStore::new(pool.clone())
            .restore_system_sources()
            .await
            .expect("restore Signal Hub sources");
        CommunicationProviderAccountStore::new(pool.clone())
            .upsert(
                &NewProviderAccount::new(
                    &account_id,
                    CommunicationProviderKind::ZulipBot,
                    "Zulip Live",
                    &realm.bot_email,
                )
                .config(json!({"base_url": realm.base_url})),
            )
            .await
            .expect("Zulip live provider account");
        bind_zulip_api_key(
            &pool,
            &mut resolver,
            &account_id,
            "secret:test:zulip-live-api-key",
            &realm.bot_api_key,
        )
        .await;

        Self {
            _ctx: ctx,
            pool,
            account_id,
            base_url: realm.base_url.clone(),
            resolver,
        }
    }

    async fn record_event(&self, event: ZulipEvent) -> ProjectedMessage {
        eprintln!(
            "[zulip-live] tracing real Zulip `{}` event {} through Макошь",
            event.event_type, event.id
        );
        let mapping_context =
            ZulipEventMappingContext::new(&self.account_id, &self.base_url, Utc::now())
                .with_import_batch_id("zulip-live-testcontainers")
                .with_scenario_id("zulip-live-provider-surface");
        let new_raw_record = observation_to_raw_communication_record(
            map_zulip_event_to_observation(&event, &mapping_context).expect("map real Zulip event"),
        );
        let ingestion = CommunicationIngestionStore::new(self.pool.clone());
        let raw_record = ingestion
            .record_raw_source(&new_raw_record)
            .await
            .expect("record real Zulip raw source");
        let accepted_event = dispatch_zulip_raw_signal(self.pool.clone(), &raw_record)
            .await
            .expect("dispatch real Zulip raw signal")
            .expect("accepted real Zulip raw signal");

        consume_accepted_signal_event(self.pool.clone(), &accepted_event)
            .await
            .expect("project real Zulip accepted signal")
            .expect("real Zulip projection")
    }

    async fn assert_task_review_candidate(&self, projected: &ProjectedMessage) {
        let refreshed = refresh_message_task_candidates_into_review(
            &self.pool,
            std::slice::from_ref(&projected.message_id),
        )
        .await
        .expect("refresh live Zulip task candidates into review");
        assert_eq!(refreshed, 1);

        let candidate_row = sqlx::query(
            r#"
        SELECT title, candidate_kind, review_state, due_text, evidence_excerpt
        FROM task_candidates
        WHERE source_id = $1
          AND source_kind = 'observation'
        "#,
        )
        .bind(&projected.observation_id)
        .fetch_one(&self.pool)
        .await
        .expect("live Zulip task candidate row");
        assert_eq!(
            candidate_row
                .try_get::<String, _>("candidate_kind")
                .expect("candidate_kind"),
            "task"
        );
        assert_eq!(
            candidate_row
                .try_get::<String, _>("review_state")
                .expect("review_state"),
            "suggested"
        );
        assert_eq!(
            candidate_row.try_get::<String, _>("title").expect("title"),
            "Надо подготовить Zulip факты до завтра."
        );
        assert_eq!(
            candidate_row
                .try_get::<Option<String>, _>("due_text")
                .expect("due_text")
                .as_deref(),
            Some("завтра")
        );
        assert_eq!(
            candidate_row
                .try_get::<String, _>("evidence_excerpt")
                .expect("evidence_excerpt"),
            "Надо подготовить Zulip факты до завтра."
        );

        let review_count: i64 = sqlx::query_scalar(
            r#"
        SELECT count(*)::BIGINT
        FROM review_items item
        JOIN review_item_evidence evidence
          ON evidence.review_item_id = item.review_item_id
        WHERE evidence.observation_id = $1
          AND item.item_kind = 'potential_task'
          AND item.status = 'new'
        "#,
        )
        .bind(&projected.observation_id)
        .fetch_one(&self.pool)
        .await
        .expect("live Zulip task review item count");
        assert_eq!(review_count, 1);

        let task_count: i64 =
            sqlx::query_scalar("SELECT count(*)::BIGINT FROM tasks WHERE source_id = $1")
                .bind(&projected.message_id)
                .fetch_one(&self.pool)
                .await
                .expect("auto-created task count");
        let obligation_count: i64 =
            sqlx::query_scalar("SELECT count(*)::BIGINT FROM obligations WHERE statement = $1")
                .bind("Надо подготовить Zulip факты до завтра.")
                .fetch_one(&self.pool)
                .await
                .expect("auto-created obligation count");
        assert_eq!(task_count, 0);
        assert_eq!(obligation_count, 0);
        eprintln!(
            "[zulip-live] Макошь pipeline produced review task candidate without durable task"
        );
    }

    async fn assert_reaction_state(&self, projected: &ProjectedMessage, is_active: bool) {
        let active: bool = sqlx::query_scalar(
            r#"
            SELECT is_active
            FROM communication_message_reactions
            WHERE message_id = $1
              AND reaction = 'thumbs_up'
            "#,
        )
        .bind(&projected.message_id)
        .fetch_one(&self.pool)
        .await
        .expect("live Zulip reaction state");
        assert_eq!(active, is_active);
    }

    async fn assert_version_recorded(&self, projected: &ProjectedMessage) {
        let version_count: i64 = sqlx::query_scalar(
            r#"
            SELECT count(*)::BIGINT
            FROM communication_message_versions
            WHERE message_id = $1
            "#,
        )
        .bind(&projected.message_id)
        .fetch_one(&self.pool)
        .await
        .expect("live Zulip message version count");
        assert_eq!(version_count, 1);
    }

    async fn assert_tombstone_recorded(&self, projected: &ProjectedMessage) {
        let tombstone_count: i64 = sqlx::query_scalar(
            r#"
            SELECT count(*)::BIGINT
            FROM communication_message_tombstones
            WHERE message_id = $1
              AND is_provider_delete = TRUE
              AND is_local_visible = FALSE
            "#,
        )
        .bind(&projected.message_id)
        .fetch_one(&self.pool)
        .await
        .expect("live Zulip message tombstone count");
        assert_eq!(tombstone_count, 1);
    }

    async fn assert_backend_workers_round_trip_real_zulip_command(
        &self,
        realm: &ProvisionedZulipRealm,
    ) {
        eprintln!("[zulip-live] exercising backend workers against real Zulip realm");
        let ingest_worker = ZulipEventIngestWorker::new(self.pool.clone(), self.resolver.clone());
        let baseline = ingest_worker
            .poll_account(&self.account_id, Utc::now())
            .await
            .expect("register live Zulip worker event queue");
        assert_eq!(baseline.accounts_scanned, 1);
        assert_eq!(baseline.accounts_failed, 0);
        assert_eq!(baseline.queues_registered, 1);

        let imported = self
            .store_worker_upload_attachment("zulip-live-worker-upload-import")
            .await;
        let command_store = CommunicationProviderCommandStore::new(self.pool.clone());
        let command = command_store
            .enqueue(
                &NewCommunicationProviderCommand::new(
                    "zulip-live-worker-stream-upload-1",
                    &self.account_id,
                    "zulip",
                    "send_stream_message_with_upload",
                    "zulip-live-worker:stream-upload:1",
                    "makosh-live-test",
                )
                .provider_conversation_id(format!("{}/live-worker-upload", realm.stream_name))
                .target_ref(json!({
                    "stream": realm.stream_name,
                    "topic": "live-worker-upload"
                }))
                .payload(json!({
                    "stream": realm.stream_name,
                    "topic": "live-worker-upload",
                    "content": "Надо проверить backend worker upload до пятницы.",
                    "attachment_id": imported.attachment_id,
                    "blob_id": imported.blob_id
                })),
            )
            .await
            .expect("enqueue live Zulip worker upload command");

        let command_report = ZulipCommandWorker::new(self.pool.clone(), self.resolver.clone())
            .execute_due_for_account(&self.account_id, Utc::now(), 10)
            .await
            .expect("execute live Zulip worker command");
        assert_eq!(command_report.claimed, 1);
        assert_eq!(command_report.completed, 1);
        assert_eq!(command_report.retrying, 0);
        assert_eq!(command_report.dead_lettered, 0);

        let completed = self
            .provider_command(&command_store, &command.command_id)
            .await;
        assert_eq!(completed.status, "completed");
        assert_eq!(completed.command_kind, "send_stream_message_with_upload");
        assert_eq!(completed.reconciliation_status, "awaiting_provider");
        let provider_message_id = completed
            .provider_message_id
            .clone()
            .expect("live worker command provider_message_id");

        let accepted_event = self
            .wait_for_ingested_accepted_zulip_message(
                &ingest_worker,
                &provider_message_id,
                "worker upload message",
            )
            .await;
        let projected = consume_accepted_signal_event(self.pool.clone(), &accepted_event.event)
            .await
            .expect("project live worker accepted Zulip message")
            .expect("live worker Zulip projection");
        assert_eq!(projected.provider_record_id, provider_message_id);
        assert_eq!(projected.channel_kind, "zulip");
        assert_eq!(
            projected
                .message_metadata
                .get("attachment_state")
                .and_then(|state| state.get("bytes_state"))
                .and_then(Value::as_str),
            Some("not_transferred")
        );

        reconcile_zulip_provider_observation_event(
            self.pool.clone(),
            InMemoryEventBus::new(),
            accepted_event.clone(),
        )
        .await
        .expect("reconcile live worker Zulip command observation");
        let reconciled = self
            .provider_command(&command_store, &command.command_id)
            .await;
        assert_eq!(reconciled.reconciliation_status, "observed");
        assert_eq!(
            reconciled.provider_state["observed_via"],
            json!("signal_hub_accepted_event")
        );

        let blob_root = tempdir().expect("live Zulip materialization blob root");
        let download_report =
            ZulipAttachmentDownloadWorker::new(self.pool.clone(), self.resolver.clone())
                .with_blob_root(blob_root.path())
                .download_due_for_account(&self.account_id, Utc::now(), 10)
                .await
                .expect("download live worker Zulip attachment");
        assert_eq!(download_report.accounts_scanned, 1);
        assert_eq!(download_report.accounts_failed, 0);
        assert!(
            download_report.attachments_downloaded >= 1,
            "expected at least the worker command attachment to be downloaded, got {download_report:?}"
        );
        assert!(
            download_report.attachments_materialized >= 1,
            "expected at least the worker command attachment to be materialized, got {download_report:?}"
        );
        assert_eq!(download_report.attachments_failed, 0);

        let attachment_count: i64 = sqlx::query_scalar(
            r#"
            SELECT count(*)::BIGINT
            FROM communication_attachments
            WHERE message_id = $1
            "#,
        )
        .bind(&projected.message_id)
        .fetch_one(&self.pool)
        .await
        .expect("live worker materialized attachment count");
        assert_eq!(attachment_count, 1);
        eprintln!(
            "[zulip-live] backend worker command, ingest, reconciliation and attachment materialization passed"
        );
    }

    async fn assert_event_ingest_reregisters_bad_live_queue(&self, realm: &ProvisionedZulipRealm) {
        eprintln!("[zulip-live] exercising live Zulip event queue re-registration");
        let ingestion = CommunicationIngestionStore::new(self.pool.clone());
        ingestion
            .save_checkpoint(&NewIngestionCheckpoint::new(
                &self.account_id,
                "zulip:event_queue",
                json!({
                    "queue_id": "expired-live-zulip-queue",
                    "last_event_id": 0
                }),
            ))
            .await
            .expect("save live expired Zulip queue checkpoint");

        let ingest_worker = ZulipEventIngestWorker::new(self.pool.clone(), self.resolver.clone());
        let recovery_report = ingest_worker
            .poll_account(&self.account_id, Utc::now())
            .await
            .expect("poll live expired Zulip event queue");
        assert_eq!(recovery_report.accounts_scanned, 1);
        assert_eq!(recovery_report.accounts_failed, 0);
        assert_eq!(recovery_report.queues_registered, 1);
        assert!(
            recovery_report.checkpoints_saved >= 2,
            "expired queue recovery should save registered and final checkpoints, got {recovery_report:?}"
        );

        let checkpoint = ingestion
            .checkpoint(&self.account_id, "zulip:event_queue")
            .await
            .expect("live recovered Zulip queue checkpoint query")
            .expect("live recovered Zulip queue checkpoint");
        assert_ne!(
            checkpoint.checkpoint["queue_id"],
            json!("expired-live-zulip-queue")
        );

        let bot_client = ZulipApiClient::new(
            ZulipClientConfig::new(&realm.base_url, &realm.bot_email, &realm.bot_api_key)
                .expect("valid live Zulip bot client after queue recovery"),
        );
        let recovery_message = bot_client
            .send_stream_message(
                &realm.stream_name,
                "live-queue-recovery",
                "Надо проверить восстановление Zulip очереди до вечера.",
            )
            .await
            .expect("send live queue recovery message");
        let provider_message_id = recovery_message
            .id
            .expect("live queue recovery message id")
            .to_string();
        let accepted_event = self
            .wait_for_ingested_accepted_zulip_message(
                &ingest_worker,
                &provider_message_id,
                "queue recovery message",
            )
            .await;
        let projected = consume_accepted_signal_event(self.pool.clone(), &accepted_event.event)
            .await
            .expect("project live queue recovery accepted Zulip message")
            .expect("live queue recovery Zulip projection");
        assert_eq!(projected.provider_record_id, provider_message_id);
        assert_eq!(projected.subject, "makosh-lab / live-queue-recovery");
        eprintln!("[zulip-live] live Zulip event queue re-registration passed");
    }

    async fn store_worker_upload_attachment(
        &self,
        attachment_id: &str,
    ) -> ImportedCommunicationAttachment {
        let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
        let local_blob = blob_store
            .put_blob(b"zulip live worker attachment bytes")
            .await
            .expect("live worker local blob");
        let storage = CommunicationStorageStore::new(self.pool.clone());
        let stored_blob = storage
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&local_blob).content_type("text/plain"),
            )
            .await
            .expect("live worker stored blob metadata");
        storage
            .upsert_imported_attachment(
                &NewCommunicationAttachmentImport::new(
                    attachment_id,
                    &stored_blob.blob_id,
                    "text/plain",
                    local_blob.size_bytes,
                    &local_blob.sha256,
                    "zulip-live-worker",
                )
                .account_id(&self.account_id)
                .channel_kind("zulip")
                .filename("worker-evidence.txt")
                .source_kind("zulip_live_worker_test")
                .metadata(json!({"scenario": "zulip_live_worker_round_trip"})),
            )
            .await
            .expect("live worker imported attachment")
    }

    async fn wait_for_ingested_accepted_zulip_message(
        &self,
        ingest_worker: &ZulipEventIngestWorker<InMemorySecretResolver>,
        provider_message_id: &str,
        description: &str,
    ) -> StoredEventEnvelope {
        let deadline = Instant::now() + Duration::from_secs(30);
        let started = Instant::now();
        let mut next_log = Instant::now() + LIVE_WAIT_LOG_INTERVAL;
        loop {
            let report = ingest_worker
                .poll_account(&self.account_id, Utc::now())
                .await
                .expect("poll live Zulip worker event queue");
            if report.events_received > 0 {
                eprintln!(
                    "[zulip-live] ingest poll received {} events, accepted {}",
                    report.events_received, report.accepted_signals
                );
            }

            if let Some(event) = self
                .accepted_zulip_message_for_provider_message(provider_message_id)
                .await
            {
                return event;
            }

            if Instant::now() >= deadline {
                panic!(
                    "{description} accepted event for provider message {provider_message_id} did not arrive"
                );
            }
            if Instant::now() >= next_log {
                eprintln!(
                    "[zulip-live] waiting for {description} accepted event; provider_message_id={provider_message_id} elapsed={}s",
                    started.elapsed().as_secs()
                );
                next_log = Instant::now() + LIVE_WAIT_LOG_INTERVAL;
            }
            sleep(Duration::from_millis(250)).await;
        }
    }

    async fn accepted_zulip_message_for_provider_message(
        &self,
        provider_message_id: &str,
    ) -> Option<StoredEventEnvelope> {
        let events = EventStore::new(self.pool.clone())
            .list_matching(
                EventLogQuery::default()
                    .event_type("signal.accepted.zulip.message")
                    .limit(200),
            )
            .await
            .expect("list accepted Zulip message events");
        for event in events {
            let Some(raw_record_id) = event
                .event
                .subject
                .get("raw_record_id")
                .and_then(Value::as_str)
            else {
                continue;
            };
            let Some(raw_record) = CommunicationIngestionStore::new(self.pool.clone())
                .raw_record(raw_record_id)
                .await
                .expect("accepted Zulip raw record lookup")
            else {
                continue;
            };
            if raw_record
                .payload
                .get("provider_message_id")
                .and_then(Value::as_str)
                == Some(provider_message_id)
            {
                return Some(event);
            }
        }
        None
    }

    async fn provider_command(
        &self,
        store: &CommunicationProviderCommandStore,
        command_id: &str,
    ) -> CommunicationProviderCommand {
        store
            .list(&self.account_id, "zulip", 50)
            .await
            .expect("list live Zulip provider commands")
            .into_iter()
            .find(|item| item.command_id == command_id)
            .expect("live Zulip provider command")
    }
}

struct ZulipEventCursor {
    queue_id: String,
    last_event_id: i64,
}

impl ZulipEventCursor {
    fn new(queue_id: String, last_event_id: i64) -> Self {
        Self {
            queue_id,
            last_event_id,
        }
    }

    async fn wait_for_message_event(
        &mut self,
        client: &ZulipApiClient,
        message_id: i64,
    ) -> Result<ZulipEvent, String> {
        self.wait_for_event(client, "message", |event| {
            event.event_type == "message" && message_event_id(event) == Some(message_id)
        })
        .await
    }

    async fn wait_for_event(
        &mut self,
        client: &ZulipApiClient,
        description: &str,
        mut predicate: impl FnMut(&ZulipEvent) -> bool,
    ) -> Result<ZulipEvent, String> {
        let deadline = Instant::now() + Duration::from_secs(30);
        let started = Instant::now();
        let mut next_log = Instant::now() + LIVE_WAIT_LOG_INTERVAL;

        loop {
            let events = client
                .get_events(&self.queue_id, self.last_event_id, true)
                .await
                .map_err(|error| error.to_string())?;
            for event in events.events {
                self.last_event_id = self.last_event_id.max(event.id);
                if predicate(&event) {
                    return Ok(event);
                }
            }

            if Instant::now() >= deadline {
                return Err(format!(
                    "{description} event did not arrive on Zulip queue {}",
                    self.queue_id
                ));
            }

            if Instant::now() >= next_log {
                eprintln!(
                    "[zulip-live] waiting for {description} event; queue={} last_event_id={} elapsed={}s",
                    self.queue_id,
                    self.last_event_id,
                    started.elapsed().as_secs()
                );
                next_log = Instant::now() + LIVE_WAIT_LOG_INTERVAL;
            }
            sleep(Duration::from_millis(250)).await;
        }
    }
}

fn message_event_id(event: &ZulipEvent) -> Option<i64> {
    event
        .data
        .get("message")
        .and_then(|message| message.get("id"))
        .and_then(Value::as_i64)
}

fn event_i64(event: &ZulipEvent, field: &str) -> Option<i64> {
    event.data.get(field).and_then(Value::as_i64)
}

fn event_str<'a>(event: &'a ZulipEvent, field: &str) -> Option<&'a str> {
    event.data.get(field).and_then(Value::as_str)
}

async fn bind_zulip_api_key(
    pool: &PgPool,
    resolver: &mut InMemorySecretResolver,
    account_id: &str,
    secret_ref: &str,
    secret_value: &str,
) {
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(&NewSecretReference::new(
            secret_ref,
            SecretKind::ApiToken,
            SecretStoreKind::TestDouble,
            "Zulip API key",
        ))
        .await
        .expect("live Zulip secret reference");
    CommunicationProviderSecretBindingStore::new(pool.clone())
        .bind(&NewProviderAccountSecretBinding::new(
            account_id,
            ProviderAccountSecretPurpose::ZulipApiKey,
            secret_ref,
        ))
        .await
        .expect("live Zulip API key binding");
    resolver
        .insert(secret_ref, secret_value)
        .expect("insert live Zulip test secret");
}

fn write_lab_backend_evidence_report(realm: &ProvisionedZulipRealm) {
    let Ok(report_dir) = env::var("MAKOSH_ZULIP_LIVE_REPORT_DIR") else {
        return;
    };

    fs::create_dir_all(&report_dir).expect("create Zulip live Lab backend report directory");
    let finished_at = Utc::now().to_rfc3339();
    let scenario_id = env::var("MAKOSH_ZULIP_LIVE_SCENARIO_ID")
        .unwrap_or_else(|_| "zulip_backend_live_trace".to_owned());
    let report = json!({
        "harness": "backend/tests/zulip_live.rs",
        "scenario_id": scenario_id,
        "provider": "zulip",
        "finished_at": finished_at,
        "status": "passed",
        "fixture": {
            "kind": "zulip_testcontainers",
            "base_url": realm.base_url,
            "stream_name": realm.stream_name,
            "owner_email": realm.owner_email,
            "owner_user_id": realm.owner_user_id,
            "bot_email": realm.bot_email,
            "bot_user_id": realm.bot_user_id,
            "human_email": realm.human_email,
            "human_user_id": realm.human_user_id
        },
        "observed_stages": [
            "provider_api.stream_message_sent",
            "provider_api.direct_message_sent",
            "provider_api.file_uploaded",
            "provider_api.user_upload_downloaded",
            "provider_api.reaction_added",
            "provider_api.reaction_removed",
            "provider_api.message_updated",
            "provider_api.message_deleted",
            "signal.raw.zulip.message.observed",
            "signal.raw.zulip.reaction.observed",
            "signal.raw.zulip.message_update.observed",
            "signal.raw.zulip.message_delete.observed",
            "signal.accepted.zulip.message",
            "signal.accepted.zulip.reaction",
            "signal.accepted.zulip.message_update",
            "signal.accepted.zulip.message_delete",
            "communication.message.recorded",
            "communication.message.updated",
            "communication_message_reactions",
            "communication_message_versions",
            "communication_message_tombstones",
            "zulip.direct_conversation",
            "zulip_attachment_metadata",
            "attachment_state.materialized",
            "communication_attachments",
            "task_candidates",
            "review_items",
            "provider_command.completed",
            "zulip.command.reconciled",
            "zulip_event_queue.registered",
            "zulip_event_queue.reregistered",
            "ingestion_checkpoint.persisted"
        ],
        "credential_payload_present": false
    });

    let report_path = Path::new(&report_dir).join(format!(
        "zulip_backend_live_trace-{}.json",
        safe_report_id(&finished_at)
    ));
    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&report).expect("serialize Zulip live Lab backend report"),
    )
    .expect("write Zulip live Lab backend report");
    eprintln!(
        "[zulip-live] Lab backend evidence report written: {}",
        report_path.display()
    );
}

fn safe_report_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}
