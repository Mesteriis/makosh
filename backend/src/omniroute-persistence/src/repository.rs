use makosh_omniroute_core::{OmniRouteRequestReceiptV1, validate_request_receipt_v1};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmniRouteReplayOutcomeV1 {
    Accepted,
    Replayed,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OmniRoutePersistedRunV1 {
    pub request_id: [u8; 16],
    pub state: u16,
    pub terminal_result: Option<Vec<u8>>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmniRoutePersistenceErrorV1 {
    InvalidInput,
    Conflict,
    NotFound,
    StorageUnavailable,
}
#[derive(Clone)]
pub struct OmniRoutePersistenceV1 {
    pool: PgPool,
}
impl OmniRoutePersistenceV1 {
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
    ) -> Result<Self, OmniRoutePersistenceErrorV1> {
        if host.is_empty() || port == 0 || database_id != binding.identity().database_id() {
            return Err(OmniRoutePersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(u16::try_from(port).map_err(|_| OmniRoutePersistenceErrorV1::InvalidInput)?)
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
    pub async fn accept_once(
        &self,
        value: &OmniRouteRequestReceiptV1,
        model_receipt_sha256: [u8; 32],
    ) -> Result<OmniRouteReplayOutcomeV1, OmniRoutePersistenceErrorV1> {
        validate_request_receipt_v1(value)
            .map_err(|_| OmniRoutePersistenceErrorV1::InvalidInput)?;
        if model_receipt_sha256.iter().all(|b| *b == 0) {
            return Err(OmniRoutePersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        owner(&mut tx, &value.logical_owner_id).await?;
        let row=sqlx::query("SELECT contract_name,request_sha256,model_receipt_sha256,settings_revision FROM makosh_data.omniroute_runs WHERE logical_owner_id=$1 AND request_id=$2 FOR UPDATE").bind(&value.logical_owner_id).bind(value.request_id.as_slice()).fetch_optional(&mut*tx).await.map_err(storage)?;
        if let Some(row) = row {
            let same: String = row.try_get("contract_name").map_err(storage)?;
            let request: Vec<u8> = row.try_get("request_sha256").map_err(storage)?;
            let model: Vec<u8> = row.try_get("model_receipt_sha256").map_err(storage)?;
            let revision: i64 = row.try_get("settings_revision").map_err(storage)?;
            tx.rollback().await.map_err(storage)?;
            return if same == value.contract_name
                && request == value.request_sha256
                && model == model_receipt_sha256
                && revision
                    == i64::try_from(value.settings_revision)
                        .map_err(|_| OmniRoutePersistenceErrorV1::InvalidInput)?
            {
                Ok(OmniRouteReplayOutcomeV1::Replayed)
            } else {
                Err(OmniRoutePersistenceErrorV1::Conflict)
            };
        }
        sqlx::query("INSERT INTO makosh_data.omniroute_runs(logical_owner_id,request_id,contract_name,request_sha256,model_receipt_sha256,settings_revision,state,accepted_at_unix_millis) VALUES($1,$2,$3,$4,$5,$6,1,$7)").bind(&value.logical_owner_id).bind(value.request_id.as_slice()).bind(&value.contract_name).bind(value.request_sha256.as_slice()).bind(model_receipt_sha256.as_slice()).bind(i64::try_from(value.settings_revision).map_err(|_|OmniRoutePersistenceErrorV1::InvalidInput)?).bind(value.accepted_at_unix_millis).execute(&mut*tx).await.map_err(storage)?;
        tx.commit().await.map_err(storage)?;
        Ok(OmniRouteReplayOutcomeV1::Accepted)
    }
    pub async fn complete(
        &self,
        owner_id: &str,
        request_id: [u8; 16],
        terminal_result: &[u8],
        completed_at: i64,
    ) -> Result<(), OmniRoutePersistenceErrorV1> {
        validate_owner(owner_id)?;
        if request_id.iter().all(|b| *b == 0)
            || terminal_result.is_empty()
            || terminal_result.len() > 262144
            || completed_at <= 0
        {
            return Err(OmniRoutePersistenceErrorV1::InvalidInput);
        }
        let result_sha: [u8; 32] = Sha256::digest(terminal_result).into();
        let mut tx = self.pool.begin().await.map_err(storage)?;
        owner(&mut tx, owner_id).await?;
        let changed=sqlx::query("UPDATE makosh_data.omniroute_runs SET state=3,terminal_result_bytes=$3,terminal_result_sha256=$4,completed_at_unix_millis=$5 WHERE logical_owner_id=$1 AND request_id=$2 AND state=1 AND accepted_at_unix_millis<=$5").bind(owner_id).bind(request_id.as_slice()).bind(terminal_result).bind(result_sha.as_slice()).bind(completed_at).execute(&mut*tx).await.map_err(storage)?.rows_affected();
        if changed != 1 {
            return Err(OmniRoutePersistenceErrorV1::Conflict);
        }
        tx.commit().await.map_err(storage)
    }
}
fn validate_owner(v: &str) -> Result<(), OmniRoutePersistenceErrorV1> {
    if v.is_empty() || v.len() > 128 || !v.is_ascii() {
        Err(OmniRoutePersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}
async fn owner(
    tx: &mut Transaction<'_, Postgres>,
    v: &str,
) -> Result<(), OmniRoutePersistenceErrorV1> {
    validate_owner(v)?;
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(v)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    Ok(())
}
fn storage(_: sqlx::Error) -> OmniRoutePersistenceErrorV1 {
    OmniRoutePersistenceErrorV1::StorageUnavailable
}
