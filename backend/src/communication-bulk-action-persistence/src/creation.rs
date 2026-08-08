use makosh_communication_bulk_action_core::{BulkDeliveryDraftV1, validate_bulk_delivery_v1};
use sha2::{Digest, Sha256};
use sqlx::Row;

use crate::{
    BulkDeliveryPersistenceErrorV1, CommunicationBulkActionPersistenceV1,
    realtime::insert_batch_transition, valid_bounded_identity, valid_timestamp,
};

const STATE_PENDING: i16 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateBulkDeliveryV1 {
    pub logical_owner_id: String,
    pub draft: BulkDeliveryDraftV1,
    pub created_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreateBulkDeliveryOutcomeV1 {
    Created { state_revision: u64 },
    Existing { state_revision: u64 },
}

impl CommunicationBulkActionPersistenceV1 {
    pub async fn create_batch(
        &self,
        command: CreateBulkDeliveryV1,
    ) -> Result<CreateBulkDeliveryOutcomeV1, BulkDeliveryPersistenceErrorV1> {
        if !valid_bounded_identity(&command.logical_owner_id)
            || !valid_timestamp(command.created_at_unix_seconds)
        {
            return Err(BulkDeliveryPersistenceErrorV1::InvalidInput);
        }
        let draft = validate_bulk_delivery_v1(command.draft)
            .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidInput)?;
        let fingerprint = request_fingerprint(&draft);
        let target_count = i16::try_from(draft.targets.len())
            .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communication_bulk_action_batches (
               logical_owner_id, batch_id, request_fingerprint, target_count,
               state_revision, created_at_unix_seconds, updated_at_unix_seconds
             ) VALUES ($1, $2, $3, $4, 1, $5, $5)
             ON CONFLICT (logical_owner_id, batch_id) DO NOTHING",
        )
        .bind(&command.logical_owner_id)
        .bind(draft.batch_id.as_slice())
        .bind(fingerprint.as_slice())
        .bind(target_count)
        .bind(command.created_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected()
            == 1;

        if inserted {
            for (ordinal, target) in draft.targets.iter().enumerate() {
                let ordinal = i16::try_from(ordinal)
                    .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidInput)?;
                sqlx::query(
                    "INSERT INTO makosh_data.communication_bulk_action_targets (
                       logical_owner_id, batch_id, target_operation_id, ordinal,
                       canonical_conversation_id, canonical_reply_message_id,
                       body_utf8, state, attempt_count, claim_epoch,
                       created_at_unix_seconds, updated_at_unix_seconds
                     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, 0, $9, $9)",
                )
                .bind(&command.logical_owner_id)
                .bind(draft.batch_id.as_slice())
                .bind(target.operation_id.as_slice())
                .bind(ordinal)
                .bind(target.conversation_id.as_slice())
                .bind(target.reply_to_message_id.map(|id| id.to_vec()))
                .bind(&target.body_utf8)
                .bind(STATE_PENDING)
                .bind(command.created_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
            }
            insert_batch_transition(
                &mut transaction,
                &command.logical_owner_id,
                &draft.batch_id,
                command.created_at_unix_seconds,
            )
            .await?;
        }

        let row = sqlx::query(
            "SELECT request_fingerprint, target_count, state_revision
             FROM makosh_data.communication_bulk_action_batches
             WHERE logical_owner_id = $1 AND batch_id = $2",
        )
        .bind(&command.logical_owner_id)
        .bind(draft.batch_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let stored_fingerprint: Vec<u8> = row
            .try_get("request_fingerprint")
            .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?;
        let stored_target_count: i16 = row
            .try_get("target_count")
            .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?;
        if stored_fingerprint.as_slice() != fingerprint || stored_target_count != target_count {
            return Err(BulkDeliveryPersistenceErrorV1::Conflict);
        }
        let state_revision = positive_u64(
            row.try_get("state_revision")
                .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?,
        )?;
        transaction
            .commit()
            .await
            .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(if inserted {
            CreateBulkDeliveryOutcomeV1::Created { state_revision }
        } else {
            CreateBulkDeliveryOutcomeV1::Existing { state_revision }
        })
    }
}

fn request_fingerprint(draft: &BulkDeliveryDraftV1) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(draft.batch_id);
    digest.update((draft.targets.len() as u64).to_be_bytes());
    for target in &draft.targets {
        digest.update(target.operation_id);
        digest.update(target.conversation_id);
        match target.reply_to_message_id {
            Some(reply) => {
                digest.update([1]);
                digest.update(reply);
            }
            None => digest.update([0]),
        }
        digest.update((target.body_utf8.len() as u64).to_be_bytes());
        digest.update(&target.body_utf8);
    }
    digest.finalize().into()
}

fn positive_u64(value: i64) -> Result<u64, BulkDeliveryPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(BulkDeliveryPersistenceErrorV1::InvalidRow)
}

#[cfg(test)]
mod tests {
    use makosh_communication_bulk_action_core::BulkDeliveryTargetDraftV1;

    use super::*;

    fn draft(body: &[u8]) -> BulkDeliveryDraftV1 {
        BulkDeliveryDraftV1 {
            batch_id: [1; 16],
            targets: vec![BulkDeliveryTargetDraftV1 {
                operation_id: [2; 16],
                conversation_id: [3; 16],
                reply_to_message_id: None,
                body_utf8: body.to_vec(),
            }],
        }
    }

    #[test]
    fn fingerprint_is_stable_and_covers_private_body() {
        assert_eq!(
            request_fingerprint(&draft(b"one")),
            request_fingerprint(&draft(b"one"))
        );
        assert_ne!(
            request_fingerprint(&draft(b"one")),
            request_fingerprint(&draft(b"two"))
        );
    }
}
