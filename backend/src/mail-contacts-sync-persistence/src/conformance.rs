//! Explicit disposable PostgreSQL boundary for conformance only.

use std::str::FromStr;

use sqlx::{
    Executor,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
    mail_contacts_sync_storage_bundle_v1,
};

pub struct MailContactsSyncPersistenceConformanceV1;

impl MailContactsSyncPersistenceConformanceV1 {
    pub async fn connect_url(
        database_url: &str,
    ) -> Result<MailContactsSyncPersistenceV1, MailContactsSyncPersistenceErrorV1> {
        if database_url.trim().is_empty() {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::from_str(database_url)
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidInput)?;
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .map_err(|_| MailContactsSyncPersistenceErrorV1::StorageUnavailable)?;
        Ok(MailContactsSyncPersistenceV1::from_test_pool(pool))
    }

    pub async fn install_schema(
        persistence: &MailContactsSyncPersistenceV1,
    ) -> Result<(), MailContactsSyncPersistenceErrorV1> {
        persistence
            .pool
            .execute("DROP SCHEMA IF EXISTS makosh_data CASCADE")
            .await
            .map_err(|_| MailContactsSyncPersistenceErrorV1::StorageUnavailable)?;
        persistence
            .pool
            .execute("CREATE SCHEMA makosh_data")
            .await
            .map_err(|_| MailContactsSyncPersistenceErrorV1::StorageUnavailable)?;
        for step in mail_contacts_sync_storage_bundle_v1().steps {
            let sql = std::str::from_utf8(&step.forward_sql_utf8)
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidInput)?;
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql.to_owned()))
                .execute(&persistence.pool)
                .await
                .map_err(|_| MailContactsSyncPersistenceErrorV1::StorageUnavailable)?;
        }
        Ok(())
    }
}
