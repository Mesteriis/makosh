use makosh_obligations_api::{ObligationEvidence, ObligationUpsert, ObligationWritePort};
use makosh_obligations_postgres::ObligationPostgresReadQuery;
use sqlx::PgPool;

use super::errors::ObligationStoreError;
use super::ids::obligation_id;
use super::models::evidence::NewObligationEvidence;
use super::models::obligation::NewObligation;
use super::models::read_model::Obligation;
use super::models::states::ObligationReviewState;

/// Workflow-owned obligation command port. Concrete SQL storage stays inside
/// the obligations bounded context while workflows depend on this semantic API.
#[derive(Clone)]
pub struct ObligationReviewPort {
    writer: ObligationPostgresReadQuery,
}

impl ObligationReviewPort {
    pub fn new(pool: PgPool) -> Self {
        Self {
            writer: ObligationPostgresReadQuery::new(pool.clone()),
        }
    }

    pub async fn upsert_with_evidence(
        &self,
        obligation: &NewObligation,
        evidence: &[NewObligationEvidence],
    ) -> Result<Obligation, ObligationReviewPortError> {
        let input = ObligationUpsert {
            obligation_id: obligation_id(obligation),
            obligated_entity_kind: obligation.obligated_entity_kind.as_str().to_owned(),
            obligated_entity_id: obligation.obligated_entity_id.clone(),
            beneficiary_entity_kind: obligation
                .beneficiary_entity_kind
                .map(|v| v.as_str().to_owned()),
            beneficiary_entity_id: obligation.beneficiary_entity_id.clone(),
            statement: obligation.statement.clone(),
            status: obligation.status.as_str().to_owned(),
            review_state: obligation.review_state.as_str().to_owned(),
            due_at: obligation.due_at,
            condition: obligation.condition.clone(),
            risk_state: obligation.risk_state.as_str().to_owned(),
            confidence: obligation.confidence,
            metadata: obligation.metadata.clone(),
        };
        let items: Vec<ObligationEvidence> = evidence
            .iter()
            .map(|item| ObligationEvidence {
                source_kind: item.source_kind.as_str().to_owned(),
                source_id: item.source_id.clone(),
                observation_id: item.observation_id.clone(),
                excerpt: item.quote.clone(),
                confidence: item.confidence,
                metadata: item.metadata.clone(),
            })
            .collect();
        let read =
            self.writer.upsert(&input, &items).await.map_err(|e| {
                ObligationReviewPortError(ObligationStoreError::Write(e.to_string()))
            })?;
        from_api(read).map_err(ObligationReviewPortError)
    }

    pub async fn set_review_state(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
    ) -> Result<Obligation, ObligationReviewPortError> {
        self.set_review_state_with_observation(obligation_id, review_state, None, None)
            .await
            .map_err(|e| ObligationReviewPortError(ObligationStoreError::Write(e.to_string())))
    }

    pub async fn set_review_state_with_observation(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
        observation_id: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Obligation, ObligationStoreError> {
        let read = self
            .writer
            .set_review_state_with_observation(
                obligation_id,
                review_state.as_str(),
                observation_id,
                metadata,
            )
            .await
            .map_err(|error| ObligationStoreError::Write(error.to_string()))?;
        from_api(read)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        obligation: &NewObligation,
        evidence: &[NewObligationEvidence],
    ) -> Result<Obligation, ObligationStoreError> {
        let input = ObligationUpsert {
            obligation_id: obligation_id(obligation),
            obligated_entity_kind: obligation.obligated_entity_kind.as_str().to_owned(),
            obligated_entity_id: obligation.obligated_entity_id.clone(),
            beneficiary_entity_kind: obligation
                .beneficiary_entity_kind
                .map(|v| v.as_str().to_owned()),
            beneficiary_entity_id: obligation.beneficiary_entity_id.clone(),
            statement: obligation.statement.clone(),
            status: obligation.status.as_str().to_owned(),
            review_state: obligation.review_state.as_str().to_owned(),
            due_at: obligation.due_at,
            condition: obligation.condition.clone(),
            risk_state: obligation.risk_state.as_str().to_owned(),
            confidence: obligation.confidence,
            metadata: obligation.metadata.clone(),
        };
        let items: Vec<ObligationEvidence> = evidence
            .iter()
            .map(|item| ObligationEvidence {
                source_kind: item.source_kind.as_str().to_owned(),
                source_id: item.source_id.clone(),
                observation_id: item.observation_id.clone(),
                excerpt: item.quote.clone(),
                confidence: item.confidence,
                metadata: item.metadata.clone(),
            })
            .collect();
        let value =
            makosh_obligations_postgres::ObligationPostgresReadQuery::upsert_in_transaction(
                transaction,
                &input,
                &items,
            )
            .await
            .map_err(|e| ObligationStoreError::Write(e.to_string()))?;
        from_api(value)
    }

    pub(crate) async fn set_review_state_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        obligation_id: &str,
        review_state: ObligationReviewState,
        observation_id: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Obligation, ObligationStoreError> {
        let value = makosh_obligations_postgres::ObligationPostgresReadQuery::set_review_state_in_transaction(transaction, obligation_id, review_state.as_str(), observation_id, metadata).await.map_err(|e| ObligationStoreError::Write(e.to_string()))?;
        from_api(value)
    }
}

fn from_api(
    value: makosh_obligations_api::ObligationRead,
) -> Result<Obligation, ObligationStoreError> {
    Ok(Obligation {
        obligation_id: value.obligation_id,
        obligated_entity_kind: super::models::entity_kind::ObligationEntityKind::parse(
            value.obligated_entity_kind,
        )?,
        obligated_entity_id: value.obligated_entity_id,
        beneficiary_entity_kind: value
            .beneficiary_entity_kind
            .map(super::models::entity_kind::ObligationEntityKind::parse)
            .transpose()?,
        beneficiary_entity_id: value.beneficiary_entity_id,
        statement: value.statement,
        status: match value.status.as_str() {
            "open" => super::models::states::ObligationStatus::Open,
            "fulfilled" => super::models::states::ObligationStatus::Fulfilled,
            "waived" => super::models::states::ObligationStatus::Waived,
            "disputed" => super::models::states::ObligationStatus::Disputed,
            "canceled" => super::models::states::ObligationStatus::Canceled,
            other => return Err(ObligationStoreError::UnknownStatus(other.to_owned())),
        },
        review_state: super::models::states::ObligationReviewState::parse(value.review_state)?,
        due_at: value.due_at,
        condition: value.condition,
        risk_state: match value.risk_state.as_str() {
            "none" => super::models::states::ObligationRiskState::None,
            "watch" => super::models::states::ObligationRiskState::Watch,
            "at_risk" => super::models::states::ObligationRiskState::AtRisk,
            "breached" => super::models::states::ObligationRiskState::Breached,
            other => return Err(ObligationStoreError::UnknownRiskState(other.to_owned())),
        },
        confidence: value.confidence,
        metadata: value.metadata,
        created_at: value.created_at,
        updated_at: value.updated_at,
    })
}

#[derive(Debug, thiserror::Error)]
#[error("obligation review persistence failed: {0}")]
pub struct ObligationReviewPortError(#[from] ObligationStoreError);

impl ObligationReviewPortError {
    pub fn into_inner(self) -> ObligationStoreError {
        self.0
    }
}
