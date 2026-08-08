use makosh_relationships_api::{
    RelationshipRead, RelationshipReviewError, RelationshipReviewFuture, RelationshipReviewPort,
    RelationshipReviewRequest,
};
use sqlx::postgres::PgPool;

use crate::application::review_transitions::RelationshipReviewApplicationService;
use crate::domains::relationships::models::RelationshipReviewState;

pub struct RelationshipReviewAdapter {
    service: RelationshipReviewApplicationService,
}

impl RelationshipReviewAdapter {
    pub fn from_pool(pool: PgPool) -> Self {
        Self {
            service: RelationshipReviewApplicationService::new(pool),
        }
    }
}

impl RelationshipReviewPort for RelationshipReviewAdapter {
    fn review<'a>(
        &'a self,
        relationship_id: &'a str,
        request: RelationshipReviewRequest,
    ) -> RelationshipReviewFuture<'a> {
        Box::pin(async move {
            if relationship_id.trim().is_empty() {
                return Err(RelationshipReviewError::InvalidReview(
                    "relationship_id must not be empty",
                ));
            }
            let state = RelationshipReviewState::parse(&request.review_state)
                .map_err(|_| RelationshipReviewError::InvalidReview("review_state is invalid"))?;
            let relationship = self
                .service
                .review_manual(relationship_id, state)
                .await
                .map_err(|error| RelationshipReviewError::Failed(error.to_string()))?;
            Ok(to_read(relationship))
        })
    }
}

fn to_read(relationship: crate::domains::relationships::models::Relationship) -> RelationshipRead {
    RelationshipRead {
        relationship_id: relationship.relationship_id,
        source_entity_kind: relationship.source_entity_kind.as_str().to_owned(),
        source_entity_id: relationship.source_entity_id,
        target_entity_kind: relationship.target_entity_kind.as_str().to_owned(),
        target_entity_id: relationship.target_entity_id,
        relationship_type: relationship.relationship_type,
        trust_score: relationship.trust_score,
        strength_score: relationship.strength_score,
        confidence: relationship.confidence,
        review_state: relationship.review_state.as_str().to_owned(),
        valid_from: relationship.valid_from,
        valid_to: relationship.valid_to,
        metadata: relationship.metadata,
        created_at: relationship.created_at,
        updated_at: relationship.updated_at,
    }
}
