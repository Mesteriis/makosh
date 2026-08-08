use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use sqlx::PgPool;

use crate::integrations::telegram::client::commands::mark_command_reconciled;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::models::topics::{NewTelegramTopic, TelegramTopic};
use crate::integrations::telegram::client::store::TelegramStore;
use crate::integrations::telegram::tdjson::parsing::events::TelegramTdlibTopicUpdateSnapshot;
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibTopicSnapshot;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;
use makosh_events_postgres::store::EventStore;

use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, publish_command_reconciled_events,
};
use super::topics::telegram_topic_id;

pub(super) async fn publish_topic_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    snapshot: &TelegramTdlibTopicUpdateSnapshot,
) {
    let Some(store) = telegram_store else {
        return;
    };
    let pool = store.pool();

    let observed_at = Utc::now();
    let topic = match upsert_topic_snapshot(
        store,
        account_id,
        &snapshot.provider_chat_id,
        &snapshot.topic,
    )
    .await
    {
        Ok(Some(topic)) => topic,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project topic update");
            return;
        }
    };

    let reconciled = match reconcile_topic_commands_from_provider_state(
        pool,
        account_id,
        &snapshot.provider_chat_id,
        snapshot.topic.provider_topic_id,
        snapshot.topic.is_closed,
        observed_at,
        "tdlib.updateForumTopicInfo",
    )
    .await
    {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(error = %error, topic_id = %topic.topic_id, "Telegram runtime event bridge: failed to reconcile topic commands");
            Vec::new()
        }
    };
    let context = TelegramRuntimeEventBridgeContext::new(Some(store.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, "updateForumTopicInfo").await;
    }

    let Ok(event) = topic_updated_event(account_id, &topic, observed_at) else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append topic event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) async fn upsert_topic_snapshot(
    store: &TelegramStore,
    account_id: &str,
    provider_chat_id: &str,
    snapshot: &TelegramTdlibTopicSnapshot,
) -> Result<Option<TelegramTopic>, TelegramError> {
    let Some(chat) = store.telegram_chat(account_id, provider_chat_id).await? else {
        return Ok(None);
    };

    let topic = NewTelegramTopic {
        topic_id: telegram_topic_id(&chat.telegram_chat_id, snapshot.provider_topic_id),
        telegram_chat_id: chat.telegram_chat_id.clone(),
        account_id: chat.account_id,
        provider_topic_id: snapshot.provider_topic_id,
        provider_chat_id: provider_chat_id.to_owned(),
        title: snapshot.title.clone(),
        icon_emoji: snapshot.icon_emoji.clone(),
        is_pinned: snapshot.is_pinned,
        is_closed: snapshot.is_closed,
        unread_count: topic_unread_count(snapshot.unread_count),
        last_message_at: snapshot.last_message_at,
    };

    crate::integrations::telegram::client::topics::upsert_topic(store.pool(), &topic)
        .await
        .map(Some)
}

fn topic_unread_count(unread_count: i64) -> i32 {
    unread_count.clamp(0, i32::MAX as i64) as i32
}

async fn reconcile_topic_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_topic_id: i64,
    is_closed: bool,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let command_kind = if is_closed {
        "topic_close"
    } else {
        "topic_reopen"
    };
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = $3
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
          AND payload->>'provider_topic_id' = $4
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(command_kind)
    .bind(provider_topic_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command =
            crate::integrations::telegram::client::rows::row_to_telegram_provider_write_command(
                row,
            )?;
        let provider_state = json!({
            "provider_chat_id": provider_chat_id,
            "provider_topic_id": provider_topic_id,
            "is_closed": is_closed,
            "observed_via": observed_via,
        });
        let result_payload = json!({
            "source": observed_via,
            "provider_chat_id": provider_chat_id,
            "provider_topic_id": provider_topic_id,
            "is_closed": is_closed,
            "provider_observed_at": observed_at,
        });
        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                provider_state,
                result_payload,
            )
            .await?,
        );
    }

    Ok(reconciled)
}

fn topic_updated_event(
    account_id: &str,
    topic: &TelegramTopic,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_topic_{}_{}_{}",
            account_id,
            topic.topic_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::TOPIC_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_topic",
            "id": topic.topic_id,
            "telegram_chat_id": topic.telegram_chat_id,
            "provider_chat_id": topic.provider_chat_id,
            "provider_topic_id": topic.provider_topic_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": topic.telegram_chat_id,
        "provider_chat_id": topic.provider_chat_id,
        "provider_topic_id": topic.provider_topic_id,
        "topic_id": topic.topic_id,
        "topic": topic,
        "source": "tdlib.updateForumTopicInfo"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateForumTopicInfo"
    }))
    .build()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use makosh_backend_testkit::context::TestContext;
    use sqlx::Row;

    use super::*;
    use crate::integrations::telegram::client::commands::insert_command;
    use crate::integrations::telegram::client::models::chats::{
        NewTelegramChat, TelegramChatKind, TelegramSyncState,
    };

    #[test]
    fn topic_updated_event_contains_sanitized_projection_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let topic = TelegramTopic {
            topic_id: "telegram_topic:v1:test".to_owned(),
            telegram_chat_id: "telegram_chat:v1:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_topic_id: 42,
            provider_chat_id: "-100123".to_owned(),
            title: "Release notes".to_owned(),
            icon_emoji: Some("123456".to_owned()),
            is_pinned: true,
            is_closed: false,
            unread_count: 0,
            last_message_at: None,
            metadata: json!({}),
            created_at: occurred_at,
            updated_at: occurred_at,
        };

        let event = topic_updated_event("acct-1", &topic, occurred_at).expect("event");

        assert_eq!(event.event_type, telegram_event_types::TOPIC_UPDATED);
        assert_eq!(event.subject["id"], "telegram_topic:v1:test");
        assert_eq!(event.payload["topic"]["title"], "Release notes");
        assert_eq!(event.payload["topic"]["is_pinned"], true);
    }

    #[tokio::test]
    async fn publish_topic_event_reconciles_topic_close_and_appends_runtime_events() {
        let ctx = TestContext::new().await;
        let pool = ctx.pool().clone();
        let account_id = "acct-1";
        let provider_chat_id = "chat-1";
        let event_bus = InMemoryEventBus::new();

        crate::test_support::upsert_telegram_runtime_account(
            &pool,
            account_id,
            "Topic Runtime Account",
            "telegram-ext-acct-1",
        )
        .await;

        let chat = crate::test_support::telegram_store(&pool)
            .upsert_chat(&NewTelegramChat {
                account_id: account_id.to_owned(),
                provider_chat_id: provider_chat_id.to_owned(),
                chat_kind: TelegramChatKind::Group,
                title: "Forum Chat".to_owned(),
                username: None,
                sync_state: TelegramSyncState::Synced,
                last_message_at: None,
                metadata: json!({}),
            })
            .await
            .expect("seed chat");

        let command_id = "cmd-topic-close-1";
        insert_command(
            &pool,
            command_id,
            account_id,
            "topic_close",
            "topic_close:42:seed",
            provider_chat_id,
            None,
            "available",
            "provider_write",
            "confirmed",
            "makosh-frontend",
            json!({
                "provider_topic_id": 42,
                "is_closed": true
            }),
            json!({
                "telegram_chat_id": chat.telegram_chat_id,
                "provider_chat_id": provider_chat_id,
                "provider_topic_id": 42
            }),
            json!({"source": "test"}),
        )
        .await
        .expect("seed topic close command");

        let snapshot = TelegramTdlibTopicUpdateSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            topic: TelegramTdlibTopicSnapshot {
                provider_topic_id: 42,
                title: "Release Notes".to_owned(),
                icon_emoji: None,
                is_pinned: false,
                is_closed: true,
                unread_count: 7,
                last_message_at: None,
            },
        };

        publish_topic_event(
            &Some(crate::test_support::telegram_store(&pool)),
            &event_bus,
            account_id,
            &snapshot,
        )
        .await;

        let rows: Vec<(String, serde_json::Value, serde_json::Value)> = sqlx::query_as(
            r#"
            SELECT event_type, subject, payload
            FROM event_log
            WHERE event_type IN (
                'telegram.command.status_changed',
                'telegram.command.reconciled',
                'telegram.topic.updated'
            )
            ORDER BY position ASC
            "#,
        )
        .fetch_all(&pool)
        .await
        .expect("topic runtime events");

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
        assert_eq!(rows[0].1["id"], json!(command_id));
        assert_eq!(rows[0].2["status"], json!("completed"));
        assert_eq!(rows[1].0, telegram_event_types::COMMAND_RECONCILED);
        assert_eq!(rows[1].1["id"], json!(command_id));
        assert_eq!(rows[1].2["reconciliation_status"], json!("observed"));
        assert_eq!(rows[2].0, telegram_event_types::TOPIC_UPDATED);
        assert_eq!(rows[2].1["provider_topic_id"], json!(42));
        assert_eq!(rows[2].2["topic"]["is_closed"], json!(true));
        assert_eq!(rows[2].2["topic"]["unread_count"], json!(7));

        let topic_unread_count: i32 = sqlx::query_scalar(
            "SELECT unread_count FROM telegram_topics WHERE provider_topic_id = 42",
        )
        .fetch_one(&pool)
        .await
        .expect("topic unread count");
        assert_eq!(topic_unread_count, 7);

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

        let stored_topic_id: String =
            sqlx::query_scalar("SELECT topic_id FROM telegram_topics WHERE provider_topic_id = 42")
                .fetch_one(&pool)
                .await
                .expect("stored topic id");

        let topic_observation_rows = sqlx::query(
            r#"
            SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
            FROM observation_links link
            JOIN observations observation
              ON observation.observation_id = link.observation_id
            JOIN observation_kind_definitions kind
              ON kind.kind_definition_id = observation.kind_definition_id
            WHERE link.domain = 'telegram'
              AND link.entity_kind = 'topic'
              AND link.entity_id = $1
            ORDER BY observation.captured_at ASC
            "#,
        )
        .bind(stored_topic_id)
        .fetch_all(&pool)
        .await
        .expect("topic observations");
        assert!(
            topic_observation_rows.iter().any(|row| {
                row.get::<String, _>("kind_code") == "TELEGRAM_TOPIC"
                    && row.get::<String, _>("relationship_kind") == "upsert"
                    && row.get::<serde_json::Value, _>("payload")["provider_topic_id"] == json!(42)
                    && row.get::<serde_json::Value, _>("payload")["is_closed"] == json!(true)
            }),
            "topic upsert observation must exist"
        );
    }
}
