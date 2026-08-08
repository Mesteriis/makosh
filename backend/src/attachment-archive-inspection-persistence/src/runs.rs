use makosh_attachment_archive_inspection_core::{
    ArchiveEntryInspectionV1, ArchiveInspectionErrorV1, ArchiveInspectionJoinDecisionV1,
    ArchiveInspectionRejectionV1, ArchiveInspectionReportV1, ArchiveInspectionRequestV1,
    ArchiveInspectionStateV1, ArchiveInspectionStatusV1, ArchiveInspectionTransitionV1,
    accepted_archive_inspection_status_v1, archive_inspection_rejection_evidence_id_v1,
    decide_archive_inspection_join_v1, transition_archive_inspection_status_v1,
    validate_archive_inspection_status_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    ARCHIVE_INSPECTION_REALTIME_LIMIT_V1, ArchiveInspectionPersistenceErrorV1,
    ArchiveInspectionRealtimeTransitionV1, AttachmentArchiveInspectionPersistenceV1,
    CreateArchiveInspectionRunOutcomeV1, CreateArchiveInspectionRunV1,
    PersistedArchiveInspectionRunV1, archive_inspection_request_fingerprint_v1,
    archive_inspection_run_id_v1,
    custody::enqueue_archive_inspection_custody_delegation,
    id16, id32,
    model::{
        entry_kind_from_code, error_code, error_from_code, state_code, state_from_code,
        validate_create,
    },
    observations::{load_candidate, load_candidate_envelope_sha256, load_safety},
    unsigned,
};

const SELECT_RUN: &str = "SELECT logical_owner_id, run_id, operation_id, request_fingerprint, attachment_anchor_id, state, state_revision, error_code, rejection_evidence_id, created_at_unix_millis, updated_at_unix_millis FROM makosh_data.attachment_archive_inspection_runs WHERE logical_owner_id = $1 AND run_id = $2";
const SELECT_RUN_FOR_UPDATE: &str = "SELECT logical_owner_id, run_id, operation_id, request_fingerprint, attachment_anchor_id, state, state_revision, error_code, rejection_evidence_id, created_at_unix_millis, updated_at_unix_millis FROM makosh_data.attachment_archive_inspection_runs WHERE logical_owner_id = $1 AND run_id = $2 FOR UPDATE";
const SELECT_RUN_BY_OPERATION: &str = "SELECT logical_owner_id, run_id, operation_id, request_fingerprint, attachment_anchor_id, state, state_revision, error_code, rejection_evidence_id, created_at_unix_millis, updated_at_unix_millis FROM makosh_data.attachment_archive_inspection_runs WHERE logical_owner_id = $1 AND operation_id = $2";

impl AttachmentArchiveInspectionPersistenceV1 {
    pub async fn create_run(
        &self,
        create: &CreateArchiveInspectionRunV1,
    ) -> Result<CreateArchiveInspectionRunOutcomeV1, ArchiveInspectionPersistenceErrorV1> {
        validate_create(create)?;
        let run_id = archive_inspection_run_id_v1(&create.logical_owner_id, create.operation_id);
        let fingerprint = archive_inspection_request_fingerprint_v1(create.attachment_anchor_id);
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        lock_anchor(
            &mut transaction,
            &create.logical_owner_id,
            create.attachment_anchor_id,
        )
        .await?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_archive_inspection_runs (logical_owner_id, run_id, operation_id, request_fingerprint, attachment_anchor_id, state, state_revision, error_code, rejection_evidence_id, created_at_unix_millis, updated_at_unix_millis) VALUES ($1, $2, $3, $4, $5, 1, 1, NULL, NULL, $6, $6) ON CONFLICT (logical_owner_id, operation_id) DO NOTHING",
        )
        .bind(&create.logical_owner_id)
        .bind(run_id.as_slice())
        .bind(create.operation_id.as_slice())
        .bind(fingerprint.as_slice())
        .bind(create.attachment_anchor_id.as_slice())
        .bind(create.created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
        .rows_affected()
            == 1;

        if !inserted {
            let existing = load_by_operation_tx(
                &mut transaction,
                &create.logical_owner_id,
                create.operation_id,
            )
            .await?
            .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
            transaction
                .commit()
                .await
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
            if existing.request_fingerprint == fingerprint {
                let replay = self
                    .load_by_operation(&create.logical_owner_id, create.operation_id)
                    .await?
                    .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
                return Ok(CreateArchiveInspectionRunOutcomeV1::Replayed(replay));
            }
            return Ok(CreateArchiveInspectionRunOutcomeV1::OperationCollision);
        }

        insert_realtime_transition(
            &mut transaction,
            &create.logical_owner_id,
            run_id,
            &accepted_archive_inspection_status_v1(),
            create.created_at_unix_millis,
        )
        .await?;
        settle_run(
            &mut transaction,
            &create.logical_owner_id,
            run_id,
            create.created_at_unix_millis,
        )
        .await?;
        let created = load_run_for_update(&mut transaction, &create.logical_owner_id, run_id)
            .await?
            .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
        transaction
            .commit()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        Ok(CreateArchiveInspectionRunOutcomeV1::Created(created))
    }

    pub async fn load_run(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
    ) -> Result<Option<PersistedArchiveInspectionRunV1>, ArchiveInspectionPersistenceErrorV1> {
        let row = sqlx::query(SELECT_RUN)
            .bind(logical_owner_id)
            .bind(run_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        let Some(row) = row else {
            return Ok(None);
        };
        persisted_from_row(&self.pool, row).await.map(Some)
    }

    pub async fn load_by_operation(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
    ) -> Result<Option<PersistedArchiveInspectionRunV1>, ArchiveInspectionPersistenceErrorV1> {
        let row = sqlx::query(SELECT_RUN_BY_OPERATION)
            .bind(logical_owner_id)
            .bind(operation_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        let Some(row) = row else {
            return Ok(None);
        };
        persisted_from_row(&self.pool, row).await.map(Some)
    }

    pub async fn client_realtime_window(
        &self,
        logical_owner_id: &str,
        after_sequence: u64,
        limit: u32,
    ) -> Result<Vec<ArchiveInspectionRealtimeTransitionV1>, ArchiveInspectionPersistenceErrorV1>
    {
        if logical_owner_id.is_empty() || limit == 0 || limit > ARCHIVE_INSPECTION_REALTIME_LIMIT_V1
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT realtime_sequence, run_id, state, state_revision, error_code, occurred_at_unix_millis FROM makosh_data.attachment_archive_inspection_realtime WHERE logical_owner_id = $1 AND realtime_sequence > $2 ORDER BY realtime_sequence ASC LIMIT $3",
        )
        .bind(logical_owner_id)
        .bind(i64::try_from(after_sequence).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        rows.into_iter().map(realtime_from_row).collect()
    }
}

pub(crate) async fn settle_anchor_runs(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<u32, ArchiveInspectionPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT run_id FROM makosh_data.attachment_archive_inspection_runs WHERE logical_owner_id = $1 AND attachment_anchor_id = $2 AND state IN (1, 2) ORDER BY run_id FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(attachment_anchor_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    let mut transitioned = 0_u32;
    for row in rows {
        let run_id = id16(
            row.try_get::<Vec<u8>, _>("run_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?;
        if settle_run(
            transaction,
            logical_owner_id,
            run_id,
            occurred_at_unix_millis,
        )
        .await?
        {
            transitioned = transitioned
                .checked_add(1)
                .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
        }
    }
    Ok(transitioned)
}

pub(crate) async fn settle_run(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<bool, ArchiveInspectionPersistenceErrorV1> {
    let current = load_run_for_update(transaction, logical_owner_id, run_id)
        .await?
        .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    if matches!(
        current.status.state,
        ArchiveInspectionStateV1::Inspecting
            | ArchiveInspectionStateV1::Ready
            | ArchiveInspectionStateV1::Rejected
    ) {
        return Ok(false);
    }
    let candidate = load_candidate(
        transaction,
        logical_owner_id,
        current.request.attachment_anchor_id,
    )
    .await?;
    let safety = load_safety(
        transaction,
        logical_owner_id,
        current.request.attachment_anchor_id,
    )
    .await?;
    let (transition, rejection_evidence_id) = match decide_archive_inspection_join_v1(
        &current.request,
        candidate.as_ref(),
        safety.as_ref(),
    ) {
        ArchiveInspectionJoinDecisionV1::Waiting => {
            if current.status.state == ArchiveInspectionStateV1::AwaitingEvidence {
                return Ok(false);
            }
            (ArchiveInspectionTransitionV1::AwaitEvidence, None)
        }
        ArchiveInspectionJoinDecisionV1::CustodyDelegationRequired(intent) => {
            let candidate_envelope_sha256 = load_candidate_envelope_sha256(
                transaction,
                logical_owner_id,
                current.request.attachment_anchor_id,
            )
            .await?
            .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
            enqueue_archive_inspection_custody_delegation(
                transaction,
                logical_owner_id,
                &intent,
                candidate_envelope_sha256,
                occurred_at_unix_millis,
            )
            .await?;
            if current.status.state == ArchiveInspectionStateV1::AwaitingEvidence {
                return Ok(false);
            }
            (ArchiveInspectionTransitionV1::AwaitEvidence, None)
        }
        ArchiveInspectionJoinDecisionV1::Reject(rejection) => {
            let error = if rejection == ArchiveInspectionRejectionV1::NotSafe {
                ArchiveInspectionErrorV1::NotSafe
            } else {
                ArchiveInspectionErrorV1::Unavailable
            };
            (
                ArchiveInspectionTransitionV1::Reject(error),
                Some(archive_inspection_rejection_evidence_id_v1(
                    &current.request,
                    rejection,
                )),
            )
        }
    };
    let next = transition_archive_inspection_status_v1(&current.status, transition)
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    persist_status(
        transaction,
        logical_owner_id,
        run_id,
        current.status.state_revision,
        &next,
        rejection_evidence_id,
        occurred_at_unix_millis,
    )
    .await?;
    Ok(true)
}

pub(crate) async fn persist_status(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    expected_revision: u64,
    next: &ArchiveInspectionStatusV1,
    rejection_evidence_id: Option<[u8; 16]>,
    occurred_at_unix_millis: i64,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    if !validate_archive_inspection_status_v1(next) {
        return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
    }
    let rows = sqlx::query(
        "UPDATE makosh_data.attachment_archive_inspection_runs SET state = $3, state_revision = $4, error_code = $5, rejection_evidence_id = $6, updated_at_unix_millis = $7 WHERE logical_owner_id = $1 AND run_id = $2 AND state_revision = $8",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(state_code(next.state))
    .bind(i64::try_from(next.state_revision).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
    .bind(next.error.map(error_code))
    .bind(rejection_evidence_id.map(|value| value.to_vec()))
    .bind(occurred_at_unix_millis)
    .bind(i64::try_from(expected_revision).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    if rows != 1 {
        return Err(ArchiveInspectionPersistenceErrorV1::StorageUnavailable);
    }
    insert_realtime_transition(
        transaction,
        logical_owner_id,
        run_id,
        next,
        occurred_at_unix_millis,
    )
    .await
}

pub(crate) async fn insert_realtime_transition(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    status: &ArchiveInspectionStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_archive_inspection_realtime (logical_owner_id, run_id, state, state_revision, error_code, occurred_at_unix_millis) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(state_code(status.state))
    .bind(i64::try_from(status.state_revision).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
    .bind(status.error.map(error_code))
    .bind(occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)
}

pub(crate) async fn lock_anchor(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    let mut bytes = [0_u8; 8];
    let digest = archive_inspection_request_fingerprint_v1(attachment_anchor_id);
    bytes.copy_from_slice(&digest[..8]);
    let owner_salt = logical_owner_id
        .as_bytes()
        .iter()
        .fold(0_i64, |value, byte| value.rotate_left(5) ^ i64::from(*byte));
    let key = i64::from_be_bytes(bytes) ^ owner_salt;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(key)
        .execute(&mut **transaction)
        .await
        .map(|_| ())
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)
}

pub(crate) async fn load_run_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
) -> Result<Option<PersistedArchiveInspectionRunV1>, ArchiveInspectionPersistenceErrorV1> {
    let row = sqlx::query(SELECT_RUN_FOR_UPDATE)
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    row.map(persisted_without_report_from_row).transpose()
}

async fn load_by_operation_tx(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
) -> Result<Option<PersistedArchiveInspectionRunV1>, ArchiveInspectionPersistenceErrorV1> {
    let row = sqlx::query(SELECT_RUN_BY_OPERATION)
        .bind(logical_owner_id)
        .bind(operation_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    row.map(persisted_without_report_from_row).transpose()
}

async fn persisted_from_row(
    pool: &sqlx::PgPool,
    row: sqlx::postgres::PgRow,
) -> Result<PersistedArchiveInspectionRunV1, ArchiveInspectionPersistenceErrorV1> {
    let mut persisted = persisted_without_report_from_row(row)?;
    if persisted.status.state == ArchiveInspectionStateV1::Ready {
        persisted.status.report =
            Some(load_report(pool, &persisted.logical_owner_id, persisted.request.run_id).await?);
    }
    if !validate_archive_inspection_status_v1(&persisted.status) {
        return Err(ArchiveInspectionPersistenceErrorV1::InvalidRow);
    }
    Ok(persisted)
}

fn persisted_without_report_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PersistedArchiveInspectionRunV1, ArchiveInspectionPersistenceErrorV1> {
    let logical_owner_id = row
        .try_get::<String, _>("logical_owner_id")
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    let state = state_from_code(
        row.try_get("state")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
    )?;
    let error = row
        .try_get::<Option<i16>, _>("error_code")
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
        .map(error_from_code)
        .transpose()?;
    let status = ArchiveInspectionStatusV1 {
        state,
        state_revision: unsigned(
            row.try_get("state_revision")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        )?,
        report: None,
        error,
    };
    if state != ArchiveInspectionStateV1::Ready && !validate_archive_inspection_status_v1(&status) {
        return Err(ArchiveInspectionPersistenceErrorV1::InvalidRow);
    }
    Ok(PersistedArchiveInspectionRunV1 {
        logical_owner_id,
        request: ArchiveInspectionRequestV1 {
            run_id: id16(
                row.try_get::<Vec<u8>, _>("run_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
            operation_id: id16(
                row.try_get::<Vec<u8>, _>("operation_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
            attachment_anchor_id: id16(
                row.try_get::<Vec<u8>, _>("attachment_anchor_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
        },
        request_fingerprint: id32(
            row.try_get::<Vec<u8>, _>("request_fingerprint")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?,
        status,
        rejection_evidence_id: row
            .try_get::<Option<Vec<u8>>, _>("rejection_evidence_id")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
            .map(|value| id16(&value))
            .transpose()?,
        created_at_unix_millis: row
            .try_get("created_at_unix_millis")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        updated_at_unix_millis: row
            .try_get("updated_at_unix_millis")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
    })
}

async fn load_report(
    pool: &sqlx::PgPool,
    logical_owner_id: &str,
    run_id: [u8; 16],
) -> Result<ArchiveInspectionReportV1, ArchiveInspectionPersistenceErrorV1> {
    let report = sqlx::query(
        "SELECT entry_count, total_uncompressed_bytes FROM makosh_data.attachment_archive_inspection_reports WHERE logical_owner_id = $1 AND run_id = $2",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .fetch_one(pool)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    let entry_count = usize::try_from(
        report
            .try_get::<i32, _>("entry_count")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
    )
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    let total_uncompressed_bytes = unsigned(
        report
            .try_get("total_uncompressed_bytes")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
    )?;
    let rows = sqlx::query(
        "SELECT normalized_path_utf8, compressed_size, uncompressed_size, entry_kind FROM makosh_data.attachment_archive_inspection_report_entries WHERE logical_owner_id = $1 AND run_id = $2 ORDER BY entry_ordinal",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .fetch_all(pool)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    let entries = rows
        .into_iter()
        .map(|row| {
            let path = row
                .try_get::<Vec<u8>, _>("normalized_path_utf8")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
            Ok(ArchiveEntryInspectionV1 {
                normalized_path: String::from_utf8(path)
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
                compressed_size: unsigned(
                    row.try_get("compressed_size")
                        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
                )?,
                uncompressed_size: unsigned(
                    row.try_get("uncompressed_size")
                        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
                )?,
                kind: entry_kind_from_code(
                    row.try_get("entry_kind")
                        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
                )?,
            })
        })
        .collect::<Result<Vec<_>, ArchiveInspectionPersistenceErrorV1>>()?;
    Ok(ArchiveInspectionReportV1 {
        entry_count,
        total_uncompressed_bytes,
        entries,
    })
}

fn realtime_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<ArchiveInspectionRealtimeTransitionV1, ArchiveInspectionPersistenceErrorV1> {
    Ok(ArchiveInspectionRealtimeTransitionV1 {
        sequence: unsigned(
            row.try_get("realtime_sequence")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        )?,
        run_id: id16(
            row.try_get::<Vec<u8>, _>("run_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?,
        state: state_from_code(
            row.try_get("state")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        )?,
        state_revision: unsigned(
            row.try_get("state_revision")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        )?,
        error: row
            .try_get::<Option<i16>, _>("error_code")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
            .map(error_from_code)
            .transpose()?,
        occurred_at_unix_millis: row
            .try_get("occurred_at_unix_millis")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
    })
}
