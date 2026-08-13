use makosh_decisions_core::{
    DecisionAlternativeStateV1, DecisionAlternativeV1, DecisionEvidenceLinkV1,
    DecisionLifecycleErrorV1, DecisionRecordV1, DecisionStateV1, DecisionTimestampV1,
    add_alternative_v1, add_evidence_v1, cancel_v1, create_decision_v1, decide_v1,
    remove_alternative_v1, remove_evidence_v1, supersede_v1, update_alternative_v1,
    update_decision_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    DecisionLifecycleCommitV1, DecisionLifecycleMutationV1, DecisionLifecycleOperationOutcomeV1,
    DecisionLifecycleOperationV1, DecisionOutboxRecordV1, DecisionsPersistenceErrorV1,
    model::{DECISIONS_MAX_CLIENT_MESSAGE_BYTES_V1, valid_commit, valid_operation, valid_owner},
};

#[derive(Clone)]
pub struct DecisionsPersistenceV1 {
    pool: PgPool,
}

pub struct DecisionOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: DecisionOutboxRecordV1,
    created_at_unix_millis: i64,
}

impl DecisionOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &DecisionOutboxRecordV1 {
        &self.record
    }

    pub async fn mark_published(
        mut self,
        expected_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), DecisionsPersistenceErrorV1> {
        if expected_sha256 != self.record.envelope_sha256
            || Sha256::digest(&self.record.envelope_bytes).as_slice() != expected_sha256
            || published_at_unix_millis < self.created_at_unix_millis
        {
            return Err(DecisionsPersistenceErrorV1::OutboxConflict);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.decisions_outbox SET published_at_unix_millis=$3 \
             WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 \
             AND published_at_unix_millis IS NULL",
        )
        .bind(&self.logical_owner_id)
        .bind(self.record.message_id.as_slice())
        .bind(published_at_unix_millis)
        .bind(expected_sha256.as_slice())
        .execute(&mut *self.transaction)
        .await
        .map_err(|_| storage())?
        .rows_affected();
        if affected != 1 {
            return Err(DecisionsPersistenceErrorV1::OutboxConflict);
        }
        self.transaction.commit().await.map_err(|_| storage())
    }
}

impl DecisionsPersistenceV1 {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, DecisionsPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(DecisionsPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(u16::try_from(pgbouncer_port).map_err(|_| storage())?)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(|_| storage())?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), DecisionsPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| storage())
    }

    async fn begin_owner(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, DecisionsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(DecisionsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        Ok(transaction)
    }

    pub async fn load_operation_replay(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
        request_sha256: [u8; 32],
        request_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, DecisionsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || operation_id.iter().all(|byte| *byte == 0)
            || request_sha256.iter().all(|byte| *byte == 0)
            || request_bytes.is_empty()
            || request_bytes.len() > DECISIONS_MAX_CLIENT_MESSAGE_BYTES_V1
            || Sha256::digest(request_bytes).as_slice() != request_sha256
        {
            return Err(DecisionsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let replay = load_replay(
            &mut transaction,
            logical_owner_id,
            operation_id,
            request_sha256,
            request_bytes,
            None,
        )
        .await?;
        transaction.commit().await.map_err(|_| storage())?;
        Ok(replay)
    }

    pub async fn apply_lifecycle_operation<F>(
        &self,
        input: DecisionLifecycleOperationV1,
        build_commit: F,
    ) -> Result<DecisionLifecycleOperationOutcomeV1, DecisionsPersistenceErrorV1>
    where
        F: FnOnce(
            &DecisionRecordV1,
        ) -> Result<DecisionLifecycleCommitV1, DecisionsPersistenceErrorV1>,
    {
        if !valid_operation(&input)
            || Sha256::digest(&input.request_bytes).as_slice() != input.request_sha256
        {
            return Err(DecisionsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || encode($2, 'hex'), 0))")
            .bind(&input.logical_owner_id)
            .bind(input.operation_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        if let Some(response_bytes) = load_replay(
            &mut transaction,
            &input.logical_owner_id,
            input.operation_id,
            input.request_sha256,
            &input.request_bytes,
            Some(input.mutation.operation_kind()),
        )
        .await?
        {
            transaction.commit().await.map_err(|_| storage())?;
            return Ok(DecisionLifecycleOperationOutcomeV1::Replayed { response_bytes });
        }

        let creating = matches!(input.mutation, DecisionLifecycleMutationV1::Create { .. });
        let mut decision = match &input.mutation {
            DecisionLifecycleMutationV1::Create {
                owner,
                operation_id,
                title,
                question,
                created_at,
            } => {
                if owner != &input.logical_owner_id || operation_id != &input.operation_id {
                    return Err(DecisionsPersistenceErrorV1::InvalidInput);
                }
                create_decision_v1(
                    owner.clone(),
                    *operation_id,
                    title.clone(),
                    question.clone(),
                    *created_at,
                )
                .map_err(core_error)?
            }
            mutation => load_decision(
                &mut transaction,
                &input.logical_owner_id,
                mutation
                    .decision_id()
                    .ok_or(DecisionsPersistenceErrorV1::InvalidInput)?,
                true,
            )
            .await?
            .ok_or(DecisionsPersistenceErrorV1::NotFound)?,
        };
        apply_mutation(&mut decision, &input.mutation)?;
        persist_decision(&mut transaction, &decision, creating).await?;
        persist_alternatives(&mut transaction, &decision).await?;
        persist_evidence(&mut transaction, &decision).await?;

        let commit = build_commit(&decision)?;
        if !valid_commit(&commit)
            || Sha256::digest(&commit.response_bytes).as_slice() != commit.response_sha256
            || Sha256::digest(&commit.lifecycle_event.envelope_bytes).as_slice()
                != commit.lifecycle_event.envelope_sha256
        {
            return Err(DecisionsPersistenceErrorV1::InvalidInput);
        }
        if sqlx::query(
            "INSERT INTO makosh_data.decisions_outbox \
             (logical_owner_id,message_id,envelope_sha256,envelope_bytes,created_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(commit.lifecycle_event.message_id.as_slice())
        .bind(commit.lifecycle_event.envelope_sha256.as_slice())
        .bind(&commit.lifecycle_event.envelope_bytes)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?
        .rows_affected()
            != 1
        {
            return Err(DecisionsPersistenceErrorV1::OutboxConflict);
        }
        if sqlx::query(
            "INSERT INTO makosh_data.decisions_client_operations \
             (logical_owner_id,operation_id,operation_kind,request_sha256,request_bytes, \
              decision_id,decision_revision,response_sha256,response_bytes,received_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(input.mutation.operation_kind())
        .bind(input.request_sha256.as_slice())
        .bind(&input.request_bytes)
        .bind(decision.decision_id.as_slice())
        .bind(i64_value(decision.decision_revision)?)
        .bind(commit.response_sha256.as_slice())
        .bind(&commit.response_bytes)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?
        .rows_affected()
            != 1
        {
            return Err(DecisionsPersistenceErrorV1::OperationConflict);
        }
        transaction.commit().await.map_err(|_| storage())?;
        Ok(DecisionLifecycleOperationOutcomeV1::Applied {
            decision: Box::new(decision),
            response_bytes: commit.response_bytes,
        })
    }

    pub async fn get_decision(
        &self,
        logical_owner_id: &str,
        decision_id: [u8; 16],
    ) -> Result<Option<DecisionRecordV1>, DecisionsPersistenceErrorV1> {
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let value = load_decision(&mut transaction, logical_owner_id, decision_id, false).await?;
        transaction.commit().await.map_err(|_| storage())?;
        Ok(value)
    }

    pub async fn list_decisions(
        &self,
        logical_owner_id: &str,
        after_decision_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<DecisionRecordV1>, DecisionsPersistenceErrorV1> {
        if limit == 0 || limit > 200 {
            return Err(DecisionsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = if let Some(after) = after_decision_id {
            sqlx::query(
                "SELECT decision_id FROM makosh_data.decisions_records \
                 WHERE logical_owner_id=$1 AND decision_id>$2 ORDER BY decision_id LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(after.as_slice())
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(|_| storage())?
        } else {
            sqlx::query(
                "SELECT decision_id FROM makosh_data.decisions_records \
                 WHERE logical_owner_id=$1 ORDER BY decision_id LIMIT $2",
            )
            .bind(logical_owner_id)
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(|_| storage())?
        };
        let mut decisions = Vec::with_capacity(rows.len());
        for row in rows {
            let id = bytes16(row.try_get("decision_id").map_err(|_| storage())?)?;
            decisions.push(
                load_decision(&mut transaction, logical_owner_id, id, false)
                    .await?
                    .ok_or(DecisionsPersistenceErrorV1::StorageUnavailable)?,
            );
        }
        transaction.commit().await.map_err(|_| storage())?;
        Ok(decisions)
    }

    pub async fn claim_next_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Option<DecisionOutboxPublishClaimV1>, DecisionsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(DecisionsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        let row = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,created_at_unix_millis \
             FROM makosh_data.decisions_outbox WHERE logical_owner_id=$1 \
             AND published_at_unix_millis IS NULL ORDER BY outbox_sequence \
             LIMIT 1 FOR UPDATE SKIP LOCKED",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let Some(row) = row else {
            transaction.commit().await.map_err(|_| storage())?;
            return Ok(None);
        };
        let record = DecisionOutboxRecordV1 {
            message_id: bytes16(row.try_get("message_id").map_err(|_| storage())?)?,
            envelope_sha256: bytes32(row.try_get("envelope_sha256").map_err(|_| storage())?)?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
        };
        if Sha256::digest(&record.envelope_bytes).as_slice() != record.envelope_sha256 {
            return Err(DecisionsPersistenceErrorV1::StorageUnavailable);
        }
        Ok(Some(DecisionOutboxPublishClaimV1 {
            transaction,
            logical_owner_id: logical_owner_id.to_owned(),
            record,
            created_at_unix_millis: row
                .try_get("created_at_unix_millis")
                .map_err(|_| storage())?,
        }))
    }
}

fn apply_mutation(
    decision: &mut DecisionRecordV1,
    mutation: &DecisionLifecycleMutationV1,
) -> Result<(), DecisionsPersistenceErrorV1> {
    match mutation {
        DecisionLifecycleMutationV1::Create { .. } => Ok(()),
        DecisionLifecycleMutationV1::Update {
            expected_revision,
            title,
            question,
            changed_at,
            ..
        } => update_decision_v1(
            decision,
            *expected_revision,
            title.clone(),
            question.clone(),
            *changed_at,
        )
        .map_err(core_error),
        DecisionLifecycleMutationV1::AddAlternative {
            expected_revision,
            operation_id,
            title,
            description,
            changed_at,
            ..
        } => add_alternative_v1(
            decision,
            *expected_revision,
            *operation_id,
            title.clone(),
            description.clone(),
            *changed_at,
        )
        .map(|_| ())
        .map_err(core_error),
        DecisionLifecycleMutationV1::UpdateAlternative {
            expected_revision,
            alternative_id,
            expected_alternative_revision,
            title,
            description,
            changed_at,
            ..
        } => update_alternative_v1(
            decision,
            *expected_revision,
            *alternative_id,
            *expected_alternative_revision,
            title.clone(),
            description.clone(),
            *changed_at,
        )
        .map_err(core_error),
        DecisionLifecycleMutationV1::RemoveAlternative {
            expected_revision,
            alternative_id,
            expected_alternative_revision,
            changed_at,
            ..
        } => remove_alternative_v1(
            decision,
            *expected_revision,
            *alternative_id,
            *expected_alternative_revision,
            *changed_at,
        )
        .map_err(core_error),
        DecisionLifecycleMutationV1::AddEvidence {
            expected_revision,
            evidence,
            changed_at,
            ..
        } => add_evidence_v1(decision, *expected_revision, evidence.clone(), *changed_at)
            .map_err(core_error),
        DecisionLifecycleMutationV1::RemoveEvidence {
            expected_revision,
            evidence_link_id,
            changed_at,
            ..
        } => remove_evidence_v1(decision, *expected_revision, *evidence_link_id, *changed_at)
            .map_err(core_error),
        DecisionLifecycleMutationV1::Decide {
            expected_revision,
            selected_alternative_id,
            rationale,
            changed_at,
            ..
        } => decide_v1(
            decision,
            *expected_revision,
            *selected_alternative_id,
            rationale.clone(),
            *changed_at,
        )
        .map_err(core_error),
        DecisionLifecycleMutationV1::Supersede {
            expected_revision,
            replacement_decision_id,
            changed_at,
            ..
        } => supersede_v1(
            decision,
            *expected_revision,
            *replacement_decision_id,
            *changed_at,
        )
        .map_err(core_error),
        DecisionLifecycleMutationV1::Cancel {
            expected_revision,
            changed_at,
            ..
        } => cancel_v1(decision, *expected_revision, *changed_at).map_err(core_error),
    }
}

async fn persist_decision(
    transaction: &mut Transaction<'_, Postgres>,
    value: &DecisionRecordV1,
    creating: bool,
) -> Result<(), DecisionsPersistenceErrorV1> {
    let affected = if creating {
        sqlx::query(
            "INSERT INTO makosh_data.decisions_records \
             (logical_owner_id,decision_id,title,question,rationale,decision_state, \
              selected_alternative_id,superseded_by_decision_id,decision_revision, \
              created_at_unix_seconds,created_at_nanos,updated_at_unix_seconds,updated_at_nanos) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
        )
        .bind(&value.logical_owner_id)
        .bind(value.decision_id.as_slice())
        .bind(&value.title)
        .bind(&value.question)
        .bind(&value.rationale)
        .bind(state_code(value.state))
        .bind(
            value
                .selected_alternative_id
                .as_ref()
                .map(|id| id.as_slice()),
        )
        .bind(
            value
                .superseded_by_decision_id
                .as_ref()
                .map(|id| id.as_slice()),
        )
        .bind(i64_value(value.decision_revision)?)
        .bind(value.created_at.unix_seconds)
        .bind(value.created_at.nanos)
        .bind(value.updated_at.unix_seconds)
        .bind(value.updated_at.nanos)
        .execute(&mut **transaction)
        .await
        .map_err(|_| storage())?
        .rows_affected()
    } else {
        sqlx::query(
            "UPDATE makosh_data.decisions_records SET title=$3,question=$4,rationale=$5, \
             decision_state=$6,selected_alternative_id=$7,superseded_by_decision_id=$8, \
             decision_revision=$9,updated_at_unix_seconds=$10,updated_at_nanos=$11 \
             WHERE logical_owner_id=$1 AND decision_id=$2",
        )
        .bind(&value.logical_owner_id)
        .bind(value.decision_id.as_slice())
        .bind(&value.title)
        .bind(&value.question)
        .bind(&value.rationale)
        .bind(state_code(value.state))
        .bind(
            value
                .selected_alternative_id
                .as_ref()
                .map(|id| id.as_slice()),
        )
        .bind(
            value
                .superseded_by_decision_id
                .as_ref()
                .map(|id| id.as_slice()),
        )
        .bind(i64_value(value.decision_revision)?)
        .bind(value.updated_at.unix_seconds)
        .bind(value.updated_at.nanos)
        .execute(&mut **transaction)
        .await
        .map_err(|_| storage())?
        .rows_affected()
    };
    (affected == 1)
        .then_some(())
        .ok_or(DecisionsPersistenceErrorV1::RevisionConflict)
}

async fn persist_alternatives(
    transaction: &mut Transaction<'_, Postgres>,
    decision: &DecisionRecordV1,
) -> Result<(), DecisionsPersistenceErrorV1> {
    sqlx::query(
        "DELETE FROM makosh_data.decisions_alternatives WHERE logical_owner_id=$1 AND decision_id=$2",
    )
    .bind(&decision.logical_owner_id)
    .bind(decision.decision_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    for value in &decision.alternatives {
        sqlx::query(
            "INSERT INTO makosh_data.decisions_alternatives \
             (logical_owner_id,decision_id,alternative_id,title,description,alternative_state, \
              alternative_revision,updated_at_decision_revision,created_at_unix_seconds, \
              created_at_nanos,updated_at_unix_seconds,updated_at_nanos) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
        )
        .bind(&decision.logical_owner_id)
        .bind(decision.decision_id.as_slice())
        .bind(value.alternative_id.as_slice())
        .bind(&value.title)
        .bind(&value.description)
        .bind(alternative_state_code(value.state))
        .bind(i64_value(value.alternative_revision)?)
        .bind(i64_value(value.updated_at_decision_revision)?)
        .bind(value.created_at.unix_seconds)
        .bind(value.created_at.nanos)
        .bind(value.updated_at.unix_seconds)
        .bind(value.updated_at.nanos)
        .execute(&mut **transaction)
        .await
        .map_err(|_| storage())?;
    }
    Ok(())
}

async fn persist_evidence(
    transaction: &mut Transaction<'_, Postgres>,
    decision: &DecisionRecordV1,
) -> Result<(), DecisionsPersistenceErrorV1> {
    sqlx::query(
        "DELETE FROM makosh_data.decisions_evidence_links WHERE logical_owner_id=$1 AND decision_id=$2",
    )
    .bind(&decision.logical_owner_id)
    .bind(decision.decision_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    for value in &decision.evidence {
        sqlx::query(
            "INSERT INTO makosh_data.decisions_evidence_links \
             (logical_owner_id,decision_id,evidence_link_id,evidence_owner_id, \
              evidence_record_id,evidence_revision,evidence_digest) \
             VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(&decision.logical_owner_id)
        .bind(decision.decision_id.as_slice())
        .bind(value.evidence_link_id.as_slice())
        .bind(&value.evidence_owner_id)
        .bind(value.evidence_record_id.as_slice())
        .bind(i64_value(value.evidence_revision)?)
        .bind(value.evidence_digest.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(|_| storage())?;
    }
    Ok(())
}

async fn load_decision(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    decision_id: [u8; 16],
    lock: bool,
) -> Result<Option<DecisionRecordV1>, DecisionsPersistenceErrorV1> {
    let row = if lock {
        sqlx::query(
            "SELECT * FROM makosh_data.decisions_records \
             WHERE logical_owner_id=$1 AND decision_id=$2 FOR UPDATE",
        )
        .bind(owner)
        .bind(decision_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| storage())?
    } else {
        sqlx::query(
            "SELECT * FROM makosh_data.decisions_records WHERE logical_owner_id=$1 AND decision_id=$2",
        )
        .bind(owner)
        .bind(decision_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| storage())?
    };
    let Some(row) = row else { return Ok(None) };
    let alternatives = sqlx::query(
        "SELECT * FROM makosh_data.decisions_alternatives \
         WHERE logical_owner_id=$1 AND decision_id=$2 ORDER BY alternative_id",
    )
    .bind(owner)
    .bind(decision_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| storage())?
    .into_iter()
    .map(decode_alternative)
    .collect::<Result<Vec<_>, _>>()?;
    let evidence = sqlx::query(
        "SELECT * FROM makosh_data.decisions_evidence_links \
         WHERE logical_owner_id=$1 AND decision_id=$2 ORDER BY evidence_link_id",
    )
    .bind(owner)
    .bind(decision_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| storage())?
    .into_iter()
    .map(decode_evidence)
    .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(DecisionRecordV1 {
        decision_id,
        logical_owner_id: row.try_get("logical_owner_id").map_err(|_| storage())?,
        title: row.try_get("title").map_err(|_| storage())?,
        question: row.try_get("question").map_err(|_| storage())?,
        rationale: row.try_get("rationale").map_err(|_| storage())?,
        state: decode_state(row.try_get("decision_state").map_err(|_| storage())?)?,
        selected_alternative_id: optional_bytes16(
            row.try_get("selected_alternative_id")
                .map_err(|_| storage())?,
        )?,
        superseded_by_decision_id: optional_bytes16(
            row.try_get("superseded_by_decision_id")
                .map_err(|_| storage())?,
        )?,
        decision_revision: u64_value(row.try_get("decision_revision").map_err(|_| storage())?)?,
        alternatives,
        evidence,
        created_at: timestamp(&row, "created_at_unix_seconds", "created_at_nanos")?,
        updated_at: timestamp(&row, "updated_at_unix_seconds", "updated_at_nanos")?,
    }))
}

fn decode_alternative(
    row: sqlx::postgres::PgRow,
) -> Result<DecisionAlternativeV1, DecisionsPersistenceErrorV1> {
    Ok(DecisionAlternativeV1 {
        alternative_id: bytes16(row.try_get("alternative_id").map_err(|_| storage())?)?,
        decision_id: bytes16(row.try_get("decision_id").map_err(|_| storage())?)?,
        title: row.try_get("title").map_err(|_| storage())?,
        description: row.try_get("description").map_err(|_| storage())?,
        state: decode_alternative_state(row.try_get("alternative_state").map_err(|_| storage())?)?,
        alternative_revision: u64_value(
            row.try_get("alternative_revision").map_err(|_| storage())?,
        )?,
        updated_at_decision_revision: u64_value(
            row.try_get("updated_at_decision_revision")
                .map_err(|_| storage())?,
        )?,
        created_at: timestamp(&row, "created_at_unix_seconds", "created_at_nanos")?,
        updated_at: timestamp(&row, "updated_at_unix_seconds", "updated_at_nanos")?,
    })
}

fn decode_evidence(
    row: sqlx::postgres::PgRow,
) -> Result<DecisionEvidenceLinkV1, DecisionsPersistenceErrorV1> {
    Ok(DecisionEvidenceLinkV1 {
        evidence_link_id: bytes16(row.try_get("evidence_link_id").map_err(|_| storage())?)?,
        evidence_owner_id: row.try_get("evidence_owner_id").map_err(|_| storage())?,
        evidence_record_id: bytes16(row.try_get("evidence_record_id").map_err(|_| storage())?)?,
        evidence_revision: u64_value(row.try_get("evidence_revision").map_err(|_| storage())?)?,
        evidence_digest: bytes32(row.try_get("evidence_digest").map_err(|_| storage())?)?,
    })
}

async fn load_replay(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    operation_id: [u8; 16],
    request_sha256: [u8; 32],
    request_bytes: &[u8],
    operation_kind: Option<i16>,
) -> Result<Option<Vec<u8>>, DecisionsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT operation_kind,request_sha256,request_bytes,response_sha256,response_bytes \
         FROM makosh_data.decisions_client_operations \
         WHERE logical_owner_id=$1 AND operation_id=$2 FOR UPDATE",
    )
    .bind(owner)
    .bind(operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    let Some(row) = row else { return Ok(None) };
    let stored_kind: i16 = row.try_get("operation_kind").map_err(|_| storage())?;
    let stored_sha: Vec<u8> = row.try_get("request_sha256").map_err(|_| storage())?;
    let stored_bytes: Vec<u8> = row.try_get("request_bytes").map_err(|_| storage())?;
    let response_sha: Vec<u8> = row.try_get("response_sha256").map_err(|_| storage())?;
    let response: Vec<u8> = row.try_get("response_bytes").map_err(|_| storage())?;
    if operation_kind.is_some_and(|value| value != stored_kind)
        || stored_sha.as_slice() != request_sha256
        || stored_bytes != request_bytes
        || Sha256::digest(&response).as_slice() != response_sha
    {
        return Err(DecisionsPersistenceErrorV1::OperationConflict);
    }
    Ok(Some(response))
}

fn state_code(value: DecisionStateV1) -> i16 {
    match value {
        DecisionStateV1::Draft => 1,
        DecisionStateV1::Decided => 2,
        DecisionStateV1::Superseded => 3,
        DecisionStateV1::Cancelled => 4,
    }
}
fn decode_state(value: i16) -> Result<DecisionStateV1, DecisionsPersistenceErrorV1> {
    match value {
        1 => Ok(DecisionStateV1::Draft),
        2 => Ok(DecisionStateV1::Decided),
        3 => Ok(DecisionStateV1::Superseded),
        4 => Ok(DecisionStateV1::Cancelled),
        _ => Err(storage()),
    }
}
fn alternative_state_code(value: DecisionAlternativeStateV1) -> i16 {
    match value {
        DecisionAlternativeStateV1::Candidate => 1,
        DecisionAlternativeStateV1::Selected => 2,
        DecisionAlternativeStateV1::Rejected => 3,
    }
}
fn decode_alternative_state(
    value: i16,
) -> Result<DecisionAlternativeStateV1, DecisionsPersistenceErrorV1> {
    match value {
        1 => Ok(DecisionAlternativeStateV1::Candidate),
        2 => Ok(DecisionAlternativeStateV1::Selected),
        3 => Ok(DecisionAlternativeStateV1::Rejected),
        _ => Err(storage()),
    }
}
fn timestamp(
    row: &sqlx::postgres::PgRow,
    seconds: &str,
    nanos: &str,
) -> Result<DecisionTimestampV1, DecisionsPersistenceErrorV1> {
    Ok(DecisionTimestampV1 {
        unix_seconds: row.try_get(seconds).map_err(|_| storage())?,
        nanos: row.try_get(nanos).map_err(|_| storage())?,
    })
}
fn optional_bytes16(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; 16]>, DecisionsPersistenceErrorV1> {
    value.map(bytes16).transpose()
}
fn bytes16(value: Vec<u8>) -> Result<[u8; 16], DecisionsPersistenceErrorV1> {
    value.try_into().map_err(|_| storage())
}
fn bytes32(value: Vec<u8>) -> Result<[u8; 32], DecisionsPersistenceErrorV1> {
    value.try_into().map_err(|_| storage())
}
fn i64_value(value: u64) -> Result<i64, DecisionsPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| DecisionsPersistenceErrorV1::InvalidInput)
}
fn u64_value(value: i64) -> Result<u64, DecisionsPersistenceErrorV1> {
    u64::try_from(value).map_err(|_| storage())
}
fn storage() -> DecisionsPersistenceErrorV1 {
    DecisionsPersistenceErrorV1::StorageUnavailable
}
fn core_error(value: DecisionLifecycleErrorV1) -> DecisionsPersistenceErrorV1 {
    match value {
        DecisionLifecycleErrorV1::InvalidRevision | DecisionLifecycleErrorV1::RevisionOverflow => {
            DecisionsPersistenceErrorV1::RevisionConflict
        }
        DecisionLifecycleErrorV1::InvalidStateTransition => {
            DecisionsPersistenceErrorV1::StateConflict
        }
        _ => DecisionsPersistenceErrorV1::InvalidInput,
    }
}
