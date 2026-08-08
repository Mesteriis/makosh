use makosh_attachment_text_extraction_core::{
    AttachmentTextExtractionRequestV1, AttachmentTextExtractionStatusV1,
    validate_attachment_text_status_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentTextExtractionPersistenceErrorV1, AttachmentTextExtractionPersistenceV1,
    CreateAttachmentTextExtractionRunOutcomeV1, CreateAttachmentTextExtractionRunV1,
    PersistedAttachmentTextArtifactV1, PersistedAttachmentTextExtractionRunV1,
    TextExtractionRealtimeTransitionV1,
    model::{
        attachment_text_extraction_request_fingerprint_v1, attachment_text_extraction_run_id_v1,
        error_code, error_from_code, format_code, format_from_code, state_code, state_from_code,
        valid_id16, valid_owner, valid_sha256, valid_timestamp_millis, validate_create,
    },
    observations::{lock_anchor, settle_run},
};

impl AttachmentTextExtractionPersistenceV1 {
    pub async fn create_run(
        &self,
        create: &CreateAttachmentTextExtractionRunV1,
    ) -> Result<
        CreateAttachmentTextExtractionRunOutcomeV1,
        AttachmentTextExtractionPersistenceErrorV1,
    > {
        validate_create(create)?;
        let run_id =
            attachment_text_extraction_run_id_v1(&create.logical_owner_id, create.operation_id);
        let fingerprint =
            attachment_text_extraction_request_fingerprint_v1(create.attachment_anchor_id);
        let status = makosh_attachment_text_extraction_core::accepted_attachment_text_status_v1();
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        lock_anchor(
            &mut transaction,
            &create.logical_owner_id,
            create.attachment_anchor_id,
        )
        .await?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_text_extraction_runs (logical_owner_id, run_id, operation_id, request_fingerprint, attachment_anchor_id, state, state_revision, format_code, extracted_size_bytes, extraction_truncated, error_code, created_at_unix_millis, updated_at_unix_millis) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, 0, FALSE, NULL, $8, $8) ON CONFLICT (logical_owner_id, operation_id) DO NOTHING",
        )
        .bind(&create.logical_owner_id)
        .bind(run_id.as_slice())
        .bind(create.operation_id.as_slice())
        .bind(fingerprint.as_slice())
        .bind(create.attachment_anchor_id.as_slice())
        .bind(state_code(status.state))
        .bind(i64::try_from(status.state_revision).map_err(invalid_input)?)
        .bind(create.created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        if inserted.rows_affected() == 1 {
            append_realtime(
                &mut transaction,
                &create.logical_owner_id,
                run_id,
                &status,
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
                .ok_or(AttachmentTextExtractionPersistenceErrorV1::InvalidRow)?;
            transaction.commit().await.map_err(storage_unavailable)?;
            return Ok(CreateAttachmentTextExtractionRunOutcomeV1::Created(created));
        }
        let existing = find_by_operation(
            &mut transaction,
            &create.logical_owner_id,
            create.operation_id,
        )
        .await?
        .ok_or(AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable)?;
        transaction.commit().await.map_err(storage_unavailable)?;
        if existing.request_fingerprint != fingerprint {
            return Ok(CreateAttachmentTextExtractionRunOutcomeV1::OperationCollision);
        }
        Ok(CreateAttachmentTextExtractionRunOutcomeV1::Replayed(
            existing,
        ))
    }

    pub async fn find_run(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
    ) -> Result<
        Option<PersistedAttachmentTextExtractionRunV1>,
        AttachmentTextExtractionPersistenceErrorV1,
    > {
        if !valid_owner(logical_owner_id) || !valid_id16(&run_id) {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT logical_owner_id, run_id, operation_id, request_fingerprint, attachment_anchor_id, state, state_revision, format_code, extracted_size_bytes, extraction_truncated, error_code, created_at_unix_millis, updated_at_unix_millis FROM makosh_data.attachment_text_extraction_runs WHERE logical_owner_id = $1 AND run_id = $2",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_unavailable)?;
        row.map(run_from_row).transpose()
    }

    pub async fn find_artifact(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
    ) -> Result<Option<PersistedAttachmentTextArtifactV1>, AttachmentTextExtractionPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id) || !valid_id16(&run_id) {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT run_id,derived_reference_id,derived_receipt_sha256,source_receipt_sha256,parser_identity_sha256,format_code,extracted_size_bytes,extraction_truncated FROM makosh_data.attachment_text_extraction_artifacts WHERE logical_owner_id=$1 AND run_id=$2",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_unavailable)?
        .map(artifact_from_row)
        .transpose()
    }

    pub async fn realtime_after(
        &self,
        logical_owner_id: &str,
        after_sequence: u64,
        limit: u32,
    ) -> Result<Vec<TextExtractionRealtimeTransitionV1>, AttachmentTextExtractionPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id)
            || limit == 0
            || limit > crate::ATTACHMENT_TEXT_EXTRACTION_REALTIME_LIMIT_V1
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT realtime_sequence,run_id,state,state_revision,format_code,extracted_size_bytes,extraction_truncated,error_code,occurred_at_unix_millis FROM makosh_data.attachment_text_extraction_realtime WHERE logical_owner_id=$1 AND realtime_sequence>$2 ORDER BY realtime_sequence LIMIT $3",
        )
        .bind(logical_owner_id)
        .bind(i64::try_from(after_sequence).map_err(invalid_input)?)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_unavailable)?
        .into_iter()
        .map(realtime_from_row)
        .collect()
    }
}

async fn find_by_operation(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
) -> Result<
    Option<PersistedAttachmentTextExtractionRunV1>,
    AttachmentTextExtractionPersistenceErrorV1,
> {
    let row = sqlx::query(
        "SELECT logical_owner_id, run_id, operation_id, request_fingerprint, attachment_anchor_id, state, state_revision, format_code, extracted_size_bytes, extraction_truncated, error_code, created_at_unix_millis, updated_at_unix_millis FROM makosh_data.attachment_text_extraction_runs WHERE logical_owner_id = $1 AND operation_id = $2",
    )
    .bind(logical_owner_id)
    .bind(operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    row.map(run_from_row).transpose()
}

pub(crate) async fn update_run_status(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    expected_revision: u64,
    next: &AttachmentTextExtractionStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<bool, AttachmentTextExtractionPersistenceErrorV1> {
    let result = sqlx::query(
        "UPDATE makosh_data.attachment_text_extraction_runs SET state = $1, state_revision = $2, format_code = $3, extracted_size_bytes = $4, extraction_truncated = $5, error_code = $6, updated_at_unix_millis = $7 WHERE logical_owner_id = $8 AND run_id = $9 AND state_revision = $10",
    )
    .bind(state_code(next.state))
    .bind(i64::try_from(next.state_revision).map_err(invalid_input)?)
    .bind(next.format.map(format_code))
    .bind(i64::try_from(next.extracted_size_bytes).map_err(invalid_input)?)
    .bind(next.extraction_truncated)
    .bind(next.error.map(error_code))
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(i64::try_from(expected_revision).map_err(invalid_input)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    Ok(result.rows_affected() == 1)
}

pub(crate) async fn append_realtime(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    status: &AttachmentTextExtractionStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_text_extraction_realtime (logical_owner_id, run_id, state, state_revision, format_code, extracted_size_bytes, extraction_truncated, error_code, occurred_at_unix_millis) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(state_code(status.state))
    .bind(i64::try_from(status.state_revision).map_err(invalid_input)?)
    .bind(status.format.map(format_code))
    .bind(i64::try_from(status.extracted_size_bytes).map_err(invalid_input)?)
    .bind(status.extraction_truncated)
    .bind(status.error.map(error_code))
    .bind(occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    Ok(())
}

fn run_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PersistedAttachmentTextExtractionRunV1, AttachmentTextExtractionPersistenceErrorV1> {
    let run_id = id16(row.try_get::<Vec<u8>, _>("run_id").map_err(invalid_row)?)?;
    let operation_id = id16(
        row.try_get::<Vec<u8>, _>("operation_id")
            .map_err(invalid_row)?,
    )?;
    let attachment_anchor_id = id16(
        row.try_get::<Vec<u8>, _>("attachment_anchor_id")
            .map_err(invalid_row)?,
    )?;
    let status = AttachmentTextExtractionStatusV1 {
        state: state_from_code(row.try_get("state").map_err(invalid_row)?)?,
        state_revision: u64::try_from(
            row.try_get::<i64, _>("state_revision")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        format: row
            .try_get::<Option<i16>, _>("format_code")
            .map_err(invalid_row)?
            .map(format_from_code)
            .transpose()?,
        extracted_size_bytes: u64::try_from(
            row.try_get::<i64, _>("extracted_size_bytes")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        extraction_truncated: row.try_get("extraction_truncated").map_err(invalid_row)?,
        error: row
            .try_get::<Option<i16>, _>("error_code")
            .map_err(invalid_row)?
            .map(error_from_code)
            .transpose()?,
    };
    if !validate_attachment_text_status_v1(&status) {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
    }
    Ok(PersistedAttachmentTextExtractionRunV1 {
        logical_owner_id: row.try_get("logical_owner_id").map_err(invalid_row)?,
        request: AttachmentTextExtractionRequestV1 {
            run_id,
            operation_id,
            attachment_anchor_id,
        },
        request_fingerprint: id32(
            row.try_get::<Vec<u8>, _>("request_fingerprint")
                .map_err(invalid_row)?,
        )?,
        status,
        created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(invalid_row)?,
        updated_at_unix_millis: row.try_get("updated_at_unix_millis").map_err(invalid_row)?,
    })
}

pub(crate) async fn load_run_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
) -> Result<
    Option<PersistedAttachmentTextExtractionRunV1>,
    AttachmentTextExtractionPersistenceErrorV1,
> {
    sqlx::query(
        "SELECT logical_owner_id,run_id,operation_id,request_fingerprint,attachment_anchor_id,state,state_revision,format_code,extracted_size_bytes,extraction_truncated,error_code,created_at_unix_millis,updated_at_unix_millis FROM makosh_data.attachment_text_extraction_runs WHERE logical_owner_id=$1 AND run_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .map(run_from_row)
    .transpose()
}

fn artifact_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PersistedAttachmentTextArtifactV1, AttachmentTextExtractionPersistenceErrorV1> {
    let artifact = PersistedAttachmentTextArtifactV1 {
        run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
        derived_reference_id: id16(row.try_get("derived_reference_id").map_err(invalid_row)?)?,
        derived_receipt_sha256: id32(row.try_get("derived_receipt_sha256").map_err(invalid_row)?)?,
        source_receipt_sha256: id32(row.try_get("source_receipt_sha256").map_err(invalid_row)?)?,
        parser_identity_sha256: id32(row.try_get("parser_identity_sha256").map_err(invalid_row)?)?,
        format: format_from_code(row.try_get("format_code").map_err(invalid_row)?)?,
        extracted_size_bytes: u64::try_from(
            row.try_get::<i64, _>("extracted_size_bytes")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        extraction_truncated: row.try_get("extraction_truncated").map_err(invalid_row)?,
    };
    if !valid_id16(&artifact.run_id)
        || !valid_id16(&artifact.derived_reference_id)
        || !valid_sha256(&artifact.derived_receipt_sha256)
        || !valid_sha256(&artifact.source_receipt_sha256)
        || !valid_sha256(&artifact.parser_identity_sha256)
        || !(1..=1_048_576).contains(&artifact.extracted_size_bytes)
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
    }
    Ok(artifact)
}

fn realtime_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<TextExtractionRealtimeTransitionV1, AttachmentTextExtractionPersistenceErrorV1> {
    let transition = TextExtractionRealtimeTransitionV1 {
        sequence: u64::try_from(
            row.try_get::<i64, _>("realtime_sequence")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
        state: state_from_code(row.try_get("state").map_err(invalid_row)?)?,
        state_revision: u64::try_from(
            row.try_get::<i64, _>("state_revision")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        format: row
            .try_get::<Option<i16>, _>("format_code")
            .map_err(invalid_row)?
            .map(format_from_code)
            .transpose()?,
        extracted_size_bytes: u64::try_from(
            row.try_get::<i64, _>("extracted_size_bytes")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        extraction_truncated: row.try_get("extraction_truncated").map_err(invalid_row)?,
        error: row
            .try_get::<Option<i16>, _>("error_code")
            .map_err(invalid_row)?
            .map(error_from_code)
            .transpose()?,
        occurred_at_unix_millis: row
            .try_get("occurred_at_unix_millis")
            .map_err(invalid_row)?,
    };
    let status = AttachmentTextExtractionStatusV1 {
        state: transition.state,
        state_revision: transition.state_revision,
        format: transition.format,
        extracted_size_bytes: transition.extracted_size_bytes,
        extraction_truncated: transition.extraction_truncated,
        error: transition.error,
    };
    if transition.sequence == 0
        || !valid_timestamp_millis(transition.occurred_at_unix_millis)
        || !validate_attachment_text_status_v1(&status)
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
    }
    Ok(transition)
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], AttachmentTextExtractionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::InvalidRow)
}

fn id32(value: Vec<u8>) -> Result<[u8; 32], AttachmentTextExtractionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::InvalidRow)
}

fn storage_unavailable<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable
}

fn invalid_input<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::InvalidInput
}

fn invalid_row<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::InvalidRow
}
