use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::review_links::{
    link_domain_entity, link_domain_entity_in_transaction,
};

pub(crate) async fn link_calendar_entity(
    pool: &PgPool,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    base_metadata: Value,
    extra_metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let metadata = merge_metadata(base_metadata, extra_metadata);
    link_domain_entity(
        pool,
        observation_id,
        "calendar",
        entity_kind,
        entity_id.into(),
        relationship_kind,
        None,
        Some(metadata),
    )
    .await
}

pub(crate) async fn link_calendar_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    base_metadata: Value,
    extra_metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let metadata = merge_metadata(base_metadata, extra_metadata);
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "calendar",
        entity_kind,
        entity_id.into(),
        relationship_kind,
        None,
        Some(metadata),
    )
    .await
}

fn merge_metadata(base_metadata: Value, extra_metadata: Option<Value>) -> Value {
    match extra_metadata {
        Some(extra) if base_metadata.is_object() && extra.is_object() => {
            let mut merged = base_metadata;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => base_metadata,
    }
}
