use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::errors::ReviewInboxError;
use super::models::{ReviewItem, ReviewItemStatus};
use super::store::ReviewInboxStore;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

#[derive(Clone)]
pub struct ReviewInboxService {
    pool: PgPool,
}

impl ReviewInboxService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn transition_status_from_manual(
        &self,
        review_item_id: &str,
        status: ReviewItemStatus,
        captured_by: &'static str,
        endpoint: &'static str,
    ) -> Result<ReviewItem, ReviewInboxServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "review_item_id": review_item_id,
                        "operation": "review_item_status_transition",
                        "status": status.as_str(),
                    }),
                    format!("review-item://{review_item_id}/{}", status.as_str()),
                )
                .provenance(json!({
                    "captured_by": captured_by,
                    "endpoint": endpoint,
                    "status": status.as_str(),
                })),
            )
            .await
            .map_err(ReviewInboxServiceError::StatusObservationCapture)?;

        Ok(ReviewInboxStore::new(self.pool.clone())
            .set_status_with_observation(
                review_item_id,
                status,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": captured_by,
                    "endpoint": endpoint,
                    "status": status.as_str(),
                })),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum ReviewInboxServiceError {
    #[error("review status observation capture failed")]
    StatusObservationCapture(#[source] ObservationStoreError),

    #[error("review promotion observation capture failed")]
    PromotionObservationCapture(#[source] ObservationStoreError),

    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),
}
