use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;

use super::errors::TaskCoreError;

pub(crate) async fn materialize_task_observation_link_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: Option<&str>,
    relationship_kind: Option<&str>,
    task_id: &str,
    metadata: Option<Value>,
) -> Result<(), TaskCoreError> {
    let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "tasks",
        "task",
        task_id.to_owned(),
        relationship_kind.filter(|value| !value.is_empty()),
        None,
        metadata,
    )
    .await?;
    Ok(())
}

pub(crate) async fn materialize_task_entity_link_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: Option<&str>,
    entity_kind: &str,
    entity_id: &str,
    relationship_kind: Option<&str>,
    confidence: Option<f64>,
    metadata: Option<Value>,
) -> Result<(), TaskCoreError> {
    let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "tasks",
        entity_kind,
        entity_id.to_owned(),
        relationship_kind.filter(|value| !value.is_empty()),
        confidence,
        metadata,
    )
    .await?;
    Ok(())
}
