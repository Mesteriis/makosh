use makosh_search_core::{SearchProjectionDocumentV1, validate_search_projection_document_v1};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchEnvelopeRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplySearchDocumentV1 {
    pub input: SearchEnvelopeRecordV1,
    pub projection_generation: u64,
    pub document: SearchProjectionDocumentV1,
    pub token_digests: Vec<[u8; 32]>,
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchReplayOutcomeV1 {
    Applied,
    Replayed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchHitRecordV1 {
    pub source_owner: String,
    pub entity_kind: String,
    pub entity_id: [u8; 16],
    pub source_revision: u64,
    pub lifecycle_state: String,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchProjectionStatusRecordV1 {
    pub active_generation: u64,
    pub indexed_entities: u64,
    pub source_events: u64,
    pub rebuilt_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchCursorRecordV1 {
    pub source_owner: String,
    pub entity_kind: String,
    pub entity_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchPersistenceErrorV1 {
    InvalidInput,
    Conflict,
    RevisionConflict,
    NotFound,
    StorageUnavailable,
}

#[derive(Clone)]
pub struct SearchPersistenceV1 {
    pool: PgPool,
}

impl SearchPersistenceV1 {
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
    ) -> Result<Self, SearchPersistenceErrorV1> {
        if host.is_empty()
            || port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(SearchPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(u16::try_from(port).map_err(|_| SearchPersistenceErrorV1::StorageUnavailable)?)
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

    pub async fn verify_storage_ready(&self) -> Result<(), SearchPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage)
    }

    pub async fn ensure_live_generation(
        &self,
        owner: &str,
        now_unix_millis: i64,
    ) -> Result<u64, SearchPersistenceErrorV1> {
        validate_owner(owner)?;
        if now_unix_millis <= 0 {
            return Err(SearchPersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        sqlx::query("INSERT INTO makosh_data.search_projection_control (logical_owner_id,active_projection_generation,next_projection_generation,rebuilt_at_unix_millis) VALUES ($1,1,2,$2) ON CONFLICT (logical_owner_id) DO NOTHING")
            .bind(owner).bind(now_unix_millis).execute(&mut *tx).await.map_err(storage)?;
        sqlx::query("INSERT INTO makosh_data.search_projection_rebuilds (logical_owner_id,projection_generation,state,expected_source_count,applied_source_count,started_at_unix_millis,completed_at_unix_millis) VALUES ($1,1,2,0,0,$2,$2) ON CONFLICT (logical_owner_id,projection_generation) DO NOTHING")
            .bind(owner).bind(now_unix_millis).execute(&mut *tx).await.map_err(storage)?;
        let generation: i64 = sqlx::query_scalar("SELECT active_projection_generation FROM makosh_data.search_projection_control WHERE logical_owner_id=$1")
            .bind(owner).fetch_one(&mut *tx).await.map_err(storage)?;
        tx.commit().await.map_err(storage)?;
        u64::try_from(generation).map_err(|_| SearchPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn start_rebuild(
        &self,
        owner: &str,
        expected_source_count: u64,
        started_at_unix_millis: i64,
    ) -> Result<u64, SearchPersistenceErrorV1> {
        validate_owner(owner)?;
        if started_at_unix_millis <= 0 {
            return Err(SearchPersistenceErrorV1::InvalidInput);
        }
        let expected = i64::try_from(expected_source_count)
            .map_err(|_| SearchPersistenceErrorV1::InvalidInput)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let row = sqlx::query("SELECT active_projection_generation,next_projection_generation FROM makosh_data.search_projection_control WHERE logical_owner_id=$1 FOR UPDATE")
            .bind(owner).fetch_optional(&mut *tx).await.map_err(storage)?;
        let generation = if let Some(row) = row {
            let next: i64 = row.try_get("next_projection_generation").map_err(storage)?;
            sqlx::query("UPDATE makosh_data.search_projection_control SET next_projection_generation=$2 WHERE logical_owner_id=$1")
                .bind(owner).bind(next.checked_add(1).ok_or(SearchPersistenceErrorV1::InvalidInput)?)
                .execute(&mut *tx).await.map_err(storage)?;
            next
        } else {
            sqlx::query("INSERT INTO makosh_data.search_projection_control (logical_owner_id,active_projection_generation,next_projection_generation,rebuilt_at_unix_millis) VALUES ($1,0,2,$2)")
                .bind(owner).bind(started_at_unix_millis).execute(&mut *tx).await.map_err(storage)?;
            1
        };
        sqlx::query("INSERT INTO makosh_data.search_projection_rebuilds (logical_owner_id,projection_generation,state,expected_source_count,applied_source_count,started_at_unix_millis,completed_at_unix_millis) VALUES ($1,$2,1,$3,0,$4,NULL)")
            .bind(owner).bind(generation).bind(expected).bind(started_at_unix_millis)
            .execute(&mut *tx).await.map_err(storage)?;
        tx.commit().await.map_err(storage)?;
        u64::try_from(generation).map_err(|_| SearchPersistenceErrorV1::InvalidInput)
    }

    pub async fn apply_document_once(
        &self,
        input: &ApplySearchDocumentV1,
    ) -> Result<SearchReplayOutcomeV1, SearchPersistenceErrorV1> {
        validate_input(input)?;
        let owner = &input.document.logical_owner_id;
        let generation = i64::try_from(input.projection_generation)
            .map_err(|_| SearchPersistenceErrorV1::InvalidInput)?;
        let revision = i64::try_from(input.document.source_revision)
            .map_err(|_| SearchPersistenceErrorV1::InvalidInput)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        if let Some(row) = sqlx::query("SELECT envelope_sha256,envelope_bytes FROM makosh_data.search_projection_inbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE")
            .bind(owner).bind(input.input.message_id.as_slice()).fetch_optional(&mut *tx).await.map_err(storage)? {
            let sha: Vec<u8> = row.try_get("envelope_sha256").map_err(storage)?;
            let bytes: Vec<u8> = row.try_get("envelope_bytes").map_err(storage)?;
            tx.rollback().await.map_err(storage)?;
            return if sha == input.input.envelope_sha256 && bytes == input.input.envelope_bytes {
                Ok(SearchReplayOutcomeV1::Replayed)
            } else { Err(SearchPersistenceErrorV1::Conflict) };
        }
        let state: Option<i16> = sqlx::query_scalar("SELECT state FROM makosh_data.search_projection_rebuilds WHERE logical_owner_id=$1 AND projection_generation=$2 FOR UPDATE")
            .bind(owner).bind(generation).fetch_optional(&mut *tx).await.map_err(storage)?;
        let active: Option<i64> = sqlx::query_scalar("SELECT active_projection_generation FROM makosh_data.search_projection_control WHERE logical_owner_id=$1 FOR UPDATE")
            .bind(owner).fetch_optional(&mut *tx).await.map_err(storage)?;
        if state != Some(1) && !(state == Some(2) && active == Some(generation)) {
            return Err(SearchPersistenceErrorV1::Conflict);
        }
        let existing: Option<i64> = sqlx::query_scalar("SELECT source_revision FROM makosh_data.search_projection_documents WHERE logical_owner_id=$1 AND projection_generation=$2 AND source_owner=$3 AND entity_kind=$4 AND entity_id=$5 FOR UPDATE")
            .bind(owner).bind(generation).bind(&input.document.source_owner).bind(&input.document.entity_kind).bind(input.document.entity_id.as_slice())
            .fetch_optional(&mut *tx).await.map_err(storage)?;
        if existing.is_some_and(|value| value >= revision) {
            return Err(SearchPersistenceErrorV1::RevisionConflict);
        }
        sqlx::query("INSERT INTO makosh_data.search_projection_documents (logical_owner_id,projection_generation,source_owner,entity_kind,entity_id,source_revision,lifecycle_state,occurred_at_unix_millis,deleted_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (logical_owner_id,projection_generation,source_owner,entity_kind,entity_id) DO UPDATE SET source_revision=EXCLUDED.source_revision,lifecycle_state=EXCLUDED.lifecycle_state,occurred_at_unix_millis=EXCLUDED.occurred_at_unix_millis,deleted_at=EXCLUDED.deleted_at")
            .bind(owner).bind(generation).bind(&input.document.source_owner).bind(&input.document.entity_kind).bind(input.document.entity_id.as_slice()).bind(revision).bind(&input.document.lifecycle_state).bind(input.document.occurred_at_unix_millis).bind(input.document.deleted.then_some(input.completed_at_unix_millis))
            .execute(&mut *tx).await.map_err(storage)?;
        sqlx::query("DELETE FROM makosh_data.search_projection_tokens WHERE logical_owner_id=$1 AND projection_generation=$2 AND source_owner=$3 AND entity_kind=$4 AND entity_id=$5")
            .bind(owner).bind(generation).bind(&input.document.source_owner).bind(&input.document.entity_kind).bind(input.document.entity_id.as_slice()).execute(&mut *tx).await.map_err(storage)?;
        for digest in &input.token_digests {
            sqlx::query("INSERT INTO makosh_data.search_projection_tokens (logical_owner_id,projection_generation,source_owner,entity_kind,entity_id,token_digest) VALUES ($1,$2,$3,$4,$5,$6)")
                .bind(owner).bind(generation).bind(&input.document.source_owner).bind(&input.document.entity_kind).bind(input.document.entity_id.as_slice()).bind(digest.as_slice()).execute(&mut *tx).await.map_err(storage)?;
        }
        sqlx::query("INSERT INTO makosh_data.search_projection_inbox (logical_owner_id,message_id,envelope_sha256,envelope_bytes,source_owner,source_revision,completed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7)")
            .bind(owner).bind(input.input.message_id.as_slice()).bind(input.input.envelope_sha256.as_slice()).bind(&input.input.envelope_bytes).bind(&input.document.source_owner).bind(revision).bind(input.completed_at_unix_millis).execute(&mut *tx).await.map_err(storage)?;
        if state == Some(1) {
            sqlx::query("UPDATE makosh_data.search_projection_rebuilds SET applied_source_count=applied_source_count+1 WHERE logical_owner_id=$1 AND projection_generation=$2")
                .bind(owner).bind(generation).execute(&mut *tx).await.map_err(storage)?;
        }
        tx.commit().await.map_err(storage)?;
        Ok(SearchReplayOutcomeV1::Applied)
    }

    pub async fn complete_rebuild(
        &self,
        owner: &str,
        projection_generation: u64,
        completed_at_unix_millis: i64,
    ) -> Result<(), SearchPersistenceErrorV1> {
        validate_owner(owner)?;
        let generation = i64::try_from(projection_generation)
            .map_err(|_| SearchPersistenceErrorV1::InvalidInput)?;
        if generation <= 0 || completed_at_unix_millis <= 0 {
            return Err(SearchPersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let row = sqlx::query("SELECT state,expected_source_count,applied_source_count,started_at_unix_millis FROM makosh_data.search_projection_rebuilds WHERE logical_owner_id=$1 AND projection_generation=$2 FOR UPDATE")
            .bind(owner).bind(generation).fetch_optional(&mut *tx).await.map_err(storage)?
            .ok_or(SearchPersistenceErrorV1::NotFound)?;
        let state: i16 = row.try_get("state").map_err(storage)?;
        let expected: i64 = row.try_get("expected_source_count").map_err(storage)?;
        let applied: i64 = row.try_get("applied_source_count").map_err(storage)?;
        let started: i64 = row.try_get("started_at_unix_millis").map_err(storage)?;
        if state != 1 || expected != applied || completed_at_unix_millis < started {
            return Err(SearchPersistenceErrorV1::Conflict);
        }
        sqlx::query("UPDATE makosh_data.search_projection_rebuilds SET state=2,completed_at_unix_millis=$3 WHERE logical_owner_id=$1 AND projection_generation=$2")
            .bind(owner).bind(generation).bind(completed_at_unix_millis).execute(&mut *tx).await.map_err(storage)?;
        let affected = sqlx::query("UPDATE makosh_data.search_projection_control SET active_projection_generation=$2,rebuilt_at_unix_millis=$3 WHERE logical_owner_id=$1 AND active_projection_generation<$2")
            .bind(owner).bind(generation).bind(completed_at_unix_millis).execute(&mut *tx).await.map_err(storage)?.rows_affected();
        if affected != 1 {
            return Err(SearchPersistenceErrorV1::Conflict);
        }
        tx.commit().await.map_err(storage)
    }

    pub async fn query_active(
        &self,
        owner: &str,
        token_digests: &[[u8; 32]],
        after: Option<&SearchCursorRecordV1>,
        limit: u32,
    ) -> Result<Vec<SearchHitRecordV1>, SearchPersistenceErrorV1> {
        validate_owner(owner)?;
        if token_digests.is_empty() || token_digests.len() > 16 || !(1..=100).contains(&limit) {
            return Err(SearchPersistenceErrorV1::InvalidInput);
        }
        let digests = token_digests
            .iter()
            .map(|value| value.to_vec())
            .collect::<Vec<_>>();
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let (after_owner, after_kind, after_id) = after.map_or_else(
            || (String::new(), String::new(), Vec::new()),
            |value| {
                (
                    value.source_owner.clone(),
                    value.entity_kind.clone(),
                    value.entity_id.to_vec(),
                )
            },
        );
        let rows = sqlx::query("SELECT d.source_owner,d.entity_kind,d.entity_id,d.source_revision,d.lifecycle_state,d.occurred_at_unix_millis FROM makosh_data.search_projection_control c JOIN makosh_data.search_projection_documents d ON d.logical_owner_id=c.logical_owner_id AND d.projection_generation=c.active_projection_generation JOIN makosh_data.search_projection_tokens t ON t.logical_owner_id=d.logical_owner_id AND t.projection_generation=d.projection_generation AND t.source_owner=d.source_owner AND t.entity_kind=d.entity_kind AND t.entity_id=d.entity_id WHERE c.logical_owner_id=$1 AND d.deleted_at IS NULL AND t.token_digest=ANY($2) AND ($4='' OR (d.source_owner,d.entity_kind,d.entity_id)>($4,$5,$6)) GROUP BY d.source_owner,d.entity_kind,d.entity_id,d.source_revision,d.lifecycle_state,d.occurred_at_unix_millis HAVING COUNT(DISTINCT t.token_digest)=$3 ORDER BY d.source_owner,d.entity_kind,d.entity_id LIMIT $7")
            .bind(owner).bind(&digests).bind(i64::try_from(digests.len()).map_err(|_| SearchPersistenceErrorV1::InvalidInput)?).bind(after_owner).bind(after_kind).bind(after_id).bind(i64::from(limit))
            .fetch_all(&mut *tx).await.map_err(storage)?;
        tx.rollback().await.map_err(storage)?;
        rows.into_iter()
            .map(|row| {
                let entity: Vec<u8> = row.try_get("entity_id").map_err(storage)?;
                Ok(SearchHitRecordV1 {
                    source_owner: row.try_get("source_owner").map_err(storage)?,
                    entity_kind: row.try_get("entity_kind").map_err(storage)?,
                    entity_id: entity
                        .try_into()
                        .map_err(|_| SearchPersistenceErrorV1::StorageUnavailable)?,
                    source_revision: u64::try_from(
                        row.try_get::<i64, _>("source_revision").map_err(storage)?,
                    )
                    .map_err(|_| SearchPersistenceErrorV1::StorageUnavailable)?,
                    lifecycle_state: row.try_get("lifecycle_state").map_err(storage)?,
                    occurred_at_unix_millis: row
                        .try_get("occurred_at_unix_millis")
                        .map_err(storage)?,
                })
            })
            .collect()
    }

    pub async fn projection_status(
        &self,
        owner: &str,
    ) -> Result<SearchProjectionStatusRecordV1, SearchPersistenceErrorV1> {
        validate_owner(owner)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut tx, owner).await?;
        let row = sqlx::query("SELECT c.active_projection_generation,c.rebuilt_at_unix_millis,(SELECT COUNT(*) FROM makosh_data.search_projection_documents d WHERE d.logical_owner_id=c.logical_owner_id AND d.projection_generation=c.active_projection_generation AND d.deleted_at IS NULL) AS indexed_entities,(SELECT COUNT(*) FROM makosh_data.search_projection_inbox i WHERE i.logical_owner_id=c.logical_owner_id) AS source_events FROM makosh_data.search_projection_control c WHERE c.logical_owner_id=$1")
            .bind(owner).fetch_optional(&mut *tx).await.map_err(storage)?
            .ok_or(SearchPersistenceErrorV1::NotFound)?;
        tx.rollback().await.map_err(storage)?;
        Ok(SearchProjectionStatusRecordV1 {
            active_generation: u64::try_from(
                row.try_get::<i64, _>("active_projection_generation")
                    .map_err(storage)?,
            )
            .map_err(|_| SearchPersistenceErrorV1::StorageUnavailable)?,
            indexed_entities: u64::try_from(
                row.try_get::<i64, _>("indexed_entities").map_err(storage)?,
            )
            .map_err(|_| SearchPersistenceErrorV1::StorageUnavailable)?,
            source_events: u64::try_from(row.try_get::<i64, _>("source_events").map_err(storage)?)
                .map_err(|_| SearchPersistenceErrorV1::StorageUnavailable)?,
            rebuilt_at_unix_millis: row.try_get("rebuilt_at_unix_millis").map_err(storage)?,
        })
    }
}

fn validate_input(input: &ApplySearchDocumentV1) -> Result<(), SearchPersistenceErrorV1> {
    validate_search_projection_document_v1(&input.document)
        .map_err(|_| SearchPersistenceErrorV1::InvalidInput)?;
    if input.projection_generation == 0
        || input.completed_at_unix_millis < input.document.occurred_at_unix_millis
        || input.input.message_id.iter().all(|byte| *byte == 0)
        || input.input.envelope_bytes.is_empty()
        || input.input.envelope_bytes.len() > 65_536
        || <[u8; 32]>::from(Sha256::digest(&input.input.envelope_bytes))
            != input.input.envelope_sha256
        || (input.document.deleted && !input.token_digests.is_empty())
        || input
            .token_digests
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        Err(SearchPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}

async fn set_owner(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
) -> Result<(), SearchPersistenceErrorV1> {
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(owner)
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    Ok(())
}
fn validate_owner(owner: &str) -> Result<(), SearchPersistenceErrorV1> {
    if owner.is_empty()
        || owner.len() > 128
        || !owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Err(SearchPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}
fn storage<T>(_error: T) -> SearchPersistenceErrorV1 {
    SearchPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> ApplySearchDocumentV1 {
        let bytes = b"canonical-envelope".to_vec();
        ApplySearchDocumentV1 {
            input: SearchEnvelopeRecordV1 {
                message_id: [1; 16],
                envelope_sha256: Sha256::digest(&bytes).into(),
                envelope_bytes: bytes,
            },
            projection_generation: 1,
            document: SearchProjectionDocumentV1 {
                logical_owner_id: "owner-1".to_owned(),
                source_owner: "tasks".to_owned(),
                entity_kind: "task".to_owned(),
                entity_id: [2; 16],
                source_revision: 1,
                lifecycle_state: "active".to_owned(),
                occurred_at_unix_millis: 1000,
                deleted: false,
            },
            token_digests: vec![[3; 32], [4; 32]],
            completed_at_unix_millis: 1001,
        }
    }

    #[test]
    fn input_requires_exact_hash_sorted_tokens_and_empty_tombstone_tokens() {
        assert_eq!(validate_input(&input()), Ok(()));
        let mut changed = input();
        changed.input.envelope_bytes.push(1);
        assert_eq!(
            validate_input(&changed),
            Err(SearchPersistenceErrorV1::InvalidInput)
        );
        let mut unordered = input();
        unordered.token_digests.reverse();
        assert_eq!(
            validate_input(&unordered),
            Err(SearchPersistenceErrorV1::InvalidInput)
        );
        let mut deleted = input();
        deleted.document.deleted = true;
        deleted.document.lifecycle_state.clear();
        assert_eq!(
            validate_input(&deleted),
            Err(SearchPersistenceErrorV1::InvalidInput)
        );
        deleted.token_digests.clear();
        assert_eq!(validate_input(&deleted), Ok(()));
    }
}
