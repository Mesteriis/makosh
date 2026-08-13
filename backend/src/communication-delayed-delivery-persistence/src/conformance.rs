//! Explicit connection boundary for disposable PostgreSQL conformance only.

use std::str::FromStr;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::{CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryPersistenceErrorV1};

pub struct DelayedDeliveryPersistenceConformanceV1;

impl DelayedDeliveryPersistenceConformanceV1 {
    pub async fn connect_url_as_nobypass_rls_role(
        database_url: &str,
    ) -> Result<CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryPersistenceErrorV1> {
        if database_url.trim().is_empty() {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::from_str(database_url)
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidInput)?;
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .after_connect(|connection, _meta| {
                Box::pin(async move {
                    sqlx::query("SET ROLE makosh_delayed_delivery_rls_test")
                        .execute(connection)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(options)
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(CommunicationDelayedDeliveryPersistenceV1 { pool })
    }

    pub async fn connect_url(
        database_url: &str,
    ) -> Result<CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryPersistenceErrorV1> {
        if database_url.trim().is_empty() {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::from_str(database_url)
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidInput)?;
        connect_options(options).await
    }

    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        database_id: &str,
    ) -> Result<CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryPersistenceErrorV1> {
        if host.trim().is_empty()
            || port == 0
            || username.trim().is_empty()
            || password.is_empty()
            || database_id.trim().is_empty()
        {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(port)
            .username(username)
            .password(password)
            .database(database_id);
        connect_options(options).await
    }
}

async fn connect_options(
    options: PgConnectOptions,
) -> Result<CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryPersistenceErrorV1> {
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
    Ok(CommunicationDelayedDeliveryPersistenceV1 { pool })
}
