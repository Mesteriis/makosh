use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use std::collections::HashMap;
use std::net::SocketAddr;

use axum::body::Bytes;
use axum::body::{Body, to_bytes};
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use chrono::Utc;
use makosh_backend_testkit::context::TestContext;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tempfile::tempdir;
use tokio::net::TcpListener;
use tower::ServiceExt;
use url::form_urlencoded;

use makosh_communications_api::commands::NewCommunicationProviderCommand;
use makosh_communications_api::evidence::NewIngestionCheckpoint;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_events_api::{EventEnvelope, StoredEventEnvelope};
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::application::zulip_attachment_download::ZulipAttachmentDownloadWorker;
use makosh_hub_backend::application::zulip_command_executor::ZulipCommandWorker;
use makosh_hub_backend::application::zulip_event_ingest::ZulipEventIngestWorker;
use makosh_hub_backend::application::zulip_provider_observation_reconciliation::reconcile_zulip_provider_observation_event;
use makosh_hub_backend::domains::communications::messages::models::ProjectedMessage;
use makosh_hub_backend::domains::communications::messages::provider_channel_store::ProviderChannelMessageStore;
use makosh_hub_backend::domains::communications::messages::provider_observation_projection::consume_accepted_signal_event;
use makosh_hub_backend::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use makosh_hub_backend::domains::communications::storage::models::{
    NewCommunicationAttachmentImport, NewCommunicationBlob,
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
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::review_inbox::refresh_message_task_candidates_into_review;
use makosh_hub_backend::workflows::zulip_attachment_storage::{
    ZulipAttachmentBytes, persist_zulip_attachment_bytes,
};
use makosh_provider_zulip::client::{
    ZulipApiClient, ZulipClientConfig, ZulipReactionRequest, ZulipUpdateMessageRequest,
};
use makosh_provider_zulip::event_mapper::{
    ZulipEventMappingContext, map_zulip_event_to_observation, zulip_raw_signal_event_type,
};
use makosh_provider_zulip::models::ZulipEvent;

const LOCAL_API_TOKEN: &str = "zulip-api-test-secret";

#[test]
fn zulip_provider_and_secret_kinds_are_account_scoped() {
    assert_eq!(
        CommunicationProviderKind::try_from("zulip_bot").expect("zulip bot provider"),
        CommunicationProviderKind::ZulipBot
    );
    assert!(CommunicationProviderKind::ZulipBot.is_zulip());
    assert!(!CommunicationProviderKind::ZulipBot.is_email());
    assert!(!CommunicationProviderKind::ZulipBot.is_telegram());
    assert!(!CommunicationProviderKind::ZulipBot.is_whatsapp());
    assert!(!CommunicationProviderKind::ZulipBot.is_zoom());

    assert!(ProviderAccountSecretPurpose::ZulipApiKey.accepts_secret_kind(SecretKind::ApiToken));
    assert!(!ProviderAccountSecretPurpose::ZulipApiKey.accepts_secret_kind(SecretKind::Password));
    assert!(!ProviderAccountSecretPurpose::ZulipApiKey.accepts_secret_kind(SecretKind::OauthToken));
}

#[test]
fn zulip_client_config_debug_redacts_api_key() {
    let config = ZulipClientConfig::new(
        "http://localhost:8080",
        "bot@example.test",
        "zulip-api-key-must-not-leak",
    )
    .expect("valid Zulip client config");

    let debug = format!("{config:?}");

    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("zulip-api-key-must-not-leak"));
}

#[tokio::test]
async fn zulip_account_setup_stores_api_key_in_host_vault() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let account_id = format!(
        "zulip-bot-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_pairs([
            ("MAKOSH_DEV_MODE", "true"),
            (
                "MAKOSH_VAULT_HOME",
                vault_dir.path().join("vault").to_str().expect("vault path"),
            ),
            (
                "MAKOSH_DEV_KEY_PATH",
                vault_dir
                    .path()
                    .join("dev")
                    .join("master.key")
                    .to_str()
                    .expect("dev key path"),
            ),
        ])
        .expect("config"),
        database,
    );

    let entropy_response = app
        .clone()
        .oneshot(json_post_request(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);
    let create_response = app
        .clone()
        .oneshot(json_post_request("/api/v1/vault/create", json!({})))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(json_post_request(
            "/api/v1/integrations/zulip/accounts",
            json!({
                "account_id": account_id,
                "display_name": "Zulip Bot",
                "external_account_id": "bot@example.test",
                "base_url": "http://localhost:8080/",
                "api_key": "zulip-api-key-must-stay-in-host-vault"
            }),
        ))
        .await
        .expect("Zulip setup response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["account_id"], json!(account_id));
    assert_eq!(body["provider_kind"], json!("zulip_bot"));
    assert_eq!(body["base_url"], json!("http://localhost:8080"));
    assert_eq!(
        body["credential_binding"]["secret_purpose"],
        json!("zulip_api_key")
    );
    assert_eq!(
        body["credential_binding"]["secret_kind"],
        json!("api_token")
    );
    assert_eq!(
        body["credential_binding"]["store_kind"],
        json!("host_vault")
    );
    assert!(
        !body
            .to_string()
            .contains("zulip-api-key-must-stay-in-host-vault")
    );

    let account = sqlx::query(
        "SELECT provider_kind, external_account_id, config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("Zulip provider account");
    assert_eq!(
        account
            .try_get::<String, _>("provider_kind")
            .expect("provider_kind"),
        "zulip_bot"
    );
    assert_eq!(
        account
            .try_get::<String, _>("external_account_id")
            .expect("external_account_id"),
        "bot@example.test"
    );
    let config = account.try_get::<Value, _>("config").expect("config");
    assert_eq!(config["base_url"], json!("http://localhost:8080"));
    assert_eq!(config["credentials"]["api_key_bound"], json!(true));
    assert!(config.get("api_key").is_none());
    assert!(
        !config
            .to_string()
            .contains("zulip-api-key-must-stay-in-host-vault")
    );

    let secret_ref = body["credential_binding"]["secret_ref"]
        .as_str()
        .expect("secret ref");
    let reference = SecretReferenceStore::new(pool.clone())
        .secret_reference(secret_ref)
        .await
        .expect("secret reference query")
        .expect("secret reference exists");
    assert_eq!(reference.secret_kind, SecretKind::ApiToken);
    assert_eq!(reference.store_kind, SecretStoreKind::HostVault);
    assert_eq!(reference.metadata["provider"], json!("zulip_bot"));
    assert_eq!(reference.metadata["account_id"], json!(account_id));
    assert!(
        !reference
            .metadata
            .to_string()
            .contains("zulip-api-key-must-stay-in-host-vault")
    );

    let binding_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM communication_provider_account_secret_refs
        WHERE account_id = $1
          AND secret_purpose = 'zulip_api_key'
          AND secret_ref = $2
        "#,
    )
    .bind(&account_id)
    .bind(secret_ref)
    .fetch_one(&pool)
    .await
    .expect("Zulip secret binding count");
    assert_eq!(binding_count, 1);

    let database_payload_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM encrypted_secret_vault_entries WHERE secret_ref = $1",
    )
    .bind(secret_ref)
    .fetch_one(&pool)
    .await
    .expect("database secret payload count");
    assert_eq!(database_payload_count, 0);
}

#[tokio::test]
async fn zulip_upload_command_endpoints_enqueue_reference_only_commands() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "zulip-api-upload-account",
            CommunicationProviderKind::ZulipBot,
            "Zulip API Upload",
            "bot@example.test",
        ))
        .await
        .expect("provider account");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let stream_response = app
        .clone()
        .oneshot(json_post_request(
            "/api/v1/integrations/zulip/accounts/zulip-api-upload-account/commands/stream-upload",
            json!({
                "idempotency_key": "zulip-api-stream-upload-1",
                "stream": "Макошь Lab",
                "topic": "Tasks",
                "content": "Надо проверить retention.",
                "attachment_id": "zulip-stream-upload-import-1",
                "blob_id": "blob:v1:zulip-stream-upload-import-1",
                "filename": "evidence.txt"
            }),
        ))
        .await
        .expect("Zulip stream upload command enqueue response");

    assert_eq!(stream_response.status(), StatusCode::OK);
    let stream_body = response_json(stream_response).await;
    assert_eq!(stream_body["account_id"], json!("zulip-api-upload-account"));
    assert_eq!(stream_body["channel_kind"], json!("zulip"));
    assert_eq!(
        stream_body["command_kind"],
        json!("send_stream_message_with_upload")
    );
    assert_eq!(stream_body["status"], json!("queued"));
    assert_eq!(stream_body["reconciliation_status"], json!("not_observed"));
    assert_eq!(
        stream_body["payload"]["attachment_id"],
        json!("zulip-stream-upload-import-1")
    );
    assert_eq!(
        stream_body["payload"]["blob_id"],
        json!("blob:v1:zulip-stream-upload-import-1")
    );
    assert_eq!(stream_body["payload"]["filename"], json!("evidence.txt"));
    assert!(!format!("{stream_body:?}").contains("zulip attachment bytes"));

    let direct_response = app
        .clone()
        .oneshot(json_post_request(
            "/api/v1/integrations/zulip/accounts/zulip-api-upload-account/commands/direct-upload",
            json!({
                "idempotency_key": "zulip-api-direct-upload-1",
                "recipients": ["101", "202"],
                "content": "Direct evidence.",
                "attachment_id": "zulip-direct-upload-import-1",
                "filename": "direct.txt"
            }),
        ))
        .await
        .expect("Zulip direct upload command enqueue response");
    assert_eq!(direct_response.status(), StatusCode::OK);
    let direct_body = response_json(direct_response).await;
    assert_eq!(
        direct_body["command_kind"],
        json!("send_direct_message_with_upload")
    );
    assert_eq!(direct_body["payload"]["recipients"], json!(["101", "202"]));
    assert_eq!(
        direct_body["payload"]["attachment_id"],
        json!("zulip-direct-upload-import-1")
    );
    assert_eq!(direct_body["payload"]["blob_id"], Value::Null);
    assert!(!format!("{direct_body:?}").contains("zulip attachment bytes"));

    let upload_response = app
        .oneshot(json_post_request(
            "/api/v1/integrations/zulip/accounts/zulip-api-upload-account/commands/upload",
            json!({
                "idempotency_key": "zulip-api-upload-only-1",
                "blob_id": "blob:v1:zulip-upload-only-1",
                "filename": "upload-only.txt"
            }),
        ))
        .await
        .expect("Zulip upload-only command enqueue response");
    assert_eq!(upload_response.status(), StatusCode::OK);
    let upload_body = response_json(upload_response).await;
    assert_eq!(upload_body["command_kind"], json!("upload_file"));
    assert_eq!(upload_body["payload"]["attachment_id"], Value::Null);
    assert_eq!(
        upload_body["payload"]["blob_id"],
        json!("blob:v1:zulip-upload-only-1")
    );
    assert!(!format!("{upload_body:?}").contains("zulip attachment bytes"));

    let commands = CommunicationProviderCommandStore::new(pool)
        .list("zulip-api-upload-account", "zulip", 10)
        .await
        .expect("list queued Zulip commands");
    assert_eq!(commands.len(), 3);
    let stream = commands
        .iter()
        .find(|command| command.idempotency_key == "zulip-api-stream-upload-1")
        .expect("stream upload command");
    assert_eq!(stream.command_kind, "send_stream_message_with_upload");
    assert_eq!(
        stream.provider_conversation_id.as_deref(),
        Some("Макошь Lab/Tasks")
    );
    let direct = commands
        .iter()
        .find(|command| command.idempotency_key == "zulip-api-direct-upload-1")
        .expect("direct upload command");
    assert_eq!(direct.command_kind, "send_direct_message_with_upload");
    assert!(direct.provider_conversation_id.is_none());
    assert_eq!(direct.payload["recipients"], json!(["101", "202"]));
    let upload = commands
        .iter()
        .find(|command| command.idempotency_key == "zulip-api-upload-only-1")
        .expect("upload-only command");
    assert_eq!(upload.command_kind, "upload_file");
    assert!(
        !commands
            .iter()
            .any(|command| format!("{:?}", command.payload).contains("zulip attachment bytes"))
    );
}

#[tokio::test]
async fn zulip_api_client_supports_message_lifecycle_reactions_and_uploads() {
    let base_url = spawn_fake_zulip_api().await;
    let client = ZulipApiClient::new(
        ZulipClientConfig::new(&base_url, "bot@example.test", "zulip-test-key")
            .expect("valid Zulip client config"),
    );

    let stream_message = client
        .send_stream_message("Макошь Lab", "Tasks", "Надо проверить retention.")
        .await
        .expect("send stream message");
    assert_eq!(stream_message.id, Some(7001));

    let direct_message = client
        .send_direct_message(&["alice@example.test"], "Direct note")
        .await
        .expect("send direct message");
    assert_eq!(direct_message.id, Some(7002));

    let direct_by_user_id = client
        .send_direct_message_to_user_ids(&[101], "Direct note by user id")
        .await
        .expect("send direct message by user id");
    assert_eq!(direct_by_user_id.id, Some(7003));

    let update = client
        .update_message(
            7001,
            &ZulipUpdateMessageRequest::new()
                .content("Updated retention note")
                .topic("Follow-up")
                .stream_id(99)
                .propagate_mode("change_all"),
        )
        .await
        .expect("update message");
    assert_eq!(update.result, "success");

    let reaction = ZulipReactionRequest::new("thumbs_up")
        .emoji_code("1f44d")
        .reaction_type("unicode_emoji");
    let added = client
        .add_reaction(7001, &reaction)
        .await
        .expect("add reaction");
    assert_eq!(added.result, "success");

    let removed = client
        .remove_reaction(7001, &reaction)
        .await
        .expect("remove reaction");
    assert_eq!(removed.result, "success");

    let uploaded = client
        .upload_file_bytes("evidence.txt", b"zulip attachment bytes".to_vec())
        .await
        .expect("upload file");
    assert_eq!(uploaded.uri, "/user_uploads/evidence.txt");

    let downloaded = client
        .download_user_upload("/user_uploads/evidence.txt")
        .await
        .expect("download user upload");
    assert_eq!(downloaded.bytes, b"zulip downloaded attachment bytes");
    assert_eq!(downloaded.content_type.as_deref(), Some("text/plain"));

    let absolute_downloaded = client
        .download_user_upload(&format!("{base_url}/user_uploads/evidence.txt"))
        .await
        .expect("download same-origin absolute user upload");
    assert_eq!(
        absolute_downloaded.bytes,
        b"zulip downloaded attachment bytes"
    );

    let rejected = client
        .download_user_upload("https://example.test/user_uploads/evidence.txt")
        .await
        .expect_err("cross-realm user upload URL must be rejected");
    assert!(matches!(
        rejected,
        makosh_provider_zulip::client::ZulipClientError::InvalidRequest(_)
    ));

    let deleted = client.delete_message(7001).await.expect("delete message");
    assert_eq!(deleted.result, "success");
}

#[tokio::test]
async fn zulip_provider_commands_are_durable_idempotent_and_retryable() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "zulip-command-account",
            CommunicationProviderKind::ZulipBot,
            "Zulip Commands",
            "zulip-command-bot@example.test",
        ))
        .await
        .expect("provider account");

    let store = CommunicationProviderCommandStore::new(pool.clone());
    let command = store
        .enqueue(
            &NewCommunicationProviderCommand::new(
                "zulip-command-send-1",
                "zulip-command-account",
                "zulip",
                "send_stream_message",
                "send:zulip-command-send-1",
                "makosh-frontend",
            )
            .provider_conversation_id("Макошь Lab/Tasks")
            .target_ref(json!({"stream": "Макошь Lab", "topic": "Tasks"}))
            .payload(json!({
                "stream": "Макошь Lab",
                "topic": "Tasks",
                "content": "Надо проверить Zulip provider command queue."
            })),
        )
        .await
        .expect("enqueue Zulip command");
    assert_eq!(command.status, "queued");
    assert_eq!(command.channel_kind, "zulip");
    assert_eq!(command.command_kind, "send_stream_message");

    let duplicate = store
        .enqueue(
            &NewCommunicationProviderCommand::new(
                "zulip-command-send-duplicate",
                "zulip-command-account",
                "zulip",
                "send_stream_message",
                "send:zulip-command-send-1",
                "makosh-frontend",
            )
            .provider_conversation_id("Макошь Lab/Tasks")
            .target_ref(json!({"stream": "Макошь Lab", "topic": "Tasks"}))
            .payload(json!({
                "stream": "Макошь Lab",
                "topic": "Tasks",
                "content": "duplicate should not replace original"
            })),
        )
        .await
        .expect("idempotent enqueue");
    assert_eq!(duplicate.command_id, command.command_id);
    assert_eq!(
        duplicate.payload["content"],
        json!("Надо проверить Zulip provider command queue.")
    );

    let claimed = store
        .claim_due("zulip-command-account", "zulip", Utc::now(), 10)
        .await
        .expect("claim due Zulip commands");
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].command_id, command.command_id);
    assert_eq!(claimed[0].status, "executing");
    assert_eq!(claimed[0].retry_count, 1);

    let failed = store
        .mark_failed(
            &command.command_id,
            "zulip",
            Utc::now(),
            "provider timeout",
            json!({"attempt": 1}),
        )
        .await
        .expect("mark failed")
        .expect("failed command");
    assert_eq!(failed.status, "retrying");
    assert_eq!(failed.last_error.as_deref(), Some("provider timeout"));

    let retry_at = failed.next_attempt_at.expect("scheduled retry time");
    let retried = store
        .claim_due("zulip-command-account", "zulip", retry_at, 10)
        .await
        .expect("claim retrying Zulip commands");
    assert_eq!(retried.len(), 1);
    assert_eq!(retried[0].retry_count, 2);

    let completed = store
        .mark_completed(
            &command.command_id,
            "zulip",
            Utc::now(),
            json!({"provider_message_id": 7001}),
        )
        .await
        .expect("mark completed")
        .expect("completed command");
    assert_eq!(completed.status, "completed");
    assert_eq!(completed.result_payload["provider_message_id"], json!(7001));
    assert!(completed.completed_at.is_some());

    let claimed_after_completion = store
        .claim_due("zulip-command-account", "zulip", Utc::now(), 10)
        .await
        .expect("claim after completion");
    assert!(claimed_after_completion.is_empty());
}

#[tokio::test]
async fn zulip_command_worker_executes_due_stream_command_with_resolved_api_key() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let base_url = spawn_fake_zulip_api().await;
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(
            &NewProviderAccount::new(
                "zulip-worker-account",
                CommunicationProviderKind::ZulipBot,
                "Zulip Worker",
                "bot@example.test",
            )
            .config(json!({"base_url": base_url})),
        )
        .await
        .expect("provider account");
    let mut resolver = InMemorySecretResolver::new();
    bind_zulip_api_key(
        &pool,
        &mut resolver,
        "zulip-worker-account",
        "secret:test:zulip-worker-api-key",
        "zulip-test-key",
    )
    .await;

    let store = CommunicationProviderCommandStore::new(pool.clone());
    let command = store
        .enqueue(
            &NewCommunicationProviderCommand::new(
                "zulip-worker-send-1",
                "zulip-worker-account",
                "zulip",
                "send_stream_message",
                "send:zulip-worker-send-1",
                "makosh-frontend",
            )
            .provider_conversation_id("Макошь Lab/Tasks")
            .target_ref(json!({"stream": "Макошь Lab", "topic": "Tasks"}))
            .payload(json!({
                "stream": "Макошь Lab",
                "topic": "Tasks",
                "content": "Надо проверить retention."
            })),
        )
        .await
        .expect("enqueue Zulip command");

    let worker = ZulipCommandWorker::new(pool.clone(), resolver);
    let report = worker
        .execute_due_for_account("zulip-worker-account", Utc::now(), 10)
        .await
        .expect("execute due Zulip commands");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(report.retrying, 0);
    assert_eq!(report.dead_lettered, 0);

    let commands = store
        .list("zulip-worker-account", "zulip", 10)
        .await
        .expect("list Zulip commands");
    let completed = commands
        .iter()
        .find(|item| item.command_id == command.command_id)
        .expect("completed command");
    assert_eq!(completed.status, "completed");
    assert_eq!(completed.provider_message_id.as_deref(), Some("7001"));
    assert_eq!(completed.reconciliation_status, "awaiting_provider");
    assert_eq!(completed.result_payload["provider_message_id"], json!(7001));
    assert!(completed.completed_at.is_some());
}

#[tokio::test]
async fn zulip_command_worker_uploads_local_attachment_for_stream_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let base_url = spawn_fake_zulip_api().await;
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(
            &NewProviderAccount::new(
                "zulip-worker-upload-account",
                CommunicationProviderKind::ZulipBot,
                "Zulip Worker Upload",
                "bot@example.test",
            )
            .config(json!({"base_url": base_url})),
        )
        .await
        .expect("provider account");
    let mut resolver = InMemorySecretResolver::new();
    bind_zulip_api_key(
        &pool,
        &mut resolver,
        "zulip-worker-upload-account",
        "secret:test:zulip-worker-upload-api-key",
        "zulip-test-key",
    )
    .await;

    let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let local_blob = blob_store
        .put_blob(b"zulip attachment bytes")
        .await
        .expect("local blob");
    let storage = CommunicationStorageStore::new(pool.clone());
    let stored_blob = storage
        .upsert_blob(&NewCommunicationBlob::from_local_blob(&local_blob).content_type("text/plain"))
        .await
        .expect("stored blob metadata");
    let imported = storage
        .upsert_imported_attachment(
            &NewCommunicationAttachmentImport::new(
                "zulip-upload-import-1",
                &stored_blob.blob_id,
                "text/plain",
                local_blob.size_bytes,
                &local_blob.sha256,
                "zulip-upload-test",
            )
            .account_id("zulip-worker-upload-account")
            .channel_kind("zulip")
            .filename("evidence.txt")
            .source_kind("zulip_upload_test")
            .metadata(json!({"test": "zulip_command_worker_upload"})),
        )
        .await
        .expect("imported attachment");

    let store = CommunicationProviderCommandStore::new(pool.clone());
    let command = store
        .enqueue(
            &NewCommunicationProviderCommand::new(
                "zulip-worker-upload-send-1",
                "zulip-worker-upload-account",
                "zulip",
                "send_stream_message_with_upload",
                "send-upload:zulip-worker-upload-send-1",
                "makosh-frontend",
            )
            .provider_conversation_id("Макошь Lab/Tasks")
            .target_ref(json!({"stream": "Макошь Lab", "topic": "Tasks"}))
            .payload(json!({
                "stream": "Макошь Lab",
                "topic": "Tasks",
                "content": "Надо проверить retention.",
                "attachment_id": imported.attachment_id,
                "blob_id": imported.blob_id
            })),
        )
        .await
        .expect("enqueue Zulip upload command");

    let worker = ZulipCommandWorker::new(pool.clone(), resolver);
    let report = worker
        .execute_due_for_account("zulip-worker-upload-account", Utc::now(), 10)
        .await
        .expect("execute upload Zulip command");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(report.retrying, 0);
    assert_eq!(report.dead_lettered, 0);

    let commands = store
        .list("zulip-worker-upload-account", "zulip", 10)
        .await
        .expect("list Zulip commands");
    let completed = commands
        .iter()
        .find(|item| item.command_id == command.command_id)
        .expect("completed command");
    assert_eq!(completed.status, "completed");
    assert_eq!(completed.provider_message_id.as_deref(), Some("7001"));
    assert_eq!(completed.reconciliation_status, "awaiting_provider");
    assert_eq!(completed.result_payload["provider_message_id"], json!(7001));
    assert_eq!(
        completed.result_payload["upload_uri"],
        json!("/user_uploads/evidence.txt")
    );
    assert_eq!(
        completed.result_payload["attachment_id"],
        json!("zulip-upload-import-1")
    );
    assert_eq!(
        completed.result_payload["blob_id"],
        json!(stored_blob.blob_id)
    );
    assert_eq!(completed.result_payload["filename"], json!("evidence.txt"));
    assert_eq!(
        completed.result_payload["content_type"],
        json!("text/plain")
    );
    assert!(completed.completed_at.is_some());
}

#[tokio::test]
async fn zulip_command_worker_sanitizes_provider_errors_before_retrying() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let base_url = spawn_fake_zulip_api().await;
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(
            &NewProviderAccount::new(
                "zulip-worker-failure-account",
                CommunicationProviderKind::ZulipBot,
                "Zulip Worker Failure",
                "bot@example.test",
            )
            .config(json!({"base_url": base_url})),
        )
        .await
        .expect("provider account");
    let mut resolver = InMemorySecretResolver::new();
    bind_zulip_api_key(
        &pool,
        &mut resolver,
        "zulip-worker-failure-account",
        "secret:test:zulip-worker-failure-api-key",
        "zulip-api-key-must-not-leak",
    )
    .await;

    let store = CommunicationProviderCommandStore::new(pool.clone());
    let command = store
        .enqueue(
            &NewCommunicationProviderCommand::new(
                "zulip-worker-send-failure-1",
                "zulip-worker-failure-account",
                "zulip",
                "send_stream_message",
                "send:zulip-worker-send-failure-1",
                "makosh-frontend",
            )
            .provider_conversation_id("Макошь Lab/Tasks")
            .target_ref(json!({"stream": "Макошь Lab", "topic": "Tasks"}))
            .payload(json!({
                "stream": "Макошь Lab",
                "topic": "Tasks",
                "content": "Надо проверить retention."
            })),
        )
        .await
        .expect("enqueue Zulip command");

    let worker = ZulipCommandWorker::new(pool.clone(), resolver);
    let report = worker
        .execute_due_for_account("zulip-worker-failure-account", Utc::now(), 10)
        .await
        .expect("execute due Zulip commands");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.completed, 0);
    assert_eq!(report.retrying, 1);
    assert_eq!(report.dead_lettered, 0);

    let commands = store
        .list("zulip-worker-failure-account", "zulip", 10)
        .await
        .expect("list Zulip commands");
    let retrying = commands
        .iter()
        .find(|item| item.command_id == command.command_id)
        .expect("retrying command");
    assert_eq!(retrying.status, "retrying");
    assert_eq!(
        retrying.last_error.as_deref(),
        Some("Zulip API returned HTTP 401")
    );
    assert!(
        !format!("{retrying:?}").contains("zulip-api-key-must-not-leak"),
        "provider command debug output leaked API key"
    );
}

#[tokio::test]
async fn zulip_command_worker_executes_due_commands_across_zulip_accounts() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let base_url = spawn_fake_zulip_api().await;
    let account_store = CommunicationProviderAccountStore::new(pool.clone());
    let mut resolver = InMemorySecretResolver::new();

    for (account_id, external_account_id) in [
        ("zulip-worker-multi-a", "bot+a@example.test"),
        ("zulip-worker-multi-b", "bot+b@example.test"),
    ] {
        account_store
            .upsert(
                &NewProviderAccount::new(
                    account_id,
                    CommunicationProviderKind::ZulipBot,
                    format!("Zulip Multi {account_id}"),
                    external_account_id,
                )
                .config(json!({"base_url": base_url})),
            )
            .await
            .expect("provider account");
        bind_zulip_api_key(
            &pool,
            &mut resolver,
            account_id,
            &format!("secret:test:{account_id}:zulip-api-key"),
            "zulip-test-key",
        )
        .await;
    }

    let store = CommunicationProviderCommandStore::new(pool.clone());
    for account_id in ["zulip-worker-multi-a", "zulip-worker-multi-b"] {
        store
            .enqueue(
                &NewCommunicationProviderCommand::new(
                    format!("{account_id}-send-1"),
                    account_id,
                    "zulip",
                    "send_stream_message",
                    format!("send:{account_id}:1"),
                    "makosh-frontend",
                )
                .provider_conversation_id("Макошь Lab/Tasks")
                .target_ref(json!({"stream": "Макошь Lab", "topic": "Tasks"}))
                .payload(json!({
                    "stream": "Макошь Lab",
                    "topic": "Tasks",
                    "content": "Надо проверить retention."
                })),
            )
            .await
            .expect("enqueue Zulip command");
    }

    let worker = ZulipCommandWorker::new(pool.clone(), resolver);
    let report = worker
        .execute_due(Utc::now(), 10)
        .await
        .expect("execute due Zulip commands");

    assert_eq!(report.accounts_scanned, 2);
    assert_eq!(report.accounts_failed, 0);
    assert_eq!(report.claimed, 2);
    assert_eq!(report.completed, 2);

    for account_id in ["zulip-worker-multi-a", "zulip-worker-multi-b"] {
        let commands = store
            .list(account_id, "zulip", 10)
            .await
            .expect("list Zulip commands");
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].status, "completed");
        assert_eq!(
            commands[0].result_payload["provider_message_id"],
            json!(7001)
        );
    }
}

#[tokio::test]
async fn zulip_event_ingest_worker_polls_queue_records_raw_signal_and_checkpoint() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let base_url = spawn_fake_zulip_api().await;
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(
            &NewProviderAccount::new(
                "zulip-ingest-account",
                CommunicationProviderKind::ZulipBot,
                "Zulip Ingest",
                "bot@example.test",
            )
            .config(json!({"base_url": base_url})),
        )
        .await
        .expect("provider account");
    let mut resolver = InMemorySecretResolver::default();
    bind_zulip_api_key(
        &pool,
        &mut resolver,
        "zulip-ingest-account",
        "secret-zulip-ingest-api-key",
        "zulip-test-key",
    )
    .await;

    let worker = ZulipEventIngestWorker::new(pool.clone(), resolver);
    let report = worker
        .poll_account("zulip-ingest-account", Utc::now())
        .await
        .expect("poll Zulip event queue");

    assert_eq!(report.accounts_scanned, 1);
    assert_eq!(report.queues_registered, 1);
    assert_eq!(report.events_received, 1);
    assert_eq!(report.raw_records_recorded, 1);
    assert_eq!(report.accepted_signals, 1);

    let checkpoint = CommunicationIngestionStore::new(pool.clone())
        .checkpoint("zulip-ingest-account", "zulip:event_queue")
        .await
        .expect("Zulip event checkpoint")
        .expect("Zulip event checkpoint exists");
    assert_eq!(checkpoint.checkpoint["queue_id"], json!("zulip-fake-queue"));
    assert_eq!(checkpoint.checkpoint["last_event_id"], json!(42));

    let raw_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM event_log
        WHERE event_type = 'signal.raw.zulip.message.observed'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("raw signal count");
    assert_eq!(raw_signal_count, 1);

    let accepted_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM event_log
        WHERE event_type = 'signal.accepted.zulip.message'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("accepted signal count");
    assert_eq!(accepted_signal_count, 1);

    let second_report = worker
        .poll_account("zulip-ingest-account", Utc::now())
        .await
        .expect("poll Zulip event queue again");
    assert_eq!(second_report.queues_registered, 0);
    assert_eq!(second_report.events_received, 0);

    let repeated_raw_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM event_log
        WHERE event_type = 'signal.raw.zulip.message.observed'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("repeated raw signal count");
    assert_eq!(repeated_raw_signal_count, 1);
}

#[tokio::test]
async fn zulip_event_ingest_worker_reregisters_expired_queue_checkpoint() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let base_url = spawn_fake_zulip_api().await;
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(
            &NewProviderAccount::new(
                "zulip-expired-queue-account",
                CommunicationProviderKind::ZulipBot,
                "Zulip Expired Queue",
                "bot@example.test",
            )
            .config(json!({"base_url": base_url})),
        )
        .await
        .expect("provider account");
    let ingestion = CommunicationIngestionStore::new(pool.clone());
    ingestion
        .save_checkpoint(&NewIngestionCheckpoint::new(
            "zulip-expired-queue-account",
            "zulip:event_queue",
            json!({
                "queue_id": "expired-zulip-queue",
                "last_event_id": 41
            }),
        ))
        .await
        .expect("expired queue checkpoint");
    let mut resolver = InMemorySecretResolver::default();
    bind_zulip_api_key(
        &pool,
        &mut resolver,
        "zulip-expired-queue-account",
        "secret-zulip-expired-queue-api-key",
        "zulip-test-key",
    )
    .await;

    let worker = ZulipEventIngestWorker::new(pool.clone(), resolver);
    let report = worker
        .poll_account("zulip-expired-queue-account", Utc::now())
        .await
        .expect("poll Zulip expired event queue");

    assert_eq!(report.accounts_scanned, 1);
    assert_eq!(report.queues_registered, 1);
    assert_eq!(report.events_received, 1);
    assert_eq!(report.raw_records_recorded, 1);
    assert_eq!(report.accepted_signals, 1);
    assert_eq!(report.checkpoints_saved, 2);

    let checkpoint = ingestion
        .checkpoint("zulip-expired-queue-account", "zulip:event_queue")
        .await
        .expect("Zulip event checkpoint")
        .expect("Zulip event checkpoint exists");
    assert_eq!(checkpoint.checkpoint["queue_id"], json!("zulip-fake-queue"));
    assert_eq!(checkpoint.checkpoint["last_event_id"], json!(42));

    let raw_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM event_log
        WHERE event_type = 'signal.raw.zulip.message.observed'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("raw signal count");
    assert_eq!(raw_signal_count, 1);
}

#[tokio::test]
async fn zulip_event_ingest_worker_sanitizes_provider_api_errors() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let base_url = spawn_fake_zulip_api().await;
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(
            &NewProviderAccount::new(
                "zulip-ingest-failure-account",
                CommunicationProviderKind::ZulipBot,
                "Zulip Ingest Failure",
                "bot@example.test",
            )
            .config(json!({"base_url": base_url})),
        )
        .await
        .expect("provider account");
    let mut resolver = InMemorySecretResolver::default();
    bind_zulip_api_key(
        &pool,
        &mut resolver,
        "zulip-ingest-failure-account",
        "secret-zulip-ingest-failure-api-key",
        "zulip-api-key-must-not-leak",
    )
    .await;

    let worker = ZulipEventIngestWorker::new(pool, resolver);
    let error = worker
        .poll_account("zulip-ingest-failure-account", Utc::now())
        .await
        .expect_err("provider API error");
    let public_error = error.to_string();

    assert!(public_error.contains("HTTP 401"));
    assert!(!public_error.contains("zulip-api-key-must-not-leak"));
    assert!(!public_error.contains("provider echoed"));
}

#[tokio::test]
async fn zulip_provider_observation_reconciles_completed_send_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let base_url = spawn_fake_zulip_api().await;
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(
            &NewProviderAccount::new(
                "zulip-reconcile-account",
                CommunicationProviderKind::ZulipBot,
                "Zulip Reconcile",
                "bot@example.test",
            )
            .config(json!({"base_url": base_url})),
        )
        .await
        .expect("provider account");
    let mut resolver = InMemorySecretResolver::new();
    bind_zulip_api_key(
        &pool,
        &mut resolver,
        "zulip-reconcile-account",
        "secret:test:zulip-reconcile-api-key",
        "zulip-test-key",
    )
    .await;

    let store = CommunicationProviderCommandStore::new(pool.clone());
    let command = store
        .enqueue(
            &NewCommunicationProviderCommand::new(
                "zulip-reconcile-send-1",
                "zulip-reconcile-account",
                "zulip",
                "send_stream_message",
                "send:zulip-reconcile-send-1",
                "makosh-frontend",
            )
            .provider_conversation_id("Макошь Lab/Tasks")
            .target_ref(json!({"stream": "Макошь Lab", "topic": "Tasks"}))
            .payload(json!({
                "stream": "Макошь Lab",
                "topic": "Tasks",
                "content": "Надо проверить retention."
            })),
        )
        .await
        .expect("enqueue Zulip command");

    ZulipCommandWorker::new(pool.clone(), resolver)
        .execute_due_for_account("zulip-reconcile-account", Utc::now(), 10)
        .await
        .expect("execute Zulip command");

    let awaiting = store
        .list("zulip-reconcile-account", "zulip", 10)
        .await
        .expect("list commands")
        .into_iter()
        .find(|item| item.command_id == command.command_id)
        .expect("completed command awaiting observation");
    assert_eq!(awaiting.status, "completed");
    assert_eq!(awaiting.provider_message_id.as_deref(), Some("7001"));
    assert_eq!(awaiting.reconciliation_status, "awaiting_provider");

    let event: ZulipEvent = serde_json::from_value(json!({
        "id": 4201,
        "type": "message",
        "message": {
            "id": 7001,
            "content": "Надо проверить retention.",
            "sender_email": "bot@example.test",
            "sender_full_name": "Макошь Bot",
            "stream_id": 10,
            "display_recipient": "Макошь Lab",
            "topic": "Tasks"
        }
    }))
    .expect("valid Zulip observed message");
    let new_raw_record = observation_to_raw_communication_record(
        map_zulip_event_to_observation(
            &event,
            &ZulipEventMappingContext::new(
                "zulip-reconcile-account",
                "http://localhost:8080",
                Utc::now(),
            ),
        )
        .expect("map observed Zulip message"),
    );
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(&new_raw_record)
        .await
        .expect("record raw Zulip observation");
    let accepted_event = dispatch_zulip_raw_signal(pool.clone(), &raw_record)
        .await
        .expect("dispatch raw Zulip signal")
        .expect("accepted Zulip observation");

    reconcile_zulip_provider_observation_event(
        pool.clone(),
        InMemoryEventBus::new(),
        StoredEventEnvelope {
            position: 0,
            event: accepted_event.clone(),
        },
    )
    .await
    .expect("reconcile Zulip provider observation");

    let reconciled = store
        .list("zulip-reconcile-account", "zulip", 10)
        .await
        .expect("list reconciled commands")
        .into_iter()
        .find(|item| item.command_id == command.command_id)
        .expect("reconciled command");
    assert_eq!(reconciled.status, "completed");
    assert_eq!(reconciled.reconciliation_status, "observed");
    assert_eq!(
        reconciled.provider_state["observed_via"],
        json!("signal_hub_accepted_event")
    );
    assert_eq!(
        reconciled.provider_state["raw_record_id"],
        raw_record.raw_record_id
    );
    assert!(reconciled.provider_observed_at.is_some());
    assert!(reconciled.reconciled_at.is_some());

    let reconciled_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM event_log
        WHERE event_type = 'zulip.command.reconciled'
          AND causation_id = $1
        "#,
    )
    .bind(&accepted_event.event_id)
    .fetch_one(&pool)
    .await
    .expect("Zulip command reconciled event count");
    assert_eq!(reconciled_event_count, 1);

    reconcile_zulip_provider_observation_event(
        pool.clone(),
        InMemoryEventBus::new(),
        StoredEventEnvelope {
            position: 0,
            event: accepted_event.clone(),
        },
    )
    .await
    .expect("reconcile Zulip provider observation again");

    let repeated_reconciled_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM event_log
        WHERE event_type = 'zulip.command.reconciled'
          AND causation_id = $1
        "#,
    )
    .bind(&accepted_event.event_id)
    .fetch_one(&pool)
    .await
    .expect("repeated Zulip command reconciled event count");
    assert_eq!(repeated_reconciled_event_count, 1);
}

#[tokio::test]
async fn zulip_raw_signal_projects_message_and_is_idempotent() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "zulip-trace-account",
            CommunicationProviderKind::ZulipBot,
            "Zulip Trace",
            "zulip-trace-bot@example.test",
        ))
        .await
        .expect("provider account");

    let event: ZulipEvent = serde_json::from_value(json!({
        "id": 42,
        "type": "message",
        "message": {
            "id": 7001,
            "content": "Надо проверить backup retention до пятницы.",
            "sender_email": "bot@example.test",
            "sender_full_name": "Макошь Bot",
            "stream_id": 10,
            "display_recipient": "Макошь Lab",
            "topic": "Tasks"
        }
    }))
    .expect("valid Zulip event");
    let mapping_context =
        ZulipEventMappingContext::new("zulip-trace-account", "http://localhost:8080", Utc::now())
            .with_import_batch_id("zulip-trace-batch")
            .with_scenario_id("zulip-trace-scenario");
    let new_raw_record = observation_to_raw_communication_record(
        map_zulip_event_to_observation(&event, &mapping_context)
            .expect("map Zulip event to raw record"),
    );
    assert_eq!(
        zulip_raw_signal_event_type("message"),
        "signal.raw.zulip.message.observed"
    );

    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let raw_record = ingestion
        .record_raw_source(&new_raw_record)
        .await
        .expect("record Zulip raw source");
    let accepted_event = dispatch_zulip_raw_signal(pool.clone(), &raw_record)
        .await
        .expect("dispatch Zulip raw signal")
        .expect("accepted Zulip signal");
    assert_eq!(accepted_event.event_type, "signal.accepted.zulip.message");

    let projected = consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted Zulip signal")
        .expect("projected Zulip message");
    assert_eq!(projected.channel_kind, "zulip");
    assert_eq!(projected.provider_record_id, "7001");
    assert_eq!(projected.subject, "Макошь Lab / Tasks");
    assert_eq!(projected.sender_display_name.as_deref(), Some("Макошь Bot"));

    let repeated_raw_record = ingestion
        .record_raw_source(&new_raw_record)
        .await
        .expect("record repeated Zulip raw source");
    let repeated_accepted_event = dispatch_zulip_raw_signal(pool.clone(), &repeated_raw_record)
        .await
        .expect("dispatch repeated Zulip raw signal")
        .expect("accepted repeated Zulip signal");
    consume_accepted_signal_event(pool.clone(), &repeated_accepted_event)
        .await
        .expect("project repeated accepted Zulip signal");

    assert_eq!(repeated_raw_record.raw_record_id, raw_record.raw_record_id);
    assert_eq!(repeated_accepted_event.event_id, accepted_event.event_id);

    let message_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM communication_messages
        WHERE raw_record_id = $1
        "#,
    )
    .bind(&raw_record.raw_record_id)
    .fetch_one(&pool)
    .await
    .expect("communication message count");
    assert_eq!(message_count, 1);

    let raw_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM event_log
        WHERE event_type = 'signal.raw.zulip.message.observed'
          AND correlation_id = $1
        "#,
    )
    .bind(&raw_record.observation_id)
    .fetch_one(&pool)
    .await
    .expect("raw signal count");
    assert_eq!(raw_event_count, 1);

    let communication_recorded_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM event_log
        WHERE event_type = 'communication.message.recorded'
          AND causation_id = $1
        "#,
    )
    .bind(&accepted_event.event_id)
    .fetch_one(&pool)
    .await
    .expect("communication recorded event count");
    assert_eq!(communication_recorded_count, 1);
}

#[tokio::test]
async fn zulip_raw_signal_projects_direct_message_without_stream_topic_shape() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "zulip-direct-trace-account",
            CommunicationProviderKind::ZulipBot,
            "Zulip Direct Trace",
            "zulip-direct-bot@example.test",
        ))
        .await
        .expect("provider account");

    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let mapping_context = ZulipEventMappingContext::new(
        "zulip-direct-trace-account",
        "http://localhost:8080",
        Utc::now(),
    )
    .with_import_batch_id("zulip-direct-trace-batch")
    .with_scenario_id("zulip-direct-trace-scenario");

    let (_, _, projected) = record_dispatch_consume_zulip_event(
        &pool,
        &ingestion,
        &mapping_context,
        json!({
            "id": 43,
            "type": "message",
            "message": {
                "id": 7005,
                "type": "private",
                "content": "Direct note for Макошь.",
                "sender_email": "alice@example.test",
                "sender_full_name": "Alice",
                "display_recipient": [
                    {
                        "id": 101,
                        "email": "alice@example.test",
                        "full_name": "Alice"
                    },
                    {
                        "id": 102,
                        "email": "bot@example.test",
                        "full_name": "Макошь Bot"
                    }
                ]
            }
        }),
    )
    .await;

    assert_eq!(projected.channel_kind, "zulip");
    assert_eq!(projected.provider_record_id, "7005");
    assert_eq!(projected.subject, "Direct / Alice, Макошь Bot");
    assert_eq!(
        projected.recipients,
        vec![
            "alice@example.test".to_owned(),
            "bot@example.test".to_owned()
        ]
    );
    assert_ne!(projected.subject, "Zulip / message");
    assert!(
        projected
            .conversation_id
            .as_deref()
            .unwrap_or_default()
            .starts_with("zulip:direct_conversation:")
    );
    assert_eq!(projected.sender_display_name.as_deref(), Some("Alice"));
    assert_eq!(projected.message_metadata["message_type"], json!("private"));
    assert_eq!(
        projected.message_metadata["direct_recipients"][0]["email"],
        json!("alice@example.test")
    );
}

#[tokio::test]
async fn zulip_attachment_metadata_remains_evidence_only_until_bytes_are_transferred() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "zulip-attachment-account",
            CommunicationProviderKind::ZulipBot,
            "Zulip Attachments",
            "zulip-attachment-bot@example.test",
        ))
        .await
        .expect("provider account");

    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let mapping_context = ZulipEventMappingContext::new(
        "zulip-attachment-account",
        "http://localhost:8080",
        Utc::now(),
    )
    .with_import_batch_id("zulip-attachment-batch")
    .with_scenario_id("zulip-attachment-evidence-scenario");

    let event_value = json!({
        "id": 46,
        "type": "message",
        "message": {
            "id": 7002,
            "content": "См. evidence.pdf.",
            "sender_email": "bot@example.test",
            "sender_full_name": "Макошь Bot",
            "stream_id": 10,
            "display_recipient": "Макошь Lab",
            "topic": "Evidence",
            "attachments": [
                {
                    "id": "zulip-file-1",
                    "name": "evidence.pdf",
                    "content_type": "application/pdf",
                    "size": 2048,
                    "url": "/user_uploads/1/evidence.pdf",
                    "path_id": "1/evidence.pdf"
                }
            ]
        }
    });

    let (raw_record_id, accepted_event, projected) = record_dispatch_consume_zulip_event(
        &pool,
        &ingestion,
        &mapping_context,
        event_value.clone(),
    )
    .await;

    assert_eq!(accepted_event.event_type, "signal.accepted.zulip.message");
    assert_eq!(
        projected.message_metadata["attachments"][0]["provider_attachment_id"],
        json!("zulip-file-1")
    );
    assert_eq!(
        projected.message_metadata["attachments"][0]["filename"],
        json!("evidence.pdf")
    );
    assert_eq!(
        projected.message_metadata["attachments"][0]["bytes_state"],
        json!("not_transferred")
    );
    assert_eq!(
        projected.message_metadata["attachments"][0]["scan_status"],
        json!("not_scanned")
    );
    assert_eq!(
        projected.message_metadata["attachment_state"]["materialization_state"],
        json!("not_materialized")
    );

    let attachment_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM communication_attachments
        WHERE raw_record_id = $1
           OR message_id = $2
        "#,
    )
    .bind(&raw_record_id)
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("Zulip canonical attachment count");
    assert_eq!(attachment_count, 0);

    let (repeated_raw_record_id, repeated_accepted_event, repeated_projection) =
        record_dispatch_consume_zulip_event(&pool, &ingestion, &mapping_context, event_value).await;
    assert_eq!(repeated_raw_record_id, raw_record_id);
    assert_eq!(repeated_accepted_event.event_id, accepted_event.event_id);
    assert_eq!(repeated_projection.message_id, projected.message_id);

    let repeated_attachment_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM communication_attachments
        WHERE raw_record_id = $1
           OR message_id = $2
        "#,
    )
    .bind(&raw_record_id)
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("repeated Zulip canonical attachment count");
    assert_eq!(repeated_attachment_count, 0);
}

#[tokio::test]
async fn zulip_attachment_bytes_materialize_after_safe_transfer() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let blob_root = tempdir().expect("blob root");
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "zulip-materialize-account",
            CommunicationProviderKind::ZulipBot,
            "Zulip Materialize",
            "zulip-materialize-bot@example.test",
        ))
        .await
        .expect("provider account");

    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let mapping_context = ZulipEventMappingContext::new(
        "zulip-materialize-account",
        "http://localhost:8080",
        Utc::now(),
    )
    .with_import_batch_id("zulip-materialize-batch")
    .with_scenario_id("zulip-attachment-materialize-scenario");

    let (_, _, projected) = record_dispatch_consume_zulip_event(
        &pool,
        &ingestion,
        &mapping_context,
        json!({
            "id": 48,
            "type": "message",
            "message": {
                "id": 7003,
                "content": "См. вложение.",
                "sender_email": "bot@example.test",
                "sender_full_name": "Макошь Bot",
                "stream_id": 10,
                "display_recipient": "Макошь Lab",
                "topic": "Evidence",
                "attachments": [
                    {
                        "id": "zulip-file-2",
                        "name": "evidence.pdf",
                        "content_type": "application/pdf",
                        "size": 26,
                        "url": "/user_uploads/1/evidence.pdf",
                        "path_id": "1/evidence.pdf"
                    }
                ]
            }
        }),
    )
    .await;

    let request = ZulipAttachmentBytes {
        account_id: "zulip-materialize-account".to_owned(),
        provider_message_id: "7003".to_owned(),
        provider_attachment_id: "zulip-file-2".to_owned(),
        filename: Some("evidence.pdf".to_owned()),
        content_type: Some("application/pdf".to_owned()),
        bytes: b"%PDF-1.7 zulip evidence\n".to_vec(),
    };

    let message_lookup = ProviderChannelMessageStore::new(pool.clone());
    let materialized =
        persist_zulip_attachment_bytes(pool.clone(), &message_lookup, &request, blob_root.path())
            .await
            .expect("materialize Zulip attachment bytes");
    assert_eq!(materialized.message_id, projected.message_id);
    assert_eq!(materialized.raw_record_id, projected.raw_record_id);
    assert_eq!(materialized.provider_message_id, "7003");
    assert_eq!(materialized.provider_attachment_id, "zulip-file-2");
    assert_eq!(materialized.content_type, "application/pdf");
    assert_eq!(materialized.scan_status, "not_scanned");
    assert_eq!(
        materialized.message_metadata["attachments"][0]["materialization_state"],
        json!("materialized")
    );
    assert_eq!(
        materialized.message_metadata["attachments"][0]["bytes_state"],
        json!("transferred")
    );
    assert_eq!(
        materialized.message_metadata["attachments"][0]["blob_id"],
        json!(materialized.blob_id)
    );
    assert_eq!(
        materialized.message_metadata["attachment_state"]["materialization_state"],
        json!("materialized")
    );

    let attachment_row = sqlx::query(
        r#"
        SELECT provider_attachment_id, filename, content_type, scan_status
        FROM communication_attachments
        WHERE message_id = $1
        "#,
    )
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("materialized attachment row");
    assert_eq!(
        attachment_row
            .try_get::<String, _>("provider_attachment_id")
            .expect("provider attachment id"),
        "zulip-file-2"
    );
    assert_eq!(
        attachment_row
            .try_get::<Option<String>, _>("filename")
            .expect("filename"),
        Some("evidence.pdf".to_owned())
    );
    assert_eq!(
        attachment_row
            .try_get::<String, _>("content_type")
            .expect("content type"),
        "application/pdf"
    );
    assert_eq!(
        attachment_row
            .try_get::<String, _>("scan_status")
            .expect("scan status"),
        "not_scanned"
    );

    let repeated =
        persist_zulip_attachment_bytes(pool.clone(), &message_lookup, &request, blob_root.path())
            .await
            .expect("repeat materialization is idempotent");
    assert_eq!(repeated.attachment_id, materialized.attachment_id);
    assert_eq!(repeated.blob_id, materialized.blob_id);

    let attachment_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM communication_attachments
        WHERE message_id = $1
        "#,
    )
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("materialized attachment count");
    assert_eq!(attachment_count, 1);
}

#[tokio::test]
async fn zulip_attachment_download_worker_materializes_pending_user_upload() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let base_url = spawn_fake_zulip_api().await;
    let blob_root = tempdir().expect("blob root");
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(
            &NewProviderAccount::new(
                "zulip-download-account",
                CommunicationProviderKind::ZulipBot,
                "Zulip Download",
                "bot@example.test",
            )
            .config(json!({"base_url": base_url})),
        )
        .await
        .expect("provider account");
    let mut resolver = InMemorySecretResolver::default();
    bind_zulip_api_key(
        &pool,
        &mut resolver,
        "zulip-download-account",
        "secret-zulip-download-api-key",
        "zulip-test-key",
    )
    .await;

    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let mapping_context = ZulipEventMappingContext::new(
        "zulip-download-account",
        "http://localhost:8080",
        Utc::now(),
    )
    .with_import_batch_id("zulip-download-batch")
    .with_scenario_id("zulip-attachment-download-scenario");

    let (_, _, projected) = record_dispatch_consume_zulip_event(
        &pool,
        &ingestion,
        &mapping_context,
        json!({
            "id": 49,
            "type": "message",
            "message": {
                "id": 7004,
                "content": "См. вложение из Zulip.",
                "sender_email": "bot@example.test",
                "sender_full_name": "Макошь Bot",
                "stream_id": 10,
                "display_recipient": "Макошь Lab",
                "topic": "Evidence",
                "attachments": [
                    {
                        "id": "zulip-file-download",
                        "name": "evidence.txt",
                        "content_type": "text/plain",
                        "size": 32,
                        "url": "/user_uploads/evidence.txt",
                        "path_id": "evidence.txt"
                    }
                ]
            }
        }),
    )
    .await;

    let worker =
        ZulipAttachmentDownloadWorker::new(pool.clone(), resolver).with_blob_root(blob_root.path());
    let report = worker
        .download_due_for_account("zulip-download-account", Utc::now(), 10)
        .await
        .expect("download pending Zulip attachment");

    assert_eq!(report.accounts_scanned, 1);
    assert_eq!(report.accounts_failed, 0);
    assert_eq!(report.candidates_seen, 1);
    assert_eq!(report.attachments_downloaded, 1);
    assert_eq!(report.attachments_materialized, 1);
    assert_eq!(report.attachments_failed, 0);

    let attachment_row = sqlx::query(
        r#"
        SELECT provider_attachment_id, filename, content_type, scan_status, size_bytes
        FROM communication_attachments
        WHERE message_id = $1
        "#,
    )
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("downloaded Zulip attachment row");
    assert_eq!(
        attachment_row
            .try_get::<String, _>("provider_attachment_id")
            .expect("provider attachment id"),
        "zulip-file-download"
    );
    assert_eq!(
        attachment_row
            .try_get::<Option<String>, _>("filename")
            .expect("filename"),
        Some("evidence.txt".to_owned())
    );
    assert_eq!(
        attachment_row
            .try_get::<String, _>("content_type")
            .expect("content type"),
        "text/plain"
    );
    assert_eq!(
        attachment_row
            .try_get::<String, _>("scan_status")
            .expect("scan status"),
        "not_scanned"
    );
    assert_eq!(
        attachment_row
            .try_get::<i64, _>("size_bytes")
            .expect("size bytes"),
        "zulip downloaded attachment bytes".len() as i64
    );

    let metadata: Value = sqlx::query_scalar(
        "SELECT message_metadata FROM communication_messages WHERE message_id = $1",
    )
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("message metadata");
    assert_eq!(
        metadata["attachments"][0]["materialization_state"],
        json!("materialized")
    );
    assert_eq!(
        metadata["attachments"][0]["bytes_state"],
        json!("transferred")
    );
    assert_eq!(
        metadata["attachment_state"]["materialization_state"],
        json!("materialized")
    );

    let repeated = worker
        .download_due_for_account("zulip-download-account", Utc::now(), 10)
        .await
        .expect("repeat Zulip attachment download worker");
    assert_eq!(repeated.candidates_seen, 0);
    assert_eq!(repeated.attachments_materialized, 0);

    let attachment_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM communication_attachments
        WHERE message_id = $1
        "#,
    )
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("downloaded Zulip attachment count");
    assert_eq!(attachment_count, 1);
}

#[tokio::test]
async fn zulip_message_can_feed_review_task_candidate_without_auto_creating_task() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "zulip-review-account",
            CommunicationProviderKind::ZulipBot,
            "Zulip Review",
            "zulip-review-bot@example.test",
        ))
        .await
        .expect("provider account");

    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let mapping_context =
        ZulipEventMappingContext::new("zulip-review-account", "http://localhost:8080", Utc::now())
            .with_import_batch_id("zulip-review-batch")
            .with_scenario_id("zulip-message-to-review-task-candidate");

    let (_, accepted_event, projected) = record_dispatch_consume_zulip_event(
        &pool,
        &ingestion,
        &mapping_context,
        json!({
            "id": 47,
            "type": "message",
            "message": {
                "id": 7003,
                "content": "Надо проверить backup retention до пятницы.",
                "sender_email": "bot@example.test",
                "sender_full_name": "Макошь Bot",
                "stream_id": 10,
                "display_recipient": "Макошь Lab",
                "topic": "Tasks"
            }
        }),
    )
    .await;
    assert_eq!(accepted_event.event_type, "signal.accepted.zulip.message");

    let refreshed = refresh_message_task_candidates_into_review(
        &pool,
        std::slice::from_ref(&projected.message_id),
    )
    .await
    .expect("refresh Zulip task candidates into review");
    assert_eq!(refreshed, 1);

    let candidate_row = sqlx::query(
        r#"
        SELECT task_candidate_id, title, candidate_kind, review_state, due_text, evidence_excerpt
        FROM task_candidates
        WHERE source_id = $1
          AND source_kind = 'observation'
        "#,
    )
    .bind(&projected.observation_id)
    .fetch_one(&pool)
    .await
    .expect("Zulip task candidate row");
    let task_candidate_id = candidate_row
        .try_get::<String, _>("task_candidate_id")
        .expect("task_candidate_id");
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
        "Надо проверить backup retention до пятницы."
    );
    assert_eq!(
        candidate_row
            .try_get::<Option<String>, _>("due_text")
            .expect("due_text")
            .as_deref(),
        Some("пятницы")
    );
    assert_eq!(
        candidate_row
            .try_get::<String, _>("evidence_excerpt")
            .expect("evidence_excerpt"),
        "Надо проверить backup retention до пятницы."
    );

    let review_row = sqlx::query(
        r#"
        SELECT item.item_kind, item.status, item.title, item.metadata
        FROM review_items item
        JOIN review_item_evidence evidence
          ON evidence.review_item_id = item.review_item_id
        WHERE evidence.observation_id = $1
          AND item.item_kind = 'potential_task'
        "#,
    )
    .bind(&projected.observation_id)
    .fetch_one(&pool)
    .await
    .expect("Zulip task review item");
    assert_eq!(
        review_row
            .try_get::<String, _>("item_kind")
            .expect("item_kind"),
        "potential_task"
    );
    assert_eq!(
        review_row.try_get::<String, _>("status").expect("status"),
        "new"
    );
    assert_eq!(
        review_row.try_get::<String, _>("title").expect("title"),
        "Надо проверить backup retention до пятницы."
    );
    let review_metadata = review_row
        .try_get::<Value, _>("metadata")
        .expect("review metadata");
    assert_eq!(
        review_metadata["task_candidate_id"],
        json!(task_candidate_id)
    );
    assert_eq!(review_metadata["mirrored_from"], json!("task_candidates"));

    let task_count: i64 =
        sqlx::query_scalar("SELECT count(*)::BIGINT FROM tasks WHERE source_id = $1")
            .bind(&projected.message_id)
            .fetch_one(&pool)
            .await
            .expect("auto-created task count");
    let obligation_count: i64 =
        sqlx::query_scalar("SELECT count(*)::BIGINT FROM obligations WHERE statement = $1")
            .bind("Надо проверить backup retention до пятницы.")
            .fetch_one(&pool)
            .await
            .expect("auto-created obligation count");
    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn zulip_message_drives_review_attention_card_and_context_pack_trace_chain() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let pool = ctx.pool().clone();
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "zulip-review-scene-account",
            CommunicationProviderKind::ZulipBot,
            "Zulip Review Scene",
            "zulip-review-scene-bot@example.test",
        ))
        .await
        .expect("provider account");

    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let mapping_context = ZulipEventMappingContext::new(
        "zulip-review-scene-account",
        "http://localhost:8080",
        Utc::now(),
    )
    .with_import_batch_id("zulip-review-scene-batch")
    .with_scenario_id("zulip-message-to-review-task-candidate");

    let (_, accepted_event, projected) = record_dispatch_consume_zulip_event(
        &pool,
        &ingestion,
        &mapping_context,
        json!({
            "id": 48,
            "type": "message",
            "message": {
                "id": 7101,
                "content": "Надо проверить резервные копии до пятницы.",
                "sender_email": "bot@example.test",
                "sender_full_name": "Макошь Bot",
                "stream_id": 10,
                "display_recipient": "Макошь Lab",
                "topic": "Tasks"
            }
        }),
    )
    .await;
    assert_eq!(accepted_event.event_type, "signal.accepted.zulip.message");
    assert!(!projected.observation_id.is_empty());
    assert!(!projected.message_id.is_empty());

    let refreshed = refresh_message_task_candidates_into_review(
        &pool,
        std::slice::from_ref(&projected.message_id),
    )
    .await
    .expect("refresh Zulip task candidates into review");
    assert_eq!(refreshed, 1);

    let review_item_id = sqlx::query_scalar::<_, String>(
        r#"
        SELECT item.review_item_id
        FROM review_items item
        JOIN review_item_evidence evidence
          ON evidence.review_item_id = item.review_item_id
        WHERE evidence.observation_id = $1
          AND item.item_kind = 'potential_task'
          AND item.status = 'new'
        LIMIT 1
        "#,
    )
    .bind(&projected.observation_id)
    .fetch_one(&pool)
    .await
    .expect("resolve Zulip review item id");

    let app = {
        let database = Database::connect(Some(&database_url))
            .await
            .expect("database connection");
        build_router_with_database(
            makosh_backend_testkit::app::config_with_secret_and_database_url(
                LOCAL_API_TOKEN,
                database_url.as_str(),
            ),
            database,
        )
    };

    let attention_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/review/attention-cards?status=active&limit=20")
                .header("x-makosh-secret", LOCAL_API_TOKEN)
                .body(Body::empty())
                .expect("attention cards request"),
        )
        .await
        .expect("attention cards response");
    assert_eq!(attention_response.status(), StatusCode::OK);
    let attention_body = response_json(attention_response).await;
    let cards = attention_body["cards"]
        .as_array()
        .expect("attention cards array");
    let target_card = cards
        .iter()
        .find(|card| {
            card["review_item_ids"].as_array().is_some_and(|ids| {
                ids.iter().any(|id| {
                    id.as_str()
                        .is_some_and(|value| value == review_item_id.as_str())
                })
            })
        })
        .expect("attention card for Zulip-driven item");
    let attention_trace_id = target_card["trace_id"]
        .as_str()
        .expect("attention card trace id");
    assert!(!attention_trace_id.is_empty());
    let card_explainability = target_card["explainability"]["why_this_matters"]
        .as_str()
        .expect("attention explainability");
    assert!(
        card_explainability.contains("potential task"),
        "explainability should describe item intent"
    );

    let context_pack_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/review/items/{review_item_id}/context-pack"
                ))
                .header("x-makosh-secret", LOCAL_API_TOKEN)
                .body(Body::empty())
                .expect("review context-pack request"),
        )
        .await
        .expect("review context-pack response");
    assert_eq!(context_pack_response.status(), StatusCode::OK);
    let context_pack = response_json(context_pack_response).await;
    assert_eq!(context_pack["kind"], json!("review"));
    assert_eq!(context_pack["subject_id"], json!(review_item_id));
    let context_trace_id = context_pack["content"]["trace"]["trace_id"]
        .as_str()
        .expect("context pack trace id");
    assert!(!context_trace_id.is_empty());
    assert_eq!(
        context_pack["content"]["review_item"]["review_item_id"],
        json!(review_item_id)
    );
    assert!(
        !context_pack["content"]["evidence"]
            .as_array()
            .expect("context pack evidence")
            .is_empty()
    );
    assert_eq!(attention_trace_id, context_trace_id);

    let communication_recorded_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'communication.message.recorded'
          AND subject ->> 'message_id' = $1
        "#,
    )
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("communication message recorded event count");
    assert_eq!(communication_recorded_count, 1);

    let review_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'review.item.available.v1'
          AND subject ->> 'review_item_id' = $1
        "#,
    )
    .bind(&review_item_id)
    .fetch_one(&pool)
    .await
    .expect("review item available event count");
    assert_eq!(review_event_count, 1);
}

#[tokio::test]
async fn zulip_reaction_edit_delete_signals_materialize_canonical_state() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "zulip-lifecycle-account",
            CommunicationProviderKind::ZulipBot,
            "Zulip Lifecycle",
            "zulip-lifecycle-bot@example.test",
        ))
        .await
        .expect("provider account");

    let ingestion = CommunicationIngestionStore::new(pool.clone());
    let mapping_context = ZulipEventMappingContext::new(
        "zulip-lifecycle-account",
        "http://localhost:8080",
        Utc::now(),
    )
    .with_import_batch_id("zulip-lifecycle-batch")
    .with_scenario_id("zulip-lifecycle-scenario");

    let (_, message_accepted_event, message) = record_dispatch_consume_zulip_event(
        &pool,
        &ingestion,
        &mapping_context,
        json!({
            "id": 42,
            "type": "message",
            "message": {
                "id": 7001,
                "content": "Надо проверить backup retention до пятницы.",
                "sender_email": "bot@example.test",
                "sender_full_name": "Макошь Bot",
                "stream_id": 10,
                "display_recipient": "Макошь Lab",
                "topic": "Tasks"
            }
        }),
    )
    .await;
    assert_eq!(
        message_accepted_event.event_type,
        "signal.accepted.zulip.message"
    );

    let (_, reaction_accepted_event, reaction_projection) = record_dispatch_consume_zulip_event(
        &pool,
        &ingestion,
        &mapping_context,
        json!({
            "id": 43,
            "type": "reaction",
            "message_id": 7001,
            "emoji_name": "+1",
            "emoji_code": "1f44d",
            "reaction_type": "unicode_emoji",
            "op": "add",
            "user_id": 55,
            "user": {
                "full_name": "Zulip Reactor",
                "email": "reactor@example.test"
            }
        }),
    )
    .await;
    assert_eq!(
        reaction_accepted_event.event_type,
        "signal.accepted.zulip.reaction"
    );
    assert_eq!(reaction_projection.message_id, message.message_id);

    let reaction_row = sqlx::query(
        r#"
        SELECT reaction, is_active, provider_actor_id, sender_display_name
        FROM communication_message_reactions
        WHERE message_id = $1
        "#,
    )
    .bind(&message.message_id)
    .fetch_one(&pool)
    .await
    .expect("Zulip reaction row");
    assert_eq!(
        reaction_row
            .try_get::<String, _>("reaction")
            .expect("reaction"),
        "+1"
    );
    assert!(
        reaction_row
            .try_get::<bool, _>("is_active")
            .expect("is_active")
    );
    assert_eq!(
        reaction_row
            .try_get::<String, _>("provider_actor_id")
            .expect("provider_actor_id"),
        "55"
    );
    assert_eq!(
        reaction_row
            .try_get::<String, _>("sender_display_name")
            .expect("sender_display_name"),
        "Zulip Reactor"
    );

    let (_, update_accepted_event, update_projection) = record_dispatch_consume_zulip_event(
        &pool,
        &ingestion,
        &mapping_context,
        json!({
            "id": 44,
            "type": "update_message",
            "message_id": 7001,
            "content": "Надо проверить backup retention и расписать owner до пятницы.",
            "prev_content": "Надо проверить backup retention до пятницы.",
            "topic": "Tasks",
            "edit_timestamp": 1782720060
        }),
    )
    .await;
    assert_eq!(
        update_accepted_event.event_type,
        "signal.accepted.zulip.message_update"
    );
    assert_eq!(update_projection.message_id, message.message_id);
    assert_eq!(
        update_projection.body_text,
        "Надо проверить backup retention и расписать owner до пятницы."
    );

    let updated_message_row = sqlx::query(
        "SELECT body_text, message_metadata FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message.message_id)
    .fetch_one(&pool)
    .await
    .expect("updated Zulip message row");
    assert_eq!(
        updated_message_row
            .try_get::<String, _>("body_text")
            .expect("body_text"),
        "Надо проверить backup retention и расписать owner до пятницы."
    );
    let updated_metadata = updated_message_row
        .try_get::<Value, _>("message_metadata")
        .expect("message_metadata");
    assert_eq!(updated_metadata["edited"], json!(true));

    let version_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM communication_message_versions
        WHERE message_id = $1
          AND source_event = $2
          AND body_text = $3
        "#,
    )
    .bind(&message.message_id)
    .bind(&update_accepted_event.event_id)
    .bind("Надо проверить backup retention и расписать owner до пятницы.")
    .fetch_one(&pool)
    .await
    .expect("Zulip message version count");
    assert_eq!(version_count, 1);

    let (_, delete_accepted_event, delete_projection) = record_dispatch_consume_zulip_event(
        &pool,
        &ingestion,
        &mapping_context,
        json!({
            "id": 45,
            "type": "delete_message",
            "message_id": 7001,
            "message_type": "stream",
            "stream_id": 10,
            "topic": "Tasks"
        }),
    )
    .await;
    assert_eq!(
        delete_accepted_event.event_type,
        "signal.accepted.zulip.message_delete"
    );
    assert_eq!(delete_projection.message_id, message.message_id);

    let tombstone_row = sqlx::query(
        r#"
        SELECT reason_class, actor_class, is_provider_delete, is_local_visible
        FROM communication_message_tombstones
        WHERE message_id = $1
          AND source_event = $2
        "#,
    )
    .bind(&message.message_id)
    .bind(&delete_accepted_event.event_id)
    .fetch_one(&pool)
    .await
    .expect("Zulip tombstone row");
    assert_eq!(
        tombstone_row
            .try_get::<String, _>("reason_class")
            .expect("reason_class"),
        "deleted_by_provider"
    );
    assert_eq!(
        tombstone_row
            .try_get::<String, _>("actor_class")
            .expect("actor_class"),
        "provider"
    );
    assert!(
        tombstone_row
            .try_get::<bool, _>("is_provider_delete")
            .expect("is_provider_delete")
    );
    assert!(
        !tombstone_row
            .try_get::<bool, _>("is_local_visible")
            .expect("is_local_visible")
    );

    consume_accepted_signal_event(pool.clone(), &reaction_accepted_event)
        .await
        .expect("replay Zulip reaction signal");
    consume_accepted_signal_event(pool.clone(), &update_accepted_event)
        .await
        .expect("replay Zulip update signal");
    consume_accepted_signal_event(pool.clone(), &delete_accepted_event)
        .await
        .expect("replay Zulip delete signal");

    let reaction_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM communication_message_reactions
        WHERE message_id = $1
          AND reaction = '+1'
        "#,
    )
    .bind(&message.message_id)
    .fetch_one(&pool)
    .await
    .expect("Zulip reaction idempotency count");
    assert_eq!(reaction_count, 1);

    let version_replay_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM communication_message_versions
        WHERE message_id = $1
          AND source_event = $2
        "#,
    )
    .bind(&message.message_id)
    .bind(&update_accepted_event.event_id)
    .fetch_one(&pool)
    .await
    .expect("Zulip version idempotency count");
    assert_eq!(version_replay_count, 1);

    let tombstone_replay_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM communication_message_tombstones
        WHERE message_id = $1
          AND source_event = $2
        "#,
    )
    .bind(&message.message_id)
    .bind(&delete_accepted_event.event_id)
    .fetch_one(&pool)
    .await
    .expect("Zulip tombstone idempotency count");
    assert_eq!(tombstone_replay_count, 1);

    let communication_updated_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM event_log
        WHERE event_type = 'communication.message.updated'
          AND causation_id = ANY($1)
        "#,
    )
    .bind(vec![
        reaction_accepted_event.event_id.clone(),
        update_accepted_event.event_id.clone(),
        delete_accepted_event.event_id.clone(),
    ])
    .fetch_one(&pool)
    .await
    .expect("communication updated event count");
    assert_eq!(communication_updated_count, 3);
}

async fn record_dispatch_consume_zulip_event(
    pool: &PgPool,
    ingestion: &CommunicationIngestionStore,
    mapping_context: &ZulipEventMappingContext,
    event_value: Value,
) -> (String, EventEnvelope, ProjectedMessage) {
    let event: ZulipEvent = serde_json::from_value(event_value).expect("valid Zulip event");
    let new_raw_record = observation_to_raw_communication_record(
        map_zulip_event_to_observation(&event, mapping_context)
            .expect("map Zulip event to raw record"),
    );
    let raw_record = ingestion
        .record_raw_source(&new_raw_record)
        .await
        .expect("record Zulip raw source");
    let accepted_event = dispatch_zulip_raw_signal(pool.clone(), &raw_record)
        .await
        .expect("dispatch Zulip raw signal")
        .expect("accepted Zulip signal");
    let projected = consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted Zulip signal")
        .expect("projected Zulip message");

    (raw_record.raw_record_id, accepted_event, projected)
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
        .expect("secret reference");
    CommunicationProviderSecretBindingStore::new(pool.clone())
        .bind(&NewProviderAccountSecretBinding::new(
            account_id,
            ProviderAccountSecretPurpose::ZulipApiKey,
            secret_ref,
        ))
        .await
        .expect("Zulip API key binding");
    resolver
        .insert(secret_ref, secret_value)
        .expect("insert test secret");
}

fn json_post_request(path: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

fn vault_entropy_events(count: usize) -> Vec<Value> {
    (0..count)
        .map(|index| {
            json!({
                "x": index % 997,
                "y": index % 577,
                "dx": (index % 11) as i64 - 5,
                "dy": (index % 13) as i64 - 6,
                "timestamp_ms": index * 5,
                "velocity": (index % 19) as f64 / 10.0,
                "acceleration": (index % 23) as f64 / 100.0,
                "interval_ms": 5
            })
        })
        .collect()
}

async fn spawn_fake_zulip_api() -> String {
    let app = Router::new()
        .route("/api/v1/register", post(fake_zulip_register_queue))
        .route("/api/v1/events", get(fake_zulip_get_events))
        .route("/api/v1/messages", post(fake_zulip_send_message))
        .route(
            "/api/v1/messages/{message_id}",
            patch(fake_zulip_update_message).delete(fake_zulip_delete_message),
        )
        .route(
            "/api/v1/messages/{message_id}/reactions",
            post(fake_zulip_add_reaction).delete(fake_zulip_remove_reaction),
        )
        .route("/api/v1/user_uploads", post(fake_zulip_upload_file))
        .route("/user_uploads/{*path}", get(fake_zulip_download_file));

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("bind fake Zulip API");
    let address = listener.local_addr().expect("fake Zulip API address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fake Zulip API");
    });

    format!("http://{address}")
}

async fn fake_zulip_register_queue(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    if !zulip_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "result": "error",
                "msg": "provider echoed zulip-api-key-must-not-leak"
            })),
        );
    }
    let form = parse_form(&body);
    let event_types = form
        .get("event_types")
        .and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
        .expect("event types");
    assert!(event_types.contains(&"message".to_owned()));
    assert!(event_types.contains(&"reaction".to_owned()));
    assert!(event_types.contains(&"update_message".to_owned()));
    assert!(event_types.contains(&"delete_message".to_owned()));

    (
        StatusCode::OK,
        Json(json!({
            "result": "success",
            "msg": "",
            "queue_id": "zulip-fake-queue",
            "last_event_id": 0
        })),
    )
}

async fn fake_zulip_get_events(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if !zulip_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "result": "error",
                "msg": "provider echoed zulip-api-key-must-not-leak"
            })),
        );
    }
    if query.get("queue_id").map(String::as_str) == Some("expired-zulip-queue") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "result": "error",
                "msg": "BAD_EVENT_QUEUE_ID"
            })),
        );
    }
    assert_eq!(
        query.get("queue_id").map(String::as_str),
        Some("zulip-fake-queue")
    );
    assert_eq!(query.get("dont_block").map(String::as_str), Some("true"));
    let last_event_id = query
        .get("last_event_id")
        .and_then(|value| value.parse::<i64>().ok())
        .expect("last_event_id");
    let events = if last_event_id < 42 {
        json!([{
            "id": 42,
            "type": "message",
            "message": {
                "id": 7001,
                "content": "Надо проверить backup retention до пятницы.",
                "sender_email": "bot@example.test",
                "sender_full_name": "Макошь Bot",
                "stream_id": 10,
                "display_recipient": "Макошь Lab",
                "topic": "Tasks"
            }
        }])
    } else {
        json!([])
    };

    (
        StatusCode::OK,
        Json(json!({
            "result": "success",
            "msg": "",
            "queue_id": "zulip-fake-queue",
            "events": events
        })),
    )
}

async fn fake_zulip_send_message(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    if !zulip_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "result": "error",
                "msg": "provider echoed zulip-api-key-must-not-leak"
            })),
        );
    }
    let form = parse_form(&body);
    let response_id = match form.get("type").map(String::as_str) {
        Some("stream") => {
            assert_eq!(form.get("to").map(String::as_str), Some("Макошь Lab"));
            assert_eq!(form.get("topic").map(String::as_str), Some("Tasks"));
            let content = form.get("content").map(String::as_str);
            assert!(
                matches!(
                    content,
                    Some("Надо проверить retention.")
                        | Some("Надо проверить retention.\n/user_uploads/evidence.txt")
                ),
                "unexpected Zulip stream content: {content:?}"
            );
            7001
        }
        Some("direct") => match form.get("content").map(String::as_str) {
            Some("Direct note") => {
                assert_eq!(
                    form.get("to").map(String::as_str),
                    Some("[\"alice@example.test\"]")
                );
                7002
            }
            Some("Direct note by user id") => {
                assert_eq!(form.get("to").map(String::as_str), Some("[101]"));
                7003
            }
            other => panic!("unexpected Zulip direct content: {other:?}"),
        },
        other => panic!("unexpected Zulip message type: {other:?}"),
    };

    (
        StatusCode::OK,
        Json(json!({"result": "success", "msg": "", "id": response_id})),
    )
}

async fn fake_zulip_update_message(
    Path(message_id): Path<i64>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    assert_eq!(message_id, 7001);
    if !zulip_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"result": "error", "msg": "unauthorized"})),
        );
    }
    let form = parse_form(&body);
    assert_eq!(
        form.get("content").map(String::as_str),
        Some("Updated retention note")
    );
    assert_eq!(form.get("topic").map(String::as_str), Some("Follow-up"));
    assert_eq!(form.get("stream_id").map(String::as_str), Some("99"));
    assert_eq!(
        form.get("propagate_mode").map(String::as_str),
        Some("change_all")
    );

    (
        StatusCode::OK,
        Json(json!({"result": "success", "msg": ""})),
    )
}

async fn fake_zulip_delete_message(
    Path(message_id): Path<i64>,
    headers: HeaderMap,
) -> impl IntoResponse {
    assert_eq!(message_id, 7001);
    if !zulip_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"result": "error", "msg": "unauthorized"})),
        );
    }
    (
        StatusCode::OK,
        Json(json!({"result": "success", "msg": ""})),
    )
}

async fn fake_zulip_add_reaction(
    Path(message_id): Path<i64>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    assert_eq!(message_id, 7001);
    assert_zulip_reaction_request(headers, body);
    (
        StatusCode::OK,
        Json(json!({"result": "success", "msg": ""})),
    )
}

async fn fake_zulip_remove_reaction(
    Path(message_id): Path<i64>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    assert_eq!(message_id, 7001);
    assert_zulip_reaction_request(headers, body);
    (
        StatusCode::OK,
        Json(json!({"result": "success", "msg": ""})),
    )
}

async fn fake_zulip_upload_file(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    if !zulip_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"result": "error", "msg": "unauthorized"})),
        );
    }
    let content_type = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    assert!(
        content_type.starts_with("multipart/form-data; boundary="),
        "unexpected upload content-type: {content_type}"
    );
    let upload_body = String::from_utf8_lossy(&body);
    assert!(upload_body.contains("evidence.txt"));
    assert!(upload_body.contains("zulip attachment bytes"));

    (
        StatusCode::OK,
        Json(json!({
            "result": "success",
            "msg": "",
            "uri": "/user_uploads/evidence.txt"
        })),
    )
}

async fn fake_zulip_download_file(
    Path(path): Path<String>,
    headers: HeaderMap,
) -> axum::response::Response {
    if !zulip_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"result": "error", "msg": "unauthorized"})),
        )
            .into_response();
    }
    assert_eq!(path, "evidence.txt");

    (
        StatusCode::OK,
        [("content-type", "text/plain")],
        "zulip downloaded attachment bytes",
    )
        .into_response()
}

fn assert_zulip_reaction_request(headers: HeaderMap, body: Bytes) {
    assert!(zulip_authorized(&headers));
    let form = parse_form(&body);
    assert_eq!(
        form.get("emoji_name").map(String::as_str),
        Some("thumbs_up")
    );
    assert_eq!(form.get("emoji_code").map(String::as_str), Some("1f44d"));
    assert_eq!(
        form.get("reaction_type").map(String::as_str),
        Some("unicode_emoji")
    );
}

fn parse_form(body: &Bytes) -> HashMap<String, String> {
    form_urlencoded::parse(body.as_ref()).into_owned().collect()
}

fn zulip_authorized(headers: &HeaderMap) -> bool {
    matches!(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
        Some("Basic Ym90QGV4YW1wbGUudGVzdDp6dWxpcC10ZXN0LWtleQ==")
            | Some("Basic Ym90K2FAZXhhbXBsZS50ZXN0Onp1bGlwLXRlc3Qta2V5")
            | Some("Basic Ym90K2JAZXhhbXBsZS50ZXN0Onp1bGlwLXRlc3Qta2V5")
    )
}
