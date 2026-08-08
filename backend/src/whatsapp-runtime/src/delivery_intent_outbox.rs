//! WhatsApp-owned exact-byte relay for delivery-intent terminal results.

use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_whatsapp_persistence::{WhatsAppDeliveryIntentStoreV1, WhatsAppDurablePersistenceError};

#[derive(Debug)]
pub enum WhatsAppDeliveryIntentOutboxRelayErrorV1 {
    Persistence(WhatsAppDurablePersistenceError),
    Unavailable,
}

pub async fn relay_whatsapp_delivery_intent_outbox_once_v1(
    store: &WhatsAppDeliveryIntentStoreV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, WhatsAppDeliveryIntentOutboxRelayErrorV1> {
    let records = store
        .pending_result_outbox(64)
        .await
        .map_err(WhatsAppDeliveryIntentOutboxRelayErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| WhatsAppDeliveryIntentOutboxRelayErrorV1::Unavailable)?;
        store
            .mark_result_outbox_published(record.message_id(), published_at_unix_seconds)
            .await
            .map_err(WhatsAppDeliveryIntentOutboxRelayErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}
