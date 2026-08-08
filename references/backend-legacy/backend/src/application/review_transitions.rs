use chrono::Utc;
use serde_json::json;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};
use thiserror::Error;

use crate::domains::decisions::errors::DecisionStoreError;
use crate::domains::decisions::models::decision::Decision;
use crate::domains::decisions::models::states::DecisionReviewState;
use crate::domains::decisions::ports::DecisionReviewPort;
use crate::domains::obligations::errors::ObligationStoreError;
use crate::domains::obligations::models::entity_kind::ObligationEntityKind;
use crate::domains::obligations::models::evidence::NewObligationEvidence;
use crate::domains::obligations::models::obligation::NewObligation;
use crate::domains::obligations::models::read_model::Obligation;
use crate::domains::obligations::ports::ObligationReviewPort;
use crate::domains::relationships::models::{Relationship, RelationshipReviewState};
use crate::domains::tasks::candidates::constants::{
    OBLIGATION_CANDIDATE_METADATA_KEY, TASK_CANDIDATE_KIND_OBLIGATION_TASK,
};
use crate::domains::tasks::candidates::errors::TaskCandidateError;
use crate::domains::tasks::candidates::ids::task_id_from_candidate;
use crate::domains::tasks::candidates::models::StoredCandidateRow;
use crate::domains::tasks::candidates::models::{
    TaskCandidateReviewCommand, TaskCandidateReviewCommandResult, TaskCandidateReviewState,
};
use crate::domains::tasks::candidates::store::TaskCandidateStore;
use crate::domains::tasks::core::errors::TaskCoreError;
use crate::domains::tasks::core::obligation_links::ObligationTaskLinkStore;
use crate::engines::obligation::models::ObligationCandidate;
use crate::workflows::review_mirror::{
    ReviewMirrorError, decision::sync_decision_review_state_with_observation,
    obligation::sync_obligation_review_state_with_observation,
    relationship::sync_relationship_review_state_with_observation,
    task::sync_task_candidate_review_state_in_transaction,
};
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

use super::relationship_graph::{RelationshipGraphCoordinator, RelationshipGraphCoordinatorError};

#[derive(Clone)]
pub struct DecisionReviewApplicationService {
    pool: PgPool,
}

impl DecisionReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
    ) -> Result<Decision, DecisionReviewApplicationError> {
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
                    "captured_by": "decision_review_application.review_manual",
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
                    "captured_by": "decision_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        sync_decision_review_state_with_observation(
            &self.pool,
            &decision,
            &observation.observation_id,
        )
        .await?;

        Ok(decision)
    }
}

#[derive(Debug, Error)]
pub enum DecisionReviewApplicationError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Decision(#[from] DecisionStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}

#[derive(Clone)]
pub struct ObligationReviewApplicationService {
    pool: PgPool,
}

impl ObligationReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
    ) -> Result<Obligation, ObligationReviewApplicationError> {
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
                    "captured_by": "obligation_review_application.review_manual",
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
            .await?;

        sync_obligation_review_state_with_observation(
            &self.pool,
            &obligation,
            &observation.observation_id,
        )
        .await?;

        Ok(obligation)
    }
}

#[derive(Debug, Error)]
pub enum ObligationReviewApplicationError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}

#[derive(Clone)]
pub struct RelationshipReviewApplicationService {
    pool: PgPool,
}

impl RelationshipReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
    ) -> Result<Relationship, RelationshipReviewApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "relationship_id": relationship_id,
                        "review_state": review_state.as_str(),
                        "operation": "relationship_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("relationship://{relationship_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "relationship_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let relationship = RelationshipGraphCoordinator::new(self.pool.clone())
            .set_review_state_with_observation(
                relationship_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "relationship_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        sync_relationship_review_state_with_observation(
            &self.pool,
            &relationship,
            &observation.observation_id,
        )
        .await?;

        Ok(relationship)
    }
}

#[derive(Debug, Error)]
pub enum RelationshipReviewApplicationError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    RelationshipGraph(#[from] RelationshipGraphCoordinatorError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}

#[derive(Clone)]
pub struct TaskCandidateReviewApplicationService {
    pool: PgPool,
}

impl TaskCandidateReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        command: &TaskCandidateReviewCommand,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateReviewApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "task_candidate_id": command.task_candidate_id,
                        "command_id": command.command_id,
                        "review_state": command.review_state.as_str(),
                        "actor_id": command.actor_id,
                        "operation": "task_candidate_review",
                    }),
                    format!(
                        "task-candidate://{}/review/{}",
                        command.task_candidate_id, command.command_id
                    ),
                )
                .provenance(json!({
                    "captured_by": "task_candidate_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        self.review_with_observation(
            command,
            &observation.observation_id,
            json!({
                "captured_by": "task_candidate_review_application.review_manual",
                "operation": "review_manual",
            }),
        )
        .await
    }

    pub async fn review_with_observation(
        &self,
        command: &TaskCandidateReviewCommand,
        observation_id: &str,
        metadata: serde_json::Value,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateReviewApplicationError> {
        let mut transaction = self.pool.begin().await?;
        let (result, candidate) = TaskCandidateStore::set_review_state_in_transaction(
            &mut transaction,
            command,
            Some(observation_id),
            Some(metadata),
        )
        .await?;
        sync_obligation_candidate_in_transaction(
            &mut transaction,
            &result.task_candidate_id,
            &candidate,
            result.review_state,
        )
        .await?;
        if result.review_state != TaskCandidateReviewState::UserConfirmed {
            TaskCandidateStore::delete_task_for_candidate_in_transaction(
                &mut transaction,
                &result.task_candidate_id,
            )
            .await?;
        }
        sync_task_candidate_review_state_in_transaction(
            &mut transaction,
            &result.task_candidate_id,
            &candidate,
            result.review_state,
        )
        .await?;
        transaction.commit().await?;

        Ok(result)
    }

    pub async fn apply_review_event(
        &self,
        event: &makosh_events_api::EventEnvelope,
    ) -> Result<(), TaskCandidateReviewApplicationError> {
        let mut transaction = self.pool.begin().await?;
        let (task_candidate_id, review_state, candidate) =
            TaskCandidateStore::apply_review_event_in_transaction(&mut transaction, event).await?;
        sync_obligation_candidate_in_transaction(
            &mut transaction,
            &task_candidate_id,
            &candidate,
            review_state,
        )
        .await?;
        if review_state != TaskCandidateReviewState::UserConfirmed {
            TaskCandidateStore::delete_task_for_candidate_in_transaction(
                &mut transaction,
                &task_candidate_id,
            )
            .await?;
        }
        sync_task_candidate_review_state_in_transaction(
            &mut transaction,
            &task_candidate_id,
            &candidate,
            review_state,
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum TaskCandidateReviewApplicationError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateError),

    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),

    #[error(transparent)]
    TaskCore(#[from] TaskCoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}

async fn sync_obligation_candidate_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    candidate: &StoredCandidateRow,
    review_state: TaskCandidateReviewState,
) -> Result<(), TaskCandidateReviewApplicationError> {
    if candidate.candidate_kind != TASK_CANDIDATE_KIND_OBLIGATION_TASK {
        return Ok(());
    }

    match review_state {
        TaskCandidateReviewState::UserConfirmed => {
            let observation_id = candidate.observation_id.as_deref().ok_or_else(|| {
                TaskCandidateError::ObservationRequired(task_candidate_id.to_owned())
            })?;
            let obligation_candidate =
                obligation_candidate_from_metadata(&candidate.candidate_metadata)?;
            let mut obligation = NewObligation::new(
                map_obligation_entity_kind(obligation_candidate.obligated_entity_kind),
                obligation_candidate.obligated_entity_id.clone(),
                obligation_candidate.statement.clone(),
                obligation_candidate.confidence,
                ObligationReviewState::UserConfirmed,
            )
            .metadata(json!({
                "task_candidate_id": task_candidate_id,
                "candidate_kind": TASK_CANDIDATE_KIND_OBLIGATION_TASK,
            }));
            if let (Some(kind), Some(entity_id)) = (
                obligation_candidate.beneficiary_entity_kind,
                obligation_candidate.beneficiary_entity_id.as_deref(),
            ) {
                obligation = obligation.beneficiary(map_obligation_entity_kind(kind), entity_id);
            }
            if let Some(condition) = obligation_candidate.condition.as_deref() {
                obligation = obligation.condition(condition);
            }

            let evidence = [NewObligationEvidence::observation(observation_id)
                .quote(obligation_candidate.quote.clone())
                .confidence(obligation_candidate.confidence)
                .metadata(json!({ "task_candidate_id": task_candidate_id }))];
            let stored = ObligationReviewPort::upsert_with_evidence_in_transaction(
                transaction,
                &obligation,
                &evidence,
            )
            .await?;
            ObligationTaskLinkStore::link_fulfillment_task_in_transaction(
                transaction,
                &stored.obligation_id,
                &task_id_from_candidate(task_candidate_id),
            )
            .await?;
        }
        TaskCandidateReviewState::Suggested | TaskCandidateReviewState::UserRejected => {
            let linked_obligation_ids =
                ObligationTaskLinkStore::obligation_ids_for_candidate_in_transaction(
                    transaction,
                    task_candidate_id,
                )
                .await?;
            let obligation_review_state = match review_state {
                TaskCandidateReviewState::Suggested => ObligationReviewState::Suggested,
                TaskCandidateReviewState::UserRejected => ObligationReviewState::UserRejected,
                TaskCandidateReviewState::UserConfirmed => unreachable!(),
            };
            for obligation_id in linked_obligation_ids {
                ObligationReviewPort::set_review_state_in_transaction(
                    transaction,
                    &obligation_id,
                    obligation_review_state,
                    candidate.observation_id.as_deref(),
                    Some(json!({
                        "task_candidate_id": task_candidate_id,
                        "review_state": review_state.as_str(),
                    })),
                )
                .await?;
            }
        }
    }

    Ok(())
}

fn obligation_candidate_from_metadata(
    metadata: &serde_json::Value,
) -> Result<ObligationCandidate, TaskCandidateReviewApplicationError> {
    let candidate = metadata
        .get(OBLIGATION_CANDIDATE_METADATA_KEY)
        .cloned()
        .ok_or_else(|| {
            TaskCandidateError::InvalidCandidateMetadata(
                OBLIGATION_CANDIDATE_METADATA_KEY.to_owned(),
            )
        })?;
    Ok(serde_json::from_value(candidate).map_err(TaskCandidateError::from)?)
}

fn map_obligation_entity_kind(
    value: crate::engines::obligation::models::ObligationEntityKind,
) -> ObligationEntityKind {
    match value {
        crate::engines::obligation::models::ObligationEntityKind::Persona => {
            ObligationEntityKind::Persona
        }
        crate::engines::obligation::models::ObligationEntityKind::Organization => {
            ObligationEntityKind::Organization
        }
        crate::engines::obligation::models::ObligationEntityKind::Project => {
            ObligationEntityKind::Project
        }
        crate::engines::obligation::models::ObligationEntityKind::Communication => {
            ObligationEntityKind::Communication
        }
        crate::engines::obligation::models::ObligationEntityKind::Document => {
            ObligationEntityKind::Document
        }
        crate::engines::obligation::models::ObligationEntityKind::Task => {
            ObligationEntityKind::Task
        }
        crate::engines::obligation::models::ObligationEntityKind::Event => {
            ObligationEntityKind::Event
        }
        crate::engines::obligation::models::ObligationEntityKind::Decision => {
            ObligationEntityKind::Decision
        }
        crate::engines::obligation::models::ObligationEntityKind::Obligation => {
            ObligationEntityKind::Obligation
        }
        crate::engines::obligation::models::ObligationEntityKind::Knowledge => {
            ObligationEntityKind::Knowledge
        }
    }
}
use crate::domains::obligations::models::states::ObligationReviewState;
