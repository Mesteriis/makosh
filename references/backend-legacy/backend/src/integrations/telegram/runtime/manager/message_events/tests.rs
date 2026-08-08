use chrono::Utc;
use makosh_backend_testkit::context::TestContext;
use serde_json::json;

use super::*;
use crate::integrations::telegram::client::models::chats::TelegramChatKind;
use crate::integrations::telegram::client::models::messages::{
    NewTelegramMessage, TelegramDeliveryState,
};

async fn seed_runtime_account(pool: &sqlx::PgPool, account_id: &str, external: &str) {
    crate::test_support::upsert_telegram_runtime_account(
        pool,
        account_id,
        "Telegram Runtime Account",
        external,
    )
    .await;
}

#[tokio::test]
async fn publish_message_content_updated_event_skips_without_projected_message() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-content-runtime";
    let provider_chat_id = "-100content-runtime";
    let provider_message_id = "42";
    let provider_message_ref = format!("{provider_chat_id}:{provider_message_id}");
    let event_bus = InMemoryEventBus::new();
    let mut events = event_bus.subscribe();

    seed_runtime_account(&pool, account_id, "telegram-ext-content").await;

    let store = crate::test_support::telegram_store(&pool);
    let _observed = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_ref.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Content Runtime Chat".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "before".to_owned(),
            import_batch_id: "telegram-runtime-content".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    let snapshot = TelegramTdlibMessageContentSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        text: "after".to_owned(),
        new_content: json!({
            "@type": "messageText",
            "text": {"@type": "formattedText", "text": "after"}
        }),
        source_event: "updateMessageContent".to_owned(),
        raw: json!({"@type": "message"}),
    };

    publish_message_content_updated_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    assert!(events.try_recv().is_err());
}

#[tokio::test]
async fn publish_message_created_event_publishes_signal_hub_raw_signal_instead_of_legacy_event() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-created-runtime";
    let provider_chat_id = "-100created-runtime";
    let event_bus = InMemoryEventBus::new();
    let mut events = event_bus.subscribe();

    seed_runtime_account(&pool, account_id, "telegram-ext-created").await;

    let snapshot = TelegramTdlibMessageSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: "42".to_owned(),
        sender_id: "user:777".to_owned(),
        sender_display_name: "Alice".to_owned(),
        text: "hello from runtime".to_owned(),
        occurred_at: Utc::now(),
        delivery_state: TelegramDeliveryState::Received,
        raw: json!({
            "@type": "message",
            "chat_id": provider_chat_id,
            "id": 42,
            "content": {
                "@type": "messageText",
                "text": {
                    "@type": "formattedText",
                    "text": "hello from runtime"
                }
            }
        }),
    };

    publish_message_created_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    let event = events.try_recv().expect("raw signal broadcast");
    assert_eq!(event.event_type, "signal.raw.telegram.message.observed");

    let legacy_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'telegram.message.created'",
    )
    .fetch_one(&pool)
    .await
    .expect("legacy event count");
    assert_eq!(legacy_count, 0);

    let raw_signal_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.telegram.message.observed'",
    )
    .fetch_one(&pool)
    .await
    .expect("raw signal count");
    assert_eq!(raw_signal_count, 1);

    let raw_record_id = event.source["source_id"]
        .as_str()
        .expect("raw signal source_id");
    let raw_record = crate::test_support::load_communication_raw_record(&pool, raw_record_id).await;
    assert_eq!(raw_record.account_id, account_id);
    assert_eq!(
        raw_record.provider_record_id,
        format!("{provider_chat_id}:42")
    );
}

#[tokio::test]
async fn publish_message_edited_event_skips_without_projected_message() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-edited-runtime";
    let provider_chat_id = "-100edited-runtime";
    let provider_message_id = "42";
    let provider_message_ref = format!("{provider_chat_id}:{provider_message_id}");
    let event_bus = InMemoryEventBus::new();
    let mut events = event_bus.subscribe();
    let edit_timestamp = Utc::now();

    seed_runtime_account(&pool, account_id, "telegram-ext-edited").await;

    let store = crate::test_support::telegram_store(&pool);
    let _observed = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_ref.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Edited Runtime Chat".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "hello".to_owned(),
            import_batch_id: "telegram-runtime-edited".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    let snapshot = TelegramTdlibMessageEditedSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        edit_timestamp,
        reply_markup: Some(json!({
            "@type": "replyMarkupInlineKeyboard",
            "rows": []
        })),
        source_event: "updateMessageEdited".to_owned(),
        raw: json!({"@type": "message"}),
    };

    publish_message_edited_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    assert!(events.try_recv().is_err());
}

#[tokio::test]
async fn publish_reaction_changed_event_skips_without_projected_message() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-reaction-runtime";
    let provider_chat_id = "-100reaction-runtime";
    let provider_message_id = "42";
    let provider_message_ref = format!("{provider_chat_id}:{provider_message_id}");
    let event_bus = InMemoryEventBus::new();
    let mut events = event_bus.subscribe();

    seed_runtime_account(&pool, account_id, "telegram-ext-reaction").await;

    let store = crate::test_support::telegram_store(&pool);
    let _observed = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_ref.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Reaction Runtime Chat".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "hello".to_owned(),
            import_batch_id: "telegram-runtime-reactions".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    let snapshot = TelegramTdlibMessageInteractionInfoSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        source_event: "updateMessageInteractionInfo".to_owned(),
        raw: json!({
            "@type": "message",
            "interaction_info": {
                "@type": "messageInteractionInfo",
                "reactions": {
                    "@type": "messageReactions",
                    "reactions": [
                        {
                            "@type": "messageReaction",
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "total_count": 1,
                            "is_chosen": true
                        }
                    ],
                    "recent_reactions": [
                        {
                            "@type": "messageReaction",
                            "sender_id": {
                                "@type": "messageSenderUser",
                                "user_id": 888
                            },
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "is_outgoing": false
                        }
                    ]
                }
            }
        }),
    };

    publish_reaction_changed_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    assert!(events.try_recv().is_err());
}
