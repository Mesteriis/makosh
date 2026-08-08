use makosh_events_api::{EventEnvelope, StoredEventEnvelope};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::application::relationship_graph::{
    RelationshipGraphCoordinator, RelationshipGraphCoordinatorError,
};
use crate::domains::decisions::models::decision::NewDecision;
use crate::domains::decisions::ports::DecisionReviewPortError;
use crate::domains::projects::link_reviews::errors::ProjectLinkReviewError;
use crate::domains::projects::link_reviews::models::{
    ProjectLinkReviewState, ProjectLinkTargetKind,
};
use crate::domains::relationships::models::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewState,
};
use crate::workflows::review_mirror::{
    ReviewMirrorError, relationship::ensure_relationship_review_item,
    relationship::sync_relationship_review_state_in_transaction,
};
use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

pub const PROJECT_LINK_REVIEW_EFFECTS_CONSUMER: &str = "project_link_review_effects";
pub const PROJECT_LINK_REVIEW_EVENT_TYPE: &str = "project.link_review_state_changed";

#[derive(Debug, Error)]
pub enum ProjectLinkReviewEffectsWorkflowError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Decision(#[from] DecisionReviewPortError),

    #[error(transparent)]
    RelationshipGraph(#[from] RelationshipGraphCoordinatorError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),

    #[error(transparent)]
    ProjectLinkReview(#[from] ProjectLinkReviewError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),

    #[error("event payload field {field} is invalid: {value}")]
    InvalidPayloadField { field: &'static str, value: String },
}

pub async fn project_link_review_effect_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_link_review_effect(&pool, &event.event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn project_link_review_effect(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ProjectLinkReviewEffectsWorkflowError> {
    project_link_review_effect_inner(pool, event).await
}

async fn project_link_review_effect_inner(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ProjectLinkReviewEffectsWorkflowError> {
    if event.event_type != PROJECT_LINK_REVIEW_EVENT_TYPE {
        return Ok(());
    }

    let review = ProjectLinkReviewEffect::from_payload(&event.payload)?;
    let observation = capture_review_observation(pool, event, &review).await?;
    let relationship =
        materialize_relationship(pool, event, &review, &observation.observation_id).await?;
    sync_relationship_review_item(pool, &relationship, &observation.observation_id).await?;

    if review.review_state == ProjectLinkReviewState::UserConfirmed {
        let _ = materialize_decision(pool, event, &review, &observation.observation_id).await?;
    }

    Ok(())
}

async fn capture_review_observation(
    pool: &PgPool,
    event: &EventEnvelope,
    review: &ProjectLinkReviewEffect,
) -> Result<makosh_observations_api::models::Observation, ProjectLinkReviewEffectsWorkflowError> {
    Ok(ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PROJECT_LINK_REVIEW",
                ObservationOriginKind::LocalRuntime,
                event.occurred_at,
                json!({
                    "project_id": review.project_id,
                    "target_kind": review.target_kind.as_str(),
                    "target_id": review.target_id,
                    "review_state": review.review_state.as_str(),
                }),
                format!(
                    "project://{}/link-review/{}/{}",
                    review.project_id,
                    review.target_kind.as_str(),
                    review.target_id
                ),
            )
            .confidence(review.confidence())
            .provenance(json!({
                "captured_by": "project_link_review_effects",
                "event_id": event.event_id,
            })),
        )
        .await?)
}

async fn materialize_relationship(
    pool: &PgPool,
    event: &EventEnvelope,
    review: &ProjectLinkReviewEffect,
    observation_id: &str,
) -> Result<
    crate::domains::relationships::models::Relationship,
    ProjectLinkReviewEffectsWorkflowError,
> {
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Project,
        source_entity_id: review.project_id.clone(),
        target_entity_kind: review.relationship_target_kind(),
        target_entity_id: review.target_id.clone(),
        relationship_type: review.relationship_type().to_owned(),
        trust_score: review.confidence(),
        strength_score: review.confidence(),
        confidence: review.confidence(),
        review_state: review.relationship_review_state(),
        valid_from: None,
        valid_to: None,
        metadata: json!({
            "compatibility_table": "project_link_reviews",
            "project_link_review_event_id": event.event_id,
            "project_id": review.project_id,
            "target_kind": review.target_kind.as_str(),
            "target_id": review.target_id,
        }),
    };
    let evidence = NewRelationshipEvidence::observation(observation_id.to_owned())
        .excerpt(review.evidence_text())
        .metadata(json!({
            "compatibility_table": "project_link_reviews",
            "event_id": event.event_id,
        }));

    Ok(RelationshipGraphCoordinator::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?)
}

async fn sync_relationship_review_item(
    pool: &PgPool,
    relationship: &crate::domains::relationships::models::Relationship,
    observation_id: &str,
) -> Result<(), ProjectLinkReviewEffectsWorkflowError> {
    let _ = ensure_relationship_review_item(
        pool,
        crate::workflows::review_mirror::relationship::RelationshipReviewInput {
            relationship_id: &relationship.relationship_id,
            relationship_type: &relationship.relationship_type,
            source_entity_kind: relationship.source_entity_kind.as_str(),
            source_entity_id: &relationship.source_entity_id,
            target_entity_kind: relationship.target_entity_kind.as_str(),
            target_entity_id: &relationship.target_entity_id,
            confidence: relationship.confidence,
            summary: None,
            observation_id,
        },
    )
    .await?;

    let mut transaction = pool.begin().await?;
    sync_relationship_review_state_in_transaction(&mut transaction, relationship).await?;
    transaction.commit().await?;
    Ok(())
}

async fn materialize_decision(
    pool: &PgPool,
    event: &EventEnvelope,
    review: &ProjectLinkReviewEffect,
    observation_id: &str,
) -> Result<
    crate::domains::decisions::models::decision::Decision,
    ProjectLinkReviewEffectsWorkflowError,
> {
    let decision = NewDecision::new(
        "Project link review confirmed",
        format!(
            "User confirmed a {} link candidate for this project.",
            review.target_kind.as_str()
        ),
        1.0,
        DecisionReviewState::UserConfirmed,
    )
    .decided_at(event.occurred_at)
    .metadata(json!({
        "project_link_review_event_id": event.event_id,
        "project_id": review.project_id,
        "target_kind": review.target_kind.as_str(),
        "target_id": review.target_id,
    }));
    let evidence = NewDecisionEvidence::observation(observation_id.to_owned())
        .quote(review.evidence_text())
        .metadata(json!({
            "compatibility_table": "project_link_reviews",
            "event_id": event.event_id,
        }));
    let impacted_entities = [
        NewDecisionImpactedEntity::new(DecisionEntityKind::Project, review.project_id.clone())
            .impact_type("project_link_review"),
        NewDecisionImpactedEntity::new(review.decision_target_kind(), review.target_id.clone())
            .impact_type("project_link_review"),
    ];

    Ok(
        crate::domains::decisions::ports::DecisionReviewPort::new(pool.clone())
            .upsert_with_evidence(&decision, &[evidence], &impacted_entities)
            .await?,
    )
}

struct ProjectLinkReviewEffect {
    project_id: String,
    target_kind: ProjectLinkTargetKind,
    target_id: String,
    review_state: ProjectLinkReviewState,
}

impl ProjectLinkReviewEffect {
    fn from_payload(payload: &Value) -> Result<Self, ProjectLinkReviewEffectsWorkflowError> {
        Ok(Self {
            project_id: required_string(payload, "project_id")?.to_owned(),
            target_kind: ProjectLinkTargetKind::parse(required_string(payload, "target_kind")?)?,
            target_id: required_string(payload, "target_id")?.to_owned(),
            review_state: ProjectLinkReviewState::parse(required_string(payload, "review_state")?)?,
        })
    }

    fn relationship_target_kind(&self) -> RelationshipEntityKind {
        match self.target_kind {
            ProjectLinkTargetKind::Message => RelationshipEntityKind::Communication,
            ProjectLinkTargetKind::Document => RelationshipEntityKind::Document,
        }
    }

    fn decision_target_kind(&self) -> DecisionEntityKind {
        match self.target_kind {
            ProjectLinkTargetKind::Message => DecisionEntityKind::Communication,
            ProjectLinkTargetKind::Document => DecisionEntityKind::Document,
        }
    }

    fn relationship_type(&self) -> &'static str {
        match self.target_kind {
            ProjectLinkTargetKind::Message => "project_has_message",
            ProjectLinkTargetKind::Document => "project_has_document",
        }
    }

    fn relationship_review_state(&self) -> RelationshipReviewState {
        match self.review_state {
            ProjectLinkReviewState::Suggested => RelationshipReviewState::Suggested,
            ProjectLinkReviewState::UserConfirmed => RelationshipReviewState::UserConfirmed,
            ProjectLinkReviewState::UserRejected => RelationshipReviewState::UserRejected,
        }
    }

    fn confidence(&self) -> f64 {
        match self.review_state {
            ProjectLinkReviewState::Suggested => 0.65,
            ProjectLinkReviewState::UserConfirmed => 1.0,
            ProjectLinkReviewState::UserRejected => 0.0,
        }
    }

    fn evidence_text(&self) -> &'static str {
        match (self.target_kind, self.review_state) {
            (ProjectLinkTargetKind::Message, ProjectLinkReviewState::Suggested) => {
                "User reset message link review for project."
            }
            (ProjectLinkTargetKind::Document, ProjectLinkReviewState::Suggested) => {
                "User reset document link review for project."
            }
            (ProjectLinkTargetKind::Message, ProjectLinkReviewState::UserConfirmed) => {
                "User confirmed message link to project."
            }
            (ProjectLinkTargetKind::Document, ProjectLinkReviewState::UserConfirmed) => {
                "User confirmed document link to project."
            }
            (ProjectLinkTargetKind::Message, ProjectLinkReviewState::UserRejected) => {
                "User rejected message link review for project."
            }
            (ProjectLinkTargetKind::Document, ProjectLinkReviewState::UserRejected) => {
                "User rejected document link review for project."
            }
        }
    }
}

fn required_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, ProjectLinkReviewEffectsWorkflowError> {
    let raw =
        payload
            .get(field)
            .ok_or(ProjectLinkReviewEffectsWorkflowError::MissingPayloadField(
                field,
            ))?;
    raw.as_str()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(
            || ProjectLinkReviewEffectsWorkflowError::InvalidPayloadField {
                field,
                value: raw.to_string(),
            },
        )
}
use crate::domains::decisions::models::entity_kind::DecisionEntityKind;
use crate::domains::decisions::models::evidence::NewDecisionEvidence;
use crate::domains::decisions::models::impacted_entity::NewDecisionImpactedEntity;
use crate::domains::decisions::models::states::DecisionReviewState;
