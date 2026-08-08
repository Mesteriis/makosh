use sqlx::Row;

use crate::{
    CommunicationTranslationPersistenceErrorV1, CommunicationTranslationPersistenceV1,
    UnpublishedCommunicationTranslationEventV1,
    model::{
        COMMUNICATION_TRANSLATION_MAX_EVENT_BYTES_V1, COMMUNICATION_TRANSLATION_OUTBOX_LIMIT_V1,
        nonzero, valid_identity, valid_timestamp,
    },
};

impl CommunicationTranslationPersistenceV1 {
    pub async fn unpublished_source_prepare_events(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<
        Vec<UnpublishedCommunicationTranslationEventV1>,
        CommunicationTranslationPersistenceErrorV1,
    > {
        if !valid_identity(logical_owner_id)
            || !(1..=COMMUNICATION_TRANSLATION_OUTBOX_LIMIT_V1).contains(&limit)
        {
            return Err(CommunicationTranslationPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT message_id, envelope_sha256, envelope_bytes
             FROM makosh_data.communication_translation_outbox
             WHERE logical_owner_id = $1 AND published_at_unix_millis IS NULL
             ORDER BY created_at_unix_millis, message_id
             LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationTranslationPersistenceErrorV1::StorageUnavailable)?
        .into_iter()
        .map(|row| {
            let message_id: Vec<u8> = row
                .try_get("message_id")
                .map_err(|_| CommunicationTranslationPersistenceErrorV1::InvalidRow)?;
            let envelope_sha256: Vec<u8> = row
                .try_get("envelope_sha256")
                .map_err(|_| CommunicationTranslationPersistenceErrorV1::InvalidRow)?;
            let envelope_bytes: Vec<u8> = row
                .try_get("envelope_bytes")
                .map_err(|_| CommunicationTranslationPersistenceErrorV1::InvalidRow)?;
            let event = UnpublishedCommunicationTranslationEventV1 {
                message_id: message_id
                    .try_into()
                    .map_err(|_| CommunicationTranslationPersistenceErrorV1::InvalidRow)?,
                envelope_sha256: envelope_sha256
                    .try_into()
                    .map_err(|_| CommunicationTranslationPersistenceErrorV1::InvalidRow)?,
                envelope_bytes,
            };
            if !nonzero(&event.message_id)
                || !nonzero(&event.envelope_sha256)
                || event.envelope_bytes.is_empty()
                || event.envelope_bytes.len() > COMMUNICATION_TRANSLATION_MAX_EVENT_BYTES_V1
            {
                return Err(CommunicationTranslationPersistenceErrorV1::InvalidRow);
            }
            Ok(event)
        })
        .collect()
    }

    pub async fn mark_source_prepare_published(
        &self,
        logical_owner_id: &str,
        message_id: &[u8; 16],
        envelope_sha256: &[u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), CommunicationTranslationPersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !nonzero(message_id)
            || !nonzero(envelope_sha256)
            || !valid_timestamp(published_at_unix_millis)
        {
            return Err(CommunicationTranslationPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_translation_outbox
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
        .map_err(|_| CommunicationTranslationPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if updated == 1 {
            Ok(())
        } else {
            Err(CommunicationTranslationPersistenceErrorV1::RevisionConflict)
        }
    }
}
