use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

use super::errors::DecisionStoreError;
use super::models::decision::Decision;
use super::models::states::DecisionReviewState;
use super::ports::DecisionReviewPort;

#[derive(Clone)]
pub struct DecisionCommandService {
    pool: PgPool,
}

impl DecisionCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
    ) -> Result<Decision, DecisionCommandServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "decision_id": decision_id,
                        "review_state": review_state.as_str(),
                        "operation": "decision_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("decision://{decision_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "decisions_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let decision = DecisionReviewPort::new(self.pool.clone())
            .set_review_state_with_observation(
                decision_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "decisions_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        Ok(decision)
    }
}

#[derive(Debug, Error)]
pub enum DecisionCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Decision(#[from] DecisionStoreError),
}
