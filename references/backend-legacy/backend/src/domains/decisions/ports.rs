use makosh_decisions_api::{
    DecisionEvidence, DecisionImpactedEntity, DecisionUpsert, DecisionWritePort,
};
use makosh_decisions_postgres::DecisionPostgresReadQuery;
use sqlx::PgPool;

use super::errors::DecisionStoreError;
use super::models::decision::{Decision, NewDecision};
use super::models::evidence::NewDecisionEvidence;
use super::models::impacted_entity::NewDecisionImpactedEntity;
use super::models::states::DecisionReviewState;
use super::store::DecisionStore;

/// Workflow-facing decision command port. SQL ownership remains in Decisions.
#[derive(Clone)]
pub struct DecisionReviewPort {
    writer: DecisionPostgresReadQuery,
    store: DecisionStore,
}

impl DecisionReviewPort {
    pub fn new(pool: PgPool) -> Self {
        Self {
            writer: DecisionPostgresReadQuery::new(pool.clone()),
            store: DecisionStore::new(pool.clone()),
        }
    }

    pub async fn upsert_with_evidence(
        &self,
        decision: &NewDecision,
        evidence: &[NewDecisionEvidence],
        impacted_entities: &[NewDecisionImpactedEntity],
    ) -> Result<Decision, DecisionReviewPortError> {
        let input = DecisionUpsert {
            decision_id: super::ids::decision_id(decision),
            title: decision.title.clone(),
            status: decision.status.as_str().to_owned(),
            rationale: decision.rationale.clone(),
            alternatives: decision.alternatives.clone(),
            decided_by_entity_kind: decision
                .decided_by_entity_kind
                .map(|v| v.as_str().to_owned()),
            decided_by_entity_id: decision.decided_by_entity_id.clone(),
            decided_at: decision.decided_at,
            review_state: decision.review_state.as_str().to_owned(),
            confidence: decision.confidence,
            metadata: decision.metadata.clone(),
        };
        let evidence_api = evidence
            .iter()
            .map(|v| DecisionEvidence {
                source_kind: v.source_kind.as_str().to_owned(),
                source_id: v.source_id.clone(),
                observation_id: v.observation_id.clone(),
                excerpt: v.quote.clone(),
                confidence: v.confidence,
                metadata: v.metadata.clone(),
            })
            .collect::<Vec<_>>();
        let impacted_api = impacted_entities
            .iter()
            .map(|v| DecisionImpactedEntity {
                entity_kind: v.entity_kind.as_str().to_owned(),
                entity_id: v.entity_id.clone(),
                impact_type: v.impact_type.clone(),
                metadata: v.metadata.clone(),
            })
            .collect::<Vec<_>>();
        let value = self
            .writer
            .upsert(&input, &evidence_api, &impacted_api)
            .await
            .map_err(|e| DecisionReviewPortError(DecisionStoreError::Write(e.to_string())))?;
        from_api(value).map_err(DecisionReviewPortError)
    }

    pub async fn set_review_state(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
    ) -> Result<Decision, DecisionReviewPortError> {
        self.set_review_state_with_observation(decision_id, review_state, None, None)
            .await
            .map_err(DecisionReviewPortError)
    }

    pub async fn set_review_state_with_observation(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
        observation_id: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Decision, DecisionStoreError> {
        let value = self
            .writer
            .set_review_state_with_observation(
                decision_id,
                review_state.as_str(),
                observation_id,
                metadata,
            )
            .await
            .map_err(|e| DecisionStoreError::Write(e.to_string()))?;
        from_api(value)
    }

    pub async fn refresh_message_candidates_for_ids(
        &self,
        message_ids: &[String],
    ) -> Result<usize, DecisionReviewPortError> {
        self.store
            .refresh_message_candidates_for_ids(message_ids)
            .await
            .map_err(DecisionReviewPortError)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        decision: &NewDecision,
        evidence: &[NewDecisionEvidence],
        impacted_entities: &[NewDecisionImpactedEntity],
    ) -> Result<Decision, DecisionStoreError> {
        let input = DecisionUpsert {
            decision_id: super::ids::decision_id(decision),
            title: decision.title.clone(),
            status: decision.status.as_str().to_owned(),
            rationale: decision.rationale.clone(),
            alternatives: decision.alternatives.clone(),
            decided_by_entity_kind: decision
                .decided_by_entity_kind
                .map(|v| v.as_str().to_owned()),
            decided_by_entity_id: decision.decided_by_entity_id.clone(),
            decided_at: decision.decided_at,
            review_state: decision.review_state.as_str().to_owned(),
            confidence: decision.confidence,
            metadata: decision.metadata.clone(),
        };
        let evidence_api = evidence
            .iter()
            .map(|v| DecisionEvidence {
                source_kind: v.source_kind.as_str().to_owned(),
                source_id: v.source_id.clone(),
                observation_id: v.observation_id.clone(),
                excerpt: v.quote.clone(),
                confidence: v.confidence,
                metadata: v.metadata.clone(),
            })
            .collect::<Vec<_>>();
        let impacted_api = impacted_entities
            .iter()
            .map(|v| DecisionImpactedEntity {
                entity_kind: v.entity_kind.as_str().to_owned(),
                entity_id: v.entity_id.clone(),
                impact_type: v.impact_type.clone(),
                metadata: v.metadata.clone(),
            })
            .collect::<Vec<_>>();
        let value = DecisionPostgresReadQuery::upsert_in_transaction(
            transaction,
            &input,
            &evidence_api,
            &impacted_api,
        )
        .await
        .map_err(|e| DecisionStoreError::Write(e.to_string()))?;
        from_api(value)
    }
}

fn from_api(value: makosh_decisions_api::DecisionRead) -> Result<Decision, DecisionStoreError> {
    Ok(Decision {
        decision_id: value.decision_id,
        title: value.title,
        status: match value.status.as_str() {
            "active" => super::models::states::DecisionStatus::Active,
            "superseded" => super::models::states::DecisionStatus::Superseded,
            "reversed" => super::models::states::DecisionStatus::Reversed,
            "deprecated" => super::models::states::DecisionStatus::Deprecated,
            other => return Err(DecisionStoreError::UnknownStatus(other.to_owned())),
        },
        rationale: value.rationale,
        alternatives: value.alternatives,
        decided_by_entity_kind: value
            .decided_by_entity_kind
            .map(super::models::entity_kind::DecisionEntityKind::parse)
            .transpose()?,
        decided_by_entity_id: value.decided_by_entity_id,
        decided_at: value.decided_at,
        review_state: super::models::states::DecisionReviewState::parse(value.review_state)?,
        confidence: value.confidence,
        metadata: value.metadata,
        created_at: value.created_at,
        updated_at: value.updated_at,
    })
}

#[derive(Debug, thiserror::Error)]
#[error("decision review persistence failed: {0}")]
pub struct DecisionReviewPortError(#[from] DecisionStoreError);

impl DecisionReviewPortError {
    pub fn into_inner(self) -> DecisionStoreError {
        self.0
    }
}
