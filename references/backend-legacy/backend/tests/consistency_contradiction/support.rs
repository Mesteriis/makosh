#![allow(dead_code)]

use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};

use makosh_communications_api::evidence::NewRawCommunicationRecord;

use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::provider_observation_projection::consume_accepted_signal_event;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::signal_hub::telegram::dispatch_telegram_raw_signal;
use makosh_hub_backend::domains::signal_hub::whatsapp::dispatch_whatsapp_raw_signal;
use makosh_hub_backend::domains::signal_hub::zulip::dispatch_zulip_raw_signal;

use makosh_hub_backend::platform::storage::database::Database;
use makosh_provider_orchestration::observation_to_raw_communication_record;
use makosh_provider_zulip::event_mapper::{
    ZulipEventMappingContext, map_zulip_event_to_observation,
};
use makosh_provider_zulip::models::ZulipEvent;
use serde_json::json;
use sqlx::postgres::PgPool;

pub async fn live_consistency_pool(_test_name: &str) -> Option<PgPool> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(database.pool().expect("configured pool").clone())
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub async fn seed_message(
    pool: &PgPool,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Polygraph Gmail",
            format!("polygraph-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");

    let raw_record_id = format!("raw_polygraph_{suffix}_{provider_record_id}");
    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:polygraph:{suffix}:{provider_record_id}"),
                format!("batch-polygraph-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"polygraph_test"})),
        )
        .await
        .expect("raw message");

    let message_store = MessageProjectionStore::new(pool.clone());
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

pub async fn seed_telegram_message(
    pool: &PgPool,
    suffix: u128,
    sender_id: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_telegram_{suffix}");
    let provider_chat_id = format!("telegram-chat-{suffix}");
    let provider_message_id = format!("telegram-message-{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::TelegramUser,
            "Polygraph Telegram",
            format!("polygraph-telegram-{suffix}"),
        ))
        .await
        .expect("provider account");

    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_polygraph_telegram_{suffix}"),
                &account_id,
                "telegram_message",
                &provider_message_id,
                format!("sha256:polygraph:telegram:{suffix}"),
                format!("batch-polygraph-telegram-{suffix}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": format!("Polygraph Telegram {suffix}"),
                    "chat_kind": "private",
                    "sender_id": sender_id,
                    "sender_display_name": "Polygraph Telegram Sender",
                    "text": body_text,
                    "delivery_state": "received",
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "source": "polygraph_test",
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw telegram message");

    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &raw)
        .await
        .expect("dispatch telegram raw signal")
        .expect("accepted telegram signal");
    consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted telegram signal")
        .expect("projected telegram message")
        .message_id
}

pub async fn seed_whatsapp_message(
    pool: &PgPool,
    suffix: u128,
    sender_id: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_whatsapp_{suffix}");
    let provider_chat_id = format!("whatsapp-chat-{suffix}");
    let provider_message_id = format!("whatsapp-message-{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::WhatsappWeb,
            "Polygraph WhatsApp",
            format!("polygraph-whatsapp-{suffix}"),
        ))
        .await
        .expect("provider account");

    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_polygraph_whatsapp_{suffix}"),
                &account_id,
                "whatsapp_web_message",
                &provider_message_id,
                format!("sha256:polygraph:whatsapp:{suffix}"),
                format!("batch-polygraph-whatsapp-{suffix}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": format!("Polygraph WhatsApp {suffix}"),
                    "sender_id": sender_id,
                    "sender_display_name": "Polygraph WhatsApp Sender",
                    "text": body_text,
                    "delivery_state": "received",
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "source": "polygraph_test",
                "provider": "whatsapp",
                "provider_kind": "whatsapp_web",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw WhatsApp message");

    let accepted_event = dispatch_whatsapp_raw_signal(pool.clone(), &raw)
        .await
        .expect("dispatch WhatsApp raw signal")
        .expect("accepted WhatsApp signal");
    consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted WhatsApp signal")
        .expect("projected WhatsApp message")
        .message_id
}

pub async fn seed_zulip_message(
    pool: &PgPool,
    suffix: u128,
    sender_email: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_zulip_{suffix}");
    let provider_message_id = 10_000_000_i64 + (suffix % 1_000_000) as i64;
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::ZulipBot,
            "Polygraph Zulip",
            format!("polygraph-zulip-{suffix}"),
        ))
        .await
        .expect("provider account");

    let event: ZulipEvent = serde_json::from_value(json!({
        "id": provider_message_id + 1,
        "type": "message",
        "message": {
            "id": provider_message_id,
            "content": body_text,
            "sender_email": sender_email,
            "sender_full_name": "Polygraph Zulip Sender",
            "stream_id": 10,
            "display_recipient": "Polygraph Zulip",
            "topic": "Facts"
        }
    }))
    .expect("valid Zulip event");
    let mapping_context =
        ZulipEventMappingContext::new(&account_id, "http://localhost:8080", Utc::now())
            .with_import_batch_id(format!("batch-polygraph-zulip-{suffix}"))
            .with_scenario_id(format!("polygraph-zulip-{suffix}"));
    let new_raw_record = observation_to_raw_communication_record(
        map_zulip_event_to_observation(&event, &mapping_context).expect("map Zulip event"),
    );
    let raw = ingestion_store
        .record_raw_source(&new_raw_record)
        .await
        .expect("record raw Zulip message");
    let accepted_event = dispatch_zulip_raw_signal(pool.clone(), &raw)
        .await
        .expect("dispatch Zulip raw signal")
        .expect("accepted Zulip signal");
    consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted Zulip signal")
        .expect("projected Zulip message")
        .message_id
}
