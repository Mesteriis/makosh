use serde_json::Value;
use sqlx::postgres::PgPool;

use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::review_links::link_domain_entity;

pub(crate) async fn link_calendar_entity(
    pool: &PgPool,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    link_domain_entity(
        pool,
        observation_id,
        "calendar",
        entity_kind,
        entity_id.into(),
        None,
        None,
        metadata,
    )
    .await
}
