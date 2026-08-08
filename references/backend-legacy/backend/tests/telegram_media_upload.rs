use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "telegram-media-upload-test-secret";

#[tokio::test]
async fn telegram_media_upload_imports_attachment_and_queues_provider_command() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-media-{suffix}");
    create_tdlib_account(&pool, &account_id, &suffix).await;
    let app = router(database, &database_url);

    let import_response = app
        .clone()
        .oneshot(post(
            "/api/v1/communications/attachments/import",
            json!({
                "account_id": account_id,
                "channel_kind": "telegram_user",
                "filename": "upload-note.txt",
                "content_type": "text/plain",
                "content_base64": "SGVybWVzIG1lZGlhIHVwbG9hZCBmaXh0dXJl",
                "metadata": {"source": "telegram_media_upload_test"}
            }),
        ))
        .await
        .expect("import response");
    assert_eq!(import_response.status(), StatusCode::OK);
    let imported = json_body(import_response).await;
    assert_eq!(imported["scan_status"], "not_scanned");
    let attachment_id = imported["attachment_id"]
        .as_str()
        .expect("attachment id")
        .to_owned();
    let blob_id = imported["blob_id"].as_str().expect("blob id").to_owned();
    let attachment_observation = sqlx::query(
        r#"
        SELECT kind.code AS kind_code,
               observation.origin_kind,
               observation.payload,
               link.relationship_kind
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        JOIN observation_links link
          ON link.observation_id = observation.observation_id
        WHERE link.domain = 'communications'
          AND link.entity_kind = 'attachment_import'
          AND link.entity_id = $1
        "#,
    )
    .bind(&attachment_id)
    .fetch_one(&pool)
    .await
    .expect("attachment import observation");
    assert_eq!(
        attachment_observation.get::<String, _>("kind_code"),
        "COMMUNICATION_ATTACHMENT"
    );
    assert_eq!(
        attachment_observation.get::<String, _>("origin_kind"),
        "manual"
    );
    assert_eq!(
        attachment_observation.get::<String, _>("relationship_kind"),
        "attachment_import"
    );
    let attachment_payload = attachment_observation.get::<Value, _>("payload");
    assert_eq!(attachment_payload["attachment_id"], attachment_id);
    assert_eq!(attachment_payload["channel_kind"], "telegram_user");
    assert_eq!(attachment_payload["content_type"], "text/plain");
    assert_eq!(attachment_payload["filename"], "upload-note.txt");

    let unscanned_command_id = format!("tcmd_media_upload_unscanned_{suffix}");
    let unscanned_response = app
        .clone()
        .oneshot(post(
            "/api/v1/integrations/telegram/provider-media/upload",
            json!({
                "command_id": unscanned_command_id.clone(),
                "account_id": account_id.clone(),
                "provider_chat_id": "123456789",
                "attachment_id": attachment_id.clone(),
                "media_type": "document"
            }),
        ))
        .await
        .expect("unscanned upload response");
    assert_eq!(unscanned_response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        provider_command_count(&pool, &unscanned_command_id).await,
        0,
        "unscanned media must not reach the provider outbox"
    );

    let blob_only_command_id = format!("tcmd_media_upload_blob_only_{suffix}");
    let blob_only_response = app
        .clone()
        .oneshot(post(
            "/api/v1/integrations/telegram/provider-media/upload",
            json!({
                "command_id": blob_only_command_id.clone(),
                "account_id": account_id.clone(),
                "provider_chat_id": "123456789",
                "blob_id": blob_id,
                "media_type": "document"
            }),
        ))
        .await
        .expect("blob-only upload response");
    assert_eq!(blob_only_response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        provider_command_count(&pool, &blob_only_command_id).await,
        0,
        "a blob without an attachment scan must not reach the provider outbox"
    );

    mark_attachment_clean(&pool, &attachment_id).await;

    let command_id = format!("tcmd_media_upload_{suffix}");
    let upload_response = app
        .clone()
        .oneshot(post(
            "/api/v1/integrations/telegram/provider-media/upload",
            json!({
                "command_id": command_id.clone(),
                "account_id": account_id.clone(),
                "provider_chat_id": "123456789",
                "attachment_id": attachment_id.clone(),
                "media_type": "document",
                "caption": "caption must not be written to audit metadata"
            }),
        ))
        .await
        .expect("upload response");
    assert_eq!(upload_response.status(), StatusCode::OK);
    let uploaded = json_body(upload_response).await;
    assert_eq!(uploaded["status"], "queued");
    assert_eq!(uploaded["reconciliation_status"], "not_observed");
    assert_eq!(uploaded["blob_id"], blob_id);

    let duplicate_upload_response = app
        .clone()
        .oneshot(post(
            "/api/v1/integrations/telegram/provider-media/upload",
            json!({
                "command_id": format!("tcmd_media_upload_duplicate_{suffix}"),
                "account_id": account_id.clone(),
                "provider_chat_id": "123456789",
                "attachment_id": attachment_id.clone(),
                "media_type": "document",
                "caption": "caption must not be written to audit metadata"
            }),
        ))
        .await
        .expect("duplicate upload response");
    assert_eq!(duplicate_upload_response.status(), StatusCode::OK);
    let duplicate_uploaded = json_body(duplicate_upload_response).await;
    assert_eq!(duplicate_uploaded["command_id"], command_id);

    let command = sqlx::query(
        r#"
        SELECT command_kind, status, reconciliation_status, payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("queued command");
    assert_eq!(command.get::<String, _>("command_kind"), "send_media");
    assert_eq!(command.get::<String, _>("status"), "queued");
    assert_eq!(
        command.get::<String, _>("reconciliation_status"),
        "not_observed"
    );
    let payload = command.get::<Value, _>("payload");
    assert_eq!(payload["media_type"], "document");
    assert_eq!(payload["attachment_id"], attachment_id);
    assert_eq!(payload["blob_id"], blob_id);

    let command_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM telegram_provider_write_commands WHERE account_id = $1 AND command_kind = 'send_media'",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("command count");
    assert_eq!(command_count, 1);

    let audit = sqlx::query(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.media.upload'
          AND target_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("media upload audit");
    let audit_metadata = audit.get::<Value, _>("metadata");
    assert_eq!(audit_metadata["capability"], "telegram.media.upload");
    assert_eq!(audit_metadata.get("caption"), None);

    let started_events = event_count(&pool, "telegram.media.upload.started", &command_id).await;
    let status_events = event_count(&pool, "telegram.command.status_changed", &command_id).await;
    assert_eq!(started_events, 1);
    assert_eq!(status_events, 1);
    let started_status = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT payload->>'status'
        FROM event_log
        WHERE event_type = 'telegram.media.upload.started'
          AND subject->>'id' = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("started event status");
    assert_eq!(started_status.as_deref(), Some("queued"));
    let started_payload = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'telegram.media.upload.started'
          AND subject->>'id' = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("started event payload");
    assert_eq!(started_payload["command_kind"], "send_media");
    assert_eq!(started_payload["payload"]["attachment_id"], attachment_id);
    assert_eq!(started_payload["payload"]["filename"], "upload-note.txt");
    assert_eq!(started_payload["capability_state"], "available");
}

#[tokio::test]
async fn telegram_media_upload_rejects_malicious_import_before_outbox_insert() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-media-malicious-{suffix}");
    create_tdlib_account(&pool, &account_id, &suffix).await;
    let app = router(database, &database_url);

    let import_response = app
        .clone()
        .oneshot(post(
            "/api/v1/communications/attachments/import",
            json!({
                "account_id": account_id,
                "channel_kind": "telegram_user",
                "filename": "payload.exe",
                "content_type": "application/octet-stream",
                "content_base64": "TVqQAAAA"
            }),
        ))
        .await
        .expect("import response");
    assert_eq!(import_response.status(), StatusCode::OK);
    let imported = json_body(import_response).await;
    assert_eq!(imported["scan_status"], "malicious");

    let command_id = format!("tcmd_media_upload_reject_{suffix}");
    let upload_response = app
        .oneshot(post(
            "/api/v1/integrations/telegram/provider-media/upload",
            json!({
                "command_id": command_id.clone(),
                "account_id": account_id.clone(),
                "provider_chat_id": "123456789",
                "attachment_id": imported["attachment_id"].clone(),
                "media_type": "document"
            }),
        ))
        .await
        .expect("upload response");
    assert_eq!(upload_response.status(), StatusCode::BAD_REQUEST);

    let command_count = provider_command_count(&pool, &command_id).await;
    assert_eq!(command_count, 0);
}

async fn mark_attachment_clean(pool: &sqlx::PgPool, attachment_id: &str) {
    sqlx::query(
        r#"
        UPDATE communication_attachment_imports
        SET scan_status = 'clean',
            scan_engine = 'test-scanner',
            scan_checked_at = now(),
            scan_summary = 'Synthetic fixture is clean'
        WHERE attachment_id = $1
        "#,
    )
    .bind(attachment_id)
    .execute(pool)
    .await
    .expect("mark attachment clean");
}

async fn provider_command_count(pool: &sqlx::PgPool, command_id: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM telegram_provider_write_commands WHERE command_id = $1",
    )
    .bind(command_id)
    .fetch_one(pool)
    .await
    .expect("provider command count")
}

async fn create_tdlib_account(pool: &sqlx::PgPool, account_id: &str, suffix: &str) {
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::TelegramUser,
                format!("Telegram Media {suffix}"),
                format!("tg-media-{suffix}"),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");
}

fn router(database: Database, database_url: &str) -> axum::Router {
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url,
        ),
        database,
    )
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

async fn event_count(pool: &sqlx::PgPool, event_type: &str, subject_id: &str) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM event_log
        WHERE event_type = $1
          AND subject->>'id' = $2
        "#,
    )
    .bind(event_type)
    .bind(subject_id)
    .fetch_one(pool)
    .await
    .expect("event count")
}

fn unique_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    format!("{now}")
}
