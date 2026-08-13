use makosh_review_attention_core::{
    ApplyReviewAttentionV1, ReviewAttentionCommandV1, ReviewAttentionErrorV1,
    ReviewAttentionOutcomeV1, ReviewAttentionV1, ReviewDispositionV1, ReviewImportanceV1,
    ReviewTimestampV1, STABLE_ID_BYTES_V1, apply_review_attention_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::realtime::insert_realtime_transition;

const DISPOSITION_PENDING: i16 = 1;
const DISPOSITION_REVIEWED: i16 = 2;
const DISPOSITION_DISMISSED: i16 = 3;
const IMPORTANCE_NORMAL: i16 = 1;
const IMPORTANCE_IMPORTANT: i16 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyReviewAttentionOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub expected_revision: u64,
    pub command: ReviewAttentionCommandV1,
    pub applied_at: ReviewTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAttentionPersistenceOutcomeV1 {
    pub attention: ReviewAttentionV1,
    pub replayed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewAttentionPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    OperationConflict,
    Domain(ReviewAttentionErrorV1),
}

#[derive(Clone)]
pub struct ReviewAttentionPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl ReviewAttentionPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ReviewAttentionPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ReviewAttentionPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| ReviewAttentionPersistenceErrorV1::StorageUnavailable)?;
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(port)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(|error| {
                report_developer_database_error("connect", &error);
                ReviewAttentionPersistenceErrorV1::StorageUnavailable
            })?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), ReviewAttentionPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| {
                report_developer_database_error("readiness", &error);
                ReviewAttentionPersistenceErrorV1::StorageUnavailable
            })
    }

    pub async fn apply_operation(
        &self,
        operation: ApplyReviewAttentionOperationV1,
    ) -> Result<ReviewAttentionPersistenceOutcomeV1, ReviewAttentionPersistenceErrorV1> {
        validate_operation(&operation)?;
        let request_sha256 = request_sha256(&operation);
        let mut transaction = self
            .begin_owner_transaction(&operation.logical_owner_id)
            .await?;
        let reserved = sqlx::query(
            "INSERT INTO makosh_data.review_attention_operations (
               logical_owner_id, operation_id, request_sha256, expected_revision,
               completed, requested_at_unix_seconds
             ) VALUES ($1, $2, $3, $4, FALSE, $5)
             ON CONFLICT (logical_owner_id, operation_id) DO NOTHING",
        )
        .bind(&operation.logical_owner_id)
        .bind(operation.operation_id.as_slice())
        .bind(request_sha256.as_slice())
        .bind(signed(operation.expected_revision)?)
        .bind(operation.applied_at.unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected()
            == 1;

        if !reserved {
            let replay = load_operation_replay(
                &mut transaction,
                &operation.logical_owner_id,
                &operation.operation_id,
                &request_sha256,
                &operation.source_evidence_id,
            )
            .await?;
            transaction.commit().await.map_err(storage_error)?;
            return Ok(replay);
        }

        let current = load_attention_for_update(
            &mut transaction,
            &operation.logical_owner_id,
            &operation.source_evidence_id,
        )
        .await?;
        let outcome = apply_review_attention_v1(
            current.as_ref(),
            &ApplyReviewAttentionV1 {
                logical_owner_id: operation.logical_owner_id.clone(),
                source_evidence_id: operation.source_evidence_id,
                expected_revision: operation.expected_revision,
                command: operation.command,
                applied_at: operation.applied_at,
            },
        )
        .map_err(ReviewAttentionPersistenceErrorV1::Domain)?;
        if outcome.changed {
            persist_attention(&mut transaction, &operation.logical_owner_id, &outcome).await?;
            insert_realtime_transition(
                &mut transaction,
                &operation.logical_owner_id,
                &outcome.attention,
            )
            .await?;
        }
        complete_operation(
            &mut transaction,
            &operation.logical_owner_id,
            &operation.operation_id,
            &outcome.attention,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(ReviewAttentionPersistenceOutcomeV1 {
            attention: outcome.attention,
            replayed: false,
        })
    }

    pub(crate) async fn begin_owner_transaction(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, ReviewAttentionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(ReviewAttentionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage_error)?;
        Ok(transaction)
    }
}

pub(crate) fn valid_owner(owner: &str) -> bool {
    !owner.is_empty()
        && owner.len() <= 128
        && owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn report_developer_database_error(stage: &str, error: &sqlx::Error) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_none() {
        return;
    }
    let code = error
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .unwrap_or(std::borrow::Cow::Borrowed("transport"));
    eprintln!("developer_review_attention_database_error stage={stage} code={code}");
}

async fn load_attention_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    source_evidence_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<Option<ReviewAttentionV1>, ReviewAttentionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT attention_id, source_evidence_id, state_revision, disposition,
                pinned, importance, snoozed_until_unix_seconds,
                snoozed_until_nanos, updated_at_unix_seconds, updated_at_nanos
         FROM makosh_data.review_attention_state
         WHERE logical_owner_id = $1 AND source_evidence_id = $2
         FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(source_evidence_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?;
    row.map(|row| attention_from_row(&row)).transpose()
}

async fn persist_attention(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    outcome: &ReviewAttentionOutcomeV1,
) -> Result<(), ReviewAttentionPersistenceErrorV1> {
    let attention = &outcome.attention;
    sqlx::query(
        "INSERT INTO makosh_data.review_attention_state (
           logical_owner_id, attention_id, source_evidence_id, state_revision,
           disposition, pinned, importance, snoozed_until_unix_seconds,
           snoozed_until_nanos, updated_at_unix_seconds, updated_at_nanos
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT (logical_owner_id, source_evidence_id) DO UPDATE SET
           state_revision = EXCLUDED.state_revision,
           disposition = EXCLUDED.disposition,
           pinned = EXCLUDED.pinned,
           importance = EXCLUDED.importance,
           snoozed_until_unix_seconds = EXCLUDED.snoozed_until_unix_seconds,
           snoozed_until_nanos = EXCLUDED.snoozed_until_nanos,
           updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds,
           updated_at_nanos = EXCLUDED.updated_at_nanos
         WHERE makosh_data.review_attention_state.state_revision = $12",
    )
    .bind(logical_owner_id)
    .bind(attention.attention_id.as_slice())
    .bind(attention.source_evidence_id.as_slice())
    .bind(signed(attention.revision)?)
    .bind(disposition_code(attention.disposition))
    .bind(attention.pinned)
    .bind(importance_code(attention.importance))
    .bind(attention.snoozed_until.map(|value| value.unix_seconds))
    .bind(attention.snoozed_until.map(|value| value.nanos))
    .bind(attention.updated_at.unix_seconds)
    .bind(attention.updated_at.nanos)
    .bind(signed(attention.revision.saturating_sub(1))?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)
    .and_then(|result| {
        (result.rows_affected() == 1).then_some(()).ok_or(
            ReviewAttentionPersistenceErrorV1::Domain(ReviewAttentionErrorV1::RevisionConflict),
        )
    })
}

async fn complete_operation(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: &[u8; STABLE_ID_BYTES_V1],
    attention: &ReviewAttentionV1,
) -> Result<(), ReviewAttentionPersistenceErrorV1> {
    let result = sqlx::query(
        "UPDATE makosh_data.review_attention_operations SET
           attention_id = $3,
           result_revision = $4,
           result_disposition = $5,
           result_pinned = $6,
           result_importance = $7,
           result_snoozed_until_unix_seconds = $8,
           result_snoozed_until_nanos = $9,
           result_updated_at_unix_seconds = $10,
           result_updated_at_nanos = $11,
           completed = TRUE
         WHERE logical_owner_id = $1 AND operation_id = $2 AND completed = FALSE",
    )
    .bind(logical_owner_id)
    .bind(operation_id.as_slice())
    .bind(attention.attention_id.as_slice())
    .bind(signed(attention.revision)?)
    .bind(disposition_code(attention.disposition))
    .bind(attention.pinned)
    .bind(importance_code(attention.importance))
    .bind(attention.snoozed_until.map(|value| value.unix_seconds))
    .bind(attention.snoozed_until.map(|value| value.nanos))
    .bind(attention.updated_at.unix_seconds)
    .bind(attention.updated_at.nanos)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(ReviewAttentionPersistenceErrorV1::OperationConflict)
    }
}

async fn load_operation_replay(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: &[u8; STABLE_ID_BYTES_V1],
    request_sha256: &[u8; 32],
    source_evidence_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<ReviewAttentionPersistenceOutcomeV1, ReviewAttentionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT request_sha256, attention_id, result_revision, result_disposition,
                result_pinned, result_importance,
                result_snoozed_until_unix_seconds, result_snoozed_until_nanos,
                result_updated_at_unix_seconds, result_updated_at_nanos, completed
         FROM makosh_data.review_attention_operations
         WHERE logical_owner_id = $1 AND operation_id = $2
         FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(operation_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)?;
    let stored_sha256: Vec<u8> = row.try_get("request_sha256").map_err(row_error)?;
    let completed: bool = row.try_get("completed").map_err(row_error)?;
    if stored_sha256.as_slice() != request_sha256 || !completed {
        return Err(ReviewAttentionPersistenceErrorV1::OperationConflict);
    }
    Ok(ReviewAttentionPersistenceOutcomeV1 {
        attention: ReviewAttentionV1 {
            attention_id: id16(row.try_get("attention_id").map_err(row_error)?)?,
            source_evidence_id: *source_evidence_id,
            revision: positive_u64(row.try_get("result_revision").map_err(row_error)?)?,
            disposition: disposition(row.try_get("result_disposition").map_err(row_error)?)?,
            pinned: row.try_get("result_pinned").map_err(row_error)?,
            importance: importance(row.try_get("result_importance").map_err(row_error)?)?,
            snoozed_until: optional_timestamp(
                row.try_get("result_snoozed_until_unix_seconds")
                    .map_err(row_error)?,
                row.try_get("result_snoozed_until_nanos")
                    .map_err(row_error)?,
            )?,
            updated_at: timestamp(
                row.try_get("result_updated_at_unix_seconds")
                    .map_err(row_error)?,
                row.try_get("result_updated_at_nanos").map_err(row_error)?,
            )?,
        },
        replayed: true,
    })
}

pub(crate) fn attention_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<ReviewAttentionV1, ReviewAttentionPersistenceErrorV1> {
    Ok(ReviewAttentionV1 {
        attention_id: id16(row.try_get("attention_id").map_err(row_error)?)?,
        source_evidence_id: id16(row.try_get("source_evidence_id").map_err(row_error)?)?,
        revision: positive_u64(row.try_get("state_revision").map_err(row_error)?)?,
        disposition: disposition(row.try_get("disposition").map_err(row_error)?)?,
        pinned: row.try_get("pinned").map_err(row_error)?,
        importance: importance(row.try_get("importance").map_err(row_error)?)?,
        snoozed_until: optional_timestamp(
            row.try_get("snoozed_until_unix_seconds")
                .map_err(row_error)?,
            row.try_get("snoozed_until_nanos").map_err(row_error)?,
        )?,
        updated_at: timestamp(
            row.try_get("updated_at_unix_seconds").map_err(row_error)?,
            row.try_get("updated_at_nanos").map_err(row_error)?,
        )?,
    })
}

fn validate_operation(
    operation: &ApplyReviewAttentionOperationV1,
) -> Result<(), ReviewAttentionPersistenceErrorV1> {
    if operation.operation_id.iter().all(|byte| *byte == 0) {
        return Err(ReviewAttentionPersistenceErrorV1::InvalidInput);
    }
    apply_review_attention_v1(
        None,
        &ApplyReviewAttentionV1 {
            logical_owner_id: operation.logical_owner_id.clone(),
            source_evidence_id: operation.source_evidence_id,
            expected_revision: 0,
            command: ReviewAttentionCommandV1::MarkPending,
            applied_at: operation.applied_at,
        },
    )
    .map(|_| ())
    .map_err(ReviewAttentionPersistenceErrorV1::Domain)
}

fn request_sha256(operation: &ApplyReviewAttentionOperationV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.review.attention.operation.v1");
    hash.update([0]);
    hash.update((operation.logical_owner_id.len() as u64).to_be_bytes());
    hash.update(operation.logical_owner_id.as_bytes());
    hash.update(operation.operation_id);
    hash.update(operation.source_evidence_id);
    hash.update(operation.expected_revision.to_be_bytes());
    match operation.command {
        ReviewAttentionCommandV1::MarkPending => hash.update([1]),
        ReviewAttentionCommandV1::MarkReviewed => hash.update([2]),
        ReviewAttentionCommandV1::Dismiss => hash.update([3]),
        ReviewAttentionCommandV1::SetPinned(value) => hash.update([4, u8::from(value)]),
        ReviewAttentionCommandV1::SetImportance(value) => {
            hash.update([
                5,
                u8::try_from(importance_code(value)).expect("importance code"),
            ]);
        }
        ReviewAttentionCommandV1::SnoozeUntil(value) => {
            hash.update([6]);
            hash.update(value.unix_seconds.to_be_bytes());
            hash.update(value.nanos.to_be_bytes());
        }
        ReviewAttentionCommandV1::ClearSnooze => hash.update([7]),
    }
    hash.update(operation.applied_at.unix_seconds.to_be_bytes());
    hash.update(operation.applied_at.nanos.to_be_bytes());
    hash.finalize().into()
}

pub(crate) const fn disposition_code(value: ReviewDispositionV1) -> i16 {
    match value {
        ReviewDispositionV1::Pending => DISPOSITION_PENDING,
        ReviewDispositionV1::Reviewed => DISPOSITION_REVIEWED,
        ReviewDispositionV1::Dismissed => DISPOSITION_DISMISSED,
    }
}

pub(crate) const fn importance_code(value: ReviewImportanceV1) -> i16 {
    match value {
        ReviewImportanceV1::Normal => IMPORTANCE_NORMAL,
        ReviewImportanceV1::Important => IMPORTANCE_IMPORTANT,
    }
}

pub(crate) fn disposition(
    value: i16,
) -> Result<ReviewDispositionV1, ReviewAttentionPersistenceErrorV1> {
    match value {
        DISPOSITION_PENDING => Ok(ReviewDispositionV1::Pending),
        DISPOSITION_REVIEWED => Ok(ReviewDispositionV1::Reviewed),
        DISPOSITION_DISMISSED => Ok(ReviewDispositionV1::Dismissed),
        _ => Err(ReviewAttentionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn importance(
    value: i16,
) -> Result<ReviewImportanceV1, ReviewAttentionPersistenceErrorV1> {
    match value {
        IMPORTANCE_NORMAL => Ok(ReviewImportanceV1::Normal),
        IMPORTANCE_IMPORTANT => Ok(ReviewImportanceV1::Important),
        _ => Err(ReviewAttentionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn optional_timestamp(
    seconds: Option<i64>,
    nanos: Option<i32>,
) -> Result<Option<ReviewTimestampV1>, ReviewAttentionPersistenceErrorV1> {
    match (seconds, nanos) {
        (None, None) => Ok(None),
        (Some(seconds), Some(nanos)) => timestamp(seconds, nanos).map(Some),
        _ => Err(ReviewAttentionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn timestamp(
    unix_seconds: i64,
    nanos: i32,
) -> Result<ReviewTimestampV1, ReviewAttentionPersistenceErrorV1> {
    if unix_seconds <= 0 || !(0..1_000_000_000).contains(&nanos) {
        return Err(ReviewAttentionPersistenceErrorV1::InvalidRow);
    }
    Ok(ReviewTimestampV1 {
        unix_seconds,
        nanos,
    })
}

pub(crate) fn id16(
    value: Vec<u8>,
) -> Result<[u8; STABLE_ID_BYTES_V1], ReviewAttentionPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; STABLE_ID_BYTES_V1]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewAttentionPersistenceErrorV1::InvalidRow)
}

pub(crate) fn positive_u64(value: i64) -> Result<u64, ReviewAttentionPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(ReviewAttentionPersistenceErrorV1::InvalidRow)
}

fn signed(value: u64) -> Result<i64, ReviewAttentionPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ReviewAttentionPersistenceErrorV1::InvalidInput)
}

fn storage_error(_: sqlx::Error) -> ReviewAttentionPersistenceErrorV1 {
    ReviewAttentionPersistenceErrorV1::StorageUnavailable
}

fn row_error(_: sqlx::Error) -> ReviewAttentionPersistenceErrorV1 {
    ReviewAttentionPersistenceErrorV1::InvalidRow
}

#[cfg(test)]
mod tests {
    use super::*;

    fn operation(command: ReviewAttentionCommandV1) -> ApplyReviewAttentionOperationV1 {
        ApplyReviewAttentionOperationV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [1; STABLE_ID_BYTES_V1],
            source_evidence_id: [2; STABLE_ID_BYTES_V1],
            expected_revision: 0,
            command,
            applied_at: ReviewTimestampV1 {
                unix_seconds: 1_783_100_000,
                nanos: 3,
            },
        }
    }

    #[test]
    fn operation_hash_is_stable_and_covers_all_semantics() {
        let baseline = operation(ReviewAttentionCommandV1::SetPinned(true));
        assert_eq!(request_sha256(&baseline), request_sha256(&baseline));
        let mut changed = baseline.clone();
        changed.expected_revision = 1;
        assert_ne!(request_sha256(&baseline), request_sha256(&changed));
        changed = baseline.clone();
        changed.command = ReviewAttentionCommandV1::SetPinned(false);
        assert_ne!(request_sha256(&baseline), request_sha256(&changed));
        changed = baseline.clone();
        changed.source_evidence_id = [3; STABLE_ID_BYTES_V1];
        assert_ne!(request_sha256(&baseline), request_sha256(&changed));
    }

    #[test]
    fn operation_validation_rejects_zero_id_and_invalid_owner() {
        let mut invalid = operation(ReviewAttentionCommandV1::MarkPending);
        invalid.operation_id = [0; STABLE_ID_BYTES_V1];
        assert_eq!(
            validate_operation(&invalid),
            Err(ReviewAttentionPersistenceErrorV1::InvalidInput)
        );
        invalid = operation(ReviewAttentionCommandV1::MarkPending);
        invalid.logical_owner_id = "review/provider".to_owned();
        assert_eq!(
            validate_operation(&invalid),
            Err(ReviewAttentionPersistenceErrorV1::Domain(
                ReviewAttentionErrorV1::InvalidOwner
            ))
        );
    }
}
