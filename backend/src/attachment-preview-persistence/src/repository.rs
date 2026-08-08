use makosh_attachment_preview_core::{
    AttachmentPreviewStatusV1, accepted_attachment_preview_status_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentPreviewPersistenceErrorV1, AttachmentPreviewPersistenceV1,
    CreateAttachmentPreviewRunOutcomeV1, CreateAttachmentPreviewRunV1,
    PersistedAttachmentPreviewArtifactV1, PersistedAttachmentPreviewRunV1,
    PreviewRealtimeTransitionV1,
    model::{
        attachment_preview_request_fingerprint_v1, attachment_preview_run_id_v1, content_type_code,
        content_type_from_code, error_code, error_from_code, kind_code, kind_from_code, state_code,
        state_from_code, valid_id16, valid_owner, valid_sha256, valid_timestamp_millis,
        validate_create, validate_status,
    },
};

impl AttachmentPreviewPersistenceV1 {
    pub async fn create_run(
        &self,
        create: &CreateAttachmentPreviewRunV1,
    ) -> Result<CreateAttachmentPreviewRunOutcomeV1, AttachmentPreviewPersistenceErrorV1> {
        validate_create(create)?;
        let run_id = attachment_preview_run_id_v1(&create.logical_owner_id, create.operation_id);
        let fingerprint = attachment_preview_request_fingerprint_v1(create.attachment_anchor_id);
        let status = accepted_attachment_preview_status_v1();
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        lock_anchor(
            &mut transaction,
            &create.logical_owner_id,
            create.attachment_anchor_id,
        )
        .await?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_preview_runs (logical_owner_id,run_id,operation_id,request_fingerprint,attachment_anchor_id,state,state_revision,preview_kind,content_type,preview_size_bytes,truncated,error_code,created_at_unix_millis,updated_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,NULL,NULL,0,FALSE,NULL,$8,$8) ON CONFLICT (logical_owner_id,operation_id) DO NOTHING",
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
            crate::evidence::settle_run(
                &mut transaction,
                &create.logical_owner_id,
                run_id,
                create.created_at_unix_millis,
            )
            .await?;
            let created = load_run_for_update(&mut transaction, &create.logical_owner_id, run_id)
                .await?
                .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
            transaction.commit().await.map_err(storage_unavailable)?;
            return Ok(CreateAttachmentPreviewRunOutcomeV1::Created(created));
        }
        let existing = find_by_operation(
            &mut transaction,
            &create.logical_owner_id,
            create.operation_id,
        )
        .await?
        .ok_or(AttachmentPreviewPersistenceErrorV1::StorageUnavailable)?;
        transaction.commit().await.map_err(storage_unavailable)?;
        if existing.request_fingerprint == fingerprint {
            Ok(CreateAttachmentPreviewRunOutcomeV1::Replayed(existing))
        } else {
            Ok(CreateAttachmentPreviewRunOutcomeV1::OperationCollision)
        }
    }

    pub async fn find_run(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
    ) -> Result<Option<PersistedAttachmentPreviewRunV1>, AttachmentPreviewPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !valid_id16(&run_id) {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(RUN_SELECT_V1)
            .bind(logical_owner_id)
            .bind(run_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_unavailable)?
            .map(run_from_row)
            .transpose()
    }

    pub async fn find_artifact(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
    ) -> Result<Option<PersistedAttachmentPreviewArtifactV1>, AttachmentPreviewPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id) || !valid_id16(&run_id) {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT run_id,derived_reference_id,derived_receipt_sha256,source_receipt_sha256,renderer_identity_sha256,preview_kind,content_type,preview_size_bytes,truncated,runtime_generation,grant_epoch FROM makosh_data.attachment_preview_artifacts WHERE logical_owner_id=$1 AND run_id=$2",
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
    ) -> Result<Vec<PreviewRealtimeTransitionV1>, AttachmentPreviewPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || limit == 0
            || limit > crate::ATTACHMENT_PREVIEW_REALTIME_LIMIT_V1
        {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT realtime_sequence,run_id,state,state_revision,preview_kind,content_type,preview_size_bytes,truncated,error_code,occurred_at_unix_millis FROM makosh_data.attachment_preview_realtime WHERE logical_owner_id=$1 AND realtime_sequence>$2 ORDER BY realtime_sequence LIMIT $3",
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

const RUN_SELECT_V1: &str = "SELECT logical_owner_id,run_id,operation_id,request_fingerprint,attachment_anchor_id,state,state_revision,preview_kind,content_type,preview_size_bytes,truncated,error_code,created_at_unix_millis,updated_at_unix_millis FROM makosh_data.attachment_preview_runs WHERE logical_owner_id=$1 AND run_id=$2";

async fn find_by_operation(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
) -> Result<Option<PersistedAttachmentPreviewRunV1>, AttachmentPreviewPersistenceErrorV1> {
    sqlx::query(
        "SELECT logical_owner_id,run_id,operation_id,request_fingerprint,attachment_anchor_id,state,state_revision,preview_kind,content_type,preview_size_bytes,truncated,error_code,created_at_unix_millis,updated_at_unix_millis FROM makosh_data.attachment_preview_runs WHERE logical_owner_id=$1 AND operation_id=$2",
    )
        .bind(logical_owner_id)
        .bind(operation_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_unavailable)?
        .map(run_from_row)
        .transpose()
}

pub(crate) async fn load_run_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
) -> Result<Option<PersistedAttachmentPreviewRunV1>, AttachmentPreviewPersistenceErrorV1> {
    sqlx::query(
        "SELECT logical_owner_id,run_id,operation_id,request_fingerprint,attachment_anchor_id,state,state_revision,preview_kind,content_type,preview_size_bytes,truncated,error_code,created_at_unix_millis,updated_at_unix_millis FROM makosh_data.attachment_preview_runs WHERE logical_owner_id=$1 AND run_id=$2 FOR UPDATE",
    )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_unavailable)?
        .map(run_from_row)
        .transpose()
}

pub(crate) async fn update_run_status(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    expected_revision: u64,
    next: &AttachmentPreviewStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<bool, AttachmentPreviewPersistenceErrorV1> {
    if !validate_status(next) || !valid_timestamp_millis(occurred_at_unix_millis) {
        return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
    }
    let result = sqlx::query(
        "UPDATE makosh_data.attachment_preview_runs SET state=$1,state_revision=$2,preview_kind=$3,content_type=$4,preview_size_bytes=$5,truncated=$6,error_code=$7,updated_at_unix_millis=$8 WHERE logical_owner_id=$9 AND run_id=$10 AND state_revision=$11",
    )
    .bind(state_code(next.state))
    .bind(i64::try_from(next.state_revision).map_err(invalid_input)?)
    .bind(next.preview_kind.map(kind_code))
    .bind(next.content_type.map(content_type_code))
    .bind(i64::try_from(next.preview_size_bytes).map_err(invalid_input)?)
    .bind(next.truncated)
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
    status: &AttachmentPreviewStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
    if !validate_status(status) || !valid_timestamp_millis(occurred_at_unix_millis) {
        return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
    }
    sqlx::query(
        "INSERT INTO makosh_data.attachment_preview_realtime (logical_owner_id,run_id,state,state_revision,preview_kind,content_type,preview_size_bytes,truncated,error_code,occurred_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(state_code(status.state))
    .bind(i64::try_from(status.state_revision).map_err(invalid_input)?)
    .bind(status.preview_kind.map(kind_code))
    .bind(status.content_type.map(content_type_code))
    .bind(i64::try_from(status.preview_size_bytes).map_err(invalid_input)?)
    .bind(status.truncated)
    .bind(status.error.map(error_code))
    .bind(occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    Ok(())
}

pub(crate) async fn lock_anchor(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attachment_anchor_id: [u8; 16],
) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(b"makosh.attachment-preview.anchor-lock.v1\0");
    hasher.update(logical_owner_id.as_bytes());
    hasher.update(attachment_anchor_id);
    let digest: [u8; 32] = hasher.finalize().into();
    let key = i64::from_be_bytes(digest[..8].try_into().expect("digest prefix"));
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(key)
        .execute(&mut **transaction)
        .await
        .map_err(storage_unavailable)?;
    Ok(())
}

fn run_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PersistedAttachmentPreviewRunV1, AttachmentPreviewPersistenceErrorV1> {
    let status = status_from_row(&row)?;
    let run = PersistedAttachmentPreviewRunV1 {
        logical_owner_id: row.try_get("logical_owner_id").map_err(invalid_row)?,
        run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
        operation_id: id16(row.try_get("operation_id").map_err(invalid_row)?)?,
        request_fingerprint: id32(row.try_get("request_fingerprint").map_err(invalid_row)?)?,
        attachment_anchor_id: id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?,
        status,
        created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(invalid_row)?,
        updated_at_unix_millis: row.try_get("updated_at_unix_millis").map_err(invalid_row)?,
    };
    if !valid_owner(&run.logical_owner_id)
        || !valid_id16(&run.run_id)
        || !valid_id16(&run.operation_id)
        || !valid_sha256(&run.request_fingerprint)
        || !valid_id16(&run.attachment_anchor_id)
        || !valid_timestamp_millis(run.created_at_unix_millis)
        || run.updated_at_unix_millis < run.created_at_unix_millis
    {
        return Err(AttachmentPreviewPersistenceErrorV1::InvalidRow);
    }
    Ok(run)
}

fn status_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<AttachmentPreviewStatusV1, AttachmentPreviewPersistenceErrorV1> {
    let status = AttachmentPreviewStatusV1 {
        state: state_from_code(row.try_get("state").map_err(invalid_row)?)?,
        state_revision: u64::try_from(
            row.try_get::<i64, _>("state_revision")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        preview_kind: row
            .try_get::<Option<i16>, _>("preview_kind")
            .map_err(invalid_row)?
            .map(kind_from_code)
            .transpose()?,
        content_type: row
            .try_get::<Option<i16>, _>("content_type")
            .map_err(invalid_row)?
            .map(content_type_from_code)
            .transpose()?,
        preview_size_bytes: u64::try_from(
            row.try_get::<i64, _>("preview_size_bytes")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        truncated: row.try_get("truncated").map_err(invalid_row)?,
        error: row
            .try_get::<Option<i16>, _>("error_code")
            .map_err(invalid_row)?
            .map(error_from_code)
            .transpose()?,
    };
    if validate_status(&status) {
        Ok(status)
    } else {
        Err(AttachmentPreviewPersistenceErrorV1::InvalidRow)
    }
}

fn artifact_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PersistedAttachmentPreviewArtifactV1, AttachmentPreviewPersistenceErrorV1> {
    let artifact = PersistedAttachmentPreviewArtifactV1 {
        run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
        derived_reference_id: id16(row.try_get("derived_reference_id").map_err(invalid_row)?)?,
        derived_receipt_sha256: id32(row.try_get("derived_receipt_sha256").map_err(invalid_row)?)?,
        source_receipt_sha256: id32(row.try_get("source_receipt_sha256").map_err(invalid_row)?)?,
        renderer_identity_sha256: id32(
            row.try_get("renderer_identity_sha256")
                .map_err(invalid_row)?,
        )?,
        preview_kind: kind_from_code(row.try_get("preview_kind").map_err(invalid_row)?)?,
        content_type: content_type_from_code(row.try_get("content_type").map_err(invalid_row)?)?,
        preview_size_bytes: u64::try_from(
            row.try_get::<i64, _>("preview_size_bytes")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        truncated: row.try_get("truncated").map_err(invalid_row)?,
        runtime_generation: u64::try_from(
            row.try_get::<i64, _>("runtime_generation")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        grant_epoch: u64::try_from(row.try_get::<i64, _>("grant_epoch").map_err(invalid_row)?)
            .map_err(invalid_row)?,
    };
    if !valid_artifact(&artifact) {
        return Err(AttachmentPreviewPersistenceErrorV1::InvalidRow);
    }
    Ok(artifact)
}

fn realtime_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PreviewRealtimeTransitionV1, AttachmentPreviewPersistenceErrorV1> {
    let transition = PreviewRealtimeTransitionV1 {
        sequence: u64::try_from(
            row.try_get::<i64, _>("realtime_sequence")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
        status: status_from_row(&row)?,
        occurred_at_unix_millis: row
            .try_get("occurred_at_unix_millis")
            .map_err(invalid_row)?,
    };
    if transition.sequence == 0
        || !valid_id16(&transition.run_id)
        || !valid_timestamp_millis(transition.occurred_at_unix_millis)
    {
        return Err(AttachmentPreviewPersistenceErrorV1::InvalidRow);
    }
    Ok(transition)
}

pub(crate) fn valid_artifact(value: &PersistedAttachmentPreviewArtifactV1) -> bool {
    valid_id16(&value.run_id)
        && valid_id16(&value.derived_reference_id)
        && valid_sha256(&value.derived_receipt_sha256)
        && valid_sha256(&value.source_receipt_sha256)
        && valid_sha256(&value.renderer_identity_sha256)
        && value.runtime_generation > 0
        && value.grant_epoch > 0
        && makosh_attachment_preview_core::validate_preview_output_v1(
            value.content_type,
            value.preview_size_bytes,
        )
        .is_ok()
        && value.preview_kind
            != makosh_attachment_preview_api::wire::AttachmentPreviewKindV1::Unspecified
}

pub(crate) fn id16(value: Vec<u8>) -> Result<[u8; 16], AttachmentPreviewPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidRow)
}

pub(crate) fn id32(value: Vec<u8>) -> Result<[u8; 32], AttachmentPreviewPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidRow)
}

pub(crate) fn storage_unavailable<T>(_: T) -> AttachmentPreviewPersistenceErrorV1 {
    AttachmentPreviewPersistenceErrorV1::StorageUnavailable
}

pub(crate) fn invalid_input<T>(_: T) -> AttachmentPreviewPersistenceErrorV1 {
    AttachmentPreviewPersistenceErrorV1::InvalidInput
}

pub(crate) fn invalid_row<T>(_: T) -> AttachmentPreviewPersistenceErrorV1 {
    AttachmentPreviewPersistenceErrorV1::InvalidRow
}
