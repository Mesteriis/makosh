//! Explicit disposable PostgreSQL boundary for conformance only.

use std::str::FromStr;

use sqlx::{
    Executor,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
    schema::communication_cross_channel_forward_storage_bundle_v1,
};

pub struct CrossChannelForwardPersistenceConformanceV1;

impl CrossChannelForwardPersistenceConformanceV1 {
    pub async fn connect_url(
        database_url: &str,
    ) -> Result<CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1>
    {
        if database_url.trim().is_empty() {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::from_str(database_url)
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        Ok(CommunicationCrossChannelForwardPersistenceV1 { pool })
    }

    pub async fn install_schema(
        persistence: &CommunicationCrossChannelForwardPersistenceV1,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        persistence
            .pool
            .execute("CREATE SCHEMA IF NOT EXISTS makosh_data;")
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        for step in communication_cross_channel_forward_storage_bundle_v1().steps {
            let sql = std::str::from_utf8(&step.forward_sql_utf8)
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql.to_owned()))
                .execute(&persistence.pool)
                .await
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        }
        Ok(())
    }
}
