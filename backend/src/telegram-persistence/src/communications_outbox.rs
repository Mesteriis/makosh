//! Telegram-owned adapter for the canonical exact-byte outbox relay.

use makosh_events_protocol::delivery::{
    OutboxEntryV1, OutboxPublishReceiptV1, OutboxRelayErrorV1, OwnerOutboxStorePortV1,
};

use crate::TelegramDurablePersistence;

pub struct TelegramCommunicationsOutboxStoreV1<'a> {
    durable: &'a TelegramDurablePersistence,
    published_at_unix_seconds: i64,
}

impl<'a> TelegramCommunicationsOutboxStoreV1<'a> {
    #[must_use]
    pub const fn new(
        durable: &'a TelegramDurablePersistence,
        published_at_unix_seconds: i64,
    ) -> Self {
        Self {
            durable,
            published_at_unix_seconds,
        }
    }
}

impl OwnerOutboxStorePortV1 for TelegramCommunicationsOutboxStoreV1<'_> {
    async fn next_pending(&mut self) -> Result<Option<OutboxEntryV1>, OutboxRelayErrorV1> {
        let record = self
            .durable
            .pending_communications_outbox(1)
            .await
            .map_err(|_| OutboxRelayErrorV1::Persistence)?
            .into_iter()
            .next();
        record
            .map(|record| {
                let outbox_id = message_id_hex(record.message_id());
                OutboxEntryV1::new(outbox_id, record)
            })
            .transpose()
    }

    async fn mark_published(
        &mut self,
        entry: &OutboxEntryV1,
        _receipt: &OutboxPublishReceiptV1,
    ) -> Result<(), OutboxRelayErrorV1> {
        self.durable
            .mark_communications_outbox_published(
                entry.record().message_id(),
                self.published_at_unix_seconds,
            )
            .await
            .map_err(|_| OutboxRelayErrorV1::Persistence)?
            .then_some(())
            .ok_or(OutboxRelayErrorV1::Persistence)
    }
}

fn message_id_hex(message_id: &[u8; 16]) -> String {
    message_id
        .iter()
        .fold(String::with_capacity(32), |mut id, byte| {
            use std::fmt::Write as _;

            write!(&mut id, "{byte:02x}").expect("writing to String cannot fail");
            id
        })
}
