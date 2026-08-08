use sqlx::Row;

use crate::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
    operations::STATE_ACCEPTED, valid_bounded_identity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardSourcePrepareCandidateV1 {
    pub forward_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub target_conversation_id: [u8; 16],
}

impl CommunicationCrossChannelForwardPersistenceV1 {
    pub async fn next_source_prepare_candidate(
        &self,
        logical_owner_id: &str,
    ) -> Result<
        Option<CrossChannelForwardSourcePrepareCandidateV1>,
        CrossChannelForwardPersistenceErrorV1,
    > {
        if !valid_bounded_identity(logical_owner_id) {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT forward_id, source_message_id, target_conversation_id
             FROM makosh_data.communication_cross_channel_forward_operations
             WHERE logical_owner_id = $1 AND state = $2
             ORDER BY created_at_unix_millis, forward_id
             LIMIT 1",
        )
        .bind(logical_owner_id)
        .bind(STATE_ACCEPTED)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        row.map(|row| {
            Ok(CrossChannelForwardSourcePrepareCandidateV1 {
                forward_id: id16(
                    row.try_get("forward_id")
                        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
                )?,
                source_message_id: id16(
                    row.try_get("source_message_id")
                        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
                )?,
                target_conversation_id: id16(
                    row.try_get("target_conversation_id")
                        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
                )?,
            })
        })
        .transpose()
    }
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], CrossChannelForwardPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)
}
