use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::decisions::models::decision::Decision;
use crate::domains::decisions::models::states::DecisionReviewState;
use crate::domains::obligations::models::read_model::Obligation;
use crate::domains::obligations::models::states::ObligationReviewState;
use crate::domains::personas::identity::models::PersonaIdentityCandidatePayload;
use crate::domains::personas::identity::models::PersonaIdentityReviewState;
use crate::domains::projects::link_reviews::models::{
    ProjectLinkReviewState, ProjectLinkTargetKind,
};
use crate::domains::relationships::models::{Relationship, RelationshipReviewState};
use crate::domains::review::errors::ReviewInboxError;
use crate::domains::review::models::{
    NewReviewItem, NewReviewItemEvidence, ReviewItem, ReviewItemKind, ReviewItemStatus,
    ReviewPromotionTarget,
};
use crate::domains::review::ports::ReviewInboxPort;
use crate::domains::tasks::candidates::ids::task_id_from_candidate;
use crate::domains::tasks::candidates::models::StoredCandidateRow;
use crate::domains::tasks::candidates::models::TaskCandidateReviewState;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

#[derive(Debug, Error)]
pub enum ReviewMirrorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error("review-backed observation is required: {0}")]
    ObservationRequired(String),
}

pub(crate) async fn sync_identity_candidate_to_review(
    pool: &sqlx::postgres::PgPool,
    payload: &PersonaIdentityCandidatePayload,
) -> Result<(), ReviewMirrorError> {
    let observation = ObservationStore::new(pool.clone())
        .capture(&identity_candidate_observation(payload))
        .await?;
    let _ =
        ensure_identity_candidate_review_item(pool, payload, &observation.observation_id).await?;
    Ok(())
}

pub(crate) async fn sync_identity_candidate_to_review_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonaIdentityCandidatePayload,
) -> Result<(), ReviewMirrorError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &identity_candidate_observation(payload),
    )
    .await?;
    let _ = ensure_identity_candidate_review_item_in_transaction(
        transaction,
        payload,
        &observation.observation_id,
    )
    .await?;
    Ok(())
}

pub(crate) async fn sync_identity_candidate_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
    review_state: PersonaIdentityReviewState,
    payload: &PersonaIdentityCandidatePayload,
) -> Result<(), ReviewMirrorError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &identity_candidate_observation(payload),
    )
    .await?;
    let review_item = ensure_identity_candidate_review_item_in_transaction(
        transaction,
        payload,
        &observation.observation_id,
    )
    .await?;

    match review_state {
        PersonaIdentityReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        PersonaIdentityReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        PersonaIdentityReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new("personas", "identity_candidate", identity_candidate_id),
            )
            .await?;
        }
    }

    Ok(())
}

pub(crate) async fn ensure_identity_candidate_review_item(
    pool: &sqlx::postgres::PgPool,
    payload: &PersonaIdentityCandidatePayload,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let item = ensure_identity_candidate_review_item_in_transaction(
        &mut transaction,
        payload,
        observation_id,
    )
    .await?;
    transaction.commit().await?;
    Ok(item)
}

pub(crate) async fn ensure_identity_candidate_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonaIdentityCandidatePayload,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    let identity_candidate_id = payload.identity_candidate_id();
    let evidence = identity_candidate_review_evidence(&identity_candidate_id, observation_id);

    match ReviewInboxPort::find_latest_by_kind_and_metadata_in_transaction(
        transaction,
        ReviewItemKind::IdentityCandidate,
        &json!({
            "identity_candidate_id": identity_candidate_id,
        }),
    )
    .await?
    {
        Some(item) => Ok(ReviewInboxPort::attach_evidence_in_transaction(
            transaction,
            &item.review_item_id,
            &[evidence],
        )
        .await?),
        None => {
            let item = identity_candidate_review_item(payload);
            Ok(ReviewInboxPort::create_with_evidence_in_transaction(
                transaction,
                &item,
                &[evidence],
            )
            .await?)
        }
    }
}

mod helpers;
use helpers::*;
pub(crate) mod decision;
pub(crate) mod obligation;
pub(crate) mod project_link;
pub(crate) mod relationship;
pub(crate) mod task;
