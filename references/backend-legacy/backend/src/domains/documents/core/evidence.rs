use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;

pub(crate) async fn link_document_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    document_id: impl Into<String>,
    relationship_kind: Option<&str>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "documents",
        "document",
        document_id.into(),
        relationship_kind,
        None,
        metadata,
    )
    .await
}
