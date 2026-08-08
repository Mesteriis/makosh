use makosh_call_transcription_api::READ_TICKET_TTL_SECONDS_V1;
use makosh_call_transcription_core::CallTranscriptionStateV1;
use sqlx::Row;

use crate::{
    CallTranscriptionPersistenceErrorV1, CallTranscriptionPersistenceV1,
    IssueCallTranscriptTicketV1, IssuedCallTranscriptTicketV1, RedeemedCallTranscriptTicketV1,
    model::{valid_id16, valid_owner, valid_sha256},
    repository::{id16, id32, invalid_input, row_error, signed, state_code, storage_error},
};

impl CallTranscriptionPersistenceV1 {
    pub async fn issue_read_ticket(
        &self,
        logical_owner_id: &str,
        request: IssueCallTranscriptTicketV1,
    ) -> Result<IssuedCallTranscriptTicketV1, CallTranscriptionPersistenceErrorV1> {
        if !valid_ticket_request(logical_owner_id, &request) {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        let expires_at = request
            .now_unix_seconds
            .checked_add(READ_TICKET_TTL_SECONDS_V1)
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let row = sqlx::query(
            "SELECT state_revision,artifact_reference_id,artifact_receipt_sha256,
             artifact_transcript_size_bytes,artifact_runtime_generation,artifact_grant_epoch
             FROM makosh_data.call_transcription_runs WHERE logical_owner_id=$1
               AND run_id=$2 AND state=$3 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(request.run_id.as_slice())
        .bind(state_code(CallTranscriptionStateV1::Ready))
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(CallTranscriptionPersistenceErrorV1::NotFound)?;
        let runtime_generation = unsigned_row(&row, "artifact_runtime_generation")?;
        let grant_epoch = unsigned_row(&row, "artifact_grant_epoch")?;
        if runtime_generation != request.runtime_generation || grant_epoch != request.grant_epoch {
            return Err(CallTranscriptionPersistenceErrorV1::StaleFence);
        }
        let state_revision = unsigned_row(&row, "state_revision")?;
        let transcript_size_bytes = unsigned_row(&row, "artifact_transcript_size_bytes")?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.call_transcription_read_tickets (
               logical_owner_id,ticket_sha256,device_actor_sha256,client_session_sha256,
               run_id,state_revision,artifact_reference_id,artifact_receipt_sha256,
               transcript_size_bytes,runtime_generation,grant_epoch,expires_at_unix_seconds,
               used_at_unix_seconds,created_at_unix_seconds
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,NULL,$13)
             ON CONFLICT (logical_owner_id,ticket_sha256) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(request.ticket_sha256.as_slice())
        .bind(request.device_actor_sha256.as_slice())
        .bind(request.client_session_sha256.as_slice())
        .bind(request.run_id.as_slice())
        .bind(signed(state_revision)?)
        .bind(
            row.try_get::<Vec<u8>, _>("artifact_reference_id")
                .map_err(row_error)?,
        )
        .bind(
            row.try_get::<Vec<u8>, _>("artifact_receipt_sha256")
                .map_err(row_error)?,
        )
        .bind(signed(transcript_size_bytes)?)
        .bind(signed(runtime_generation)?)
        .bind(signed(grant_epoch)?)
        .bind(expires_at)
        .bind(request.now_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if inserted != 1 {
            return Err(CallTranscriptionPersistenceErrorV1::RequestConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(IssuedCallTranscriptTicketV1 {
            run_id: request.run_id,
            expires_at_unix_seconds: expires_at,
            transcript_size_bytes,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn redeem_read_ticket(
        &self,
        logical_owner_id: &str,
        ticket_sha256: [u8; 32],
        device_actor_sha256: [u8; 32],
        client_session_sha256: [u8; 32],
        runtime_generation: u64,
        grant_epoch: u64,
        now_unix_seconds: i64,
    ) -> Result<RedeemedCallTranscriptTicketV1, CallTranscriptionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_sha256(&ticket_sha256)
            || !valid_sha256(&device_actor_sha256)
            || !valid_sha256(&client_session_sha256)
            || runtime_generation == 0
            || grant_epoch == 0
            || now_unix_seconds <= 0
        {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let row = sqlx::query(
            "SELECT t.device_actor_sha256,t.client_session_sha256,t.run_id,t.state_revision,
             t.artifact_reference_id,t.artifact_receipt_sha256,t.transcript_size_bytes,
             t.runtime_generation,t.grant_epoch,t.expires_at_unix_seconds,t.used_at_unix_seconds,
             r.state,r.state_revision AS current_state_revision,
             r.artifact_reference_id AS current_reference_id,
             r.artifact_receipt_sha256 AS current_receipt_sha256,
             r.artifact_runtime_generation AS current_runtime_generation,
             r.artifact_grant_epoch AS current_grant_epoch
             FROM makosh_data.call_transcription_read_tickets t
             JOIN makosh_data.call_transcription_runs r
               ON r.logical_owner_id=t.logical_owner_id AND r.run_id=t.run_id
             WHERE t.logical_owner_id=$1 AND t.ticket_sha256=$2 FOR UPDATE OF t,r",
        )
        .bind(logical_owner_id)
        .bind(ticket_sha256.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(CallTranscriptionPersistenceErrorV1::NotFound)?;
        if row
            .try_get::<Option<i64>, _>("used_at_unix_seconds")
            .map_err(row_error)?
            .is_some()
        {
            return Err(CallTranscriptionPersistenceErrorV1::TicketUsed);
        }
        let expires_at: i64 = row.try_get("expires_at_unix_seconds").map_err(row_error)?;
        if now_unix_seconds > expires_at {
            return Err(CallTranscriptionPersistenceErrorV1::TicketExpired);
        }
        let exact_actor =
            id32(row.try_get("device_actor_sha256").map_err(row_error)?)? == device_actor_sha256;
        let exact_session = id32(row.try_get("client_session_sha256").map_err(row_error)?)?
            == client_session_sha256;
        let exact_fence = unsigned_row(&row, "runtime_generation")? == runtime_generation
            && unsigned_row(&row, "grant_epoch")? == grant_epoch
            && unsigned_row(&row, "current_runtime_generation")? == runtime_generation
            && unsigned_row(&row, "current_grant_epoch")? == grant_epoch;
        let exact_artifact = id16(row.try_get("artifact_reference_id").map_err(row_error)?)?
            == id16(row.try_get("current_reference_id").map_err(row_error)?)?
            && id32(row.try_get("artifact_receipt_sha256").map_err(row_error)?)?
                == id32(row.try_get("current_receipt_sha256").map_err(row_error)?)?;
        let exact_revision = row.try_get::<i16, _>("state").map_err(row_error)?
            == state_code(CallTranscriptionStateV1::Ready)
            && unsigned_row(&row, "state_revision")?
                == unsigned_row(&row, "current_state_revision")?;
        if !exact_actor || !exact_session || !exact_fence || !exact_artifact || !exact_revision {
            return Err(CallTranscriptionPersistenceErrorV1::StaleFence);
        }
        let changed = sqlx::query(
            "UPDATE makosh_data.call_transcription_read_tickets SET used_at_unix_seconds=$1
             WHERE logical_owner_id=$2 AND ticket_sha256=$3
               AND used_at_unix_seconds IS NULL AND expires_at_unix_seconds>=$1",
        )
        .bind(now_unix_seconds)
        .bind(logical_owner_id)
        .bind(ticket_sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if changed != 1 {
            return Err(CallTranscriptionPersistenceErrorV1::TicketUsed);
        }
        let redeemed = RedeemedCallTranscriptTicketV1 {
            run_id: id16(row.try_get("run_id").map_err(row_error)?)?,
            artifact_reference_id: id16(row.try_get("artifact_reference_id").map_err(row_error)?)?,
            artifact_receipt_sha256: id32(
                row.try_get("artifact_receipt_sha256").map_err(row_error)?,
            )?,
            transcript_size_bytes: unsigned_row(&row, "transcript_size_bytes")?,
        };
        transaction.commit().await.map_err(storage_error)?;
        Ok(redeemed)
    }
}

fn valid_ticket_request(logical_owner_id: &str, request: &IssueCallTranscriptTicketV1) -> bool {
    valid_owner(logical_owner_id)
        && valid_sha256(&request.ticket_sha256)
        && valid_sha256(&request.device_actor_sha256)
        && valid_sha256(&request.client_session_sha256)
        && valid_id16(&request.run_id)
        && request.runtime_generation > 0
        && request.grant_epoch > 0
        && request.now_unix_seconds > 0
}

fn unsigned_row(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<u64, CallTranscriptionPersistenceErrorV1> {
    u64::try_from(row.try_get::<i64, _>(name).map_err(row_error)?).map_err(invalid_input)
}
