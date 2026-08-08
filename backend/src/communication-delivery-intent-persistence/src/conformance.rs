//! Explicit disposable PostgreSQL boundary for conformance only.

use std::str::FromStr;

use sqlx::{
    Executor,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    CommunicationDeliveryIntentPersistenceV1, DeliveryIntentPersistenceErrorV1,
    schema::communication_delivery_intent_storage_bundle_v1,
};

pub struct DeliveryIntentPersistenceConformanceV1;

impl DeliveryIntentPersistenceConformanceV1 {
    pub async fn connect_url(
        database_url: &str,
    ) -> Result<CommunicationDeliveryIntentPersistenceV1, DeliveryIntentPersistenceErrorV1> {
        if database_url.trim().is_empty() {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::from_str(database_url)
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?;
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        Ok(CommunicationDeliveryIntentPersistenceV1 { pool })
    }

    pub async fn install_schema(
        persistence: &CommunicationDeliveryIntentPersistenceV1,
    ) -> Result<(), DeliveryIntentPersistenceErrorV1> {
        persistence
            .pool
            .execute("CREATE SCHEMA IF NOT EXISTS makosh_data;")
            .await
            .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        for step in communication_delivery_intent_storage_bundle_v1().steps {
            let sql = std::str::from_utf8(&step.forward_sql_utf8)
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?;
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql.to_owned()))
                .execute(&persistence.pool)
                .await
                .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        }
        Ok(())
    }
}
