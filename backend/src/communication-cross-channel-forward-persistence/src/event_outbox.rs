use makosh_events_protocol::delivery::{OutboxRecordError, OutboxRecordV1};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
    valid_id16, valid_timestamp,
};

pub(crate) const OUTBOX_SOURCE_PREPARE: i16 = 1;
pub(crate) const OUTBOX_DELIVERY_SUBMIT: i16 = 2;
type ExistingOutboxRowV1 = (Vec<u8>, Vec<u8>, i16, String, Vec<u8>);

impl CommunicationCrossChannelForwardPersistenceV1 {
    pub async fn pending_event_outbox(
        &self,
        limit: u32,
    ) -> Result<Vec<OutboxRecordV1>, CrossChannelForwardPersistenceErrorV1> {
        if limit == 0 || limit > 256 {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT exact_envelope_bytes
             FROM makosh_data.communication_cross_channel_forward_event_outbox
             WHERE published_at_unix_millis IS NULL
             ORDER BY created_at_unix_millis, message_id
             LIMIT $1",
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        rows.into_iter()
            .map(|row| {
                OutboxRecordV1::accept(
                    row.try_get::<Vec<u8>, _>("exact_envelope_bytes")
                        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
                )
                .map_err(outbox_record_error)
            })
            .collect()
    }

    pub async fn mark_event_outbox_published(
        &self,
        message_id: [u8; 16],
        published_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_id16(&message_id) || !valid_timestamp(published_at_unix_millis) {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_event_outbox
             SET published_at_unix_millis = $2
             WHERE message_id = $1 AND published_at_unix_millis IS NULL",
        )
        .bind(message_id.as_slice())
        .bind(published_at_unix_millis)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(storage_error)
    }
}

pub(crate) async fn insert_exact_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    forward_id: &[u8; 16],
    event_kind: i16,
    outbox: &OutboxRecordV1,
    created_at_unix_millis: i64,
) -> Result<bool, CrossChannelForwardPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_cross_channel_forward_event_outbox (
            message_id, envelope_sha256, exact_envelope_bytes, event_kind,
            logical_owner_id, forward_id, created_at_unix_millis,
            published_at_unix_millis
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL)
         ON CONFLICT DO NOTHING",
    )
    .bind(outbox.message_id().as_slice())
    .bind(outbox.envelope_sha256().as_slice())
    .bind(outbox.exact_bytes())
    .bind(event_kind)
    .bind(logical_owner_id)
    .bind(forward_id.as_slice())
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if inserted.rows_affected() == 1 {
        return Ok(true);
    }
    let existing: Option<ExistingOutboxRowV1> = sqlx::query_as(
        "SELECT envelope_sha256, exact_envelope_bytes, event_kind,
                logical_owner_id, forward_id
         FROM makosh_data.communication_cross_channel_forward_event_outbox
         WHERE message_id = $1
            OR (
              logical_owner_id = $2
              AND forward_id = $3
              AND event_kind = $4
            )
         ORDER BY CASE WHEN message_id = $1 THEN 0 ELSE 1 END
         LIMIT 1",
    )
    .bind(outbox.message_id().as_slice())
    .bind(logical_owner_id)
    .bind(forward_id.as_slice())
    .bind(event_kind)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if existing
        .as_ref()
        .is_some_and(|(hash, exact_bytes, kind, owner, existing_forward)| {
            hash.as_slice() == outbox.envelope_sha256()
                && exact_bytes.as_slice() == outbox.exact_bytes()
                && *kind == event_kind
                && owner == logical_owner_id
                && existing_forward.as_slice() == forward_id
        })
    {
        Ok(false)
    } else {
        Err(CrossChannelForwardPersistenceErrorV1::Conflict)
    }
}

fn storage_error(_: sqlx::Error) -> CrossChannelForwardPersistenceErrorV1 {
    CrossChannelForwardPersistenceErrorV1::StorageUnavailable
}

fn outbox_record_error(_: OutboxRecordError) -> CrossChannelForwardPersistenceErrorV1 {
    CrossChannelForwardPersistenceErrorV1::InvalidRow
}
