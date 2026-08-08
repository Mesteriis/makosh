use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::review_links::{
    link_domain_entity_in_transaction, materialize_review_transition_link_in_transaction,
};

use super::models::states::DecisionReviewState;

pub(crate) async fn link_decision_support_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    decision_id: impl Into<String>,
    confidence: f64,
    metadata: Value,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "decisions",
        "decision",
        decision_id.into(),
        Some("supports"),
        Some(confidence),
        Some(metadata),
    )
    .await
}

pub(crate) async fn link_decision_review_transition_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: Option<&str>,
    decision_id: &str,
    review_state: DecisionReviewState,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    materialize_review_transition_link_in_transaction(
        transaction,
        observation_id,
        "decisions",
        "decision",
        decision_id,
        "review_state",
        review_state.as_str(),
        metadata,
    )
    .await
}
