use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::domains::review::errors::ReviewInboxError;
use crate::domains::review::models::{
    NewReviewItem, NewReviewItemEvidence, ReviewItemKind, ReviewItemStatus,
};
use crate::domains::review::ports::ReviewInboxPort;
use crate::engines::consistency::{
    errors::ConsistencyError,
    models::{ContradictionObservation, ContradictionReviewState},
    store::ContradictionObservationStore,
};
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

#[derive(Clone)]
pub struct ContradictionReviewService {
    pool: PgPool,
}

impl ContradictionReviewService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        observation_id: &str,
        review_state: ContradictionReviewState,
        resolution: Option<&str>,
    ) -> Result<ContradictionObservation, ContradictionReviewServiceError> {
        let review_observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "contradiction_observation_id": observation_id,
                        "review_state": review_state.as_str(),
                        "resolution": resolution,
                        "operation": "contradiction_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("contradiction://{observation_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "consistency.review_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let observation = ContradictionObservationStore::new(self.pool.clone())
            .set_review_state_with_observation(
                observation_id,
                review_state,
                "makosh-frontend",
                resolution,
                Some(&review_observation.observation_id),
                None,
            )
            .await?;

        sync_contradiction_review_item(&self.pool, &observation).await?;

        Ok(observation)
    }
}

pub async fn sync_contradiction_review_item(
    pool: &PgPool,
    contradiction: &ContradictionObservation,
) -> Result<(), ContradictionReviewWorkflowError> {
    let mut transaction = pool.begin().await?;
    sync_contradiction_review_item_in_transaction(&mut transaction, contradiction).await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn sync_contradiction_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<(), ContradictionReviewWorkflowError> {
    ensure_contradiction_review_item_in_transaction(transaction, contradiction).await?;
    sync_contradiction_review_state_in_transaction(transaction, contradiction).await?;
    Ok(())
}

async fn ensure_contradiction_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<(), ContradictionReviewWorkflowError> {
    let evidence_observation =
        capture_evidence_observation_in_transaction(transaction, contradiction).await?;
    let item = NewReviewItem::new(
        ReviewItemKind::ContradictionCandidate,
        contradiction.conflict_type.clone(),
        contradiction_summary(contradiction),
        contradiction.confidence,
    )
    .metadata(json!({
        "mirrored_from": "contradictions",
        "contradiction_observation_id": contradiction.observation_id,
        "severity": contradiction.severity.as_str(),
        "old_source_kind": contradiction.old_source_kind.as_str(),
        "old_source_id": contradiction.old_source_id,
        "new_source_kind": contradiction.new_source_kind.as_str(),
        "new_source_id": contradiction.new_source_id,
    }));
    let evidence = NewReviewItemEvidence::new(evidence_observation.observation_id)
        .role("primary")
        .metadata(json!({
            "mirrored_from": "contradictions",
            "contradiction_observation_id": contradiction.observation_id,
        }));
    let _ = ReviewInboxPort::create_with_evidence_in_transaction(transaction, &item, &[evidence])
        .await?;
    Ok(())
}

async fn capture_evidence_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<makosh_observations_api::models::Observation, ObservationStoreError> {
    ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "CONTRADICTION_OBSERVATION",
            ObservationOriginKind::LocalRuntime,
            contradiction.created_at,
            json!({
                "contradiction_observation_id": contradiction.observation_id,
                "conflict_type": contradiction.conflict_type,
                "old_claim": contradiction.old_claim,
                "new_claim": contradiction.new_claim,
                "severity": contradiction.severity.as_str(),
                "review_state": contradiction.review_state.as_str(),
                "affected_entities": contradiction.affected_entities,
            }),
            format!("contradiction://{}", contradiction.observation_id),
        )
        .confidence(contradiction.confidence)
        .provenance(json!({
            "engine": "consistency",
            "pipeline": "contradiction_observations",
        })),
    )
    .await
}

async fn sync_contradiction_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    contradiction: &ContradictionObservation,
) -> Result<(), ContradictionReviewWorkflowError> {
    let review_item = ReviewInboxPort::find_latest_by_kind_and_metadata_in_transaction(
        transaction,
        ReviewItemKind::ContradictionCandidate,
        &json!({
            "contradiction_observation_id": contradiction.observation_id,
        }),
    )
    .await?
    .ok_or_else(|| ReviewInboxError::ReviewItemNotFound(contradiction.observation_id.clone()))?;

    let status = match contradiction.review_state {
        ContradictionReviewState::Suggested => ReviewItemStatus::New,
        ContradictionReviewState::UserConfirmed => ReviewItemStatus::Approved,
        ContradictionReviewState::UserRejected => ReviewItemStatus::Dismissed,
    };

    let _ = ReviewInboxPort::transition_status_in_transaction(
        transaction,
        &review_item.review_item_id,
        status,
    )
    .await?;
    Ok(())
}

fn contradiction_summary(contradiction: &ContradictionObservation) -> String {
    format!(
        "{} -> {}",
        contradiction.old_claim.trim(),
        contradiction.new_claim.trim()
    )
}

#[derive(Debug, Error)]
pub enum ContradictionReviewWorkflowError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Consistency(#[from] ConsistencyError),

    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}

#[derive(Debug, Error)]
pub enum ContradictionReviewServiceError {
    #[error(transparent)]
    Consistency(#[from] ConsistencyError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    ReviewWorkflow(#[from] ContradictionReviewWorkflowError),
}
