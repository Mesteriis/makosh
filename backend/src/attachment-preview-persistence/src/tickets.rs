//! Hashed, one-use, actor-bound private-content read tickets.

use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_READ_TICKET_TTL_SECONDS_V1, wire::AttachmentPreviewStateV1,
};
use sqlx::Row;

use crate::{
    AttachmentPreviewPersistenceErrorV1, AttachmentPreviewPersistenceV1,
    IssueAttachmentPreviewTicketV1, IssuedAttachmentPreviewTicketV1,
    RedeemedAttachmentPreviewTicketV1,
    model::{content_type_code, content_type_from_code, valid_id16, valid_owner, valid_sha256},
    repository::{id16, id32, invalid_row, storage_unavailable},
};

impl AttachmentPreviewPersistenceV1 {
    pub async fn issue_read_ticket(
        &self,
        logical_owner_id: &str,
        request: IssueAttachmentPreviewTicketV1,
    ) -> Result<IssuedAttachmentPreviewTicketV1, AttachmentPreviewPersistenceErrorV1> {
        if !valid_ticket_request(logical_owner_id, &request) {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let expires_at = request
            .now_unix_seconds
            .checked_add(ATTACHMENT_PREVIEW_READ_TICKET_TTL_SECONDS_V1)
            .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let row = sqlx::query(
            "SELECT r.state_revision,r.content_type,r.preview_size_bytes,a.derived_reference_id,a.derived_receipt_sha256,a.renderer_identity_sha256,a.runtime_generation,a.grant_epoch FROM makosh_data.attachment_preview_runs r JOIN makosh_data.attachment_preview_artifacts a ON a.logical_owner_id=r.logical_owner_id AND a.run_id=r.run_id WHERE r.logical_owner_id=$1 AND r.run_id=$2 AND r.state=$3 FOR UPDATE OF r,a",
        )
        .bind(logical_owner_id)
        .bind(request.run_id.as_slice())
        .bind(AttachmentPreviewStateV1::Ready as i16)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_unavailable)?
        .ok_or(AttachmentPreviewPersistenceErrorV1::NotFound)?;
        let runtime_generation = u64::try_from(
            row.try_get::<i64, _>("runtime_generation")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?;
        let grant_epoch = u64::try_from(row.try_get::<i64, _>("grant_epoch").map_err(invalid_row)?)
            .map_err(invalid_row)?;
        if runtime_generation != request.runtime_generation || grant_epoch != request.grant_epoch {
            return Err(AttachmentPreviewPersistenceErrorV1::StaleFence);
        }
        let state_revision = u64::try_from(
            row.try_get::<i64, _>("state_revision")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?;
        let content_type =
            content_type_from_code(row.try_get("content_type").map_err(invalid_row)?)?;
        let preview_size_bytes = u64::try_from(
            row.try_get::<i64, _>("preview_size_bytes")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_preview_read_tickets (logical_owner_id,ticket_sha256,device_actor_sha256,run_id,state_revision,derived_reference_id,derived_receipt_sha256,renderer_identity_sha256,content_type,preview_size_bytes,runtime_generation,grant_epoch,expires_at_unix_seconds,used_at_unix_seconds,created_at_unix_seconds) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,NULL,$14) ON CONFLICT (logical_owner_id,ticket_sha256) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(request.ticket_sha256.as_slice())
        .bind(request.device_actor_sha256.as_slice())
        .bind(request.run_id.as_slice())
        .bind(i64::try_from(state_revision).map_err(invalid_input)?)
        .bind(row.try_get::<Vec<u8>, _>("derived_reference_id").map_err(invalid_row)?)
        .bind(row.try_get::<Vec<u8>, _>("derived_receipt_sha256").map_err(invalid_row)?)
        .bind(row.try_get::<Vec<u8>, _>("renderer_identity_sha256").map_err(invalid_row)?)
        .bind(content_type_code(content_type))
        .bind(i64::try_from(preview_size_bytes).map_err(invalid_input)?)
        .bind(i64::try_from(runtime_generation).map_err(invalid_input)?)
        .bind(i64::try_from(grant_epoch).map_err(invalid_input)?)
        .bind(expires_at)
        .bind(request.now_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        if inserted.rows_affected() != 1 {
            return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
        }
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(IssuedAttachmentPreviewTicketV1 {
            run_id: request.run_id,
            expires_at_unix_seconds: expires_at,
            content_type,
            preview_size_bytes,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn redeem_read_ticket(
        &self,
        logical_owner_id: &str,
        ticket_sha256: [u8; 32],
        device_actor_sha256: [u8; 32],
        runtime_generation: u64,
        grant_epoch: u64,
        now_unix_seconds: i64,
    ) -> Result<RedeemedAttachmentPreviewTicketV1, AttachmentPreviewPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_sha256(&ticket_sha256)
            || !valid_sha256(&device_actor_sha256)
            || runtime_generation == 0
            || grant_epoch == 0
            || now_unix_seconds <= 0
        {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let row = sqlx::query(
            "SELECT t.device_actor_sha256,t.run_id,t.state_revision,t.derived_reference_id,t.derived_receipt_sha256,t.renderer_identity_sha256,t.content_type,t.preview_size_bytes,t.runtime_generation,t.grant_epoch,t.expires_at_unix_seconds,t.used_at_unix_seconds,r.state,r.state_revision AS current_state_revision,a.derived_reference_id AS current_reference_id,a.derived_receipt_sha256 AS current_receipt_sha256,a.renderer_identity_sha256 AS current_renderer_sha256,a.runtime_generation AS current_runtime_generation,a.grant_epoch AS current_grant_epoch FROM makosh_data.attachment_preview_read_tickets t JOIN makosh_data.attachment_preview_runs r ON r.logical_owner_id=t.logical_owner_id AND r.run_id=t.run_id JOIN makosh_data.attachment_preview_artifacts a ON a.logical_owner_id=t.logical_owner_id AND a.run_id=t.run_id WHERE t.logical_owner_id=$1 AND t.ticket_sha256=$2 FOR UPDATE OF t,r,a",
        )
        .bind(logical_owner_id)
        .bind(ticket_sha256.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_unavailable)?
        .ok_or(AttachmentPreviewPersistenceErrorV1::NotFound)?;
        if row
            .try_get::<Option<i64>, _>("used_at_unix_seconds")
            .map_err(invalid_row)?
            .is_some()
        {
            return Err(AttachmentPreviewPersistenceErrorV1::TicketUsed);
        }
        let expires_at: i64 = row
            .try_get("expires_at_unix_seconds")
            .map_err(invalid_row)?;
        if now_unix_seconds > expires_at {
            return Err(AttachmentPreviewPersistenceErrorV1::TicketExpired);
        }
        let exact_actor =
            id32(row.try_get("device_actor_sha256").map_err(invalid_row)?)? == device_actor_sha256;
        let exact_fence = u64_from_row(&row, "runtime_generation")? == runtime_generation
            && u64_from_row(&row, "grant_epoch")? == grant_epoch
            && u64_from_row(&row, "current_runtime_generation")? == runtime_generation
            && u64_from_row(&row, "current_grant_epoch")? == grant_epoch;
        let exact_artifact = id16(row.try_get("derived_reference_id").map_err(invalid_row)?)?
            == id16(row.try_get("current_reference_id").map_err(invalid_row)?)?
            && id32(row.try_get("derived_receipt_sha256").map_err(invalid_row)?)?
                == id32(row.try_get("current_receipt_sha256").map_err(invalid_row)?)?
            && id32(
                row.try_get("renderer_identity_sha256")
                    .map_err(invalid_row)?,
            )? == id32(
                row.try_get("current_renderer_sha256")
                    .map_err(invalid_row)?,
            )?;
        let exact_revision = row.try_get::<i16, _>("state").map_err(invalid_row)?
            == AttachmentPreviewStateV1::Ready as i16
            && u64_from_row(&row, "state_revision")?
                == u64_from_row(&row, "current_state_revision")?;
        if !exact_actor || !exact_fence || !exact_artifact || !exact_revision {
            return Err(AttachmentPreviewPersistenceErrorV1::StaleFence);
        }
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_preview_read_tickets SET used_at_unix_seconds=$1 WHERE logical_owner_id=$2 AND ticket_sha256=$3 AND used_at_unix_seconds IS NULL AND expires_at_unix_seconds>=$1",
        )
        .bind(now_unix_seconds)
        .bind(logical_owner_id)
        .bind(ticket_sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?
        .rows_affected();
        if changed != 1 {
            return Err(AttachmentPreviewPersistenceErrorV1::TicketUsed);
        }
        let redeemed = RedeemedAttachmentPreviewTicketV1 {
            run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
            derived_reference_id: id16(row.try_get("derived_reference_id").map_err(invalid_row)?)?,
            derived_receipt_sha256: id32(
                row.try_get("derived_receipt_sha256").map_err(invalid_row)?,
            )?,
            renderer_identity_sha256: id32(
                row.try_get("renderer_identity_sha256")
                    .map_err(invalid_row)?,
            )?,
            content_type: content_type_from_code(
                row.try_get("content_type").map_err(invalid_row)?,
            )?,
            preview_size_bytes: u64_from_row(&row, "preview_size_bytes")?,
        };
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(redeemed)
    }
}

fn valid_ticket_request(logical_owner_id: &str, request: &IssueAttachmentPreviewTicketV1) -> bool {
    valid_owner(logical_owner_id)
        && valid_sha256(&request.ticket_sha256)
        && valid_sha256(&request.device_actor_sha256)
        && valid_id16(&request.run_id)
        && request.runtime_generation > 0
        && request.grant_epoch > 0
        && request.now_unix_seconds > 0
}

fn u64_from_row(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<u64, AttachmentPreviewPersistenceErrorV1> {
    u64::try_from(row.try_get::<i64, _>(name).map_err(invalid_row)?).map_err(invalid_row)
}

fn invalid_input<T>(_: T) -> AttachmentPreviewPersistenceErrorV1 {
    AttachmentPreviewPersistenceErrorV1::InvalidInput
}
