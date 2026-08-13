//! Disposable PostgreSQL evidence for the rebuildable Identity Resolution engine.

use std::str::FromStr;

use sqlx::{
    Executor, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    IdentityResolutionPersistenceErrorV1, IdentityResolutionPersistenceV1,
    identity_resolution_storage_bundle_v1,
};

pub struct IdentityResolutionPersistenceConformanceV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentityResolutionDurableCountsV1 {
    pub candidates: i64,
    pub inbox: i64,
    pub outbox: i64,
    pub pending_outbox: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityResolutionRlsEvidenceV1 {
    pub visible_owners: Vec<String>,
    pub cross_owner_updates: u64,
    pub cross_owner_deletes: u64,
    pub cross_owner_insert_blocked: bool,
}

impl IdentityResolutionPersistenceConformanceV1 {
    pub async fn connect_url(
        database_url: &str,
    ) -> Result<IdentityResolutionPersistenceV1, IdentityResolutionPersistenceErrorV1> {
        if database_url.trim().is_empty() {
            return Err(IdentityResolutionPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::from_str(database_url)
            .map_err(|_| IdentityResolutionPersistenceErrorV1::InvalidInput)?;
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .map_err(|_| storage())?;
        Ok(IdentityResolutionPersistenceV1::new(pool))
    }

    pub async fn install_schema(
        persistence: &IdentityResolutionPersistenceV1,
    ) -> Result<(), IdentityResolutionPersistenceErrorV1> {
        reset_schema(persistence).await?;
        for step in identity_resolution_storage_bundle_v1().steps {
            let sql = std::str::from_utf8(&step.forward_sql_utf8)
                .map_err(|_| IdentityResolutionPersistenceErrorV1::InvalidInput)?;
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql.to_owned()))
                .execute(persistence.pool())
                .await
                .map_err(|_| storage())?;
        }
        Ok(())
    }

    pub async fn durable_counts(
        persistence: &IdentityResolutionPersistenceV1,
        owner: &str,
    ) -> Result<IdentityResolutionDurableCountsV1, IdentityResolutionPersistenceErrorV1> {
        let mut tx = persistence.pool().begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, owner).await?;
        let candidates = count(&mut tx, "identity_resolution_candidates", owner).await?;
        let inbox = count(&mut tx, "identity_resolution_inbox", owner).await?;
        let outbox = count(&mut tx, "identity_resolution_outbox", owner).await?;
        let pending_outbox = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.identity_resolution_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL",
        )
        .bind(owner)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| storage())?;
        tx.rollback().await.map_err(|_| storage())?;
        Ok(IdentityResolutionDurableCountsV1 {
            candidates,
            inbox,
            outbox,
            pending_outbox,
        })
    }

    pub async fn rls_evidence(
        persistence: &IdentityResolutionPersistenceV1,
        visible_owner: &str,
        hidden_owner: &str,
    ) -> Result<IdentityResolutionRlsEvidenceV1, IdentityResolutionPersistenceErrorV1> {
        sqlx::raw_sql(sqlx::AssertSqlSafe(
            "DROP ROLE IF EXISTS makosh_identity_resolution_rls_test; CREATE ROLE makosh_identity_resolution_rls_test NOSUPERUSER NOBYPASSRLS NOLOGIN; GRANT USAGE ON SCHEMA makosh_data TO makosh_identity_resolution_rls_test; GRANT USAGE,SELECT ON ALL SEQUENCES IN SCHEMA makosh_data TO makosh_identity_resolution_rls_test; GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA makosh_data TO makosh_identity_resolution_rls_test;".to_owned(),
        ))
        .execute(persistence.pool())
        .await
        .map_err(|_| storage())?;
        let mut tx = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SET LOCAL ROLE makosh_identity_resolution_rls_test")
            .execute(&mut *tx)
            .await
            .map_err(|_| storage())?;
        set_owner(&mut tx, visible_owner).await?;
        let visible_owners = sqlx::query(
            "SELECT logical_owner_id FROM makosh_data.identity_resolution_candidates ORDER BY logical_owner_id",
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| storage())?
        .iter()
        .map(|row| row.get(0))
        .collect();
        let cross_owner_updates = sqlx::query(
            "UPDATE makosh_data.identity_resolution_candidates SET resulting_owner_revision=resulting_owner_revision WHERE logical_owner_id=$1",
        )
        .bind(hidden_owner)
        .execute(&mut *tx)
        .await
        .map_err(|_| storage())?
        .rows_affected();
        let cross_owner_deletes = sqlx::query(
            "DELETE FROM makosh_data.identity_resolution_candidates WHERE logical_owner_id=$1",
        )
        .bind(hidden_owner)
        .execute(&mut *tx)
        .await
        .map_err(|_| storage())?
        .rows_affected();
        let cross_owner_insert_blocked = sqlx::query(
            "INSERT INTO makosh_data.identity_resolution_outbox (logical_owner_id,message_id,envelope_sha256,envelope_bytes,candidate_id,created_at_unix_millis) VALUES ($1,$2,$3,$4,$5,1000)",
        )
        .bind(hidden_owner)
        .bind([201_u8; 16].as_slice())
        .bind([202_u8; 32].as_slice())
        .bind([203_u8].as_slice())
        .bind([204_u8; 16].as_slice())
        .execute(&mut *tx)
        .await
        .is_err();
        tx.rollback().await.map_err(|_| storage())?;
        sqlx::raw_sql(sqlx::AssertSqlSafe(
            "DROP OWNED BY makosh_identity_resolution_rls_test; DROP ROLE makosh_identity_resolution_rls_test;".to_owned(),
        ))
        .execute(persistence.pool())
        .await
        .map_err(|_| storage())?;
        Ok(IdentityResolutionRlsEvidenceV1 {
            visible_owners,
            cross_owner_updates,
            cross_owner_deletes,
            cross_owner_insert_blocked,
        })
    }
}

async fn count(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    table: &'static str,
    owner: &str,
) -> Result<i64, IdentityResolutionPersistenceErrorV1> {
    let sql = match table {
        "identity_resolution_candidates" => {
            "SELECT COUNT(*) FROM makosh_data.identity_resolution_candidates WHERE logical_owner_id=$1"
        }
        "identity_resolution_inbox" => {
            "SELECT COUNT(*) FROM makosh_data.identity_resolution_inbox WHERE logical_owner_id=$1"
        }
        "identity_resolution_outbox" => {
            "SELECT COUNT(*) FROM makosh_data.identity_resolution_outbox WHERE logical_owner_id=$1"
        }
        _ => return Err(IdentityResolutionPersistenceErrorV1::InvalidInput),
    };
    sqlx::query_scalar(sql)
        .bind(owner)
        .fetch_one(&mut **tx)
        .await
        .map_err(|_| storage())
}

async fn set_owner(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    owner: &str,
) -> Result<(), IdentityResolutionPersistenceErrorV1> {
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(owner)
        .execute(&mut **tx)
        .await
        .map(|_| ())
        .map_err(|_| storage())
}

async fn reset_schema(
    persistence: &IdentityResolutionPersistenceV1,
) -> Result<(), IdentityResolutionPersistenceErrorV1> {
    let expected_database = std::env::var("MAKOSH_IDENTITY_RESOLUTION_DISPOSABLE_DATABASE")
        .map_err(|_| IdentityResolutionPersistenceErrorV1::InvalidInput)?;
    let expected_sentinel = std::env::var("MAKOSH_IDENTITY_RESOLUTION_DISPOSABLE_SENTINEL")
        .map_err(|_| IdentityResolutionPersistenceErrorV1::InvalidInput)?;
    if !expected_database.starts_with("makosh_identity_res_conformance_")
        || !expected_database
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        || expected_sentinel.len() != 64
        || !expected_sentinel
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(IdentityResolutionPersistenceErrorV1::InvalidInput);
    }
    let current_database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(persistence.pool())
        .await
        .map_err(|_| storage())?;
    let sentinel: String = sqlx::query_scalar(
        "SELECT token FROM public.makosh_identity_resolution_disposable_sentinel WHERE sentinel_id=1",
    )
    .fetch_one(persistence.pool()).await.map_err(|_| storage())?;
    if current_database != expected_database || sentinel != expected_sentinel {
        return Err(IdentityResolutionPersistenceErrorV1::InvalidInput);
    }
    persistence
        .pool()
        .execute("DROP SCHEMA IF EXISTS makosh_data CASCADE")
        .await
        .map_err(|_| storage())?;
    persistence
        .pool()
        .execute("CREATE SCHEMA makosh_data")
        .await
        .map(|_| ())
        .map_err(|_| storage())
}

const fn storage() -> IdentityResolutionPersistenceErrorV1 {
    IdentityResolutionPersistenceErrorV1::StorageUnavailable
}
