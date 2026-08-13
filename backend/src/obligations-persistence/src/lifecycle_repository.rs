use makosh_obligations_core::{
    ObligationEvidenceLinkV1, ObligationLifecycleErrorV1, ObligationLifecycleStateV1,
    ObligationRecordV1, ObligationTimestampV1, add_obligation_evidence_v1,
    remove_obligation_evidence_v1, set_obligation_state_v1, update_obligation_content_v1,
    validate_obligation_record_v1,
};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    ObligationsLifecycleCommitV1, ObligationsLifecycleMutationV1,
    ObligationsLifecycleOperationOutcomeV1, ObligationsLifecycleOperationV1,
    ObligationsPersistenceErrorV1, ObligationsPersistenceV1,
    model::{valid_lifecycle_commit, valid_lifecycle_operation},
};

impl ObligationsPersistenceV1 {
    pub async fn load_lifecycle_operation_replay(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
        request_sha256: [u8; 32],
        request_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, ObligationsPersistenceErrorV1> {
        if operation_id.iter().all(|byte| *byte == 0)
            || request_sha256.iter().all(|byte| *byte == 0)
            || request_bytes.is_empty()
            || request_bytes.len() > crate::model::OBLIGATIONS_MAX_CLIENT_MESSAGE_BYTES_V1
            || Sha256::digest(request_bytes).as_slice() != request_sha256
        {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT request_sha256, request_bytes, response_sha256, response_bytes \
             FROM makosh_data.obligations_client_operations \
             WHERE logical_owner_id=$1 AND operation_id=$2",
        )
        .bind(logical_owner_id)
        .bind(operation_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let response = row
            .map(|row| {
                let stored_request_sha =
                    fixed::<32>(row.try_get("request_sha256").map_err(storage)?)?;
                let stored_request_bytes: Vec<u8> =
                    row.try_get("request_bytes").map_err(storage)?;
                let response_sha = fixed::<32>(row.try_get("response_sha256").map_err(storage)?)?;
                let response_bytes: Vec<u8> = row.try_get("response_bytes").map_err(storage)?;
                if stored_request_sha != request_sha256
                    || stored_request_bytes != request_bytes
                    || Sha256::digest(&response_bytes).as_slice() != response_sha
                {
                    return Err(ObligationsPersistenceErrorV1::OperationConflict);
                }
                Ok(response_bytes)
            })
            .transpose()?;
        transaction.commit().await.map_err(storage)?;
        Ok(response)
    }

    pub async fn apply_lifecycle_operation<F>(
        &self,
        input: ObligationsLifecycleOperationV1,
        build_commit: F,
    ) -> Result<ObligationsLifecycleOperationOutcomeV1, ObligationsPersistenceErrorV1>
    where
        F: FnOnce(
            &ObligationRecordV1,
        ) -> Result<ObligationsLifecycleCommitV1, ObligationsPersistenceErrorV1>,
    {
        if !valid_lifecycle_operation(&input) {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || encode($2, 'hex'), 0))")
            .bind(&input.logical_owner_id)
            .bind(input.operation_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        if let Some(response) = load_operation_replay(&mut transaction, &input).await? {
            transaction.commit().await.map_err(storage)?;
            return Ok(ObligationsLifecycleOperationOutcomeV1::Replayed {
                response_bytes: response,
            });
        }

        let obligation_id = mutation_obligation_id(&input.mutation);
        let mut obligation = load_obligation(
            &mut transaction,
            &input.logical_owner_id,
            obligation_id,
            true,
        )
        .await?
        .ok_or(ObligationsPersistenceErrorV1::NotFound)?;

        match &input.mutation {
            ObligationsLifecycleMutationV1::Update {
                expected_revision,
                statement,
                condition,
                due_at,
                obligated_party_id,
                beneficiary_party_id,
                changed_at,
                ..
            } => update_obligation_content_v1(
                &mut obligation,
                *expected_revision,
                statement.clone(),
                condition.clone(),
                *due_at,
                *obligated_party_id,
                *beneficiary_party_id,
                *changed_at,
            )
            .map_err(core_error)?,
            ObligationsLifecycleMutationV1::SetState {
                expected_revision,
                state,
                changed_at,
                ..
            } => set_obligation_state_v1(&mut obligation, *expected_revision, *state, *changed_at)
                .map_err(core_error)?,
            ObligationsLifecycleMutationV1::AddEvidence {
                expected_revision,
                evidence,
                changed_at,
                ..
            } => add_obligation_evidence_v1(
                &mut obligation,
                *expected_revision,
                evidence.clone(),
                *changed_at,
            )
            .map_err(core_error)?,
            ObligationsLifecycleMutationV1::RemoveEvidence {
                expected_revision,
                evidence_link_id,
                changed_at,
                ..
            } => remove_obligation_evidence_v1(
                &mut obligation,
                *expected_revision,
                *evidence_link_id,
                *changed_at,
            )
            .map_err(core_error)?,
        }
        validate_obligation_record_v1(&obligation).map_err(core_error)?;

        persist_obligation(&mut transaction, &obligation, false).await?;
        let commit = build_commit(&obligation)?;
        if !valid_lifecycle_commit(&commit) {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        insert_event(
            &mut transaction,
            &input.logical_owner_id,
            &commit,
            input.received_at_unix_millis,
        )
        .await?;
        insert_operation(&mut transaction, &input, &obligation, &commit).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(ObligationsLifecycleOperationOutcomeV1::Applied {
            obligation: Box::new(obligation),
            response_bytes: commit.response_bytes,
        })
    }

    pub async fn get_lifecycle_obligation(
        &self,
        logical_owner_id: &str,
        obligation_id: [u8; 16],
    ) -> Result<Option<ObligationRecordV1>, ObligationsPersistenceErrorV1> {
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let obligation =
            load_obligation(&mut transaction, logical_owner_id, obligation_id, false).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(obligation)
    }

    pub async fn list_lifecycle_obligations(
        &self,
        logical_owner_id: &str,
        after_obligation_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<ObligationRecordV1>, ObligationsPersistenceErrorV1> {
        if limit == 0 || limit > 201 {
            return Err(ObligationsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT obligation_id FROM makosh_data.obligations_state \
             WHERE logical_owner_id = $1 AND ($2::bytea IS NULL OR obligation_id > $2) \
             ORDER BY obligation_id LIMIT $3",
        )
        .bind(logical_owner_id)
        .bind(after_obligation_id.map(|value| value.to_vec()))
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?;
        let mut obligations = Vec::with_capacity(rows.len());
        for row in rows {
            let obligation_id = fixed::<16>(row.try_get("obligation_id").map_err(storage)?)?;
            obligations.push(
                load_obligation(&mut transaction, logical_owner_id, obligation_id, false)
                    .await?
                    .ok_or(ObligationsPersistenceErrorV1::InvalidRow)?,
            );
        }
        transaction.commit().await.map_err(storage)?;
        Ok(obligations)
    }
}

async fn load_operation_replay(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ObligationsLifecycleOperationV1,
) -> Result<Option<Vec<u8>>, ObligationsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT operation_kind, request_sha256, request_bytes, response_sha256, response_bytes \
         FROM makosh_data.obligations_client_operations \
         WHERE logical_owner_id = $1 AND operation_id = $2 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let request_sha = fixed::<32>(row.try_get("request_sha256").map_err(storage)?)?;
    let request_bytes: Vec<u8> = row.try_get("request_bytes").map_err(storage)?;
    let response_sha = fixed::<32>(row.try_get("response_sha256").map_err(storage)?)?;
    let response_bytes: Vec<u8> = row.try_get("response_bytes").map_err(storage)?;
    if row.try_get::<i16, _>("operation_kind").map_err(storage)? != input.mutation.operation_kind()
        || request_sha != input.request_sha256
        || request_bytes != input.request_bytes
        || Sha256::digest(&response_bytes).as_slice() != response_sha
    {
        return Err(ObligationsPersistenceErrorV1::OperationConflict);
    }
    Ok(Some(response_bytes))
}

async fn load_obligation(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    obligation_id: [u8; 16],
    lock: bool,
) -> Result<Option<ObligationRecordV1>, ObligationsPersistenceErrorV1> {
    let sql = if lock {
        "SELECT obligation_id, logical_owner_id, statement, condition, due_at_unix_seconds, due_at_nanos, \
         status, obligation_revision, obligated_party_id, beneficiary_party_id, \
         created_at_unix_seconds, created_at_nanos, \
         updated_at_unix_seconds, updated_at_nanos FROM makosh_data.obligations_state \
         WHERE logical_owner_id = $1 AND obligation_id = $2 FOR UPDATE"
    } else {
        "SELECT obligation_id, logical_owner_id, statement, condition, due_at_unix_seconds, due_at_nanos, \
         status, obligation_revision, obligated_party_id, beneficiary_party_id, \
         created_at_unix_seconds, created_at_nanos, \
         updated_at_unix_seconds, updated_at_nanos FROM makosh_data.obligations_state \
         WHERE logical_owner_id = $1 AND obligation_id = $2"
    };
    let Some(row) = sqlx::query(sql)
        .bind(logical_owner_id)
        .bind(obligation_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?
    else {
        return Ok(None);
    };
    let evidence_links = sqlx::query(
        "SELECT evidence_link_id, evidence_owner_id, evidence_record_id, evidence_revision, evidence_digest \
         FROM makosh_data.obligations_evidence WHERE logical_owner_id = $1 AND obligation_id = $2 \
         ORDER BY evidence_link_id",
    )
    .bind(logical_owner_id)
    .bind(obligation_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?
    .into_iter()
    .map(|row| {
        Ok(ObligationEvidenceLinkV1 {
            evidence_link_id: fixed(row.try_get("evidence_link_id").map_err(storage)?)?,
            evidence_owner_id: row.try_get("evidence_owner_id").map_err(storage)?,
            evidence_record_id: fixed(row.try_get("evidence_record_id").map_err(storage)?)?,
            evidence_revision: positive_u64(row.try_get("evidence_revision").map_err(storage)?)?,
            evidence_digest: fixed(row.try_get("evidence_digest").map_err(storage)?)?,
        })
    })
    .collect::<Result<Vec<_>, ObligationsPersistenceErrorV1>>()?;
    let due_seconds: Option<i64> = row.try_get("due_at_unix_seconds").map_err(storage)?;
    let due_nanos: Option<i32> = row.try_get("due_at_nanos").map_err(storage)?;
    let due_at = match (due_seconds, due_nanos) {
        (None, None) => None,
        (Some(unix_seconds), Some(nanos)) => Some(ObligationTimestampV1 {
            unix_seconds,
            nanos,
        }),
        _ => return Err(ObligationsPersistenceErrorV1::InvalidRow),
    };
    let obligation = ObligationRecordV1 {
        obligation_id: fixed(row.try_get("obligation_id").map_err(storage)?)?,
        logical_owner_id: row.try_get("logical_owner_id").map_err(storage)?,
        statement: row.try_get("statement").map_err(storage)?,
        condition: row.try_get("condition").map_err(storage)?,
        due_at,
        state: decode_state(row.try_get("status").map_err(storage)?)?,
        obligation_revision: positive_u64(row.try_get("obligation_revision").map_err(storage)?)?,
        obligated_party_id: fixed(row.try_get("obligated_party_id").map_err(storage)?)?,
        beneficiary_party_id: row
            .try_get::<Option<Vec<u8>>, _>("beneficiary_party_id")
            .map_err(storage)?
            .map(fixed)
            .transpose()?,
        evidence_links,
        created_at: ObligationTimestampV1 {
            unix_seconds: row.try_get("created_at_unix_seconds").map_err(storage)?,
            nanos: row.try_get("created_at_nanos").map_err(storage)?,
        },
        updated_at: ObligationTimestampV1 {
            unix_seconds: row.try_get("updated_at_unix_seconds").map_err(storage)?,
            nanos: row.try_get("updated_at_nanos").map_err(storage)?,
        },
    };
    validate_obligation_record_v1(&obligation).map_err(core_error)?;
    Ok(Some(obligation))
}

async fn persist_obligation(
    transaction: &mut Transaction<'_, Postgres>,
    obligation: &ObligationRecordV1,
    create: bool,
) -> Result<(), ObligationsPersistenceErrorV1> {
    if create {
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.obligations_state (logical_owner_id, obligation_id, statement, condition, \
             due_at_unix_seconds, due_at_nanos, status, obligation_revision, obligated_party_id, \
             beneficiary_party_id, created_at_unix_seconds, created_at_nanos, \
             updated_at_unix_seconds, updated_at_nanos) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14) ON CONFLICT DO NOTHING",
        )
        .bind(&obligation.logical_owner_id)
        .bind(obligation.obligation_id.as_slice())
        .bind(&obligation.statement)
        .bind(&obligation.condition)
        .bind(obligation.due_at.map(|value| value.unix_seconds))
        .bind(obligation.due_at.map(|value| value.nanos))
        .bind(encode_state(obligation.state))
        .bind(i64_value(obligation.obligation_revision)?)
        .bind(obligation.obligated_party_id.as_slice())
        .bind(obligation.beneficiary_party_id.map(|value| value.to_vec()))
        .bind(obligation.created_at.unix_seconds)
        .bind(obligation.created_at.nanos)
        .bind(obligation.updated_at.unix_seconds)
        .bind(obligation.updated_at.nanos)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
        if inserted.rows_affected() != 1 {
            return Err(ObligationsPersistenceErrorV1::ObligationConflict);
        }
    } else {
        let updated = sqlx::query(
            "UPDATE makosh_data.obligations_state SET statement=$3, condition=$4, due_at_unix_seconds=$5, \
             due_at_nanos=$6, status=$7, obligation_revision=$8, obligated_party_id=$9, \
             beneficiary_party_id=$10, updated_at_unix_seconds=$11, updated_at_nanos=$12 \
             WHERE logical_owner_id=$1 AND obligation_id=$2 AND obligation_revision=$13",
        )
        .bind(&obligation.logical_owner_id)
        .bind(obligation.obligation_id.as_slice())
        .bind(&obligation.statement)
        .bind(&obligation.condition)
        .bind(obligation.due_at.map(|value| value.unix_seconds))
        .bind(obligation.due_at.map(|value| value.nanos))
        .bind(encode_state(obligation.state))
        .bind(i64_value(obligation.obligation_revision)?)
        .bind(obligation.obligated_party_id.as_slice())
        .bind(obligation.beneficiary_party_id.map(|value| value.to_vec()))
        .bind(obligation.updated_at.unix_seconds)
        .bind(obligation.updated_at.nanos)
        .bind(i64_value(obligation.obligation_revision - 1)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
        if updated.rows_affected() != 1 {
            return Err(ObligationsPersistenceErrorV1::RevisionConflict);
        }
    }
    sqlx::query("DELETE FROM makosh_data.obligations_evidence WHERE logical_owner_id=$1 AND obligation_id=$2")
    .bind(&obligation.logical_owner_id)
    .bind(obligation.obligation_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    for evidence in &obligation.evidence_links {
        sqlx::query(
            "INSERT INTO makosh_data.obligations_evidence (logical_owner_id, obligation_id, evidence_link_id, \
             evidence_owner_id, evidence_record_id, evidence_revision, evidence_digest) \
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

async fn insert_event(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    commit: &ObligationsLifecycleCommitV1,
    created_at_unix_millis: i64,
) -> Result<(), ObligationsPersistenceErrorV1> {
    let result = sqlx::query(
        "INSERT INTO makosh_data.obligations_outbox (logical_owner_id, message_id, envelope_sha256, \
         envelope_bytes, created_at_unix_millis) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(commit.lifecycle_event.message_id.as_slice())
    .bind(commit.lifecycle_event.envelope_sha256.as_slice())
    .bind(&commit.lifecycle_event.envelope_bytes)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(ObligationsPersistenceErrorV1::InboxConflict);
    }
    Ok(())
}

async fn insert_operation(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ObligationsLifecycleOperationV1,
    obligation: &ObligationRecordV1,
    commit: &ObligationsLifecycleCommitV1,
) -> Result<(), ObligationsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.obligations_client_operations (logical_owner_id, operation_id, \
         operation_kind, request_sha256, request_bytes, obligation_id, obligation_revision, response_sha256, \
         response_bytes, received_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(&input.logical_owner_id)
    .bind(input.operation_id.as_slice())
    .bind(input.mutation.operation_kind())
    .bind(input.request_sha256.as_slice())
    .bind(&input.request_bytes)
    .bind(obligation.obligation_id.as_slice())
    .bind(i64_value(obligation.obligation_revision)?)
    .bind(commit.response_sha256.as_slice())
    .bind(&commit.response_bytes)
    .bind(input.received_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    Ok(())
}

fn mutation_obligation_id(value: &ObligationsLifecycleMutationV1) -> [u8; 16] {
    match value {
        ObligationsLifecycleMutationV1::Update { obligation_id, .. }
        | ObligationsLifecycleMutationV1::SetState { obligation_id, .. }
        | ObligationsLifecycleMutationV1::AddEvidence { obligation_id, .. }
        | ObligationsLifecycleMutationV1::RemoveEvidence { obligation_id, .. } => *obligation_id,
    }
}

fn decode_state(value: i16) -> Result<ObligationLifecycleStateV1, ObligationsPersistenceErrorV1> {
    match value {
        1 => Ok(ObligationLifecycleStateV1::Open),
        2 => Ok(ObligationLifecycleStateV1::Fulfilled),
        3 => Ok(ObligationLifecycleStateV1::Waived),
        4 => Ok(ObligationLifecycleStateV1::Breached),
        5 => Ok(ObligationLifecycleStateV1::Cancelled),
        _ => Err(ObligationsPersistenceErrorV1::InvalidRow),
    }
}

fn encode_state(value: ObligationLifecycleStateV1) -> i16 {
    match value {
        ObligationLifecycleStateV1::Open => 1,
        ObligationLifecycleStateV1::Fulfilled => 2,
        ObligationLifecycleStateV1::Waived => 3,
        ObligationLifecycleStateV1::Breached => 4,
        ObligationLifecycleStateV1::Cancelled => 5,
    }
}

fn core_error(value: ObligationLifecycleErrorV1) -> ObligationsPersistenceErrorV1 {
    match value {
        ObligationLifecycleErrorV1::RevisionConflict
        | ObligationLifecycleErrorV1::RevisionOverflow => {
            ObligationsPersistenceErrorV1::RevisionConflict
        }
        ObligationLifecycleErrorV1::EvidenceNotFound => ObligationsPersistenceErrorV1::NotFound,
        ObligationLifecycleErrorV1::EvidenceExists
        | ObligationLifecycleErrorV1::InvalidStateTransition => {
            ObligationsPersistenceErrorV1::ObligationConflict
        }
        _ => ObligationsPersistenceErrorV1::InvalidInput,
    }
}

fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], ObligationsPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ObligationsPersistenceErrorV1::InvalidRow)
}

fn positive_u64(value: i64) -> Result<u64, ObligationsPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(ObligationsPersistenceErrorV1::InvalidRow)
}

fn i64_value(value: u64) -> Result<i64, ObligationsPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ObligationsPersistenceErrorV1::InvalidInput)
}

fn storage(_: sqlx::Error) -> ObligationsPersistenceErrorV1 {
    ObligationsPersistenceErrorV1::StorageUnavailable
}
