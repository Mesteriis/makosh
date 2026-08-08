use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::{Postgres, Transaction};

use crate::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
    event_outbox::{OUTBOX_DELIVERY_SUBMIT, OUTBOX_SOURCE_PREPARE, insert_exact_outbox},
    operations::{STATE_ACCEPTED, STATE_DISPATCHING, STATE_PREPARING_SOURCE, STATE_REJECTED},
    realtime::insert_forward_transition,
    valid_bounded_identity, valid_id16, valid_timestamp,
};

const INBOX_SOURCE_PREPARED: i16 = 1;
const INBOX_SOURCE_REJECTED: i16 = 2;
const MAX_BODY_BYTES_V1: u64 = 64 * 1024;
const MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardPreparedEventV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub forward_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub target_conversation_id: [u8; 16],
    pub source_evidence_id: [u8; 16],
    pub source_evidence_revision: u64,
    pub source_body: CrossChannelForwardBlobReceiptV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardRejectedEventV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub forward_id: [u8; 16],
    pub rejection_code: u16,
}

impl CommunicationCrossChannelForwardPersistenceV1 {
    pub async fn persist_source_prepare_outbox(
        &self,
        logical_owner_id: &str,
        forward_id: [u8; 16],
        source_prepare_outbox: &OutboxRecordV1,
        created_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !valid_id16(&forward_id)
            || !valid_timestamp(created_at_unix_millis)
            || source_prepare_outbox.message_id() != &forward_id
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin().await?;
        let inserted = insert_exact_outbox(
            &mut transaction,
            logical_owner_id,
            &forward_id,
            OUTBOX_SOURCE_PREPARE,
            source_prepare_outbox,
            created_at_unix_millis,
        )
        .await?;
        if !inserted {
            return commit(transaction).await;
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_operations
             SET state = $1, state_revision = state_revision + 1,
                 claimed_by = NULL, lease_expires_at_unix_millis = NULL,
                 updated_at_unix_millis = $2
             WHERE logical_owner_id = $3 AND forward_id = $4
               AND state = $5",
        )
        .bind(STATE_PREPARING_SOURCE)
        .bind(created_at_unix_millis)
        .bind(logical_owner_id)
        .bind(forward_id.as_slice())
        .bind(STATE_ACCEPTED)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(CrossChannelForwardPersistenceErrorV1::Conflict);
        }
        insert_forward_transition(
            &mut transaction,
            logical_owner_id,
            &forward_id,
            created_at_unix_millis,
        )
        .await?;
        commit(transaction).await
    }

    pub async fn persist_source_prepared_and_delivery_submit(
        &self,
        prepared: &CrossChannelForwardPreparedEventV1,
        delivery_body: &CrossChannelForwardBlobReceiptV1,
        delivery_submit_outbox: &OutboxRecordV1,
        consumed_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_prepared_event(prepared)
            || !valid_blob_receipt(delivery_body)
            || prepared.source_body.declared_bytes != delivery_body.declared_bytes
            || prepared.source_body.sha256 != delivery_body.sha256
            || !valid_timestamp(consumed_at_unix_millis)
            || !valid_id16(delivery_submit_outbox.message_id())
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin().await?;
        if inbox_duplicate(
            &mut transaction,
            prepared.result_message_id,
            prepared.envelope_sha256,
            INBOX_SOURCE_PREPARED,
            &prepared.logical_owner_id,
            prepared.forward_id,
            consumed_at_unix_millis,
        )
        .await?
        {
            return commit(transaction).await;
        }
        let _inserted = insert_exact_outbox(
            &mut transaction,
            &prepared.logical_owner_id,
            &prepared.forward_id,
            OUTBOX_DELIVERY_SUBMIT,
            delivery_submit_outbox,
            consumed_at_unix_millis,
        )
        .await?;
        let source_revision = i64::try_from(prepared.source_evidence_revision)
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let source_body_length = i32::try_from(prepared.source_body.declared_bytes)
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let delivery_body_length = i64::try_from(delivery_body.declared_bytes)
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_operations
             SET state = $1, state_revision = state_revision + 1,
                 source_evidence_id = $2, source_revision = $3,
                 source_body_sha256 = $4, source_body_length = $5,
                 source_blob_reference = $6, source_custody_proof = $7,
                 source_result_message_id = $8,
                 delivery_body_reference_id = $9,
                 delivery_body_declared_bytes = $10,
                 delivery_body_sha256 = $11,
                 delivery_body_custody_proof = $12,
                 delivery_intent_command_id = $13,
                 delivery_submit_message_id = $14,
                 attempt_count = 0, claimed_by = NULL,
                 lease_expires_at_unix_millis = NULL,
                 updated_at_unix_millis = $15
             WHERE logical_owner_id = $16 AND forward_id = $17
               AND source_message_id = $18 AND target_conversation_id = $19
               AND state = $20 AND source_result_message_id IS NULL",
        )
        .bind(STATE_DISPATCHING)
        .bind(prepared.source_evidence_id.as_slice())
        .bind(source_revision)
        .bind(prepared.source_body.sha256.as_slice())
        .bind(source_body_length)
        .bind(prepared.source_body.reference_id.as_slice())
        .bind(&prepared.source_body.custody_transfer_source_proof)
        .bind(prepared.result_message_id.as_slice())
        .bind(delivery_body.reference_id.as_slice())
        .bind(delivery_body_length)
        .bind(delivery_body.sha256.as_slice())
        .bind(&delivery_body.custody_transfer_source_proof)
        .bind(prepared.forward_id.as_slice())
        .bind(delivery_submit_outbox.message_id().as_slice())
        .bind(consumed_at_unix_millis)
        .bind(&prepared.logical_owner_id)
        .bind(prepared.forward_id.as_slice())
        .bind(prepared.source_message_id.as_slice())
        .bind(prepared.target_conversation_id.as_slice())
        .bind(STATE_PREPARING_SOURCE)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(CrossChannelForwardPersistenceErrorV1::Conflict);
        }
        insert_forward_transition(
            &mut transaction,
            &prepared.logical_owner_id,
            &prepared.forward_id,
            consumed_at_unix_millis,
        )
        .await?;
        commit(transaction).await
    }

    pub async fn persist_source_rejected(
        &self,
        rejected: &CrossChannelForwardRejectedEventV1,
        consumed_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_rejected_event(rejected) || !valid_timestamp(consumed_at_unix_millis) {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin().await?;
        if inbox_duplicate(
            &mut transaction,
            rejected.result_message_id,
            rejected.envelope_sha256,
            INBOX_SOURCE_REJECTED,
            &rejected.logical_owner_id,
            rejected.forward_id,
            consumed_at_unix_millis,
        )
        .await?
        {
            return commit(transaction).await;
        }
        let error_code = i16::try_from(rejected.rejection_code)
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_operations
             SET state = $1, state_revision = state_revision + 1,
                 source_result_message_id = $2, error_code = $3,
                 claimed_by = NULL, lease_expires_at_unix_millis = NULL,
                 updated_at_unix_millis = $4
             WHERE logical_owner_id = $5 AND forward_id = $6
               AND state = $7 AND source_result_message_id IS NULL",
        )
        .bind(STATE_REJECTED)
        .bind(rejected.result_message_id.as_slice())
        .bind(error_code)
        .bind(consumed_at_unix_millis)
        .bind(&rejected.logical_owner_id)
        .bind(rejected.forward_id.as_slice())
        .bind(STATE_PREPARING_SOURCE)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(CrossChannelForwardPersistenceErrorV1::Conflict);
        }
        insert_forward_transition(
            &mut transaction,
            &rejected.logical_owner_id,
            &rejected.forward_id,
            consumed_at_unix_millis,
        )
        .await?;
        commit(transaction).await
    }

    pub(crate) async fn begin(
        &self,
    ) -> Result<Transaction<'_, Postgres>, CrossChannelForwardPersistenceErrorV1> {
        self.pool.begin().await.map_err(storage_error)
    }
}

pub(crate) async fn inbox_duplicate(
    transaction: &mut Transaction<'_, Postgres>,
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    event_kind: i16,
    logical_owner_id: &str,
    forward_id: [u8; 16],
    consumed_at_unix_millis: i64,
) -> Result<bool, CrossChannelForwardPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_cross_channel_forward_event_inbox (
            message_id, envelope_sha256, event_kind, logical_owner_id,
            forward_id, consumed_at_unix_millis
         ) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT DO NOTHING",
    )
    .bind(message_id.as_slice())
    .bind(envelope_sha256.as_slice())
    .bind(event_kind)
    .bind(logical_owner_id)
    .bind(forward_id.as_slice())
    .bind(consumed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if inserted.rows_affected() == 1 {
        return Ok(false);
    }
    let existing: Option<(Vec<u8>, i16, String, Vec<u8>)> = sqlx::query_as(
        "SELECT envelope_sha256, event_kind, logical_owner_id, forward_id
         FROM makosh_data.communication_cross_channel_forward_event_inbox
         WHERE message_id = $1",
    )
    .bind(message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if existing
        .as_ref()
        .is_some_and(|(hash, kind, owner, existing_forward)| {
            hash.as_slice() == envelope_sha256
                && *kind == event_kind
                && owner == logical_owner_id
                && existing_forward.as_slice() == forward_id
        })
    {
        Ok(true)
    } else {
        Err(CrossChannelForwardPersistenceErrorV1::Conflict)
    }
}

fn valid_prepared_event(value: &CrossChannelForwardPreparedEventV1) -> bool {
    valid_id16(&value.result_message_id)
        && valid_sha256(&value.envelope_sha256)
        && valid_bounded_identity(&value.logical_owner_id)
        && valid_id16(&value.forward_id)
        && valid_id16(&value.source_message_id)
        && valid_id16(&value.target_conversation_id)
        && valid_id16(&value.source_evidence_id)
        && value.source_evidence_revision > 0
        && valid_blob_receipt(&value.source_body)
}

fn valid_rejected_event(value: &CrossChannelForwardRejectedEventV1) -> bool {
    valid_id16(&value.result_message_id)
        && valid_sha256(&value.envelope_sha256)
        && valid_bounded_identity(&value.logical_owner_id)
        && valid_id16(&value.forward_id)
        && (1..=7).contains(&value.rejection_code)
}

fn valid_blob_receipt(value: &CrossChannelForwardBlobReceiptV1) -> bool {
    valid_id16(&value.reference_id)
        && (1..=MAX_BODY_BYTES_V1).contains(&value.declared_bytes)
        && valid_sha256(&value.sha256)
        && (1..=MAX_CUSTODY_PROOF_BYTES_V1).contains(&value.custody_transfer_source_proof.len())
}

fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) async fn commit(
    transaction: Transaction<'_, Postgres>,
) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
    transaction.commit().await.map_err(storage_error)
}

fn storage_error(_: sqlx::Error) -> CrossChannelForwardPersistenceErrorV1 {
    CrossChannelForwardPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt() -> CrossChannelForwardBlobReceiptV1 {
        CrossChannelForwardBlobReceiptV1 {
            reference_id: [1; 16],
            declared_bytes: 42,
            sha256: [2; 32],
            custody_transfer_source_proof: vec![3; 64],
        }
    }

    #[test]
    fn event_handoff_is_bodyless_bounded_and_hash_preserving() {
        let prepared = CrossChannelForwardPreparedEventV1 {
            result_message_id: [4; 16],
            envelope_sha256: [5; 32],
            logical_owner_id: "owner-1".to_owned(),
            forward_id: [6; 16],
            source_message_id: [7; 16],
            target_conversation_id: [8; 16],
            source_evidence_id: [9; 16],
            source_evidence_revision: 1,
            source_body: receipt(),
        };
        assert!(valid_prepared_event(&prepared));
        let delivery = receipt();
        assert_eq!(prepared.source_body.declared_bytes, delivery.declared_bytes);
        assert_eq!(prepared.source_body.sha256, delivery.sha256);
    }

    #[test]
    fn rejection_codes_are_closed() {
        let mut rejected = CrossChannelForwardRejectedEventV1 {
            result_message_id: [1; 16],
            envelope_sha256: [2; 32],
            logical_owner_id: "owner-1".to_owned(),
            forward_id: [3; 16],
            rejection_code: 7,
        };
        assert!(valid_rejected_event(&rejected));
        rejected.rejection_code = 8;
        assert!(!valid_rejected_event(&rejected));
    }
}
