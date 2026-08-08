use makosh_knowledge_core::{
    KnowledgeNoteSourceBasisV1, KnowledgeNoteTopicHintV1, VerifiedKnowledgeNoteStatusV1,
    VerifiedKnowledgeNoteV1, create_verified_knowledge_note_from_reviewed_candidate_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::model::{
    KNOWLEDGE_OUTBOX_LIMIT_V1, KNOWLEDGE_RECOVERY_LIMIT_V1, valid_cleanup, valid_identity,
    valid_note, valid_outbox, valid_reservation,
};
use crate::row_codec::{decode_command, decode_outbox};
use crate::{
    CompleteReviewedCandidateKnowledgeNoteV1, KnowledgeOutboxRecordV1, KnowledgePersistenceErrorV1,
    PersistReviewedCandidateMaterializationV1, PersistedReviewedCandidateCommandV1,
    RejectReviewedCandidateKnowledgeNoteV1, ReserveReviewedCandidateCommandOutcomeV1,
    ReserveReviewedCandidateCommandV1,
};

const COMMAND_COLUMNS: &str = "logical_owner_id, command_message_id, command_envelope_sha256, \
    command_id, command_fingerprint, approved_candidate_id, candidate_digest, \
    source_evidence_id, source_evidence_revision, review_id, decision_revision, \
    decided_by_owner_device_id, candidate_blob_reference_id, candidate_blob_declared_bytes, \
    candidate_blob_sha256, candidate_blob_custody_proof, materialized_blob_reference_id, \
    materialized_blob_declared_bytes, materialized_blob_sha256, materialized_blob_custody_proof, \
    cleanup_completed_at_unix_millis, completed, rejected, note_id, \
    note_creation_fingerprint, received_at_unix_millis";

#[derive(Clone)]
pub struct KnowledgePersistenceV1 {
    pool: PgPool,
}

impl KnowledgePersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, KnowledgePersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(KnowledgePersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(
                u16::try_from(pgbouncer_port)
                    .map_err(|_| KnowledgePersistenceErrorV1::StorageUnavailable)?,
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

    pub async fn verify_storage_ready(&self) -> Result<(), KnowledgePersistenceErrorV1> {
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
    ) -> Result<ReserveReviewedCandidateCommandOutcomeV1, KnowledgePersistenceErrorV1> {
        if !valid_reservation(input) {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let fingerprint = input.command_fingerprint();
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.knowledge_reviewed_candidate_inbox (\
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
        .execute(&self.pool)
        .await
        .map_err(storage)?
        .rows_affected()
            == 1;
        let persisted = self
            .load_command(&input.logical_owner_id, input.command_message_id)
            .await?
            .ok_or(KnowledgePersistenceErrorV1::CommandConflict)?;
        if persisted.command_message_id != input.command_message_id
            || persisted.command_envelope_sha256 != input.command_envelope_sha256
            || persisted.command_id != input.command_id
            || persisted.approved_candidate_id != input.approved_candidate_id
            || persisted.command_fingerprint != fingerprint
        {
            return Err(KnowledgePersistenceErrorV1::CommandConflict);
        }
        Ok(if inserted {
            ReserveReviewedCandidateCommandOutcomeV1::Reserved(persisted)
        } else {
            ReserveReviewedCandidateCommandOutcomeV1::Existing(persisted)
        })
    }

    pub async fn persist_materialization(
        &self,
        input: &PersistReviewedCandidateMaterializationV1,
    ) -> Result<(), KnowledgePersistenceErrorV1> {
        if !valid_identity(&input.logical_owner_id) || !valid_cleanup(&input.materialization) {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let result = sqlx::query(
            "UPDATE makosh_data.knowledge_reviewed_candidate_inbox \
             SET materialized_blob_reference_id = $3, materialized_blob_declared_bytes = $4, \
             materialized_blob_sha256 = $5, materialized_blob_custody_proof = $6 \
             WHERE logical_owner_id = $1 AND command_message_id = $2 \
             AND (materialized_blob_reference_id IS NULL OR (materialized_blob_reference_id = $3 \
             AND materialized_blob_declared_bytes = $4 AND materialized_blob_sha256 = $5 \
             AND materialized_blob_custody_proof = $6))",
        )
        .bind(&input.logical_owner_id)
        .bind(input.command_message_id.as_slice())
        .bind(input.materialization.reference_id.as_slice())
        .bind(i64_value(input.materialization.declared_bytes)?)
        .bind(input.materialization.sha256.as_slice())
        .bind(&input.materialization.custody_proof)
        .execute(&self.pool)
        .await
        .map_err(storage)?;
        if result.rows_affected() != 1 {
            return Err(KnowledgePersistenceErrorV1::CommandConflict);
        }
        Ok(())
    }

    pub async fn complete_note(
        &self,
        input: CompleteReviewedCandidateKnowledgeNoteV1,
    ) -> Result<VerifiedKnowledgeNoteV1, KnowledgePersistenceErrorV1> {
        if !valid_identity(&input.logical_owner_id)
            || !valid_outbox(&input.created_result)
            || input.occurred_at_unix_millis <= 0
        {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let creation_fingerprint = input.creation_fingerprint()?;
        let note = create_verified_knowledge_note_from_reviewed_candidate_v1(input.draft)
            .map_err(|_| KnowledgePersistenceErrorV1::InvalidInput)?;
        if !valid_note(&note) || note.logical_owner_id != input.logical_owner_id {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let command = lock_command(
            &mut transaction,
            &input.logical_owner_id,
            input.command_message_id,
        )
        .await?;
        if command.approved_candidate_id != note.provenance.approved_candidate_id
            || command.candidate_digest != note.provenance.candidate_digest
            || command.source_evidence_id != note.provenance.source_evidence_id
            || command.source_evidence_revision != note.provenance.source_evidence_revision
            || command.review_id != note.provenance.review_id
            || command.decision_revision != note.provenance.decision_revision
            || command.decided_by_owner_device_id != note.provenance.decided_by_owner_device_id
        {
            return Err(KnowledgePersistenceErrorV1::KnowledgeNoteConflict);
        }
        if command.completed {
            if !command.rejected
                && command.note_id == Some(note.note_id)
                && command.note_creation_fingerprint == Some(creation_fingerprint)
                && outbox_matches(
                    &mut transaction,
                    &input.logical_owner_id,
                    &input.created_result,
                )
                .await?
            {
                transaction.commit().await.map_err(storage)?;
                return Ok(note);
            }
            return Err(KnowledgePersistenceErrorV1::KnowledgeNoteConflict);
        }
        insert_note(&mut transaction, &note).await?;
        let updated = sqlx::query(
            "UPDATE makosh_data.knowledge_reviewed_candidate_inbox SET completed = TRUE, \
             note_id = $3, note_creation_fingerprint = $4, completed_at_unix_millis = $5 \
             WHERE logical_owner_id = $1 AND command_message_id = $2 AND NOT completed",
        )
        .bind(&input.logical_owner_id)
        .bind(input.command_message_id.as_slice())
        .bind(note.note_id.as_slice())
        .bind(creation_fingerprint.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if updated.rows_affected() != 1 {
            return Err(KnowledgePersistenceErrorV1::KnowledgeNoteConflict);
        }
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.created_result,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(note)
    }

    pub async fn reject_note(
        &self,
        input: &RejectReviewedCandidateKnowledgeNoteV1,
    ) -> Result<(), KnowledgePersistenceErrorV1> {
        if !valid_identity(&input.logical_owner_id)
            || !valid_outbox(&input.rejected_result)
            || input.occurred_at_unix_millis <= 0
        {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let command = lock_command(
            &mut transaction,
            &input.logical_owner_id,
            input.command_message_id,
        )
        .await?;
        if command.completed {
            return if command.rejected
                && outbox_matches(
                    &mut transaction,
                    &input.logical_owner_id,
                    &input.rejected_result,
                )
                .await?
            {
                transaction.commit().await.map_err(storage)
            } else {
                Err(KnowledgePersistenceErrorV1::KnowledgeNoteConflict)
            };
        }
        sqlx::query(
            "UPDATE makosh_data.knowledge_reviewed_candidate_inbox SET completed = TRUE, \
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
    ) -> Result<(), KnowledgePersistenceErrorV1> {
        if !valid_identity(logical_owner_id) || completed_at_unix_millis <= 0 {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let result = sqlx::query(
            "UPDATE makosh_data.knowledge_reviewed_candidate_inbox \
             SET cleanup_completed_at_unix_millis = $3 \
             WHERE logical_owner_id = $1 AND command_message_id = $2 \
             AND materialized_blob_reference_id IS NOT NULL \
             AND (cleanup_completed_at_unix_millis IS NULL OR cleanup_completed_at_unix_millis = $3)",
        )
        .bind(logical_owner_id)
        .bind(command_message_id.as_slice())
        .bind(completed_at_unix_millis)
        .execute(&self.pool)
        .await
        .map_err(storage)?;
        if result.rows_affected() != 1 {
            return Err(KnowledgePersistenceErrorV1::CommandConflict);
        }
        Ok(())
    }

    pub async fn load_recoverable_commands(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<PersistedReviewedCandidateCommandV1>, KnowledgePersistenceErrorV1> {
        if !valid_identity(logical_owner_id) {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        let query = format!(
            "SELECT {COMMAND_COLUMNS} FROM makosh_data.knowledge_reviewed_candidate_inbox \
             WHERE logical_owner_id = $1 AND (NOT completed OR \
             (materialized_blob_reference_id IS NOT NULL AND cleanup_completed_at_unix_millis IS NULL)) \
             ORDER BY received_at_unix_millis, command_message_id LIMIT $2"
        );
        sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(logical_owner_id)
            .bind(i64::from(KNOWLEDGE_RECOVERY_LIMIT_V1))
            .fetch_all(&self.pool)
            .await
            .map_err(storage)?
            .iter()
            .map(decode_command)
            .collect()
    }

    pub async fn load_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<KnowledgeOutboxRecordV1>, KnowledgePersistenceErrorV1> {
        if !valid_identity(logical_owner_id) {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT message_id, envelope_sha256, envelope_bytes FROM makosh_data.knowledge_outbox \
             WHERE logical_owner_id = $1 AND published_at_unix_millis IS NULL \
             ORDER BY created_at_unix_millis, message_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(KNOWLEDGE_OUTBOX_LIMIT_V1))
        .fetch_all(&self.pool)
        .await
        .map_err(storage)?
        .iter()
        .map(decode_outbox)
        .collect()
    }

    pub async fn mark_outbox_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        published_at_unix_millis: i64,
    ) -> Result<(), KnowledgePersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || message_id.iter().all(|byte| *byte == 0)
            || published_at_unix_millis <= 0
        {
            return Err(KnowledgePersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "UPDATE makosh_data.knowledge_outbox SET published_at_unix_millis = $3 \
             WHERE logical_owner_id = $1 AND message_id = $2 \
             AND (published_at_unix_millis IS NULL OR published_at_unix_millis = $3)",
        )
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .bind(published_at_unix_millis)
        .execute(&self.pool)
        .await
        .map_err(storage)?;
        Ok(())
    }

    async fn load_command(
        &self,
        logical_owner_id: &str,
        command_message_id: [u8; 16],
    ) -> Result<Option<PersistedReviewedCandidateCommandV1>, KnowledgePersistenceErrorV1> {
        let query = format!(
            "SELECT {COMMAND_COLUMNS} FROM makosh_data.knowledge_reviewed_candidate_inbox \
             WHERE logical_owner_id = $1 AND command_message_id = $2"
        );
        sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(logical_owner_id)
            .bind(command_message_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage)?
            .as_ref()
            .map(decode_command)
            .transpose()
    }
}

async fn lock_command(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    command_message_id: [u8; 16],
) -> Result<PersistedReviewedCandidateCommandV1, KnowledgePersistenceErrorV1> {
    let query = format!(
        "SELECT {COMMAND_COLUMNS} FROM makosh_data.knowledge_reviewed_candidate_inbox \
         WHERE logical_owner_id = $1 AND command_message_id = $2 FOR UPDATE"
    );
    let row = sqlx::query(sqlx::AssertSqlSafe(query))
        .bind(logical_owner_id)
        .bind(command_message_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?
        .ok_or(KnowledgePersistenceErrorV1::NotFound)?;
    decode_command(&row)
}

async fn insert_note(
    transaction: &mut Transaction<'_, Postgres>,
    note: &VerifiedKnowledgeNoteV1,
) -> Result<(), KnowledgePersistenceErrorV1> {
    let status = match note.status {
        VerifiedKnowledgeNoteStatusV1::Verified => 1_i16,
    };
    let topic_hints = note
        .topic_hints
        .iter()
        .copied()
        .map(topic_hint_code)
        .collect::<Vec<_>>();
    let source_basis = source_basis_code(note.source_basis);
    let result = sqlx::query(
        "INSERT INTO makosh_data.knowledge_state (logical_owner_id, note_id, title, excerpt, \
         topic_hints, source_basis, confidence_basis_points, status, note_revision, \
         approved_candidate_id, candidate_digest, source_evidence_id, source_evidence_revision, \
         review_id, decision_revision, decided_by_owner_device_id, created_at_unix_seconds, \
         created_at_nanos, updated_at_unix_seconds, updated_at_nanos) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20) \
         ON CONFLICT (logical_owner_id, approved_candidate_id) DO NOTHING",
    )
    .bind(&note.logical_owner_id)
    .bind(note.note_id.as_slice())
    .bind(&note.title)
    .bind(&note.excerpt)
    .bind(topic_hints)
    .bind(source_basis)
    .bind(
        i32::try_from(note.confidence_basis_points)
            .map_err(|_| KnowledgePersistenceErrorV1::InvalidInput)?,
    )
    .bind(status)
    .bind(i64_value(note.note_revision)?)
    .bind(note.provenance.approved_candidate_id.as_slice())
    .bind(note.provenance.candidate_digest.as_slice())
    .bind(note.provenance.source_evidence_id.as_slice())
    .bind(i64_value(note.provenance.source_evidence_revision)?)
    .bind(note.provenance.review_id.as_slice())
    .bind(i64_value(note.provenance.decision_revision)?)
    .bind(note.provenance.decided_by_owner_device_id.as_slice())
    .bind(note.created_at.unix_seconds)
    .bind(note.created_at.nanos)
    .bind(note.updated_at.unix_seconds)
    .bind(note.updated_at.nanos)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(KnowledgePersistenceErrorV1::KnowledgeNoteConflict);
    }
    Ok(())
}

async fn insert_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    record: &KnowledgeOutboxRecordV1,
    created_at_unix_millis: i64,
) -> Result<(), KnowledgePersistenceErrorV1> {
    let result = sqlx::query(
        "INSERT INTO makosh_data.knowledge_outbox (logical_owner_id, message_id, envelope_sha256, \
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
        return Err(KnowledgePersistenceErrorV1::InboxConflict);
    }
    Ok(())
}

async fn outbox_matches(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    record: &KnowledgeOutboxRecordV1,
) -> Result<bool, KnowledgePersistenceErrorV1> {
    let existing = sqlx::query(
        "SELECT envelope_sha256, envelope_bytes FROM makosh_data.knowledge_outbox \
         WHERE logical_owner_id = $1 AND message_id = $2",
    )
    .bind(logical_owner_id)
    .bind(record.message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(existing) = existing else {
        return Ok(false);
    };
    use sqlx::Row;
    let envelope_sha256: Vec<u8> = existing
        .try_get("envelope_sha256")
        .map_err(|_| KnowledgePersistenceErrorV1::InvalidRow)?;
    let envelope_bytes: Vec<u8> = existing
        .try_get("envelope_bytes")
        .map_err(|_| KnowledgePersistenceErrorV1::InvalidRow)?;
    Ok(envelope_sha256 == record.envelope_sha256 && envelope_bytes == record.envelope_bytes)
}

fn source_basis_code(value: KnowledgeNoteSourceBasisV1) -> i16 {
    match value {
        KnowledgeNoteSourceBasisV1::Subject => 1,
        KnowledgeNoteSourceBasisV1::Body => 2,
        KnowledgeNoteSourceBasisV1::Combined => 3,
    }
}

fn topic_hint_code(value: KnowledgeNoteTopicHintV1) -> i16 {
    match value {
        KnowledgeNoteTopicHintV1::Financial => 1,
        KnowledgeNoteTopicHintV1::Legal => 2,
        KnowledgeNoteTopicHintV1::DecisionStatement => 3,
        KnowledgeNoteTopicHintV1::DeadlineStatement => 4,
    }
}

fn i64_value(value: u64) -> Result<i64, KnowledgePersistenceErrorV1> {
    i64::try_from(value).map_err(|_| KnowledgePersistenceErrorV1::InvalidInput)
}

fn storage(_: sqlx::Error) -> KnowledgePersistenceErrorV1 {
    KnowledgePersistenceErrorV1::StorageUnavailable
}
