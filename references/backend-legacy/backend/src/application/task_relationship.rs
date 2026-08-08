use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::domains::relationships::models::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewState,
};
use crate::domains::tasks::core::errors::TaskCoreError;
use crate::domains::tasks::core::relations::{TaskRelation, TaskRelationStore};
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

use super::relationship_graph::{RelationshipGraphCoordinator, RelationshipGraphCoordinatorError};

const TASK_RELATIONSHIP_EVIDENCE_EXCERPT: &str =
    "Task relation was recorded through compatibility task relation data.";

#[derive(Clone)]
pub struct TaskRelationshipApplicationService {
    pool: PgPool,
}

impl TaskRelationshipApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_manual(
        &self,
        task_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
    ) -> Result<TaskRelation, TaskRelationshipApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "TASK_MUTATION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "task_id": task_id,
                        "entity_type": entity_type,
                        "entity_id": entity_id,
                        "relation_type": relation_type,
                    }),
                    format!("task://{task_id}/relation"),
                )
                .provenance(json!({
                    "captured_by": "task_relationship_application.add_manual",
                    "operation": "add_manual",
                })),
            )
            .await?;

        self.link(
            task_id,
            entity_type,
            entity_id,
            relation_type,
            &format!("observation:{}", observation.observation_id),
        )
        .await
    }

    pub async fn link(
        &self,
        task_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
        source: &str,
    ) -> Result<TaskRelation, TaskRelationshipApplicationError> {
        let mut transaction = self.pool.begin().await?;
        let relation = TaskRelationStore::link_in_transaction(
            &mut transaction,
            task_id,
            entity_type,
            entity_id,
            relation_type,
            source,
        )
        .await?;
        materialize_relationship_in_transaction(&mut transaction, &relation).await?;
        transaction.commit().await?;
        Ok(relation)
    }
}

async fn materialize_relationship_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relation: &TaskRelation,
) -> Result<(), TaskRelationshipApplicationError> {
    let Ok(target_entity_kind) = RelationshipEntityKind::parse(&relation.entity_type) else {
        return Ok(());
    };
    let observation_id = if let Some(observation_id) = relation
        .source
        .strip_prefix("observation:")
        .filter(|value| !value.is_empty())
    {
        observation_id.to_owned()
    } else {
        ObservationStore::capture_in_transaction(transaction, &relation_observation(relation))
            .await?
            .observation_id
    };
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Task,
        source_entity_id: relation.task_id.clone(),
        target_entity_kind,
        target_entity_id: relation.entity_id.clone(),
        relationship_type: relation.relation_type.clone(),
        trust_score: relation.confidence,
        strength_score: relation.confidence,
        confidence: relation.confidence,
        review_state: RelationshipReviewState::UserConfirmed,
        valid_from: None,
        valid_to: None,
        metadata: json!({
            "compatibility_table": "task_relations",
            "compatibility_record_id": relation.id,
            "task_id": relation.task_id,
            "entity_type": relation.entity_type,
            "entity_id": relation.entity_id,
            "source": relation.source,
        }),
    };
    let evidence = NewRelationshipEvidence::observation(observation_id)
        .excerpt(TASK_RELATIONSHIP_EVIDENCE_EXCERPT)
        .metadata(json!({
            "compatibility_table": "task_relations",
            "compatibility_record_id": relation.id,
            "task_id": relation.task_id,
            "entity_type": relation.entity_type,
            "entity_id": relation.entity_id,
        }));
    RelationshipGraphCoordinator::upsert_with_evidence_in_transaction(
        transaction,
        &relationship,
        &[evidence],
    )
    .await?;
    Ok(())
}

fn relation_observation(relation: &TaskRelation) -> NewObservation {
    let origin_kind = ObservationOriginKind::parse(&relation.source)
        .unwrap_or(ObservationOriginKind::LocalRuntime);
    NewObservation::new(
        "TASK_MUTATION",
        origin_kind,
        relation.created_at,
        json!({
            "task_id": relation.task_id,
            "entity_type": relation.entity_type,
            "entity_id": relation.entity_id,
            "relation_type": relation.relation_type,
            "source": relation.source,
            "compatibility_record_id": relation.id,
        }),
        format!("task://{}/relation/{}", relation.task_id, relation.id),
    )
    .provenance(json!({
        "captured_by": "task_relationship_application",
        "operation": "materialize_relationship",
        "source": relation.source,
    }))
}

#[derive(Debug, Error)]
pub enum TaskRelationshipApplicationError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Task(#[from] TaskCoreError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    RelationshipGraph(#[from] RelationshipGraphCoordinatorError),
}
