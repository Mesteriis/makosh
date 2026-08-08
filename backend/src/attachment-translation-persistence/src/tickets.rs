use makosh_attachment_translation_api::ATTACHMENT_TRANSLATION_READ_TICKET_TTL_SECONDS_V1;
use makosh_attachment_translation_core::AttachmentTranslationStateV1;
use sqlx::Row;

use crate::{
    AttachmentTranslationPersistenceErrorV1, AttachmentTranslationPersistenceV1,
    IssueAttachmentTranslationTicketV1, IssuedAttachmentTranslationTicketV1,
    RedeemedAttachmentTranslationTicketV1,
    model::{nonzero, valid_identity},
};

impl AttachmentTranslationPersistenceV1 {
    pub async fn issue_read_ticket(
        &self,
        logical_owner_id: &str,
        request: IssueAttachmentTranslationTicketV1,
    ) -> Result<IssuedAttachmentTranslationTicketV1, AttachmentTranslationPersistenceErrorV1> {
        if !valid_ticket_request(logical_owner_id, &request) {
            return Err(AttachmentTranslationPersistenceErrorV1::InvalidInput);
        }
        let expires_at_unix_seconds = request
            .now_unix_seconds
            .checked_add(ATTACHMENT_TRANSLATION_READ_TICKET_TTL_SECONDS_V1)
            .ok_or(AttachmentTranslationPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AttachmentTranslationPersistenceErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "SELECT state_revision,artifact_id,artifact_translated_sha256,\
                    artifact_translated_size_bytes,artifact_runtime_generation,artifact_grant_epoch \
             FROM makosh_data.attachment_translation_runs \
             WHERE logical_owner_id=$1 AND run_id=$2 AND state=$3 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(request.run_id.as_slice())
        .bind(state_code(AttachmentTranslationStateV1::Ready))
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| AttachmentTranslationPersistenceErrorV1::StorageUnavailable)?
        .ok_or(AttachmentTranslationPersistenceErrorV1::NotFound)?;
        let runtime_generation = positive_u64(&row, "artifact_runtime_generation")?;
        let grant_epoch = positive_u64(&row, "artifact_grant_epoch")?;
        if runtime_generation != request.runtime_generation || grant_epoch != request.grant_epoch {
            return Err(AttachmentTranslationPersistenceErrorV1::StaleFence);
        }
        let state_revision = positive_u64(&row, "state_revision")?;
        let reference_id = id16(row.try_get("artifact_id").map_err(invalid_row)?)?;
        let receipt_sha256 = id32(
            row.try_get("artifact_translated_sha256")
                .map_err(invalid_row)?,
        )?;
        let translated_size_bytes = positive_u64(&row, "artifact_translated_size_bytes")?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_translation_read_tickets (\
               logical_owner_id,ticket_sha256,device_actor_sha256,run_id,state_revision,\
               artifact_reference_id,artifact_receipt_sha256,translated_size_bytes,\
               runtime_generation,grant_epoch,expires_at_unix_seconds,used_at_unix_seconds,\
               created_at_unix_seconds\
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,NULL,$12)\
             ON CONFLICT (logical_owner_id,ticket_sha256) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(request.ticket_sha256.as_slice())
        .bind(request.device_actor_sha256.as_slice())
        .bind(request.run_id.as_slice())
        .bind(signed(state_revision)?)
        .bind(reference_id.as_slice())
        .bind(receipt_sha256.as_slice())
        .bind(signed(translated_size_bytes)?)
        .bind(signed(runtime_generation)?)
        .bind(signed(grant_epoch)?)
        .bind(expires_at_unix_seconds)
        .bind(request.now_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AttachmentTranslationPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if inserted != 1 {
            return Err(AttachmentTranslationPersistenceErrorV1::RequestConflict);
        }
        transaction
            .commit()
            .await
            .map_err(|_| AttachmentTranslationPersistenceErrorV1::StorageUnavailable)?;
        Ok(IssuedAttachmentTranslationTicketV1 {
            run_id: request.run_id,
            expires_at_unix_seconds,
            translated_size_bytes,
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
    ) -> Result<RedeemedAttachmentTranslationTicketV1, AttachmentTranslationPersistenceErrorV1>
    {
        if !valid_identity(logical_owner_id)
            || !nonzero(&ticket_sha256)
            || !nonzero(&device_actor_sha256)
            || runtime_generation == 0
            || grant_epoch == 0
            || now_unix_seconds <= 0
        {
            return Err(AttachmentTranslationPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AttachmentTranslationPersistenceErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "SELECT t.device_actor_sha256,t.run_id,t.state_revision,t.artifact_reference_id,\
                    t.artifact_receipt_sha256,t.translated_size_bytes,t.runtime_generation,\
                    t.grant_epoch,t.expires_at_unix_seconds,t.used_at_unix_seconds,\
                    r.state,r.state_revision AS current_state_revision,\
                    r.artifact_id AS current_reference_id,\
                    r.artifact_translated_sha256 AS current_receipt_sha256,\
                    r.artifact_runtime_generation AS current_runtime_generation,\
                    r.artifact_grant_epoch AS current_grant_epoch \
             FROM makosh_data.attachment_translation_read_tickets t \
             JOIN makosh_data.attachment_translation_runs r \
               ON r.logical_owner_id=t.logical_owner_id AND r.run_id=t.run_id \
             WHERE t.logical_owner_id=$1 AND t.ticket_sha256=$2 FOR UPDATE OF t,r",
        )
        .bind(logical_owner_id)
        .bind(ticket_sha256.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| AttachmentTranslationPersistenceErrorV1::StorageUnavailable)?
        .ok_or(AttachmentTranslationPersistenceErrorV1::NotFound)?;
        if row
            .try_get::<Option<i64>, _>("used_at_unix_seconds")
            .map_err(invalid_row)?
            .is_some()
        {
            return Err(AttachmentTranslationPersistenceErrorV1::TicketUsed);
        }
        let expires_at_unix_seconds: i64 = row
            .try_get("expires_at_unix_seconds")
            .map_err(invalid_row)?;
        if now_unix_seconds > expires_at_unix_seconds {
            return Err(AttachmentTranslationPersistenceErrorV1::TicketExpired);
        }
        let exact_actor =
            id32(row.try_get("device_actor_sha256").map_err(invalid_row)?)? == device_actor_sha256;
        let exact_fence = positive_u64(&row, "runtime_generation")? == runtime_generation
            && positive_u64(&row, "grant_epoch")? == grant_epoch
            && positive_u64(&row, "current_runtime_generation")? == runtime_generation
            && positive_u64(&row, "current_grant_epoch")? == grant_epoch;
        let reference_id = id16(row.try_get("artifact_reference_id").map_err(invalid_row)?)?;
        let receipt_sha256 = id32(
            row.try_get("artifact_receipt_sha256")
                .map_err(invalid_row)?,
        )?;
        let exact_artifact = reference_id
            == id16(row.try_get("current_reference_id").map_err(invalid_row)?)?
            && receipt_sha256 == id32(row.try_get("current_receipt_sha256").map_err(invalid_row)?)?;
        let exact_revision = row.try_get::<i16, _>("state").map_err(invalid_row)?
            == state_code(AttachmentTranslationStateV1::Ready)
            && positive_u64(&row, "state_revision")?
                == positive_u64(&row, "current_state_revision")?;
        if !exact_actor || !exact_fence || !exact_artifact || !exact_revision {
            return Err(AttachmentTranslationPersistenceErrorV1::StaleFence);
        }
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_translation_read_tickets \
             SET used_at_unix_seconds=$1 \
             WHERE logical_owner_id=$2 AND ticket_sha256=$3 \
               AND used_at_unix_seconds IS NULL AND expires_at_unix_seconds >= $1",
        )
        .bind(now_unix_seconds)
        .bind(logical_owner_id)
        .bind(ticket_sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| AttachmentTranslationPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if changed != 1 {
            return Err(AttachmentTranslationPersistenceErrorV1::TicketUsed);
        }
        let redeemed = RedeemedAttachmentTranslationTicketV1 {
            run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
            artifact_reference_id: reference_id,
            artifact_receipt_sha256: receipt_sha256,
            translated_size_bytes: positive_u64(&row, "translated_size_bytes")?,
        };
        transaction
            .commit()
            .await
            .map_err(|_| AttachmentTranslationPersistenceErrorV1::StorageUnavailable)?;
        Ok(redeemed)
    }
}

fn valid_ticket_request(
    logical_owner_id: &str,
    request: &IssueAttachmentTranslationTicketV1,
) -> bool {
    valid_identity(logical_owner_id)
        && nonzero(&request.ticket_sha256)
        && nonzero(&request.device_actor_sha256)
        && nonzero(&request.run_id)
        && request.runtime_generation > 0
        && request.grant_epoch > 0
        && request.now_unix_seconds > 0
}

const fn state_code(value: AttachmentTranslationStateV1) -> i16 {
    match value {
        AttachmentTranslationStateV1::Accepted => 1,
        AttachmentTranslationStateV1::AwaitingSource => 2,
        AttachmentTranslationStateV1::AwaitingInference => 3,
        AttachmentTranslationStateV1::MaterializingResult => 4,
        AttachmentTranslationStateV1::Ready => 5,
        AttachmentTranslationStateV1::Rejected => 6,
    }
}

fn positive_u64(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<u64, AttachmentTranslationPersistenceErrorV1> {
    let value: i64 = row.try_get(column).map_err(invalid_row)?;
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(AttachmentTranslationPersistenceErrorV1::InvalidRow)
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], AttachmentTranslationPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(nonzero)
        .ok_or(AttachmentTranslationPersistenceErrorV1::InvalidRow)
}

fn id32(value: Vec<u8>) -> Result<[u8; 32], AttachmentTranslationPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(nonzero)
        .ok_or(AttachmentTranslationPersistenceErrorV1::InvalidRow)
}

fn signed(value: u64) -> Result<i64, AttachmentTranslationPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidInput)
}

fn invalid_row<T>(_: T) -> AttachmentTranslationPersistenceErrorV1 {
    AttachmentTranslationPersistenceErrorV1::InvalidRow
}
