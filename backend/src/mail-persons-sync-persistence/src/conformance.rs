//! Explicit disposable PostgreSQL boundary for Mail-to-Person workflow conformance.

use std::str::FromStr;

use sqlx::{
    Executor, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    MailPersonsSyncAccountLifecycleKindV1, MailPersonsSyncPersistenceErrorV1,
    MailPersonsSyncPersistenceV1, mail_persons_sync_storage_bundle_v1,
};

pub struct MailPersonsSyncPersistenceConformanceV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncRlsEvidenceV1 {
    pub visible_owners: Vec<String>,
    pub cross_owner_updates: u64,
    pub cross_owner_deletes: u64,
    pub cross_owner_insert_blocked: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncAccountLifecycleEvidenceV1 {
    pub inbox_count: i64,
    pub outbox_count: i64,
    pub mapping_revision: Option<u64>,
    pub state: Option<MailPersonsSyncAccountLifecycleKindV1>,
    pub schedule_revision: Option<u64>,
}

impl MailPersonsSyncPersistenceConformanceV1 {
    pub async fn account_lifecycle_evidence(
        persistence: &MailPersonsSyncPersistenceV1,
        owner: &str,
        account_public_id: [u8; 16],
    ) -> Result<MailPersonsSyncAccountLifecycleEvidenceV1, MailPersonsSyncPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, owner).await?;
        let binding = sqlx::query(
            "SELECT mapping_revision,state,schedule_revision FROM \
             makosh_data.mail_persons_sync_account_bindings WHERE logical_owner_id=$1 \
             AND account_public_id=$2",
        )
        .bind(owner)
        .bind(account_public_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let inbox_count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_account_inbox \
             WHERE logical_owner_id=$1 AND account_public_id=$2",
        )
        .bind(owner)
        .bind(account_public_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let outbox_count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_schedule_control_outbox \
             WHERE logical_owner_id=$1 AND account_public_id=$2",
        )
        .bind(owner)
        .bind(account_public_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let (mapping_revision, state, schedule_revision) = binding
            .map(|row| {
                let state = match row.try_get::<i16, _>("state").map_err(|_| storage())? {
                    1 => MailPersonsSyncAccountLifecycleKindV1::Ready,
                    2 => MailPersonsSyncAccountLifecycleKindV1::Retired,
                    _ => return Err(MailPersonsSyncPersistenceErrorV1::StateConflict),
                };
                Ok((
                    Some(
                        row.try_get::<i64, _>("mapping_revision")
                            .map_err(|_| storage())?
                            .try_into()
                            .map_err(|_| MailPersonsSyncPersistenceErrorV1::StateConflict)?,
                    ),
                    Some(state),
                    Some(
                        row.try_get::<i64, _>("schedule_revision")
                            .map_err(|_| storage())?
                            .try_into()
                            .map_err(|_| MailPersonsSyncPersistenceErrorV1::StateConflict)?,
                    ),
                ))
            })
            .transpose()?
            .unwrap_or((None, None, None));
        transaction.rollback().await.map_err(|_| storage())?;
        Ok(MailPersonsSyncAccountLifecycleEvidenceV1 {
            inbox_count,
            outbox_count,
            mapping_revision,
            state,
            schedule_revision,
        })
    }

    pub async fn connect_url(
        database_url: &str,
    ) -> Result<MailPersonsSyncPersistenceV1, MailPersonsSyncPersistenceErrorV1> {
        if database_url.trim().is_empty() {
            return Err(MailPersonsSyncPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::from_str(database_url)
            .map_err(|_| MailPersonsSyncPersistenceErrorV1::InvalidInput)?;
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .map_err(|_| storage())?;
        Ok(MailPersonsSyncPersistenceV1::new(pool))
    }

    pub async fn install_schema(
        persistence: &MailPersonsSyncPersistenceV1,
    ) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        reset_schema(persistence).await?;
        let bundle = mail_persons_sync_storage_bundle_v1();
        if bundle.steps.is_empty() {
            return Err(MailPersonsSyncPersistenceErrorV1::InvalidInput);
        }
        for step in bundle.steps {
            let sql = std::str::from_utf8(&step.forward_sql_utf8)
                .map_err(|_| MailPersonsSyncPersistenceErrorV1::InvalidInput)?;
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql.to_owned()))
                .execute(persistence.pool())
                .await
                .map_err(|_| storage())?;
        }
        Ok(())
    }

    pub async fn durable_counts(
        persistence: &MailPersonsSyncPersistenceV1,
        owner: &str,
    ) -> Result<(i64, i64, i64), MailPersonsSyncPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, owner).await?;
        let inbox = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_inbox WHERE logical_owner_id=$1",
        )
        .bind(owner)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let outbox = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_outbox WHERE logical_owner_id=$1",
        )
        .bind(owner)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        let sources = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_sources WHERE logical_owner_id=$1",
        )
        .bind(owner)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| storage())?;
        transaction.rollback().await.map_err(|_| storage())?;
        Ok((inbox, outbox, sources))
    }

    pub async fn corrupt_outbox_bytes(
        persistence: &MailPersonsSyncPersistenceV1,
        owner: &str,
        message_id: [u8; 16],
    ) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, owner).await?;
        sqlx::query("UPDATE makosh_data.mail_persons_sync_outbox SET envelope_bytes='corrupt' WHERE logical_owner_id=$1 AND message_id=$2")
            .bind(owner).bind(message_id.as_slice()).execute(&mut *transaction).await.map_err(|_| storage())?;
        transaction.commit().await.map_err(|_| storage())
    }

    pub async fn rls_evidence(
        persistence: &MailPersonsSyncPersistenceV1,
        visible_owner: &str,
        hidden_owner: &str,
    ) -> Result<MailPersonsSyncRlsEvidenceV1, MailPersonsSyncPersistenceErrorV1> {
        sqlx::raw_sql(sqlx::AssertSqlSafe(
            "DROP ROLE IF EXISTS makosh_mail_persons_sync_rls_test; \
             CREATE ROLE makosh_mail_persons_sync_rls_test NOSUPERUSER NOBYPASSRLS NOLOGIN; \
             GRANT USAGE ON SCHEMA makosh_data TO makosh_mail_persons_sync_rls_test; \
             GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA makosh_data TO makosh_mail_persons_sync_rls_test;".to_owned(),
        ))
        .execute(persistence.pool()).await.map_err(|_| storage())?;
        let mut transaction = persistence.pool().begin().await.map_err(|_| storage())?;
        sqlx::query("SET LOCAL ROLE makosh_mail_persons_sync_rls_test")
            .execute(&mut *transaction)
            .await
            .map_err(|_| storage())?;
        set_owner(&mut transaction, visible_owner).await?;
        let visible_owners = sqlx::query("SELECT logical_owner_id FROM makosh_data.mail_persons_sync_runs ORDER BY logical_owner_id")
            .fetch_all(&mut *transaction).await.map_err(|_| storage())?
            .iter().map(|row| row.get(0)).collect();
        let cross_owner_updates = sqlx::query("UPDATE makosh_data.mail_persons_sync_runs SET state_revision=state_revision WHERE logical_owner_id=$1")
            .bind(hidden_owner).execute(&mut *transaction).await.map_err(|_| storage())?.rows_affected();
        let cross_owner_deletes =
            sqlx::query("DELETE FROM makosh_data.mail_persons_sync_runs WHERE logical_owner_id=$1")
                .bind(hidden_owner)
                .execute(&mut *transaction)
                .await
                .map_err(|_| storage())?
                .rows_affected();
        let cross_owner_insert_blocked = sqlx::query(
            "INSERT INTO makosh_data.mail_persons_sync_runs \
             (logical_owner_id,account_public_id,run_id,run_fingerprint,state,state_revision,next_page_sequence,processed_pages,processed_sources,rejection_code,created_at_unix_millis,updated_at_unix_millis) \
             VALUES ($1,$2,$3,$4,1,1,1,0,0,NULL,1000,1000)",
        )
        .bind(hidden_owner).bind([201_u8;16].as_slice()).bind([202_u8;16].as_slice())
        .bind([203_u8;32].as_slice()).execute(&mut *transaction).await.is_err();
        transaction.rollback().await.map_err(|_| storage())?;
        sqlx::raw_sql(sqlx::AssertSqlSafe(
            "DROP OWNED BY makosh_mail_persons_sync_rls_test; DROP ROLE makosh_mail_persons_sync_rls_test;".to_owned(),
        )).execute(persistence.pool()).await.map_err(|_| storage())?;
        Ok(MailPersonsSyncRlsEvidenceV1 {
            visible_owners,
            cross_owner_updates,
            cross_owner_deletes,
            cross_owner_insert_blocked,
        })
    }
}

async fn set_owner(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    owner: &str,
) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(owner)
        .execute(&mut **transaction)
        .await
        .map(|_| ())
        .map_err(|_| storage())
}

async fn reset_schema(
    persistence: &MailPersonsSyncPersistenceV1,
) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
    let expected_database = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_DISPOSABLE_DATABASE")
        .map_err(|_| MailPersonsSyncPersistenceErrorV1::InvalidInput)?;
    let expected_sentinel = std::env::var("MAKOSH_MAIL_PERSONS_SYNC_DISPOSABLE_SENTINEL")
        .map_err(|_| MailPersonsSyncPersistenceErrorV1::InvalidInput)?;
    if !expected_database.starts_with("makosh_mail_persons_sync_conformance_")
        || !expected_database
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        || expected_sentinel.len() != 64
        || !expected_sentinel
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(MailPersonsSyncPersistenceErrorV1::InvalidInput);
    }
    let current_database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(persistence.pool())
        .await
        .map_err(|_| storage())?;
    let sentinel: String = sqlx::query_scalar(
        "SELECT token FROM public.makosh_mail_persons_sync_disposable_sentinel WHERE sentinel_id=1",
    )
    .fetch_one(persistence.pool())
    .await
    .map_err(|_| storage())?;
    if current_database != expected_database || sentinel != expected_sentinel {
        return Err(MailPersonsSyncPersistenceErrorV1::InvalidInput);
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

const fn storage() -> MailPersonsSyncPersistenceErrorV1 {
    MailPersonsSyncPersistenceErrorV1::StorageUnavailable
}
