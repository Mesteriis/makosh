use makosh_communication_delivery_intent_core::CommunicationProviderProvenanceV1;
use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::Row;

use crate::{
    CommunicationDeliveryIntentPersistenceV1,
    intents::{
        DeliveryIntentClaimV1, DeliveryIntentPersistenceErrorV1, DeliveryIntentStatusRecordV1,
        STATE_PROVIDER_CONFIRMED, STATE_REJECTED, STATE_RESOLVING_ROUTE,
        STATE_SUBMITTED_TO_PROVIDER, insert_transition, provider_code, status_from_row,
        valid_claim,
    },
    valid_bounded_identity, valid_id16, valid_id32, valid_timestamp,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnqueueProviderCommandOutcomeV1 {
    Created,
    Existing,
}

#[derive(Clone, Debug)]
pub struct ProviderCommandOutboxEntryV1 {
    pub provider: CommunicationProviderProvenanceV1,
    pub record: OutboxRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalDeliveryResultValueV1 {
    Succeeded { provider_operation_id: Vec<u8> },
    Rejected { rejection_code: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalDeliveryResultV1 {
    pub envelope_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub command_message_id: [u8; 16],
    pub logical_owner_id: String,
    pub intent_id: [u8; 16],
    pub value: TerminalDeliveryResultValueV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplyTerminalDeliveryResultOutcomeV1 {
    Applied(DeliveryIntentStatusRecordV1),
    Duplicate(DeliveryIntentStatusRecordV1),
}

impl CommunicationDeliveryIntentPersistenceV1 {
    pub async fn enqueue_provider_command(
        &self,
        claim: &DeliveryIntentClaimV1,
        provider: CommunicationProviderProvenanceV1,
        record: &OutboxRecordV1,
        now_unix_seconds: i64,
    ) -> Result<EnqueueProviderCommandOutcomeV1, DeliveryIntentPersistenceErrorV1> {
        if !valid_claim(claim)
            || claim.route.provider != provider
            || !valid_timestamp(now_unix_seconds)
        {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let claim_epoch = i64::try_from(claim.claim_epoch)
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?;
        let provider_kind = provider_code(provider);
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        let claim_exists = sqlx::query_scalar::<_, i32>(
            "SELECT 1
             FROM makosh_data.communication_delivery_intent_jobs
             WHERE logical_owner_id = $1 AND intent_id = $2
               AND state = $3 AND claimed_by = $4 AND claim_epoch = $5
               AND lease_expires_at_unix_seconds >= $6
             FOR UPDATE",
        )
        .bind(&claim.logical_owner_id)
        .bind(claim.intent_id.as_slice())
        .bind(STATE_RESOLVING_ROUTE)
        .bind(&claim.worker_id)
        .bind(claim_epoch)
        .bind(now_unix_seconds)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
        .is_some();
        if !claim_exists {
            return Err(DeliveryIntentPersistenceErrorV1::ClaimLost);
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communication_delivery_intent_provider_outbox (
               message_id, envelope_sha256, exact_envelope_bytes,
               logical_owner_id, intent_id, provider_kind, claim_epoch,
               created_at_unix_seconds
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(record.message_id().as_slice())
        .bind(record.envelope_sha256().as_slice())
        .bind(record.exact_bytes())
        .bind(&claim.logical_owner_id)
        .bind(claim.intent_id.as_slice())
        .bind(provider_kind)
        .bind(claim_epoch)
        .bind(now_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
        .rows_affected()
            == 1;
        let outcome = if inserted {
            EnqueueProviderCommandOutcomeV1::Created
        } else {
            let row = sqlx::query(
                "SELECT envelope_sha256, exact_envelope_bytes,
                        logical_owner_id, intent_id, provider_kind, claim_epoch
                 FROM makosh_data.communication_delivery_intent_provider_outbox
                 WHERE message_id = $1",
            )
            .bind(record.message_id().as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
            .ok_or(DeliveryIntentPersistenceErrorV1::Conflict)?;
            let existing_sha256: Vec<u8> = row
                .try_get("envelope_sha256")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            let existing_bytes: Vec<u8> = row
                .try_get("exact_envelope_bytes")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            let existing_owner: String = row
                .try_get("logical_owner_id")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            let existing_intent: Vec<u8> = row
                .try_get("intent_id")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            let existing_provider: i16 = row
                .try_get("provider_kind")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            let existing_claim_epoch: i64 = row
                .try_get("claim_epoch")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            if existing_sha256.as_slice() != record.envelope_sha256().as_slice()
                || existing_bytes.as_slice() != record.exact_bytes()
                || existing_owner != claim.logical_owner_id
                || existing_intent.as_slice() != claim.intent_id.as_slice()
                || existing_provider != provider_kind
                || existing_claim_epoch != claim_epoch
            {
                return Err(DeliveryIntentPersistenceErrorV1::Conflict);
            }
            EnqueueProviderCommandOutcomeV1::Existing
        };
        transaction
            .commit()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        Ok(outcome)
    }

    pub async fn pending_provider_commands(
        &self,
        provider: CommunicationProviderProvenanceV1,
        limit: i64,
    ) -> Result<Vec<ProviderCommandOutboxEntryV1>, DeliveryIntentPersistenceErrorV1> {
        if !(1..=256).contains(&limit) {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT exact_envelope_bytes
             FROM makosh_data.communication_delivery_intent_provider_outbox
             WHERE provider_kind = $1 AND published_at_unix_seconds IS NULL
             ORDER BY created_at_unix_seconds ASC, message_id ASC
             LIMIT $2",
        )
        .bind(provider_code(provider))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        rows.into_iter()
            .map(|row| {
                let exact_bytes: Vec<u8> = row
                    .try_get("exact_envelope_bytes")
                    .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
                let record = OutboxRecordV1::accept(exact_bytes)
                    .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
                Ok(ProviderCommandOutboxEntryV1 { provider, record })
            })
            .collect()
    }

    pub async fn provider_command_for_claim(
        &self,
        claim: &DeliveryIntentClaimV1,
    ) -> Result<Option<ProviderCommandOutboxEntryV1>, DeliveryIntentPersistenceErrorV1> {
        if !valid_claim(claim) {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT exact_envelope_bytes
             FROM makosh_data.communication_delivery_intent_provider_outbox
             WHERE logical_owner_id = $1
               AND intent_id = $2
               AND provider_kind = $3
               AND published_at_unix_seconds IS NULL",
        )
        .bind(&claim.logical_owner_id)
        .bind(claim.intent_id.as_slice())
        .bind(provider_code(claim.route.provider))
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        row.map(|row| {
            let exact_bytes: Vec<u8> = row
                .try_get("exact_envelope_bytes")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            let record = OutboxRecordV1::accept(exact_bytes)
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            Ok(ProviderCommandOutboxEntryV1 {
                provider: claim.route.provider,
                record,
            })
        })
        .transpose()
    }

    pub async fn mark_provider_command_published(
        &self,
        claim: &DeliveryIntentClaimV1,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentPersistenceErrorV1> {
        if !valid_claim(claim)
            || !valid_id16(&message_id)
            || !valid_timestamp(published_at_unix_seconds)
        {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let claim_epoch = i64::try_from(claim.claim_epoch)
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        let outbox = sqlx::query(
            "SELECT published_at_unix_seconds
             FROM makosh_data.communication_delivery_intent_provider_outbox
             WHERE message_id = $1 AND logical_owner_id = $2 AND intent_id = $3
               AND provider_kind = $4
             FOR UPDATE",
        )
        .bind(message_id.as_slice())
        .bind(&claim.logical_owner_id)
        .bind(claim.intent_id.as_slice())
        .bind(provider_code(claim.route.provider))
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
        .ok_or(DeliveryIntentPersistenceErrorV1::Conflict)?;
        let already_published: Option<i64> = outbox
            .try_get("published_at_unix_seconds")
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
        if already_published.is_some() {
            let status =
                status_in_transaction(&mut transaction, &claim.logical_owner_id, &claim.intent_id)
                    .await?;
            transaction
                .commit()
                .await
                .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
            return Ok(status);
        }
        let row = sqlx::query(
            "UPDATE makosh_data.communication_delivery_intent_jobs
             SET state = $1,
                 state_revision = state_revision + 1,
                 provider_operation_id = $2,
                 claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL,
                 updated_at_unix_seconds = $3
             WHERE logical_owner_id = $4 AND intent_id = $5
               AND state = $6 AND claimed_by = $7 AND claim_epoch = $8
               AND lease_expires_at_unix_seconds >= $3
             RETURNING intent_id, state, state_revision,
                       provider_operation_id, rejection_code",
        )
        .bind(STATE_SUBMITTED_TO_PROVIDER)
        .bind(message_id.as_slice())
        .bind(published_at_unix_seconds)
        .bind(&claim.logical_owner_id)
        .bind(claim.intent_id.as_slice())
        .bind(STATE_RESOLVING_ROUTE)
        .bind(&claim.worker_id)
        .bind(claim_epoch)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
        .ok_or(DeliveryIntentPersistenceErrorV1::ClaimLost)?;
        let status = status_from_row(&row)?;
        sqlx::query(
            "UPDATE makosh_data.communication_delivery_intent_provider_outbox
             SET published_at_unix_seconds = $1
             WHERE message_id = $2 AND published_at_unix_seconds IS NULL",
        )
        .bind(published_at_unix_seconds)
        .bind(message_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        insert_transition(
            &mut transaction,
            &claim.logical_owner_id,
            &claim.intent_id,
            status.state_revision,
            STATE_SUBMITTED_TO_PROVIDER,
            None,
            published_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        Ok(status)
    }

    pub async fn apply_terminal_result(
        &self,
        result: &TerminalDeliveryResultV1,
        consumed_at_unix_seconds: i64,
    ) -> Result<ApplyTerminalDeliveryResultOutcomeV1, DeliveryIntentPersistenceErrorV1> {
        if !valid_terminal_result(result) || !valid_timestamp(consumed_at_unix_seconds) {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        let existing = sqlx::query(
            "SELECT envelope_sha256, logical_owner_id, intent_id, command_message_id
             FROM makosh_data.communication_delivery_intent_result_inbox
             WHERE message_id = $1
             FOR UPDATE",
        )
        .bind(result.envelope_message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        if let Some(existing) = existing {
            let digest: Vec<u8> = existing
                .try_get("envelope_sha256")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            let owner: String = existing
                .try_get("logical_owner_id")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            let intent: Vec<u8> = existing
                .try_get("intent_id")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            let command: Vec<u8> = existing
                .try_get("command_message_id")
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
            if digest.as_slice() != result.envelope_sha256.as_slice()
                || owner != result.logical_owner_id
                || intent.as_slice() != result.intent_id.as_slice()
                || command.as_slice() != result.command_message_id.as_slice()
            {
                return Err(DeliveryIntentPersistenceErrorV1::Conflict);
            }
            let status = status_in_transaction(
                &mut transaction,
                &result.logical_owner_id,
                &result.intent_id,
            )
            .await?;
            transaction
                .commit()
                .await
                .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
            return Ok(ApplyTerminalDeliveryResultOutcomeV1::Duplicate(status));
        }
        let command_exists = sqlx::query_scalar::<_, i32>(
            "SELECT 1
             FROM makosh_data.communication_delivery_intent_provider_outbox
             WHERE message_id = $1 AND logical_owner_id = $2 AND intent_id = $3
               AND published_at_unix_seconds IS NOT NULL",
        )
        .bind(result.command_message_id.as_slice())
        .bind(&result.logical_owner_id)
        .bind(result.intent_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
        .is_some();
        if !command_exists {
            return Err(DeliveryIntentPersistenceErrorV1::Conflict);
        }
        let (target_state, provider_operation_id, rejection_code) = match &result.value {
            TerminalDeliveryResultValueV1::Succeeded {
                provider_operation_id,
            } => (
                STATE_PROVIDER_CONFIRMED,
                Some(provider_operation_id.as_slice()),
                None,
            ),
            TerminalDeliveryResultValueV1::Rejected { rejection_code } => (
                STATE_REJECTED,
                None,
                Some(
                    i16::try_from(*rejection_code)
                        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?,
                ),
            ),
        };
        let row = sqlx::query(
            "UPDATE makosh_data.communication_delivery_intent_jobs
             SET state = $1,
                 state_revision = state_revision + 1,
                 provider_operation_id = $2,
                 rejection_code = $3,
                 updated_at_unix_seconds = $4
             WHERE logical_owner_id = $5 AND intent_id = $6 AND state = $7
             RETURNING intent_id, state, state_revision,
                       provider_operation_id, rejection_code",
        )
        .bind(target_state)
        .bind(provider_operation_id)
        .bind(rejection_code)
        .bind(consumed_at_unix_seconds)
        .bind(&result.logical_owner_id)
        .bind(result.intent_id.as_slice())
        .bind(STATE_SUBMITTED_TO_PROVIDER)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
        .ok_or(DeliveryIntentPersistenceErrorV1::Conflict)?;
        let status = status_from_row(&row)?;
        insert_transition(
            &mut transaction,
            &result.logical_owner_id,
            &result.intent_id,
            status.state_revision,
            target_state,
            status.rejection_code,
            consumed_at_unix_seconds,
        )
        .await?;
        sqlx::query(
            "INSERT INTO makosh_data.communication_delivery_intent_result_inbox (
               message_id, envelope_sha256, logical_owner_id, intent_id,
               command_message_id, consumed_at_unix_seconds
             ) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(result.envelope_message_id.as_slice())
        .bind(result.envelope_sha256.as_slice())
        .bind(&result.logical_owner_id)
        .bind(result.intent_id.as_slice())
        .bind(result.command_message_id.as_slice())
        .bind(consumed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        transaction
            .commit()
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        Ok(ApplyTerminalDeliveryResultOutcomeV1::Applied(status))
    }
}

fn valid_terminal_result(result: &TerminalDeliveryResultV1) -> bool {
    valid_id16(&result.envelope_message_id)
        && valid_id32(&result.envelope_sha256)
        && valid_id16(&result.command_message_id)
        && valid_bounded_identity(&result.logical_owner_id)
        && valid_id16(&result.intent_id)
        && match &result.value {
            TerminalDeliveryResultValueV1::Succeeded {
                provider_operation_id,
            } => !provider_operation_id.is_empty() && provider_operation_id.len() <= 256,
            TerminalDeliveryResultValueV1::Rejected { rejection_code } => {
                (1..=32).contains(rejection_code)
            }
        }
}

async fn status_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    intent_id: &[u8; 16],
) -> Result<DeliveryIntentStatusRecordV1, DeliveryIntentPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT intent_id, state, state_revision,
                provider_operation_id, rejection_code
         FROM makosh_data.communication_delivery_intent_jobs
         WHERE logical_owner_id = $1 AND intent_id = $2",
    )
    .bind(logical_owner_id)
    .bind(intent_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?
    .ok_or(DeliveryIntentPersistenceErrorV1::Conflict)?;
    status_from_row(&row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_validation_rejects_missing_identity_and_unbounded_receipt() {
        let mut result = TerminalDeliveryResultV1 {
            envelope_message_id: [1; 16],
            envelope_sha256: [2; 32],
            command_message_id: [3; 16],
            logical_owner_id: "owner-1".to_owned(),
            intent_id: [4; 16],
            value: TerminalDeliveryResultValueV1::Succeeded {
                provider_operation_id: vec![5; 256],
            },
        };
        assert!(valid_terminal_result(&result));
        result.envelope_message_id = [0; 16];
        assert!(!valid_terminal_result(&result));
        result.envelope_message_id = [1; 16];
        result.value = TerminalDeliveryResultValueV1::Succeeded {
            provider_operation_id: vec![5; 257],
        };
        assert!(!valid_terminal_result(&result));
    }
}
