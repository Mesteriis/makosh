//! Owner-local saved-search definitions containing keyed token digests only.

use std::collections::HashSet;

use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    CanonicalReadPageV1, CommunicationsDurablePersistence, CommunicationsPersistenceError,
};

const ACTIVE: i16 = 1;
const DELETED: i16 = 2;
const CREATED: i16 = 1;
const REPLACED: i16 = 2;
const REMOVED: i16 = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsSavedSearchWriteV1 {
    pub saved_search_id: [u8; 16],
    pub name: String,
    pub description: Option<String>,
    pub account_id: Option<[u8; 16]>,
    pub token_digests: Vec<[u8; 32]>,
    pub key_schema_revision: u32,
    pub changed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsSavedSearchSummaryV1 {
    pub saved_search_id: [u8; 16],
    pub name: String,
    pub description: Option<String>,
    pub account_id: Option<[u8; 16]>,
    pub token_count: u16,
    pub revision: u64,
    pub created_at_unix_seconds: i64,
    pub updated_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsSavedSearchDefinitionV1 {
    pub summary: CommunicationsSavedSearchSummaryV1,
    pub token_digests: Vec<[u8; 32]>,
    pub key_schema_revision: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommunicationsSavedSearchListAfterV1 {
    pub updated_at_unix_seconds: i64,
    pub saved_search_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSavedSearchMutationErrorV1 {
    Invalid,
    AccountNotFound,
    NotFound,
    RevisionConflict,
    StorageUnavailable,
}

enum LockedSavedSearchV1 {
    Missing,
    Tombstone,
    Active(CommunicationsSavedSearchDefinitionV1),
}

impl CommunicationsDurablePersistence {
    pub async fn create_saved_search(
        &self,
        write: &CommunicationsSavedSearchWriteV1,
    ) -> Result<CommunicationsSavedSearchSummaryV1, CommunicationsSavedSearchMutationErrorV1> {
        validate_write(write)?;
        let mut transaction = begin(&self.pool).await?;
        ensure_account_scope(&mut transaction, write.account_id).await?;
        match load_definition_for_update(&mut transaction, write.saved_search_id).await? {
            LockedSavedSearchV1::Active(existing) => {
                if existing.summary.revision == 1
                    && existing.key_schema_revision == write.key_schema_revision
                    && same_definition(&existing, write)
                {
                    commit(transaction).await?;
                    return Ok(existing.summary);
                }
                rollback(transaction).await?;
                return Err(CommunicationsSavedSearchMutationErrorV1::RevisionConflict);
            }
            LockedSavedSearchV1::Tombstone => {
                rollback(transaction).await?;
                return Err(CommunicationsSavedSearchMutationErrorV1::RevisionConflict);
            }
            LockedSavedSearchV1::Missing => {}
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communications_saved_query_definitions \
             (saved_search_id, name, description, account_id, token_count, \
              key_schema_revision, lifecycle_state, revision, \
              created_at_unix_seconds, updated_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8, $8) \
             ON CONFLICT (saved_search_id) DO NOTHING",
        )
        .bind(write.saved_search_id.as_slice())
        .bind(&write.name)
        .bind(&write.description)
        .bind(write.account_id.map(|value| value.to_vec()))
        .bind(i16::try_from(write.token_digests.len()).map_err(|_| invalid())?)
        .bind(i32::try_from(write.key_schema_revision).map_err(|_| invalid())?)
        .bind(ACTIVE)
        .bind(write.changed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        if inserted.rows_affected() == 0 {
            return match load_definition_for_update(&mut transaction, write.saved_search_id).await?
            {
                LockedSavedSearchV1::Active(existing)
                    if existing.summary.revision == 1
                        && existing.key_schema_revision == write.key_schema_revision
                        && same_definition(&existing, write) =>
                {
                    commit(transaction).await?;
                    Ok(existing.summary)
                }
                LockedSavedSearchV1::Missing
                | LockedSavedSearchV1::Tombstone
                | LockedSavedSearchV1::Active(_) => {
                    rollback(transaction).await?;
                    Err(CommunicationsSavedSearchMutationErrorV1::RevisionConflict)
                }
            };
        }
        replace_digests(
            &mut transaction,
            write.saved_search_id,
            &write.token_digests,
        )
        .await?;
        append_audit(&mut transaction, write, 1, CREATED).await?;
        commit(transaction).await?;
        Ok(summary_from_write(write, 1, write.changed_at_unix_seconds))
    }

    pub async fn replace_saved_search(
        &self,
        expected_revision: u64,
        write: &CommunicationsSavedSearchWriteV1,
    ) -> Result<CommunicationsSavedSearchSummaryV1, CommunicationsSavedSearchMutationErrorV1> {
        validate_write(write)?;
        if expected_revision == 0 {
            return Err(invalid());
        }
        let mut transaction = begin(&self.pool).await?;
        let existing =
            match load_definition_for_update(&mut transaction, write.saved_search_id).await? {
                LockedSavedSearchV1::Active(existing) => existing,
                LockedSavedSearchV1::Missing | LockedSavedSearchV1::Tombstone => {
                    rollback(transaction).await?;
                    return Err(CommunicationsSavedSearchMutationErrorV1::NotFound);
                }
            };
        if existing.summary.revision != expected_revision {
            rollback(transaction).await?;
            return Err(CommunicationsSavedSearchMutationErrorV1::RevisionConflict);
        }
        ensure_account_scope(&mut transaction, write.account_id).await?;
        let revision = expected_revision.checked_add(1).ok_or_else(invalid)?;
        sqlx::query(
            "UPDATE makosh_data.communications_saved_query_definitions SET \
             name = $2, description = $3, account_id = $4, token_count = $5, \
             key_schema_revision = $6, revision = $7, updated_at_unix_seconds = $8 \
             WHERE saved_search_id = $1 AND lifecycle_state = $9 AND revision = $10",
        )
        .bind(write.saved_search_id.as_slice())
        .bind(&write.name)
        .bind(&write.description)
        .bind(write.account_id.map(|value| value.to_vec()))
        .bind(i16::try_from(write.token_digests.len()).map_err(|_| invalid())?)
        .bind(i32::try_from(write.key_schema_revision).map_err(|_| invalid())?)
        .bind(i64::try_from(revision).map_err(|_| invalid())?)
        .bind(write.changed_at_unix_seconds)
        .bind(ACTIVE)
        .bind(i64::try_from(expected_revision).map_err(|_| invalid())?)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        replace_digests(
            &mut transaction,
            write.saved_search_id,
            &write.token_digests,
        )
        .await?;
        append_audit(&mut transaction, write, revision, REPLACED).await?;
        commit(transaction).await?;
        Ok(summary_from_write(
            write,
            revision,
            existing.summary.created_at_unix_seconds,
        ))
    }

    pub async fn delete_saved_search(
        &self,
        saved_search_id: [u8; 16],
        expected_revision: u64,
        changed_at_unix_seconds: i64,
    ) -> Result<u64, CommunicationsSavedSearchMutationErrorV1> {
        if saved_search_id.iter().all(|byte| *byte == 0)
            || expected_revision == 0
            || !valid_timestamp(changed_at_unix_seconds)
        {
            return Err(invalid());
        }
        let mut transaction = begin(&self.pool).await?;
        let existing = match load_definition_for_update(&mut transaction, saved_search_id).await? {
            LockedSavedSearchV1::Active(existing) => existing,
            LockedSavedSearchV1::Missing | LockedSavedSearchV1::Tombstone => {
                rollback(transaction).await?;
                return Err(CommunicationsSavedSearchMutationErrorV1::NotFound);
            }
        };
        if existing.summary.revision != expected_revision {
            rollback(transaction).await?;
            return Err(CommunicationsSavedSearchMutationErrorV1::RevisionConflict);
        }
        let revision = expected_revision.checked_add(1).ok_or_else(invalid)?;
        sqlx::query(
            "UPDATE makosh_data.communications_saved_query_definitions SET \
             lifecycle_state = $2, revision = $3, updated_at_unix_seconds = $4 \
             WHERE saved_search_id = $1 AND lifecycle_state = $5 AND revision = $6",
        )
        .bind(saved_search_id.as_slice())
        .bind(DELETED)
        .bind(i64::try_from(revision).map_err(|_| invalid())?)
        .bind(changed_at_unix_seconds)
        .bind(ACTIVE)
        .bind(i64::try_from(expected_revision).map_err(|_| invalid())?)
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        sqlx::query(
            "DELETE FROM makosh_data.communications_saved_query_token_digests \
             WHERE saved_search_id = $1",
        )
        .bind(saved_search_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        append_audit_hash(
            &mut transaction,
            saved_search_id,
            revision,
            REMOVED,
            definition_sha256(&existing),
            changed_at_unix_seconds,
        )
        .await?;
        commit(transaction).await?;
        Ok(revision)
    }

    pub async fn list_saved_searches(
        &self,
        after: Option<CommunicationsSavedSearchListAfterV1>,
        limit: u16,
    ) -> Result<
        CanonicalReadPageV1<CommunicationsSavedSearchSummaryV1>,
        CommunicationsSavedSearchMutationErrorV1,
    > {
        if limit == 0 || limit > 100 {
            return Err(invalid());
        }
        let (after_updated_at, after_id) = after
            .map(|value| {
                (
                    Some(value.updated_at_unix_seconds),
                    Some(value.saved_search_id.to_vec()),
                )
            })
            .unwrap_or((None, None));
        let rows = sqlx::query(
            "SELECT saved_search_id, name, description, account_id, token_count, \
             revision, created_at_unix_seconds, updated_at_unix_seconds \
             FROM makosh_data.communications_saved_query_definitions \
             WHERE lifecycle_state = $1 \
               AND ($2::BIGINT IS NULL OR updated_at_unix_seconds < $2 \
                 OR (updated_at_unix_seconds = $2 AND saved_search_id > $3)) \
             ORDER BY updated_at_unix_seconds DESC, saved_search_id ASC LIMIT $4",
        )
        .bind(ACTIVE)
        .bind(after_updated_at)
        .bind(after_id)
        .bind(i64::from(limit) + 1)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| storage())?;
        let mut items = rows
            .into_iter()
            .map(|row| summary_from_row(&row))
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > usize::from(limit);
        items.truncate(usize::from(limit));
        Ok(CanonicalReadPageV1 { items, has_more })
    }

    pub async fn saved_search_definition(
        &self,
        saved_search_id: [u8; 16],
    ) -> Result<CommunicationsSavedSearchDefinitionV1, CommunicationsSavedSearchMutationErrorV1>
    {
        let row = sqlx::query(
            "SELECT saved_search_id, name, description, account_id, token_count, \
             key_schema_revision, revision, created_at_unix_seconds, \
             updated_at_unix_seconds FROM makosh_data.communications_saved_query_definitions \
             WHERE saved_search_id = $1 AND lifecycle_state = $2",
        )
        .bind(saved_search_id.as_slice())
        .bind(ACTIVE)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| storage())?
        .ok_or(CommunicationsSavedSearchMutationErrorV1::NotFound)?;
        let summary = summary_from_row(&row)?;
        let key_schema_revision: i32 = row.try_get("key_schema_revision").map_err(|_| invalid())?;
        let token_digests = load_digests(&self.pool, saved_search_id).await?;
        if token_digests.len() != usize::from(summary.token_count) {
            return Err(storage());
        }
        Ok(CommunicationsSavedSearchDefinitionV1 {
            summary,
            token_digests,
            key_schema_revision: u32::try_from(key_schema_revision).map_err(|_| invalid())?,
        })
    }
}

async fn begin(
    pool: &sqlx::PgPool,
) -> Result<Transaction<'_, Postgres>, CommunicationsSavedSearchMutationErrorV1> {
    pool.begin().await.map_err(|_| storage())
}

async fn commit(
    transaction: Transaction<'_, Postgres>,
) -> Result<(), CommunicationsSavedSearchMutationErrorV1> {
    transaction.commit().await.map_err(|_| storage())
}

async fn rollback(
    transaction: Transaction<'_, Postgres>,
) -> Result<(), CommunicationsSavedSearchMutationErrorV1> {
    transaction.rollback().await.map_err(|_| storage())
}

async fn ensure_account_scope(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Option<[u8; 16]>,
) -> Result<(), CommunicationsSavedSearchMutationErrorV1> {
    let Some(account_id) = account_id else {
        return Ok(());
    };
    let exists = sqlx::query(
        "SELECT account_id FROM makosh_data.communications_accounts \
         WHERE account_id = $1 FOR SHARE",
    )
    .bind(account_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| storage())?
    .is_some();
    if exists {
        Ok(())
    } else {
        Err(CommunicationsSavedSearchMutationErrorV1::AccountNotFound)
    }
}

async fn load_definition_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    saved_search_id: [u8; 16],
) -> Result<LockedSavedSearchV1, CommunicationsSavedSearchMutationErrorV1> {
    let Some(row) = sqlx::query(
        "SELECT saved_search_id, name, description, account_id, token_count, \
         key_schema_revision, revision, created_at_unix_seconds, \
         updated_at_unix_seconds, lifecycle_state \
         FROM makosh_data.communications_saved_query_definitions \
         WHERE saved_search_id = $1 FOR UPDATE",
    )
    .bind(saved_search_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| storage())?
    else {
        return Ok(LockedSavedSearchV1::Missing);
    };
    let lifecycle_state: i16 = row.try_get("lifecycle_state").map_err(|_| invalid())?;
    if lifecycle_state != ACTIVE {
        return Ok(LockedSavedSearchV1::Tombstone);
    }
    let summary = summary_from_row(&row)?;
    let key_schema_revision: i32 = row.try_get("key_schema_revision").map_err(|_| invalid())?;
    let digest_rows = sqlx::query(
        "SELECT token_digest FROM makosh_data.communications_saved_query_token_digests \
         WHERE saved_search_id = $1 ORDER BY position ASC",
    )
    .bind(saved_search_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    let token_digests = digest_rows
        .into_iter()
        .map(|row| {
            digest32(
                row.try_get::<Vec<u8>, _>("token_digest")
                    .map_err(|_| invalid())?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(LockedSavedSearchV1::Active(
        CommunicationsSavedSearchDefinitionV1 {
            summary,
            token_digests,
            key_schema_revision: u32::try_from(key_schema_revision).map_err(|_| invalid())?,
        },
    ))
}

async fn load_digests(
    pool: &sqlx::PgPool,
    saved_search_id: [u8; 16],
) -> Result<Vec<[u8; 32]>, CommunicationsSavedSearchMutationErrorV1> {
    sqlx::query(
        "SELECT token_digest FROM makosh_data.communications_saved_query_token_digests \
         WHERE saved_search_id = $1 ORDER BY position ASC",
    )
    .bind(saved_search_id.as_slice())
    .fetch_all(pool)
    .await
    .map_err(|_| storage())?
    .into_iter()
    .map(|row| {
        digest32(
            row.try_get::<Vec<u8>, _>("token_digest")
                .map_err(|_| invalid())?,
        )
    })
    .collect()
}

async fn replace_digests(
    transaction: &mut Transaction<'_, Postgres>,
    saved_search_id: [u8; 16],
    token_digests: &[[u8; 32]],
) -> Result<(), CommunicationsSavedSearchMutationErrorV1> {
    sqlx::query(
        "DELETE FROM makosh_data.communications_saved_query_token_digests \
         WHERE saved_search_id = $1",
    )
    .bind(saved_search_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    for (position, digest) in token_digests.iter().enumerate() {
        sqlx::query(
            "INSERT INTO makosh_data.communications_saved_query_token_digests \
             (saved_search_id, position, token_digest) VALUES ($1, $2, $3)",
        )
        .bind(saved_search_id.as_slice())
        .bind(i16::try_from(position).map_err(|_| invalid())?)
        .bind(digest.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(|_| storage())?;
    }
    Ok(())
}

async fn append_audit(
    transaction: &mut Transaction<'_, Postgres>,
    write: &CommunicationsSavedSearchWriteV1,
    revision: u64,
    change_kind: i16,
) -> Result<(), CommunicationsSavedSearchMutationErrorV1> {
    let summary = summary_from_write(write, revision, write.changed_at_unix_seconds);
    append_audit_hash(
        transaction,
        write.saved_search_id,
        revision,
        change_kind,
        definition_sha256(&CommunicationsSavedSearchDefinitionV1 {
            summary,
            token_digests: write.token_digests.clone(),
            key_schema_revision: write.key_schema_revision,
        }),
        write.changed_at_unix_seconds,
    )
    .await
}

async fn append_audit_hash(
    transaction: &mut Transaction<'_, Postgres>,
    saved_search_id: [u8; 16],
    revision: u64,
    change_kind: i16,
    definition_sha256: [u8; 32],
    changed_at_unix_seconds: i64,
) -> Result<(), CommunicationsSavedSearchMutationErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.communications_saved_query_audit \
         (saved_search_id, revision, change_kind, definition_sha256, changed_at_unix_seconds) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(saved_search_id.as_slice())
    .bind(i64::try_from(revision).map_err(|_| invalid())?)
    .bind(change_kind)
    .bind(definition_sha256.as_slice())
    .bind(changed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| storage())?;
    Ok(())
}

fn validate_write(
    write: &CommunicationsSavedSearchWriteV1,
) -> Result<(), CommunicationsSavedSearchMutationErrorV1> {
    let unique = write.token_digests.iter().copied().collect::<HashSet<_>>();
    if write.saved_search_id.iter().all(|byte| *byte == 0)
        || write.name.is_empty()
        || write.name.len() > 128
        || write.name.chars().any(char::is_control)
        || write.description.as_ref().is_some_and(|value| {
            value.is_empty() || value.len() > 512 || value.chars().any(char::is_control)
        })
        || write
            .account_id
            .is_some_and(|value| value.iter().all(|byte| *byte == 0))
        || write.token_digests.is_empty()
        || write.token_digests.len() > 16
        || unique.len() != write.token_digests.len()
        || write.key_schema_revision == 0
        || !valid_timestamp(write.changed_at_unix_seconds)
    {
        return Err(invalid());
    }
    Ok(())
}

fn same_definition(
    existing: &CommunicationsSavedSearchDefinitionV1,
    write: &CommunicationsSavedSearchWriteV1,
) -> bool {
    existing.summary.name == write.name
        && existing.summary.description == write.description
        && existing.summary.account_id == write.account_id
        && existing.token_digests == write.token_digests
}

fn summary_from_write(
    write: &CommunicationsSavedSearchWriteV1,
    revision: u64,
    created_at_unix_seconds: i64,
) -> CommunicationsSavedSearchSummaryV1 {
    CommunicationsSavedSearchSummaryV1 {
        saved_search_id: write.saved_search_id,
        name: write.name.clone(),
        description: write.description.clone(),
        account_id: write.account_id,
        token_count: u16::try_from(write.token_digests.len())
            .expect("validated saved-search token count fits u16"),
        revision,
        created_at_unix_seconds,
        updated_at_unix_seconds: write.changed_at_unix_seconds,
    }
}

fn summary_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<CommunicationsSavedSearchSummaryV1, CommunicationsSavedSearchMutationErrorV1> {
    let saved_search_id = id16(row.try_get("saved_search_id").map_err(|_| invalid())?)?;
    let account_id = row
        .try_get::<Option<Vec<u8>>, _>("account_id")
        .map_err(|_| invalid())?
        .map(id16)
        .transpose()?;
    let token_count: i16 = row.try_get("token_count").map_err(|_| invalid())?;
    let revision: i64 = row.try_get("revision").map_err(|_| invalid())?;
    Ok(CommunicationsSavedSearchSummaryV1 {
        saved_search_id,
        name: row.try_get("name").map_err(|_| invalid())?,
        description: row.try_get("description").map_err(|_| invalid())?,
        account_id,
        token_count: u16::try_from(token_count).map_err(|_| invalid())?,
        revision: u64::try_from(revision).map_err(|_| invalid())?,
        created_at_unix_seconds: row
            .try_get("created_at_unix_seconds")
            .map_err(|_| invalid())?,
        updated_at_unix_seconds: row
            .try_get("updated_at_unix_seconds")
            .map_err(|_| invalid())?,
    })
}

fn definition_sha256(definition: &CommunicationsSavedSearchDefinitionV1) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communications.saved-search.definition.v1\0");
    digest.update(definition.summary.saved_search_id);
    digest.update((definition.summary.name.len() as u64).to_be_bytes());
    digest.update(definition.summary.name.as_bytes());
    if let Some(description) = &definition.summary.description {
        digest.update([1]);
        digest.update((description.len() as u64).to_be_bytes());
        digest.update(description.as_bytes());
    } else {
        digest.update([0]);
    }
    if let Some(account_id) = definition.summary.account_id {
        digest.update([1]);
        digest.update(account_id);
    } else {
        digest.update([0]);
    }
    digest.update(definition.key_schema_revision.to_be_bytes());
    for token_digest in &definition.token_digests {
        digest.update(token_digest);
    }
    digest.finalize().into()
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], CommunicationsSavedSearchMutationErrorV1> {
    value.try_into().map_err(|_| invalid())
}

fn digest32(value: Vec<u8>) -> Result<[u8; 32], CommunicationsSavedSearchMutationErrorV1> {
    value.try_into().map_err(|_| invalid())
}

const fn valid_timestamp(value: i64) -> bool {
    value >= -62_135_596_800 && value <= 253_402_300_799
}

const fn invalid() -> CommunicationsSavedSearchMutationErrorV1 {
    CommunicationsSavedSearchMutationErrorV1::Invalid
}

const fn storage() -> CommunicationsSavedSearchMutationErrorV1 {
    CommunicationsSavedSearchMutationErrorV1::StorageUnavailable
}

impl From<CommunicationsPersistenceError> for CommunicationsSavedSearchMutationErrorV1 {
    fn from(_: CommunicationsPersistenceError) -> Self {
        Self::StorageUnavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_validation_rejects_duplicate_digests_without_sql() {
        let digest = [7; 32];
        let write = CommunicationsSavedSearchWriteV1 {
            saved_search_id: [1; 16],
            name: "review".to_owned(),
            description: None,
            account_id: None,
            token_digests: vec![digest, digest],
            key_schema_revision: 1,
            changed_at_unix_seconds: 1,
        };

        assert_eq!(
            validate_write(&write),
            Err(CommunicationsSavedSearchMutationErrorV1::Invalid),
        );
    }

    #[test]
    fn audit_fingerprint_is_stable_without_query_plaintext() {
        let definition = CommunicationsSavedSearchDefinitionV1 {
            summary: CommunicationsSavedSearchSummaryV1 {
                saved_search_id: [1; 16],
                name: "review".to_owned(),
                description: Some("owner".to_owned()),
                account_id: Some([2; 16]),
                token_count: 1,
                revision: 1,
                created_at_unix_seconds: 3,
                updated_at_unix_seconds: 3,
            },
            token_digests: vec![[4; 32]],
            key_schema_revision: 1,
        };

        assert_eq!(
            definition_sha256(&definition),
            definition_sha256(&definition)
        );
    }
}
