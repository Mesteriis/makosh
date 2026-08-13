use makosh_obligations_core::{
    ObligationStatusV1, ObligationV1, create_obligation_from_reviewed_candidate_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::model::{
    OBLIGATIONS_RECOVERY_LIMIT_V1, valid_cleanup, valid_identity, valid_obligation, valid_outbox,
    valid_reservation,
};
use crate::row_codec::{decode_command, decode_outbox};
use crate::{
    CompleteReviewedCandidateObligationV1, ObligationsOutboxRecordV1,
    ObligationsPersistenceErrorV1, PersistReviewedCandidateMaterializationV1,
    PersistedReviewedCandidateCommandV1, RejectReviewedCandidateObligationV1,
    ReserveReviewedCandidateCommandOutcomeV1, ReserveReviewedCandidateCommandV1,
};

const COMMAND_COLUMNS: &str = "logical_owner_id, command_message_id, command_envelope_sha256, \
    command_id, command_fingerprint, approved_candidate_id, candidate_digest, \
    source_evidence_id, source_evidence_revision, review_id, decision_revision, \
    decided_by_owner_device_id, candidate_blob_reference_id, candidate_blob_declared_bytes, \
    candidate_blob_sha256, candidate_blob_custody_proof, materialized_blob_reference_id, \
    cleanup_completed_at_unix_millis, completed, rejected, obligation_id, received_at_unix_millis";

#[derive(Clone)]
pub struct ObligationsPersistenceV1 {
    pub(crate) pool: PgPool,
}

pub struct ObligationsOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: ObligationsOutboxRecordV1,
    created_at_unix_millis: i64,
}

impl ObligationsOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &ObligationsOutboxRecordV1 {
        &self.record
    }

    pub async fn mark_published(
        mut self,
        expected_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), ObligationsPersistenceErrorV1> {
        if expected_sha256 != self.record.envelope_sha256
            || Sha256::digest(&self.record.envelope_bytes).as_slice() != expected_sha256
            || published_at_unix_millis < self.created_at_unix_millis
        {
            return Err(ObligationsPersistenceErrorV1::InboxConflict);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.obligations_outbox SET published_at_unix_millis = $3 \
             WHERE logical_owner_id = $1 AND message_id = $2 AND envelope_sha256 = $4 \
             AND published_at_unix_millis IS NULL",
        )
        .bind(&self.logical_owner_id)
        .bind(self.record.message_id.as_slice())
        .bind(published_at_unix_millis)
        .bind(expected_sha256.as_slice())
        .execute(&mut *self.transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if affected != 1 {
            return Err(ObligationsPersistenceErrorV1::InboxConflict);
        }
        self.transaction.commit().await.map_err(storage)
    }
}

impl ObligationsPersistenceV1 {
    pub(crate) async fn begin_owner(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, ObligationsPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        Ok(transaction)
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ObligationsPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ObligationsPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(
                u16::try_from(pgbouncer_port)
                    .map_err(|_| ObligationsPersistenceErrorV1::StorageUnavailable)?,
            )
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(storage)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), ObligationsPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage)
    }

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn reserve_command(
        &self,
        input: &ReserveReviewedCandidateCommandV1,
    ) -> Result<ReserveReviewedCandidateCommandOutcomeV1, ObligationsPersistenceErrorV1> {
        if !valid_reservation(input) {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        let fingerprint = input.command_fingerprint();
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.obligations_reviewed_candidate_inbox (\
             logical_owner_id, command_message_id, command_envelope_sha256, command_id, \
             command_fingerprint, approved_candidate_id, candidate_digest, source_evidence_id, \
             source_evidence_revision, review_id, decision_revision, decided_by_owner_device_id, \
             candidate_blob_reference_id, candidate_blob_declared_bytes, candidate_blob_sha256, \
             candidate_blob_custody_proof, received_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17) \
             ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.command_message_id.as_slice())
        .bind(input.command_envelope_sha256.as_slice())
        .bind(input.command_id.as_slice())
        .bind(fingerprint.as_slice())
        .bind(input.approved_candidate_id.as_slice())
        .bind(input.candidate_digest.as_slice())
        .bind(input.source_evidence_id.as_slice())
        .bind(i64_value(input.source_evidence_revision)?)
        .bind(input.review_id.as_slice())
        .bind(i64_value(input.decision_revision)?)
        .bind(input.decided_by_owner_device_id.as_slice())
        .bind(input.candidate_content.reference_id.as_slice())
        .bind(i64_value(input.candidate_content.declared_bytes)?)
        .bind(input.candidate_content.sha256.as_slice())
        .bind(&input.candidate_content.custody_transfer_source_proof)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected()
            == 1;
        let persisted = load_command_in_transaction(
            &mut transaction,
            &input.logical_owner_id,
            input.command_message_id,
            false,
        )
        .await?
        .ok_or(ObligationsPersistenceErrorV1::CommandConflict)?;
        if persisted.command_message_id != input.command_message_id
            || persisted.command_envelope_sha256 != input.command_envelope_sha256
            || persisted.command_id != input.command_id
            || persisted.approved_candidate_id != input.approved_candidate_id
            || persisted.command_fingerprint != fingerprint
        {
            return Err(ObligationsPersistenceErrorV1::CommandConflict);
        }
        let outcome = if inserted {
            ReserveReviewedCandidateCommandOutcomeV1::Reserved(persisted)
        } else {
            ReserveReviewedCandidateCommandOutcomeV1::Existing(persisted)
        };
        transaction.commit().await.map_err(storage)?;
        Ok(outcome)
    }

    pub async fn persist_materialization(
        &self,
        input: &PersistReviewedCandidateMaterializationV1,
    ) -> Result<(), ObligationsPersistenceErrorV1> {
        if !valid_identity(&input.logical_owner_id) || !valid_cleanup(&input.materialization) {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        let result = sqlx::query(
            "UPDATE makosh_data.obligations_reviewed_candidate_inbox \
             SET materialized_blob_reference_id = $3 \
             WHERE logical_owner_id = $1 AND command_message_id = $2 \
             AND (materialized_blob_reference_id IS NULL OR materialized_blob_reference_id = $3)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.command_message_id.as_slice())
        .bind(input.materialization.reference_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if result.rows_affected() != 1 {
            return Err(ObligationsPersistenceErrorV1::CommandConflict);
        }
        transaction.commit().await.map_err(storage)
    }

    pub async fn complete_obligation(
        &self,
        input: CompleteReviewedCandidateObligationV1,
    ) -> Result<ObligationV1, ObligationsPersistenceErrorV1> {
        if !valid_identity(&input.logical_owner_id)
            || !valid_outbox(&input.created_result)
            || input.occurred_at_unix_millis <= 0
        {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let obligation = create_obligation_from_reviewed_candidate_v1(input.draft)
            .map_err(|_| ObligationsPersistenceErrorV1::InvalidInput)?;
        if !valid_obligation(&obligation) || obligation.logical_owner_id != input.logical_owner_id {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        let command = lock_command(
            &mut transaction,
            &input.logical_owner_id,
            input.command_message_id,
        )
        .await?;
        if command.approved_candidate_id != obligation.provenance.approved_candidate_id
            || command.candidate_digest != obligation.provenance.candidate_digest
            || command.source_evidence_id != obligation.provenance.source_evidence_id
            || command.source_evidence_revision != obligation.provenance.source_evidence_revision
            || command.review_id != obligation.provenance.review_id
            || command.decision_revision != obligation.provenance.decision_revision
            || command.decided_by_owner_device_id
                != obligation.provenance.decided_by_owner_device_id
        {
            return Err(ObligationsPersistenceErrorV1::ObligationConflict);
        }
        if command.completed {
            if !command.rejected && command.obligation_id == Some(obligation.obligation_id) {
                transaction.commit().await.map_err(storage)?;
                return Ok(obligation);
            }
            return Err(ObligationsPersistenceErrorV1::ObligationConflict);
        }
        insert_obligation(&mut transaction, &obligation).await?;
        let updated = sqlx::query(
            "UPDATE makosh_data.obligations_reviewed_candidate_inbox SET completed = TRUE, \
             obligation_id = $3, completed_at_unix_millis = $4 \
             WHERE logical_owner_id = $1 AND command_message_id = $2 AND NOT completed",
        )
        .bind(&input.logical_owner_id)
        .bind(input.command_message_id.as_slice())
        .bind(obligation.obligation_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if updated.rows_affected() != 1 {
            return Err(ObligationsPersistenceErrorV1::ObligationConflict);
        }
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.created_result,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(obligation)
    }

    pub async fn reject_obligation(
        &self,
        input: &RejectReviewedCandidateObligationV1,
    ) -> Result<(), ObligationsPersistenceErrorV1> {
        if !valid_identity(&input.logical_owner_id)
            || !valid_outbox(&input.rejected_result)
            || input.occurred_at_unix_millis <= 0
        {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        let command = lock_command(
            &mut transaction,
            &input.logical_owner_id,
            input.command_message_id,
        )
        .await?;
        if command.completed {
            return if command.rejected {
                transaction.commit().await.map_err(storage)
            } else {
                Err(ObligationsPersistenceErrorV1::ObligationConflict)
            };
        }
        sqlx::query(
            "UPDATE makosh_data.obligations_reviewed_candidate_inbox SET completed = TRUE, \
             rejected = TRUE, completed_at_unix_millis = $3 \
             WHERE logical_owner_id = $1 AND command_message_id = $2 AND NOT completed",
        )
        .bind(&input.logical_owner_id)
        .bind(input.command_message_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.rejected_result,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage)
    }

    pub async fn complete_blob_cleanup(
        &self,
        logical_owner_id: &str,
        command_message_id: [u8; 16],
        completed_at_unix_millis: i64,
    ) -> Result<(), ObligationsPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) || completed_at_unix_millis <= 0 {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let result = sqlx::query(
            "UPDATE makosh_data.obligations_reviewed_candidate_inbox \
             SET cleanup_completed_at_unix_millis = $3 \
             WHERE logical_owner_id = $1 AND command_message_id = $2 \
             AND materialized_blob_reference_id IS NOT NULL \
             AND (cleanup_completed_at_unix_millis IS NULL OR cleanup_completed_at_unix_millis = $3)",
        )
        .bind(logical_owner_id)
        .bind(command_message_id.as_slice())
        .bind(completed_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if result.rows_affected() != 1 {
            return Err(ObligationsPersistenceErrorV1::CommandConflict);
        }
        transaction.commit().await.map_err(storage)
    }

    pub async fn load_recoverable_commands(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<PersistedReviewedCandidateCommandV1>, ObligationsPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let query = format!(
            "SELECT {COMMAND_COLUMNS} FROM makosh_data.obligations_reviewed_candidate_inbox \
             WHERE logical_owner_id = $1 AND (NOT completed OR \
             (materialized_blob_reference_id IS NOT NULL AND cleanup_completed_at_unix_millis IS NULL)) \
             ORDER BY received_at_unix_millis, command_message_id LIMIT $2"
        );
        let commands = sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(logical_owner_id)
            .bind(i64::from(OBLIGATIONS_RECOVERY_LIMIT_V1))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage)?
            .iter()
            .map(decode_command)
            .collect::<Result<Vec<_>, _>>()?;
        transaction.commit().await.map_err(storage)?;
        Ok(commands)
    }

    pub async fn claim_next_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Option<ObligationsOutboxPublishClaimV1>, ObligationsPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        let row = sqlx::query(
            "SELECT message_id, envelope_sha256, envelope_bytes, created_at_unix_millis \
             FROM makosh_data.obligations_outbox WHERE logical_owner_id = $1 \
             AND published_at_unix_millis IS NULL \
             ORDER BY created_at_unix_millis, message_id FOR UPDATE SKIP LOCKED LIMIT 1",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let Some(row) = row else {
            transaction.rollback().await.map_err(storage)?;
            return Ok(None);
        };
        let record = decode_outbox(&row)?;
        if !valid_outbox(&record) {
            return Err(ObligationsPersistenceErrorV1::InvalidRow);
        }
        let created_at_unix_millis = row.try_get("created_at_unix_millis").map_err(storage)?;
        Ok(Some(ObligationsOutboxPublishClaimV1 {
            transaction,
            logical_owner_id: logical_owner_id.to_owned(),
            record,
            created_at_unix_millis,
        }))
    }
}

async fn load_command_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    command_message_id: [u8; 16],
    lock: bool,
) -> Result<Option<PersistedReviewedCandidateCommandV1>, ObligationsPersistenceErrorV1> {
    let lock_clause = if lock { " FOR UPDATE" } else { "" };
    let query = format!(
        "SELECT {COMMAND_COLUMNS} FROM makosh_data.obligations_reviewed_candidate_inbox \
         WHERE logical_owner_id = $1 AND command_message_id = $2{lock_clause}"
    );
    sqlx::query(sqlx::AssertSqlSafe(query))
        .bind(logical_owner_id)
        .bind(command_message_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?
        .as_ref()
        .map(decode_command)
        .transpose()
}

async fn lock_command(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    command_message_id: [u8; 16],
) -> Result<PersistedReviewedCandidateCommandV1, ObligationsPersistenceErrorV1> {
    load_command_in_transaction(transaction, logical_owner_id, command_message_id, true)
        .await?
        .ok_or(ObligationsPersistenceErrorV1::NotFound)
}

async fn insert_obligation(
    transaction: &mut Transaction<'_, Postgres>,
    obligation: &ObligationV1,
) -> Result<(), ObligationsPersistenceErrorV1> {
    let status = match obligation.status {
        ObligationStatusV1::Open => 1_i16,
    };
    let result = sqlx::query(
        "INSERT INTO makosh_data.obligations_state (logical_owner_id, obligation_id, statement, condition, \
         due_at_unix_seconds, due_at_nanos, status, obligation_revision, obligated_party_id, \
         beneficiary_party_id, approved_candidate_id, candidate_digest, \
         source_evidence_id, source_evidence_revision, review_id, decision_revision, \
         decided_by_owner_device_id, created_at_unix_seconds, created_at_nanos, \
         updated_at_unix_seconds, updated_at_nanos) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21) \
         ON CONFLICT (logical_owner_id, approved_candidate_id) DO NOTHING",
    )
    .bind(&obligation.logical_owner_id)
    .bind(obligation.obligation_id.as_slice())
    .bind(&obligation.statement)
    .bind(&obligation.condition)
    .bind(obligation.due_at.map(|value| value.unix_seconds))
    .bind(obligation.due_at.map(|value| value.nanos))
    .bind(status)
    .bind(i64_value(obligation.obligation_revision)?)
    .bind(obligation.obligated_party_id.as_slice())
    .bind(obligation.beneficiary_party_id.map(|value| value.to_vec()))
    .bind(obligation.provenance.approved_candidate_id.as_slice())
    .bind(obligation.provenance.candidate_digest.as_slice())
    .bind(obligation.provenance.source_evidence_id.as_slice())
    .bind(i64_value(obligation.provenance.source_evidence_revision)?)
    .bind(obligation.provenance.review_id.as_slice())
    .bind(i64_value(obligation.provenance.decision_revision)?)
    .bind(obligation.provenance.decided_by_owner_device_id.as_slice())
    .bind(obligation.created_at.unix_seconds)
    .bind(obligation.created_at.nanos)
    .bind(obligation.updated_at.unix_seconds)
    .bind(obligation.updated_at.nanos)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(ObligationsPersistenceErrorV1::ObligationConflict);
    }
    for evidence in &obligation.evidence_links {
        sqlx::query(
            "INSERT INTO makosh_data.obligations_evidence (logical_owner_id, obligation_id, \
             evidence_link_id, evidence_owner_id, evidence_record_id, evidence_revision, evidence_digest) \
             VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(&obligation.logical_owner_id)
        .bind(obligation.obligation_id.as_slice())
        .bind(evidence.evidence_link_id.as_slice())
        .bind(&evidence.evidence_owner_id)
        .bind(evidence.evidence_record_id.as_slice())
        .bind(i64_value(evidence.evidence_revision)?)
        .bind(evidence.evidence_digest.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    }
    Ok(())
}

async fn insert_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    record: &ObligationsOutboxRecordV1,
    created_at_unix_millis: i64,
) -> Result<(), ObligationsPersistenceErrorV1> {
    let result = sqlx::query(
        "INSERT INTO makosh_data.obligations_outbox (logical_owner_id, message_id, envelope_sha256, \
         envelope_bytes, created_at_unix_millis) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(record.message_id.as_slice())
    .bind(record.envelope_sha256.as_slice())
    .bind(&record.envelope_bytes)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(ObligationsPersistenceErrorV1::InboxConflict);
    }
    Ok(())
}

fn i64_value(value: u64) -> Result<i64, ObligationsPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ObligationsPersistenceErrorV1::InvalidInput)
}

fn storage(_: sqlx::Error) -> ObligationsPersistenceErrorV1 {
    ObligationsPersistenceErrorV1::StorageUnavailable
}
