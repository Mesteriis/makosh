//! Telegram Calls exact-byte outbox adapter for Communications call evidence.

use makosh_events_protocol::delivery::{
    OutboxEntryV1, OutboxPublishReceiptV1, OutboxRecordV1, OutboxRelayErrorV1,
    OwnerOutboxStorePortV1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{TelegramCallsPersistence, TelegramCallsPersistenceError};

pub struct TelegramCallEvidenceOutboxStoreV1<'a> {
    persistence: &'a TelegramCallsPersistence,
    published_at_unix_seconds: i64,
}

impl<'a> TelegramCallEvidenceOutboxStoreV1<'a> {
    #[must_use]
    pub const fn new(
        persistence: &'a TelegramCallsPersistence,
        published_at_unix_seconds: i64,
    ) -> Self {
        Self {
            persistence,
            published_at_unix_seconds,
        }
    }
}

impl OwnerOutboxStorePortV1 for TelegramCallEvidenceOutboxStoreV1<'_> {
    async fn next_pending(&mut self) -> Result<Option<OutboxEntryV1>, OutboxRelayErrorV1> {
        self.persistence
            .pending_call_evidence_outbox()
            .await
            .map_err(|_| OutboxRelayErrorV1::Persistence)?
            .map(|record| OutboxEntryV1::new(message_id_hex(record.message_id()), record))
            .transpose()
    }

    async fn mark_published(
        &mut self,
        entry: &OutboxEntryV1,
        _receipt: &OutboxPublishReceiptV1,
    ) -> Result<(), OutboxRelayErrorV1> {
        self.persistence
            .mark_call_evidence_outbox_published(
                entry.record().message_id(),
                self.published_at_unix_seconds,
            )
            .await
            .map_err(|_| OutboxRelayErrorV1::Persistence)?
            .then_some(())
            .ok_or(OutboxRelayErrorV1::Persistence)
    }
}

pub(crate) async fn insert_call_evidence_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    record: &OutboxRecordV1,
    created_at_unix_seconds: u64,
) -> Result<(), TelegramCallsPersistenceError> {
    let insert_result = sqlx::query(
        "INSERT INTO makosh_data.telegram_call_evidence_outbox (\
             message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds\
         ) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING",
    )
    .bind(record.message_id().as_slice())
    .bind(record.envelope_sha256().as_slice())
    .bind(record.exact_bytes())
    .bind(
        i64::try_from(created_at_unix_seconds)
            .map_err(|_| TelegramCallsPersistenceError::InvalidRequest("created_at"))?,
    )
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsPersistenceError::Database)?;

    if insert_result.rows_affected() == 0 {
        let existing = sqlx::query(
            "SELECT envelope_sha256, exact_envelope_bytes \
             FROM makosh_data.telegram_call_evidence_outbox WHERE message_id = $1",
        )
        .bind(record.message_id().as_slice())
        .fetch_one(&mut **transaction)
        .await
        .map_err(|_| TelegramCallsPersistenceError::Database)?;
        let existing_hash: Vec<u8> = existing
            .try_get("envelope_sha256")
            .map_err(|_| TelegramCallsPersistenceError::Database)?;
        let existing_bytes: Vec<u8> = existing
            .try_get("exact_envelope_bytes")
            .map_err(|_| TelegramCallsPersistenceError::Database)?;
        if existing_hash.as_slice() != record.envelope_sha256()
            || existing_bytes.as_slice() != record.exact_bytes()
        {
            return Err(TelegramCallsPersistenceError::IdempotencyConflict);
        }
    }

    Ok(())
}

impl TelegramCallsPersistence {
    async fn pending_call_evidence_outbox(
        &self,
    ) -> Result<Option<OutboxRecordV1>, TelegramCallsPersistenceError> {
        let row = sqlx::query(
            "SELECT exact_envelope_bytes \
             FROM makosh_data.telegram_call_evidence_outbox \
             WHERE published_at_unix_seconds IS NULL \
             ORDER BY created_at_unix_seconds, message_id LIMIT 1",
        )
        .fetch_optional(self.owner_pool())
        .await
        .map_err(|_| TelegramCallsPersistenceError::Database)?;
        row.map(|row| {
            let bytes: Vec<u8> = row
                .try_get("exact_envelope_bytes")
                .map_err(|_| TelegramCallsPersistenceError::InvalidRow)?;
            OutboxRecordV1::accept(bytes).map_err(|_| TelegramCallsPersistenceError::InvalidRow)
        })
        .transpose()
    }

    async fn mark_call_evidence_outbox_published(
        &self,
        message_id: &[u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, TelegramCallsPersistenceError> {
        if published_at_unix_seconds <= 0 {
            return Err(TelegramCallsPersistenceError::InvalidRequest(
                "published_at",
            ));
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.telegram_call_evidence_outbox \
             SET published_at_unix_seconds = $2 \
             WHERE message_id = $1 AND published_at_unix_seconds IS NULL",
        )
        .bind(message_id.as_slice())
        .bind(published_at_unix_seconds)
        .execute(self.owner_pool())
        .await
        .map_err(|_| TelegramCallsPersistenceError::Database)?;
        Ok(updated.rows_affected() == 1)
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
