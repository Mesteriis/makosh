use super::super::{TelegramMemberSyncContext, TelegramRuntimeEventBridgeContext};
use super::sync_provider_roster_snapshots;
use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::chats::NewTelegramChatParticipant;
use crate::integrations::telegram::client::models::chats::TelegramChat;
use crate::integrations::telegram::client::models::chats::{
    NewTelegramChat, TelegramChatKind, TelegramSyncState,
};
use crate::integrations::telegram::client::participants::upsert_chat_participant;
use crate::integrations::telegram::client::store::TelegramStore;
use crate::platform::config::app_config::AppConfig;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;
use crate::platform::secrets::resolver::InMemorySecretResolver;
use crate::platform::secrets::store::SecretReferenceStore;
use makosh_backend_testkit::context::TestContext;
use serde_json::json;
use sqlx::{PgPool, Row};

async fn seed_chat(
    pool: &PgPool,
    account_id: &str,
    external_account_id: &str,
    provider_chat_id: &str,
) -> Result<TelegramChat, TelegramError> {
    crate::test_support::upsert_telegram_runtime_account(
        pool,
        account_id,
        "Runtime Participant Account",
        external_account_id,
    )
    .await;
    crate::test_support::telegram_store(pool)
        .upsert_chat(&NewTelegramChat {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Runtime Participants".to_owned(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: None,
            metadata: json!({}),
        })
        .await
}

#[tokio::test]
async fn sync_provider_roster_snapshots_appends_join_reconciliation_after_participant_update() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-1";
    let provider_chat_id = "-10042";
    let chat = seed_chat(&pool, account_id, "user:42", provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "join-runtime-reconciled";
    insert_command(
        &pool,
        command_id,
        account_id,
        "join",
        "join:runtime:test",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({}),
        json!({"telegram_chat_id": chat.telegram_chat_id, "provider_chat_id": provider_chat_id}),
        json!({"source": "test"}),
    )
    .await
    .expect("seed join command");

    let provider_account_store = crate::test_support::communication_provider_account_store(&pool);
    let provider_secret_binding_store =
        crate::test_support::communication_provider_secret_binding_store(&pool);
    let telegram_store = crate::test_support::telegram_store(&pool);
    let secret_store = SecretReferenceStore::new(pool.clone());
    let secret_resolver = InMemorySecretResolver::new();
    let config = AppConfig::default();
    let event_bridge = Some(TelegramRuntimeEventBridgeContext::new(
        Some(telegram_store.clone()),
        InMemoryEventBus::new(),
    ));
    let context = TelegramMemberSyncContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &telegram_store,
        secret_store: &secret_store,
        secret_resolver: &secret_resolver,
        config: &config,
        event_bridge,
    };

    sync_provider_roster_snapshots(
        context,
        &chat,
        "user:42",
        vec![
            crate::integrations::telegram::tdjson::snapshots::TelegramTdlibChatMemberSnapshot {
                provider_member_id: "user:42".to_owned(),
                display_name: Some("Owner User".to_owned()),
                username: Some("owner".to_owned()),
                role: "member".to_owned(),
                status: "member".to_owned(),
                is_admin: false,
                is_owner: false,
                permissions: json!({}),
                raw: json!({}),
            },
        ],
        "tdlib.getSupergroupMembers",
        true,
    )
    .await
    .expect("sync members");

    let rows: Vec<(String, serde_json::Value, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, subject, payload
        FROM event_log
        WHERE event_type IN (
            'telegram.participant.updated',
            'telegram.command.status_changed',
            'telegram.command.reconciled'
        )
        ORDER BY position ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("runtime events");

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, telegram_event_types::PARTICIPANT_UPDATED);
    assert_eq!(rows[0].1["provider_member_id"], json!("user:42"));
    assert_eq!(rows[1].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(rows[1].1["id"], json!(command_id));
    assert_eq!(rows[2].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(rows[2].1["id"], json!(command_id));

    let command_status: Option<(String, String)> = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_optional(&pool)
    .await
    .expect("command status");
    assert_eq!(
        command_status,
        Some(("completed".to_owned(), "observed".to_owned()))
    );
    let participant_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'chat_participant'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(format!("{}:user:42", chat.telegram_chat_id))
    .fetch_all(&pool)
    .await
    .expect("participant observations");
    assert!(
        participant_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_CHAT_PARTICIPANT"
                && row.get::<String, _>("relationship_kind") == "upsert"
                && row.get::<serde_json::Value, _>("payload")["provider_member_id"]
                    == json!("user:42")
        }),
        "participant upsert observation must exist"
    );
}

#[tokio::test]
async fn sync_provider_roster_snapshots_appends_leave_reconciliation_after_absence_update() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-2";
    let provider_chat_id = "-10043";
    let chat = seed_chat(&pool, account_id, "user:42", provider_chat_id)
        .await
        .expect("seed chat");
    let _ = upsert_chat_participant(
        &pool,
        &NewTelegramChatParticipant {
            participant_id: "participant-self".to_owned(),
            telegram_chat_id: chat.telegram_chat_id.clone(),
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_member_id: "user:42".to_owned(),
            display_name: Some("Owner User".to_owned()),
            username: Some("owner".to_owned()),
            role: "member".to_owned(),
            status: "member".to_owned(),
            is_admin: false,
            is_owner: false,
            permissions: json!({}),
            raw_payload: json!({}),
            source: "tdlib".to_owned(),
        },
    )
    .await
    .expect("seed participant");
    let command_id = "leave-runtime-reconciled";
    insert_command(
        &pool,
        command_id,
        account_id,
        "leave",
        "leave:runtime:test",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({}),
        json!({"telegram_chat_id": chat.telegram_chat_id, "provider_chat_id": provider_chat_id}),
        json!({"source": "test"}),
    )
    .await
    .expect("seed leave command");

    let provider_account_store = crate::test_support::communication_provider_account_store(&pool);
    let provider_secret_binding_store =
        crate::test_support::communication_provider_secret_binding_store(&pool);
    let telegram_store = crate::test_support::telegram_store(&pool);
    let secret_store = SecretReferenceStore::new(pool.clone());
    let secret_resolver = InMemorySecretResolver::new();
    let config = AppConfig::default();
    let event_bridge = Some(TelegramRuntimeEventBridgeContext::new(
        Some(telegram_store.clone()),
        InMemoryEventBus::new(),
    ));
    let context = TelegramMemberSyncContext {
        provider_account_store: &provider_account_store,
        provider_secret_binding_store: &provider_secret_binding_store,
        telegram_store: &telegram_store,
        secret_store: &secret_store,
        secret_resolver: &secret_resolver,
        config: &config,
        event_bridge,
    };

    sync_provider_roster_snapshots(
        context,
        &chat,
        "user:42",
        Vec::new(),
        "tdlib.getSupergroupMembers",
        true,
    )
    .await
    .expect("sync members");

    let rows: Vec<(String, serde_json::Value, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, subject, payload
        FROM event_log
        WHERE event_type IN (
            'telegram.participant.updated',
            'telegram.command.status_changed',
            'telegram.command.reconciled'
        )
        ORDER BY position ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("runtime events");

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, telegram_event_types::PARTICIPANT_UPDATED);
    assert_eq!(rows[0].1["provider_member_id"], json!("user:42"));
    assert_eq!(
        rows[0].2["participant"]["status"],
        json!("absent_exhaustive")
    );
    assert_eq!(rows[1].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(rows[1].1["id"], json!(command_id));
    assert_eq!(rows[2].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(rows[2].1["id"], json!(command_id));

    let command_status: Option<(String, String)> = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_optional(&pool)
    .await
    .expect("command status");
    assert_eq!(
        command_status,
        Some(("completed".to_owned(), "observed".to_owned()))
    );
    let participant_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'chat_participant'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(format!("{}:user:42", chat.telegram_chat_id))
    .fetch_all(&pool)
    .await
    .expect("participant observations");
    assert!(
        participant_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_CHAT_PARTICIPANT"
                && row.get::<String, _>("relationship_kind") == "upsert"
        }),
        "seed participant upsert observation must exist"
    );
    assert!(
        participant_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_CHAT_PARTICIPANT"
                && row.get::<String, _>("relationship_kind") == "absent_exhaustive"
                && row.get::<serde_json::Value, _>("payload")["status"]
                    == json!("absent_exhaustive")
        }),
        "participant absent_exhaustive observation must exist"
    );
}
