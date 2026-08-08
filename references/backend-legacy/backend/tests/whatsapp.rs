use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::{TimeZone, Utc};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_events_api::EventLogQuery;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::engines::timeline::TimelineEngine;

use makosh_hub_backend::platform::secrets::models::SecretKind;
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::HostVault;
use makosh_hub_backend::vault::models::{EntropyEvent, HostVaultConfig, SecretEntryContext};

const LOCAL_API_TOKEN: &str = "whatsapp-api-test-secret";

#[test]
fn whatsapp_provider_and_secret_kinds_are_account_scoped() {
    assert_eq!(
        CommunicationProviderKind::try_from("whatsapp_web").expect("whatsapp web provider"),
        CommunicationProviderKind::WhatsappWeb
    );
    assert!(CommunicationProviderKind::WhatsappWeb.is_whatsapp());
    assert!(!CommunicationProviderKind::WhatsappWeb.is_email());
    assert!(!CommunicationProviderKind::WhatsappWeb.is_telegram());

    assert!(
        ProviderAccountSecretPurpose::WhatsappWebSessionKey
            .accepts_secret_kind(SecretKind::PrivateKey)
    );
    assert!(
        ProviderAccountSecretPurpose::WhatsappWebSessionKey.accepts_secret_kind(SecretKind::Other)
    );
    assert!(
        !ProviderAccountSecretPurpose::WhatsappWebSessionKey
            .accepts_secret_kind(SecretKind::Password)
    );
    assert!(
        !ProviderAccountSecretPurpose::WhatsappWebSessionKey
            .accepts_secret_kind(SecretKind::ApiToken)
    );
}

#[tokio::test]
async fn whatsapp_provider_neutral_communications_routes_dispatch_to_whatsapp_commands() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-communications-routes-{suffix}");
    let chat_id = format!("wa-communications-chat-{suffix}");
    let provider_message_id = format!("wa-communications-message-{suffix}");

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Communications Routes",
                "external_account_id": format!("wa-communications-routes-{suffix}"),
                "device_name": "Макошь Communications Routes",
                "local_state_path": format!("docker/data/whatsapp/communications-routes-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fixture account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "WhatsApp Communications",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "WhatsApp Sender",
                "text": "Projected message for provider-neutral dispatch.",
                "import_batch_id": format!("whatsapp-communications-routes-{suffix}"),
                "occurred_at": "2026-06-06T13:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fixture message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("projected message id")
        .to_owned();

    let send_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/conversations/{chat_id}/messages"),
            json!({
                "account_id": account_id,
                "text": "provider-neutral send to whatsapp"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("communications send response");
    assert_eq!(send_response.status(), StatusCode::OK);
    let send_body = json_body(send_response).await;
    assert_eq!(send_body["provider"], json!("whatsapp"));
    assert_eq!(send_body["channel_kind"], json!("whatsapp_web"));
    assert_eq!(send_body["provider_chat_id"], json!(chat_id));
    assert_eq!(send_body["status"], json!("queued"));

    let reply_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/messages/{message_id}/reply"),
            json!({
                "text": "provider-neutral reply to whatsapp"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("communications reply response");
    assert_eq!(reply_response.status(), StatusCode::OK);
    let reply_body = json_body(reply_response).await;
    assert_eq!(reply_body["provider"], json!("whatsapp"));
    assert_eq!(reply_body["channel_kind"], json!("whatsapp_web"));
    assert_eq!(reply_body["provider_chat_id"], json!(chat_id));
    assert_eq!(reply_body["status"], json!("queued"));

    let forward_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/messages/{message_id}/forward"),
            json!({
                "conversation_id": chat_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("communications forward response");
    assert_eq!(forward_response.status(), StatusCode::OK);
    let forward_body = json_body(forward_response).await;
    assert_eq!(forward_body["provider"], json!("whatsapp"));
    assert_eq!(forward_body["channel_kind"], json!("whatsapp_web"));
    assert_eq!(forward_body["provider_chat_id"], json!(chat_id));
    assert_eq!(forward_body["status"], json!("queued"));

    let pin_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/conversations/{chat_id}/pin"),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("communications conversation pin response");
    assert_eq!(pin_response.status(), StatusCode::OK);
    let pin_body = json_body(pin_response).await;
    assert_eq!(pin_body["provider"], json!("whatsapp"));
    assert_eq!(pin_body["channel_kind"], json!("whatsapp_web"));
    assert_eq!(pin_body["provider_chat_id"], json!(chat_id));
    assert_eq!(pin_body["status"], json!("queued"));
    assert_eq!(pin_body["action"], json!("pin"));
    assert_eq!(pin_body["active"], json!(true));

    let unpin_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/conversations/{chat_id}/unpin"),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("communications conversation unpin response");
    assert_eq!(unpin_response.status(), StatusCode::OK);
    let unpin_body = json_body(unpin_response).await;
    assert_eq!(unpin_body["provider"], json!("whatsapp"));
    assert_eq!(unpin_body["channel_kind"], json!("whatsapp_web"));
    assert_eq!(unpin_body["provider_chat_id"], json!(chat_id));
    assert_eq!(unpin_body["status"], json!("queued"));
    assert_eq!(unpin_body["action"], json!("unpin"));
    assert_eq!(unpin_body["active"], json!(false));

    for (path, action, active) in [
        ("archive", "archive", true),
        ("unarchive", "unarchive", false),
        ("mute", "mute", true),
        ("unmute", "unmute", false),
        ("read", "mark_read", false),
        ("unread", "mark_unread", true),
    ] {
        let response = app
            .clone()
            .oneshot(json_post_request_with_actor(
                &format!("/api/v1/communications/conversations/{chat_id}/{path}"),
                json!({}),
                LOCAL_API_TOKEN,
            ))
            .await
            .expect("communications conversation lifecycle response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["provider"], json!("whatsapp"));
        assert_eq!(body["channel_kind"], json!("whatsapp_web"));
        assert_eq!(body["provider_chat_id"], json!(chat_id));
        assert_eq!(body["status"], json!("queued"));
        assert_eq!(body["action"], json!(action));
        assert_eq!(body["active"], json!(active));
    }
}

#[tokio::test]
async fn whatsapp_fixture_message_ingestion_refreshes_decision_and_obligation_candidates_against_postgres()
 {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-candidate-{suffix}");
    let chat_id = format!("wa-candidate-chat-{suffix}");
    let decision_title = format!("Use WhatsApp evidence for shared memory {suffix}");
    let decision_rationale = "chat context must feed the same domain model";
    let obligation_statement = format!("send the WhatsApp alignment note {suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Candidate Source",
                "external_account_id": format!("wa-candidate-{suffix}"),
                "device_name": "Макошь Desktop Fixture",
                "local_state_path": format!("docker/data/whatsapp/candidate-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);
    let signal_connection = sqlx::query(
        r#"
        SELECT source_code, status, settings
        FROM signal_connections
        WHERE source_code = 'whatsapp'
          AND settings->>'account_id' = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("whatsapp signal connection");
    let signal_settings: Value = signal_connection
        .try_get("settings")
        .expect("whatsapp signal settings");
    assert_eq!(
        signal_connection
            .try_get::<String, _>("source_code")
            .expect("signal source"),
        "whatsapp"
    );
    assert_eq!(
        signal_connection
            .try_get::<String, _>("status")
            .expect("signal status"),
        "connected"
    );
    assert_eq!(signal_settings["account_id"], json!(account_id));
    assert_eq!(signal_settings["provider_kind"], json!("whatsapp_web"));

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("wa-candidate-message-{suffix}"),
                "chat_title": "WhatsApp Candidate Review",
                "sender_id": format!("whatsapp-candidate-sender-{suffix}"),
                "sender_display_name": "WhatsApp Candidate Sender",
                "text": format!(
                    "Decision: {decision_title} because {decision_rationale}. I will {obligation_statement} by Friday 5pm."
                ),
                "import_batch_id": format!("whatsapp-candidate-fixture-{suffix}"),
                "occurred_at": "2026-06-06T13:30:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();
    let observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let raw_signal_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.whatsapp.message.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("raw whatsapp signal count");
    let accepted_signal_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.whatsapp.message'",
    )
    .fetch_one(&pool)
    .await
    .expect("accepted whatsapp signal count");
    let legacy_integration_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type LIKE 'integration.whatsapp.%'",
    )
    .fetch_one(&pool)
    .await
    .expect("legacy whatsapp integration event count");
    assert_eq!(raw_signal_count, 1);
    assert_eq!(accepted_signal_count, 1);
    assert_eq!(legacy_integration_count, 0);
    let trace_rows = sqlx::query(
        r#"
        SELECT event_id, event_type, causation_id, correlation_id
        FROM event_log
        WHERE correlation_id = $1
          AND event_type IN (
              'observation.captured.v1',
              'signal.raw.whatsapp.message.observed',
              'signal.accepted.whatsapp.message',
              'communication.message.recorded'
          )
        ORDER BY position ASC
        "#,
    )
    .bind(&observation_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp trace rows");
    assert_eq!(trace_rows.len(), 4);
    let observation_event_id = format!("event:v1:observation-captured:{observation_id}");
    let raw_event_id: String = trace_rows[1].try_get("event_id").expect("raw event id");
    let accepted_event_id: String = trace_rows[2]
        .try_get("event_id")
        .expect("accepted event id");
    assert_eq!(
        trace_rows[0]
            .try_get::<String, _>("event_id")
            .expect("observation event id"),
        observation_event_id
    );
    assert_eq!(
        trace_rows[1]
            .try_get::<Option<String>, _>("causation_id")
            .expect("raw causation")
            .as_deref(),
        Some(observation_event_id.as_str())
    );
    assert_eq!(
        trace_rows[2]
            .try_get::<Option<String>, _>("causation_id")
            .expect("accepted causation")
            .as_deref(),
        Some(raw_event_id.as_str())
    );
    assert_eq!(
        trace_rows[3]
            .try_get::<Option<String>, _>("causation_id")
            .expect("communication causation")
            .as_deref(),
        Some(accepted_event_id.as_str())
    );
    assert!(trace_rows.iter().all(|row| {
        row.try_get::<Option<String>, _>("correlation_id")
            .expect("trace correlation")
            .as_deref()
            == Some(observation_id.as_str())
    }));

    let decision_row: (String, String, String, String, String) = sqlx::query_as(
        r#"
        SELECT d.title, d.rationale, d.review_state, e.source_kind, e.source_id
        FROM decisions d
        JOIN decision_evidence e ON e.decision_id = d.decision_id
        WHERE e.source_kind = 'communication'
          AND e.source_id = $1
          AND d.title = $2
        "#,
    )
    .bind(&message_id)
    .bind(&decision_title)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp message should create a suggested Decision candidate");
    assert_eq!(decision_row.1, decision_rationale);
    assert_eq!(decision_row.2, "suggested");
    assert_eq!(decision_row.3, "communication");
    assert_eq!(decision_row.4, message_id);

    let task_candidate_row: (String, String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT title, review_state, candidate_kind, due_text
        FROM task_candidates
        WHERE source_kind = 'observation'
          AND source_id = $1
          AND candidate_kind = 'obligation_task'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp message should create an obligation-derived task candidate");
    assert_eq!(task_candidate_row.0, obligation_statement);
    assert_eq!(task_candidate_row.1, "suggested");
    assert_eq!(task_candidate_row.2, "obligation_task");
    assert_eq!(task_candidate_row.3.as_deref(), Some("Friday 5pm"));

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
            .expect("accepted obligation count");
    assert_eq!(task_count, 0);
    assert_eq!(obligation_count, 0);
}

#[tokio::test]
async fn whatsapp_web_session_metadata_is_account_scoped_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-web-{suffix}");
    let session_id = format!("whatsapp-session-{suffix}");
    let secret_ref = format!("secret-whatsapp-session-{suffix}");

    store
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::WhatsappWeb,
                "WhatsApp Web fixture",
                format!("wa-device-{suffix}"),
            )
            .config(json!({
                "runtime": "fixture",
                "local_state_path": format!("docker/data/whatsapp/{account_id}")
            })),
        )
        .await
        .expect("store WhatsApp Web provider account");

    sqlx::query(
        r#"
        INSERT INTO secret_references (
            secret_ref,
            secret_kind,
            store_kind,
            label,
            metadata
        )
        VALUES ($1, 'private_key', 'test_double', 'WhatsApp Web session key', '{}'::jsonb)
        "#,
    )
    .bind(&secret_ref)
    .execute(&pool)
    .await
    .expect("store session secret reference metadata");

    let binding = store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &account_id,
            ProviderAccountSecretPurpose::WhatsappWebSessionKey,
            &secret_ref,
        ))
        .await
        .expect("bind WhatsApp Web session key");
    assert_eq!(
        binding.secret_purpose,
        ProviderAccountSecretPurpose::WhatsappWebSessionKey
    );

    sqlx::query(
        r#"
        INSERT INTO whatsapp_web_sessions (
            session_id,
            account_id,
            device_name,
            companion_runtime,
            link_state,
            local_state_path,
            metadata
        )
        VALUES ($1, $2, 'Макошь Desktop', 'fixture', 'fixture', $3, $4)
        "#,
    )
    .bind(&session_id)
    .bind(&account_id)
    .bind(format!("docker/data/whatsapp/{account_id}"))
    .bind(json!({"source": "whatsapp-test"}))
    .execute(&pool)
    .await
    .expect("store WhatsApp Web session metadata");

    let stored_state: String =
        sqlx::query_scalar("SELECT link_state FROM whatsapp_web_sessions WHERE account_id = $1")
            .bind(&account_id)
            .fetch_one(&pool)
            .await
            .expect("fetch WhatsApp Web session state");
    assert_eq!(stored_state, "fixture");
}

#[tokio::test]
async fn whatsapp_authorized_session_material_is_stored_in_host_vault_against_postgres() {
    let test_context = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = test_context.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
        LOCAL_API_TOKEN,
        database_url.as_str(),
    )
    .with_test_dev_mode()
    .with_test_pairs([
        (
            "MAKOSH_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "MAKOSH_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
    ])
    .expect("config");
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-vault-{suffix}");
    let session_material = format!("whatsapp-session-material-{suffix}");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Vault Source",
                "external_account_id": format!("wa-vault-{suffix}"),
                "device_name": "Макошь Desktop Fixture",
                "local_state_path": format!("docker/data/whatsapp/vault-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let authorized_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/sessions/authorized",
            json!({
                "account_id": account_id,
                "session_material": session_material,
                "secret_kind": "other",
                "label": "WhatsApp fixture session credential",
                "metadata": {
                    "source": "whatsapp-authorized-session-test",
                    "runtime": "fixture"
                }
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("authorized session response");
    assert_eq!(authorized_response.status(), StatusCode::OK);
    let authorized_body = json_body(authorized_response).await;
    let secret_ref = format!("secret:provider-account:{account_id}:whatsapp_web_session_key");
    assert_eq!(authorized_body["secret_ref"], json!(secret_ref));
    assert_eq!(
        authorized_body["secret_purpose"],
        json!("whatsapp_web_session_key")
    );
    assert_eq!(authorized_body["secret_kind"], json!("other"));
    assert_eq!(authorized_body["store_kind"], json!("host_vault"));
    assert_eq!(authorized_body.get("session_material"), None);

    let status_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime status response");
    let status_status = status_response.status();
    let status_body = json_body(status_response).await;
    assert_eq!(
        status_status,
        StatusCode::OK,
        "status response body: {status_body}"
    );
    assert_eq!(status_body["status"], json!("linked"));
    assert_eq!(status_body["session_restore_available"], json!(true));
    assert_eq!(status_body["session_secret_ref"], json!(secret_ref));
    assert_eq!(status_body.get("session_material"), None);

    let linked_health_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/runtime/health?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("linked runtime health response");
    assert_eq!(linked_health_response.status(), StatusCode::OK);
    let linked_health_body = json_body(linked_health_response).await;
    assert_eq!(linked_health_body["healthy"], json!(false));
    assert_eq!(linked_health_body["status"], json!("degraded"));
    assert_eq!(
        linked_health_body["checks"]["session"]["restore_available"],
        json!(true)
    );
    assert_eq!(
        linked_health_body["checks"]["storage"]["binding_store"],
        json!("host_vault")
    );
    assert_eq!(
        linked_health_body["checks"]["runtime"]["lifecycle_state"],
        json!("linked")
    );
    assert_eq!(
        linked_health_body["checks"]["validation"]["status"],
        json!("degraded")
    );
    assert_eq!(linked_health_body.get("session_material"), None);

    let start_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime/start",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    assert_eq!(start_body["status"], json!("available"));
    assert_eq!(start_body["session_restore_available"], json!(true));
    assert_eq!(start_body["session_secret_ref"], json!(secret_ref));
    assert_eq!(start_body.get("session_material"), None);

    let stop_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime/stop",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime stop response");
    assert_eq!(stop_response.status(), StatusCode::OK);
    let stop_body = json_body(stop_response).await;
    assert_eq!(stop_body["status"], json!("linked"));
    assert_eq!(stop_body["session_restore_available"], json!(true));
    assert_eq!(stop_body["session_secret_ref"], json!(secret_ref));

    let revoke_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime/revoke",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime revoke response");
    assert_eq!(revoke_response.status(), StatusCode::OK);
    let revoke_body = json_body(revoke_response).await;
    assert_eq!(revoke_body["status"], json!("revoked"));
    assert_eq!(revoke_body["session_restore_available"], json!(false));
    assert_eq!(revoke_body["session_secret_ref"], Value::Null);
    assert_json_array_contains(&revoke_body["runtime_blockers"], "whatsapp_session_revoked");

    let relink_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime/relink",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime relink response");
    assert_eq!(relink_response.status(), StatusCode::OK);
    let relink_body = json_body(relink_response).await;
    assert_eq!(relink_body["status"], json!("link_required"));
    assert_eq!(relink_body["session_restore_available"], json!(false));
    assert_eq!(relink_body["session_secret_ref"], Value::Null);
    assert_json_array_contains(
        &relink_body["runtime_blockers"],
        "whatsapp_session_link_required",
    );

    let rotate_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime/rotate",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime rotate response");
    assert_eq!(rotate_response.status(), StatusCode::OK);
    let rotate_body = json_body(rotate_response).await;
    assert_eq!(rotate_body["status"], json!("link_required"));
    assert_eq!(rotate_body["session_restore_available"], json!(false));
    assert_eq!(rotate_body["session_secret_ref"], Value::Null);
    assert_json_array_contains(
        &rotate_body["runtime_blockers"],
        "whatsapp_session_link_required",
    );

    let health_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/runtime/health?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime health response");
    assert_eq!(health_response.status(), StatusCode::OK);
    let health_body = json_body(health_response).await;
    assert_eq!(health_body["status"], json!("blocked"));
    assert_eq!(
        health_body["checks"]["session_restore_available"],
        json!(false)
    );
    assert_eq!(health_body["checks"]["session_secret_ref"], Value::Null);
    assert_eq!(
        health_body["checks"]["session"]["restore_available"],
        json!(false)
    );
    assert_eq!(
        health_body["checks"]["validation"]["status"],
        json!("blocked")
    );
    assert_eq!(health_body.get("session_material"), None);

    let deleted_reference = SecretReferenceStore::new(pool.clone())
        .secret_reference(&secret_ref)
        .await
        .expect("load secret reference");
    assert_eq!(deleted_reference, None);

    let deleted_binding = CommunicationIngestionStore::new(pool.clone())
        .provider_account_secret_binding(
            &account_id,
            ProviderAccountSecretPurpose::WhatsappWebSessionKey,
        )
        .await
        .expect("load WhatsApp session binding");
    assert_eq!(deleted_binding, None);

    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    let deleted_secret = vault
        .read_secret(&secret_ref)
        .expect_err("deleted host vault secret");
    assert!(
        deleted_secret
            .to_string()
            .contains("secret was not found in host vault")
    );

    let remove_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime/remove",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime remove response");
    assert_eq!(remove_response.status(), StatusCode::OK);
    let remove_body = json_body(remove_response).await;
    assert_eq!(remove_body["account_id"], json!(account_id));
    assert_eq!(remove_body["provider_kind"], json!("whatsapp_web"));
    assert_eq!(remove_body["removed"], json!(true));
    assert_eq!(remove_body["unbound_secret_refs"], json!([]));

    let removed_account = CommunicationIngestionStore::new(pool.clone())
        .provider_account(&account_id)
        .await
        .expect("load removed account")
        .expect("removed account");
    assert_eq!(removed_account.config["lifecycle_state"], json!("removed"));
    let retained_binding = CommunicationIngestionStore::new(pool.clone())
        .provider_account_secret_binding(
            &account_id,
            ProviderAccountSecretPurpose::WhatsappWebSessionKey,
        )
        .await
        .expect("load retained binding");
    assert_eq!(retained_binding, None);

    let runtime_events =
        whatsapp_event_payloads(&pool, "whatsapp.runtime.status_changed", &account_id).await;
    assert_eq!(runtime_events.len(), 7);
    assert_eq!(runtime_events[0]["status"], json!("linked"));
    assert_eq!(runtime_events[0]["source"], json!("session_authorized"));
    assert_eq!(runtime_events[1]["status"], json!("available"));
    assert_eq!(runtime_events[1]["source"], json!("runtime_start"));
    assert_eq!(runtime_events[2]["status"], json!("linked"));
    assert_eq!(runtime_events[2]["source"], json!("runtime_stop"));
    assert_eq!(runtime_events[3]["status"], json!("revoked"));
    assert_eq!(runtime_events[3]["source"], json!("runtime_revoke"));
    assert_eq!(runtime_events[4]["status"], json!("link_required"));
    assert_eq!(runtime_events[4]["source"], json!("runtime_relink"));
    assert_eq!(runtime_events[5]["status"], json!("link_required"));
    assert_eq!(runtime_events[5]["source"], json!("runtime_rotate"));
    assert_eq!(runtime_events[6]["status"], json!("removed"));
    assert_eq!(runtime_events[6]["source"], json!("runtime_remove"));
    assert!(runtime_events.iter().all(|payload| {
        payload.get("session_material").is_none() && payload.get("session_secret_ref").is_none()
    }));

    let session_events =
        whatsapp_event_payloads(&pool, "whatsapp.session.link_state_changed", &account_id).await;
    assert_eq!(session_events.len(), 5);
    assert_eq!(session_events[0]["link_state"], json!("linked"));
    assert_eq!(session_events[0]["source"], json!("session_authorized"));
    assert_eq!(session_events[1]["link_state"], json!("revoked"));
    assert_eq!(session_events[1]["source"], json!("runtime_revoke"));
    assert_eq!(session_events[2]["link_state"], json!("link_required"));
    assert_eq!(session_events[2]["source"], json!("runtime_relink"));
    assert_eq!(session_events[3]["link_state"], json!("link_required"));
    assert_eq!(session_events[3]["source"], json!("runtime_rotate"));
    assert_eq!(session_events[4]["link_state"], json!("removed"));
    assert_eq!(session_events[4]["source"], json!("runtime_remove"));
    assert!(session_events.iter().all(|payload| {
        payload.get("session_material").is_none() && payload.get("session_secret_ref").is_none()
    }));

    let accepted_runtime_event_sources: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->'metadata'->>'source'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .expect("accepted whatsapp runtime-event sources");
    assert_eq!(
        accepted_runtime_event_sources,
        vec![
            "session_authorized",
            "runtime_start",
            "runtime_stop",
            "runtime_revoke",
            "runtime_relink",
            "runtime_rotate",
            "runtime_remove",
        ]
    );
}

#[tokio::test]
async fn whatsapp_runtime_bridge_authorized_session_material_is_stored_in_host_vault() {
    let test_context = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = test_context.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
        LOCAL_API_TOKEN,
        database_url.as_str(),
    )
    .with_test_dev_mode()
    .with_test_pairs([
        (
            "MAKOSH_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "MAKOSH_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
    ])
    .expect("config");
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-vault-{suffix}");
    let session_material = format!("whatsapp-runtime-bridge-session-{suffix}");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Vault Source",
            "external_account_id": format!("wa-runtime-bridge-vault-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-vault-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized",
            json!({
                "account_id": account_id,
                "session_material": session_material,
                "secret_kind": "other",
                "label": "WhatsApp runtime bridge session credential",
                "metadata": {
                    "source": "runtime-bridge-authorized-session-test",
                    "runtime": "webview_companion"
                }
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge authorized session response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let secret_ref = format!("secret:provider-account:{account_id}:whatsapp_web_session_key");
    assert_eq!(body["secret_ref"], json!(secret_ref));
    assert_eq!(body["store_kind"], json!("host_vault"));
    assert_eq!(body.get("session_material"), None);

    let status: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.runtime.status_changed'
          AND source->>'account_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge status payload");
    assert_eq!(status["status"], json!("linked"));
    assert_eq!(status["source"], json!("session_authorized"));
    assert!(status.get("session_material").is_none());
}

#[tokio::test]
async fn whatsapp_reauthorizing_session_rotates_vault_material_and_emits_rotated_events() {
    let test_context = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = test_context.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
        LOCAL_API_TOKEN,
        database_url.as_str(),
    )
    .with_test_dev_mode()
    .with_test_pairs([
        (
            "MAKOSH_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "MAKOSH_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
    ])
    .expect("config");
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-session-rotation-{suffix}");
    let first_session_material = format!("whatsapp-session-rotation-first-{suffix}");
    let second_session_material = format!("whatsapp-session-rotation-second-{suffix}");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Session Rotation",
            "external_account_id": format!("wa-session-rotation-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/session-rotation-{suffix}")
        }),
    )
    .await;

    let first_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/sessions/authorized",
            json!({
                "account_id": account_id,
                "session_material": first_session_material,
                "secret_kind": "other",
                "label": "WhatsApp initial session credential",
                "metadata": {
                    "source": "whatsapp-session-rotation-initial",
                    "runtime": "fixture"
                }
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("first authorized session response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let second_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/sessions/authorized",
            json!({
                "account_id": account_id,
                "session_material": second_session_material,
                "secret_kind": "other",
                "label": "WhatsApp rotated session credential",
                "metadata": {
                    "source": "whatsapp-session-rotation-rotated",
                    "runtime": "fixture"
                }
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("second authorized session response");
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_body = json_body(second_response).await;
    let secret_ref = format!("secret:provider-account:{account_id}:whatsapp_web_session_key");
    assert_eq!(second_body["secret_ref"], json!(secret_ref));

    let store = CommunicationIngestionStore::new(pool.clone());
    let bindings = store
        .provider_account_secret_bindings(&account_id)
        .await
        .expect("load provider account secret bindings");
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].secret_ref, secret_ref);

    let secret_reference = SecretReferenceStore::new(pool.clone())
        .secret_reference(&secret_ref)
        .await
        .expect("load rotated secret reference")
        .expect("rotated secret reference");
    assert_eq!(
        secret_reference.metadata["source"],
        json!("whatsapp-session-rotation-rotated")
    );

    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    let stored_secret = vault.read_secret(&secret_ref).expect("read rotated secret");
    assert_eq!(stored_secret, second_session_material);

    let runtime_events =
        whatsapp_event_payloads(&pool, "whatsapp.runtime.status_changed", &account_id).await;
    assert_eq!(runtime_events.len(), 2);
    assert_eq!(runtime_events[0]["source"], json!("session_authorized"));
    assert_eq!(runtime_events[1]["source"], json!("session_rotated"));
    assert_eq!(runtime_events[1]["status"], json!("linked"));
    assert!(runtime_events.iter().all(|payload| {
        payload.get("session_material").is_none() && payload.get("session_secret_ref").is_none()
    }));

    let session_events =
        whatsapp_event_payloads(&pool, "whatsapp.session.link_state_changed", &account_id).await;
    assert_eq!(session_events.len(), 2);
    assert_eq!(session_events[0]["source"], json!("session_authorized"));
    assert_eq!(session_events[1]["source"], json!("session_rotated"));
    assert_eq!(session_events[1]["link_state"], json!("linked"));
    assert!(session_events.iter().all(|payload| {
        payload.get("session_material").is_none() && payload.get("session_secret_ref").is_none()
    }));

    let accepted_runtime_event_sources: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->'metadata'->>'source'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->>'runtime_event_kind' IN ('session_authorized', 'session_rotated')
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .expect("accepted runtime event sources");
    assert_eq!(
        accepted_runtime_event_sources,
        vec!["session_authorized", "session_rotated"]
    );
}

#[tokio::test]
async fn whatsapp_runtime_bridge_live_webview_status_exposes_live_capabilities() {
    let test_context = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
        LOCAL_API_TOKEN,
        database_url.as_str(),
    )
    .with_test_dev_mode()
    .with_test_pairs([
        (
            "MAKOSH_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "MAKOSH_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
    ])
    .expect("config");
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-live-{suffix}");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "provider_shape": "whatsapp_web_companion",
            "display_name": "WhatsApp Live Runtime Bridge",
            "external_account_id": format!("wa-runtime-bridge-live-{suffix}"),
            "device_name": "Макошь Desktop Live Bridge",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-live-{suffix}")
        }),
    )
    .await;

    let authorized_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized",
            json!({
                "account_id": account_id,
                "session_material": format!("whatsapp-runtime-bridge-live-session-{suffix}"),
                "secret_kind": "other",
                "label": "WhatsApp runtime bridge live session credential",
                "metadata": {
                    "source": "runtime-bridge-live-webview-status-test",
                    "runtime": "webview_companion"
                }
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge authorized session response");
    assert_eq!(authorized_response.status(), StatusCode::OK);

    let runtime_event_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
            json!({
                "account_id": account_id,
                "provider_event_id": format!("wa-runtime-bridge-live-event-{suffix}"),
                "runtime_event_kind": "runtime.available",
                "runtime_status": "available",
                "lifecycle_state": "available",
                "severity": "info",
                "metadata": {
                    "runtime": "webview_companion"
                },
                "import_batch_id": format!("wa-runtime-bridge-live-batch-{suffix}"),
                "observed_at": Utc::now(),
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge runtime event response");
    assert_eq!(runtime_event_response.status(), StatusCode::OK);

    let status_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime status response");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_body = json_body(status_response).await;
    assert_eq!(status_body["runtime_kind"], json!("webview_companion"));
    assert_eq!(status_body["status"], json!("available"));
    assert_eq!(status_body["live_runtime_available"], json!(true));
    assert_eq!(status_body["live_send_available"], json!(true));
    assert_eq!(status_body["media_download_available"], json!(true));
    assert_eq!(status_body["media_upload_available"], json!(true));
    assert_eq!(status_body["session_restore_available"], json!(true));
    assert!(
        !status_body["runtime_blockers"]
            .as_array()
            .expect("runtime blockers")
            .iter()
            .any(|item| item == "live_whatsapp_runtime_blocked")
    );

    let start_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime/start",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    assert_eq!(start_body["runtime_kind"], json!("webview_companion"));
    assert_eq!(start_body["status"], json!("available"));
    assert_eq!(start_body["live_runtime_available"], json!(true));
    assert_eq!(start_body["live_send_available"], json!(true));
    assert_eq!(start_body["media_download_available"], json!(true));
    assert_eq!(start_body["media_upload_available"], json!(true));
    assert_eq!(start_body["session_restore_available"], json!(true));
    assert_eq!(start_body["last_error"], Value::Null);

    let health_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/runtime/health?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime health response");
    assert_eq!(health_response.status(), StatusCode::OK);
    let health_body = json_body(health_response).await;
    assert_eq!(health_body["healthy"], json!(true));
    assert_eq!(health_body["status"], json!("available"));
    assert_eq!(health_body["checks"]["live_runtime_available"], json!(true));
    assert_eq!(
        health_body["checks"]["session_restore_available"],
        json!(true)
    );
    assert_eq!(
        health_body["checks"]["runtime"]["kind"],
        json!("webview_companion")
    );
    assert_eq!(
        health_body["checks"]["webview"]["hidden_runtime_available"],
        json!(true)
    );
    assert_eq!(
        health_body["checks"]["validation"]["status"],
        json!("available")
    );

    let capabilities_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/accounts/{account_id}/capabilities"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account capabilities response");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = json_body(capabilities_response).await;
    assert_eq!(
        capabilities_body["account_scope"]["runtime_kind"],
        json!("webview_companion")
    );
    assert_eq!(
        capabilities_body["account_scope"]["lifecycle_state"],
        json!("available")
    );
    assert_capability_status(&capabilities_body, "messages.send_text", "degraded", true);
    assert_capability_status(&capabilities_body, "media.download", "degraded", false);
    assert_capability_status(&capabilities_body, "media.upload_send", "degraded", true);

    let stored_runtime: String = sqlx::query_scalar(
        "SELECT config->>'runtime' FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("stored runtime kind");
    assert_eq!(stored_runtime, "webview_companion");
}

#[tokio::test]
async fn whatsapp_router_build_does_not_restore_vault_session_without_operator_action() {
    let test_context = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-startup-restore-{suffix}");
    let secret_ref = format!("secret:provider-account:{account_id}:whatsapp_web_session_key");
    let session_material = format!("whatsapp-startup-session-material-{suffix}");
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

    store
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::WhatsappWeb,
                "WhatsApp Startup Restore",
                format!("wa-startup-restore-{suffix}"),
            )
            .config(json!({
                "runtime": "fixture",
                "provider_shape": "whatsapp_web_companion",
                "lifecycle_state": "linked",
                "local_state_path": format!("docker/data/whatsapp/{account_id}")
            })),
        )
        .await
        .expect("store WhatsApp provider account");

    sqlx::query(
        r#"
        INSERT INTO whatsapp_web_sessions (
            session_id,
            account_id,
            device_name,
            companion_runtime,
            link_state,
            local_state_path,
            metadata
        )
        VALUES ($1, $2, 'Макошь Desktop', 'fixture', 'linked', $3, $4)
        "#,
    )
    .bind(format!("whatsapp-startup-session-{suffix}"))
    .bind(&account_id)
    .bind(format!("docker/data/whatsapp/{account_id}"))
    .bind(json!({"source": "whatsapp-startup-restore-test"}))
    .execute(&pool)
    .await
    .expect("store WhatsApp session metadata");

    sqlx::query(
        r#"
        INSERT INTO secret_references (
            secret_ref,
            secret_kind,
            store_kind,
            label,
            metadata
        )
        VALUES ($1, 'other', 'host_vault', 'WhatsApp startup restore session', $2)
        "#,
    )
    .bind(&secret_ref)
    .bind(json!({
        "account_id": account_id,
        "purpose": "whatsapp_web_session_key",
        "source": "whatsapp-startup-restore-test"
    }))
    .execute(&pool)
    .await
    .expect("store WhatsApp secret reference");

    store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &account_id,
            ProviderAccountSecretPurpose::WhatsappWebSessionKey,
            &secret_ref,
        ))
        .await
        .expect("bind WhatsApp session secret");

    let vault = HostVault::new(HostVaultConfig {
        home: vault_home.clone(),
        dev_mode: true,
        dev_key_path: dev_key_path.clone(),
    })
    .expect("host vault");
    vault
        .collect_entropy(host_vault_entropy_events(2_000))
        .expect("collect host vault entropy");
    vault.create().expect("create host vault");
    vault.unlock_existing().expect("unlock host vault");
    vault
        .store_secret(
            &secret_ref,
            &session_material,
            SecretEntryContext {
                entry_kind: "provider_session",
                account_id: &account_id,
                purpose: "whatsapp_web_session_key",
                secret_kind: "other",
                label: "WhatsApp startup restore session",
                metadata: &json!({
                    "source": "whatsapp-startup-restore-test"
                }),
            },
        )
        .expect("store WhatsApp session material");

    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
        LOCAL_API_TOKEN,
        database_url.as_str(),
    )
    .with_test_dev_mode()
    .with_test_pairs([
        (
            "MAKOSH_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "MAKOSH_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
    ])
    .expect("config");
    let _app = build_router_with_database(config, database);

    let runtime_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE source->>'account_id' = $1
          AND payload->>'source' = 'startup_restore_reconcile'
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("load unexpected startup restore event count");
    assert_eq!(runtime_event_count, 0);

    let lifecycle_state: String = sqlx::query_scalar(
        "SELECT config->>'lifecycle_state' FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("load stored WhatsApp lifecycle state");
    assert_eq!(lifecycle_state, "linked");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_message_ingests_into_signal_spine() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-message-{suffix}");
    let provider_message_id = format!("wa-runtime-bridge-message-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Message Source",
            "external_account_id": format!("wa-runtime-bridge-message-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-message-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": format!("wa-runtime-bridge-chat-{suffix}"),
                "provider_message_id": provider_message_id,
                "chat_title": "Runtime Bridge Chat",
                "sender_id": format!("wa-runtime-bridge-sender-{suffix}"),
                "sender_display_name": "Runtime Bridge Sender",
                "text": "runtime bridge observed message",
                "import_batch_id": format!("wa-runtime-bridge-batch-{suffix}"),
                "occurred_at": "2026-06-06T13:30:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message response");
    assert_eq!(response.status(), StatusCode::OK);

    let payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.message.created'
          AND payload->>'provider_message_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&provider_message_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge created payload");
    assert_eq!(payload["source"], json!("runtime_bridge_message_ingest"));
    assert!(payload.get("text").is_none());

    let raw_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.whatsapp.message.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("runtime bridge raw count");
    let accepted_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.whatsapp.message'",
    )
    .fetch_one(&pool)
    .await
    .expect("runtime bridge accepted count");
    assert_eq!(raw_count, 1);
    assert_eq!(accepted_count, 1);
}

#[tokio::test]
async fn whatsapp_runtime_bridge_media_lifecycle_emits_media_and_runtime_events() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-media-{suffix}");
    let command_id = format!("wa-runtime-bridge-media-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Media Source",
            "external_account_id": format!("wa-runtime-bridge-media-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-media-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle",
            json!({
                "account_id": account_id,
                "command_id": command_id,
                "media_direction": "download",
                "lifecycle_phase": "progress",
                "provider_chat_id": format!("wa-runtime-bridge-chat-{suffix}"),
                "provider_message_id": format!("wa-runtime-bridge-message-{suffix}"),
                "provider_media_id": format!("wa-runtime-bridge-media-id-{suffix}"),
                "progress_percent": 42,
                "content_type": "image/jpeg",
                "filename": "bridge.jpg"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge media lifecycle response");
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let media_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.media.download.progress'
          AND payload->>'command_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge media payload");
    assert_eq!(media_payload["progress_percent"], json!(42));
    assert_eq!(
        media_payload["source"],
        json!("runtime_bridge_media_lifecycle")
    );

    let runtime_kind: String = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'command_id' = $2
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge accepted runtime event");
    assert_eq!(runtime_kind, "media.download.progress");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_receipt_records_live_observed_source_in_raw_provenance() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-receipt-source-{suffix}");
    let provider_chat_id = format!("wa-runtime-bridge-receipt-chat-{suffix}");
    let provider_message_id = format!("wa-runtime-bridge-receipt-message-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge Receipt Source",
                "external_account_id": format!("wa-runtime-bridge-receipt-source-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge receipt account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "Runtime Bridge Receipt Source Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Receipt Sender",
                "text": "Receipt source record target.",
                "import_batch_id": format!("whatsapp-runtime-bridge-receipt-message-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message response");
    assert_eq!(message_response.status(), StatusCode::OK);

    let receipt_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/receipts",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "delivery_state": "sent",
                "import_batch_id": format!("whatsapp-runtime-bridge-receipt-{suffix}"),
                "observed_at": "2026-06-06T12:01:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge receipt response");
    assert_eq!(receipt_response.status(), StatusCode::OK);
    let receipt_body = json_body(receipt_response).await;
    let raw_record_id = receipt_body["raw_record_id"]
        .as_str()
        .expect("receipt raw_record_id");

    let provenance: Value = sqlx::query_scalar(
        "SELECT provenance FROM communication_raw_records WHERE raw_record_id = $1",
    )
    .bind(raw_record_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge receipt raw provenance");
    assert_eq!(
        provenance["observed_source"],
        json!("provider_observed.runtime_bridge_receipt")
    );
}

#[tokio::test]
async fn whatsapp_runtime_bridge_status_view_and_delete_record_live_observed_source_in_raw_provenance()
 {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-status-source-{suffix}");
    let provider_status_id = format!("wa-runtime-bridge-status-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge Status Source",
                "external_account_id": format!("wa-runtime-bridge-status-source-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge status account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let status_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/statuses",
            json!({
                "account_id": account_id,
                "provider_status_id": provider_status_id,
                "sender_id": format!("wa:+34status{suffix}"),
                "sender_display_name": "Status Author",
                "text": "Observed live runtime bridge status.",
                "import_batch_id": format!("whatsapp-runtime-bridge-status-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge status response");
    assert_eq!(status_response.status(), StatusCode::OK);

    let status_view_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/status-views",
            json!({
                "account_id": account_id,
                "provider_status_id": provider_status_id,
                "viewer_id": format!("viewer-{suffix}"),
                "viewer_display_name": "Viewer",
                "import_batch_id": format!("whatsapp-runtime-bridge-status-view-{suffix}"),
                "observed_at": "2026-06-06T12:01:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge status view response");
    assert_eq!(status_view_response.status(), StatusCode::OK);
    let status_view_body = json_body(status_view_response).await;

    let status_delete_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/status-deletes",
            json!({
                "account_id": account_id,
                "provider_status_id": provider_status_id,
                "actor_class": "self",
                "reason_class": "status_expired",
                "import_batch_id": format!("whatsapp-runtime-bridge-status-delete-{suffix}"),
                "observed_at": "2026-06-06T12:02:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge status delete response");
    assert_eq!(status_delete_response.status(), StatusCode::OK);
    let status_delete_body = json_body(status_delete_response).await;

    let status_view_provenance: Value = sqlx::query_scalar(
        "SELECT provenance FROM communication_raw_records WHERE raw_record_id = $1",
    )
    .bind(
        status_view_body["raw_record_id"]
            .as_str()
            .expect("status view raw_record_id"),
    )
    .fetch_one(&pool)
    .await
    .expect("runtime bridge status view raw provenance");
    assert_eq!(
        status_view_provenance["observed_source"],
        json!("provider_observed.runtime_bridge_status_view")
    );

    let status_delete_provenance: Value = sqlx::query_scalar(
        "SELECT provenance FROM communication_raw_records WHERE raw_record_id = $1",
    )
    .bind(
        status_delete_body["raw_record_id"]
            .as_str()
            .expect("status delete raw_record_id"),
    )
    .fetch_one(&pool)
    .await
    .expect("runtime bridge status delete raw provenance");
    assert_eq!(
        status_delete_provenance["observed_source"],
        json!("provider_observed.runtime_bridge_status_delete")
    );
}

#[tokio::test]
async fn whatsapp_runtime_bridge_presence_and_call_record_live_observed_source_in_raw_provenance() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-presence-call-source-{suffix}");
    let provider_identity_id = format!("wa:+3412345{suffix}");
    let provider_call_id = format!("wa-runtime-bridge-call-{suffix}");
    let provider_chat_id = format!("wa-runtime-bridge-presence-call-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge Presence Call Source",
                "external_account_id": format!("wa-runtime-bridge-presence-call-source-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge presence/call account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let presence_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/presence",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_identity_id": provider_identity_id,
                "identity_kind": "whatsapp_user",
                "display_name": "Presence User",
                "presence_state": "typing",
                "last_seen_at": "2026-06-06T11:59:00Z",
                "import_batch_id": format!("whatsapp-runtime-bridge-presence-{suffix}"),
                "observed_at": "2026-06-06T12:00:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge presence response");
    assert_eq!(presence_response.status(), StatusCode::OK);
    let presence_body = json_body(presence_response).await;

    let call_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/calls",
            json!({
                "account_id": account_id,
                "provider_call_id": provider_call_id,
                "provider_chat_id": provider_chat_id,
                "direction": "incoming",
                "call_state": "missed",
                "started_at": "2026-06-06T12:57:00Z",
                "ended_at": "2026-06-06T12:57:12Z",
                "metadata": {
                    "call_kind": "voice",
                    "provider_participant_id": provider_identity_id
                },
                "import_batch_id": format!("whatsapp-runtime-bridge-call-{suffix}"),
                "observed_at": "2026-06-06T12:59:50Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge call response");
    assert_eq!(call_response.status(), StatusCode::OK);
    let call_body = json_body(call_response).await;

    let presence_provenance: Value = sqlx::query_scalar(
        "SELECT provenance FROM communication_raw_records WHERE raw_record_id = $1",
    )
    .bind(
        presence_body["raw_record_id"]
            .as_str()
            .expect("presence raw_record_id"),
    )
    .fetch_one(&pool)
    .await
    .expect("runtime bridge presence raw provenance");
    assert_eq!(
        presence_provenance["observed_source"],
        json!("provider_observed.runtime_bridge_presence")
    );

    let call_provenance: Value = sqlx::query_scalar(
        "SELECT provenance FROM communication_raw_records WHERE raw_record_id = $1",
    )
    .bind(
        call_body["raw_record_id"]
            .as_str()
            .expect("call raw_record_id"),
    )
    .fetch_one(&pool)
    .await
    .expect("runtime bridge call raw provenance");
    assert_eq!(
        call_provenance["observed_source"],
        json!("provider_observed.runtime_bridge_call")
    );
}

#[tokio::test]
async fn whatsapp_runtime_bridge_message_family_records_live_observed_source_in_raw_provenance() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-message-family-source-{suffix}");
    let provider_chat_id = format!("wa-runtime-bridge-message-family-chat-{suffix}");
    let provider_message_id = format!("wa-runtime-bridge-message-family-message-{suffix}");
    let provider_status_id = format!("wa-runtime-bridge-message-family-status-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge Message Family Source",
                "external_account_id": format!("wa-runtime-bridge-message-family-source-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message family account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "Runtime Bridge Message Family Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Message Sender",
                "text": "Seed runtime bridge message family payload.",
                "import_batch_id": format!("whatsapp-runtime-bridge-message-family-message-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message family message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;

    let reaction_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/reactions",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "provider_actor_id": format!("actor-{suffix}"),
                "sender_display_name": "Reaction Sender",
                "reaction": "fire",
                "is_active": true,
                "import_batch_id": format!("whatsapp-runtime-bridge-message-family-reaction-{suffix}"),
                "observed_at": "2026-06-06T12:01:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message family reaction response");
    assert_eq!(reaction_response.status(), StatusCode::OK);
    let reaction_body = json_body(reaction_response).await;

    let update_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/message-updates",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "text": "Updated runtime bridge message family payload.",
                "import_batch_id": format!("whatsapp-runtime-bridge-message-family-update-{suffix}"),
                "observed_at": "2026-06-06T12:02:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message family update response");
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_body = json_body(update_response).await;

    let media_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/media",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "provider_attachment_id": format!("attachment-{suffix}"),
                "filename": "bridge.jpg",
                "content_type": "image/jpeg",
                "size_bytes": 128,
                "sha256": "000000000000000000000000000000000000000000000000000000000000002a",
                "storage_kind": "local_blob",
                "storage_path": format!("docker/data/whatsapp/runtime-bridge-media-family-{suffix}.jpg"),
                "import_batch_id": format!("whatsapp-runtime-bridge-message-family-media-{suffix}"),
                "observed_at": "2026-06-06T12:03:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message family media response");
    assert_eq!(media_response.status(), StatusCode::OK);
    let media_body = json_body(media_response).await;

    let delete_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/message-deletes",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reason_class": "deleted_for_everyone",
                "actor_class": "self",
                "import_batch_id": format!("whatsapp-runtime-bridge-message-family-delete-{suffix}"),
                "observed_at": "2026-06-06T12:04:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message family delete response");
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_body = json_body(delete_response).await;

    let dialog_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/dialogs",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "chat_title": "Runtime Bridge Message Family Chat",
                "chat_kind": "group",
                "participant_count": 3,
                "import_batch_id": format!("whatsapp-runtime-bridge-message-family-dialog-{suffix}"),
                "observed_at": "2026-06-06T12:05:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message family dialog response");
    assert_eq!(dialog_response.status(), StatusCode::OK);
    let dialog_body = json_body(dialog_response).await;

    let participant_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/participants",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "chat_title": "Runtime Bridge Message Family Chat",
                "chat_kind": "group",
                "provider_identity_id": format!("participant-{suffix}"),
                "identity_kind": "whatsapp_user",
                "display_name": "Participant",
                "role": "member",
                "status": "joined",
                "import_batch_id": format!("whatsapp-runtime-bridge-message-family-participant-{suffix}"),
                "observed_at": "2026-06-06T12:06:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message family participant response");
    assert_eq!(participant_response.status(), StatusCode::OK);
    let participant_body = json_body(participant_response).await;

    let status_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/statuses",
            json!({
                "account_id": account_id,
                "provider_status_id": provider_status_id,
                "sender_id": format!("status-sender-{suffix}"),
                "sender_display_name": "Status Sender",
                "text": "Runtime bridge status family payload.",
                "import_batch_id": format!("whatsapp-runtime-bridge-message-family-status-{suffix}"),
                "occurred_at": "2026-06-06T12:07:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge message family status response");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_body = json_body(status_response).await;

    let raw_expectations = vec![
        (
            message_body["raw_record_id"].as_str().expect("message raw"),
            "provider_observed.runtime_bridge_message",
        ),
        (
            reaction_body["raw_record_id"]
                .as_str()
                .expect("reaction raw"),
            "provider_observed.runtime_bridge_reaction",
        ),
        (
            update_body["raw_record_id"].as_str().expect("update raw"),
            "provider_observed.runtime_bridge_message_update",
        ),
        (
            media_body["raw_record_id"].as_str().expect("media raw"),
            "provider_observed.runtime_bridge_media",
        ),
        (
            delete_body["raw_record_id"].as_str().expect("delete raw"),
            "provider_observed.runtime_bridge_message_delete",
        ),
        (
            dialog_body["raw_record_id"].as_str().expect("dialog raw"),
            "provider_observed.runtime_bridge_dialog",
        ),
        (
            participant_body["raw_record_id"]
                .as_str()
                .expect("participant raw"),
            "provider_observed.runtime_bridge_participant",
        ),
        (
            status_body["raw_record_id"].as_str().expect("status raw"),
            "provider_observed.runtime_bridge_status",
        ),
    ];

    for (raw_record_id, expected_source) in raw_expectations {
        let provenance: Value = sqlx::query_scalar(
            "SELECT provenance FROM communication_raw_records WHERE raw_record_id = $1",
        )
        .bind(raw_record_id)
        .fetch_one(&pool)
        .await
        .expect("runtime bridge family raw provenance");
        assert_eq!(provenance["observed_source"], json!(expected_source));
    }
}

#[tokio::test]
async fn whatsapp_runtime_bridge_runtime_event_records_live_observed_source_in_raw_provenance() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-runtime-event-source-{suffix}");
    let provider_event_id = format!("wa-runtime-bridge-runtime-event-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Runtime Event Source",
            "external_account_id": format!("wa-runtime-bridge-runtime-event-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-runtime-event-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
            json!({
                "account_id": account_id,
                "provider_event_id": provider_event_id,
                "runtime_event_kind": "provider.runtime.warning",
                "runtime_status": "degraded",
                "lifecycle_state": "degraded",
                "severity": "warning",
                "metadata": {
                    "warning_code": "bridge_warning"
                },
                "import_batch_id": format!("whatsapp-runtime-bridge-runtime-event-ingest-{suffix}"),
                "observed_at": "2026-06-06T12:00:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge runtime event response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;

    let provenance: Value = sqlx::query_scalar(
        "SELECT provenance FROM communication_raw_records WHERE raw_record_id = $1",
    )
    .bind(
        body["raw_record_id"]
            .as_str()
            .expect("runtime event raw_record_id"),
    )
    .fetch_one(&pool)
    .await
    .expect("runtime bridge runtime event raw provenance");
    assert_eq!(
        provenance["observed_source"],
        json!("provider_observed.runtime_bridge_runtime_event")
    );
}

#[tokio::test]
async fn whatsapp_runtime_bridge_sync_lifecycle_emits_sync_and_runtime_events() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-sync-{suffix}");
    let subject_id = format!("wa-runtime-bridge-sync-subject-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Sync Source",
            "external_account_id": format!("wa-runtime-bridge-sync-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-sync-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            json!({
                "account_id": account_id,
                "scope": "history",
                "phase": "progress",
                "subject_id": subject_id,
                "provider_chat_id": format!("wa-runtime-bridge-chat-{suffix}"),
                "synced_count": 7,
                "has_more": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge sync lifecycle response");
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let sync_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.sync.progress'
          AND source->>'account_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge sync payload");
    assert_eq!(sync_payload["scope"], json!("history"));
    assert_eq!(sync_payload["synced_count"], json!(7));
    assert_eq!(sync_payload["has_more"], json!(true));
    assert_eq!(
        sync_payload["source"],
        json!("runtime_bridge_sync_lifecycle")
    );

    let runtime_kind: String = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'subject_id' = $2
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .bind(&subject_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge sync runtime event");
    assert_eq!(runtime_kind, "sync.history.progress");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_members_sync_lifecycle_emits_sync_and_runtime_events() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-members-sync-{suffix}");
    let subject_id = format!("wa-runtime-bridge-members-subject-{suffix}");
    let provider_chat_id = format!("wa-runtime-bridge-members-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Members Sync Source",
            "external_account_id": format!("wa-runtime-bridge-members-sync-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-members-sync-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            json!({
                "account_id": account_id,
                "scope": "members",
                "phase": "progress",
                "subject_id": subject_id,
                "provider_chat_id": provider_chat_id,
                "synced_count": 3,
                "has_more": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge members sync lifecycle response");
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let sync_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.sync.progress'
          AND source->>'account_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge members sync payload");
    assert_eq!(sync_payload["scope"], json!("members"));
    assert_eq!(sync_payload["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(sync_payload["synced_count"], json!(3));
    assert_eq!(sync_payload["has_more"], json!(false));
    assert_eq!(
        sync_payload["source"],
        json!("runtime_bridge_sync_lifecycle")
    );

    let runtime_kind: String = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'subject_id' = $2
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .bind(&subject_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge members sync runtime event");
    assert_eq!(runtime_kind, "sync.members.progress");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_statuses_sync_lifecycle_emits_sync_and_runtime_events() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-statuses-sync-{suffix}");
    let subject_id = format!("wa-runtime-bridge-statuses-subject-{suffix}");
    let provider_chat_id = "status-feed";
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Statuses Sync Source",
            "external_account_id": format!("wa-runtime-bridge-statuses-sync-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-statuses-sync-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            json!({
                "account_id": account_id,
                "scope": "statuses",
                "phase": "progress",
                "subject_id": subject_id,
                "provider_chat_id": provider_chat_id,
                "synced_count": 2,
                "has_more": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge statuses sync lifecycle response");
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let sync_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.sync.progress'
          AND source->>'account_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge statuses sync payload");
    assert_eq!(sync_payload["scope"], json!("statuses"));
    assert_eq!(sync_payload["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(sync_payload["synced_count"], json!(2));
    assert_eq!(sync_payload["has_more"], json!(false));

    let runtime_kind: String = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'subject_id' = $2
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .bind(&subject_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge statuses sync runtime event");
    assert_eq!(runtime_kind, "sync.statuses.progress");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_presence_sync_lifecycle_emits_sync_and_runtime_events() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-presence-sync-{suffix}");
    let subject_id = format!("wa-runtime-bridge-presence-subject-{suffix}");
    let provider_chat_id = format!("wa-runtime-bridge-presence-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Presence Sync Source",
            "external_account_id": format!("wa-runtime-bridge-presence-sync-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-presence-sync-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            json!({
                "account_id": account_id,
                "scope": "presence",
                "phase": "progress",
                "subject_id": subject_id,
                "provider_chat_id": provider_chat_id,
                "synced_count": 1,
                "has_more": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge presence sync lifecycle response");
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let sync_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.sync.progress'
          AND source->>'account_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge presence sync payload");
    assert_eq!(sync_payload["scope"], json!("presence"));
    assert_eq!(sync_payload["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(sync_payload["synced_count"], json!(1));
    assert_eq!(sync_payload["has_more"], json!(false));

    let runtime_kind: String = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'subject_id' = $2
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .bind(&subject_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge presence sync runtime event");
    assert_eq!(runtime_kind, "sync.presence.progress");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_calls_sync_lifecycle_emits_sync_and_runtime_events() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-calls-sync-{suffix}");
    let subject_id = format!("wa-runtime-bridge-calls-subject-{suffix}");
    let provider_chat_id = format!("wa-runtime-bridge-calls-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Calls Sync Source",
            "external_account_id": format!("wa-runtime-bridge-calls-sync-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-calls-sync-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            json!({
                "account_id": account_id,
                "scope": "calls",
                "phase": "progress",
                "subject_id": subject_id,
                "provider_chat_id": provider_chat_id,
                "synced_count": 1,
                "has_more": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge calls sync lifecycle response");
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let sync_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.sync.progress'
          AND source->>'account_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge calls sync payload");
    assert_eq!(sync_payload["scope"], json!("calls"));
    assert_eq!(sync_payload["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(sync_payload["synced_count"], json!(1));
    assert_eq!(sync_payload["has_more"], json!(false));

    let runtime_kind: String = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'subject_id' = $2
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .bind(&subject_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge calls sync runtime event");
    assert_eq!(runtime_kind, "sync.calls.progress");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_contacts_sync_lifecycle_emits_sync_and_runtime_events() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-contacts-sync-{suffix}");
    let subject_id = format!("wa-runtime-bridge-contacts-subject-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Contacts Sync Source",
            "external_account_id": format!("wa-runtime-bridge-contacts-sync-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-contacts-sync-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            json!({
                "account_id": account_id,
                "scope": "contacts",
                "phase": "progress",
                "subject_id": subject_id,
                "synced_count": 2,
                "has_more": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge contacts sync lifecycle response");
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let sync_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.sync.progress'
          AND source->>'account_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge contacts sync payload");
    assert_eq!(sync_payload["scope"], json!("contacts"));
    assert_eq!(sync_payload["synced_count"], json!(2));
    assert_eq!(sync_payload["has_more"], json!(false));

    let runtime_kind: String = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'subject_id' = $2
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .bind(&subject_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge contacts sync runtime event");
    assert_eq!(runtime_kind, "sync.contacts.progress");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_media_sync_lifecycle_emits_sync_and_runtime_events() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-media-sync-{suffix}");
    let subject_id = format!("wa-runtime-bridge-media-subject-{suffix}");
    let provider_chat_id = format!("wa-runtime-bridge-media-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Media Sync Source",
            "external_account_id": format!("wa-runtime-bridge-media-sync-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-media-sync-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            json!({
                "account_id": account_id,
                "scope": "media",
                "phase": "progress",
                "subject_id": subject_id,
                "provider_chat_id": provider_chat_id,
                "synced_count": 4,
                "has_more": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge media sync lifecycle response");
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let sync_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.sync.progress'
          AND source->>'account_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge media sync payload");
    assert_eq!(sync_payload["scope"], json!("media"));
    assert_eq!(sync_payload["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(sync_payload["synced_count"], json!(4));
    assert_eq!(sync_payload["has_more"], json!(true));
    assert_eq!(
        sync_payload["source"],
        json!("runtime_bridge_sync_lifecycle")
    );

    let runtime_kind: String = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'subject_id' = $2
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .bind(&subject_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge media sync runtime event");
    assert_eq!(runtime_kind, "sync.media.progress");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_lifecycle_events_record_live_observed_source_in_raw_provenance() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let media_account_id = format!("whatsapp-runtime-bridge-media-source-{suffix}");
    let sync_account_id = format!("whatsapp-runtime-bridge-sync-source-{suffix}");
    let media_command_id = format!("wa-runtime-bridge-media-source-{suffix}");
    let sync_subject_id = format!("wa-runtime-bridge-sync-subject-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": media_account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Media Lifecycle Source",
            "external_account_id": format!("wa-runtime-bridge-media-lifecycle-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-media-lifecycle-{suffix}")
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": sync_account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Bridge Sync Lifecycle Source",
            "external_account_id": format!("wa-runtime-bridge-sync-lifecycle-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-bridge-sync-lifecycle-{suffix}")
        }),
    )
    .await;

    let media_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle",
            json!({
                "account_id": media_account_id,
                "command_id": media_command_id,
                "media_direction": "download",
                "lifecycle_phase": "progress",
                "provider_chat_id": format!("wa-runtime-bridge-media-chat-{suffix}"),
                "provider_message_id": format!("wa-runtime-bridge-media-message-{suffix}"),
                "provider_media_id": format!("wa-runtime-bridge-media-id-{suffix}"),
                "progress_percent": 42,
                "content_type": "image/jpeg",
                "filename": "bridge.jpg"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge media lifecycle provenance response");
    assert_eq!(media_response.status(), StatusCode::ACCEPTED);

    let sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            json!({
                "account_id": sync_account_id,
                "scope": "history",
                "phase": "progress",
                "subject_id": sync_subject_id,
                "provider_chat_id": format!("wa-runtime-bridge-sync-chat-{suffix}"),
                "synced_count": 7,
                "has_more": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge sync lifecycle provenance response");
    assert_eq!(sync_response.status(), StatusCode::ACCEPTED);

    let media_provenance: Value = sqlx::query_scalar(
        r#"
        SELECT provenance
        FROM communication_raw_records
        WHERE record_kind = 'whatsapp_web_runtime_event'
          AND account_id = $1
          AND payload->>'runtime_event_kind' = 'media.download.progress'
        ORDER BY captured_at DESC
        LIMIT 1
        "#,
    )
    .bind(&media_account_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge media lifecycle raw provenance");
    assert_eq!(
        media_provenance["observed_source"],
        json!("runtime_bridge_media_lifecycle")
    );

    let sync_provenance: Value = sqlx::query_scalar(
        r#"
        SELECT provenance
        FROM communication_raw_records
        WHERE record_kind = 'whatsapp_web_runtime_event'
          AND account_id = $1
          AND payload->>'runtime_event_kind' = 'sync.history.progress'
        ORDER BY captured_at DESC
        LIMIT 1
        "#,
    )
    .bind(&sync_account_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge sync lifecycle raw provenance");
    assert_eq!(
        sync_provenance["observed_source"],
        json!("runtime_bridge_sync_lifecycle")
    );
}

#[tokio::test]
async fn whatsapp_runtime_bridge_claim_commands_claims_live_due_command() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-claim-{suffix}");
    let session_secret_ref =
        format!("secret:provider-account:{account_id}:whatsapp_web_session_key");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );
    unlock_test_vault(app.clone()).await;

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge Claim Source",
                "external_account_id": format!("wa-runtime-bridge-claim-source-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("live blocked account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let authorized_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized",
            json!({
                "account_id": account_id,
                "session_material": format!("whatsapp-runtime-bridge-claim-session-{suffix}"),
                "secret_kind": "other",
                "label": "WhatsApp runtime bridge claim session credential",
                "metadata": {
                    "source": "runtime-bridge-claim-command-test",
                    "runtime": "webview_companion"
                }
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge authorized session response");
    assert_eq!(authorized_response.status(), StatusCode::OK);

    let command_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-commands/messages/send",
            json!({
                "command_id": format!("wa-runtime-bridge-claim-command-{suffix}"),
                "idempotency_key": format!("wa-runtime-bridge-claim-key-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": format!("wa-runtime-bridge-claim-chat-{suffix}"),
                "text": "runtime bridge claim message"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("provider command response");
    assert_eq!(command_response.status(), StatusCode::OK);
    let command_body = json_body(command_response).await;
    let command_id = command_body["command_id"]
        .as_str()
        .expect("command id")
        .to_owned();

    sqlx::query(
        "UPDATE whatsapp_provider_write_commands SET capability_state = 'available', status = 'queued', confirmation_decision = 'confirmed' WHERE command_id = $1",
    )
    .bind(&command_id)
    .execute(&pool)
    .await
    .expect("prepare live claimable command");

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/commands/claim",
            json!({
                "account_id": account_id,
                "limit": 10
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge claim response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["items"][0]["command_id"], json!(command_id));
    assert_eq!(body["items"][0]["status"], json!("executing"));
    assert_eq!(body["items"][0]["provider_kind"], json!("whatsapp_web"));
    assert_eq!(
        body["items"][0]["provider_shape"],
        json!("whatsapp_web_companion")
    );
    assert_eq!(body["items"][0]["runtime_kind"], json!("webview_companion"));
    assert_eq!(body["items"][0]["lifecycle_state"], json!("linked"));
    assert_eq!(body["items"][0]["session_restore_available"], json!(true));
    assert_eq!(body["items"][0]["capability_state"], json!("available"));
    assert_eq!(body["items"][0]["action_class"], json!("provider_write"));
    assert_eq!(
        body["items"][0]["confirmation_decision"],
        json!("confirmed")
    );
    assert_eq!(body["items"][0]["provider_state"], json!({}));
    assert_eq!(
        body["items"][0]["result_payload"]["delivery_state"],
        json!("not_attempted")
    );
    assert_eq!(
        body["items"][0]["audit_metadata"]["session_restore_available"],
        json!(true)
    );
    assert_eq!(
        body["items"][0]["payload"]["text"],
        json!("runtime bridge claim message")
    );
    assert_eq!(body["items"][0]["runtime_blockers"], json!([]));

    let canonical_claim_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("canonical claimed command status");
    assert_eq!(canonical_claim_status, "executing");

    let bound_secret_ref: Option<String> = sqlx::query_scalar(
        r#"
        SELECT secret_ref
        FROM communication_provider_account_secret_refs
        WHERE account_id = $1
          AND secret_purpose = 'whatsapp_web_session_key'
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("bound session secret ref");
    assert_eq!(bound_secret_ref, Some(session_secret_ref));

    let blocked_account_id = format!("whatsapp-runtime-bridge-claim-blocked-{suffix}");
    let blocked_account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": blocked_account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge Blocked Claim Source",
                "external_account_id": format!("wa-runtime-bridge-claim-blocked-source-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("live blocked account response");
    assert_eq!(blocked_account_response.status(), StatusCode::OK);

    let blocked_command_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-commands/messages/send",
            json!({
                "command_id": format!("wa-runtime-bridge-blocked-claim-command-{suffix}"),
                "idempotency_key": format!("wa-runtime-bridge-blocked-claim-key-{suffix}"),
                "account_id": blocked_account_id,
                "provider_chat_id": format!("wa-runtime-bridge-blocked-claim-chat-{suffix}"),
                "text": "runtime bridge blocked claim message"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("blocked provider command response");
    assert_eq!(blocked_command_response.status(), StatusCode::OK);
    let blocked_command_body = json_body(blocked_command_response).await;
    let blocked_command_id = blocked_command_body["command_id"]
        .as_str()
        .expect("blocked command id")
        .to_owned();

    sqlx::query(
        "UPDATE whatsapp_provider_write_commands SET capability_state = 'available', status = 'queued', confirmation_decision = 'confirmed' WHERE command_id = $1",
    )
    .bind(&blocked_command_id)
    .execute(&pool)
    .await
    .expect("prepare blocked live command");

    let blocked_claim_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/commands/claim",
            json!({
                "account_id": blocked_account_id,
                "limit": 10
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("blocked runtime bridge claim response");
    assert_eq!(blocked_claim_response.status(), StatusCode::OK);
    let blocked_claim_body = json_body(blocked_claim_response).await;
    assert_eq!(blocked_claim_body["items"], json!([]));

    let blocked_command_status: String = sqlx::query_scalar(
        "SELECT status FROM whatsapp_provider_write_commands WHERE command_id = $1",
    )
    .bind(&blocked_command_id)
    .fetch_one(&pool)
    .await
    .expect("blocked command status after claim attempt");
    assert_eq!(blocked_command_status, "queued");

    let unrestored_account_id = format!("whatsapp-runtime-bridge-claim-no-session-{suffix}");
    let unrestored_account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": unrestored_account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge No Session Claim Source",
                "external_account_id": format!("wa-runtime-bridge-claim-no-session-source-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("unrestored account response");
    assert_eq!(unrestored_account_response.status(), StatusCode::OK);

    sqlx::query(
        r#"
        UPDATE communication_provider_accounts
        SET config = config || $2::jsonb
        WHERE account_id = $1
        "#,
    )
    .bind(&unrestored_account_id)
    .bind(json!({
        "runtime": "webview_companion",
        "lifecycle_state": "linked",
        "provider_shape": "whatsapp_web_companion"
    }))
    .execute(&pool)
    .await
    .expect("force linked runtime without session");

    let unrestored_command_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-commands/messages/send",
            json!({
                "command_id": format!("wa-runtime-bridge-no-session-claim-command-{suffix}"),
                "idempotency_key": format!("wa-runtime-bridge-no-session-claim-key-{suffix}"),
                "account_id": unrestored_account_id,
                "provider_chat_id": format!("wa-runtime-bridge-no-session-claim-chat-{suffix}"),
                "text": "runtime bridge no-session claim message"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("unrestored provider command response");
    assert_eq!(unrestored_command_response.status(), StatusCode::OK);
    let unrestored_command_body = json_body(unrestored_command_response).await;
    assert_eq!(
        unrestored_command_body["session_restore_available"],
        json!(false)
    );
    let unrestored_command_id = unrestored_command_body["command_id"]
        .as_str()
        .expect("unrestored command id")
        .to_owned();

    sqlx::query(
        "UPDATE whatsapp_provider_write_commands SET capability_state = 'available', status = 'queued', confirmation_decision = 'confirmed' WHERE command_id = $1",
    )
    .bind(&unrestored_command_id)
    .execute(&pool)
    .await
    .expect("prepare unrestored live command");

    let unrestored_claim_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/commands/claim",
            json!({
                "account_id": unrestored_account_id,
                "limit": 10
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("unrestored runtime bridge claim response");
    assert_eq!(unrestored_claim_response.status(), StatusCode::OK);
    let unrestored_claim_body = json_body(unrestored_claim_response).await;
    assert_eq!(unrestored_claim_body["items"], json!([]));

    let unrestored_command_status: String = sqlx::query_scalar(
        "SELECT status FROM whatsapp_provider_write_commands WHERE command_id = $1",
    )
    .bind(&unrestored_command_id)
    .fetch_one(&pool)
    .await
    .expect("unrestored command status after claim attempt");
    assert_eq!(unrestored_command_status, "queued");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_command_failed_reschedules_live_command() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let command_id = format!("wa-runtime-bridge-failed-{suffix}");

    sqlx::query(
        r#"
        INSERT INTO communication_provider_accounts (
            account_id, provider_kind, display_name, external_account_id, config
        ) VALUES ($1, 'whatsapp_web', 'Runtime Bridge Failed Source', $2, $3::jsonb)
        "#,
    )
    .bind(format!("whatsapp-runtime-bridge-failed-account-{suffix}"))
    .bind(format!("wa-runtime-bridge-failed-source-{suffix}"))
    .bind(json!({
        "runtime": "live_blocked",
        "provider_shape": "whatsapp_web_companion",
        "lifecycle_state": "linked"
    }))
    .execute(&pool)
    .await
    .expect("insert account");

    sqlx::query(
        r#"
        INSERT INTO whatsapp_provider_write_commands (
            command_id, account_id, command_kind, idempotency_key,
            provider_chat_id, capability_state, action_class, confirmation_decision,
            status, retry_count, max_retries, payload, target_ref, result_payload,
            audit_metadata, actor_id, provider_state, reconciliation_status, created_at, updated_at,
            last_attempt_at, locked_at, locked_by
        ) VALUES (
            $1, $2, 'send_text', $3,
            $4, 'available', 'provider_write', 'confirmed',
            'executing', 1, 3, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb,
            '{}'::jsonb, 'makosh-frontend', '{}'::jsonb, 'awaiting_provider', NOW(), NOW(),
            NOW(), NOW(), 'whatsapp-runtime-bridge-worker'
        )
        "#,
    )
    .bind(&command_id)
    .bind(format!("whatsapp-runtime-bridge-failed-account-{suffix}"))
    .bind(format!("wa-runtime-bridge-failed-key-{suffix}"))
    .bind(format!("wa-runtime-bridge-failed-chat-{suffix}"))
    .execute(&pool)
    .await
    .expect("insert executing command");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        Database::connect(Some(&database_url))
            .await
            .expect("database reconnect"),
    );

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/whatsapp/runtime-bridge/commands/{command_id}/failed"),
            json!({
                "error_message": "runtime bridge simulated failure",
                "error_code": "websocket_timeout",
                "retry_after_seconds": 90
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge failed response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["command_id"], json!(command_id));
    assert_eq!(body["status"], json!("retrying"));
    assert_eq!(
        body["last_error"],
        json!("runtime bridge simulated failure")
    );
    assert_eq!(
        body["result_payload"]["failure"]["error_code"],
        json!("websocket_timeout")
    );
    assert_eq!(
        body["result_payload"]["failure"]["retry_after_seconds"],
        json!(90)
    );
    assert_eq!(
        body["provider_state"]["last_failure"]["reported_via"],
        json!("runtime_bridge_failed")
    );
    assert_eq!(
        body["provider_state"]["last_failure"]["error_code"],
        json!("websocket_timeout")
    );

    let canonical_row = sqlx::query(
        r#"
        SELECT status, last_error, next_attempt_at, result_payload, provider_state
        FROM communication_provider_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("canonical failed command status");
    assert_eq!(
        canonical_row
            .try_get::<String, _>("status")
            .expect("canonical status"),
        "retrying"
    );
    assert_eq!(
        canonical_row
            .try_get::<Option<String>, _>("last_error")
            .expect("canonical last_error"),
        Some("runtime bridge simulated failure".to_owned())
    );
    let next_attempt_at: chrono::DateTime<chrono::Utc> = canonical_row
        .try_get("next_attempt_at")
        .expect("canonical next_attempt_at");
    let retry_delay_seconds = (next_attempt_at.timestamp() - chrono::Utc::now().timestamp()).abs();
    assert!(
        (60..=120).contains(&retry_delay_seconds),
        "expected retry delay near requested 90s, got {retry_delay_seconds}s"
    );
    let canonical_result_payload: Value = canonical_row
        .try_get("result_payload")
        .expect("canonical result_payload");
    assert_eq!(
        canonical_result_payload["failure"]["error_code"],
        json!("websocket_timeout")
    );
    let canonical_provider_state: Value = canonical_row
        .try_get("provider_state")
        .expect("canonical provider_state");
    assert_eq!(
        canonical_provider_state["last_failure"]["retry_after_seconds"],
        json!(90)
    );
}

#[tokio::test]
async fn whatsapp_runtime_bridge_claim_recovers_only_live_stale_commands_for_requested_account() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let live_account_id = format!("whatsapp-runtime-bridge-stale-live-{suffix}");
    let other_live_account_id = format!("whatsapp-runtime-bridge-stale-other-live-{suffix}");
    let fixture_account_id = format!("whatsapp-runtime-bridge-stale-fixture-{suffix}");
    let live_command_id = format!("wa-runtime-bridge-stale-live-command-{suffix}");
    let other_live_command_id = format!("wa-runtime-bridge-stale-other-live-command-{suffix}");
    let fixture_command_id = format!("wa-runtime-bridge-stale-fixture-command-{suffix}");

    for (account_id, runtime_kind) in [
        (&live_account_id, "live_blocked"),
        (&other_live_account_id, "webview_companion"),
        (&fixture_account_id, "fixture"),
    ] {
        sqlx::query(
            r#"
            INSERT INTO communication_provider_accounts (
                account_id, provider_kind, display_name, external_account_id, config
            ) VALUES ($1, 'whatsapp_web', $2, $3, $4::jsonb)
            "#,
        )
        .bind(account_id)
        .bind(format!("WhatsApp account {account_id}"))
        .bind(format!("external-{account_id}"))
        .bind(json!({
            "runtime": runtime_kind,
            "provider_shape": "whatsapp_web_companion",
            "lifecycle_state": "linked"
        }))
        .execute(&pool)
        .await
        .expect("insert account");
    }

    for (command_id, account_id) in [
        (&live_command_id, &live_account_id),
        (&other_live_command_id, &other_live_account_id),
        (&fixture_command_id, &fixture_account_id),
    ] {
        sqlx::query(
            r#"
            INSERT INTO whatsapp_provider_write_commands (
                command_id, account_id, command_kind, idempotency_key,
                provider_chat_id, capability_state, action_class, confirmation_decision,
                status, retry_count, max_retries, payload, target_ref, result_payload,
                audit_metadata, actor_id, provider_state, reconciliation_status, created_at, updated_at,
                last_attempt_at, locked_at, locked_by
            ) VALUES (
                $1, $2, 'send_text', $3,
                $4, 'available', 'provider_write', 'confirmed',
                'executing', 1, 3, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb,
                '{}'::jsonb, 'makosh-frontend', '{}'::jsonb, 'awaiting_provider', NOW(), NOW(),
                NOW() - INTERVAL '5 minutes', NOW() - INTERVAL '5 minutes', 'whatsapp-worker-test'
            )
            "#,
        )
        .bind(command_id)
        .bind(account_id)
        .bind(format!("key-{command_id}"))
        .bind(format!("chat-{command_id}"))
        .execute(&pool)
        .await
        .expect("insert stale executing command");
    }

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        Database::connect(Some(&database_url))
            .await
            .expect("database reconnect"),
    );

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/commands/claim",
            json!({
                "account_id": live_account_id,
                "limit": 10
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge claim response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["items"], json!([]));

    let live_row = sqlx::query(
        "SELECT status, result_payload, provider_state FROM whatsapp_provider_write_commands WHERE command_id = $1",
    )
    .bind(&live_command_id)
    .fetch_one(&pool)
    .await
    .expect("live stale command row");
    assert_eq!(
        live_row
            .try_get::<String, _>("status")
            .expect("live status"),
        "retrying"
    );
    let live_result_payload: Value = live_row
        .try_get("result_payload")
        .expect("live result_payload");
    assert_eq!(
        live_result_payload["failure"]["error_code"],
        json!("interrupted_execution")
    );

    let other_live_status: String = sqlx::query_scalar(
        "SELECT status FROM whatsapp_provider_write_commands WHERE command_id = $1",
    )
    .bind(&other_live_command_id)
    .fetch_one(&pool)
    .await
    .expect("other live stale command status");
    assert_eq!(other_live_status, "executing");

    let fixture_status: String = sqlx::query_scalar(
        "SELECT status FROM whatsapp_provider_write_commands WHERE command_id = $1",
    )
    .bind(&fixture_command_id)
    .fetch_one(&pool)
    .await
    .expect("fixture stale command status");
    assert_eq!(fixture_status, "executing");
}

#[tokio::test]
async fn whatsapp_runtime_bridge_status_reconciles_live_publish_status_and_projects_status_feed() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-status-{suffix}");
    let command_id = format!("wa-runtime-bridge-status-command-{suffix}");
    let published_text = format!("Runtime bridge status publish {suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge Status Source",
                "external_account_id": format!("wa-runtime-bridge-status-source-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("live status account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "publish_status",
        &format!("publish-status:{suffix}"),
        "status-feed",
        None,
        json!({ "text": published_text }),
        json!({ "surface": "status_feed" }),
    )
    .await;

    let status_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/statuses",
            json!({
                "account_id": account_id,
                "provider_status_id": format!("provider-status:{command_id}"),
                "sender_id": account_id,
                "sender_display_name": "Макошь Owner",
                "text": published_text,
                "import_batch_id": format!("wa-runtime-bridge-status-{suffix}"),
                "occurred_at": "2026-06-06T12:10:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge status response");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_body = json_body(status_response).await;
    let message_id = status_body["message_id"]
        .as_str()
        .expect("status message id")
        .to_owned();

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge reconciled publish-status command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );
    let mirrored_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge mirrored command status");
    assert_eq!(mirrored_status, "completed");

    let stored_message: (String, String, String) = sqlx::query_as(
        r#"
        SELECT provider_record_id, body_text, conversation_id
        FROM communication_messages
        WHERE message_id = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge stored status message");
    assert_eq!(stored_message.0, format!("provider-status:{command_id}"));
    assert_eq!(stored_message.1, published_text);
    assert_eq!(
        stored_message.2,
        format!("whatsapp_status_feed:{account_id}")
    );

    let status_feed_conversation_id = format!("whatsapp_status_feed:{account_id}");
    let status_feed_row = sqlx::query(
        r#"
        SELECT title, provider_conversation_id, metadata
        FROM communication_conversations
        WHERE conversation_id = $1
        "#,
    )
    .bind(&status_feed_conversation_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge status feed conversation");
    assert_eq!(
        status_feed_row
            .try_get::<String, _>("title")
            .expect("status feed title"),
        "WhatsApp Status"
    );
    assert_eq!(
        status_feed_row
            .try_get::<String, _>("provider_conversation_id")
            .expect("status feed provider conversation id"),
        "status-feed"
    );
    let status_feed_metadata: Value = status_feed_row
        .try_get("metadata")
        .expect("status feed metadata");
    assert_eq!(status_feed_metadata["is_status_feed"], json!(true));
    assert_eq!(status_feed_metadata["chat_kind"], json!("status_feed"));

    let conversations_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations?account_id={account_id}&limit=20"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge conversations response");
    assert_eq!(conversations_response.status(), StatusCode::OK);
    let conversations_body = json_body(conversations_response).await;
    assert!(
        conversations_body["items"]
            .as_array()
            .expect("conversation items")
            .iter()
            .any(|item| {
                item["telegram_chat_id"] == json!(status_feed_conversation_id)
                    && item["provider_chat_id"] == json!("status-feed")
                    && item["chat_kind"] == json!("status_feed")
            }),
        "expected runtime bridge status feed in provider-neutral conversations: {conversations_body}"
    );

    let detail_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations/{status_feed_conversation_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge status conversation detail response");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = json_body(detail_response).await;
    assert_eq!(
        detail_body["item"]["provider_chat_id"],
        json!("status-feed")
    );
    assert_eq!(
        detail_body["item"]["metadata"]["is_status_feed"],
        json!(true)
    );

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("runtime bridge status reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.runtime_bridge_status")
    );

    let accepted_runtime_event_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'command_id' = $2
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("runtime bridge accepted publish-status runtime-event kinds");
    assert_eq!(
        accepted_runtime_event_kinds,
        vec!["status.publish.completed"]
    );

    let projection_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.status.updated'
          AND payload->>'provider_status_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(format!("provider-status:{command_id}"))
    .fetch_one(&pool)
    .await
    .expect("runtime bridge status projection payload");
    assert_eq!(
        projection_payload["source"],
        json!("runtime_bridge_status_ingest")
    );
    assert_eq!(projection_payload["message_id"], json!(message_id));
}

#[tokio::test]
async fn whatsapp_manual_retry_reactivates_join_group_for_live_runtime_claim() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-retry-join-live-{suffix}");
    let provider_chat_id = format!("wa-retry-join-live-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Retry Join Live",
                "external_account_id": format!("wa-retry-join-live-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("live blocked account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let command_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-commands/conversations/join",
            json!({
                "command_id": format!("wa-retry-join-live-command-{suffix}"),
                "idempotency_key": format!("wa-retry-join-live-key-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "invite_link": format!("https://chat.whatsapp.com/retry-join-live-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("join command response");
    assert_eq!(command_response.status(), StatusCode::OK);
    let command_body = json_body(command_response).await;
    let command_id = command_body["command_id"]
        .as_str()
        .expect("command id")
        .to_owned();
    assert_eq!(command_body["command_kind"], json!("join_group"));
    assert_eq!(command_body["status"], json!("blocked"));

    let retry_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/whatsapp/commands/{command_id}/retry"),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("manual retry response");
    assert_eq!(retry_response.status(), StatusCode::OK);
    let retry_body = json_body(retry_response).await;
    assert_eq!(retry_body["command_id"], json!(command_id));
    assert_eq!(retry_body["status"], json!("retrying"));

    let capability_state: String = sqlx::query_scalar(
        "SELECT capability_state FROM whatsapp_provider_write_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("join_group capability_state after manual retry");
    assert_eq!(capability_state, "available");

    let claim_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/commands/claim",
            json!({
                "account_id": account_id,
                "limit": 10
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge claim response");
    assert_eq!(claim_response.status(), StatusCode::OK);
    let claim_body = json_body(claim_response).await;
    assert_eq!(claim_body["items"][0]["command_id"], json!(command_id));
    assert_eq!(claim_body["items"][0]["command_kind"], json!("join_group"));
    assert_eq!(claim_body["items"][0]["status"], json!("executing"));
}

#[tokio::test]
async fn whatsapp_account_list_hides_removed_accounts_unless_requested() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let active_account_id = format!("whatsapp-active-{suffix}");
    let removed_account_id = format!("whatsapp-removed-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let active_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": active_account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Active",
                "external_account_id": format!("wa-active-{suffix}"),
                "device_name": "Active device",
                "local_state_path": format!("docker/data/whatsapp/blocked/active-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("active account response");
    assert_eq!(active_response.status(), StatusCode::OK);

    let removed_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": removed_account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Removed",
                "external_account_id": format!("wa-removed-{suffix}"),
                "device_name": "Removed device",
                "local_state_path": format!("docker/data/whatsapp/blocked/removed-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("removed account create response");
    assert_eq!(removed_response.status(), StatusCode::OK);

    let remove_runtime_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime/remove",
            json!({ "account_id": removed_account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("removed account runtime remove response");
    assert_eq!(remove_runtime_response.status(), StatusCode::OK);

    let active_list_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/integrations/whatsapp/accounts",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("active account list response");
    assert_eq!(active_list_response.status(), StatusCode::OK);
    let active_list_body = json_body(active_list_response).await;
    let active_items = active_list_body["items"]
        .as_array()
        .expect("active account list items");
    assert!(
        active_items
            .iter()
            .any(|item| item["account_id"] == json!(active_account_id))
    );
    assert!(
        !active_items
            .iter()
            .any(|item| item["account_id"] == json!(removed_account_id))
    );

    let all_list_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/integrations/whatsapp/accounts?include_removed=true",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("all account list response");
    assert_eq!(all_list_response.status(), StatusCode::OK);
    let all_list_body = json_body(all_list_response).await;
    let all_items = all_list_body["items"]
        .as_array()
        .expect("all account list items");
    let removed_item = all_items
        .iter()
        .find(|item| item["account_id"] == json!(removed_account_id))
        .expect("removed account in all list");
    assert_eq!(removed_item["lifecycle_state"], json!("removed"));
    assert!(
        all_items
            .iter()
            .any(|item| item["account_id"] == json!(active_account_id))
    );
}

#[tokio::test]
async fn whatsapp_runtime_lifecycle_and_login_surfaces_are_blocked_safe() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Runtime Source",
                "external_account_id": format!("wa-runtime-{suffix}"),
                "device_name": "Макошь Desktop Fixture",
                "local_state_path": format!("docker/data/whatsapp/runtime-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let status_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime status response");
    let status_status = status_response.status();
    let status_body = json_body(status_response).await;
    assert_eq!(
        status_status,
        StatusCode::OK,
        "status response body: {status_body}"
    );
    assert_eq!(status_body["account_id"], json!(account_id));
    assert_eq!(status_body["provider_kind"], json!("whatsapp_web"));
    assert_eq!(
        status_body["provider_shape"],
        json!("whatsapp_web_companion")
    );
    assert_eq!(status_body["runtime_kind"], json!("fixture"));
    assert_eq!(status_body["status"], json!("link_required"));
    assert_eq!(status_body["fixture_runtime"], json!(true));
    assert_eq!(status_body["live_runtime_available"], json!(false));
    assert_eq!(status_body["live_send_available"], json!(false));
    assert_eq!(status_body["qr_pairing_available"], json!(false));
    assert_eq!(status_body["session_restore_available"], json!(false));
    assert_eq!(status_body["session_secret_ref"], Value::Null);
    assert_json_array_contains(
        &status_body["runtime_blockers"],
        "whatsapp_session_link_required",
    );

    let start_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime/start",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    assert_eq!(start_body["status"], json!("link_required"));
    assert_eq!(start_body["runtime_kind"], json!("fixture"));
    assert_eq!(start_body["session_restore_available"], json!(false));

    let health_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/runtime/health?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime health response");
    assert_eq!(health_response.status(), StatusCode::OK);
    let health_body = json_body(health_response).await;
    assert_eq!(health_body["healthy"], json!(false));
    assert_eq!(
        health_body["provider_shape"],
        json!("whatsapp_web_companion")
    );
    assert_json_array_contains(
        &health_body["checks"]["runtime_blockers"],
        "whatsapp_session_link_required",
    );

    let initial_runtime_events =
        whatsapp_event_payloads(&pool, "whatsapp.runtime.status_changed", &account_id).await;
    assert_eq!(initial_runtime_events.len(), 1);
    assert_eq!(initial_runtime_events[0]["status"], json!("link_required"));
    assert_eq!(initial_runtime_events[0]["source"], json!("runtime_start"));
    assert!(initial_runtime_events[0].get("session_material").is_none());
    assert!(
        initial_runtime_events[0]
            .get("session_secret_ref")
            .is_none()
    );

    let qr_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/login/qr/start",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("qr link response");
    assert_eq!(qr_response.status(), StatusCode::OK);
    let qr_body = json_body(qr_response).await;
    assert_eq!(qr_body["status"], json!("qr_pending"));
    assert!(
        qr_body["qr_svg"]
            .as_str()
            .expect("fixture qr svg")
            .contains("<svg")
    );
    assert_ne!(qr_body["expires_at"], Value::Null);
    assert!(
        qr_body["runtime_blockers"]
            .as_array()
            .expect("qr blockers")
            .iter()
            .any(|item| item == "whatsapp_qr_pairing_requires_visible_runtime")
    );
    let qr_status_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime status after qr response");
    assert_eq!(qr_status_response.status(), StatusCode::OK);
    let qr_status_body = json_body(qr_status_response).await;
    assert_eq!(qr_status_body["status"], json!("qr_pending"));
    assert_eq!(qr_status_body["qr_pairing_available"], json!(true));
    assert_eq!(qr_status_body["pair_code_available"], json!(false));
    assert_json_array_contains(
        &qr_status_body["runtime_blockers"],
        "whatsapp_qr_pairing_requires_visible_runtime",
    );

    let pair_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/login/pair-code/start",
            json!({ "account_id": account_id, "phone_number": "+15551234567" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("pair code response");
    assert_eq!(pair_response.status(), StatusCode::OK);
    let pair_body = json_body(pair_response).await;
    assert_eq!(pair_body["status"], json!("pair_code_pending"));
    assert_eq!(pair_body["phone_number"], json!("+15551234567"));
    let pair_code = pair_body["pair_code"].as_str().expect("fixture pair code");
    assert_eq!(pair_code.len(), 9);
    assert!(pair_code.contains('-'));
    assert_ne!(pair_body["expires_at"], Value::Null);
    let pair_status_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime status after pair code response");
    assert_eq!(pair_status_response.status(), StatusCode::OK);
    let pair_status_body = json_body(pair_status_response).await;
    assert_eq!(pair_status_body["status"], json!("pair_code_pending"));
    assert_eq!(pair_status_body["qr_pairing_available"], json!(false));
    assert_eq!(pair_status_body["pair_code_available"], json!(true));
    assert_json_array_contains(
        &pair_status_body["runtime_blockers"],
        "whatsapp_qr_pairing_requires_visible_runtime",
    );
    let stored_session_link_state: String =
        sqlx::query_scalar("SELECT link_state FROM whatsapp_web_sessions WHERE account_id = $1")
            .bind(&account_id)
            .fetch_one(&pool)
            .await
            .expect("stored pair-code session link state");
    assert_eq!(stored_session_link_state, "pair_code_pending");

    let login_runtime_events =
        whatsapp_event_payloads(&pool, "whatsapp.runtime.status_changed", &account_id).await;
    assert_eq!(login_runtime_events.len(), 3);
    assert_eq!(login_runtime_events[1]["status"], json!("qr_pending"));
    assert_eq!(login_runtime_events[1]["source"], json!("login_qr_start"));
    assert_eq!(
        login_runtime_events[2]["status"],
        json!("pair_code_pending")
    );
    assert_eq!(
        login_runtime_events[2]["source"],
        json!("login_pair_code_start")
    );
    let login_session_events =
        whatsapp_event_payloads(&pool, "whatsapp.session.link_state_changed", &account_id).await;
    assert_eq!(login_session_events.len(), 2);
    assert_eq!(login_session_events[0]["link_state"], json!("qr_pending"));
    assert_eq!(login_session_events[0]["source"], json!("login_qr_start"));
    assert_eq!(
        login_session_events[1]["link_state"],
        json!("pair_code_pending")
    );
    assert_eq!(
        login_session_events[1]["source"],
        json!("login_pair_code_start")
    );

    let accepted_runtime_event_sources: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->'metadata'->>'source'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .expect("accepted whatsapp runtime-event sources");
    assert_eq!(
        accepted_runtime_event_sources,
        vec!["runtime_start", "login_qr_start", "login_pair_code_start"]
    );

    let provider_chat_id = format!("wa-command-chat-{suffix}");
    let send_command_id = format!("wa-send-{suffix}");
    let send_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-commands/messages/send",
            json!({
                "command_id": send_command_id,
                "idempotency_key": format!("send-text:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "text": "Do not leak this WhatsApp command body"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("send command response");
    assert_eq!(send_response.status(), StatusCode::OK);
    let send_body = json_body(send_response).await;
    assert_eq!(send_body["command_kind"], json!("send_text"));
    assert_eq!(send_body["status"], json!("blocked"));
    assert_eq!(send_body["durable_status"], json!("cancelled"));
    assert_eq!(send_body["delivery_state"], json!("not_attempted"));
    assert_eq!(send_body["session_restore_available"], json!(false));
    assert!(send_body["rendered_preview_hash"].as_str().is_some());
    assert_eq!(send_body.get("text"), None);
    assert_json_array_contains(
        &send_body["runtime_blockers"],
        "fixture_runtime_does_not_execute_provider_commands",
    );

    let reply_command_id = format!("wa-reply-{suffix}");
    let reply_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/whatsapp/provider-commands/messages/message-{suffix}/reply"
            ),
            json!({
                "command_id": reply_command_id,
                "idempotency_key": format!("reply:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "reply_to_provider_message_id": format!("wa-source-message-{suffix}"),
                "text": "Reply body must not be in command event"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reply command response");
    assert_eq!(reply_response.status(), StatusCode::OK);
    assert_eq!(
        json_body(reply_response).await["command_kind"],
        json!("reply")
    );

    let forward_command_id = format!("wa-forward-{suffix}");
    let forward_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/whatsapp/provider-commands/messages/message-{suffix}/forward"
            ),
            json!({
                "command_id": forward_command_id,
                "idempotency_key": format!("forward:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "from_provider_chat_id": format!("wa-source-chat-{suffix}"),
                "from_provider_message_id": format!("wa-source-message-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("forward command response");
    assert_eq!(forward_response.status(), StatusCode::OK);
    assert_eq!(
        json_body(forward_response).await["command_kind"],
        json!("forward")
    );

    let edit_command_id = format!("wa-edit-{suffix}");
    let edit_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/whatsapp/provider-commands/messages/message-{suffix}/edit"
            ),
            json!({
                "command_id": edit_command_id,
                "idempotency_key": format!("edit:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("wa-source-message-{suffix}"),
                "text": "Edited body must not be in command event"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("edit command response");
    assert_eq!(edit_response.status(), StatusCode::OK);
    assert_eq!(
        json_body(edit_response).await["command_kind"],
        json!("edit")
    );

    let delete_command_id = format!("wa-delete-{suffix}");
    let delete_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/whatsapp/provider-commands/messages/message-{suffix}/delete"
            ),
            json!({
                "command_id": delete_command_id,
                "idempotency_key": format!("delete:{suffix}"),
                "confirmation_decision": "pending",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("wa-source-message-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete command response");
    assert_eq!(delete_response.status(), StatusCode::OK);
    assert_eq!(
        json_body(delete_response).await["command_kind"],
        json!("delete")
    );

    let react_command_id = format!("wa-react-{suffix}");
    let react_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/whatsapp/provider-commands/messages/message-{suffix}/reactions"
            ),
            json!({
                "command_id": react_command_id,
                "idempotency_key": format!("react:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("wa-source-message-{suffix}"),
                "reaction_emoji": "👍"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("react command response");
    assert_eq!(react_response.status(), StatusCode::OK);
    assert_eq!(
        json_body(react_response).await["command_kind"],
        json!("react")
    );

    let unreact_command_id = format!("wa-unreact-{suffix}");
    let unreact_response = app
        .clone()
        .oneshot(json_delete_request_with_actor(
            &format!(
                "/api/v1/integrations/whatsapp/provider-commands/messages/message-{suffix}/reactions"
            ),
            json!({
                "command_id": unreact_command_id,
                "idempotency_key": format!("unreact:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("wa-source-message-{suffix}"),
                "reaction_emoji": "👍"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("unreact command response");
    assert_eq!(unreact_response.status(), StatusCode::OK);
    assert_eq!(
        json_body(unreact_response).await["command_kind"],
        json!("unreact")
    );

    let import_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/communications/attachments/import",
            json!({
                "account_id": account_id,
                "channel_kind": "whatsapp",
                "filename": "whatsapp-upload-note.txt",
                "content_type": "text/plain",
                "content_base64": "V2hhdHNBcHAgbWVkaWEgdXBsb2FkIGZpeHR1cmU=",
                "metadata": {"source": "whatsapp_media_upload_test"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp attachment import response");
    assert_eq!(import_response.status(), StatusCode::OK);
    let imported_attachment = json_body(import_response).await;
    let unscanned_command_id = format!("wa-media-upload-unscanned-{suffix}");
    let unscanned_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-media/upload",
            json!({
                "command_id": unscanned_command_id.clone(),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "attachment_id": imported_attachment["attachment_id"],
                "media_type": "document"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp unscanned media upload response");
    assert_eq!(unscanned_response.status(), StatusCode::BAD_REQUEST);
    let unscanned_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM whatsapp_provider_write_commands WHERE command_id = $1",
    )
    .bind(&unscanned_command_id)
    .fetch_one(&pool)
    .await
    .expect("unscanned command count");
    assert_eq!(unscanned_count, 0);
    mark_attachment_clean(
        &pool,
        imported_attachment["attachment_id"]
            .as_str()
            .expect("attachment id"),
    )
    .await;
    let upload_command_id = format!("wa-media-upload-{suffix}");
    let upload_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-media/upload",
            json!({
                "command_id": upload_command_id,
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "attachment_id": imported_attachment["attachment_id"],
                "media_type": "document",
                "caption": "caption must not be in events",
                "filename": "whatsapp-upload-note.txt"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp media upload response");
    assert_eq!(upload_response.status(), StatusCode::OK);
    let upload_body = json_body(upload_response).await;
    assert_eq!(upload_body["command_kind"], json!("send_media"));
    assert_eq!(upload_body["status"], json!("blocked"));

    let download_command_id = format!("wa-media-download-{suffix}");
    let download_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-media/download",
            json!({
                "command_id": download_command_id,
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("wa-source-message-{suffix}"),
                "provider_attachment_id": format!("wa-attachment-{suffix}"),
                "filename": "fixture.pdf",
                "content_type": "application/pdf"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp media download response");
    assert_eq!(download_response.status(), StatusCode::OK);
    let download_body = json_body(download_response).await;
    assert_eq!(download_body["command_kind"], json!("download_media"));
    assert_eq!(download_body["status"], json!("blocked"));

    let voice_note_command_id = format!("wa-voice-note-{suffix}");
    let voice_note_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-commands/messages/voice-note",
            json!({
                "command_id": voice_note_command_id,
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "attachment_id": imported_attachment["attachment_id"],
                "media_type": "voice_note",
                "filename": "voice-note.ogg"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp voice note response");
    assert_eq!(voice_note_response.status(), StatusCode::OK);
    let voice_note_body = json_body(voice_note_response).await;
    assert_eq!(voice_note_body["command_kind"], json!("send_voice_note"));
    assert_eq!(voice_note_body["status"], json!("blocked"));

    for (suffix_name, path, expected_kind, body) in [
        (
            "read",
            format!(
                "/api/v1/integrations/whatsapp/provider-commands/conversations/{provider_chat_id}/read"
            ),
            "mark_read",
            json!({
                "command_id": format!("wa-read-{suffix}"),
                "idempotency_key": format!("read:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "unread",
            format!(
                "/api/v1/integrations/whatsapp/provider-commands/conversations/{provider_chat_id}/unread"
            ),
            "mark_unread",
            json!({
                "command_id": format!("wa-unread-{suffix}"),
                "idempotency_key": format!("unread:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "archive",
            format!(
                "/api/v1/integrations/whatsapp/provider-commands/conversations/{provider_chat_id}/archive"
            ),
            "archive",
            json!({
                "command_id": format!("wa-archive-{suffix}"),
                "idempotency_key": format!("archive:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "unarchive",
            format!(
                "/api/v1/integrations/whatsapp/provider-commands/conversations/{provider_chat_id}/unarchive"
            ),
            "unarchive",
            json!({
                "command_id": format!("wa-unarchive-{suffix}"),
                "idempotency_key": format!("unarchive:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "mute",
            format!(
                "/api/v1/integrations/whatsapp/provider-commands/conversations/{provider_chat_id}/mute"
            ),
            "mute",
            json!({
                "command_id": format!("wa-mute-{suffix}"),
                "idempotency_key": format!("mute:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "unmute",
            format!(
                "/api/v1/integrations/whatsapp/provider-commands/conversations/{provider_chat_id}/unmute"
            ),
            "unmute",
            json!({
                "command_id": format!("wa-unmute-{suffix}"),
                "idempotency_key": format!("unmute:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "pin",
            format!(
                "/api/v1/integrations/whatsapp/provider-commands/conversations/{provider_chat_id}/pin"
            ),
            "pin",
            json!({
                "command_id": format!("wa-pin-{suffix}"),
                "idempotency_key": format!("pin:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "unpin",
            format!(
                "/api/v1/integrations/whatsapp/provider-commands/conversations/{provider_chat_id}/unpin"
            ),
            "unpin",
            json!({
                "command_id": format!("wa-unpin-{suffix}"),
                "idempotency_key": format!("unpin:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "join",
            "/api/v1/integrations/whatsapp/provider-commands/conversations/join".to_owned(),
            "join_group",
            json!({
                "command_id": format!("wa-join-{suffix}"),
                "idempotency_key": format!("join:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "invite_link": format!("https://chat.whatsapp.com/invite-{suffix}")
            }),
        ),
        (
            "leave",
            format!(
                "/api/v1/integrations/whatsapp/provider-commands/conversations/{provider_chat_id}/leave"
            ),
            "leave_group",
            json!({
                "command_id": format!("wa-leave-{suffix}"),
                "idempotency_key": format!("leave:{suffix}"),
                "confirmation_decision": "pending",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
    ] {
        let response = app
            .clone()
            .oneshot(json_post_request_with_actor(&path, body, LOCAL_API_TOKEN))
            .await
            .unwrap_or_else(|_| panic!("WhatsApp {suffix_name} response"));
        assert_eq!(response.status(), StatusCode::OK, "{suffix_name}");
        let payload = json_body(response).await;
        assert_eq!(
            payload["command_kind"],
            json!(expected_kind),
            "{suffix_name}"
        );
        assert_eq!(payload["status"], json!("blocked"), "{suffix_name}");
    }

    let publish_status_command_id = format!("wa-publish-status-{suffix}");
    let publish_status_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-commands/statuses/publish",
            json!({
                "command_id": publish_status_command_id,
                "idempotency_key": format!("publish-status:{suffix}"),
                "account_id": account_id,
                "text": format!("Status update {suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp publish status response");
    assert_eq!(publish_status_response.status(), StatusCode::OK);
    let publish_status_body = json_body(publish_status_response).await;
    assert_eq!(publish_status_body["command_kind"], json!("publish_status"));
    assert_eq!(publish_status_body["status"], json!("blocked"));

    let command_event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'whatsapp.command.status_changed' AND payload->>'provider_chat_id' = $1",
    )
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp command event count");
    assert_eq!(command_event_count, 20);

    let upload_lifecycle_events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE event_type IN (
            'whatsapp.media.upload.requested',
            'whatsapp.media.upload.failed'
        )
          AND payload->>'command_id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(&upload_command_id)
    .fetch_all(&pool)
    .await
    .expect("WhatsApp media upload lifecycle events");
    assert_eq!(upload_lifecycle_events.len(), 2);
    assert_eq!(
        upload_lifecycle_events[0].0,
        "whatsapp.media.upload.requested"
    );
    assert_eq!(upload_lifecycle_events[0].1["status"], json!("requested"));
    assert_eq!(upload_lifecycle_events[0].1["caption"], Value::Null);
    assert_eq!(upload_lifecycle_events[1].0, "whatsapp.media.upload.failed");
    assert_eq!(upload_lifecycle_events[1].1["status"], json!("failed"));
    assert_eq!(
        upload_lifecycle_events[1].1["error"],
        json!("fixture_runtime_does_not_execute_provider_commands")
    );

    let download_lifecycle_events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE event_type IN (
            'whatsapp.media.download.requested',
            'whatsapp.media.download.failed'
        )
          AND payload->>'command_id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(&download_command_id)
    .fetch_all(&pool)
    .await
    .expect("WhatsApp media download lifecycle events");
    assert_eq!(download_lifecycle_events.len(), 2);
    assert_eq!(
        download_lifecycle_events[0].0,
        "whatsapp.media.download.requested"
    );
    assert_eq!(download_lifecycle_events[0].1["status"], json!("requested"));
    assert_eq!(
        download_lifecycle_events[1].0,
        "whatsapp.media.download.failed"
    );
    assert_eq!(download_lifecycle_events[1].1["status"], json!("failed"));

    let blocked_media_runtime_event_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'command_id' IN ($2, $3)
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .bind(&upload_command_id)
    .bind(&download_command_id)
    .fetch_all(&pool)
    .await
    .expect("blocked whatsapp media runtime-event kinds");
    assert_eq!(
        blocked_media_runtime_event_kinds,
        vec![
            "media.upload.requested",
            "media.upload.failed",
            "media.download.requested",
            "media.download.failed",
        ]
    );
    let blocked_status_runtime_event_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'command_id' = $2
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .bind(&publish_status_command_id)
    .fetch_all(&pool)
    .await
    .expect("blocked whatsapp publish-status runtime-event kinds");
    assert_eq!(
        blocked_status_runtime_event_kinds,
        vec!["status.publish.requested", "status.publish.failed"]
    );

    let send_event_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.command.status_changed' AND payload->>'command_id' = $1",
    )
    .bind(&send_command_id)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp send command event payload");
    assert_eq!(send_event_payload["command_kind"], json!("send_text"));
    assert_eq!(send_event_payload["status"], json!("blocked"));
    assert_eq!(send_event_payload["durable_status"], json!("cancelled"));
    assert_eq!(send_event_payload.get("text"), None);
    assert_eq!(send_event_payload.get("body"), None);
    assert!(
        send_event_payload["rendered_preview_hash"]
            .as_str()
            .is_some()
    );

    let send_command_row = sqlx::query(
        r#"
        SELECT command_kind, capability_state, status, payload, audit_metadata
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&send_command_id)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp send command row");
    assert_eq!(
        send_command_row
            .try_get::<String, _>("command_kind")
            .expect("command kind"),
        "send_text"
    );
    assert_eq!(
        send_command_row
            .try_get::<String, _>("capability_state")
            .expect("capability state"),
        "blocked"
    );
    assert_eq!(
        send_command_row
            .try_get::<String, _>("status")
            .expect("status"),
        "cancelled"
    );
    let send_payload = send_command_row
        .try_get::<Value, _>("payload")
        .expect("send payload");
    assert_eq!(
        send_payload["text"],
        json!("Do not leak this WhatsApp command body")
    );
    let send_audit_metadata = send_command_row
        .try_get::<Value, _>("audit_metadata")
        .expect("send audit metadata");
    assert_eq!(send_audit_metadata.get("text"), None);
    assert!(
        send_audit_metadata["rendered_preview_hash"]
            .as_str()
            .is_some()
    );

    let duplicate_send_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-commands/messages/send",
            json!({
                "command_id": format!("wa-send-duplicate-{suffix}"),
                "idempotency_key": format!("send-text:{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "text": "A duplicate body must not create another command row"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("duplicate send command response");
    assert_eq!(duplicate_send_response.status(), StatusCode::OK);
    let duplicate_send_body = json_body(duplicate_send_response).await;
    assert_eq!(duplicate_send_body["command_id"], json!(send_command_id));

    let send_command_row_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM whatsapp_provider_write_commands WHERE account_id = $1 AND idempotency_key = $2",
    )
    .bind(&account_id)
    .bind(format!("send-text:{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("WhatsApp send command idempotency row count");
    assert_eq!(send_command_row_count, 1);

    let canonical_send_command_row = sqlx::query(
        r#"
        SELECT channel_kind, command_kind, status, payload, audit_metadata
        FROM communication_provider_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&send_command_id)
    .fetch_one(&pool)
    .await
    .expect("canonical WhatsApp provider command row");
    assert_eq!(
        canonical_send_command_row
            .try_get::<String, _>("channel_kind")
            .expect("canonical channel kind"),
        "whatsapp"
    );
    assert_eq!(
        canonical_send_command_row
            .try_get::<String, _>("command_kind")
            .expect("canonical command kind"),
        "send_text"
    );
    assert_eq!(
        canonical_send_command_row
            .try_get::<String, _>("status")
            .expect("canonical status"),
        "cancelled"
    );
    let canonical_send_payload = canonical_send_command_row
        .try_get::<Value, _>("payload")
        .expect("canonical send payload");
    assert_eq!(
        canonical_send_payload["text"],
        json!("Do not leak this WhatsApp command body")
    );
    let canonical_audit_metadata = canonical_send_command_row
        .try_get::<Value, _>("audit_metadata")
        .expect("canonical audit metadata");
    assert_eq!(canonical_audit_metadata.get("text"), None);
    assert_eq!(canonical_audit_metadata.get("body"), None);
    assert_eq!(canonical_audit_metadata.get("session_secret_ref"), None);

    let commands_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/integrations/whatsapp/commands?account_id={account_id}&provider_chat_id={provider_chat_id}&command_kinds=send_text&limit=20"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp command list response");
    assert_eq!(commands_response.status(), StatusCode::OK);
    let commands_body = json_body(commands_response).await;
    assert_eq!(commands_body["items"].as_array().expect("items").len(), 1);
    assert_eq!(
        commands_body["items"][0]["command_id"],
        json!(send_command_id)
    );
    assert_eq!(commands_body["items"][0].get("payload"), None);

    let retry_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/whatsapp/commands/{send_command_id}/retry"),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp command retry response");
    assert_eq!(retry_response.status(), StatusCode::OK);
    let retry_body = json_body(retry_response).await;
    assert_eq!(retry_body["command_id"], json!(send_command_id));
    assert_eq!(retry_body["status"], json!("retrying"));
    assert_eq!(retry_body.get("payload"), None);
    assert!(retry_body["result_payload"]["manual_retry_at"].is_string());

    let canonical_retry_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&send_command_id)
    .fetch_one(&pool)
    .await
    .expect("canonical retry status");
    assert_eq!(canonical_retry_status, "retrying");

    let retry_event_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.command.status_changed' AND payload->>'command_id' = $1 AND payload->>'source' = 'manual_retry'",
    )
    .bind(&send_command_id)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp retry command event payload");
    assert_eq!(retry_event_payload["status"], json!("retrying"));
    assert_eq!(retry_event_payload.get("payload"), None);
    assert_eq!(retry_event_payload.get("text"), None);
    assert_eq!(retry_event_payload.get("body"), None);

    let dead_letter_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/whatsapp/commands/{send_command_id}/dead-letter"),
            json!({"reason": "manual operator review requested"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp command dead-letter response");
    assert_eq!(dead_letter_response.status(), StatusCode::OK);
    let dead_letter_body = json_body(dead_letter_response).await;
    assert_eq!(dead_letter_body["command_id"], json!(send_command_id));
    assert_eq!(dead_letter_body["status"], json!("dead_letter"));
    assert_eq!(
        dead_letter_body["last_error"],
        json!("manual operator review requested")
    );
    assert_eq!(dead_letter_body.get("payload"), None);

    let canonical_dead_letter_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&send_command_id)
    .fetch_one(&pool)
    .await
    .expect("canonical dead-letter status");
    assert_eq!(canonical_dead_letter_status, "dead_letter");

    let stop_response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime/stop",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime stop response");
    assert_eq!(stop_response.status(), StatusCode::OK);
    let stop_body = json_body(stop_response).await;
    assert_eq!(stop_body["status"], json!("linked"));
}

#[tokio::test]
async fn whatsapp_api_exercises_web_fixture_foundation() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-web-api-{suffix}");
    let chat_id = format!("wa-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let capabilities_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/integrations/whatsapp/capabilities",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("capabilities response");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = json_body(capabilities_response).await;
    assert_eq!(capabilities_body["version"], json!("2.0"));
    assert_eq!(capabilities_body["runtime_mode"], json!("fixture"));
    assert_json_array_contains(
        &capabilities_body["planned_features"],
        "live_runtime_execution",
    );
    assert_capability_status(&capabilities_body, "runtime.fixture", "available", true);
    assert_capability_status(
        &capabilities_body,
        "messages.read_projection",
        "available",
        true,
    );
    assert_capability_status(&capabilities_body, "messages.send_text", "degraded", true);
    assert_capability_status(&capabilities_body, "auth.qr_link_start", "degraded", true);
    assert!(
        capabilities_body["capabilities"]
            .as_array()
            .expect("capability rows")
            .iter()
            .all(|capability| capability["capability"] != json!("business.templates")),
        "hidden-WebView WhatsApp must not advertise a Business Cloud capability"
    );
    assert!(
        capabilities_body["unsupported_features"]
            .as_array()
            .expect("unsupported features")
            .iter()
            .any(|feature| feature == "hidden_web_scraping")
    );
    assert!(
        capabilities_body["provider_shapes"]
            .as_array()
            .expect("provider shapes")
            .iter()
            .any(|shape| {
                shape["provider_shape"] == json!("whatsapp_web_companion")
                    && shape["status"] == json!("available")
            })
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Web",
                "external_account_id": format!("wa-device-{suffix}"),
                "device_name": "Макошь Desktop Fixture",
                "local_state_path": format!("docker/data/whatsapp/{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);
    let account_body = json_body(account_response).await;
    assert_eq!(account_body["provider_kind"], json!("whatsapp_web"));
    assert_eq!(account_body["runtime"], json!("fixture"));
    assert_eq!(account_body["session"]["link_state"], json!("fixture"));
    let account_capabilities_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/accounts/{account_id}/capabilities"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account capabilities response");
    assert_eq!(account_capabilities_response.status(), StatusCode::OK);
    let account_capabilities_body = json_body(account_capabilities_response).await;
    assert_eq!(
        account_capabilities_body["account_scope"]["account_id"],
        json!(account_id)
    );
    assert_eq!(
        account_capabilities_body["account_scope"]["provider_shape"],
        json!("whatsapp_web_companion")
    );
    assert_capability_status(
        &account_capabilities_body,
        "messages.send_text",
        "blocked",
        true,
    );
    assert_capability_status(
        &account_capabilities_body,
        "sessions.restore",
        "blocked",
        true,
    );
    let session_id = account_body["session"]["session_id"]
        .as_str()
        .expect("session id")
        .to_owned();
    let provider_message_id = format!("wa-message-{suffix}");
    let rich_message_metadata = json!({
        "mentions": [
            {
                "provider_identity_id": format!("wa:+3412345{suffix}"),
                "display_name": "Owner Member",
                "username": "@owner_member",
                "offset": 7,
                "length": 12
            }
        ],
        "links": [
            {
                "url": "https://example.com/makosh-whatsapp",
                "title": "Макошь WhatsApp",
                "description": "Link preview text"
            }
        ],
        "poll": {
            "question": "Ship WhatsApp runtime?",
            "options": [
                {"text": "Yes", "voter_count": 3},
                {"text": "Later", "voter_count": 1}
            ],
            "total_voter_count": 4,
            "is_closed": false
        },
        "location": {
            "label": "Madrid Office",
            "latitude": 40.4168,
            "longitude": -3.7038
        },
        "contact_card": {
            "display_name": "Ops Desk",
            "phones": ["+34910000000"]
        },
        "sticker": {
            "provider_sticker_id": format!("wa-sticker-{suffix}"),
            "emoji": "📌"
        },
        "system_message": {
            "event_type": "group_subject_changed",
            "actor_display_name": "Owner Member"
        },
        "ephemeral": {
            "disappearing": true,
            "expires_in_seconds": 86400
        },
        "view_once": {
            "is_view_once": true,
            "media_kind": "image"
        },
        "join_leave": {
            "action": "join",
            "source": "invite_link",
            "provider_member_id": format!("member-owner-{suffix}")
        },
        "debug_secret_material": {
            "session_key": format!("wa-session-secret-{suffix}"),
            "access_token": format!("wa-access-token-{suffix}"),
            "nested": {
                "refresh_token": format!("wa-refresh-token-{suffix}")
            }
        }
    });

    let dialog_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/dialogs",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "chat_title": "WhatsApp Planning",
                "chat_kind": "group",
                "is_archived": false,
                "is_pinned": true,
                "is_muted": true,
                "is_unread": true,
                "unread_count": 7,
                "participant_count": 3,
                "community_parent_chat_id": format!("community-root-{suffix}"),
                "community_parent_title": "Макошь Community",
                "invite_link": format!("https://chat.whatsapp.com/community-{suffix}"),
                "is_community_root": false,
                "is_broadcast": false,
                "is_newsletter": true,
                "avatar_metadata": {
                    "profile_picture_url": "https://example.com/whatsapp-planning.jpg",
                    "profile_picture_sha256": "avatar-sha256"
                },
                "provider_labels": ["team", "urgent"],
                "import_batch_id": format!("whatsapp-dialog-{suffix}"),
                "observed_at": "2026-06-06T12:59:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dialog response");
    let dialog_status = dialog_response.status();
    let dialog_body = json_body(dialog_response).await;
    assert_eq!(
        dialog_status,
        StatusCode::OK,
        "dialog response body: {dialog_body}"
    );
    assert!(
        dialog_body["conversation_id"]
            .as_str()
            .expect("dialog conversation id")
            .starts_with("conversation:v5:whatsapp_web:")
    );
    assert!(
        dialog_body["channel_id"]
            .as_str()
            .expect("dialog channel id")
            .starts_with("channel:v5:whatsapp_web:")
    );

    let participant_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/participants",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "chat_title": "WhatsApp Planning",
                "chat_kind": "group",
                "provider_member_id": format!("member-owner-{suffix}"),
                "provider_identity_id": format!("wa:+3412345{suffix}"),
                "identity_kind": "whatsapp_phone",
                "display_name": "Owner Member",
                "push_name": "Owner Push",
                "address": format!("+3412345{suffix}"),
                "business_profile": {
                    "category": "consulting",
                    "description": "Макошь project coordination"
                },
                "profile_photo_ref": {
                    "provider_file_id": format!("avatar-{suffix}"),
                    "sha256": "photo-sha256"
                },
                "role": "owner",
                "status": "member",
                "is_self": false,
                "is_admin": true,
                "is_owner": true,
                "import_batch_id": format!("whatsapp-participant-owner-{suffix}"),
                "observed_at": "2026-06-06T12:59:30Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("participant response");
    let participant_status = participant_response.status();
    let participant_body = json_body(participant_response).await;
    assert_eq!(
        participant_status,
        StatusCode::OK,
        "participant response body: {participant_body}"
    );
    assert_eq!(
        participant_body["conversation_id"],
        dialog_body["conversation_id"]
    );
    assert!(
        participant_body["participant_id"]
            .as_str()
            .expect("participant id")
            .starts_with("participant:v5:whatsapp_web:")
    );
    assert!(
        participant_body["identity_id"]
            .as_str()
            .expect("identity id")
            .starts_with("identity:v5:whatsapp_web:")
    );

    let participant_role_change_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/participants",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "chat_title": "WhatsApp Planning",
                "chat_kind": "group",
                "provider_member_id": format!("member-owner-{suffix}"),
                "provider_identity_id": format!("wa:+3412345{suffix}"),
                "identity_kind": "whatsapp_phone",
                "display_name": "Owner Member",
                "push_name": "Owner Push",
                "address": format!("+3412345{suffix}"),
                "business_profile": {
                    "category": "consulting",
                    "description": "Макошь project coordination"
                },
                "profile_photo_ref": {
                    "provider_file_id": format!("avatar-{suffix}"),
                    "sha256": "photo-sha256"
                },
                "role": "member",
                "status": "left",
                "is_self": false,
                "is_admin": false,
                "is_owner": false,
                "import_batch_id": format!("whatsapp-participant-owner-role-change-{suffix}"),
                "observed_at": "2026-06-06T13:00:15Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("participant role change response");
    assert_eq!(participant_role_change_response.status(), StatusCode::OK);
    let participant_role_change_body = json_body(participant_role_change_response).await;
    assert_eq!(
        participant_role_change_body["participant_id"],
        participant_body["participant_id"]
    );
    assert_eq!(
        participant_role_change_body["identity_id"],
        participant_body["identity_id"]
    );
    assert_eq!(
        participant_role_change_body["previous_role"],
        json!("owner")
    );
    assert_eq!(
        participant_role_change_body["current_role"],
        json!("member")
    );
    assert_eq!(
        participant_role_change_body["previous_status"],
        json!("member")
    );
    assert_eq!(
        participant_role_change_body["current_status"],
        json!("left")
    );
    assert_eq!(participant_role_change_body["role_changed"], json!(true));
    assert_eq!(
        participant_role_change_body["membership_changed"],
        json!(true)
    );

    let presence_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/presence",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_identity_id": format!("wa:+3412345{suffix}"),
                "identity_kind": "whatsapp_phone",
                "display_name": "Owner Member Renamed",
                "presence_state": "typing",
                "last_seen_at": "2026-06-06T12:58:00Z",
                "import_batch_id": format!("whatsapp-presence-{suffix}"),
                "observed_at": "2026-06-06T12:59:45Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("presence response");
    let presence_status = presence_response.status();
    let presence_body = json_body(presence_response).await;
    assert_eq!(
        presence_status,
        StatusCode::OK,
        "presence response body: {presence_body}"
    );
    assert_eq!(
        presence_body["identity_id"],
        participant_body["identity_id"]
    );
    let identity_metadata: Value =
        sqlx::query_scalar("SELECT metadata FROM communication_identities WHERE identity_id = $1")
            .bind(
                participant_body["identity_id"]
                    .as_str()
                    .expect("participant identity id"),
            )
            .fetch_one(&pool)
            .await
            .expect("whatsapp identity metadata after presence");
    assert_eq!(identity_metadata["push_name"], json!("Owner Push"));
    assert_eq!(
        identity_metadata["business_profile"]["category"],
        json!("consulting")
    );
    assert_eq!(
        identity_metadata["profile_photo_ref"]["provider_file_id"],
        json!(format!("avatar-{suffix}"))
    );
    assert_eq!(identity_metadata["presence_state"], json!("typing"));
    assert_eq!(
        identity_metadata["presence_provider_chat_id"],
        json!(chat_id)
    );
    assert_eq!(
        identity_metadata["presence_observed_at"],
        json!("2026-06-06T12:59:45Z")
    );
    assert_eq!(
        identity_metadata["last_seen_at"],
        json!("2026-06-06T12:58:00Z")
    );

    let call_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/calls",
            json!({
                "account_id": account_id,
                "provider_call_id": format!("wa-call-{suffix}"),
                "provider_chat_id": chat_id,
                "direction": "incoming",
                "call_state": "missed",
                "started_at": "2026-06-06T12:57:00Z",
                "ended_at": "2026-06-06T12:57:12Z",
                "metadata": {
                    "call_kind": "voice",
                    "provider_participant_id": format!("wa:+3412345{suffix}")
                },
                "import_batch_id": format!("whatsapp-call-{suffix}"),
                "observed_at": "2026-06-06T12:59:50Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("call response");
    let call_status = call_response.status();
    let call_body = json_body(call_response).await;
    assert_eq!(
        call_status,
        StatusCode::OK,
        "call response body: {call_body}"
    );
    assert!(
        call_body["call_id"]
            .as_str()
            .expect("call id")
            .starts_with("call:v5:whatsapp_web:")
    );
    let calls_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/calls?account_id={account_id}&limit=20"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("calls response");
    assert_eq!(calls_response.status(), StatusCode::OK);
    let calls_body = json_body(calls_response).await;
    assert!(
        calls_body["items"]
            .as_array()
            .expect("call items")
            .iter()
            .any(|item| {
                item["call_id"] == call_body["call_id"]
                    && item["provider_call_id"] == json!(format!("wa-call-{suffix}"))
                    && item["provider_chat_id"] == json!(chat_id)
                    && item["direction"] == json!("incoming")
                    && item["call_state"] == json!("missed")
            }),
        "expected WhatsApp call metadata in calls API list"
    );

    let conversations_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations?account_id={account_id}&limit=20"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp conversations response");
    assert_eq!(conversations_response.status(), StatusCode::OK);
    let conversations_body = json_body(conversations_response).await;
    let conversation_items = conversations_body["items"]
        .as_array()
        .expect("conversation items");
    assert!(
        conversation_items.iter().any(|item| {
            item["telegram_chat_id"] == dialog_body["conversation_id"]
                && item["provider_chat_id"] == json!(chat_id)
                && item["title"] == json!("WhatsApp Planning")
                && item["chat_kind"] == json!("group")
                && item["metadata"]["is_muted"] == json!(true)
                && item["metadata"]["is_unread"] == json!(true)
                && item["metadata"]["unread_count"] == json!(7)
                && item["metadata"]["participant_count"] == json!(3)
        }),
        "expected WhatsApp canonical conversation in provider-neutral conversations list"
    );

    let conversation_detail_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/conversations/{}",
                dialog_body["conversation_id"]
                    .as_str()
                    .expect("dialog conversation id")
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp conversation detail response");
    assert_eq!(conversation_detail_response.status(), StatusCode::OK);
    let conversation_detail_body = json_body(conversation_detail_response).await;
    assert_eq!(
        conversation_detail_body["item"]["telegram_chat_id"],
        dialog_body["conversation_id"]
    );
    assert_eq!(
        conversation_detail_body["item"]["provider_chat_id"],
        json!(chat_id)
    );
    assert_eq!(
        conversation_detail_body["item"]["metadata"]["unread_count"],
        json!(7)
    );
    assert_eq!(
        conversation_detail_body["item"]["metadata"]["community_parent_chat_id"],
        json!(format!("community-root-{suffix}"))
    );
    assert_eq!(
        conversation_detail_body["item"]["metadata"]["invite_link"],
        json!(format!("https://chat.whatsapp.com/community-{suffix}"))
    );

    let conversation_members_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/conversations/{}/members?limit=20",
                dialog_body["conversation_id"]
                    .as_str()
                    .expect("dialog conversation id")
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp conversation members response");
    assert_eq!(conversation_members_response.status(), StatusCode::OK);
    let conversation_members_body = json_body(conversation_members_response).await;
    let member_items = conversation_members_body["items"]
        .as_array()
        .expect("member items");
    assert!(
        member_items.iter().any(|item| {
            item["provider_member_id"] == json!(format!("wa:+3412345{suffix}"))
                && item["sender_display_name"] == json!("Owner Member")
                && item["role"] == json!("member")
                && item["is_owner"] == json!(false)
        }),
        "expected WhatsApp canonical participant in provider-neutral members list"
    );

    let provider_sync_members_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/whatsapp/provider-sync/conversations/{}/members",
                chat_id
            ),
            json!({
                "account_id": account_id,
                "limit": 20
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp provider-sync members response");
    assert_eq!(provider_sync_members_response.status(), StatusCode::OK);
    let provider_sync_members_body = json_body(provider_sync_members_response).await;
    assert_eq!(provider_sync_members_body["account_id"], json!(account_id));
    assert_eq!(
        provider_sync_members_body["provider_chat_id"],
        json!(chat_id)
    );
    assert_eq!(provider_sync_members_body["status"], json!("synced"));
    assert!(
        provider_sync_members_body["items"]
            .as_array()
            .expect("provider-sync member items")
            .iter()
            .any(|item| {
                item["provider_member_id"] == json!(format!("wa:+3412345{suffix}"))
                    && item["sender_display_name"] == json!("Owner Member")
                    && item["role"] == json!("member")
                    && item["status"] == json!("active")
            }),
        "expected WhatsApp canonical participant in provider-sync members list"
    );

    let conversation_search_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/conversations/search?q=Planning&account_id={account_id}&limit=20"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp conversation search response");
    assert_eq!(conversation_search_response.status(), StatusCode::OK);
    let conversation_search_body = json_body(conversation_search_response).await;
    assert!(
        conversation_search_body["items"]
            .as_array()
            .expect("conversation search items")
            .iter()
            .any(|item| item["telegram_chat_id"] == dialog_body["conversation_id"]),
        "expected WhatsApp canonical conversation in provider-neutral search results"
    );

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "WhatsApp Planning",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "WhatsApp Fixture",
                "text": "Please carry WhatsApp Web context into graph-backed recall.\nAda Lovelace <ada@acme.example>\nPlease review this.",
                "message_metadata": rich_message_metadata,
                "import_batch_id": format!("whatsapp-fixture-{suffix}"),
                "occurred_at": "2026-06-06T13:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    assert!(
        message_body["message_id"]
            .as_str()
            .expect("message id")
            .starts_with("message:v5:whatsapp_web:")
    );
    let projected_message_metadata: Value = sqlx::query_scalar(
        "SELECT message_metadata FROM communication_messages WHERE message_id = $1",
    )
    .bind(message_body["message_id"].as_str().expect("message id"))
    .fetch_one(&pool)
    .await
    .expect("projected whatsapp message metadata");
    assert_eq!(
        projected_message_metadata["message_metadata"]["mentions"],
        rich_message_metadata["mentions"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["links"],
        rich_message_metadata["links"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["location"],
        rich_message_metadata["location"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["contact_card"],
        rich_message_metadata["contact_card"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["sticker"],
        rich_message_metadata["sticker"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["whatsapp_poll"],
        rich_message_metadata["poll"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["whatsapp_link_preview"]["url"],
        rich_message_metadata["links"][0]["url"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["whatsapp_system_message"],
        rich_message_metadata["system_message"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["whatsapp_ephemeral"],
        rich_message_metadata["ephemeral"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["whatsapp_view_once"],
        rich_message_metadata["view_once"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["whatsapp_join_leave"],
        rich_message_metadata["join_leave"]
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["mention_count"],
        json!(1)
    );
    assert_eq!(
        projected_message_metadata["message_metadata"]["mention_usernames"],
        json!(["@owner_member"])
    );
    sqlx::query(
        r#"
        UPDATE communication_messages
        SET message_metadata = message_metadata || '{"is_pinned": true}'::jsonb
        WHERE message_id = $1
        "#,
    )
    .bind(message_body["message_id"].as_str().expect("message id"))
    .execute(&pool)
    .await
    .expect("mark whatsapp message pinned");

    let reply_provider_message_id = format!("wa-message-reply-{suffix}");
    let reply_command_id = format!("wa-reply-reconcile-{suffix}");
    seed_whatsapp_provider_command(
        &pool,
        &reply_command_id,
        &account_id,
        "reply",
        &format!("reply-reconcile:{suffix}"),
        &chat_id,
        Some(&provider_message_id),
        json!({
            "text": "Reply follows the original WhatsApp message.",
            "reply_to_provider_message_id": provider_message_id,
        }),
        json!({
            "provider_chat_id": chat_id,
            "provider_message_id": provider_message_id,
        }),
    )
    .await;
    let reply_message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": reply_provider_message_id,
                "chat_title": "WhatsApp Planning",
                "sender_id": format!("sender-reply-{suffix}"),
                "sender_display_name": "WhatsApp Reply Fixture",
                "text": "Reply follows the original WhatsApp message.",
                "reply_to_provider_message_id": provider_message_id,
                "import_batch_id": format!("whatsapp-fixture-reply-{suffix}"),
                "occurred_at": "2026-06-06T13:00:10Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reply message response");
    assert_eq!(reply_message_response.status(), StatusCode::OK);
    let reply_message_body = json_body(reply_message_response).await;
    let reply_command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&reply_command_id)
    .fetch_one(&pool)
    .await
    .expect("reply reconciled command");
    assert_eq!(
        reply_command_row
            .try_get::<String, _>("status")
            .expect("reply status"),
        "completed"
    );
    assert_eq!(
        reply_command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reply reconciliation status"),
        "observed"
    );
    assert!(
        reply_command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("reply completed_at")
            .is_some()
    );
    let canonical_reply_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&reply_command_id)
    .fetch_one(&pool)
    .await
    .expect("canonical reply command status");
    assert_eq!(canonical_reply_status, "completed");
    let reply_command_events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&reply_command_id)
    .fetch_all(&pool)
    .await
    .expect("reply command reconciliation events");
    assert_eq!(reply_command_events.len(), 2);
    assert_eq!(reply_command_events[0].0, "whatsapp.command.status_changed");
    assert_eq!(reply_command_events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        reply_command_events[1].1["source"],
        json!("provider_observed.fixture_message")
    );
    assert_eq!(
        reply_command_events[1].1["result_payload"]["provider_message_id"],
        json!(reply_provider_message_id)
    );
    assert_eq!(
        reply_command_events[1].1["provider_state"]["reply_to_provider_message_id"],
        json!(provider_message_id)
    );

    let forward_provider_message_id = format!("wa-message-forward-{suffix}");
    let forward_command_id = format!("wa-forward-reconcile-{suffix}");
    seed_whatsapp_provider_command(
        &pool,
        &forward_command_id,
        &account_id,
        "forward",
        &format!("forward-reconcile:{suffix}"),
        &chat_id,
        None,
        json!({
            "from_provider_chat_id": chat_id,
            "from_provider_message_id": reply_provider_message_id,
        }),
        json!({
            "provider_chat_id": chat_id,
            "from_provider_chat_id": chat_id,
            "from_provider_message_id": reply_provider_message_id,
        }),
    )
    .await;
    let forward_message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": forward_provider_message_id,
                "chat_title": "WhatsApp Planning",
                "sender_id": format!("sender-forward-{suffix}"),
                "sender_display_name": "WhatsApp Forward Fixture",
                "text": "Forward preserves observed provider lineage.",
                "forward_origin_chat_id": chat_id,
                "forward_origin_message_id": reply_provider_message_id,
                "forward_origin_sender_id": format!("sender-reply-{suffix}"),
                "forward_origin_sender_name": "WhatsApp Reply Fixture",
                "forwarded_at": "2026-06-06T13:00:20Z",
                "import_batch_id": format!("whatsapp-fixture-forward-{suffix}"),
                "occurred_at": "2026-06-06T13:00:20Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("forward message response");
    assert_eq!(forward_message_response.status(), StatusCode::OK);
    let forward_message_body = json_body(forward_message_response).await;
    let forward_command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&forward_command_id)
    .fetch_one(&pool)
    .await
    .expect("forward reconciled command");
    assert_eq!(
        forward_command_row
            .try_get::<String, _>("status")
            .expect("forward status"),
        "completed"
    );
    assert_eq!(
        forward_command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("forward reconciliation status"),
        "observed"
    );
    assert!(
        forward_command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("forward completed_at")
            .is_some()
    );
    let canonical_forward_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&forward_command_id)
    .fetch_one(&pool)
    .await
    .expect("canonical forward command status");
    assert_eq!(canonical_forward_status, "completed");
    let forward_command_events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&forward_command_id)
    .fetch_all(&pool)
    .await
    .expect("forward command reconciliation events");
    assert_eq!(forward_command_events.len(), 2);
    assert_eq!(
        forward_command_events[0].0,
        "whatsapp.command.status_changed"
    );
    assert_eq!(forward_command_events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        forward_command_events[1].1["source"],
        json!("provider_observed.fixture_message")
    );
    assert_eq!(
        forward_command_events[1].1["result_payload"]["provider_message_id"],
        json!(forward_provider_message_id)
    );
    assert_eq!(
        forward_command_events[1].1["provider_state"]["forward_origin_message_id"],
        json!(reply_provider_message_id)
    );

    let raw_signal_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.whatsapp.message.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("raw whatsapp signal count");
    let accepted_signal_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.accepted.whatsapp.message'",
    )
    .fetch_one(&pool)
    .await
    .expect("accepted whatsapp signal count");
    assert_eq!(raw_signal_count, 3);
    assert_eq!(accepted_signal_count, 3);

    let sessions_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/whatsapp/sessions?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("sessions response");
    assert_eq!(sessions_response.status(), StatusCode::OK);
    let sessions_body = json_body(sessions_response).await;
    assert_eq!(sessions_body["items"][0]["account_id"], json!(account_id));
    assert_eq!(
        sessions_body["items"][0]["last_sync_at"],
        json!("2026-06-06T13:00:20Z")
    );
    let session_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'communications'
          AND link.entity_kind = 'whatsapp_web_session'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&session_id)
    .fetch_all(&pool)
    .await
    .expect("session observations");
    assert!(
        session_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "WHATSAPP_WEB_SESSION"
                && row.get::<String, _>("relationship_kind") == "upsert"
        }),
        "session upsert observation must exist"
    );
    assert!(
        session_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "WHATSAPP_WEB_SESSION"
                && row.get::<String, _>("relationship_kind") == "sync_progress"
                && row.get::<Value, _>("payload")["last_sync_at"] == json!("2026-06-06T13:00:20Z")
        }),
        "session sync_progress observation must exist"
    );

    let messages_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages?account_id={account_id}&conversation_id={chat_id}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("messages response");
    assert_eq!(messages_response.status(), StatusCode::OK);
    let messages_body = json_body(messages_response).await;
    assert_eq!(
        messages_body["items"][0]["channel_kind"],
        json!("whatsapp_web")
    );
    assert_eq!(messages_body["items"][0]["conversation_id"], json!(chat_id));

    let projected_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM communication_messages WHERE account_id = $1 AND channel_kind = 'whatsapp_web'",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("projected WhatsApp Web count");
    assert_eq!(projected_count, 3);

    let pinned_messages_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/conversations/{}/pinned-messages?limit=20",
                dialog_body["conversation_id"]
                    .as_str()
                    .expect("dialog conversation id")
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp pinned messages response");
    assert_eq!(pinned_messages_response.status(), StatusCode::OK);
    let pinned_messages_body = json_body(pinned_messages_response).await;
    assert!(
        pinned_messages_body["items"]
            .as_array()
            .expect("pinned message items")
            .iter()
            .any(|item| item["message_id"] == message_body["message_id"]),
        "expected WhatsApp pinned messages route to use canonical projection"
    );

    let message_update_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/message-updates",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "text": "Please carry edited WhatsApp Web context into graph-backed recall.",
                "import_batch_id": format!("whatsapp-message-update-{suffix}"),
                "observed_at": "2026-06-06T13:00:30Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message update response");
    let message_update_status = message_update_response.status();
    let message_update_body = json_body(message_update_response).await;
    assert_eq!(
        message_update_status,
        StatusCode::OK,
        "message update response body: {message_update_body}"
    );
    assert_eq!(
        message_update_body["message_id"],
        message_body["message_id"]
    );
    assert!(
        message_update_body["version_id"]
            .as_str()
            .expect("version id")
            .starts_with("message_version:v5:whatsapp_web:")
    );

    let updated_message_row = sqlx::query(
        "SELECT body_text, message_metadata FROM communication_messages WHERE message_id = $1",
    )
    .bind(message_body["message_id"].as_str().expect("message id"))
    .fetch_one(&pool)
    .await
    .expect("updated whatsapp message row");
    assert_eq!(
        updated_message_row
            .try_get::<String, _>("body_text")
            .expect("updated body text"),
        "Please carry edited WhatsApp Web context into graph-backed recall."
    );
    let updated_message_metadata = updated_message_row
        .try_get::<Value, _>("message_metadata")
        .expect("updated message metadata");
    assert_eq!(updated_message_metadata["edited"], json!(true));

    let message_version_row = sqlx::query(
        "SELECT version_number, body_text FROM communication_message_versions WHERE version_id = $1",
    )
    .bind(
        message_update_body["version_id"]
            .as_str()
            .expect("version id"),
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp message version row");
    assert_eq!(
        message_version_row
            .try_get::<i32, _>("version_number")
            .expect("version number"),
        1
    );
    assert_eq!(
        message_version_row
            .try_get::<String, _>("body_text")
            .expect("version body text"),
        "Please carry edited WhatsApp Web context into graph-backed recall."
    );

    let receipt_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/receipts",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "delivery_state": "sent",
                "import_batch_id": format!("whatsapp-receipt-{suffix}"),
                "observed_at": "2026-06-06T13:00:45Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("receipt response");
    let receipt_status = receipt_response.status();
    let receipt_body = json_body(receipt_response).await;
    assert_eq!(
        receipt_status,
        StatusCode::OK,
        "receipt response body: {receipt_body}"
    );
    assert_eq!(receipt_body["message_id"], message_body["message_id"]);

    let delivery_state: String = sqlx::query_scalar(
        "SELECT delivery_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(message_body["message_id"].as_str().expect("message id"))
    .fetch_one(&pool)
    .await
    .expect("whatsapp delivery state");
    assert_eq!(delivery_state, "sent");

    let message_delete_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/message-deletes",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "reason_class": "deleted_by_provider",
                "actor_class": "provider",
                "import_batch_id": format!("whatsapp-message-delete-{suffix}"),
                "observed_at": "2026-06-06T13:00:50Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message delete response");
    let message_delete_status = message_delete_response.status();
    let message_delete_body = json_body(message_delete_response).await;
    assert_eq!(
        message_delete_status,
        StatusCode::OK,
        "message delete response body: {message_delete_body}"
    );
    assert_eq!(
        message_delete_body["message_id"],
        message_body["message_id"]
    );
    assert!(
        message_delete_body["tombstone_id"]
            .as_str()
            .expect("tombstone id")
            .starts_with("message_tombstone:v5:whatsapp_web:")
    );

    let tombstone_row = sqlx::query(
        "SELECT is_local_visible, reason_class, actor_class FROM communication_message_tombstones WHERE tombstone_id = $1",
    )
    .bind(
        message_delete_body["tombstone_id"]
            .as_str()
            .expect("tombstone id"),
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp tombstone row");
    assert!(
        !tombstone_row
            .try_get::<bool, _>("is_local_visible")
            .expect("is_local_visible")
    );
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

    let reaction_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/reactions",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "provider_actor_id": format!("reactor-{suffix}"),
                "sender_display_name": "WhatsApp Reactor",
                "reaction": "+1",
                "is_active": true,
                "import_batch_id": format!("whatsapp-reaction-{suffix}"),
                "observed_at": "2026-06-06T13:01:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reaction response");
    let reaction_status = reaction_response.status();
    let reaction_body = json_body(reaction_response).await;
    assert_eq!(
        reaction_status,
        StatusCode::OK,
        "reaction response body: {reaction_body}"
    );
    assert_eq!(reaction_body["message_id"], message_body["message_id"]);
    assert!(reaction_body["reaction_id"].as_str().is_some());

    let reaction_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM communication_message_reactions WHERE message_id = $1 AND reaction = $2 AND is_active = true",
    )
    .bind(message_body["message_id"].as_str().expect("message id"))
    .bind("+1")
    .fetch_one(&pool)
    .await
    .expect("WhatsApp reaction count");
    assert_eq!(reaction_count, 1);

    let reactions_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages/{}/reactions",
                message_body["message_id"].as_str().expect("message id")
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp reactions response");
    assert_eq!(reactions_response.status(), StatusCode::OK);
    let reactions_body = json_body(reactions_response).await;
    assert_eq!(
        reactions_body["reactions"][0]["reaction_emoji"],
        json!("+1")
    );
    assert_eq!(reactions_body["summary"]["total_reactions"], json!(1));

    let reply_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages/{}/reply-chain",
                reply_message_body["message_id"]
                    .as_str()
                    .expect("reply message id")
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp reply chain response");
    assert_eq!(reply_chain_response.status(), StatusCode::OK);
    let reply_chain_body = json_body(reply_chain_response).await;
    assert_eq!(
        reply_chain_body["reply_to"][0]["target_message_summary"]["message_id"],
        message_body["message_id"]
    );
    assert_eq!(
        reply_chain_body["reply_to"][0]["target_message_summary"]["provider_message_id"],
        json!(provider_message_id)
    );

    let forward_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages/{}/forward-chain",
                forward_message_body["message_id"]
                    .as_str()
                    .expect("forward message id")
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp forward chain response");
    assert_eq!(forward_chain_response.status(), StatusCode::OK);
    let forward_chain_body = json_body(forward_chain_response).await;
    assert_eq!(
        forward_chain_body["forwards"][0]["forward_origin_message_id"],
        json!(reply_provider_message_id)
    );
    assert_eq!(
        forward_chain_body["forwards"][0]["source_message_summary"]["message_id"],
        forward_message_body["message_id"]
    );

    let message_ref_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM communication_message_refs WHERE source_message_id IN ($1, $2)",
    )
    .bind(
        reply_message_body["message_id"]
            .as_str()
            .expect("reply message id"),
    )
    .bind(
        forward_message_body["message_id"]
            .as_str()
            .expect("forward message id"),
    )
    .fetch_one(&pool)
    .await
    .expect("WhatsApp message ref count");
    assert_eq!(message_ref_count, 2);

    let media_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/media",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "provider_attachment_id": format!("wa-media-{suffix}"),
                "filename": "planning.png",
                "content_type": "image/png",
                "size_bytes": 12,
                "sha256": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "storage_kind": "local_fs",
                "storage_path": format!("whatsapp/{suffix}/planning.png"),
                "import_batch_id": format!("whatsapp-media-{suffix}"),
                "observed_at": "2026-06-06T13:02:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("media response");
    assert_eq!(media_response.status(), StatusCode::OK);
    let media_body = json_body(media_response).await;
    assert_eq!(media_body["message_id"], message_body["message_id"]);
    assert!(media_body["attachment_id"].as_str().is_some());

    let media_row = sqlx::query(
        r#"
        SELECT content_type, scan_status
        FROM communication_attachments
        WHERE message_id = $1
        "#,
    )
    .bind(message_body["message_id"].as_str().expect("message id"))
    .fetch_one(&pool)
    .await
    .expect("WhatsApp media attachment row");
    assert_eq!(
        media_row
            .try_get::<String, _>("content_type")
            .expect("content type"),
        "image/png"
    );
    assert_eq!(
        media_row
            .try_get::<String, _>("scan_status")
            .expect("scan status"),
        "not_scanned"
    );

    let message_search_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/search/messages?q=edited%20WhatsApp%20Web&account_id={account_id}&provider_chat_id={chat_id}&limit=20"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp message search response");
    assert_eq!(message_search_response.status(), StatusCode::OK);
    let message_search_body = json_body(message_search_response).await;
    assert_eq!(message_search_body["total"], json!(1));
    assert_eq!(
        message_search_body["items"][0]["message_id"],
        message_body["message_id"]
    );
    assert_eq!(
        message_search_body["items"][0]["channel_kind"],
        json!("whatsapp_web")
    );

    let media_search_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/search/media?q=planning&account_id={account_id}&provider_chat_id={chat_id}&limit=20"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp media search response");
    assert_eq!(media_search_response.status(), StatusCode::OK);
    let media_search_body = json_body(media_search_response).await;
    let media_search_items = media_search_body["items"]
        .as_array()
        .expect("whatsapp media search items");
    assert!(
        media_search_items.iter().any(|item| {
            item["provider_attachment_id"] == json!(format!("wa-media-{suffix}"))
                && item["file_name"] == json!("planning.png")
                && item["local_path"] == json!(format!("whatsapp/{suffix}/planning.png"))
        }),
        "expected WhatsApp media search to return canonical attachment projection"
    );

    let versions_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages/{}/versions",
                message_body["message_id"].as_str().expect("message id")
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp versions response");
    assert_eq!(versions_response.status(), StatusCode::OK);
    let versions_body = json_body(versions_response).await;
    assert_eq!(versions_body["message_id"], message_body["message_id"]);
    assert_eq!(
        versions_body["versions"][0]["version_id"],
        message_update_body["version_id"]
    );

    let raw_evidence_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages/{}/raw-evidence",
                message_body["message_id"].as_str().expect("message id")
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp raw evidence response");
    assert_eq!(raw_evidence_response.status(), StatusCode::OK);
    let raw_evidence_body = json_body(raw_evidence_response).await;
    assert_eq!(
        raw_evidence_body["raw_record"]["record_kind"],
        json!("whatsapp_web_message")
    );
    assert_eq!(
        raw_evidence_body["raw_record"]["provider_record_id"],
        json!(provider_message_id)
    );
    assert_eq!(
        raw_evidence_body["raw_record"]["payload"]["message_metadata"]["debug_secret_material"]["session_key"],
        json!("[redacted]")
    );
    assert_eq!(
        raw_evidence_body["raw_record"]["payload"]["message_metadata"]["debug_secret_material"]["access_token"],
        json!("[redacted]")
    );
    assert_eq!(
        raw_evidence_body["raw_record"]["payload"]["message_metadata"]["debug_secret_material"]["nested"]
            ["refresh_token"],
        json!("[redacted]")
    );

    let tombstones_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages/{}/tombstones",
                message_body["message_id"].as_str().expect("message id")
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp tombstones response");
    assert_eq!(tombstones_response.status(), StatusCode::OK);
    let tombstones_body = json_body(tombstones_response).await;
    assert_eq!(tombstones_body["message_id"], message_body["message_id"]);
    assert_eq!(
        tombstones_body["tombstones"][0]["tombstone_id"],
        message_delete_body["tombstone_id"]
    );

    let status_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/statuses",
            json!({
                "account_id": account_id,
                "provider_status_id": format!("wa-status-{suffix}"),
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "WhatsApp Fixture",
                "sender_identity_kind": "whatsapp_phone",
                "sender_address": format!("+3498765{suffix}"),
                "sender_push_name": "Status Push",
                "sender_business_profile": {
                    "category": "advisory",
                    "description": "Status publisher profile"
                },
                "sender_profile_photo_ref": {
                    "provider_file_id": format!("status-avatar-{suffix}"),
                    "sha256": "status-photo-sha256"
                },
                "text": format!(
                    "Decision: Use WhatsApp status evidence {suffix} because statuses can carry shared context. I will publish the status summary {suffix} by Monday 9am. Attached contract pdf for review."
                ),
                "import_batch_id": format!("whatsapp-status-{suffix}"),
                "occurred_at": "2026-06-06T13:03:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status response");
    let fixture_status_status = status_response.status();
    let status_body = json_body(status_response).await;
    assert_eq!(
        fixture_status_status,
        StatusCode::OK,
        "fixture status response body: {status_body}"
    );
    assert!(
        status_body["message_id"]
            .as_str()
            .expect("status message id")
            .starts_with("message:v5:whatsapp_web:")
    );

    let status_object_type: String = sqlx::query_scalar(
        "SELECT message_metadata->>'communication_object_type' FROM communication_messages WHERE message_id = $1",
    )
    .bind(status_body["message_id"].as_str().expect("status message id"))
    .fetch_one(&pool)
    .await
    .expect("WhatsApp status object type");
    assert_eq!(status_object_type, "status");
    let status_feed_conversation_id = format!("whatsapp_status_feed:{account_id}");
    let status_feed_row = sqlx::query(
        r#"
        SELECT title, provider_conversation_id, metadata
        FROM communication_conversations
        WHERE conversation_id = $1
        "#,
    )
    .bind(&status_feed_conversation_id)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp status feed conversation");
    assert_eq!(
        status_feed_row
            .try_get::<String, _>("title")
            .expect("status feed title"),
        "WhatsApp Status"
    );
    assert_eq!(
        status_feed_row
            .try_get::<String, _>("provider_conversation_id")
            .expect("status feed provider conversation id"),
        "status-feed"
    );
    let status_feed_metadata: Value = status_feed_row
        .try_get("metadata")
        .expect("status feed metadata");
    assert_eq!(status_feed_metadata["chat_kind"], json!("status_feed"));
    assert_eq!(status_feed_metadata["is_status_feed"], json!(true));
    assert_eq!(
        status_feed_metadata["provider_status_id"],
        json!(format!("wa-status-{suffix}"))
    );
    assert_eq!(
        status_feed_metadata["status_author_identity_kind"],
        json!("whatsapp_phone")
    );

    let status_feed_conversations_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations?account_id={account_id}&limit=50"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status feed conversations response");
    assert_eq!(status_feed_conversations_response.status(), StatusCode::OK);
    let status_feed_conversations_body = json_body(status_feed_conversations_response).await;
    assert!(
        status_feed_conversations_body["items"]
            .as_array()
            .expect("status feed conversation items")
            .iter()
            .any(|item| {
                item["telegram_chat_id"] == json!(status_feed_conversation_id)
                    && item["provider_chat_id"] == json!("status-feed")
                    && item["title"] == json!("WhatsApp Status")
                    && item["chat_kind"] == json!("status_feed")
                    && item["metadata"]["is_status_feed"] == json!(true)
            }),
        "expected WhatsApp status feed in provider-neutral conversation list: {status_feed_conversations_body}"
    );

    let status_feed_detail_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations/{status_feed_conversation_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status feed conversation detail response");
    assert_eq!(status_feed_detail_response.status(), StatusCode::OK);
    let status_feed_detail_body = json_body(status_feed_detail_response).await;
    assert_eq!(
        status_feed_detail_body["item"]["provider_chat_id"],
        json!("status-feed")
    );
    assert_eq!(
        status_feed_detail_body["item"]["metadata"]["is_status_feed"],
        json!(true)
    );

    let status_feed_search_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/conversations/search?q=Status&account_id={account_id}&limit=20"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status feed conversation search response");
    assert_eq!(status_feed_search_response.status(), StatusCode::OK);
    let status_feed_search_body = json_body(status_feed_search_response).await;
    assert!(
        status_feed_search_body["items"]
            .as_array()
            .expect("status feed search items")
            .iter()
            .any(|item| item["telegram_chat_id"] == json!(status_feed_conversation_id)),
        "expected WhatsApp status feed in provider-neutral search results: {status_feed_search_body}"
    );

    let provider_sync_statuses_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-sync/statuses",
            json!({
                "account_id": account_id,
                "limit": 20
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("whatsapp provider-sync statuses response");
    assert_eq!(provider_sync_statuses_response.status(), StatusCode::OK);
    let provider_sync_statuses_body = json_body(provider_sync_statuses_response).await;
    assert_eq!(provider_sync_statuses_body["account_id"], json!(account_id));
    assert_eq!(
        provider_sync_statuses_body["provider_chat_id"],
        json!("status-feed")
    );
    assert_eq!(provider_sync_statuses_body["status"], json!("synced"));
    assert!(
        provider_sync_statuses_body["items"]
            .as_array()
            .expect("provider-sync status items")
            .iter()
            .any(|item| {
                item["provider_chat_id"] == json!("status-feed")
                    && item["message_id"] == status_body["message_id"]
            }),
        "expected WhatsApp status in provider-sync statuses list"
    );

    let status_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(
        status_body["message_id"]
            .as_str()
            .expect("status message id"),
    )
    .fetch_one(&pool)
    .await
    .expect("status observation id");
    let status_decision_title = format!("Use WhatsApp status evidence {suffix}");
    let status_decision_row: (String, String, String, String, String) = sqlx::query_as(
        r#"
        SELECT d.title, d.rationale, d.review_state, e.source_kind, e.source_id
        FROM decisions d
        JOIN decision_evidence e ON e.decision_id = d.decision_id
        WHERE e.source_kind = 'communication'
          AND e.source_id = $1
          AND d.title = $2
        "#,
    )
    .bind(
        status_body["message_id"]
            .as_str()
            .expect("status message id"),
    )
    .bind(&status_decision_title)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp status should create a suggested Decision candidate");
    assert_eq!(status_decision_row.2, "suggested");
    assert_eq!(status_decision_row.3, "communication");
    assert_eq!(
        status_decision_row.4,
        status_body["message_id"]
            .as_str()
            .expect("status message id")
    );
    let status_task_statement = format!("publish the status summary {suffix}");
    let status_task_candidate_row: (String, String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT title, review_state, candidate_kind, due_text
        FROM task_candidates
        WHERE source_kind = 'observation'
          AND source_id = $1
          AND candidate_kind = 'obligation_task'
        "#,
    )
    .bind(&status_observation_id)
    .fetch_one(&pool)
    .await
    .expect("WhatsApp status should create an obligation-derived task candidate");
    assert_eq!(status_task_candidate_row.0, status_task_statement);
    assert_eq!(status_task_candidate_row.1, "suggested");
    assert_eq!(status_task_candidate_row.2, "obligation_task");
    assert_eq!(status_task_candidate_row.3.as_deref(), Some("Monday 9am"));
    let status_knowledge_review_items = sqlx::query(
        r#"
        SELECT metadata
        FROM review_items
        WHERE item_kind = 'knowledge_candidate'
          AND review_item_id IN (
              SELECT review_item_id
              FROM review_item_evidence
              WHERE observation_id = $1
          )
        ORDER BY metadata->>'candidate_group', title
        "#,
    )
    .bind(&status_observation_id)
    .fetch_all(&pool)
    .await
    .expect("status knowledge review items");
    let mut status_candidate_groups = status_knowledge_review_items
        .into_iter()
        .map(|row| {
            let metadata: Value = row.try_get("metadata").expect("metadata");
            metadata["candidate_group"]
                .as_str()
                .expect("candidate group")
                .to_owned()
        })
        .collect::<Vec<_>>();
    status_candidate_groups.sort();
    status_candidate_groups.dedup();
    assert_eq!(
        status_candidate_groups,
        vec!["agreement".to_owned(), "document".to_owned()]
    );
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(message_body["message_id"].as_str().expect("message id"))
    .fetch_one(&pool)
    .await
    .expect("whatsapp message observation id");
    let persona_review_items = sqlx::query(
        r#"
        SELECT item_kind, title, metadata
        FROM review_items
        WHERE item_kind IN ('new_persona', 'new_organization')
          AND review_item_id IN (
              SELECT review_item_id
              FROM review_item_evidence
              WHERE observation_id = $1
          )
        ORDER BY item_kind, title
        "#,
    )
    .bind(&message_observation_id)
    .fetch_all(&pool)
    .await
    .expect("message persona review items");
    assert!(persona_review_items.iter().any(|row| {
        row.try_get::<String, _>("item_kind").ok().as_deref() == Some("new_persona")
            && row.try_get::<String, _>("title").ok().as_deref() == Some("WhatsApp Fixture")
    }));
    assert!(persona_review_items.iter().any(|row| {
        row.try_get::<String, _>("item_kind").ok().as_deref() == Some("new_persona")
            && row.try_get::<String, _>("title").ok().as_deref() == Some("Ada Lovelace")
    }));
    assert!(persona_review_items.iter().any(|row| {
        row.try_get::<String, _>("item_kind").ok().as_deref() == Some("new_organization")
            && row.try_get::<String, _>("title").ok().as_deref() == Some("acme.example")
            && row
                .try_get::<Value, _>("metadata")
                .ok()
                .and_then(|metadata| metadata.get("candidate_group").cloned())
                == Some(json!("organization"))
    }));

    let status_view_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/status-views",
            json!({
                "account_id": account_id,
                "provider_status_id": format!("wa-status-{suffix}"),
                "viewer_id": format!("viewer-{suffix}"),
                "viewer_display_name": "Status Viewer",
                "import_batch_id": format!("whatsapp-status-view-{suffix}"),
                "observed_at": "2026-06-06T13:04:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status view response");
    assert_eq!(status_view_response.status(), StatusCode::OK);
    let status_view_body = json_body(status_view_response).await;
    assert_eq!(status_view_body["message_id"], status_body["message_id"]);

    let status_media_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/media",
            json!({
                "account_id": account_id,
                "provider_chat_id": format!("whatsapp_status_feed:{account_id}"),
                "provider_message_id": format!("wa-status-{suffix}"),
                "provider_attachment_id": format!("wa-status-attachment-{suffix}"),
                "filename": "status-photo.jpg",
                "content_type": "image/jpeg",
                "size_bytes": 2048,
                "sha256": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "storage_kind": "local_fs",
                "storage_path": format!("whatsapp/{suffix}/status-photo.jpg"),
                "import_batch_id": format!("whatsapp-status-media-{suffix}"),
                "observed_at": "2026-06-06T13:03:30Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status media response");
    assert_eq!(status_media_response.status(), StatusCode::OK);
    let status_media_body = json_body(status_media_response).await;
    assert_eq!(status_media_body["message_id"], status_body["message_id"]);

    let status_reply_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("wa-status-reply-{suffix}"),
                "chat_title": "WhatsApp Planning",
                "sender_id": format!("sender-status-reply-{suffix}"),
                "sender_display_name": "WhatsApp Status Reply Fixture",
                "text": "Replying directly to observed status evidence.",
                "reply_to_provider_message_id": format!("wa-status-{suffix}"),
                "import_batch_id": format!("whatsapp-status-reply-{suffix}"),
                "occurred_at": "2026-06-06T13:03:40Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status reply response");
    assert_eq!(status_reply_response.status(), StatusCode::OK);
    let status_reply_body = json_body(status_reply_response).await;
    let status_reply_ref_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM communication_message_refs
        WHERE source_message_id = $1
          AND target_message_id = $2
          AND ref_kind = 'reply'
        "#,
    )
    .bind(
        status_reply_body["message_id"]
            .as_str()
            .expect("status reply message id"),
    )
    .bind(
        status_body["message_id"]
            .as_str()
            .expect("status message id"),
    )
    .fetch_one(&pool)
    .await
    .expect("status reply refs");
    assert_eq!(status_reply_ref_count, 1);

    let status_delete_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/status-deletes",
            json!({
                "account_id": account_id,
                "provider_status_id": format!("wa-status-{suffix}"),
                "actor_class": "self",
                "reason_class": "status_expired",
                "import_batch_id": format!("whatsapp-status-delete-{suffix}"),
                "observed_at": "2026-06-06T13:05:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status delete response");
    assert_eq!(status_delete_response.status(), StatusCode::OK);
    let status_delete_body = json_body(status_delete_response).await;
    assert_eq!(status_delete_body["message_id"], status_body["message_id"]);
    assert!(
        status_delete_body["tombstone_id"]
            .as_str()
            .expect("status tombstone id")
            .starts_with("message_tombstone:v5:whatsapp_web:")
    );

    let status_metadata: Value = sqlx::query_scalar(
        "SELECT message_metadata FROM communication_messages WHERE message_id = $1",
    )
    .bind(
        status_body["message_id"]
            .as_str()
            .expect("status message id"),
    )
    .fetch_one(&pool)
    .await
    .expect("WhatsApp status metadata");
    assert!(
        status_metadata["status_author_identity_id"]
            .as_str()
            .expect("status author identity id")
            .starts_with("identity:v5:whatsapp_web:")
    );
    assert_eq!(
        status_metadata["status_author_identity_kind"],
        json!("whatsapp_phone")
    );
    assert_eq!(
        status_metadata["status_author_address"],
        json!(format!("+3498765{suffix}"))
    );
    assert_eq!(
        status_metadata["status_author_push_name"],
        json!("Status Push")
    );
    assert_eq!(
        status_metadata["status_author_business_profile"]["category"],
        json!("advisory")
    );
    assert_eq!(
        status_metadata["status_author_profile_photo_ref"]["provider_file_id"],
        json!(format!("status-avatar-{suffix}"))
    );
    assert_eq!(status_metadata["status_viewed"], json!(true));
    assert_eq!(status_metadata["status_view_count"], json!(1));
    assert_eq!(
        status_metadata["status_last_viewer_id"],
        json!(format!("viewer-{suffix}"))
    );
    assert_eq!(status_metadata["status_deleted"], json!(true));
    assert_eq!(
        status_metadata["status_delete_reason_class"],
        json!("status_expired")
    );
    let status_attachment_row = sqlx::query(
        r#"
        SELECT message_id, filename, content_type
        FROM communication_attachments
        WHERE attachment_id = $1
        "#,
    )
    .bind(
        status_media_body["attachment_id"]
            .as_str()
            .expect("status attachment id"),
    )
    .fetch_one(&pool)
    .await
    .expect("status attachment row");
    assert_eq!(
        status_attachment_row
            .try_get::<String, _>("message_id")
            .expect("status attachment message id"),
        status_body["message_id"]
            .as_str()
            .expect("status message id")
    );
    assert_eq!(
        status_attachment_row
            .try_get::<String, _>("filename")
            .expect("status attachment filename"),
        "status-photo.jpg"
    );

    for (event_type, expected_count) in [
        ("signal.raw.whatsapp.reaction.observed", 1_i64),
        ("signal.accepted.whatsapp.reaction", 1_i64),
        ("signal.raw.whatsapp.media.observed", 2_i64),
        ("signal.accepted.whatsapp.media", 2_i64),
        ("signal.raw.whatsapp.status.observed", 1_i64),
        ("signal.accepted.whatsapp.status", 1_i64),
        ("signal.raw.whatsapp.status_view.observed", 1_i64),
        ("signal.accepted.whatsapp.status_view", 1_i64),
        ("signal.raw.whatsapp.status_delete.observed", 1_i64),
        ("signal.accepted.whatsapp.status_delete", 1_i64),
        ("signal.raw.whatsapp.presence.observed", 1_i64),
        ("signal.accepted.whatsapp.presence", 1_i64),
        ("signal.raw.whatsapp.call_metadata.observed", 1_i64),
        ("signal.accepted.whatsapp.call_metadata", 1_i64),
        ("signal.raw.whatsapp.message_update.observed", 1_i64),
        ("signal.accepted.whatsapp.message_update", 1_i64),
        ("signal.raw.whatsapp.receipt.observed", 1_i64),
        ("signal.accepted.whatsapp.receipt", 1_i64),
        ("signal.raw.whatsapp.message_delete.observed", 1_i64),
        ("signal.accepted.whatsapp.message_delete", 1_i64),
    ] {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM event_log WHERE event_type = $1")
            .bind(event_type)
            .fetch_one(&pool)
            .await
            .expect("WhatsApp fixture event count");
        assert_eq!(count, expected_count, "unexpected {event_type} event count");
    }

    for (event_type, expected_count) in [
        ("whatsapp.dialog.updated", 1_i64),
        ("whatsapp.message.created", 4_i64),
        ("whatsapp.message.updated", 1_i64),
        ("whatsapp.message.deleted", 1_i64),
        ("whatsapp.reaction.changed", 1_i64),
        ("whatsapp.receipt.changed", 1_i64),
        ("whatsapp.presence.changed", 1_i64),
        ("whatsapp.call.updated", 1_i64),
        ("whatsapp.status.updated", 2_i64),
        ("whatsapp.status.deleted", 1_i64),
    ] {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM event_log WHERE event_type = $1")
            .bind(event_type)
            .fetch_one(&pool)
            .await
            .expect("WhatsApp realtime event count");
        assert_eq!(count, expected_count, "unexpected {event_type} event count");
    }

    let message_created_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.message.created' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp message created payload");
    assert_eq!(
        message_created_payload["message_id"],
        message_body["message_id"]
    );
    assert_eq!(
        message_created_payload["provider_message_id"],
        json!(provider_message_id)
    );
    assert_eq!(message_created_payload.get("text"), None);
    assert_eq!(message_created_payload.get("body"), None);

    let message_updated_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.message.updated' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp message updated payload");
    assert_eq!(
        message_updated_payload["message_id"],
        message_body["message_id"]
    );
    assert_eq!(message_updated_payload["edited"], json!(true));
    assert_eq!(message_updated_payload.get("text"), None);

    let message_deleted_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.message.deleted' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp message deleted payload");
    assert_eq!(
        message_deleted_payload["message_id"],
        message_body["message_id"]
    );
    assert_eq!(
        message_deleted_payload["tombstone_id"],
        message_delete_body["tombstone_id"]
    );

    let status_updated_payloads: Vec<Value> = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.status.updated' ORDER BY position ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("whatsapp status updated payloads");
    assert_eq!(
        status_updated_payloads[0]["message_id"],
        status_body["message_id"]
    );
    assert_eq!(status_updated_payloads[0]["status_state"], json!("posted"));
    assert_eq!(
        status_updated_payloads[0]["sender_identity_kind"],
        json!("whatsapp_phone")
    );
    assert_eq!(
        status_updated_payloads[0]["sender_push_name"],
        json!("Status Push")
    );
    assert_eq!(status_updated_payloads[1]["status_state"], json!("viewed"));
    assert_eq!(
        status_updated_payloads[1]["viewer_id"],
        json!(format!("viewer-{suffix}"))
    );

    let status_deleted_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.status.deleted' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp status deleted payload");
    assert_eq!(
        status_deleted_payload["message_id"],
        status_body["message_id"]
    );
    assert_eq!(status_deleted_payload["status_state"], json!("deleted"));
    assert_eq!(
        status_deleted_payload["tombstone_id"],
        status_delete_body["tombstone_id"]
    );

    let status_author_identity_metadata: Value =
        sqlx::query_scalar("SELECT metadata FROM communication_identities WHERE identity_id = $1")
            .bind(
                status_metadata["status_author_identity_id"]
                    .as_str()
                    .expect("status author identity id"),
            )
            .fetch_one(&pool)
            .await
            .expect("WhatsApp status author identity metadata");
    assert_eq!(
        status_author_identity_metadata["status_author"],
        json!(true)
    );
    assert_eq!(
        status_author_identity_metadata["push_name"],
        json!("Status Push")
    );
    assert_eq!(
        status_author_identity_metadata["business_profile"]["description"],
        json!("Status publisher profile")
    );
    assert_eq!(
        status_author_identity_metadata["profile_photo_ref"]["sha256"],
        json!("status-photo-sha256")
    );

    let persona_identity_traces: Vec<(String, String, Option<String>, String, Value)> =
        sqlx::query_as(
            r#"
        SELECT identity_type, identity_value, persona_id, source, metadata
        FROM persona_identities
        WHERE source = 'communication_projection'
          AND identity_type IN ('whatsapp', 'phone')
          AND identity_value IN ($1, $2, $3, $4, $5)
        ORDER BY identity_type, identity_value
        "#,
        )
        .bind(format!("wa:+3412345{suffix}"))
        .bind(format!("+3412345{suffix}"))
        .bind(format!("sender-{suffix}"))
        .bind(format!("+3498765{suffix}"))
        .bind("+34910000000")
        .fetch_all(&pool)
        .await
        .expect("WhatsApp persona identity traces");
    assert_eq!(persona_identity_traces.len(), 5);
    assert!(persona_identity_traces.iter().any(|trace| {
        trace.0 == "whatsapp"
            && trace.1 == format!("wa:+3412345{suffix}")
            && trace.2.is_none()
            && trace.3 == "communication_projection"
            && trace.4["whatsapp_participant_evidence"]["push_name"] == json!("Owner Push")
            && trace.4["whatsapp_participant_evidence"]["business_profile"]["category"]
                == json!("consulting")
    }));
    assert!(persona_identity_traces.iter().any(|trace| {
        trace.0 == "phone"
            && trace.1 == format!("+3412345{suffix}")
            && trace.2.is_none()
            && trace.3 == "communication_projection"
            && trace.4["whatsapp_participant_evidence"]["provider_member_id"]
                == json!(format!("member-owner-{suffix}"))
    }));
    assert!(persona_identity_traces.iter().any(|trace| {
        trace.0 == "whatsapp"
            && trace.1 == format!("sender-{suffix}")
            && trace.2.is_none()
            && trace.3 == "communication_projection"
            && trace.4["whatsapp_status_author_evidence"]["sender_push_name"]
                == json!("Status Push")
    }));
    assert!(persona_identity_traces.iter().any(|trace| {
        trace.0 == "phone"
            && trace.1 == format!("+3498765{suffix}")
            && trace.2.is_none()
            && trace.3 == "communication_projection"
            && trace.4["whatsapp_status_author_evidence"]["sender_business_profile"]["category"]
                == json!("advisory")
    }));
    assert!(persona_identity_traces.iter().any(|trace| {
        trace.0 == "phone"
            && trace.1 == "+34910000000"
            && trace.2.is_none()
            && trace.3 == "communication_projection"
            && trace.4["whatsapp_contact_card_evidence"]["contact_card"]["display_name"]
                == json!("Ops Desk")
    }));

    let message_participant_trace: Option<(String, Option<String>, String, Value)> =
        sqlx::query_as(
            r#"
        SELECT identity_value, persona_id, source, metadata
        FROM persona_identities
        WHERE source = 'communication_projection'
          AND identity_type = 'message_participant'
          AND identity_value = $1
        LIMIT 1
        "#,
        )
        .bind(format!(
            "whatsapp_participant:v1:{account_id}:{chat_id}:member-owner-{suffix}"
        ))
        .fetch_optional(&pool)
        .await
        .expect("WhatsApp message participant trace");
    let message_participant_trace = message_participant_trace.expect("message participant trace");
    assert_eq!(message_participant_trace.1, None);
    assert_eq!(message_participant_trace.2, "communication_projection");
    assert_eq!(
        message_participant_trace.3["whatsapp_participant_evidence"]["role"],
        json!("member")
    );
    assert_eq!(
        message_participant_trace.3["whatsapp_participant_evidence"]["status"],
        json!("left")
    );
    assert_eq!(
        message_participant_trace.3["whatsapp_participant_evidence"]["profile_photo_ref"]["provider_file_id"],
        json!(format!("avatar-{suffix}"))
    );

    let participant_trace_links: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM observation_links link
        JOIN persona_identities identity_trace
          ON identity_trace.id::text = link.entity_id
        WHERE link.domain = 'personas'
          AND link.entity_kind = 'identity_trace'
          AND identity_trace.source = 'communication_projection'
          AND identity_trace.identity_type = 'whatsapp'
          AND identity_trace.identity_value = $1
        "#,
    )
    .bind(format!("wa:+3412345{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("participant identity trace links");
    assert!(participant_trace_links >= 1);

    let message_participant_trace_links: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM observation_links link
        JOIN persona_identities identity_trace
          ON identity_trace.id::text = link.entity_id
        WHERE link.domain = 'personas'
          AND link.entity_kind = 'identity_trace'
          AND identity_trace.source = 'communication_projection'
          AND identity_trace.identity_type = 'message_participant'
          AND identity_trace.identity_value = $1
        "#,
    )
    .bind(format!(
        "whatsapp_participant:v1:{account_id}:{chat_id}:member-owner-{suffix}"
    ))
    .fetch_one(&pool)
    .await
    .expect("message participant trace links");
    assert!(message_participant_trace_links >= 1);

    let status_trace_links: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM observation_links link
        JOIN persona_identities identity_trace
          ON identity_trace.id::text = link.entity_id
        WHERE link.domain = 'personas'
          AND link.entity_kind = 'identity_trace'
          AND identity_trace.source = 'communication_projection'
          AND identity_trace.identity_type = 'phone'
          AND identity_trace.identity_value = $1
        "#,
    )
    .bind(format!("+3498765{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("status phone trace links");
    assert!(status_trace_links >= 1);

    let reaction_changed_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.reaction.changed' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp reaction changed payload");
    assert_eq!(
        reaction_changed_payload["message_id"],
        message_body["message_id"]
    );
    assert_eq!(reaction_changed_payload["reaction"], json!("+1"));
    assert_eq!(reaction_changed_payload["is_active"], json!(true));

    let receipt_changed_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.receipt.changed' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp receipt changed payload");
    assert_eq!(
        receipt_changed_payload["message_id"],
        message_body["message_id"]
    );
    assert_eq!(receipt_changed_payload["delivery_state"], json!("sent"));

    let presence_changed_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.presence.changed' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp presence changed payload");
    assert_eq!(
        presence_changed_payload["identity_id"],
        participant_body["identity_id"]
    );
    assert_eq!(presence_changed_payload["provider_chat_id"], json!(chat_id));
    assert_eq!(
        presence_changed_payload["provider_identity_id"],
        json!(format!("wa:+3412345{suffix}"))
    );
    assert_eq!(presence_changed_payload["presence_state"], json!("typing"));
    assert_eq!(
        presence_changed_payload["last_seen_at"],
        json!("2026-06-06T12:58:00Z")
    );

    let call_updated_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.call.updated' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp call updated payload");
    assert_eq!(call_updated_payload["call_id"], call_body["call_id"]);
    assert_eq!(
        call_updated_payload["provider_call_id"],
        json!(format!("wa-call-{suffix}"))
    );
    assert_eq!(call_updated_payload["provider_chat_id"], json!(chat_id));
    assert_eq!(call_updated_payload["direction"], json!("incoming"));
    assert_eq!(call_updated_payload["call_state"], json!("missed"));

    let dialog_updated_payload: Value = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.dialog.updated' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("whatsapp dialog updated payload");
    assert_eq!(
        dialog_updated_payload["conversation_id"],
        dialog_body["conversation_id"]
    );
    assert_eq!(dialog_updated_payload["provider_chat_id"], json!(chat_id));
    assert_eq!(dialog_updated_payload["chat_kind"], json!("group"));
    assert_eq!(dialog_updated_payload["unread_count"], json!(7));
    assert_eq!(dialog_updated_payload["participant_count"], json!(3));
    assert_eq!(
        dialog_updated_payload["community_parent_chat_id"],
        json!(format!("community-root-{suffix}"))
    );
    assert_eq!(
        dialog_updated_payload["community_parent_title"],
        json!("Макошь Community")
    );
    assert_eq!(
        dialog_updated_payload["invite_link"],
        json!(format!("https://chat.whatsapp.com/community-{suffix}"))
    );
    assert_eq!(dialog_updated_payload["is_newsletter"], json!(true));
    assert_eq!(
        dialog_updated_payload["provider_labels"],
        json!(["team", "urgent"])
    );

    let projected_count_after_status: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM communication_messages WHERE account_id = $1 AND channel_kind = 'whatsapp_web'",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("projected WhatsApp Web count after status");
    assert_eq!(projected_count_after_status, 5);

    let dialog_title: String = sqlx::query_scalar(
        "SELECT title FROM communication_conversations WHERE conversation_id = $1",
    )
    .bind(
        dialog_body["conversation_id"]
            .as_str()
            .expect("dialog conversation id"),
    )
    .fetch_one(&pool)
    .await
    .expect("WhatsApp conversation title");
    assert_eq!(dialog_title, "WhatsApp Planning");

    let dialog_metadata: Value = sqlx::query_scalar(
        "SELECT metadata FROM communication_conversations WHERE conversation_id = $1",
    )
    .bind(
        dialog_body["conversation_id"]
            .as_str()
            .expect("dialog conversation id"),
    )
    .fetch_one(&pool)
    .await
    .expect("WhatsApp conversation metadata");
    assert_eq!(dialog_metadata["chat_kind"], json!("group"));
    assert_eq!(dialog_metadata["is_pinned"], json!(true));
    assert_eq!(dialog_metadata["is_muted"], json!(true));
    assert_eq!(dialog_metadata["is_unread"], json!(true));
    assert_eq!(dialog_metadata["unread_count"], json!(7));
    assert_eq!(dialog_metadata["participant_count"], json!(3));
    assert_eq!(
        dialog_metadata["community_parent_chat_id"],
        json!(format!("community-root-{suffix}"))
    );
    assert_eq!(
        dialog_metadata["community_parent_title"],
        json!("Макошь Community")
    );
    assert_eq!(
        dialog_metadata["invite_link"],
        json!(format!("https://chat.whatsapp.com/community-{suffix}"))
    );
    assert_eq!(dialog_metadata["is_community_root"], json!(false));
    assert_eq!(dialog_metadata["is_broadcast"], json!(false));
    assert_eq!(dialog_metadata["is_newsletter"], json!(true));
    assert_eq!(
        dialog_metadata["avatar_metadata"]["profile_picture_url"],
        json!("https://example.com/whatsapp-planning.jpg")
    );
    assert_eq!(
        dialog_metadata["provider_labels"],
        json!(["team", "urgent"])
    );

    let identity_row = sqlx::query(
        "SELECT identity_kind, display_name, address FROM communication_identities WHERE identity_id = $1",
    )
    .bind(participant_body["identity_id"].as_str().expect("identity id"))
    .fetch_one(&pool)
    .await
    .expect("WhatsApp identity row");
    assert_eq!(
        identity_row
            .try_get::<String, _>("identity_kind")
            .expect("identity kind"),
        "whatsapp_phone"
    );
    assert_eq!(
        identity_row
            .try_get::<String, _>("display_name")
            .expect("display name"),
        "Owner Member Renamed"
    );

    let identity_metadata_row: Value =
        sqlx::query_scalar("SELECT metadata FROM communication_identities WHERE identity_id = $1")
            .bind(
                participant_body["identity_id"]
                    .as_str()
                    .expect("identity id"),
            )
            .fetch_one(&pool)
            .await
            .expect("WhatsApp identity metadata");
    assert_eq!(identity_metadata_row["push_name"], json!("Owner Push"));
    assert_eq!(
        identity_metadata_row["display_name_history"],
        json!(["Owner Member", "Owner Member Renamed"])
    );
    assert_eq!(
        identity_metadata_row["previous_display_name"],
        json!("Owner Member")
    );
    assert_eq!(
        identity_metadata_row["business_profile"]["description"],
        json!("Макошь project coordination")
    );
    assert_eq!(
        identity_metadata_row["profile_photo_ref"]["sha256"],
        json!("photo-sha256")
    );

    let participant_metadata: Value = sqlx::query_scalar(
        "SELECT metadata FROM communication_conversation_participants WHERE participant_id = $1",
    )
    .bind(
        participant_body["participant_id"]
            .as_str()
            .expect("participant id"),
    )
    .fetch_one(&pool)
    .await
    .expect("WhatsApp participant metadata");
    assert_eq!(participant_metadata["is_owner"], json!(false));
    assert_eq!(participant_metadata["is_admin"], json!(false));
    assert_eq!(participant_metadata["push_name"], json!("Owner Push"));
    assert_eq!(participant_metadata["status"], json!("left"));
    assert_eq!(participant_metadata["previous_role"], json!("owner"));
    assert_eq!(participant_metadata["previous_status"], json!("member"));
    assert_eq!(participant_metadata["role_changed"], json!(true));
    assert_eq!(participant_metadata["membership_changed"], json!(true));
    assert_eq!(
        participant_metadata["business_profile"]["category"],
        json!("consulting")
    );
    assert_eq!(
        participant_metadata["profile_photo_ref"]["provider_file_id"],
        json!(format!("avatar-{suffix}"))
    );

    for event_type in [
        "signal.raw.whatsapp.dialog.observed",
        "signal.accepted.whatsapp.dialog",
        "signal.raw.whatsapp.participant.observed",
        "signal.accepted.whatsapp.participant",
    ] {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM event_log WHERE event_type = $1")
            .bind(event_type)
            .fetch_one(&pool)
            .await
            .expect("WhatsApp dialog/participant event count");
        let expected = if event_type.contains("participant") {
            2
        } else {
            1
        };
        assert_eq!(count, expected, "expected {expected} {event_type} event(s)");
    }

    let participant_changed_payloads: Vec<Value> = sqlx::query_scalar(
        "SELECT payload FROM event_log WHERE event_type = 'whatsapp.participant.changed' ORDER BY position ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("whatsapp participant changed payloads");
    assert_eq!(participant_changed_payloads.len(), 2);
    assert_eq!(
        participant_changed_payloads[0]["role_changed"],
        json!(false)
    );
    assert_eq!(
        participant_changed_payloads[0]["membership_changed"],
        json!(false)
    );
    assert_eq!(
        participant_changed_payloads[1]["previous_role"],
        json!("owner")
    );
    assert_eq!(participant_changed_payloads[1]["role"], json!("member"));
    assert_eq!(
        participant_changed_payloads[1]["previous_status"],
        json!("member")
    );
    assert_eq!(participant_changed_payloads[1]["status"], json!("left"));
    assert_eq!(participant_changed_payloads[1]["role_changed"], json!(true));
    assert_eq!(
        participant_changed_payloads[1]["membership_changed"],
        json!(true)
    );

    let event_store = EventStore::new(pool.clone());
    let mut replay_events = Vec::new();
    for event_type in [
        "whatsapp.status.updated",
        "whatsapp.participant.changed",
        "whatsapp.call.updated",
    ] {
        replay_events.extend(
            event_store
                .list_matching(EventLogQuery::default().event_type(event_type).limit(10))
                .await
                .expect("WhatsApp timeline replay events"),
        );
    }

    let replay = TimelineEngine::replay_event_log(
        &replay_events,
        Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap(),
        20,
    )
    .expect("WhatsApp events should replay through timeline engine");

    assert_eq!(
        replay
            .entries
            .iter()
            .filter(|entry| entry.event_type == "whatsapp.status.updated")
            .count(),
        2
    );
    assert!(replay.entries.iter().any(|entry| {
        entry.event_type == "whatsapp.status.updated"
            && entry.entity_kind == "whatsapp_status"
            && entry.entity_id
                == status_body["message_id"]
                    .as_str()
                    .expect("status message id")
            && entry.source.starts_with("communication_raw_records:")
    }));
    assert!(replay.entries.iter().any(|entry| {
        entry.event_type == "whatsapp.participant.changed"
            && entry.entity_kind == "whatsapp_participant"
            && entry.entity_id
                == participant_body["participant_id"]
                    .as_str()
                    .expect("participant id")
            && entry.source.starts_with("communication_raw_records:")
    }));
    assert!(replay.entries.iter().any(|entry| {
        entry.event_type == "whatsapp.call.updated"
            && entry.entity_kind == "whatsapp_call"
            && entry.entity_id == call_body["call_id"].as_str().expect("call id")
            && entry.source.starts_with("communication_raw_records:")
    }));
}

#[tokio::test]
async fn whatsapp_fixture_sync_surfaces_return_projected_chats_and_history() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-sync-{suffix}");
    let selected_chat_id = format!("wa-sync-chat-{suffix}");
    let other_chat_id = format!("wa-sync-other-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Sync",
            "external_account_id": format!("wa-sync-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/sync-{suffix}")
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/dialogs",
        json!({
            "account_id": account_id,
            "provider_chat_id": selected_chat_id,
            "chat_title": "Selected WhatsApp Sync Chat",
            "chat_kind": "private",
            "is_archived": false,
            "is_pinned": true,
            "import_batch_id": format!("whatsapp-dialog-{suffix}"),
            "observed_at": "2026-06-06T12:00:00Z"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/dialogs",
        json!({
            "account_id": account_id,
            "provider_chat_id": other_chat_id,
            "chat_title": "Other WhatsApp Sync Chat",
            "chat_kind": "group",
            "is_archived": true,
            "is_pinned": false,
            "import_batch_id": format!("whatsapp-dialog-other-{suffix}"),
            "observed_at": "2026-06-06T12:01:00Z"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": selected_chat_id,
            "provider_message_id": format!("selected-incoming-{suffix}"),
            "chat_title": "Selected WhatsApp Sync Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Selected WhatsApp history message.",
            "import_batch_id": format!("whatsapp-message-{suffix}"),
            "occurred_at": "2026-06-06T12:02:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": other_chat_id,
            "provider_message_id": format!("other-incoming-{suffix}"),
            "chat_title": "Other WhatsApp Sync Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "This other WhatsApp chat must not be returned.",
            "import_batch_id": format!("whatsapp-message-other-{suffix}"),
            "occurred_at": "2026-06-06T12:03:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    let provider_identity_id = format!("wa:+3412345{suffix}");
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/participants",
        json!({
            "account_id": account_id,
            "provider_chat_id": selected_chat_id,
            "provider_member_id": format!("member-{suffix}"),
            "provider_identity_id": provider_identity_id,
            "identity_kind": "whatsapp_phone",
            "address": format!("+3412345{suffix}"),
            "display_name": "Presence Person",
            "push_name": "Owner Push",
            "business_profile": {
                "category": "consulting",
                "description": "Sync contact profile"
            },
            "profile_photo_ref": {
                "provider_file_id": format!("avatar-{suffix}"),
                "sha256": "sync-photo-sha256"
            },
            "role": "member",
            "status": "active",
            "import_batch_id": format!("whatsapp-participant-{suffix}"),
            "observed_at": "2026-06-06T12:04:00Z"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/presence",
        json!({
            "account_id": account_id,
            "provider_chat_id": selected_chat_id,
            "provider_identity_id": provider_identity_id,
            "identity_kind": "whatsapp_phone",
            "display_name": "Presence Person",
            "presence_state": "typing",
            "last_seen_at": "2026-06-06T12:04:30Z",
            "import_batch_id": format!("whatsapp-presence-{suffix}"),
            "observed_at": "2026-06-06T12:05:00Z"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/calls",
        json!({
            "account_id": account_id,
            "provider_call_id": format!("wa-sync-call-{suffix}"),
            "provider_chat_id": selected_chat_id,
            "direction": "incoming",
            "call_state": "missed",
            "started_at": "2026-06-06T12:05:30Z",
            "ended_at": "2026-06-06T12:05:42Z",
            "metadata": {
                "call_kind": "voice",
                "provider_participant_id": provider_identity_id
            },
            "import_batch_id": format!("whatsapp-call-{suffix}"),
            "observed_at": "2026-06-06T12:05:50Z"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/media",
        json!({
            "account_id": account_id,
            "provider_chat_id": selected_chat_id,
            "provider_message_id": format!("selected-incoming-{suffix}"),
            "provider_attachment_id": format!("wa-sync-media-{suffix}"),
            "filename": "selected-sync.png",
            "content_type": "image/png",
            "size_bytes": 32,
            "sha256": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "storage_kind": "local_fs",
            "storage_path": format!("whatsapp/{suffix}/selected-sync.png"),
            "import_batch_id": format!("whatsapp-sync-media-{suffix}"),
            "observed_at": "2026-06-06T12:05:55Z"
        }),
    )
    .await;

    let chat_sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-sync/chats",
            json!({
                "account_id": account_id,
                "limit": 25
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp chat sync response");
    assert_eq!(chat_sync_response.status(), StatusCode::OK);
    let chat_sync_body = json_body(chat_sync_response).await;
    assert_eq!(chat_sync_body["account_id"], json!(account_id));
    assert_eq!(chat_sync_body["runtime_kind"], json!("fixture"));
    assert_eq!(chat_sync_body["status"], json!("synced"));
    assert_eq!(chat_sync_body["synced_count"], json!(2));
    assert_eq!(
        chat_sync_body["items"][0]["provider_chat_id"],
        json!(other_chat_id)
    );

    let history_sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-sync/history",
            json!({
                "account_id": account_id,
                "provider_chat_id": selected_chat_id,
                "limit": 50
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp history sync response");
    assert_eq!(history_sync_response.status(), StatusCode::OK);
    let history_sync_body = json_body(history_sync_response).await;
    assert_eq!(history_sync_body["account_id"], json!(account_id));
    assert_eq!(
        history_sync_body["provider_chat_id"],
        json!(selected_chat_id)
    );
    assert_eq!(history_sync_body["runtime_kind"], json!("fixture"));
    assert_eq!(history_sync_body["status"], json!("synced"));
    assert_eq!(history_sync_body["synced_count"], json!(1));
    assert_eq!(
        history_sync_body["items"][0]["provider_chat_id"],
        json!(selected_chat_id)
    );
    assert_eq!(
        history_sync_body["items"][0]["text"],
        json!("Selected WhatsApp history message.")
    );

    let presence_sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-sync/presence",
            json!({
                "account_id": account_id,
                "provider_chat_id": selected_chat_id,
                "limit": 20
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp presence sync response");
    assert_eq!(presence_sync_response.status(), StatusCode::OK);
    let presence_sync_body = json_body(presence_sync_response).await;
    assert_eq!(presence_sync_body["account_id"], json!(account_id));
    assert_eq!(
        presence_sync_body["provider_chat_id"],
        json!(selected_chat_id)
    );
    assert_eq!(presence_sync_body["runtime_kind"], json!("fixture"));
    assert_eq!(presence_sync_body["status"], json!("synced"));
    assert_eq!(presence_sync_body["synced_count"], json!(1));
    assert_eq!(
        presence_sync_body["items"][0]["provider_identity_id"],
        json!(provider_identity_id)
    );
    assert_eq!(
        presence_sync_body["items"][0]["presence_state"],
        json!("typing")
    );
    assert_eq!(
        presence_sync_body["items"][0]["last_seen_at"],
        json!("2026-06-06T12:04:30Z")
    );

    let calls_sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-sync/calls",
            json!({
                "account_id": account_id,
                "provider_chat_id": selected_chat_id,
                "limit": 20
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp calls sync response");
    assert_eq!(calls_sync_response.status(), StatusCode::OK);
    let calls_sync_body = json_body(calls_sync_response).await;
    assert_eq!(calls_sync_body["account_id"], json!(account_id));
    assert_eq!(calls_sync_body["provider_chat_id"], json!(selected_chat_id));
    assert_eq!(calls_sync_body["runtime_kind"], json!("fixture"));
    assert_eq!(calls_sync_body["status"], json!("synced"));
    assert_eq!(calls_sync_body["synced_count"], json!(1));
    assert_eq!(
        calls_sync_body["items"][0]["provider_call_id"],
        json!(format!("wa-sync-call-{suffix}"))
    );
    assert_eq!(calls_sync_body["items"][0]["call_state"], json!("missed"));
    assert_eq!(
        calls_sync_body["items"][0]["provider_chat_id"],
        json!(selected_chat_id)
    );

    let contacts_sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-sync/contacts",
            json!({
                "account_id": account_id,
                "limit": 20
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp contacts sync response");
    assert_eq!(contacts_sync_response.status(), StatusCode::OK);
    let contacts_sync_body = json_body(contacts_sync_response).await;
    assert_eq!(contacts_sync_body["account_id"], json!(account_id));
    assert_eq!(contacts_sync_body["runtime_kind"], json!("fixture"));
    assert_eq!(contacts_sync_body["status"], json!("synced"));
    assert!(
        contacts_sync_body["items"]
            .as_array()
            .expect("contacts sync items")
            .iter()
            .any(|item| {
                item["provider_identity_id"] == json!(format!("wa:+3412345{suffix}"))
                    && item["display_name"] == json!("Presence Person")
                    && item["push_name"] == json!("Owner Push")
                    && item["business_profile"]["category"] == json!("consulting")
                    && item["profile_photo_ref"]["provider_file_id"]
                        == json!(format!("avatar-{suffix}"))
                    && item["whatsapp_trace_metadata"]["whatsapp_participant_evidence"]["provider_member_id"]
                        == json!(format!("member-{suffix}"))
            }),
        "expected WhatsApp contact snapshot in provider-sync contacts list"
    );

    let media_sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/provider-sync/media",
            json!({
                "account_id": account_id,
                "provider_chat_id": selected_chat_id,
                "content_type": "image/",
                "limit": 20
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("WhatsApp media sync response");
    assert_eq!(media_sync_response.status(), StatusCode::OK);
    let media_sync_body = json_body(media_sync_response).await;
    assert_eq!(media_sync_body["account_id"], json!(account_id));
    assert_eq!(media_sync_body["provider_chat_id"], json!(selected_chat_id));
    assert_eq!(media_sync_body["content_type"], json!("image/"));
    assert_eq!(media_sync_body["runtime_kind"], json!("fixture"));
    assert_eq!(media_sync_body["status"], json!("synced"));
    assert_eq!(media_sync_body["synced_count"], json!(1));
    assert_eq!(
        media_sync_body["items"][0]["provider_attachment_id"],
        json!(format!("wa-sync-media-{suffix}"))
    );
    assert_eq!(
        media_sync_body["items"][0]["provider_message_id"],
        json!(format!("selected-incoming-{suffix}"))
    );
    assert_eq!(
        media_sync_body["items"][0]["storage_path"],
        json!(format!("whatsapp/{suffix}/selected-sync.png"))
    );
    assert_eq!(
        media_sync_body["items"][0]["scan_status"],
        json!("not_scanned")
    );

    let sync_events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE event_type LIKE 'whatsapp.sync.%'
          AND (
              subject->>'id' = $1
              OR subject->>'id' = $2
          )
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .bind(&selected_chat_id)
    .fetch_all(&pool)
    .await
    .expect("WhatsApp sync events");
    assert!(sync_events.iter().any(|(event_type, payload)| {
        event_type == "whatsapp.sync.started" && payload["scope"] == "chats"
    }));
    assert!(sync_events.iter().any(|(event_type, payload)| {
        event_type == "whatsapp.sync.completed" && payload["scope"] == "history"
    }));
    assert!(sync_events.iter().any(|(event_type, payload)| {
        event_type == "whatsapp.sync.completed" && payload["scope"] == "presence"
    }));
    assert!(sync_events.iter().any(|(event_type, payload)| {
        event_type == "whatsapp.sync.completed" && payload["scope"] == "calls"
    }));
    assert!(sync_events.iter().any(|(event_type, payload)| {
        event_type == "whatsapp.sync.completed" && payload["scope"] == "contacts"
    }));
    assert!(sync_events.iter().any(|(event_type, payload)| {
        event_type == "whatsapp.sync.completed" && payload["scope"] == "media"
    }));

    let accepted_runtime_event_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'scope' IN ('chats', 'history', 'presence', 'calls', 'contacts')
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .expect("accepted whatsapp sync runtime-event kinds");
    assert_eq!(
        accepted_runtime_event_kinds,
        vec![
            "sync.chats.started",
            "sync.chats.progress",
            "sync.chats.completed",
            "sync.history.started",
            "sync.history.progress",
            "sync.history.completed",
            "sync.presence.started",
            "sync.presence.progress",
            "sync.presence.completed",
            "sync.calls.started",
            "sync.calls.progress",
            "sync.calls.completed",
            "sync.contacts.started",
            "sync.contacts.progress",
            "sync.contacts.completed",
        ]
    );
}

#[tokio::test]
async fn whatsapp_fixture_reaction_reconciles_provider_command_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-react-{suffix}");
    let provider_chat_id = format!("wa-react-chat-{suffix}");
    let provider_message_id = format!("wa-react-message-{suffix}");
    let command_id = format!("wa-react-command-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Reconcile",
            "external_account_id": format!("wa-react-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/react-{suffix}")
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "chat_title": "WhatsApp Reaction Reconciliation",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Reaction Sender",
            "text": "Reaction reconciliation target.",
            "import_batch_id": format!("whatsapp-react-message-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "react",
        &format!("react:{suffix}"),
        &provider_chat_id,
        Some(&provider_message_id),
        json!({"reaction": "🔥"}),
        json!({"provider_chat_id": provider_chat_id, "provider_message_id": provider_message_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/reactions",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "provider_actor_id": format!("self-{suffix}"),
                "sender_display_name": "Owner",
                "reaction": "🔥",
                "is_active": true,
                "import_batch_id": format!("whatsapp-react-{suffix}"),
                "observed_at": "2026-06-06T12:01:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reaction reconcile response");
    assert_eq!(response.status(), StatusCode::OK);

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled whatsapp reaction command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp reaction reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.fixture_reaction")
    );
}

#[tokio::test]
async fn whatsapp_fixture_unreact_reconciles_provider_command_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-unreact-{suffix}");
    let provider_chat_id = format!("wa-unreact-chat-{suffix}");
    let provider_message_id = format!("wa-unreact-message-{suffix}");
    let command_id = format!("wa-unreact-command-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Unreact Reconcile",
            "external_account_id": format!("wa-unreact-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/unreact-{suffix}")
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "chat_title": "WhatsApp Unreact Reconciliation",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Reaction Sender",
            "text": "Unreact reconciliation target.",
            "import_batch_id": format!("whatsapp-unreact-message-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "unreact",
        &format!("unreact:{suffix}"),
        &provider_chat_id,
        Some(&provider_message_id),
        json!({"reaction_emoji": "🔥"}),
        json!({"provider_chat_id": provider_chat_id, "provider_message_id": provider_message_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/reactions",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "provider_actor_id": format!("self-{suffix}"),
                "sender_display_name": "Owner",
                "reaction": "🔥",
                "is_active": false,
                "import_batch_id": format!("whatsapp-unreact-{suffix}"),
                "observed_at": "2026-06-06T12:01:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("unreact reconcile response");
    assert_eq!(response.status(), StatusCode::OK);

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled whatsapp unreact command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp unreact reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.fixture_reaction")
    );
    assert_eq!(events[1].1["result_payload"]["is_active"], json!(false));
}

#[tokio::test]
async fn whatsapp_fixture_message_delete_reconciles_provider_command_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-delete-{suffix}");
    let provider_chat_id = format!("wa-delete-chat-{suffix}");
    let provider_message_id = format!("wa-delete-message-{suffix}");
    let command_id = format!("wa-delete-command-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Delete Reconcile",
            "external_account_id": format!("wa-delete-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/delete-{suffix}")
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "chat_title": "WhatsApp Delete Reconciliation",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Delete Sender",
            "text": "Delete reconciliation target.",
            "import_batch_id": format!("whatsapp-delete-message-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "delete",
        &format!("delete:{suffix}"),
        &provider_chat_id,
        Some(&provider_message_id),
        json!({"delete_kind": "provider_delete"}),
        json!({"provider_chat_id": provider_chat_id, "provider_message_id": provider_message_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/message-deletes",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reason_class": "deleted_by_provider",
                "actor_class": "provider",
                "import_batch_id": format!("whatsapp-delete-{suffix}"),
                "observed_at": "2026-06-06T12:01:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete reconcile response");
    assert_eq!(response.status(), StatusCode::OK);

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled whatsapp delete command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp delete reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.fixture_message_delete")
    );
    assert_eq!(events[1].1["result_payload"]["deleted"], json!(true));
}

#[tokio::test]
async fn whatsapp_fixture_message_update_reconciles_provider_command_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-edit-{suffix}");
    let provider_chat_id = format!("wa-edit-chat-{suffix}");
    let provider_message_id = format!("wa-edit-message-{suffix}");
    let command_id = format!("wa-edit-command-{suffix}");
    let edited_text = "Edited by observed WhatsApp update";
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Edit Reconcile",
            "external_account_id": format!("wa-edit-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/edit-{suffix}")
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "chat_title": "WhatsApp Edit Reconciliation",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Edit Sender",
            "text": "Original edit reconciliation target.",
            "import_batch_id": format!("whatsapp-edit-message-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "edit",
        &format!("edit:{suffix}"),
        &provider_chat_id,
        Some(&provider_message_id),
        json!({"text": edited_text}),
        json!({"provider_chat_id": provider_chat_id, "provider_message_id": provider_message_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/message-updates",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "text": edited_text,
                "import_batch_id": format!("whatsapp-edit-{suffix}"),
                "observed_at": "2026-06-06T12:01:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("edit reconcile response");
    assert_eq!(response.status(), StatusCode::OK);

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled whatsapp edit command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp edit reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.fixture_message_update")
    );
    assert_eq!(events[1].1["result_payload"]["edited"], json!(true));
    assert_eq!(events[1].1["result_payload"]["text"], json!(edited_text));
}

#[tokio::test]
async fn whatsapp_canonical_confirmed_command_is_normalized_to_queued_on_import() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-canonical-confirmed-{suffix}");
    let chat_id = format!("wa-canonical-confirmed-chat-{suffix}");
    let command_id = format!("wa-canonical-confirmed-command-{suffix}");
    let idempotency_key = format!("wa-canonical-confirmed-send:{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Canonical Confirmed Flow",
                "external_account_id": format!("wa-canonical-confirmed-device-{suffix}"),
                "device_name": "Макошь Canonical Confirmed Fixture",
                "local_state_path": format!("docker/data/whatsapp/canonical-confirmed-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id,
            config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            jsonb_build_object('source_table', 'communication_provider_accounts'),
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(&account_id)
    .execute(&pool)
    .await
    .expect("mirror communication account");

    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO communication_provider_commands (
            command_id, account_id, channel_kind, command_kind, idempotency_key,
            provider_conversation_id, provider_message_id, target_ref, payload,
            capability_state, action_class, confirmation_decision, status,
            retry_count, max_retries, last_error, result_payload, audit_metadata,
            actor_id, happened_at, completed_at, created_at, updated_at
        )
        VALUES (
            $1, $2, 'whatsapp', 'send_text', $3,
            $4, NULL, $5, $6,
            'available', 'provider_write', 'confirmed', 'confirmed',
            0, 3, NULL, '{}'::jsonb, '{}'::jsonb,
            'makosh-frontend', $7, NULL, $7, $7
        )
        "#,
    )
    .bind(&command_id)
    .bind(&account_id)
    .bind(&idempotency_key)
    .bind(&chat_id)
    .bind(json!({"provider_chat_id": chat_id}))
    .bind(json!({"text": format!("Canonical confirmed WhatsApp send {suffix}")}))
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert confirmed canonical provider command");

    trigger_whatsapp_runtime_bridge_import(app.clone(), &account_id).await;

    let whatsapp_status: String = sqlx::query_scalar(
        "SELECT status FROM whatsapp_provider_write_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("whatsapp imported command status");
    assert_eq!(whatsapp_status, "queued");

    let canonical_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("canonical normalized command status");
    assert_eq!(canonical_status, "queued");
}

#[tokio::test]
async fn whatsapp_importer_syncs_confirmed_canonical_update_into_existing_pending_outbox_command() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-canonical-sync-{suffix}");
    let chat_id = format!("wa-canonical-sync-chat-{suffix}");
    let command_id = format!("wa-canonical-sync-command-{suffix}");
    let idempotency_key = format!("wa-canonical-sync-send:{suffix}");
    let message_text = format!("Canonical sync WhatsApp send {suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Canonical Sync Flow",
                "external_account_id": format!("wa-canonical-sync-device-{suffix}"),
                "device_name": "Макошь Canonical Sync Fixture",
                "local_state_path": format!("docker/data/whatsapp/canonical-sync-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id,
            config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            jsonb_build_object('source_table', 'communication_provider_accounts'),
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(&account_id)
    .execute(&pool)
    .await
    .expect("mirror communication account");

    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO communication_provider_commands (
            command_id, account_id, channel_kind, command_kind, idempotency_key,
            provider_conversation_id, provider_message_id, target_ref, payload,
            capability_state, action_class, confirmation_decision, status,
            retry_count, max_retries, last_error, result_payload, audit_metadata,
            actor_id, happened_at, completed_at, created_at, updated_at
        )
        VALUES (
            $1, $2, 'whatsapp', 'send_text', $3,
            $4, NULL, $5, $6,
            'available', 'provider_write', 'pending', 'queued',
            0, 3, NULL, '{}'::jsonb, '{}'::jsonb,
            'makosh-frontend', $7, NULL, $7, $7
        )
        "#,
    )
    .bind(&command_id)
    .bind(&account_id)
    .bind(&idempotency_key)
    .bind(&chat_id)
    .bind(json!({"provider_chat_id": chat_id}))
    .bind(json!({"text": message_text}))
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert pending canonical provider command");

    trigger_whatsapp_runtime_bridge_import(app.clone(), &account_id).await;
    let pending_row = sqlx::query(
        r#"
        SELECT confirmation_decision, status
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("pending WhatsApp outbox row");
    assert_eq!(
        pending_row
            .try_get::<String, _>("confirmation_decision")
            .expect("confirmation_decision"),
        "pending"
    );
    assert_eq!(
        pending_row.try_get::<String, _>("status").expect("status"),
        "queued"
    );

    sqlx::query(
        r#"
        UPDATE communication_provider_commands
        SET confirmation_decision = 'confirmed',
            updated_at = NOW()
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .execute(&pool)
    .await
    .expect("confirm canonical provider command");

    trigger_whatsapp_runtime_bridge_import(app.clone(), &account_id).await;
    let canonical_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("canonical synced command status");
    assert_eq!(canonical_status, "queued");

    let whatsapp_row = sqlx::query(
        r#"
        SELECT confirmation_decision, status, result_payload
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("synced whatsapp outbox row");
    assert_eq!(
        whatsapp_row
            .try_get::<String, _>("confirmation_decision")
            .expect("whatsapp confirmation_decision"),
        "confirmed"
    );
    assert_eq!(
        whatsapp_row
            .try_get::<String, _>("status")
            .expect("whatsapp status"),
        "queued"
    );
    let whatsapp_result_payload: Value = whatsapp_row
        .try_get("result_payload")
        .expect("whatsapp result_payload");
    assert!(whatsapp_result_payload["observed_via"].is_null());
}

#[tokio::test]
async fn whatsapp_importer_syncs_canonical_target_ids_into_existing_pending_outbox_command() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-canonical-target-sync-{suffix}");
    let original_chat_id = format!("wa-canonical-target-sync-old-chat-{suffix}");
    let replacement_chat_id = format!("wa-canonical-target-sync-new-chat-{suffix}");
    let original_message_id = format!("wa-canonical-target-sync-old-message-{suffix}");
    let replacement_message_id = format!("wa-canonical-target-sync-new-message-{suffix}");
    let command_id = format!("wa-canonical-target-sync-command-{suffix}");
    let idempotency_key = format!("wa-canonical-target-sync-edit:{suffix}");
    let original_text = format!("Original WhatsApp text {suffix}");
    let replacement_text = format!("Replacement WhatsApp text {suffix}");
    let edited_text = format!("Edited through canonical target sync {suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "display_name": "WhatsApp Canonical Target Sync Flow",
                "external_account_id": format!("wa-canonical-target-sync-device-{suffix}"),
                "device_name": "Макошь Canonical Target Sync Fixture",
                "local_state_path": format!("docker/data/whatsapp/canonical-target-sync-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": original_chat_id,
            "provider_message_id": original_message_id,
            "chat_title": "WhatsApp Canonical Target Sync Original",
            "sender_id": format!("sender-old-{suffix}"),
            "sender_display_name": "Original Sender",
            "text": original_text,
            "import_batch_id": format!("wa-canonical-target-sync-original-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": replacement_chat_id,
            "provider_message_id": replacement_message_id,
            "chat_title": "WhatsApp Canonical Target Sync Replacement",
            "sender_id": format!("sender-new-{suffix}"),
            "sender_display_name": "Replacement Sender",
            "text": replacement_text,
            "import_batch_id": format!("wa-canonical-target-sync-replacement-{suffix}"),
            "occurred_at": "2026-06-06T12:01:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id,
            config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            jsonb_build_object('source_table', 'communication_provider_accounts'),
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(&account_id)
    .execute(&pool)
    .await
    .expect("mirror communication account");

    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO communication_provider_commands (
            command_id, account_id, channel_kind, command_kind, idempotency_key,
            provider_conversation_id, provider_message_id, target_ref, payload,
            capability_state, action_class, confirmation_decision, status,
            retry_count, max_retries, last_error, result_payload, audit_metadata,
            actor_id, happened_at, completed_at, created_at, updated_at
        )
        VALUES (
            $1, $2, 'whatsapp', 'edit', $3,
            $4, $5, $6, $7,
            'available', 'provider_write', 'pending', 'queued',
            0, 3, NULL, '{}'::jsonb, '{}'::jsonb,
            'makosh-frontend', $8, NULL, $8, $8
        )
        "#,
    )
    .bind(&command_id)
    .bind(&account_id)
    .bind(&idempotency_key)
    .bind(&original_chat_id)
    .bind(&original_message_id)
    .bind(json!({
        "provider_chat_id": original_chat_id,
        "provider_message_id": original_message_id
    }))
    .bind(json!({
        "provider_chat_id": original_chat_id,
        "provider_message_id": original_message_id,
        "text": edited_text
    }))
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert pending canonical edit command");

    trigger_whatsapp_runtime_bridge_import(app.clone(), &account_id).await;
    let pending_row = sqlx::query(
        r#"
        SELECT provider_chat_id, provider_message_id, confirmation_decision, status
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("pending WhatsApp outbox row");
    assert_eq!(
        pending_row
            .try_get::<String, _>("provider_chat_id")
            .expect("provider_chat_id"),
        original_chat_id
    );
    assert_eq!(
        pending_row
            .try_get::<Option<String>, _>("provider_message_id")
            .expect("provider_message_id")
            .as_deref(),
        Some(original_message_id.as_str())
    );
    assert_eq!(
        pending_row
            .try_get::<String, _>("confirmation_decision")
            .expect("confirmation_decision"),
        "pending"
    );
    assert_eq!(
        pending_row.try_get::<String, _>("status").expect("status"),
        "queued"
    );

    sqlx::query(
        r#"
        UPDATE communication_provider_commands
        SET provider_conversation_id = $2,
            provider_message_id = $3,
            target_ref = $4,
            confirmation_decision = 'confirmed',
            updated_at = NOW()
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .bind(&replacement_chat_id)
    .bind(&replacement_message_id)
    .bind(json!({
        "provider_chat_id": replacement_chat_id,
        "provider_message_id": replacement_message_id
    }))
    .execute(&pool)
    .await
    .expect("retarget canonical edit command");

    trigger_whatsapp_runtime_bridge_import(app.clone(), &account_id).await;
    let canonical_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("canonical retargeted command status");
    assert_eq!(canonical_status, "queued");

    let whatsapp_row = sqlx::query(
        r#"
        SELECT provider_chat_id, provider_message_id, confirmation_decision, status
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("synced whatsapp retargeted row");
    assert_eq!(
        whatsapp_row
            .try_get::<String, _>("provider_chat_id")
            .expect("whatsapp provider_chat_id"),
        replacement_chat_id
    );
    assert_eq!(
        whatsapp_row
            .try_get::<Option<String>, _>("provider_message_id")
            .expect("whatsapp provider_message_id")
            .as_deref(),
        Some(replacement_message_id.as_str())
    );
    assert_eq!(
        whatsapp_row
            .try_get::<String, _>("confirmation_decision")
            .expect("whatsapp confirmation_decision"),
        "confirmed"
    );
    assert_eq!(
        whatsapp_row
            .try_get::<String, _>("status")
            .expect("whatsapp status"),
        "queued"
    );

    let original_body_text: String = sqlx::query_scalar(
        r#"
        SELECT body_text
        FROM communication_messages
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&original_message_id)
    .fetch_one(&pool)
    .await
    .expect("original message body text");
    assert_eq!(original_body_text, original_text);

    let replacement_body_text: String = sqlx::query_scalar(
        r#"
        SELECT body_text
        FROM communication_messages
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&replacement_message_id)
    .fetch_one(&pool)
    .await
    .expect("replacement message body text");
    assert_eq!(replacement_body_text, replacement_text);
}

#[tokio::test]
async fn whatsapp_fixture_receipt_projects_source_record_and_emits_realtime_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-receipt-source-{suffix}");
    let provider_chat_id = format!("wa-receipt-chat-{suffix}");
    let provider_message_id = format!("wa-receipt-message-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Receipt Source",
            "external_account_id": format!("wa-receipt-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/receipt-{suffix}")
        }),
    )
    .await;
    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "WhatsApp Receipt Source",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Receipt Sender",
                "text": "Receipt source record target.",
                "import_batch_id": format!("whatsapp-receipt-message-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

    let receipt_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/receipts",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "delivery_state": "sent",
                "import_batch_id": format!("whatsapp-receipt-{suffix}"),
                "observed_at": "2026-06-06T12:01:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("receipt response");
    assert_eq!(receipt_response.status(), StatusCode::OK);
    let receipt_body = json_body(receipt_response).await;
    assert_eq!(receipt_body["message_id"], json!(message_id));

    let delivery_state: String = sqlx::query_scalar(
        "SELECT delivery_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("projected receipt delivery_state");
    assert_eq!(delivery_state, "sent");

    let signal_counts: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT event_type, COUNT(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.whatsapp.receipt.observed',
            'signal.accepted.whatsapp.receipt',
            'whatsapp.receipt.changed'
        )
          AND payload->>'provider_message_id' = $1
        GROUP BY event_type
        ORDER BY event_type
        "#,
    )
    .bind(&provider_message_id)
    .fetch_all(&pool)
    .await
    .expect("receipt signal counts");
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.raw.whatsapp.receipt.observed" && *count == 1
    }));
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.accepted.whatsapp.receipt" && *count == 1
    }));
    assert!(
        signal_counts
            .iter()
            .any(|(event_type, count)| { event_type == "whatsapp.receipt.changed" && *count == 1 })
    );

    let receipt_changed_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.receipt.changed'
          AND payload->>'provider_message_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&provider_message_id)
    .fetch_one(&pool)
    .await
    .expect("whatsapp receipt changed payload");
    assert_eq!(receipt_changed_payload["message_id"], json!(message_id));
    assert_eq!(receipt_changed_payload["delivery_state"], json!("sent"));
}

#[tokio::test]
async fn whatsapp_fixture_presence_projects_source_record_and_emits_realtime_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-presence-source-{suffix}");
    let provider_chat_id = format!("wa-presence-chat-{suffix}");
    let provider_identity_id = format!("wa:+3412345{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Presence Source",
            "external_account_id": format!("wa-presence-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/presence-{suffix}")
        }),
    )
    .await;
    let participant_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/participants",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "chat_title": "WhatsApp Presence Source",
                "chat_kind": "group",
                "provider_member_id": format!("member-{suffix}"),
                "provider_identity_id": provider_identity_id,
                "identity_kind": "whatsapp_phone",
                "display_name": "Presence Member",
                "push_name": "Presence Push",
                "address": format!("+3412345{suffix}"),
                "business_profile": {
                    "category": "consulting"
                },
                "profile_photo_ref": {
                    "provider_file_id": format!("avatar-{suffix}")
                },
                "role": "member",
                "status": "member",
                "is_self": false,
                "is_admin": false,
                "is_owner": false,
                "import_batch_id": format!("whatsapp-presence-participant-{suffix}"),
                "observed_at": "2026-06-06T12:00:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("participant response");
    assert_eq!(participant_response.status(), StatusCode::OK);
    let participant_body = json_body(participant_response).await;
    let identity_id = participant_body["identity_id"]
        .as_str()
        .expect("identity id")
        .to_owned();

    let presence_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/presence",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_identity_id": provider_identity_id,
                "identity_kind": "whatsapp_phone",
                "display_name": "Presence Member",
                "presence_state": "typing",
                "last_seen_at": "2026-06-06T11:59:00Z",
                "import_batch_id": format!("whatsapp-presence-{suffix}"),
                "observed_at": "2026-06-06T12:01:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("presence response");
    assert_eq!(presence_response.status(), StatusCode::OK);
    let presence_body = json_body(presence_response).await;
    assert_eq!(presence_body["identity_id"], json!(identity_id));

    let identity_metadata: Value =
        sqlx::query_scalar("SELECT metadata FROM communication_identities WHERE identity_id = $1")
            .bind(&identity_id)
            .fetch_one(&pool)
            .await
            .expect("presence identity metadata");
    assert_eq!(identity_metadata["presence_state"], json!("typing"));
    assert_eq!(
        identity_metadata["presence_provider_chat_id"],
        json!(provider_chat_id)
    );
    assert_eq!(
        identity_metadata["presence_observed_at"],
        json!("2026-06-06T12:01:00Z")
    );
    assert_eq!(
        identity_metadata["last_seen_at"],
        json!("2026-06-06T11:59:00Z")
    );

    let signal_counts: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT event_type, COUNT(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.whatsapp.presence.observed',
            'signal.accepted.whatsapp.presence',
            'whatsapp.presence.changed'
        )
          AND payload->>'provider_identity_id' = $1
        GROUP BY event_type
        ORDER BY event_type
        "#,
    )
    .bind(&provider_identity_id)
    .fetch_all(&pool)
    .await
    .expect("presence signal counts");
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.raw.whatsapp.presence.observed" && *count == 1
    }));
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.accepted.whatsapp.presence" && *count == 1
    }));
    assert!(
        signal_counts.iter().any(|(event_type, count)| {
            event_type == "whatsapp.presence.changed" && *count == 1
        })
    );

    let presence_changed_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.presence.changed'
          AND payload->>'provider_identity_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&provider_identity_id)
    .fetch_one(&pool)
    .await
    .expect("whatsapp presence changed payload");
    assert_eq!(presence_changed_payload["identity_id"], json!(identity_id));
    assert_eq!(
        presence_changed_payload["provider_chat_id"],
        json!(provider_chat_id)
    );
    assert_eq!(presence_changed_payload["presence_state"], json!("typing"));
    assert_eq!(
        presence_changed_payload["last_seen_at"],
        json!("2026-06-06T11:59:00Z")
    );
}

#[tokio::test]
async fn whatsapp_fixture_call_projects_source_record_and_emits_realtime_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-call-source-{suffix}");
    let provider_chat_id = format!("wa-call-chat-{suffix}");
    let provider_call_id = format!("wa-call-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Call Source",
            "external_account_id": format!("wa-call-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/call-{suffix}")
        }),
    )
    .await;

    let call_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/calls",
            json!({
                "account_id": account_id,
                "provider_call_id": provider_call_id,
                "provider_chat_id": provider_chat_id,
                "direction": "incoming",
                "call_state": "missed",
                "started_at": "2026-06-06T12:57:00Z",
                "ended_at": "2026-06-06T12:57:12Z",
                "metadata": {
                    "call_kind": "voice",
                    "provider_participant_id": format!("wa:+3412345{suffix}")
                },
                "import_batch_id": format!("whatsapp-call-{suffix}"),
                "observed_at": "2026-06-06T12:59:50Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("call response");
    assert_eq!(call_response.status(), StatusCode::OK);
    let call_body = json_body(call_response).await;
    let call_id = call_body["call_id"].as_str().expect("call id").to_owned();

    let call_row: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT call_id, provider_call_id, provider_chat_id, call_state
        FROM telegram_calls
        WHERE call_id = $1
        "#,
    )
    .bind(&call_id)
    .fetch_one(&pool)
    .await
    .expect("projected whatsapp call row");
    assert_eq!(call_row.0, call_id);
    assert_eq!(call_row.1, provider_call_id);
    assert_eq!(call_row.2, provider_chat_id);
    assert_eq!(call_row.3, "missed");

    let signal_counts: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT event_type, COUNT(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.whatsapp.call_metadata.observed',
            'signal.accepted.whatsapp.call_metadata',
            'whatsapp.call.updated'
        )
          AND payload->>'provider_call_id' = $1
        GROUP BY event_type
        ORDER BY event_type
        "#,
    )
    .bind(&provider_call_id)
    .fetch_all(&pool)
    .await
    .expect("call signal counts");
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.raw.whatsapp.call_metadata.observed" && *count == 1
    }));
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.accepted.whatsapp.call_metadata" && *count == 1
    }));
    assert!(
        signal_counts
            .iter()
            .any(|(event_type, count)| { event_type == "whatsapp.call.updated" && *count == 1 })
    );

    let call_updated_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.call.updated'
          AND payload->>'provider_call_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&provider_call_id)
    .fetch_one(&pool)
    .await
    .expect("whatsapp call updated payload");
    assert_eq!(call_updated_payload["call_id"], json!(call_id));
    assert_eq!(
        call_updated_payload["provider_chat_id"],
        json!(provider_chat_id)
    );
    assert_eq!(call_updated_payload["call_state"], json!("missed"));
}

#[tokio::test]
async fn whatsapp_fixture_runtime_event_is_captured_as_signal_and_sanitized_realtime_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-event-{suffix}");
    let provider_event_id = format!("wa-runtime-event-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Runtime Event Source",
            "external_account_id": format!("wa-runtime-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-event-{suffix}")
        }),
    )
    .await;

    let runtime_event_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/runtime-events",
            json!({
                "account_id": account_id,
                "provider_event_id": provider_event_id,
                "runtime_event_kind": "connection.degraded",
                "runtime_status": "degraded",
                "lifecycle_state": "degraded",
                "severity": "warning",
                "metadata": {
                    "error_code": "EPIPE",
                    "attempt": 2,
                    "session_key": format!("wa-session-secret-{suffix}"),
                    "nested": {
                        "refresh_token": format!("wa-refresh-token-{suffix}")
                    }
                },
                "import_batch_id": format!("whatsapp-runtime-event-{suffix}"),
                "observed_at": "2026-06-06T13:05:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime event response");
    assert_eq!(runtime_event_response.status(), StatusCode::OK);
    let runtime_event_body = json_body(runtime_event_response).await;
    let raw_record_id = runtime_event_body["raw_record_id"]
        .as_str()
        .expect("raw record id")
        .to_owned();

    let raw_signal_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'signal.raw.whatsapp.runtime_event.observed'
          AND subject->>'raw_record_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&raw_record_id)
    .fetch_one(&pool)
    .await
    .expect("whatsapp runtime-event raw payload");
    assert_eq!(
        raw_signal_payload["provider_event_id"],
        json!(provider_event_id)
    );
    assert_eq!(
        raw_signal_payload["runtime_event_kind"],
        json!("connection.degraded")
    );
    assert_eq!(
        raw_signal_payload["metadata"]["session_key"],
        json!("[redacted]")
    );
    assert_eq!(
        raw_signal_payload["metadata"]["nested"]["refresh_token"],
        json!("[redacted]")
    );

    let signal_counts: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT event_type, COUNT(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.whatsapp.runtime_event.observed',
            'signal.accepted.whatsapp.runtime_event',
            'whatsapp.runtime.event'
        )
          AND (
                subject->>'raw_record_id' = $1
                OR payload->>'provider_event_id' = $2
              )
        GROUP BY event_type
        ORDER BY event_type
        "#,
    )
    .bind(&raw_record_id)
    .bind(&provider_event_id)
    .fetch_all(&pool)
    .await
    .expect("runtime-event signal counts");
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.raw.whatsapp.runtime_event.observed" && *count == 1
    }));
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.accepted.whatsapp.runtime_event" && *count == 1
    }));
    assert!(
        signal_counts
            .iter()
            .any(|(event_type, count)| { event_type == "whatsapp.runtime.event" && *count == 1 })
    );

    let runtime_event_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.runtime.event'
          AND payload->>'provider_event_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&provider_event_id)
    .fetch_one(&pool)
    .await
    .expect("whatsapp runtime-event payload");
    assert_eq!(runtime_event_payload["runtime_status"], json!("degraded"));
    assert_eq!(runtime_event_payload["lifecycle_state"], json!("degraded"));
    assert_eq!(runtime_event_payload["severity"], json!("warning"));
    assert_eq!(
        runtime_event_payload["metadata_keys"],
        json!(["attempt", "error_code", "nested", "session_key"])
    );
    assert!(runtime_event_payload.get("metadata").is_none());
    assert!(runtime_event_payload.get("session_key").is_none());
}

#[tokio::test]
async fn whatsapp_unknown_runtime_event_defaults_to_degraded_warning_markers() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-unknown-{suffix}");
    let provider_event_id = format!("wa-runtime-unknown-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Unknown Runtime Event Source",
            "external_account_id": format!("wa-runtime-unknown-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/runtime-unknown-{suffix}")
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/runtime-events",
            json!({
                "account_id": account_id,
                "provider_event_id": provider_event_id,
                "runtime_event_kind": "provider.unknown_patch_event",
                "metadata": {
                    "unsupported_opcode": 9001
                },
                "import_batch_id": format!("whatsapp-runtime-unknown-{suffix}"),
                "observed_at": "2026-06-06T13:06:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("unknown runtime event response");
    assert_eq!(response.status(), StatusCode::OK);

    let raw_signal_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'signal.raw.whatsapp.runtime_event.observed'
          AND payload->>'provider_event_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&provider_event_id)
    .fetch_one(&pool)
    .await
    .expect("unknown runtime-event raw payload");
    assert_eq!(raw_signal_payload["runtime_status"], json!("degraded"));
    assert_eq!(raw_signal_payload["lifecycle_state"], json!("degraded"));
    assert_eq!(raw_signal_payload["severity"], json!("warning"));

    let realtime_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.runtime.event'
          AND payload->>'provider_event_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&provider_event_id)
    .fetch_one(&pool)
    .await
    .expect("unknown runtime-event realtime payload");
    assert_eq!(realtime_payload["runtime_status"], json!("degraded"));
    assert_eq!(realtime_payload["lifecycle_state"], json!("degraded"));
    assert_eq!(realtime_payload["severity"], json!("warning"));
}

#[tokio::test]
async fn whatsapp_fixture_status_view_and_delete_project_source_records_and_emit_realtime_events() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-status-source-{suffix}");
    let provider_status_id = format!("wa-status-{suffix}");
    let viewer_id = format!("viewer-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Status Source",
            "external_account_id": format!("wa-status-source-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/status-{suffix}")
        }),
    )
    .await;

    let status_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/statuses",
            json!({
                "account_id": account_id,
                "provider_status_id": provider_status_id,
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "WhatsApp Status Fixture",
                "text": "Status source-record fixture",
                "import_batch_id": format!("whatsapp-status-source-{suffix}"),
                "occurred_at": "2026-06-06T13:03:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status response");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_body = json_body(status_response).await;
    let message_id = status_body["message_id"]
        .as_str()
        .expect("status message id")
        .to_owned();

    let status_view_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/status-views",
            json!({
                "account_id": account_id,
                "provider_status_id": provider_status_id,
                "viewer_id": viewer_id,
                "viewer_display_name": "Status Viewer",
                "import_batch_id": format!("whatsapp-status-view-{suffix}"),
                "observed_at": "2026-06-06T13:04:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status view response");
    assert_eq!(status_view_response.status(), StatusCode::OK);
    let status_view_body = json_body(status_view_response).await;
    assert_eq!(status_view_body["message_id"], json!(message_id));

    let status_delete_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/status-deletes",
            json!({
                "account_id": account_id,
                "provider_status_id": provider_status_id,
                "actor_class": "self",
                "reason_class": "status_expired",
                "import_batch_id": format!("whatsapp-status-delete-{suffix}"),
                "observed_at": "2026-06-06T13:05:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status delete response");
    assert_eq!(status_delete_response.status(), StatusCode::OK);
    let status_delete_body = json_body(status_delete_response).await;
    assert_eq!(status_delete_body["message_id"], json!(message_id));
    let tombstone_id = status_delete_body["tombstone_id"]
        .as_str()
        .expect("tombstone id")
        .to_owned();

    let status_metadata: Value = sqlx::query_scalar(
        "SELECT message_metadata FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("status metadata");
    assert_eq!(status_metadata["status_viewed"], json!(true));
    assert_eq!(status_metadata["status_view_count"], json!(1));
    assert_eq!(status_metadata["status_last_viewer_id"], json!(viewer_id));
    assert_eq!(status_metadata["status_deleted"], json!(true));
    assert_eq!(
        status_metadata["status_delete_reason_class"],
        json!("status_expired")
    );

    let tombstone_row: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT tombstone_id, message_id, actor_class, reason_class
        FROM communication_message_tombstones
        WHERE tombstone_id = $1
        "#,
    )
    .bind(&tombstone_id)
    .fetch_one(&pool)
    .await
    .expect("status tombstone row");
    assert_eq!(tombstone_row.0, tombstone_id);
    assert_eq!(tombstone_row.1, message_id);
    assert_eq!(tombstone_row.2, "self");
    assert_eq!(tombstone_row.3, "status_expired");

    let signal_counts: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT event_type, COUNT(*)::bigint
        FROM event_log
        WHERE event_type IN (
            'signal.raw.whatsapp.status.observed',
            'signal.accepted.whatsapp.status',
            'signal.raw.whatsapp.status_view.observed',
            'signal.accepted.whatsapp.status_view',
            'signal.raw.whatsapp.status_delete.observed',
            'signal.accepted.whatsapp.status_delete',
            'whatsapp.status.updated',
            'whatsapp.status.deleted'
        )
          AND payload->>'provider_status_id' = $1
        GROUP BY event_type
        ORDER BY event_type
        "#,
    )
    .bind(&provider_status_id)
    .fetch_all(&pool)
    .await
    .expect("status signal counts");
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.raw.whatsapp.status.observed" && *count == 1
    }));
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.accepted.whatsapp.status" && *count == 1
    }));
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.raw.whatsapp.status_view.observed" && *count == 1
    }));
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.accepted.whatsapp.status_view" && *count == 1
    }));
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.raw.whatsapp.status_delete.observed" && *count == 1
    }));
    assert!(signal_counts.iter().any(|(event_type, count)| {
        event_type == "signal.accepted.whatsapp.status_delete" && *count == 1
    }));
    assert!(
        signal_counts
            .iter()
            .any(|(event_type, count)| { event_type == "whatsapp.status.updated" && *count == 2 })
    );
    assert!(
        signal_counts
            .iter()
            .any(|(event_type, count)| { event_type == "whatsapp.status.deleted" && *count == 1 })
    );

    let status_updated_payloads: Vec<Value> = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.status.updated'
          AND payload->>'provider_status_id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(&provider_status_id)
    .fetch_all(&pool)
    .await
    .expect("status updated payloads");
    assert_eq!(status_updated_payloads.len(), 2);
    assert_eq!(status_updated_payloads[0]["message_id"], json!(message_id));
    assert_eq!(status_updated_payloads[0]["status_state"], json!("posted"));
    assert_eq!(status_updated_payloads[1]["message_id"], json!(message_id));
    assert_eq!(status_updated_payloads[1]["status_state"], json!("viewed"));
    assert_eq!(status_updated_payloads[1]["viewer_id"], json!(viewer_id));

    let status_deleted_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'whatsapp.status.deleted'
          AND payload->>'provider_status_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&provider_status_id)
    .fetch_one(&pool)
    .await
    .expect("status deleted payload");
    assert_eq!(status_deleted_payload["message_id"], json!(message_id));
    assert_eq!(status_deleted_payload["status_state"], json!("deleted"));
    assert_eq!(status_deleted_payload["tombstone_id"], json!(tombstone_id));
}

#[tokio::test]
async fn whatsapp_fixture_status_reconciles_publish_status_command_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-status-{suffix}");
    let command_id = format!("wa-status-command-{suffix}");
    let published_text = format!("Status by observed reconciliation {suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Status Reconcile",
            "external_account_id": format!("wa-status-reconcile-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/status-reconcile-{suffix}")
        }),
    )
    .await;

    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "publish_status",
        &format!("publish-status:{suffix}"),
        "status-feed",
        None,
        json!({"text": published_text}),
        json!({"surface": "status_feed"}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/statuses",
            json!({
                "account_id": account_id,
                "provider_status_id": format!("provider-status:{command_id}"),
                "sender_id": account_id,
                "sender_display_name": "Макошь Owner",
                "text": published_text,
                "import_batch_id": format!("whatsapp-status-reconcile-{suffix}"),
                "occurred_at": "2026-06-06T12:10:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("status reconcile response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let message_id = body["message_id"].as_str().expect("message id").to_owned();

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled whatsapp publish status command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let mirrored_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("mirrored communication provider command status");
    assert_eq!(mirrored_status, "completed");

    let stored_message: (String, String) = sqlx::query_as(
        r#"
        SELECT provider_record_id, body_text
        FROM communication_messages
        WHERE message_id = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("stored status message");
    assert_eq!(stored_message.0, format!("provider-status:{command_id}"));
    assert_eq!(stored_message.1, published_text);

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp status reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.fixture_status")
    );
    assert_eq!(
        events[1].1["result_payload"]["provider_status_id"],
        json!(format!("provider-status:{command_id}"))
    );
    let accepted_runtime_event_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'command_id' = $2
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("accepted whatsapp publish status reconcile runtime-event kinds");
    assert_eq!(
        accepted_runtime_event_kinds,
        vec!["status.publish.completed"]
    );
}

#[tokio::test]
async fn whatsapp_fixture_media_reconciles_send_media_command_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-media-{suffix}");
    let provider_chat_id = format!("wa-media-chat-{suffix}");
    let command_id = format!("wa-media-command-{suffix}");
    let provider_message_id = format!("provider-message:{command_id}");
    let provider_attachment_id = format!("provider-attachment:{command_id}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Media Reconcile",
            "external_account_id": format!("wa-media-reconcile-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/media-reconcile-{suffix}")
        }),
    )
    .await;

    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "send_media",
        &format!("send-media:{suffix}"),
        &provider_chat_id,
        None,
        json!({
            "blob_id": format!("whatsapp/{suffix}/fixture.txt"),
            "media_type": "document",
            "filename": "fixture.txt"
        }),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "Media Reconciliation",
                "sender_id": account_id,
                "sender_display_name": "Макошь Owner",
                "text": "Observed media message",
                "import_batch_id": format!("whatsapp-media-message-{suffix}"),
                "occurred_at": "2026-06-06T12:09:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("media message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

    let media_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/media",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "provider_attachment_id": provider_attachment_id,
                "filename": "fixture.txt",
                "content_type": "text/plain",
                "size_bytes": 128,
                "sha256": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "storage_kind": "local_fs",
                "storage_path": format!("whatsapp/{suffix}/fixture.txt"),
                "import_batch_id": format!("whatsapp-media-reconcile-{suffix}"),
                "observed_at": "2026-06-06T12:10:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("media reconcile response");
    assert_eq!(media_response.status(), StatusCode::OK);
    let media_body = json_body(media_response).await;
    let attachment_id = media_body["attachment_id"]
        .as_str()
        .expect("attachment id")
        .to_owned();
    assert_eq!(media_body["message_id"], json!(message_id));

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled whatsapp send media command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let mirrored_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("mirrored communication provider command status");
    assert_eq!(mirrored_status, "completed");

    let attachment_row = sqlx::query(
        r#"
        SELECT message_id, provider_attachment_id, content_type, filename
        FROM communication_attachments
        WHERE attachment_id = $1
        "#,
    )
    .bind(&attachment_id)
    .fetch_one(&pool)
    .await
    .expect("projected media attachment row");
    assert_eq!(
        attachment_row
            .try_get::<String, _>("message_id")
            .expect("message id"),
        message_id
    );
    assert_eq!(
        attachment_row
            .try_get::<String, _>("provider_attachment_id")
            .expect("provider attachment id"),
        provider_attachment_id
    );
    assert_eq!(
        attachment_row
            .try_get::<String, _>("content_type")
            .expect("content type"),
        "text/plain"
    );
    assert_eq!(
        attachment_row
            .try_get::<Option<String>, _>("filename")
            .expect("filename")
            .as_deref(),
        Some("fixture.txt")
    );

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp media reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.fixture_media")
    );
    assert_eq!(
        events[1].1["result_payload"]["provider_attachment_id"],
        json!(provider_attachment_id)
    );
    assert_eq!(
        events[1].1["result_payload"]["provider_message_id"],
        json!(provider_message_id)
    );
}

#[tokio::test]
async fn whatsapp_fixture_media_reconciles_download_media_command_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-download-{suffix}");
    let provider_chat_id = format!("wa-download-chat-{suffix}");
    let command_id = format!("wa-download-command-{suffix}");
    let provider_message_id = format!("wa-source-message-{suffix}");
    let provider_attachment_id = format!("wa-download-attachment-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Download Reconcile",
            "external_account_id": format!("wa-download-reconcile-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/download-reconcile-{suffix}")
        }),
    )
    .await;

    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "download_media",
        &format!("download-media:{suffix}"),
        &provider_chat_id,
        Some(&provider_message_id),
        json!({
            "provider_attachment_id": provider_attachment_id,
            "filename": "fixture.pdf",
            "content_type": "application/pdf"
        }),
        json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id
        }),
    )
    .await;

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "Download Reconciliation",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Observed Sender",
                "text": "Observed downloadable media message",
                "import_batch_id": format!("whatsapp-download-message-{suffix}"),
                "occurred_at": "2026-06-06T12:09:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("download message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

    let media_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/media",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "provider_attachment_id": provider_attachment_id,
                "filename": "fixture.pdf",
                "content_type": "application/pdf",
                "size_bytes": 2048,
                "sha256": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
                "storage_kind": "local_fs",
                "storage_path": format!("whatsapp/{suffix}/fixture.pdf"),
                "import_batch_id": format!("whatsapp-download-reconcile-{suffix}"),
                "observed_at": "2026-06-06T12:10:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("download media reconcile response");
    assert_eq!(media_response.status(), StatusCode::OK);
    let media_body = json_body(media_response).await;
    let attachment_id = media_body["attachment_id"]
        .as_str()
        .expect("attachment id")
        .to_owned();
    assert_eq!(media_body["message_id"], json!(message_id));

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled whatsapp download media command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let mirrored_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("mirrored communication provider command status");
    assert_eq!(mirrored_status, "completed");

    let attachment_row = sqlx::query(
        r#"
        SELECT message_id, provider_attachment_id, content_type, filename
        FROM communication_attachments
        WHERE attachment_id = $1
        "#,
    )
    .bind(&attachment_id)
    .fetch_one(&pool)
    .await
    .expect("projected downloaded media attachment row");
    assert_eq!(
        attachment_row
            .try_get::<String, _>("message_id")
            .expect("message id"),
        message_id
    );
    assert_eq!(
        attachment_row
            .try_get::<String, _>("provider_attachment_id")
            .expect("provider attachment id"),
        provider_attachment_id
    );
    assert_eq!(
        attachment_row
            .try_get::<String, _>("content_type")
            .expect("content type"),
        "application/pdf"
    );
    assert_eq!(
        attachment_row
            .try_get::<Option<String>, _>("filename")
            .expect("filename")
            .as_deref(),
        Some("fixture.pdf")
    );

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp download media reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.fixture_media")
    );
    assert_eq!(
        events[1].1["result_payload"]["provider_attachment_id"],
        json!(provider_attachment_id)
    );
    assert_eq!(
        events[1].1["result_payload"]["provider_message_id"],
        json!(provider_message_id)
    );
}

#[tokio::test]
async fn whatsapp_fixture_media_reconciles_send_voice_note_command_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-voice-note-{suffix}");
    let provider_chat_id = format!("wa-voice-chat-{suffix}");
    let command_id = format!("wa-voice-command-{suffix}");
    let provider_message_id = format!("provider-message:{command_id}");
    let provider_attachment_id = format!("provider-attachment:{command_id}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Voice Note Reconcile",
            "external_account_id": format!("wa-voice-reconcile-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/voice-reconcile-{suffix}")
        }),
    )
    .await;

    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "send_voice_note",
        &format!("send-voice-note:{suffix}"),
        &provider_chat_id,
        None,
        json!({
            "blob_id": format!("whatsapp/{suffix}/voice-note.ogg"),
            "media_type": "voice_note",
            "filename": "voice-note.ogg",
            "content_type": "audio/ogg"
        }),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "Voice Note Reconciliation",
                "sender_id": account_id,
                "sender_display_name": "Макошь Owner",
                "text": "Observed voice note message",
                "import_batch_id": format!("whatsapp-voice-message-{suffix}"),
                "occurred_at": "2026-06-06T12:09:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("voice note message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

    let media_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/media",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "provider_attachment_id": provider_attachment_id,
                "filename": "voice-note.ogg",
                "content_type": "audio/ogg",
                "size_bytes": 4096,
                "sha256": "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                "storage_kind": "local_fs",
                "storage_path": format!("whatsapp/{suffix}/voice-note.ogg"),
                "import_batch_id": format!("whatsapp-voice-reconcile-{suffix}"),
                "observed_at": "2026-06-06T12:10:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("voice note media reconcile response");
    assert_eq!(media_response.status(), StatusCode::OK);
    let media_body = json_body(media_response).await;
    let attachment_id = media_body["attachment_id"]
        .as_str()
        .expect("attachment id")
        .to_owned();
    assert_eq!(media_body["message_id"], json!(message_id));

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled whatsapp send voice note command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let mirrored_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("mirrored communication provider command status");
    assert_eq!(mirrored_status, "completed");

    let attachment_row = sqlx::query(
        r#"
        SELECT message_id, provider_attachment_id, content_type, filename
        FROM communication_attachments
        WHERE attachment_id = $1
        "#,
    )
    .bind(&attachment_id)
    .fetch_one(&pool)
    .await
    .expect("projected voice note attachment row");
    assert_eq!(
        attachment_row
            .try_get::<String, _>("message_id")
            .expect("message id"),
        message_id
    );
    assert_eq!(
        attachment_row
            .try_get::<String, _>("provider_attachment_id")
            .expect("provider attachment id"),
        provider_attachment_id
    );
    assert_eq!(
        attachment_row
            .try_get::<String, _>("content_type")
            .expect("content type"),
        "audio/ogg"
    );
    assert_eq!(
        attachment_row
            .try_get::<Option<String>, _>("filename")
            .expect("filename")
            .as_deref(),
        Some("voice-note.ogg")
    );

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp voice note reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.fixture_media")
    );
    assert_eq!(
        events[1].1["result_payload"]["provider_attachment_id"],
        json!(provider_attachment_id)
    );
    assert_eq!(
        events[1].1["result_payload"]["provider_message_id"],
        json!(provider_message_id)
    );
}

#[tokio::test]
async fn whatsapp_fixture_message_reconciles_send_text_command_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-send-text-{suffix}");
    let provider_chat_id = format!("wa-send-text-chat-{suffix}");
    let command_id = format!("wa-send-text-command-{suffix}");
    let provider_message_id = format!("provider-message:{command_id}");
    let text = format!("Observed send_text reconciliation {suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Send Text Reconcile",
            "external_account_id": format!("wa-send-text-reconcile-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/send-text-reconcile-{suffix}")
        }),
    )
    .await;

    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "send_text",
        &format!("send-text:{suffix}"),
        &provider_chat_id,
        None,
        json!({"text": text}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "Send Text Reconciliation",
                "sender_id": account_id,
                "sender_display_name": "Макошь Owner",
                "text": text,
                "import_batch_id": format!("whatsapp-send-text-reconcile-{suffix}"),
                "occurred_at": "2026-06-06T12:10:00Z",
                "delivery_state": "sent"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("send text reconcile response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let message_id = body["message_id"].as_str().expect("message id").to_owned();

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled whatsapp send text command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let mirrored_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("mirrored communication provider command status");
    assert_eq!(mirrored_status, "completed");

    let stored_message: (String, String) = sqlx::query_as(
        r#"
        SELECT provider_record_id, body_text
        FROM communication_messages
        WHERE message_id = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("stored send text message");
    assert_eq!(stored_message.0, provider_message_id);
    assert_eq!(stored_message.1, text);

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp send text reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.fixture_message")
    );
    assert_eq!(
        events[1].1["result_payload"]["provider_message_id"],
        json!(provider_message_id)
    );
    assert_eq!(
        events[1].1["provider_state"]["delivery_state"],
        json!("sent")
    );

    let receipt_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/receipts",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "delivery_state": "received",
                "import_batch_id": format!("wa-runtime-bridge-send-text-receipt-{suffix}"),
                "observed_at": "2026-06-06T12:11:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge send text receipt response");
    assert_eq!(receipt_response.status(), StatusCode::OK);

    let receipt_reconciled_row = sqlx::query(
        r#"
        SELECT status, result_payload, provider_state, provider_observed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge receipt reconciled command");
    assert_eq!(
        receipt_reconciled_row
            .try_get::<String, _>("status")
            .expect("status"),
        "completed"
    );
    let receipt_result_payload: Value = receipt_reconciled_row
        .try_get("result_payload")
        .expect("result_payload");
    assert_eq!(receipt_result_payload["delivery_state"], json!("received"));
    assert_eq!(
        receipt_result_payload["provider_message_id"],
        json!(provider_message_id)
    );
    let receipt_provider_state: Value = receipt_reconciled_row
        .try_get("provider_state")
        .expect("provider_state");
    assert_eq!(receipt_provider_state["delivery_state"], json!("received"));
    assert_eq!(
        receipt_provider_state["observed_via"],
        json!("fixture_receipt")
    );
    assert!(
        receipt_reconciled_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("provider_observed_at")
            .expect("provider_observed_at")
            .is_some()
    );

    let receipt_events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("runtime bridge receipt reconciliation events");
    assert_eq!(receipt_events.len(), 4);
    assert_eq!(receipt_events[2].0, "whatsapp.command.status_changed");
    assert_eq!(receipt_events[3].0, "whatsapp.command.reconciled");
    assert_eq!(
        receipt_events[3].1["source"],
        json!("provider_observation_consumer")
    );
    assert_eq!(
        receipt_events[3].1["result_payload"]["delivery_state"],
        json!("received")
    );
}

#[tokio::test]
async fn whatsapp_runtime_bridge_message_reconciles_send_text_command_with_live_provenance() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-reconcile-send-text-{suffix}");
    let provider_chat_id = format!("wa-runtime-bridge-send-text-chat-{suffix}");
    let command_id = format!("wa-runtime-bridge-send-text-command-{suffix}");
    let provider_message_id = format!("provider-message:{command_id}");
    let text = format!("Observed runtime bridge send_text reconciliation {suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge Send Text Reconcile",
                "external_account_id": format!("wa-runtime-bridge-send-text-reconcile-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("live message reconcile account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "send_text",
        &format!("send-text:{suffix}"),
        &provider_chat_id,
        None,
        json!({"text": text}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_title": "Runtime Bridge Send Text Reconciliation",
                "sender_id": account_id,
                "sender_display_name": "Макошь Owner",
                "text": text,
                "import_batch_id": format!("wa-runtime-bridge-send-text-reconcile-{suffix}"),
                "occurred_at": "2026-06-06T12:10:00Z",
                "delivery_state": "sent"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge send text reconcile response");
    assert_eq!(response.status(), StatusCode::OK);

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge reconciled send text command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let mirrored_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge mirrored command status");
    assert_eq!(mirrored_status, "completed");

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("runtime bridge send text reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.runtime_bridge_message")
    );
    assert_eq!(
        events[1].1["result_payload"]["provider_message_id"],
        json!(provider_message_id)
    );
    assert_eq!(
        events[1].1["provider_state"]["delivery_state"],
        json!("sent")
    );
}

#[tokio::test]
async fn whatsapp_runtime_bridge_dialog_reconciles_archive_command_with_live_provenance() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-reconcile-dialog-{suffix}");
    let provider_chat_id = format!("wa-runtime-bridge-dialog-chat-{suffix}");
    let command_id = format!("wa-runtime-bridge-dialog-command-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge Dialog Reconcile",
                "external_account_id": format!("wa-runtime-bridge-dialog-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("live dialog reconcile account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "archive",
        &format!("archive:{suffix}"),
        &provider_chat_id,
        None,
        json!({"archived": true}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/dialogs",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "chat_title": "Runtime Bridge Archive Reconciliation",
                "chat_kind": "private",
                "is_archived": true,
                "is_pinned": false,
                "import_batch_id": format!("whatsapp-runtime-bridge-dialog-reconcile-{suffix}"),
                "observed_at": "2026-06-06T12:10:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge dialog reconcile response");
    assert_eq!(response.status(), StatusCode::OK);

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge reconciled whatsapp archive command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let mirrored_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge mirrored dialog command status");
    assert_eq!(mirrored_status, "completed");

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("runtime bridge dialog reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.runtime_bridge_dialog")
    );

    let runtime_event_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'command_id' = $2
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("runtime bridge archive command runtime-event kinds");
    assert_eq!(runtime_event_kinds, vec!["conversation.archive.completed"]);
}

#[tokio::test]
async fn whatsapp_runtime_bridge_participant_reconciles_join_group_command_with_live_provenance() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-runtime-bridge-reconcile-participant-{suffix}");
    let provider_chat_id = format!("wa-runtime-bridge-group-chat-{suffix}");
    let self_provider_identity_id = format!("wa-runtime-bridge-self-{suffix}");
    let command_id = format!("wa-runtime-bridge-join-group-command-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "whatsapp_web",
                "provider_shape": "whatsapp_web_companion",
                "display_name": "WhatsApp Runtime Bridge Participant Reconcile",
                "external_account_id": format!("wa-runtime-bridge-participant-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("live participant reconcile account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "join_group",
        &format!("join-group:{suffix}"),
        &provider_chat_id,
        None,
        json!({"invite_link": format!("https://chat.whatsapp.com/{suffix}")}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/participants",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "chat_title": "Runtime Bridge Group Join Reconciliation",
                "chat_kind": "group",
                "provider_identity_id": self_provider_identity_id,
                "identity_kind": "whatsapp_user",
                "display_name": "Макошь Owner",
                "role": "member",
                "status": "joined",
                "import_batch_id": format!("whatsapp-runtime-bridge-participant-reconcile-{suffix}"),
                "observed_at": "2026-06-06T12:10:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge participant reconcile response");
    assert_eq!(response.status(), StatusCode::OK);

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge reconciled join_group command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let mirrored_status: String = sqlx::query_scalar(
        "SELECT status FROM communication_provider_commands WHERE command_id = $1",
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge mirrored participant command status");
    assert_eq!(mirrored_status, "completed");

    let reconciled_event: (String, Value) = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type = 'whatsapp.command.reconciled'
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("runtime bridge join_group reconciled event");
    assert_eq!(reconciled_event.0, "whatsapp.command.reconciled");
    assert_eq!(
        reconciled_event.1["source"],
        json!("provider_observed.runtime_bridge_participant")
    );

    let runtime_event_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'command_id' = $2
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("runtime bridge join_group runtime-event kinds");
    assert_eq!(runtime_event_kinds, vec!["group.join.completed"]);
}

#[tokio::test]
async fn whatsapp_fixture_dialog_reconciles_archive_command_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-dialog-{suffix}");
    let provider_chat_id = format!("wa-dialog-chat-{suffix}");
    let command_id = format!("wa-dialog-command-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Dialog Reconcile",
            "external_account_id": format!("wa-dialog-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/dialog-{suffix}")
        }),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &command_id,
        &account_id,
        "archive",
        &format!("archive:{suffix}"),
        &provider_chat_id,
        None,
        json!({"archived": true}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/dialogs",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "chat_title": "Archive Reconciliation",
                "chat_kind": "private",
                "is_archived": true,
                "is_pinned": false,
                "import_batch_id": format!("whatsapp-dialog-reconcile-{suffix}"),
                "observed_at": "2026-06-06T12:10:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dialog reconcile response");
    assert_eq!(response.status(), StatusCode::OK);

    let command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled whatsapp archive command");
    assert_eq!(
        command_row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' = $1
          AND event_type IN ('whatsapp.command.status_changed', 'whatsapp.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("whatsapp dialog reconciliation events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "whatsapp.command.status_changed");
    assert_eq!(events[1].0, "whatsapp.command.reconciled");
    assert_eq!(
        events[1].1["source"],
        json!("provider_observed.fixture_dialog")
    );

    let runtime_event_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'command_id' = $2
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .bind(&command_id)
    .fetch_all(&pool)
    .await
    .expect("archive command runtime-event kinds");
    assert_eq!(runtime_event_kinds, vec!["conversation.archive.completed"]);
}

#[tokio::test]
async fn whatsapp_fixture_dialog_reconciles_mute_and_mark_unread_commands_via_observed_event() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-dialog-state-{suffix}");
    let provider_chat_id = format!("wa-dialog-state-chat-{suffix}");
    let mute_command_id = format!("wa-dialog-mute-command-{suffix}");
    let unread_command_id = format!("wa-dialog-unread-command-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Dialog State Reconcile",
            "external_account_id": format!("wa-dialog-state-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/dialog-state-{suffix}")
        }),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &mute_command_id,
        &account_id,
        "mute",
        &format!("mute:{suffix}"),
        &provider_chat_id,
        None,
        json!({"mute_state": "muted"}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &unread_command_id,
        &account_id,
        "mark_unread",
        &format!("mark-unread:{suffix}"),
        &provider_chat_id,
        None,
        json!({"read_state": "unread"}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/dialogs",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "chat_title": "Dialog State Reconciliation",
                "chat_kind": "private",
                "is_archived": false,
                "is_pinned": false,
                "is_muted": true,
                "is_unread": true,
                "import_batch_id": format!("whatsapp-dialog-state-reconcile-{suffix}"),
                "observed_at": "2026-06-06T12:12:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dialog state reconcile response");
    let dialog_status = response.status();
    let dialog_body = json_body(response).await;
    assert_eq!(
        dialog_status,
        StatusCode::OK,
        "response body: {dialog_body}"
    );

    for command_id in [&mute_command_id, &unread_command_id] {
        let command_row = sqlx::query(
            r#"
            SELECT status, reconciliation_status, completed_at
            FROM whatsapp_provider_write_commands
            WHERE command_id = $1
            "#,
        )
        .bind(command_id)
        .fetch_one(&pool)
        .await
        .expect("reconciled whatsapp dialog state command");
        assert_eq!(
            command_row.try_get::<String, _>("status").expect("status"),
            "completed"
        );
        assert_eq!(
            command_row
                .try_get::<String, _>("reconciliation_status")
                .expect("reconciliation status"),
            "observed"
        );
        assert!(
            command_row
                .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
                .expect("completed_at")
                .is_some()
        );
    }

    let conversation_metadata: Value = sqlx::query_scalar(
        "SELECT metadata FROM communication_conversations WHERE conversation_id = $1",
    )
    .bind(
        dialog_body["conversation_id"]
            .as_str()
            .expect("dialog conversation id"),
    )
    .fetch_one(&pool)
    .await
    .expect("dialog state conversation metadata");
    assert_eq!(conversation_metadata["is_muted"], json!(true));
    assert_eq!(conversation_metadata["is_unread"], json!(true));
}

#[tokio::test]
async fn whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event()
 {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-dialog-inverse-{suffix}");
    let provider_chat_id = format!("wa-dialog-inverse-chat-{suffix}");
    let unarchive_command_id = format!("wa-dialog-unarchive-command-{suffix}");
    let unpin_command_id = format!("wa-dialog-unpin-command-{suffix}");
    let unmute_command_id = format!("wa-dialog-unmute-command-{suffix}");
    let mark_read_command_id = format!("wa-dialog-read-command-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Dialog Inverse Reconcile",
            "external_account_id": format!("wa-dialog-inverse-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/dialog-inverse-{suffix}")
        }),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &unarchive_command_id,
        &account_id,
        "unarchive",
        &format!("unarchive:{suffix}"),
        &provider_chat_id,
        None,
        json!({"archived": false}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &unpin_command_id,
        &account_id,
        "unpin",
        &format!("unpin:{suffix}"),
        &provider_chat_id,
        None,
        json!({"pin_state": "unpinned"}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &unmute_command_id,
        &account_id,
        "unmute",
        &format!("unmute:{suffix}"),
        &provider_chat_id,
        None,
        json!({"mute_state": "unmuted"}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;
    seed_whatsapp_provider_command(
        &pool,
        &mark_read_command_id,
        &account_id,
        "mark_read",
        &format!("mark-read:{suffix}"),
        &provider_chat_id,
        None,
        json!({"read_state": "read"}),
        json!({"provider_chat_id": provider_chat_id}),
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/dialogs",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "chat_title": "Dialog Inverse State Reconciliation",
                "chat_kind": "private",
                "is_archived": false,
                "is_pinned": false,
                "is_muted": false,
                "is_unread": false,
                "import_batch_id": format!("whatsapp-dialog-inverse-reconcile-{suffix}"),
                "observed_at": "2026-06-06T12:14:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dialog inverse reconcile response");
    let dialog_status = response.status();
    let dialog_body = json_body(response).await;
    assert_eq!(
        dialog_status,
        StatusCode::OK,
        "response body: {dialog_body}"
    );

    for command_id in [
        &unarchive_command_id,
        &unpin_command_id,
        &unmute_command_id,
        &mark_read_command_id,
    ] {
        let command_row = sqlx::query(
            r#"
            SELECT status, reconciliation_status, completed_at
            FROM whatsapp_provider_write_commands
            WHERE command_id = $1
            "#,
        )
        .bind(command_id)
        .fetch_one(&pool)
        .await
        .expect("reconciled whatsapp inverse dialog state command");
        assert_eq!(
            command_row.try_get::<String, _>("status").expect("status"),
            "completed"
        );
        assert_eq!(
            command_row
                .try_get::<String, _>("reconciliation_status")
                .expect("reconciliation status"),
            "observed"
        );
        assert!(
            command_row
                .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
                .expect("completed_at")
                .is_some()
        );
    }

    let conversation_metadata: Value = sqlx::query_scalar(
        "SELECT metadata FROM communication_conversations WHERE conversation_id = $1",
    )
    .bind(
        dialog_body["conversation_id"]
            .as_str()
            .expect("dialog conversation id"),
    )
    .fetch_one(&pool)
    .await
    .expect("dialog inverse state conversation metadata");
    assert_eq!(conversation_metadata["is_archived"], json!(false));
    assert_eq!(conversation_metadata["is_pinned"], json!(false));
    assert_eq!(conversation_metadata["is_muted"], json!(false));
    assert_eq!(conversation_metadata["is_unread"], json!(false));
}

#[tokio::test]
async fn whatsapp_fixture_participant_reconciles_join_and_leave_group_commands_via_observed_event()
{
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("whatsapp-reconcile-join-leave-{suffix}");
    let provider_chat_id = format!("wa-reconcile-join-leave-chat-{suffix}");
    let join_command_id = format!("wa-reconcile-join-{suffix}");
    let leave_command_id = format!("wa-reconcile-leave-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/whatsapp/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "whatsapp_web",
            "display_name": "WhatsApp Reconcile Join/Leave Fixture",
            "external_account_id": format!("wa-reconcile-join-leave-{suffix}"),
            "device_name": "Макошь Desktop Fixture",
            "local_state_path": format!("docker/data/whatsapp/reconcile-join-leave-{suffix}")
        }),
    )
    .await;

    seed_whatsapp_provider_command(
        &pool,
        &join_command_id,
        &account_id,
        "join_group",
        &format!("join:{suffix}"),
        &provider_chat_id,
        None,
        json!({"invite_link": format!("https://chat.whatsapp.com/invite-{suffix}")}),
        json!({"provider_chat_id": provider_chat_id, "chat_kind": "group", "chat_title": "Join/Leave Reconciliation"}),
    )
    .await;

    let join_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/participants",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "chat_title": "Join/Leave Reconciliation",
                "chat_kind": "group",
                "provider_member_id": format!("self-member:{account_id}"),
                "provider_identity_id": format!("self-identity:{account_id}"),
                "identity_kind": "whatsapp_self",
                "display_name": "Макошь Owner",
                "push_name": "Макошь Owner",
                "role": "member",
                "status": "member",
                "is_self": true,
                "is_admin": false,
                "is_owner": false,
                "import_batch_id": format!("whatsapp-reconcile-join-{suffix}"),
                "observed_at": "2026-06-06T12:20:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("join participant reconcile response");
    assert_eq!(join_response.status(), StatusCode::OK);
    let join_participant_body = json_body(join_response).await;

    let join_command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&join_command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled join command");
    assert_eq!(
        join_command_row
            .try_get::<String, _>("status")
            .expect("status"),
        "completed"
    );
    assert_eq!(
        join_command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert!(
        join_command_row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at")
            .expect("completed_at")
            .is_some()
    );

    let joined_participant: (String, String, Value) = sqlx::query_as(
        r#"
        SELECT participant_id, role, metadata
        FROM communication_conversation_participants
        WHERE participant_id = $1
        "#,
    )
    .bind(
        join_participant_body["participant_id"]
            .as_str()
            .expect("join participant id"),
    )
    .fetch_one(&pool)
    .await
    .expect("joined self participant");
    assert_eq!(joined_participant.1, "member");
    assert_eq!(joined_participant.2["status"], json!("member"));

    let join_runtime_event_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'command_id' = $2
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .bind(&join_command_id)
    .fetch_all(&pool)
    .await
    .expect("join command runtime-event kinds");
    assert_eq!(join_runtime_event_kinds, vec!["group.join.completed"]);

    seed_whatsapp_provider_command(
        &pool,
        &leave_command_id,
        &account_id,
        "leave_group",
        &format!("leave:{suffix}"),
        &provider_chat_id,
        None,
        json!({"membership_state": "left"}),
        json!({"provider_chat_id": provider_chat_id, "chat_kind": "group", "chat_title": "Join/Leave Reconciliation"}),
    )
    .await;

    let leave_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/fixtures/participants",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "chat_title": "Join/Leave Reconciliation",
                "chat_kind": "group",
                "provider_member_id": format!("self-member:{account_id}"),
                "provider_identity_id": format!("self-identity:{account_id}"),
                "identity_kind": "whatsapp_self",
                "display_name": "Макошь Owner",
                "push_name": "Макошь Owner",
                "role": "member",
                "status": "left",
                "is_self": true,
                "is_admin": false,
                "is_owner": false,
                "import_batch_id": format!("whatsapp-reconcile-leave-{suffix}"),
                "observed_at": "2026-06-06T12:21:00Z"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("leave participant reconcile response");
    assert_eq!(leave_response.status(), StatusCode::OK);

    let leave_command_row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, completed_at
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&leave_command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled leave command");
    assert_eq!(
        leave_command_row
            .try_get::<String, _>("status")
            .expect("status"),
        "completed"
    );
    assert_eq!(
        leave_command_row
            .try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );

    let left_participant: (String, String, Value) = sqlx::query_as(
        r#"
        SELECT participant_id, role, metadata
        FROM communication_conversation_participants
        WHERE participant_id = $1
        "#,
    )
    .bind(&joined_participant.0)
    .fetch_one(&pool)
    .await
    .expect("left self participant");
    assert_eq!(left_participant.1, "member");
    assert_eq!(left_participant.2["status"], json!("left"));
    assert_eq!(left_participant.2["membership_changed"], json!(true));

    let leave_runtime_event_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT payload->>'runtime_event_kind'
        FROM event_log
        WHERE event_type = 'signal.accepted.whatsapp.runtime_event'
          AND source->>'account_id' = $1
          AND payload->'metadata'->>'command_id' = $2
        ORDER BY position ASC
        "#,
    )
    .bind(&account_id)
    .bind(&leave_command_id)
    .fetch_all(&pool)
    .await
    .expect("leave command runtime-event kinds");
    assert_eq!(leave_runtime_event_kinds, vec!["group.leave.completed"]);

    let reconciled_events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE payload->>'command_id' IN ($1, $2)
          AND event_type = 'whatsapp.command.reconciled'
        ORDER BY position ASC
        "#,
    )
    .bind(&join_command_id)
    .bind(&leave_command_id)
    .fetch_all(&pool)
    .await
    .expect("join/leave reconciled events");
    assert_eq!(reconciled_events.len(), 2);
    assert!(reconciled_events.iter().all(|(_, payload)| {
        payload["source"] == json!("provider_observed.fixture_participant")
    }));
}

fn assert_capability_status(body: &Value, capability: &str, status: &str, closure_gate: bool) {
    let capabilities = body["capabilities"].as_array().expect("capabilities");
    assert!(
        capabilities.iter().any(|item| {
            item["capability"] == capability
                && item["status"] == status
                && item["closure_gate"] == closure_gate
        }),
        "expected capability {capability} to have status {status} and closure_gate {closure_gate}"
    );
}

fn assert_json_array_contains(value: &Value, expected: &str) {
    assert!(
        value
            .as_array()
            .expect("JSON array")
            .iter()
            .any(|item| item == expected),
        "expected JSON array to contain {expected}"
    );
}

async fn assert_ok<S>(app: S, path: &str, body: Value)
where
    S: tower::Service<
            Request<Body>,
            Response = axum::http::Response<Body>,
            Error = std::convert::Infallible,
        > + Clone,
    S::Future: Send,
{
    let response = app
        .oneshot(json_post_request_with_actor(path, body, LOCAL_API_TOKEN))
        .await
        .expect("response");
    let status = response.status();
    let response_body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {response_body}");
}

async fn trigger_whatsapp_runtime_bridge_import<S>(app: S, account_id: &str)
where
    S: tower::Service<
            Request<Body>,
            Response = axum::http::Response<Body>,
            Error = std::convert::Infallible,
        > + Clone,
    S::Future: Send,
{
    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/whatsapp/runtime-bridge/commands/claim",
            json!({ "account_id": account_id, "limit": 10 }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime bridge import response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[allow(clippy::too_many_arguments)]
async fn seed_whatsapp_provider_command(
    pool: &sqlx::PgPool,
    command_id: &str,
    account_id: &str,
    command_kind: &str,
    idempotency_key: &str,
    provider_chat_id: &str,
    provider_message_id: Option<&str>,
    payload: Value,
    target_ref: Value,
) {
    sqlx::query(
        r#"
        INSERT INTO whatsapp_provider_write_commands (
            command_id, account_id, command_kind, idempotency_key, provider_chat_id,
            provider_message_id, target_ref, payload, capability_state, action_class,
            confirmation_decision, status, retry_count, max_retries, result_payload,
            audit_metadata, actor_id, happened_at, created_at, updated_at, reconciliation_status
        )
        VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, 'available', 'provider_write',
            'confirmed', 'executing', 0, 3, '{}'::jsonb,
            '{"provider":"whatsapp"}'::jsonb, 'makosh-frontend', '2026-06-06T11:59:00Z'::timestamptz,
            '2026-06-06T11:59:00Z'::timestamptz, '2026-06-06T11:59:00Z'::timestamptz, 'awaiting_provider'
        )
        "#,
    )
    .bind(command_id)
    .bind(account_id)
    .bind(command_kind)
    .bind(idempotency_key)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .bind(target_ref)
    .bind(payload)
    .execute(pool)
    .await
    .expect("seed whatsapp provider command");
}

fn json_post_request_with_actor(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn json_delete_request_with_actor(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(path)
        .header("x-makosh-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn get_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn whatsapp_event_payloads(
    pool: &sqlx::PgPool,
    event_type: &str,
    account_id: &str,
) -> Vec<Value> {
    sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = $1
          AND payload->>'account_id' = $2
        ORDER BY position ASC
        "#,
    )
    .bind(event_type)
    .bind(account_id)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| panic!("load {event_type} payloads for {account_id}"))
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

async fn unlock_test_vault<S>(app: S)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let entropy_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);

    let create_response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/vault/create",
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);
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

fn host_vault_entropy_events(count: usize) -> Vec<EntropyEvent> {
    (0..count)
        .map(|index| EntropyEvent {
            x: (index % 977) as f64,
            y: (index % 541) as f64,
            dx: ((index % 13) as f64) - 6.0,
            dy: ((index % 17) as f64) - 8.0,
            timestamp_ms: index as f64 * 7.0,
            velocity: (index % 29) as f64 / 10.0,
            acceleration: (index % 31) as f64 / 100.0,
            interval_ms: 7.0,
        })
        .collect()
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("json body")
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos()
        .to_string()
}
