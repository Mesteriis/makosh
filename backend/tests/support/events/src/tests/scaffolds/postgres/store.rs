//! SQLx implementation of the public owner-local delivery ports for conformance only.

use makosh_events_protocol::delivery::{
    InboxDecisionV1, OutboxEntryV1, OutboxPublishReceiptV1, OutboxRecordV1, OutboxRelayErrorV1,
    OwnerOutboxStorePortV1,
};
use sqlx::{AssertSqlSafe, PgPool, Row};

use super::super::OwnerDeliveryScaffoldV1;
use super::schema;

pub(super) struct PostgresOwnerDeliveryStore {
    pool: PgPool,
    scaffold: OwnerDeliveryScaffoldV1,
}

impl PostgresOwnerDeliveryStore {
    pub(super) fn new(pool: PgPool, scaffold: OwnerDeliveryScaffoldV1) -> Self {
        Self { pool, scaffold }
    }

    pub(super) async fn install(&self) -> Result<(), OutboxRelayErrorV1> {
        sqlx::raw_sql(AssertSqlSafe(schema::install(self.scaffold)))
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| OutboxRelayErrorV1::Persistence)
    }

    pub(super) async fn enqueue(
        &self,
        outbox_id: &str,
        record: &OutboxRecordV1,
    ) -> Result<(), OutboxRelayErrorV1> {
        sqlx::query(AssertSqlSafe(schema::insert_outbox(self.scaffold)))
            .bind(outbox_id)
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| OutboxRelayErrorV1::Persistence)
    }

    pub(super) async fn accept_inbox(
        &self,
        record: &OutboxRecordV1,
    ) -> Result<InboxDecisionV1, OutboxRelayErrorV1> {
        let inserted = sqlx::query(AssertSqlSafe(schema::insert_inbox(self.scaffold)))
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .execute(&self.pool)
            .await
            .map_err(|_| OutboxRelayErrorV1::Persistence)?
            .rows_affected();
        if inserted == 1 {
            return Ok(InboxDecisionV1::Accept);
        }
        self.classify_existing_inbox(record).await
    }

    async fn classify_existing_inbox(
        &self,
        record: &OutboxRecordV1,
    ) -> Result<InboxDecisionV1, OutboxRelayErrorV1> {
        let row = sqlx::query(AssertSqlSafe(schema::inbox_hash(self.scaffold)))
            .bind(record.message_id().as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| OutboxRelayErrorV1::Persistence)?
            .ok_or(OutboxRelayErrorV1::Persistence)?;
        let existing: Vec<u8> = row
            .try_get("envelope_sha256")
            .map_err(|_| OutboxRelayErrorV1::Persistence)?;
        Ok(if existing == record.envelope_sha256() {
            InboxDecisionV1::Duplicate
        } else {
            InboxDecisionV1::HashConflict
        })
    }
}

impl OwnerOutboxStorePortV1 for PostgresOwnerDeliveryStore {
    #[allow(clippy::manual_async_fn)] // The owner outbox port requires a Send future.
    fn next_pending(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Option<OutboxEntryV1>, OutboxRelayErrorV1>> + Send
    {
        async move {
            let row = sqlx::query(AssertSqlSafe(schema::next_pending(self.scaffold)))
                .fetch_optional(&self.pool)
                .await
                .map_err(|_| OutboxRelayErrorV1::Persistence)?;
            row.map(decode_entry).transpose()
        }
    }

    #[allow(clippy::manual_async_fn)] // The owner outbox port requires a Send future.
    fn mark_published(
        &mut self,
        entry: &OutboxEntryV1,
        receipt: &OutboxPublishReceiptV1,
    ) -> impl std::future::Future<Output = Result<(), OutboxRelayErrorV1>> + Send {
        async move {
            let result = sqlx::query(AssertSqlSafe(schema::mark_published(self.scaffold)))
                .bind(entry.outbox_id())
                .bind(receipt.stream())
                .bind(
                    i64::try_from(receipt.sequence())
                        .map_err(|_| OutboxRelayErrorV1::Persistence)?,
                )
                .execute(&self.pool)
                .await
                .map_err(|_| OutboxRelayErrorV1::Persistence)?;
            (result.rows_affected() == 1)
                .then_some(())
                .ok_or(OutboxRelayErrorV1::Persistence)
        }
    }
}

fn decode_entry(row: sqlx::postgres::PgRow) -> Result<OutboxEntryV1, OutboxRelayErrorV1> {
    let outbox_id: String = value(&row, "outbox_id")?;
    let message_id: Vec<u8> = value(&row, "message_id")?;
    let hash: Vec<u8> = value(&row, "envelope_sha256")?;
    let exact_bytes: Vec<u8> = value(&row, "exact_envelope")?;
    let record =
        OutboxRecordV1::accept(exact_bytes).map_err(|_| OutboxRelayErrorV1::Persistence)?;
    (message_id == record.message_id() && hash == record.envelope_sha256())
        .then_some(record)
        .ok_or(OutboxRelayErrorV1::Persistence)
        .and_then(|record| OutboxEntryV1::new(outbox_id, record))
}

fn value<T>(row: &sqlx::postgres::PgRow, column: &str) -> Result<T, OutboxRelayErrorV1>
where
    for<'row> T: sqlx::Decode<'row, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    row.try_get(column)
        .map_err(|_| OutboxRelayErrorV1::Persistence)
}
