use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelemostAccountRecordV1 {
    pub logical_owner_id: String,
    pub account_cursor_sha256: [u8; 32],
    pub mapping_revision: u64,
    pub lifecycle_state: u16,
    pub updated_at_unix_millis: i64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelemostObservationRecordV1 {
    pub logical_owner_id: String,
    pub message_id: [u8; 16],
    pub exact_envelope_bytes: Vec<u8>,
    pub account_cursor_sha256: [u8; 32],
    pub source_revision: u64,
    pub call_evidence_message_id: [u8; 16],
    pub call_evidence_bytes: Vec<u8>,
    pub completed_at_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemostReplayOutcomeV1 {
    Applied,
    Replayed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemostPersistenceErrorV1 {
    InvalidInput,
    Conflict,
    StorageUnavailable,
}
#[derive(Clone)]
pub struct TelemostPersistenceV1 {
    pool: PgPool,
}
impl TelemostPersistenceV1 {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        host: &str,
        port: u32,
        password: &str,
    ) -> Result<Self, TelemostPersistenceErrorV1> {
        if host.is_empty() || port == 0 || database_id != binding.identity().database_id() {
            return Err(TelemostPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(u16::try_from(port).map_err(|_| TelemostPersistenceErrorV1::InvalidInput)?)
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
    pub async fn upsert_account(
        &self,
        value: &TelemostAccountRecordV1,
    ) -> Result<(), TelemostPersistenceErrorV1> {
        validate_account(value)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        owner(&mut tx, &value.logical_owner_id).await?;
        let revision = i64::try_from(value.mapping_revision)
            .map_err(|_| TelemostPersistenceErrorV1::InvalidInput)?;
        let state = i16::try_from(value.lifecycle_state)
            .map_err(|_| TelemostPersistenceErrorV1::InvalidInput)?;
        let changed=sqlx::query("INSERT INTO makosh_data.telemost_accounts(logical_owner_id,account_cursor_sha256,mapping_revision,lifecycle_state,updated_at_unix_millis) VALUES($1,$2,$3,$4,$5) ON CONFLICT(logical_owner_id,account_cursor_sha256) DO UPDATE SET mapping_revision=EXCLUDED.mapping_revision,lifecycle_state=EXCLUDED.lifecycle_state,updated_at_unix_millis=EXCLUDED.updated_at_unix_millis WHERE makosh_data.telemost_accounts.mapping_revision<EXCLUDED.mapping_revision").bind(&value.logical_owner_id).bind(value.account_cursor_sha256.as_slice()).bind(revision).bind(state).bind(value.updated_at_unix_millis).execute(&mut*tx).await.map_err(storage)?.rows_affected();
        if changed != 1 {
            return Err(TelemostPersistenceErrorV1::Conflict);
        }
        tx.commit().await.map_err(storage)
    }
    pub async fn record_observation_once(
        &self,
        value: &TelemostObservationRecordV1,
    ) -> Result<TelemostReplayOutcomeV1, TelemostPersistenceErrorV1> {
        validate_observation(value)?;
        let request_sha: [u8; 32] = Sha256::digest(&value.exact_envelope_bytes).into();
        let output_sha: [u8; 32] = Sha256::digest(&value.call_evidence_bytes).into();
        let mut tx = self.pool.begin().await.map_err(storage)?;
        owner(&mut tx, &value.logical_owner_id).await?;
        let existing:Option<(Vec<u8>,Vec<u8>)>=sqlx::query_as("SELECT envelope_sha256,envelope_bytes FROM makosh_data.telemost_observation_inbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE").bind(&value.logical_owner_id).bind(value.message_id.as_slice()).fetch_optional(&mut*tx).await.map_err(storage)?;
        if let Some((sha, bytes)) = existing {
            tx.rollback().await.map_err(storage)?;
            return if sha == request_sha && bytes == value.exact_envelope_bytes {
                Ok(TelemostReplayOutcomeV1::Replayed)
            } else {
                Err(TelemostPersistenceErrorV1::Conflict)
            };
        }
        let account:Option<i64>=sqlx::query_scalar("SELECT mapping_revision FROM makosh_data.telemost_accounts WHERE logical_owner_id=$1 AND account_cursor_sha256=$2 FOR UPDATE").bind(&value.logical_owner_id).bind(value.account_cursor_sha256.as_slice()).fetch_optional(&mut*tx).await.map_err(storage)?;
        if account.is_none() {
            return Err(TelemostPersistenceErrorV1::Conflict);
        }
        sqlx::query("INSERT INTO makosh_data.telemost_observation_inbox(logical_owner_id,message_id,envelope_sha256,envelope_bytes,account_cursor_sha256,source_revision,completed_at_unix_millis) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(&value.logical_owner_id).bind(value.message_id.as_slice()).bind(request_sha.as_slice()).bind(&value.exact_envelope_bytes).bind(value.account_cursor_sha256.as_slice()).bind(i64::try_from(value.source_revision).map_err(|_|TelemostPersistenceErrorV1::InvalidInput)?).bind(value.completed_at_unix_millis).execute(&mut*tx).await.map_err(storage)?;
        sqlx::query("INSERT INTO makosh_data.telemost_call_evidence_outbox(logical_owner_id,message_id,envelope_sha256,envelope_bytes) VALUES($1,$2,$3,$4)").bind(&value.logical_owner_id).bind(value.call_evidence_message_id.as_slice()).bind(output_sha.as_slice()).bind(&value.call_evidence_bytes).execute(&mut*tx).await.map_err(storage)?;
        tx.commit().await.map_err(storage)?;
        Ok(TelemostReplayOutcomeV1::Applied)
    }
    pub async fn counts(
        &self,
        logical_owner_id: &str,
    ) -> Result<(i64, i64, i64), TelemostPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        owner(&mut tx, logical_owner_id).await?;
        let a = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.telemost_accounts WHERE logical_owner_id=$1",
        )
        .bind(logical_owner_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(storage)?;
        let i = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.telemost_observation_inbox WHERE logical_owner_id=$1",
        )
        .bind(logical_owner_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(storage)?;
        let o=sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.telemost_call_evidence_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL").bind(logical_owner_id).fetch_one(&mut*tx).await.map_err(storage)?;
        tx.commit().await.map_err(storage)?;
        Ok((a, i, o))
    }
}
fn validate_account(v: &TelemostAccountRecordV1) -> Result<(), TelemostPersistenceErrorV1> {
    validate_owner(&v.logical_owner_id)?;
    if v.account_cursor_sha256.iter().all(|b| *b == 0)
        || v.mapping_revision == 0
        || !(1..=3).contains(&v.lifecycle_state)
        || v.updated_at_unix_millis <= 0
    {
        Err(TelemostPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}
fn validate_observation(v: &TelemostObservationRecordV1) -> Result<(), TelemostPersistenceErrorV1> {
    validate_owner(&v.logical_owner_id)?;
    if v.message_id.iter().all(|b| *b == 0)
        || v.call_evidence_message_id.iter().all(|b| *b == 0)
        || v.account_cursor_sha256.iter().all(|b| *b == 0)
        || v.source_revision == 0
        || v.exact_envelope_bytes.is_empty()
        || v.call_evidence_bytes.is_empty()
        || v.completed_at_unix_millis <= 0
    {
        Err(TelemostPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}
fn validate_owner(v: &str) -> Result<(), TelemostPersistenceErrorV1> {
    if v.is_empty() || v.len() > 128 || !v.is_ascii() {
        Err(TelemostPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}
async fn owner(
    tx: &mut Transaction<'_, Postgres>,
    v: &str,
) -> Result<(), TelemostPersistenceErrorV1> {
    validate_owner(v)?;
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(v)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    Ok(())
}
fn storage(_: sqlx::Error) -> TelemostPersistenceErrorV1 {
    TelemostPersistenceErrorV1::StorageUnavailable
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_bytes_are_not_part_of_account_record() {
        let value = TelemostAccountRecordV1 {
            logical_owner_id: "owner-1".into(),
            account_cursor_sha256: [1; 32],
            mapping_revision: 1,
            lifecycle_state: 1,
            updated_at_unix_millis: 1,
        };
        assert_eq!(validate_account(&value), Ok(()));
    }
}
