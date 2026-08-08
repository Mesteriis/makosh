use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::domains::graph::core::models::{
    GraphEvidenceSourceKind, GraphReviewState, NewGraphEdge, NewGraphEvidence, NewGraphNode,
    RelationshipType,
};
use crate::domains::graph::ports::GraphProjectionPort;
use crate::domains::relationships::models::{
    NewRelationship, NewRelationshipEvidence, Relationship, RelationshipEntityKind,
    RelationshipReviewState,
};
use crate::platform::graph::GraphNodeKind;
use makosh_relationships_postgres::RelationshipPostgresQuery;

/// Coordinates two bounded contexts in one PostgreSQL transaction. Relationship
/// persistence remains owned by Relationships; Graph owns the materialized
/// nodes and edge.
#[derive(Clone)]
pub struct RelationshipGraphCoordinator {
    pool: PgPool,
}

impl RelationshipGraphCoordinator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        relationship: &NewRelationship,
        evidence: &[NewRelationshipEvidence],
    ) -> Result<Relationship, RelationshipGraphCoordinatorError> {
        let mut transaction = self.pool.begin().await?;
        let stored =
            Self::upsert_with_evidence_in_transaction(&mut transaction, relationship, evidence)
                .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relationship: &NewRelationship,
        evidence: &[NewRelationshipEvidence],
    ) -> Result<Relationship, RelationshipGraphCoordinatorError> {
        let stored = RelationshipPostgresQuery::upsert_in_transaction(
            transaction,
            &to_api_relationship(relationship),
            &evidence.iter().map(to_api_evidence).collect::<Vec<_>>(),
        )
        .await
        .map_err(|error| RelationshipGraphCoordinatorError::RelationshipWrite(error.to_string()))?;
        let stored = from_api_relationship(stored)
            .map_err(RelationshipGraphCoordinatorError::RelationshipWrite)?;
        materialize_relationship_graph_in_transaction(transaction, &stored, evidence).await?;
        Ok(stored)
    }

    pub async fn set_review_state_with_observation(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Relationship, RelationshipGraphCoordinatorError> {
        let mut transaction = self.pool.begin().await?;
        let relationship = RelationshipPostgresQuery::set_review_in_transaction(
            &mut transaction,
            relationship_id,
            review_state.as_str(),
            observation_id,
            metadata.clone(),
        )
        .await
        .map_err(|error| RelationshipGraphCoordinatorError::RelationshipWrite(error.to_string()))?;
        let relationship = from_api_relationship(relationship)
            .map_err(RelationshipGraphCoordinatorError::RelationshipWrite)?;
        materialize_relationship_graph_review_in_transaction(
            &mut transaction,
            &relationship,
            observation_id,
            metadata.as_ref(),
        )
        .await?;
        transaction.commit().await?;
        Ok(relationship)
    }
}

async fn materialize_relationship_graph_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &Relationship,
    evidence: &[NewRelationshipEvidence],
) -> Result<(), RelationshipGraphCoordinatorError> {
    let graph_evidence = relationship_graph_evidence(relationship, evidence);
    materialize_relationship_graph_with_evidence_in_transaction(
        transaction,
        relationship,
        std::slice::from_ref(&graph_evidence),
    )
    .await
}

async fn materialize_relationship_graph_review_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &Relationship,
    observation_id: Option<&str>,
    metadata: Option<&Value>,
) -> Result<(), RelationshipGraphCoordinatorError> {
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Relationship,
        relationship.relationship_id.clone(),
    )
    .metadata(json!({
        "relationship_id": relationship.relationship_id,
        "relationship_type": relationship.relationship_type,
        "review_state": relationship.review_state.as_str(),
    }));
    if let Some(observation_id) = observation_id {
        graph_evidence = graph_evidence.observation_id(observation_id.to_owned());
    }
    if let Some(metadata) = metadata {
        graph_evidence = graph_evidence.metadata(json!({
            "relationship_id": relationship.relationship_id,
            "relationship_type": relationship.relationship_type,
            "review_state": relationship.review_state.as_str(),
            "review_transition": metadata.clone(),
        }));
    }

    materialize_relationship_graph_with_evidence_in_transaction(
        transaction,
        relationship,
        std::slice::from_ref(&graph_evidence),
    )
    .await
}

async fn materialize_relationship_graph_with_evidence_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &Relationship,
    graph_evidence: &[NewGraphEvidence],
) -> Result<(), RelationshipGraphCoordinatorError> {
    let source_node = GraphProjectionPort::upsert_node_in_transaction(
        transaction,
        &NewGraphNode::new(
            graph_node_kind(relationship.source_entity_kind),
            relationship.source_entity_id.clone(),
            relationship.source_entity_id.clone(),
        )
        .properties(json!({
            "entity_kind": relationship.source_entity_kind.as_str(),
            "entity_id": relationship.source_entity_id,
        })),
    )
    .await?;
    let target_node = GraphProjectionPort::upsert_node_in_transaction(
        transaction,
        &NewGraphNode::new(
            graph_node_kind(relationship.target_entity_kind),
            relationship.target_entity_id.clone(),
            relationship.target_entity_id.clone(),
        )
        .properties(json!({
            "entity_kind": relationship.target_entity_kind.as_str(),
            "entity_id": relationship.target_entity_id,
        })),
    )
    .await?;

    GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
        transaction,
        &NewGraphEdge::new(
            source_node.node_id,
            target_node.node_id,
            RelationshipType::EntityRelationship,
            relationship.confidence,
            graph_review_state(relationship.review_state),
        )
        .properties(json!({
            "relationship_id": relationship.relationship_id,
            "relationship_type": relationship.relationship_type,
            "source_entity_kind": relationship.source_entity_kind.as_str(),
            "source_entity_id": relationship.source_entity_id,
            "target_entity_kind": relationship.target_entity_kind.as_str(),
            "target_entity_id": relationship.target_entity_id,
            "trust_score": relationship.trust_score,
            "strength_score": relationship.strength_score,
        })),
        graph_evidence,
    )
    .await?;

    Ok(())
}

fn graph_node_kind(entity_kind: RelationshipEntityKind) -> GraphNodeKind {
    match entity_kind {
        RelationshipEntityKind::Persona => GraphNodeKind::Persona,
        RelationshipEntityKind::Organization => GraphNodeKind::Organization,
        RelationshipEntityKind::Project => GraphNodeKind::Project,
        RelationshipEntityKind::Communication => GraphNodeKind::Message,
        RelationshipEntityKind::Document => GraphNodeKind::Document,
        RelationshipEntityKind::Task => GraphNodeKind::Task,
        RelationshipEntityKind::Event => GraphNodeKind::Event,
        RelationshipEntityKind::Decision => GraphNodeKind::Decision,
        RelationshipEntityKind::Obligation => GraphNodeKind::Obligation,
        RelationshipEntityKind::Knowledge => GraphNodeKind::Knowledge,
    }
}

fn graph_review_state(review_state: RelationshipReviewState) -> GraphReviewState {
    match review_state {
        RelationshipReviewState::Suggested => GraphReviewState::Suggested,
        RelationshipReviewState::SystemAccepted => GraphReviewState::SystemAccepted,
        RelationshipReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        RelationshipReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

fn relationship_graph_evidence(
    relationship: &Relationship,
    evidence: &[NewRelationshipEvidence],
) -> NewGraphEvidence {
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Relationship,
        relationship.relationship_id.clone(),
    )
    .metadata(json!({
        "relationship_id": relationship.relationship_id,
        "relationship_type": relationship.relationship_type,
    }));
    if let Some(item) = evidence.first() {
        if let Some(excerpt) = item.excerpt.clone() {
            graph_evidence = graph_evidence.excerpt(excerpt);
        }
        if let Some(observation_id) = item.observation_id.clone() {
            graph_evidence = graph_evidence.observation_id(observation_id);
        }
    }
    graph_evidence
}

#[derive(Debug, Error)]
pub enum RelationshipGraphCoordinatorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Graph(#[from] crate::domains::graph::ports::GraphProjectionPortError),

    #[error("relationship write failed: {0}")]
    RelationshipWrite(String),
}

fn to_api_relationship(
    relationship: &NewRelationship,
) -> makosh_relationships_api::RelationshipUpsert {
    makosh_relationships_api::RelationshipUpsert {
        relationship_id: crate::domains::relationships::ids::relationship_id(
            relationship.source_entity_kind,
            &relationship.source_entity_id,
            &relationship.relationship_type,
            relationship.target_entity_kind,
            &relationship.target_entity_id,
        ),
        source_entity_kind: api_entity_kind(relationship.source_entity_kind),
        source_entity_id: relationship.source_entity_id.clone(),
        target_entity_kind: api_entity_kind(relationship.target_entity_kind),
        target_entity_id: relationship.target_entity_id.clone(),
        relationship_type: relationship.relationship_type.clone(),
        trust_score: relationship.trust_score,
        strength_score: relationship.strength_score,
        confidence: relationship.confidence,
        review_state: api_review_state(relationship.review_state),
        valid_from: relationship.valid_from,
        valid_to: relationship.valid_to,
        metadata: relationship.metadata.clone(),
    }
}

fn to_api_evidence(
    evidence: &NewRelationshipEvidence,
) -> makosh_relationships_api::RelationshipEvidence {
    makosh_relationships_api::RelationshipEvidence {
        source_kind: match evidence.source_kind.as_str() {
            "observation" => makosh_relationships_api::RelationshipEvidenceSourceKind::Observation,
            "communication" => {
                makosh_relationships_api::RelationshipEvidenceSourceKind::Communication
            }
            "document" => makosh_relationships_api::RelationshipEvidenceSourceKind::Document,
            "event" => makosh_relationships_api::RelationshipEvidenceSourceKind::Event,
            "memory" => makosh_relationships_api::RelationshipEvidenceSourceKind::Memory,
            "knowledge" => makosh_relationships_api::RelationshipEvidenceSourceKind::Knowledge,
            "decision" => makosh_relationships_api::RelationshipEvidenceSourceKind::Decision,
            "obligation" => makosh_relationships_api::RelationshipEvidenceSourceKind::Obligation,
            "task" => makosh_relationships_api::RelationshipEvidenceSourceKind::Task,
            "project" => makosh_relationships_api::RelationshipEvidenceSourceKind::Project,
            "organization" => {
                makosh_relationships_api::RelationshipEvidenceSourceKind::Organization
            }
            _ => makosh_relationships_api::RelationshipEvidenceSourceKind::Persona,
        },
        source_id: evidence.source_id.clone(),
        observation_id: evidence.observation_id.clone(),
        excerpt: evidence.excerpt.clone(),
        metadata: evidence.metadata.clone(),
    }
}

fn api_entity_kind(
    kind: RelationshipEntityKind,
) -> makosh_relationships_api::RelationshipEntityKind {
    match kind.as_str() {
        "persona" => makosh_relationships_api::RelationshipEntityKind::Persona,
        "organization" => makosh_relationships_api::RelationshipEntityKind::Organization,
        "project" => makosh_relationships_api::RelationshipEntityKind::Project,
        "communication" => makosh_relationships_api::RelationshipEntityKind::Communication,
        "document" => makosh_relationships_api::RelationshipEntityKind::Document,
        "task" => makosh_relationships_api::RelationshipEntityKind::Task,
        "event" => makosh_relationships_api::RelationshipEntityKind::Event,
        "decision" => makosh_relationships_api::RelationshipEntityKind::Decision,
        "obligation" => makosh_relationships_api::RelationshipEntityKind::Obligation,
        _ => makosh_relationships_api::RelationshipEntityKind::Knowledge,
    }
}

fn api_review_state(
    state: RelationshipReviewState,
) -> makosh_relationships_api::RelationshipReviewState {
    match state.as_str() {
        "system_accepted" => makosh_relationships_api::RelationshipReviewState::SystemAccepted,
        "user_confirmed" => makosh_relationships_api::RelationshipReviewState::UserConfirmed,
        "user_rejected" => makosh_relationships_api::RelationshipReviewState::UserRejected,
        _ => makosh_relationships_api::RelationshipReviewState::Suggested,
    }
}

fn from_api_relationship(
    relationship: makosh_relationships_api::RelationshipRead,
) -> Result<Relationship, String> {
    Ok(Relationship {
        relationship_id: relationship.relationship_id,
        source_entity_kind: RelationshipEntityKind::parse(relationship.source_entity_kind)
            .map_err(|e| e.to_string())?,
        source_entity_id: relationship.source_entity_id,
        target_entity_kind: RelationshipEntityKind::parse(relationship.target_entity_kind)
            .map_err(|e| e.to_string())?,
        target_entity_id: relationship.target_entity_id,
        relationship_type: relationship.relationship_type,
        trust_score: relationship.trust_score,
        strength_score: relationship.strength_score,
        confidence: relationship.confidence,
        review_state: RelationshipReviewState::parse(relationship.review_state)
            .map_err(|e| e.to_string())?,
        valid_from: relationship.valid_from,
        valid_to: relationship.valid_to,
        metadata: relationship.metadata,
        created_at: relationship.created_at,
        updated_at: relationship.updated_at,
    })
}
