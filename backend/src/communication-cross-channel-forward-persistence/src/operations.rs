use makosh_communication_cross_channel_forward_core::{
    CrossChannelForwardDraftV1, CrossChannelForwardStateV1, validate_cross_channel_forward_v1,
};
use sha2::{Digest, Sha256};
use sqlx::Row;

use crate::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
    realtime::insert_forward_transition, valid_bounded_identity, valid_id16, valid_timestamp,
};

pub(crate) const STATE_ACCEPTED: i16 = 1;
pub(crate) const STATE_PREPARING_SOURCE: i16 = 2;
pub(crate) const STATE_DISPATCHING: i16 = 3;
pub(crate) const STATE_DELIVERY_ACCEPTED: i16 = 4;
pub(crate) const STATE_REJECTED: i16 = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCrossChannelForwardV1 {
    pub logical_owner_id: String,
    pub draft: CrossChannelForwardDraftV1,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreateCrossChannelForwardOutcomeV1 {
    Created { state_revision: u64 },
    Existing { state_revision: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardStatusRecordV1 {
    pub forward_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub target_conversation_id: [u8; 16],
    pub target_reply_to_message_id: Option<[u8; 16]>,
    pub state: CrossChannelForwardStateV1,
    pub state_revision: u64,
    pub delivery_intent_id: Option<[u8; 16]>,
    pub error_code: Option<u16>,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

impl CommunicationCrossChannelForwardPersistenceV1 {
    pub async fn create_forward(
        &self,
        command: CreateCrossChannelForwardV1,
    ) -> Result<CreateCrossChannelForwardOutcomeV1, CrossChannelForwardPersistenceErrorV1> {
        if !valid_bounded_identity(&command.logical_owner_id)
            || !valid_timestamp(command.created_at_unix_millis)
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let draft = validate_cross_channel_forward_v1(command.draft)
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let fingerprint = request_fingerprint(&draft);
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communication_cross_channel_forward_operations (
               logical_owner_id, forward_id, request_fingerprint,
               source_message_id, target_conversation_id,
               target_reply_message_id, state, state_revision,
               attempt_count, next_attempt_at_unix_millis, claim_epoch,
               created_at_unix_millis, updated_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, 1, 0, $8, 0, $8, $8)
             ON CONFLICT (logical_owner_id, forward_id) DO NOTHING",
        )
        .bind(&command.logical_owner_id)
        .bind(draft.forward_operation_id.as_slice())
        .bind(fingerprint.as_slice())
        .bind(draft.source_message_id.as_slice())
        .bind(draft.target_conversation_id.as_slice())
        .bind(draft.target_reply_to_message_id.map(|id| id.to_vec()))
        .bind(STATE_ACCEPTED)
        .bind(command.created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?
        .rows_affected()
            == 1;
        if inserted {
            insert_forward_transition(
                &mut transaction,
                &command.logical_owner_id,
                &draft.forward_operation_id,
                command.created_at_unix_millis,
            )
            .await?;
        }
        let row = sqlx::query(
            "SELECT request_fingerprint, state_revision
             FROM makosh_data.communication_cross_channel_forward_operations
             WHERE logical_owner_id = $1 AND forward_id = $2",
        )
        .bind(&command.logical_owner_id)
        .bind(draft.forward_operation_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        let stored_fingerprint: Vec<u8> = row
            .try_get("request_fingerprint")
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
        if stored_fingerprint.as_slice() != fingerprint {
            return Err(CrossChannelForwardPersistenceErrorV1::Conflict);
        }
        let state_revision = positive_u64(
            row.try_get("state_revision")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?;
        transaction
            .commit()
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        Ok(if inserted {
            CreateCrossChannelForwardOutcomeV1::Created { state_revision }
        } else {
            CreateCrossChannelForwardOutcomeV1::Existing { state_revision }
        })
    }

    pub async fn status(
        &self,
        logical_owner_id: &str,
        forward_id: &[u8; 16],
    ) -> Result<CrossChannelForwardStatusRecordV1, CrossChannelForwardPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id) || !valid_id16(forward_id) {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT forward_id, source_message_id, target_conversation_id,
                    target_reply_message_id, state, state_revision,
                    delivery_intent_id, delivery_intent_command_id,
                    delivery_submit_message_id, error_code,
                    created_at_unix_millis, updated_at_unix_millis
             FROM makosh_data.communication_cross_channel_forward_operations
             WHERE logical_owner_id = $1 AND forward_id = $2",
        )
        .bind(logical_owner_id)
        .bind(forward_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?
        .ok_or(CrossChannelForwardPersistenceErrorV1::NotFound)?;
        status_from_row(&row)
    }
}

pub(crate) fn status_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<CrossChannelForwardStatusRecordV1, CrossChannelForwardPersistenceErrorV1> {
    let created_at_unix_millis: i64 = row
        .try_get("created_at_unix_millis")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let updated_at_unix_millis: i64 = row
        .try_get("updated_at_unix_millis")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    if !valid_timestamp(created_at_unix_millis)
        || !valid_timestamp(updated_at_unix_millis)
        || updated_at_unix_millis < created_at_unix_millis
    {
        return Err(CrossChannelForwardPersistenceErrorV1::InvalidRow);
    }
    let error_code: Option<i16> = row
        .try_get("error_code")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let terminal_delivery_intent_id: Option<Vec<u8>> = row
        .try_get("delivery_intent_id")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let submitted_delivery_intent_id: Option<Vec<u8>> =
        row.try_get("delivery_intent_command_id")
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    if terminal_delivery_intent_id
        .as_ref()
        .zip(submitted_delivery_intent_id.as_ref())
        .is_some_and(|(terminal, submitted)| terminal != submitted)
    {
        return Err(CrossChannelForwardPersistenceErrorV1::InvalidRow);
    }
    let delivery_intent_id = terminal_delivery_intent_id.or(submitted_delivery_intent_id);
    Ok(CrossChannelForwardStatusRecordV1 {
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
        target_reply_to_message_id: optional_id16(
            row.try_get("target_reply_message_id")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        state: state_from_code(
            row.try_get("state")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        state_revision: positive_u64(
            row.try_get("state_revision")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        delivery_intent_id: optional_id16(delivery_intent_id)?,
        error_code: error_code
            .map(|value| {
                u16::try_from(value)
                    .ok()
                    .filter(|value| (1..=7).contains(value))
                    .ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)
            })
            .transpose()?,
        created_at_unix_millis,
        updated_at_unix_millis,
    })
}

pub(crate) fn state_from_code(
    state: i16,
) -> Result<CrossChannelForwardStateV1, CrossChannelForwardPersistenceErrorV1> {
    match state {
        STATE_ACCEPTED => Ok(CrossChannelForwardStateV1::Accepted),
        STATE_PREPARING_SOURCE => Ok(CrossChannelForwardStateV1::PreparingSource),
        STATE_DISPATCHING => Ok(CrossChannelForwardStateV1::Dispatching),
        STATE_DELIVERY_ACCEPTED => Ok(CrossChannelForwardStateV1::DeliveryAccepted),
        STATE_REJECTED => Ok(CrossChannelForwardStateV1::Rejected),
        _ => Err(CrossChannelForwardPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn id16(value: Vec<u8>) -> Result<[u8; 16], CrossChannelForwardPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_id16)
        .ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)
}

pub(crate) fn optional_id16(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; 16]>, CrossChannelForwardPersistenceErrorV1> {
    value.map(id16).transpose()
}

pub(crate) fn positive_u64(value: i64) -> Result<u64, CrossChannelForwardPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)
}

fn request_fingerprint(draft: &CrossChannelForwardDraftV1) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(draft.forward_operation_id);
    digest.update(draft.source_message_id);
    digest.update(draft.target_conversation_id);
    match draft.target_reply_to_message_id {
        Some(reply) => {
            digest.update([1]);
            digest.update(reply);
        }
        None => digest.update([0]),
    }
    digest.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(reply: Option<[u8; 16]>) -> CrossChannelForwardDraftV1 {
        CrossChannelForwardDraftV1 {
            forward_operation_id: [1; 16],
            source_message_id: [2; 16],
            target_conversation_id: [3; 16],
            target_reply_to_message_id: reply,
        }
    }

    #[test]
    fn request_fingerprint_is_stable_and_covers_every_identity() {
        assert_eq!(
            request_fingerprint(&draft(Some([4; 16]))),
            request_fingerprint(&draft(Some([4; 16])))
        );
        assert_ne!(
            request_fingerprint(&draft(None)),
            request_fingerprint(&draft(Some([4; 16])))
        );
        let mut other_source = draft(None);
        other_source.source_message_id = [5; 16];
        assert_ne!(
            request_fingerprint(&draft(None)),
            request_fingerprint(&other_source)
        );
    }
}
