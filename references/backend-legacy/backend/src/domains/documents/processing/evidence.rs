use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;

pub(super) async fn link_document_processing_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    metadata: Value,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "documents",
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}
