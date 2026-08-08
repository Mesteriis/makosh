use sqlx::Row;

use crate::{
    CommunicationTaskCandidatePersistenceErrorV1, CommunicationTaskCandidatePersistenceV1,
    UnpublishedCommunicationTaskCandidateEventV1,
    model::{
        COMMUNICATION_TASK_CANDIDATE_MAX_EVENT_BYTES_V1,
        COMMUNICATION_TASK_CANDIDATE_OUTBOX_LIMIT_V1, nonzero, valid_identity, valid_timestamp,
    },
};

impl CommunicationTaskCandidatePersistenceV1 {
    pub async fn unpublished_events(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<
        Vec<UnpublishedCommunicationTaskCandidateEventV1>,
        CommunicationTaskCandidatePersistenceErrorV1,
    > {
        if !valid_identity(logical_owner_id)
            || !(1..=COMMUNICATION_TASK_CANDIDATE_OUTBOX_LIMIT_V1).contains(&limit)
        {
            return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT message_id, envelope_sha256, envelope_bytes
             FROM makosh_data.communication_task_candidate_extraction_outbox
             WHERE logical_owner_id = $1 AND published_at_unix_millis IS NULL
             ORDER BY created_at_unix_millis, message_id
             LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::StorageUnavailable)?
        .into_iter()
        .map(|row| {
            let message_id: Vec<u8> = row
                .try_get("message_id")
                .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?;
            let envelope_sha256: Vec<u8> = row
                .try_get("envelope_sha256")
                .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?;
            let envelope_bytes: Vec<u8> = row
                .try_get("envelope_bytes")
                .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?;
            let event = UnpublishedCommunicationTaskCandidateEventV1 {
                message_id: message_id
                    .try_into()
                    .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?,
                envelope_sha256: envelope_sha256
                    .try_into()
                    .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?,
                envelope_bytes,
            };
            if !nonzero(&event.message_id)
                || !nonzero(&event.envelope_sha256)
                || event.envelope_bytes.is_empty()
                || event.envelope_bytes.len() > COMMUNICATION_TASK_CANDIDATE_MAX_EVENT_BYTES_V1
            {
                return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow);
            }
            Ok(event)
        })
        .collect()
    }

    pub async fn mark_event_published(
        &self,
        logical_owner_id: &str,
        message_id: &[u8; 16],
        envelope_sha256: &[u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), CommunicationTaskCandidatePersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !nonzero(message_id)
            || !nonzero(envelope_sha256)
            || !valid_timestamp(published_at_unix_millis)
        {
            return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_task_candidate_extraction_outbox
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
        .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if updated == 1 {
            Ok(())
        } else {
            Err(CommunicationTaskCandidatePersistenceErrorV1::RevisionConflict)
        }
    }
}
