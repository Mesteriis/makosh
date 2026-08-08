use makosh_events_protocol::delivery::{OutboxRecordError, OutboxRecordV1};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    CommunicationDeliveryIntentPersistenceV1, CreateDeliveryIntentV1,
    DeliveryIntentPersistenceErrorV1, intents::create_intent_in_transaction,
    valid_bounded_identity, valid_id16, valid_id32, valid_timestamp,
};

const RESULT_SUBMITTED: i16 = 1;
const RESULT_REJECTED: i16 = 2;
const CLEANUP_SUBMITTED: i16 = 1;
const CLEANUP_REJECTED: i16 = 2;
type ExistingIngressRowV1 = (Vec<u8>, Vec<u8>, String, Vec<u8>);
type ExistingIngressResultRowV1 = (Vec<u8>, Vec<u8>, i16, String, Vec<u8>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryIntentIngressEventV1 {
    pub command_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub correlation_id: [u8; 16],
    pub logical_owner_id: String,
    pub intent_id: [u8; 16],
    pub body_receipt: DeliveryIntentIngressBlobReceiptV1,
    pub consumed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryIntentIngressBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_source_proof: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentIngressDispositionV1 {
    New,
    ExactDuplicate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentIngressResultKindV1 {
    Submitted,
    Rejected,
}

impl DeliveryIntentIngressResultKindV1 {
    const fn code(self) -> i16 {
        match self {
            Self::Submitted => RESULT_SUBMITTED,
            Self::Rejected => RESULT_REJECTED,
        }
    }
}

impl CommunicationDeliveryIntentPersistenceV1 {
    pub async fn inspect_event_ingress(
        &self,
        event: &DeliveryIntentIngressEventV1,
    ) -> Result<DeliveryIntentIngressDispositionV1, DeliveryIntentPersistenceErrorV1> {
        if !valid_event(event) {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let existing: Option<ExistingIngressRowV1> = sqlx::query_as(
            "SELECT envelope_sha256, correlation_id, logical_owner_id, intent_id
             FROM makosh_data.communication_delivery_intent_ingress_inbox
             WHERE command_message_id = $1
                OR (logical_owner_id = $2 AND intent_id = $3)
             ORDER BY CASE WHEN command_message_id = $1 THEN 0 ELSE 1 END
             LIMIT 1",
        )
        .bind(event.command_message_id.as_slice())
        .bind(&event.logical_owner_id)
        .bind(event.intent_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?;
        match existing {
            None => Ok(DeliveryIntentIngressDispositionV1::New),
            Some((hash, correlation, owner, intent))
                if hash.as_slice() == event.envelope_sha256
                    && correlation.as_slice() == event.correlation_id
                    && owner == event.logical_owner_id
                    && intent.as_slice() == event.intent_id =>
            {
                Ok(DeliveryIntentIngressDispositionV1::ExactDuplicate)
            }
            Some(_) => Err(DeliveryIntentPersistenceErrorV1::Conflict),
        }
    }

    pub async fn admit_event_ingress(
        &self,
        event: &DeliveryIntentIngressEventV1,
        command: &CreateDeliveryIntentV1,
        result: &OutboxRecordV1,
    ) -> Result<DeliveryIntentIngressDispositionV1, DeliveryIntentPersistenceErrorV1> {
        if event.logical_owner_id != command.logical_owner_id
            || event.intent_id != command.intent_id
        {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let disposition = insert_or_fence_inbox(&mut transaction, event).await?;
        if disposition == DeliveryIntentIngressDispositionV1::ExactDuplicate {
            validate_existing_result(&mut transaction, event, result, RESULT_SUBMITTED).await?;
            return commit(transaction, disposition).await;
        }
        create_intent_in_transaction(&mut transaction, command).await?;
        insert_cleanup(&mut transaction, event, CLEANUP_SUBMITTED).await?;
        insert_exact_result(
            &mut transaction,
            event,
            result,
            DeliveryIntentIngressResultKindV1::Submitted,
        )
        .await?;
        commit(transaction, disposition).await
    }

    pub async fn reject_event_ingress(
        &self,
        event: &DeliveryIntentIngressEventV1,
        result: &OutboxRecordV1,
    ) -> Result<DeliveryIntentIngressDispositionV1, DeliveryIntentPersistenceErrorV1> {
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let disposition = insert_or_fence_inbox(&mut transaction, event).await?;
        if disposition == DeliveryIntentIngressDispositionV1::ExactDuplicate {
            validate_existing_result(&mut transaction, event, result, RESULT_REJECTED).await?;
            return commit(transaction, disposition).await;
        }
        insert_cleanup(&mut transaction, event, CLEANUP_REJECTED).await?;
        insert_exact_result(
            &mut transaction,
            event,
            result,
            DeliveryIntentIngressResultKindV1::Rejected,
        )
        .await?;
        commit(transaction, disposition).await
    }

    pub async fn pending_ingress_results(
        &self,
        limit: u32,
    ) -> Result<Vec<OutboxRecordV1>, DeliveryIntentPersistenceErrorV1> {
        if limit == 0 || limit > 256 {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT exact_envelope_bytes
             FROM makosh_data.communication_delivery_intent_ingress_result_outbox
             WHERE published_at_unix_seconds IS NULL
             ORDER BY created_at_unix_seconds, message_id
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
                        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?,
                )
                .map_err(outbox_error)
            })
            .collect()
    }

    pub async fn mark_ingress_result_published(
        &self,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<(), DeliveryIntentPersistenceErrorV1> {
        if !valid_id16(&message_id) || !valid_timestamp(published_at_unix_seconds) {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "UPDATE makosh_data.communication_delivery_intent_ingress_result_outbox
             SET published_at_unix_seconds = $2
             WHERE message_id = $1 AND published_at_unix_seconds IS NULL",
        )
        .bind(message_id.as_slice())
        .bind(published_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(storage_error)
    }
}

async fn insert_cleanup(
    transaction: &mut Transaction<'_, Postgres>,
    event: &DeliveryIntentIngressEventV1,
    reason: i16,
) -> Result<(), DeliveryIntentPersistenceErrorV1> {
    let declared_bytes = i64::try_from(event.body_receipt.declared_bytes)
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?;
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_delivery_intent_ingress_cleanup (
           logical_owner_id, intent_id, reference_id, declared_bytes,
           sha256, custody_source_proof, reason, attempt_count,
           next_attempt_at_unix_seconds, created_at_unix_seconds,
           updated_at_unix_seconds
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, 0, $8, $8, $8)
         ON CONFLICT (logical_owner_id, intent_id) DO NOTHING",
    )
    .bind(&event.logical_owner_id)
    .bind(event.intent_id.as_slice())
    .bind(event.body_receipt.reference_id.as_slice())
    .bind(declared_bytes)
    .bind(event.body_receipt.sha256.as_slice())
    .bind(&event.body_receipt.custody_source_proof)
    .bind(reason)
    .bind(event.consumed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if inserted.rows_affected() == 1 {
        Ok(())
    } else {
        Err(DeliveryIntentPersistenceErrorV1::Conflict)
    }
}

async fn insert_or_fence_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    event: &DeliveryIntentIngressEventV1,
) -> Result<DeliveryIntentIngressDispositionV1, DeliveryIntentPersistenceErrorV1> {
    if !valid_event(event) {
        return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
    }
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_delivery_intent_ingress_inbox (
            command_message_id, envelope_sha256, correlation_id,
            logical_owner_id, intent_id, consumed_at_unix_seconds
         ) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT DO NOTHING",
    )
    .bind(event.command_message_id.as_slice())
    .bind(event.envelope_sha256.as_slice())
    .bind(event.correlation_id.as_slice())
    .bind(&event.logical_owner_id)
    .bind(event.intent_id.as_slice())
    .bind(event.consumed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if inserted.rows_affected() == 1 {
        return Ok(DeliveryIntentIngressDispositionV1::New);
    }
    let existing: Option<ExistingIngressRowV1> = sqlx::query_as(
        "SELECT envelope_sha256, correlation_id, logical_owner_id, intent_id
         FROM makosh_data.communication_delivery_intent_ingress_inbox
         WHERE command_message_id = $1
            OR (logical_owner_id = $2 AND intent_id = $3)
         ORDER BY CASE WHEN command_message_id = $1 THEN 0 ELSE 1 END
         LIMIT 1",
    )
    .bind(event.command_message_id.as_slice())
    .bind(&event.logical_owner_id)
    .bind(event.intent_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if existing
        .as_ref()
        .is_some_and(|(hash, correlation, owner, intent)| {
            hash.as_slice() == event.envelope_sha256
                && correlation.as_slice() == event.correlation_id
                && owner == &event.logical_owner_id
                && intent.as_slice() == event.intent_id
        })
    {
        Ok(DeliveryIntentIngressDispositionV1::ExactDuplicate)
    } else {
        Err(DeliveryIntentPersistenceErrorV1::Conflict)
    }
}

async fn insert_exact_result(
    transaction: &mut Transaction<'_, Postgres>,
    event: &DeliveryIntentIngressEventV1,
    result: &OutboxRecordV1,
    result_kind: DeliveryIntentIngressResultKindV1,
) -> Result<(), DeliveryIntentPersistenceErrorV1> {
    if !valid_id16(result.message_id()) || result.message_id() == &event.command_message_id {
        return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
    }
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_delivery_intent_ingress_result_outbox (
            message_id, envelope_sha256, exact_envelope_bytes, result_kind,
            logical_owner_id, intent_id, command_message_id,
            created_at_unix_seconds, published_at_unix_seconds
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL)
         ON CONFLICT DO NOTHING",
    )
    .bind(result.message_id().as_slice())
    .bind(result.envelope_sha256().as_slice())
    .bind(result.exact_bytes())
    .bind(result_kind.code())
    .bind(&event.logical_owner_id)
    .bind(event.intent_id.as_slice())
    .bind(event.command_message_id.as_slice())
    .bind(event.consumed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if inserted.rows_affected() == 1 {
        return Ok(());
    }
    validate_existing_result(transaction, event, result, result_kind.code()).await
}

async fn validate_existing_result(
    transaction: &mut Transaction<'_, Postgres>,
    event: &DeliveryIntentIngressEventV1,
    result: &OutboxRecordV1,
    result_kind: i16,
) -> Result<(), DeliveryIntentPersistenceErrorV1> {
    let existing: Option<ExistingIngressResultRowV1> = sqlx::query_as(
        "SELECT envelope_sha256, exact_envelope_bytes, result_kind,
                logical_owner_id, intent_id
         FROM makosh_data.communication_delivery_intent_ingress_result_outbox
         WHERE command_message_id = $1",
    )
    .bind(event.command_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if existing
        .as_ref()
        .is_some_and(|(hash, exact_bytes, kind, owner, intent)| {
            hash.as_slice() == result.envelope_sha256()
                && exact_bytes.as_slice() == result.exact_bytes()
                && *kind == result_kind
                && owner == &event.logical_owner_id
                && intent.as_slice() == event.intent_id
        })
    {
        Ok(())
    } else {
        Err(DeliveryIntentPersistenceErrorV1::Conflict)
    }
}

fn valid_event(event: &DeliveryIntentIngressEventV1) -> bool {
    valid_id16(&event.command_message_id)
        && valid_id32(&event.envelope_sha256)
        && valid_id16(&event.correlation_id)
        && valid_bounded_identity(&event.logical_owner_id)
        && valid_id16(&event.intent_id)
        && valid_blob_receipt(&event.body_receipt)
        && valid_timestamp(event.consumed_at_unix_seconds)
}

fn valid_blob_receipt(receipt: &DeliveryIntentIngressBlobReceiptV1) -> bool {
    valid_id16(&receipt.reference_id)
        && (1..=16 * 1024 * 1024).contains(&receipt.declared_bytes)
        && valid_id32(&receipt.sha256)
        && (1..=2_048).contains(&receipt.custody_source_proof.len())
}

async fn commit(
    transaction: Transaction<'_, Postgres>,
    disposition: DeliveryIntentIngressDispositionV1,
) -> Result<DeliveryIntentIngressDispositionV1, DeliveryIntentPersistenceErrorV1> {
    transaction.commit().await.map_err(storage_error)?;
    Ok(disposition)
}

fn storage_error(_: sqlx::Error) -> DeliveryIntentPersistenceErrorV1 {
    DeliveryIntentPersistenceErrorV1::StorageUnavailable
}

fn outbox_error(_: OutboxRecordError) -> DeliveryIntentPersistenceErrorV1 {
    DeliveryIntentPersistenceErrorV1::InvalidRow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingress_identity_requires_exact_owner_message_hash_and_correlation() {
        let event = DeliveryIntentIngressEventV1 {
            command_message_id: [1; 16],
            envelope_sha256: [2; 32],
            correlation_id: [3; 16],
            logical_owner_id: "owner-1".to_owned(),
            intent_id: [4; 16],
            body_receipt: DeliveryIntentIngressBlobReceiptV1 {
                reference_id: [5; 16],
                declared_bytes: 42,
                sha256: [6; 32],
                custody_source_proof: vec![7; 64],
            },
            consumed_at_unix_seconds: 5,
        };
        assert!(valid_event(&event));
        assert!(!valid_event(&DeliveryIntentIngressEventV1 {
            correlation_id: [0; 16],
            ..event
        }));
    }
}
