use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::review_links::{
    link_domain_entity, link_domain_entity_in_transaction,
};

pub(crate) async fn link_persona_entity(
    pool: &PgPool,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    link_domain_entity(
        pool,
        observation_id,
        "personas",
        entity_kind,
        entity_id.into(),
        relationship_kind,
        None,
        metadata,
    )
    .await?;
    Ok(())
}

pub(crate) async fn link_persona_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "personas",
        entity_kind,
        entity_id.into(),
        relationship_kind,
        None,
        metadata,
    )
    .await?;
    Ok(())
}
