use sqlx::Row;

use crate::{
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1, OutboxEnvelopeV1,
    model::{MAIL_CONTACTS_SYNC_OUTBOX_LIMIT_V1, nonzero, valid_identity},
};

impl MailContactsSyncPersistenceV1 {
    pub async fn unpublished_commands(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<Vec<OutboxEnvelopeV1>, MailContactsSyncPersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !(1..=MAIL_CONTACTS_SYNC_OUTBOX_LIMIT_V1).contains(&limit)
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT message_id, envelope_sha256, envelope_bytes
             FROM makosh_data.mail_contacts_sync_outbox
             WHERE logical_owner_id = $1 AND published_at_unix_millis IS NULL
             ORDER BY created_at_unix_millis, message_id
             LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| MailContactsSyncPersistenceErrorV1::StorageUnavailable)?
        .into_iter()
        .map(outbox_from_row)
        .collect()
    }

    pub async fn mark_command_published(
        &self,
        logical_owner_id: &str,
        message_id: &[u8; 16],
        envelope_sha256: &[u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), MailContactsSyncPersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !nonzero(message_id)
            || !nonzero(envelope_sha256)
            || published_at_unix_millis <= 0
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_contacts_sync_outbox
             SET published_at_unix_millis = $1
             WHERE logical_owner_id = $2 AND message_id = $3
               AND envelope_sha256 = $4
               AND published_at_unix_millis IS NULL
               AND created_at_unix_millis <= $1",
        )
        .bind(published_at_unix_millis)
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .execute(&self.pool)
        .await
        .map_err(|_| MailContactsSyncPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if updated == 1 {
            Ok(())
        } else {
            Err(MailContactsSyncPersistenceErrorV1::RevisionConflict)
        }
    }
}

fn outbox_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<OutboxEnvelopeV1, MailContactsSyncPersistenceErrorV1> {
    let message_id: Vec<u8> = row
        .try_get("message_id")
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let envelope_sha256: Vec<u8> = row
        .try_get("envelope_sha256")
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let envelope = OutboxEnvelopeV1 {
        message_id: message_id
            .try_into()
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
        envelope_sha256: envelope_sha256
            .try_into()
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
        envelope_bytes: row
            .try_get("envelope_bytes")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
    };
    if crate::model::valid_envelope(&envelope) {
        Ok(envelope)
    } else {
        Err(MailContactsSyncPersistenceErrorV1::InvalidRow)
    }
}
