use makosh_consistency_core::{
    ConsistencyEdgeV1, ConsistencyNodeV1, validate_consistency_edge_v1,
    validate_consistency_node_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsistencyEnvelopeRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsistencyMutationV1 {
    UpsertNode {
        node: ConsistencyNodeV1,
        source_revision: u64,
        deleted: bool,
    },
    UpsertEdge(ConsistencyEdgeV1),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyConsistencyMutationV1 {
    pub input: ConsistencyEnvelopeRecordV1,
    pub projection_generation: u64,
    pub logical_owner_id: String,
    pub source_owner: String,
    pub source_revision: u64,
    pub mutation: ConsistencyMutationV1,
    pub completed_at_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsistencyReplayOutcomeV1 {
    Applied,
    Replayed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConsistencyStatusRecordV1 {
    pub active_generation: u64,
    pub nodes: u64,
    pub edges: u64,
    pub source_events: u64,
    pub rebuilt_at_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsistencyPersistenceErrorV1 {
    InvalidInput,
    Conflict,
    RevisionConflict,
    NotFound,
    StorageUnavailable,
}
#[derive(Clone)]
pub struct ConsistencyPersistenceV1 {
    pool: PgPool,
}
impl ConsistencyPersistenceV1 {
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
    ) -> Result<Self, ConsistencyPersistenceErrorV1> {
        if host.is_empty()
            || port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
        {
            return Err(ConsistencyPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(
                u16::try_from(port)
                    .map_err(|_| ConsistencyPersistenceErrorV1::StorageUnavailable)?,
            )
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
    pub async fn verify_storage_ready(&self) -> Result<(), ConsistencyPersistenceErrorV1> {
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
    ) -> Result<u64, ConsistencyPersistenceErrorV1> {
        validate_owner(owner)?;
        if now <= 0 {
            return Err(ConsistencyPersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        sqlx::query("INSERT INTO makosh_data.consistency_projection_control(logical_owner_id,active_projection_generation,next_projection_generation,rebuilt_at_unix_millis) VALUES($1,1,2,$2) ON CONFLICT DO NOTHING").bind(owner).bind(now).execute(&mut*tx).await.map_err(storage)?;
        sqlx::query("INSERT INTO makosh_data.consistency_projection_rebuilds(logical_owner_id,projection_generation,state,expected_source_count,applied_source_count,started_at_unix_millis,completed_at_unix_millis) VALUES($1,1,2,0,0,$2,$2) ON CONFLICT DO NOTHING").bind(owner).bind(now).execute(&mut*tx).await.map_err(storage)?;
        let generation:i64=sqlx::query_scalar("SELECT active_projection_generation FROM makosh_data.consistency_projection_control WHERE logical_owner_id=$1").bind(owner).fetch_one(&mut*tx).await.map_err(storage)?;
        tx.commit().await.map_err(storage)?;
        u64::try_from(generation).map_err(|_| ConsistencyPersistenceErrorV1::StorageUnavailable)
    }
    pub async fn start_rebuild(
        &self,
        owner: &str,
        expected: u64,
        started: i64,
    ) -> Result<u64, ConsistencyPersistenceErrorV1> {
        validate_owner(owner)?;
        if started <= 0 {
            return Err(ConsistencyPersistenceErrorV1::InvalidInput);
        }
        let expected =
            i64::try_from(expected).map_err(|_| ConsistencyPersistenceErrorV1::InvalidInput)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let row=sqlx::query("SELECT next_projection_generation FROM makosh_data.consistency_projection_control WHERE logical_owner_id=$1 FOR UPDATE").bind(owner).fetch_optional(&mut*tx).await.map_err(storage)?;
        let generation = if let Some(row) = row {
            let next: i64 = row.try_get("next_projection_generation").map_err(storage)?;
            let following = next
                .checked_add(1)
                .ok_or(ConsistencyPersistenceErrorV1::InvalidInput)?;
            sqlx::query("UPDATE makosh_data.consistency_projection_control SET next_projection_generation=$2 WHERE logical_owner_id=$1").bind(owner).bind(following).execute(&mut*tx).await.map_err(storage)?;
            next
        } else {
            sqlx::query("INSERT INTO makosh_data.consistency_projection_control(logical_owner_id,active_projection_generation,next_projection_generation,rebuilt_at_unix_millis) VALUES($1,0,2,$2)").bind(owner).bind(started).execute(&mut*tx).await.map_err(storage)?;
            1
        };
        sqlx::query("INSERT INTO makosh_data.consistency_projection_rebuilds(logical_owner_id,projection_generation,state,expected_source_count,applied_source_count,started_at_unix_millis) VALUES($1,$2,1,$3,0,$4)").bind(owner).bind(generation).bind(expected).bind(started).execute(&mut*tx).await.map_err(storage)?;
        tx.commit().await.map_err(storage)?;
        u64::try_from(generation).map_err(|_| ConsistencyPersistenceErrorV1::InvalidInput)
    }
    pub async fn apply_once(
        &self,
        input: &ApplyConsistencyMutationV1,
    ) -> Result<ConsistencyReplayOutcomeV1, ConsistencyPersistenceErrorV1> {
        validate_input(input)?;
        let generation = i64::try_from(input.projection_generation)
            .map_err(|_| ConsistencyPersistenceErrorV1::InvalidInput)?;
        let revision = i64::try_from(input.source_revision)
            .map_err(|_| ConsistencyPersistenceErrorV1::InvalidInput)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, &input.logical_owner_id).await?;
        if let Some(row)=sqlx::query("SELECT envelope_sha256,envelope_bytes FROM makosh_data.consistency_projection_inbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE").bind(&input.logical_owner_id).bind(input.input.message_id.as_slice()).fetch_optional(&mut*tx).await.map_err(storage)?{let sha:Vec<u8>=row.try_get("envelope_sha256").map_err(storage)?;let bytes:Vec<u8>=row.try_get("envelope_bytes").map_err(storage)?;tx.rollback().await.map_err(storage)?;return if sha==input.input.envelope_sha256&&bytes==input.input.envelope_bytes{Ok(ConsistencyReplayOutcomeV1::Replayed)}else{Err(ConsistencyPersistenceErrorV1::Conflict)}}
        let state:Option<i16>=sqlx::query_scalar("SELECT state FROM makosh_data.consistency_projection_rebuilds WHERE logical_owner_id=$1 AND projection_generation=$2 FOR UPDATE").bind(&input.logical_owner_id).bind(generation).fetch_optional(&mut*tx).await.map_err(storage)?;
        let active:Option<i64>=sqlx::query_scalar("SELECT active_projection_generation FROM makosh_data.consistency_projection_control WHERE logical_owner_id=$1 FOR UPDATE").bind(&input.logical_owner_id).fetch_optional(&mut*tx).await.map_err(storage)?;
        if state != Some(1) && !(state == Some(2) && active == Some(generation)) {
            return Err(ConsistencyPersistenceErrorV1::Conflict);
        }
        match &input.mutation {
            ConsistencyMutationV1::UpsertNode {
                node,
                source_revision,
                deleted,
            } => {
                let previous:Option<i64>=sqlx::query_scalar("SELECT source_revision FROM makosh_data.consistency_projection_nodes WHERE logical_owner_id=$1 AND projection_generation=$2 AND node_owner=$3 AND node_kind=$4 AND node_id=$5 FOR UPDATE").bind(&input.logical_owner_id).bind(generation).bind(&node.owner).bind(&node.kind).bind(node.id.as_slice()).fetch_optional(&mut*tx).await.map_err(storage)?;
                if previous.is_some_and(|value| {
                    value >= i64::try_from(*source_revision).unwrap_or(i64::MAX)
                }) {
                    return Err(ConsistencyPersistenceErrorV1::RevisionConflict);
                }
                sqlx::query("INSERT INTO makosh_data.consistency_projection_nodes(logical_owner_id,projection_generation,node_owner,node_kind,node_id,source_revision,deleted_at) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(logical_owner_id,projection_generation,node_owner,node_kind,node_id) DO UPDATE SET source_revision=EXCLUDED.source_revision,deleted_at=EXCLUDED.deleted_at").bind(&input.logical_owner_id).bind(generation).bind(&node.owner).bind(&node.kind).bind(node.id.as_slice()).bind(revision).bind(deleted.then_some(input.completed_at_unix_millis)).execute(&mut*tx).await.map_err(storage)?;
                if *deleted {
                    sqlx::query("UPDATE makosh_data.consistency_projection_edges SET edge_kind='',deleted_at=$6 WHERE logical_owner_id=$1 AND projection_generation=$2 AND deleted_at IS NULL AND ((source_owner=$3 AND source_kind=$4 AND source_id=$5) OR (target_owner=$3 AND target_kind=$4 AND target_id=$5))").bind(&input.logical_owner_id).bind(generation).bind(&node.owner).bind(&node.kind).bind(node.id.as_slice()).bind(input.completed_at_unix_millis).execute(&mut*tx).await.map_err(storage)?;
                }
            }
            ConsistencyMutationV1::UpsertEdge(edge) => {
                upsert_node(
                    &mut tx,
                    &input.logical_owner_id,
                    generation,
                    &edge.source,
                    revision,
                )
                .await?;
                upsert_node(
                    &mut tx,
                    &input.logical_owner_id,
                    generation,
                    &edge.target,
                    revision,
                )
                .await?;
                let previous:Option<i64>=sqlx::query_scalar("SELECT source_revision FROM makosh_data.consistency_projection_edges WHERE logical_owner_id=$1 AND projection_generation=$2 AND edge_id=$3 FOR UPDATE").bind(&input.logical_owner_id).bind(generation).bind(edge.edge_id.as_slice()).fetch_optional(&mut*tx).await.map_err(storage)?;
                if previous.is_some_and(|value| value >= revision) {
                    return Err(ConsistencyPersistenceErrorV1::RevisionConflict);
                }
                sqlx::query("INSERT INTO makosh_data.consistency_projection_edges(logical_owner_id,projection_generation,edge_id,source_owner,source_kind,source_id,target_owner,target_kind,target_id,edge_kind,source_revision,occurred_at_unix_millis,deleted_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) ON CONFLICT(logical_owner_id,projection_generation,edge_id) DO UPDATE SET source_owner=EXCLUDED.source_owner,source_kind=EXCLUDED.source_kind,source_id=EXCLUDED.source_id,target_owner=EXCLUDED.target_owner,target_kind=EXCLUDED.target_kind,target_id=EXCLUDED.target_id,edge_kind=EXCLUDED.edge_kind,source_revision=EXCLUDED.source_revision,occurred_at_unix_millis=EXCLUDED.occurred_at_unix_millis,deleted_at=EXCLUDED.deleted_at").bind(&input.logical_owner_id).bind(generation).bind(edge.edge_id.as_slice()).bind(&edge.source.owner).bind(&edge.source.kind).bind(edge.source.id.as_slice()).bind(&edge.target.owner).bind(&edge.target.kind).bind(edge.target.id.as_slice()).bind(&edge.edge_kind).bind(revision).bind(edge.occurred_at_unix_millis).bind(edge.deleted.then_some(input.completed_at_unix_millis)).execute(&mut*tx).await.map_err(storage)?;
            }
        }
        sqlx::query("INSERT INTO makosh_data.consistency_projection_inbox(logical_owner_id,message_id,envelope_sha256,envelope_bytes,source_owner,source_revision,completed_at_unix_millis) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(&input.logical_owner_id).bind(input.input.message_id.as_slice()).bind(input.input.envelope_sha256.as_slice()).bind(&input.input.envelope_bytes).bind(&input.source_owner).bind(revision).bind(input.completed_at_unix_millis).execute(&mut*tx).await.map_err(storage)?;
        if state == Some(1) {
            sqlx::query("UPDATE makosh_data.consistency_projection_rebuilds SET applied_source_count=applied_source_count+1 WHERE logical_owner_id=$1 AND projection_generation=$2").bind(&input.logical_owner_id).bind(generation).execute(&mut*tx).await.map_err(storage)?;
        }
        tx.commit().await.map_err(storage)?;
        Ok(ConsistencyReplayOutcomeV1::Applied)
    }
    pub async fn complete_rebuild(
        &self,
        owner: &str,
        generation: u64,
        completed: i64,
    ) -> Result<(), ConsistencyPersistenceErrorV1> {
        validate_owner(owner)?;
        let generation =
            i64::try_from(generation).map_err(|_| ConsistencyPersistenceErrorV1::InvalidInput)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let row=sqlx::query("SELECT state,expected_source_count,applied_source_count,started_at_unix_millis FROM makosh_data.consistency_projection_rebuilds WHERE logical_owner_id=$1 AND projection_generation=$2 FOR UPDATE").bind(owner).bind(generation).fetch_optional(&mut*tx).await.map_err(storage)?.ok_or(ConsistencyPersistenceErrorV1::NotFound)?;
        let state: i16 = row.try_get("state").map_err(storage)?;
        let expected: i64 = row.try_get("expected_source_count").map_err(storage)?;
        let applied: i64 = row.try_get("applied_source_count").map_err(storage)?;
        let started: i64 = row.try_get("started_at_unix_millis").map_err(storage)?;
        if state != 1 || expected != applied || completed < started {
            return Err(ConsistencyPersistenceErrorV1::Conflict);
        }
        sqlx::query("UPDATE makosh_data.consistency_projection_rebuilds SET state=2,completed_at_unix_millis=$3 WHERE logical_owner_id=$1 AND projection_generation=$2").bind(owner).bind(generation).bind(completed).execute(&mut*tx).await.map_err(storage)?;
        let affected=sqlx::query("UPDATE makosh_data.consistency_projection_control SET active_projection_generation=$2,rebuilt_at_unix_millis=$3 WHERE logical_owner_id=$1 AND active_projection_generation<$2").bind(owner).bind(generation).bind(completed).execute(&mut*tx).await.map_err(storage)?.rows_affected();
        if affected != 1 {
            return Err(ConsistencyPersistenceErrorV1::Conflict);
        }
        tx.commit().await.map_err(storage)
    }
    pub async fn load_active_edges(
        &self,
        owner: &str,
    ) -> Result<Vec<ConsistencyEdgeV1>, ConsistencyPersistenceErrorV1> {
        validate_owner(owner)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let rows=sqlx::query("SELECT e.edge_id,e.source_owner,e.source_kind,e.source_id,e.target_owner,e.target_kind,e.target_id,e.edge_kind,e.source_revision,e.occurred_at_unix_millis FROM makosh_data.consistency_projection_control c JOIN makosh_data.consistency_projection_edges e ON e.logical_owner_id=c.logical_owner_id AND e.projection_generation=c.active_projection_generation WHERE c.logical_owner_id=$1 AND e.deleted_at IS NULL ORDER BY e.edge_id LIMIT 10001").bind(owner).fetch_all(&mut*tx).await.map_err(storage)?;
        tx.rollback().await.map_err(storage)?;
        rows.into_iter()
            .map(|row| edge_from_row(owner, row))
            .collect()
    }
    pub async fn status(
        &self,
        owner: &str,
    ) -> Result<ConsistencyStatusRecordV1, ConsistencyPersistenceErrorV1> {
        validate_owner(owner)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let row=sqlx::query("SELECT c.active_projection_generation,c.rebuilt_at_unix_millis,(SELECT COUNT(*) FROM makosh_data.consistency_projection_nodes n WHERE n.logical_owner_id=c.logical_owner_id AND n.projection_generation=c.active_projection_generation AND n.deleted_at IS NULL) nodes,(SELECT COUNT(*) FROM makosh_data.consistency_projection_edges e WHERE e.logical_owner_id=c.logical_owner_id AND e.projection_generation=c.active_projection_generation AND e.deleted_at IS NULL) edges,(SELECT COUNT(*) FROM makosh_data.consistency_projection_inbox i WHERE i.logical_owner_id=c.logical_owner_id) source_events FROM makosh_data.consistency_projection_control c WHERE c.logical_owner_id=$1").bind(owner).fetch_optional(&mut*tx).await.map_err(storage)?.ok_or(ConsistencyPersistenceErrorV1::NotFound)?;
        tx.rollback().await.map_err(storage)?;
        Ok(ConsistencyStatusRecordV1 {
            active_generation: u64::try_from(
                row.try_get::<i64, _>("active_projection_generation")
                    .map_err(storage)?,
            )
            .map_err(|_| ConsistencyPersistenceErrorV1::StorageUnavailable)?,
            nodes: u64::try_from(row.try_get::<i64, _>("nodes").map_err(storage)?)
                .map_err(|_| ConsistencyPersistenceErrorV1::StorageUnavailable)?,
            edges: u64::try_from(row.try_get::<i64, _>("edges").map_err(storage)?)
                .map_err(|_| ConsistencyPersistenceErrorV1::StorageUnavailable)?,
            source_events: u64::try_from(row.try_get::<i64, _>("source_events").map_err(storage)?)
                .map_err(|_| ConsistencyPersistenceErrorV1::StorageUnavailable)?,
            rebuilt_at_unix_millis: row.try_get("rebuilt_at_unix_millis").map_err(storage)?,
        })
    }
}
async fn upsert_node(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    generation: i64,
    node: &ConsistencyNodeV1,
    revision: i64,
) -> Result<(), ConsistencyPersistenceErrorV1> {
    sqlx::query("INSERT INTO makosh_data.consistency_projection_nodes(logical_owner_id,projection_generation,node_owner,node_kind,node_id,source_revision,deleted_at) VALUES($1,$2,$3,$4,$5,$6,NULL) ON CONFLICT(logical_owner_id,projection_generation,node_owner,node_kind,node_id) DO UPDATE SET source_revision=GREATEST(makosh_data.consistency_projection_nodes.source_revision,EXCLUDED.source_revision),deleted_at=NULL").bind(owner).bind(generation).bind(&node.owner).bind(&node.kind).bind(node.id.as_slice()).bind(revision).execute(&mut**tx).await.map_err(storage)?;
    Ok(())
}
fn edge_from_row(
    owner: &str,
    row: sqlx::postgres::PgRow,
) -> Result<ConsistencyEdgeV1, ConsistencyPersistenceErrorV1> {
    fn id(
        row: &sqlx::postgres::PgRow,
        name: &str,
    ) -> Result<[u8; 16], ConsistencyPersistenceErrorV1> {
        row.try_get::<Vec<u8>, _>(name)
            .map_err(storage)?
            .try_into()
            .map_err(|_| ConsistencyPersistenceErrorV1::StorageUnavailable)
    }
    Ok(ConsistencyEdgeV1 {
        edge_id: id(&row, "edge_id")?,
        logical_owner_id: owner.to_owned(),
        source: ConsistencyNodeV1 {
            owner: row.try_get("source_owner").map_err(storage)?,
            kind: row.try_get("source_kind").map_err(storage)?,
            id: id(&row, "source_id")?,
        },
        target: ConsistencyNodeV1 {
            owner: row.try_get("target_owner").map_err(storage)?,
            kind: row.try_get("target_kind").map_err(storage)?,
            id: id(&row, "target_id")?,
        },
        edge_kind: row.try_get("edge_kind").map_err(storage)?,
        source_revision: u64::try_from(row.try_get::<i64, _>("source_revision").map_err(storage)?)
            .map_err(|_| ConsistencyPersistenceErrorV1::StorageUnavailable)?,
        occurred_at_unix_millis: row.try_get("occurred_at_unix_millis").map_err(storage)?,
        deleted: false,
    })
}
fn validate_input(value: &ApplyConsistencyMutationV1) -> Result<(), ConsistencyPersistenceErrorV1> {
    validate_owner(&value.logical_owner_id)?;
    if value.projection_generation == 0
        || value.source_revision == 0
        || value.completed_at_unix_millis <= 0
        || value.input.envelope_bytes.is_empty()
        || value.input.envelope_bytes.len() > 65_536
        || <[u8; 32]>::from(Sha256::digest(&value.input.envelope_bytes))
            != value.input.envelope_sha256
    {
        return Err(ConsistencyPersistenceErrorV1::InvalidInput);
    }
    match &value.mutation {
        ConsistencyMutationV1::UpsertNode {
            node,
            source_revision,
            ..
        } => {
            validate_consistency_node_v1(node)
                .map_err(|_| ConsistencyPersistenceErrorV1::InvalidInput)?;
            if *source_revision != value.source_revision {
                return Err(ConsistencyPersistenceErrorV1::InvalidInput);
            }
        }
        ConsistencyMutationV1::UpsertEdge(edge) => {
            validate_consistency_edge_v1(edge)
                .map_err(|_| ConsistencyPersistenceErrorV1::InvalidInput)?;
            if edge.logical_owner_id != value.logical_owner_id
                || edge.source_revision != value.source_revision
            {
                return Err(ConsistencyPersistenceErrorV1::InvalidInput);
            }
        }
    }
    Ok(())
}
async fn set_owner(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
) -> Result<(), ConsistencyPersistenceErrorV1> {
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(owner)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    Ok(())
}
fn validate_owner(owner: &str) -> Result<(), ConsistencyPersistenceErrorV1> {
    if owner.is_empty()
        || owner.len() > 128
        || !owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Err(ConsistencyPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}
fn storage<T>(_: T) -> ConsistencyPersistenceErrorV1 {
    ConsistencyPersistenceErrorV1::StorageUnavailable
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mutation_is_exact_and_public() {
        let bytes = b"edge-envelope".to_vec();
        let node = ConsistencyNodeV1 {
            owner: "persons".into(),
            kind: "person".into(),
            id: [2; 16],
        };
        let value = ApplyConsistencyMutationV1 {
            input: ConsistencyEnvelopeRecordV1 {
                message_id: [1; 16],
                envelope_sha256: Sha256::digest(&bytes).into(),
                envelope_bytes: bytes,
            },
            projection_generation: 1,
            logical_owner_id: "owner-1".into(),
            source_owner: "persons".into(),
            source_revision: 1,
            mutation: ConsistencyMutationV1::UpsertNode {
                node,
                source_revision: 1,
                deleted: false,
            },
            completed_at_unix_millis: 1000,
        };
        assert_eq!(validate_input(&value), Ok(()));
    }
}
