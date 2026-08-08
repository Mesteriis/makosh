use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

use super::errors::ObligationStoreError;
use super::models::read_model::Obligation;
use super::models::states::ObligationReviewState;
use super::ports::ObligationReviewPort;

#[derive(Clone)]
pub struct ObligationCommandService {
    pool: PgPool,
}

impl ObligationCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
    ) -> Result<Obligation, ObligationCommandServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "obligation_id": obligation_id,
                        "review_state": review_state.as_str(),
                        "operation": "obligation_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("obligation://{obligation_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "obligations_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let obligation = ObligationReviewPort::new(self.pool.clone())
            .set_review_state_with_observation(
                obligation_id,
                review_state,
                Some(&observation.observation_id),
                None,
            )
            .await
            .map_err(|error| ObligationCommandServiceError::Obligation(error))?;

        Ok(obligation)
    }
}

#[derive(Debug, Error)]
pub enum ObligationCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),
}
