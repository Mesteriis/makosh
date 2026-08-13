use makosh_risk_core::{RiskProjectionEntryV1, validate_risk_projection_entry_v1};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskEnvelopeRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyRiskEntryV1 {
    pub input: RiskEnvelopeRecordV1,
    pub projection_generation: u64,
    pub entry: RiskProjectionEntryV1,
    pub completed_at_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskReplayOutcomeV1 {
    Applied,
    Replayed,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskEntryRecordV1 {
    pub event_id: [u8; 16],
    pub source_owner: String,
    pub entity_kind: String,
    pub entity_id: [u8; 16],
    pub source_revision: u64,
    pub reason_code: String,
    pub severity: u32,
    pub occurred_at_unix_millis: i64,
    pub expires_at_unix_millis: i64,
    pub cleared: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskCursorRecordV1 {
    pub occurred_at_unix_millis: i64,
    pub source_owner: String,
    pub entity_kind: String,
    pub entity_id: [u8; 16],
    pub source_revision: u64,
    pub event_id: [u8; 16],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RiskStatusRecordV1 {
    pub active_generation: u64,
    pub risk_entries: u64,
    pub source_events: u64,
    pub rebuilt_at_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskPersistenceErrorV1 {
    InvalidInput,
    Conflict,
    RevisionConflict,
    NotFound,
    StorageUnavailable,
}
#[derive(Clone)]
pub struct RiskPersistenceV1 {
    pool: PgPool,
}

impl RiskPersistenceV1 {
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
    ) -> Result<Self, RiskPersistenceErrorV1> {
        if host.is_empty()
            || port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(RiskPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(u16::try_from(port).map_err(|_| RiskPersistenceErrorV1::StorageUnavailable)?)
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
    pub async fn verify_storage_ready(&self) -> Result<(), RiskPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage)
    }
    pub async fn ensure_live_generation(
        &self,
        owner: &str,
        now: i64,
    ) -> Result<u64, RiskPersistenceErrorV1> {
        validate_owner(owner)?;
        if now <= 0 {
            return Err(RiskPersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        sqlx::query("INSERT INTO makosh_data.risk_projection_control(logical_owner_id,active_projection_generation,next_projection_generation,rebuilt_at_unix_millis) VALUES($1,1,2,$2) ON CONFLICT DO NOTHING").bind(owner).bind(now).execute(&mut*tx).await.map_err(storage)?;
        sqlx::query("INSERT INTO makosh_data.risk_projection_rebuilds(logical_owner_id,projection_generation,state,expected_source_count,applied_source_count,started_at_unix_millis,completed_at_unix_millis) VALUES($1,1,2,0,0,$2,$2) ON CONFLICT DO NOTHING").bind(owner).bind(now).execute(&mut*tx).await.map_err(storage)?;
        let value:i64=sqlx::query_scalar("SELECT active_projection_generation FROM makosh_data.risk_projection_control WHERE logical_owner_id=$1").bind(owner).fetch_one(&mut*tx).await.map_err(storage)?;
        tx.commit().await.map_err(storage)?;
        u64::try_from(value).map_err(|_| RiskPersistenceErrorV1::StorageUnavailable)
    }
    pub async fn start_rebuild(
        &self,
        owner: &str,
        expected: u64,
        started: i64,
    ) -> Result<u64, RiskPersistenceErrorV1> {
        validate_owner(owner)?;
        if started <= 0 {
            return Err(RiskPersistenceErrorV1::InvalidInput);
        }
        let expected = i64::try_from(expected).map_err(|_| RiskPersistenceErrorV1::InvalidInput)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let row=sqlx::query("SELECT next_projection_generation FROM makosh_data.risk_projection_control WHERE logical_owner_id=$1 FOR UPDATE").bind(owner).fetch_optional(&mut*tx).await.map_err(storage)?;
        let generation = if let Some(row) = row {
            let next: i64 = row.try_get("next_projection_generation").map_err(storage)?;
            sqlx::query("UPDATE makosh_data.risk_projection_control SET next_projection_generation=$2 WHERE logical_owner_id=$1").bind(owner).bind(next.checked_add(1).ok_or(RiskPersistenceErrorV1::InvalidInput)?).execute(&mut*tx).await.map_err(storage)?;
            next
        } else {
            sqlx::query("INSERT INTO makosh_data.risk_projection_control(logical_owner_id,active_projection_generation,next_projection_generation,rebuilt_at_unix_millis) VALUES($1,0,2,$2)").bind(owner).bind(started).execute(&mut*tx).await.map_err(storage)?;
            1
        };
        sqlx::query("INSERT INTO makosh_data.risk_projection_rebuilds(logical_owner_id,projection_generation,state,expected_source_count,applied_source_count,started_at_unix_millis) VALUES($1,$2,1,$3,0,$4)").bind(owner).bind(generation).bind(expected).bind(started).execute(&mut*tx).await.map_err(storage)?;
        tx.commit().await.map_err(storage)?;
        u64::try_from(generation).map_err(|_| RiskPersistenceErrorV1::InvalidInput)
    }
    pub async fn apply_entry_once(
        &self,
        input: &ApplyRiskEntryV1,
    ) -> Result<RiskReplayOutcomeV1, RiskPersistenceErrorV1> {
        validate_input(input)?;
        let owner = &input.entry.logical_owner_id;
        let generation = i64::try_from(input.projection_generation)
            .map_err(|_| RiskPersistenceErrorV1::InvalidInput)?;
        let revision = i64::try_from(input.entry.source_revision)
            .map_err(|_| RiskPersistenceErrorV1::InvalidInput)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        if let Some(row)=sqlx::query("SELECT envelope_sha256,envelope_bytes FROM makosh_data.risk_projection_inbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE").bind(owner).bind(input.input.message_id.as_slice()).fetch_optional(&mut*tx).await.map_err(storage)?{let sha:Vec<u8>=row.try_get("envelope_sha256").map_err(storage)?;let bytes:Vec<u8>=row.try_get("envelope_bytes").map_err(storage)?;tx.rollback().await.map_err(storage)?;return if sha==input.input.envelope_sha256&&bytes==input.input.envelope_bytes{Ok(RiskReplayOutcomeV1::Replayed)}else{Err(RiskPersistenceErrorV1::Conflict)}}
        let state:Option<i16>=sqlx::query_scalar("SELECT state FROM makosh_data.risk_projection_rebuilds WHERE logical_owner_id=$1 AND projection_generation=$2 FOR UPDATE").bind(owner).bind(generation).fetch_optional(&mut*tx).await.map_err(storage)?;
        let active:Option<i64>=sqlx::query_scalar("SELECT active_projection_generation FROM makosh_data.risk_projection_control WHERE logical_owner_id=$1 FOR UPDATE").bind(owner).fetch_optional(&mut*tx).await.map_err(storage)?;
        if state != Some(1) && !(state == Some(2) && active == Some(generation)) {
            return Err(RiskPersistenceErrorV1::Conflict);
        }
        let max_revision:Option<i64>=sqlx::query_scalar("SELECT MAX(source_revision) FROM makosh_data.risk_projection_entries WHERE logical_owner_id=$1 AND projection_generation=$2 AND source_owner=$3 AND entity_kind=$4 AND entity_id=$5").bind(owner).bind(generation).bind(&input.entry.source_owner).bind(&input.entry.entity_kind).bind(input.entry.entity_id.as_slice()).fetch_one(&mut*tx).await.map_err(storage)?;
        if max_revision.is_some_and(|value| value >= revision) {
            return Err(RiskPersistenceErrorV1::RevisionConflict);
        }
        sqlx::query("INSERT INTO makosh_data.risk_projection_entries(logical_owner_id,projection_generation,event_id,source_owner,entity_kind,entity_id,source_revision,reason_code,severity,occurred_at_unix_millis,expires_at_unix_millis,deleted_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)").bind(owner).bind(generation).bind(input.entry.event_id.as_slice()).bind(&input.entry.source_owner).bind(&input.entry.entity_kind).bind(input.entry.entity_id.as_slice()).bind(revision).bind(&input.entry.reason_code).bind(i16::try_from(input.entry.severity).map_err(|_| RiskPersistenceErrorV1::InvalidInput)?).bind(input.entry.occurred_at_unix_millis).bind(input.entry.expires_at_unix_millis).bind(input.entry.cleared.then_some(input.completed_at_unix_millis)).execute(&mut*tx).await.map_err(storage)?;
        sqlx::query("INSERT INTO makosh_data.risk_projection_inbox(logical_owner_id,message_id,envelope_sha256,envelope_bytes,source_owner,source_revision,completed_at_unix_millis) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(owner).bind(input.input.message_id.as_slice()).bind(input.input.envelope_sha256.as_slice()).bind(&input.input.envelope_bytes).bind(&input.entry.source_owner).bind(revision).bind(input.completed_at_unix_millis).execute(&mut*tx).await.map_err(storage)?;
        if state == Some(1) {
            sqlx::query("UPDATE makosh_data.risk_projection_rebuilds SET applied_source_count=applied_source_count+1 WHERE logical_owner_id=$1 AND projection_generation=$2").bind(owner).bind(generation).execute(&mut*tx).await.map_err(storage)?;
        }
        tx.commit().await.map_err(storage)?;
        Ok(RiskReplayOutcomeV1::Applied)
    }
    pub async fn complete_rebuild(
        &self,
        owner: &str,
        generation: u64,
        completed: i64,
    ) -> Result<(), RiskPersistenceErrorV1> {
        validate_owner(owner)?;
        let generation =
            i64::try_from(generation).map_err(|_| RiskPersistenceErrorV1::InvalidInput)?;
        if generation <= 0 || completed <= 0 {
            return Err(RiskPersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let row=sqlx::query("SELECT state,expected_source_count,applied_source_count,started_at_unix_millis FROM makosh_data.risk_projection_rebuilds WHERE logical_owner_id=$1 AND projection_generation=$2 FOR UPDATE").bind(owner).bind(generation).fetch_optional(&mut*tx).await.map_err(storage)?.ok_or(RiskPersistenceErrorV1::NotFound)?;
        let state: i16 = row.try_get("state").map_err(storage)?;
        let expected: i64 = row.try_get("expected_source_count").map_err(storage)?;
        let applied: i64 = row.try_get("applied_source_count").map_err(storage)?;
        let started: i64 = row.try_get("started_at_unix_millis").map_err(storage)?;
        if state != 1 || expected != applied || completed < started {
            return Err(RiskPersistenceErrorV1::Conflict);
        }
        sqlx::query("UPDATE makosh_data.risk_projection_rebuilds SET state=2,completed_at_unix_millis=$3 WHERE logical_owner_id=$1 AND projection_generation=$2").bind(owner).bind(generation).bind(completed).execute(&mut*tx).await.map_err(storage)?;
        let affected=sqlx::query("UPDATE makosh_data.risk_projection_control SET active_projection_generation=$2,rebuilt_at_unix_millis=$3 WHERE logical_owner_id=$1 AND active_projection_generation<$2").bind(owner).bind(generation).bind(completed).execute(&mut*tx).await.map_err(storage)?.rows_affected();
        if affected != 1 {
            return Err(RiskPersistenceErrorV1::Conflict);
        }
        tx.commit().await.map_err(storage)
    }
    pub async fn list_active(
        &self,
        owner: &str,
        after: Option<&RiskCursorRecordV1>,
        limit: u32,
    ) -> Result<Vec<RiskEntryRecordV1>, RiskPersistenceErrorV1> {
        validate_owner(owner)?;
        if !(1..=101).contains(&limit) {
            return Err(RiskPersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let (after_time, after_owner, after_kind, after_id, after_revision, after_event) = after
            .map_or(
                (
                    i64::MAX,
                    String::new(),
                    String::new(),
                    Vec::new(),
                    0_i64,
                    Vec::new(),
                ),
                |value| {
                    (
                        value.occurred_at_unix_millis,
                        value.source_owner.clone(),
                        value.entity_kind.clone(),
                        value.entity_id.to_vec(),
                        i64::try_from(value.source_revision).unwrap_or(i64::MAX),
                        value.event_id.to_vec(),
                    )
                },
            );
        let rows=sqlx::query("SELECT e.event_id,e.source_owner,e.entity_kind,e.entity_id,e.source_revision,e.reason_code,e.severity,e.occurred_at_unix_millis,e.expires_at_unix_millis,e.deleted_at FROM makosh_data.risk_projection_control c JOIN makosh_data.risk_projection_entries e ON e.logical_owner_id=c.logical_owner_id AND e.projection_generation=c.active_projection_generation WHERE c.logical_owner_id=$1 AND ($3='' OR (-e.occurred_at_unix_millis,e.source_owner,e.entity_kind,e.entity_id,e.source_revision,e.event_id) > (-$2,$3,$4,$5,$6,$7)) ORDER BY e.occurred_at_unix_millis DESC,e.source_owner,e.entity_kind,e.entity_id,e.source_revision,e.event_id LIMIT $8").bind(owner).bind(after_time).bind(after_owner).bind(after_kind).bind(after_id).bind(after_revision).bind(after_event).bind(i64::from(limit)).fetch_all(&mut*tx).await.map_err(storage)?;
        tx.rollback().await.map_err(storage)?;
        rows.into_iter()
            .map(|row| {
                let event: Vec<u8> = row.try_get("event_id").map_err(storage)?;
                let entity: Vec<u8> = row.try_get("entity_id").map_err(storage)?;
                Ok(RiskEntryRecordV1 {
                    event_id: event
                        .try_into()
                        .map_err(|_| RiskPersistenceErrorV1::StorageUnavailable)?,
                    source_owner: row.try_get("source_owner").map_err(storage)?,
                    entity_kind: row.try_get("entity_kind").map_err(storage)?,
                    entity_id: entity
                        .try_into()
                        .map_err(|_| RiskPersistenceErrorV1::StorageUnavailable)?,
                    source_revision: u64::try_from(
                        row.try_get::<i64, _>("source_revision").map_err(storage)?,
                    )
                    .map_err(|_| RiskPersistenceErrorV1::StorageUnavailable)?,
                    reason_code: row.try_get("reason_code").map_err(storage)?,
                    severity: u32::try_from(row.try_get::<i16, _>("severity").map_err(storage)?)
                        .map_err(|_| RiskPersistenceErrorV1::StorageUnavailable)?,
                    occurred_at_unix_millis: row
                        .try_get("occurred_at_unix_millis")
                        .map_err(storage)?,
                    expires_at_unix_millis: row
                        .try_get("expires_at_unix_millis")
                        .map_err(storage)?,
                    cleared: row
                        .try_get::<Option<i64>, _>("deleted_at")
                        .map_err(storage)?
                        .is_some(),
                })
            })
            .collect()
    }
    pub async fn status(&self, owner: &str) -> Result<RiskStatusRecordV1, RiskPersistenceErrorV1> {
        validate_owner(owner)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let row=sqlx::query("SELECT c.active_projection_generation,c.rebuilt_at_unix_millis,(SELECT COUNT(*) FROM makosh_data.risk_projection_entries e WHERE e.logical_owner_id=c.logical_owner_id AND e.projection_generation=c.active_projection_generation) risk_entries,(SELECT COUNT(*) FROM makosh_data.risk_projection_inbox i WHERE i.logical_owner_id=c.logical_owner_id) source_events FROM makosh_data.risk_projection_control c WHERE c.logical_owner_id=$1").bind(owner).fetch_optional(&mut*tx).await.map_err(storage)?.ok_or(RiskPersistenceErrorV1::NotFound)?;
        tx.rollback().await.map_err(storage)?;
        Ok(RiskStatusRecordV1 {
            active_generation: u64::try_from(
                row.try_get::<i64, _>("active_projection_generation")
                    .map_err(storage)?,
            )
            .map_err(|_| RiskPersistenceErrorV1::StorageUnavailable)?,
            risk_entries: u64::try_from(row.try_get::<i64, _>("risk_entries").map_err(storage)?)
                .map_err(|_| RiskPersistenceErrorV1::StorageUnavailable)?,
            source_events: u64::try_from(row.try_get::<i64, _>("source_events").map_err(storage)?)
                .map_err(|_| RiskPersistenceErrorV1::StorageUnavailable)?,
            rebuilt_at_unix_millis: row.try_get("rebuilt_at_unix_millis").map_err(storage)?,
        })
    }
}
fn validate_input(value: &ApplyRiskEntryV1) -> Result<(), RiskPersistenceErrorV1> {
    validate_risk_projection_entry_v1(&value.entry)
        .map_err(|_| RiskPersistenceErrorV1::InvalidInput)?;
    if value.projection_generation == 0
        || value.completed_at_unix_millis < value.entry.occurred_at_unix_millis
        || value.input.message_id != value.entry.event_id
        || value.input.envelope_bytes.is_empty()
        || value.input.envelope_bytes.len() > 65_536
        || <[u8; 32]>::from(Sha256::digest(&value.input.envelope_bytes))
            != value.input.envelope_sha256
    {
        return Err(RiskPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}
async fn set_owner(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
) -> Result<(), RiskPersistenceErrorV1> {
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(owner)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    Ok(())
}
fn validate_owner(owner: &str) -> Result<(), RiskPersistenceErrorV1> {
    if owner.is_empty()
        || owner.len() > 128
        || !owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Err(RiskPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}
fn storage<T>(_: T) -> RiskPersistenceErrorV1 {
    RiskPersistenceErrorV1::StorageUnavailable
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_hash_and_event_identity_are_required() {
        let bytes = b"risk-event".to_vec();
        let value = ApplyRiskEntryV1 {
            input: RiskEnvelopeRecordV1 {
                message_id: [1; 16],
                envelope_sha256: Sha256::digest(&bytes).into(),
                envelope_bytes: bytes,
            },
            projection_generation: 1,
            entry: RiskProjectionEntryV1 {
                event_id: [1; 16],
                logical_owner_id: "owner-1".into(),
                source_owner: "tasks".into(),
                entity_kind: "task".into(),
                entity_id: [2; 16],
                source_revision: 1,
                reason_code: "open_task".into(),
                severity: 1,
                occurred_at_unix_millis: 1000,
                expires_at_unix_millis: 2000,
                cleared: false,
            },
            completed_at_unix_millis: 1001,
        };
        assert_eq!(validate_input(&value), Ok(()));
        let mut changed = value;
        changed.input.message_id = [3; 16];
        assert_eq!(
            validate_input(&changed),
            Err(RiskPersistenceErrorV1::InvalidInput)
        );
    }
}
