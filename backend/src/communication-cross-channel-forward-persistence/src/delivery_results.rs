use sqlx::{Postgres, Transaction};

use crate::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
    event_io::{commit, inbox_duplicate},
    operations::{STATE_DELIVERY_ACCEPTED, STATE_DISPATCHING, STATE_REJECTED, id16},
    realtime::insert_forward_transition,
    valid_bounded_identity, valid_id16, valid_timestamp,
};

const INBOX_DELIVERY_SUBMITTED: i16 = 3;
const INBOX_DELIVERY_REJECTED: i16 = 4;
const CLEANUP_DELIVERY_ACCEPTED: i16 = 1;
const CLEANUP_REJECTED: i16 = 2;

type DispatchRowV1 = (Vec<u8>, Option<Vec<u8>>, Option<Vec<u8>>);

struct DeliveryResultTransitionV1<'a> {
    result_message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    logical_owner_id: &'a str,
    delivery_intent_id: [u8; 16],
    delivery_submit_message_id: [u8; 16],
    rejection_code: Option<u16>,
    inbox_kind: i16,
    terminal_state: i16,
    cleanup_reason: i16,
    consumed_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardDeliverySubmittedEventV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub delivery_intent_id: [u8; 16],
    pub delivery_submit_message_id: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardDeliveryRejectedEventV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub delivery_intent_id: [u8; 16],
    pub delivery_submit_message_id: [u8; 16],
    pub rejection_code: u16,
}

impl CommunicationCrossChannelForwardPersistenceV1 {
    pub async fn persist_delivery_submitted(
        &self,
        event: &CrossChannelForwardDeliverySubmittedEventV1,
        consumed_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_submitted(event) || !valid_timestamp(consumed_at_unix_millis) {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        self.persist_delivery_result(DeliveryResultTransitionV1 {
            result_message_id: event.result_message_id,
            envelope_sha256: event.envelope_sha256,
            logical_owner_id: &event.logical_owner_id,
            delivery_intent_id: event.delivery_intent_id,
            delivery_submit_message_id: event.delivery_submit_message_id,
            rejection_code: None,
            inbox_kind: INBOX_DELIVERY_SUBMITTED,
            terminal_state: STATE_DELIVERY_ACCEPTED,
            cleanup_reason: CLEANUP_DELIVERY_ACCEPTED,
            consumed_at_unix_millis,
        })
        .await
    }

    pub async fn persist_delivery_rejected(
        &self,
        event: &CrossChannelForwardDeliveryRejectedEventV1,
        consumed_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_rejected(event) || !valid_timestamp(consumed_at_unix_millis) {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        self.persist_delivery_result(DeliveryResultTransitionV1 {
            result_message_id: event.result_message_id,
            envelope_sha256: event.envelope_sha256,
            logical_owner_id: &event.logical_owner_id,
            delivery_intent_id: event.delivery_intent_id,
            delivery_submit_message_id: event.delivery_submit_message_id,
            rejection_code: Some(event.rejection_code),
            inbox_kind: INBOX_DELIVERY_REJECTED,
            terminal_state: STATE_REJECTED,
            cleanup_reason: CLEANUP_REJECTED,
            consumed_at_unix_millis,
        })
        .await
    }

    async fn persist_delivery_result(
        &self,
        transition: DeliveryResultTransitionV1<'_>,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        let DeliveryResultTransitionV1 {
            result_message_id,
            envelope_sha256,
            logical_owner_id,
            delivery_intent_id,
            delivery_submit_message_id,
            rejection_code,
            inbox_kind,
            terminal_state,
            cleanup_reason,
            consumed_at_unix_millis,
        } = transition;
        let mut transaction = self.begin().await?;
        let (forward_id, source_reference, source_proof) = dispatch_row(
            &mut transaction,
            logical_owner_id,
            &delivery_intent_id,
            &delivery_submit_message_id,
        )
        .await?;
        if inbox_duplicate(
            &mut transaction,
            result_message_id,
            envelope_sha256,
            inbox_kind,
            logical_owner_id,
            forward_id,
            consumed_at_unix_millis,
        )
        .await?
        {
            return commit(transaction).await;
        }
        let source_reference =
            source_reference.ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
        let source_proof = source_proof.ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
        let cleanup_inserted = sqlx::query(
            "INSERT INTO makosh_data.communication_cross_channel_forward_cleanup (
               logical_owner_id, forward_id, source_blob_reference,
               source_custody_proof, reason, attempt_count,
               next_attempt_at_unix_millis, created_at_unix_millis,
               updated_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5, 0, $6, $6, $6)
             ON CONFLICT (logical_owner_id, forward_id) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(forward_id.as_slice())
        .bind(source_reference)
        .bind(source_proof)
        .bind(cleanup_reason)
        .bind(consumed_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if cleanup_inserted != 1 {
            return Err(CrossChannelForwardPersistenceErrorV1::Conflict);
        }
        let rejection_code = rejection_code
            .map(i16::try_from)
            .transpose()
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_operations
             SET state = $1, state_revision = state_revision + 1,
                 delivery_intent_id = $2, error_code = $3,
                 source_blob_reference = NULL, source_custody_proof = NULL,
                 updated_at_unix_millis = $4
             WHERE logical_owner_id = $5 AND forward_id = $6
               AND delivery_intent_command_id = $2
               AND delivery_submit_message_id = $7
               AND state = $8",
        )
        .bind(terminal_state)
        .bind(delivery_intent_id.as_slice())
        .bind(rejection_code)
        .bind(consumed_at_unix_millis)
        .bind(logical_owner_id)
        .bind(forward_id.as_slice())
        .bind(delivery_submit_message_id.as_slice())
        .bind(STATE_DISPATCHING)
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
            consumed_at_unix_millis,
        )
        .await?;
        commit(transaction).await
    }
}

async fn dispatch_row(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    delivery_intent_id: &[u8; 16],
    delivery_submit_message_id: &[u8; 16],
) -> Result<([u8; 16], Option<Vec<u8>>, Option<Vec<u8>>), CrossChannelForwardPersistenceErrorV1> {
    let row: DispatchRowV1 = sqlx::query_as(
        "SELECT forward_id, source_blob_reference, source_custody_proof
         FROM makosh_data.communication_cross_channel_forward_operations
         WHERE logical_owner_id = $1
           AND delivery_intent_command_id = $2
           AND delivery_submit_message_id = $3
         FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(delivery_intent_id.as_slice())
    .bind(delivery_submit_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?
    .ok_or(CrossChannelForwardPersistenceErrorV1::NotFound)?;
    Ok((id16(row.0)?, row.1, row.2))
}

fn valid_submitted(event: &CrossChannelForwardDeliverySubmittedEventV1) -> bool {
    valid_id16(&event.result_message_id)
        && event.envelope_sha256.iter().any(|byte| *byte != 0)
        && valid_bounded_identity(&event.logical_owner_id)
        && valid_id16(&event.delivery_intent_id)
        && valid_id16(&event.delivery_submit_message_id)
}

fn valid_rejected(event: &CrossChannelForwardDeliveryRejectedEventV1) -> bool {
    valid_id16(&event.result_message_id)
        && event.envelope_sha256.iter().any(|byte| *byte != 0)
        && valid_bounded_identity(&event.logical_owner_id)
        && valid_id16(&event.delivery_intent_id)
        && valid_id16(&event.delivery_submit_message_id)
        && (1..=5).contains(&event.rejection_code)
}

fn storage_error(_: sqlx::Error) -> CrossChannelForwardPersistenceErrorV1 {
    CrossChannelForwardPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_result_identity_is_bounded_and_closed() {
        let submitted = CrossChannelForwardDeliverySubmittedEventV1 {
            result_message_id: [1; 16],
            envelope_sha256: [2; 32],
            logical_owner_id: "owner-1".to_owned(),
            delivery_intent_id: [3; 16],
            delivery_submit_message_id: [4; 16],
        };
        assert!(valid_submitted(&submitted));
        assert!(valid_rejected(
            &CrossChannelForwardDeliveryRejectedEventV1 {
                result_message_id: submitted.result_message_id,
                envelope_sha256: submitted.envelope_sha256,
                logical_owner_id: submitted.logical_owner_id,
                delivery_intent_id: submitted.delivery_intent_id,
                delivery_submit_message_id: submitted.delivery_submit_message_id,
                rejection_code: 5,
            }
        ));
    }
}
