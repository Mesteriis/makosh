use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use makosh_observations_postgres::errors::ObservationStoreError;

use super::super::evidence::link_mail_entity_in_transaction;

#[derive(Clone)]
pub struct ProviderSendStore {
    pool: PgPool,
}

impl ProviderSendStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record_sent_with_observation(
        &self,
        observation_id: &str,
        provider_message_id: &str,
        transport: &str,
        metadata: Option<Value>,
    ) -> Result<(), ProviderSendStoreError> {
        if observation_id.trim().is_empty() {
            return Err(ProviderSendStoreError::Invalid(
                "observation_id must not be empty",
            ));
        }
        if provider_message_id.trim().is_empty() {
            return Err(ProviderSendStoreError::Invalid(
                "provider_message_id must not be empty",
            ));
        }
        if transport.trim().is_empty() {
            return Err(ProviderSendStoreError::Invalid(
                "transport must not be empty",
            ));
        }

        let mut transaction = self.pool.begin().await?;
        link_mail_entity_in_transaction(
            &mut transaction,
            observation_id.trim(),
            "provider_send",
            provider_message_id.trim().to_owned(),
            "provider_send",
            json!({
                "transport": transport.trim(),
                "status": "sent",
            }),
            metadata,
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ProviderSendStoreError {
    #[error("invalid provider send payload: {0}")]
    Invalid(&'static str),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
