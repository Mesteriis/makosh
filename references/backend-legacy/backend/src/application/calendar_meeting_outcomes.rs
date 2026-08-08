use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::domains::calendar::evidence::link_calendar_entity_in_transaction;
use crate::domains::calendar::meetings::errors::MeetingsError;
use crate::domains::calendar::meetings::models::MeetingOutcome;
use crate::domains::calendar::ports::{CalendarEventQueryPort, MeetingOutcomePort};
use crate::domains::decisions::errors::DecisionStoreError;
use crate::domains::decisions::models::decision::NewDecision;
use crate::domains::decisions::ports::DecisionReviewPort;
use crate::domains::obligations::errors::ObligationStoreError;
use crate::domains::obligations::models::entity_kind::ObligationEntityKind;
use crate::domains::obligations::models::evidence::NewObligationEvidence;
use crate::domains::obligations::models::obligation::NewObligation;
use crate::domains::obligations::models::source_kind::ObligationEvidenceSourceKind;
use crate::domains::obligations::ports::ObligationReviewPort;
use crate::workflows::review_mirror::{
    ReviewMirrorError, decision::sync_decision_review_state_in_transaction,
    obligation::sync_obligation_review_state_in_transaction,
};
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

#[derive(Clone)]
pub struct CalendarMeetingOutcomeApplicationService {
    pool: PgPool,
}

impl CalendarMeetingOutcomeApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_manual(
        &self,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_person_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<MeetingOutcome, CalendarMeetingOutcomeApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "MEETING",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "event_id": event_id,
                        "outcome_type": outcome_type,
                        "title": title,
                        "description": description,
                        "owner_person_id": owner_person_id,
                        "due_date": due_date,
                    }),
                    format!("calendar-event://{event_id}/meeting-outcome"),
                )
                .provenance(json!({
                    "captured_by": "calendar_meeting_outcome_application.add_manual",
                    "operation": "add_manual",
                })),
            )
            .await?;

        let mut transaction = self.pool.begin().await?;
        let mut outcome = MeetingOutcomePort::add_with_observation_in_transaction(
            &mut transaction,
            event_id,
            outcome_type,
            title,
            description,
            owner_person_id,
            due_date,
            Some(&format!("observation:{}", observation.observation_id)),
            Some(&observation.observation_id),
        )
        .await?;

        if let Some(linked_entity_id) =
            linked_entity_for_outcome(&mut transaction, &outcome).await?
        {
            outcome = MeetingOutcomePort::set_linked_entity_id_in_transaction(
                &mut transaction,
                &outcome.id,
                &linked_entity_id,
            )
            .await?;
            link_calendar_entity_in_transaction(
                &mut transaction,
                &observation.observation_id,
                "meeting_outcome",
                outcome.id.clone(),
                None,
                json!({
                    "event_id": event_id,
                    "outcome_type": outcome.outcome_type,
                    "linked_entity_id": outcome.linked_entity_id,
                }),
                None,
            )
            .await?;
        }

        transaction.commit().await?;
        Ok(outcome)
    }
}

async fn linked_entity_for_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
) -> Result<Option<String>, CalendarMeetingOutcomeApplicationError> {
    let evidence_observation_id = calendar_event_observation_id(transaction, &outcome.event_id)
        .await?
        .unwrap_or_else(|| outcome.event_id.clone());
    match outcome.outcome_type.as_str() {
        "decision" => {
            linked_decision_for_outcome(transaction, outcome, &evidence_observation_id).await
        }
        "promise" => {
            linked_obligation_for_outcome(transaction, outcome, &evidence_observation_id).await
        }
        _ => Ok(None),
    }
}

async fn calendar_event_observation_id(
    transaction: &mut Transaction<'_, Postgres>,
    event_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    CalendarEventQueryPort::observation_id_in_transaction(transaction, event_id).await
}

async fn linked_decision_for_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
    observation_id: &str,
) -> Result<Option<String>, CalendarMeetingOutcomeApplicationError> {
    let decision = NewDecision::new(
        outcome.title.clone(),
        outcome
            .description
            .clone()
            .unwrap_or_else(|| outcome.title.clone()),
        outcome.confidence,
        DecisionReviewState::Suggested,
    )
    .metadata(json!({
        "source_domain": "calendar",
        "source_entity_kind": "meeting_outcome",
        "meeting_outcome_id": outcome.id,
        "event_id": outcome.event_id,
    }));
    let evidence =
        NewDecisionEvidence::new(DecisionEvidenceSourceKind::Event, outcome.event_id.clone())
            .with_observation_id(Some(observation_id.to_owned()))
            .quote(
                outcome
                    .description
                    .clone()
                    .unwrap_or_else(|| outcome.title.clone()),
            )
            .metadata(json!({
                "source_domain": "calendar",
                "meeting_outcome_id": outcome.id,
            }));
    let impacted_entity =
        NewDecisionImpactedEntity::new(DecisionEntityKind::Event, outcome.event_id.clone())
            .impact_type("meeting_outcome")
            .metadata(json!({ "meeting_outcome_id": outcome.id }));
    let stored = DecisionReviewPort::upsert_with_evidence_in_transaction(
        transaction,
        &decision,
        &[evidence],
        &[impacted_entity],
    )
    .await?;
    sync_decision_review_state_in_transaction(transaction, &stored).await?;
    Ok(Some(stored.decision_id))
}

async fn linked_obligation_for_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
    observation_id: &str,
) -> Result<Option<String>, CalendarMeetingOutcomeApplicationError> {
    let Some(owner_person_id) = outcome
        .owner_person_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(None);
    };
    let mut obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        owner_person_id.to_owned(),
        outcome.title.clone(),
        outcome.confidence,
        ObligationReviewState::Suggested,
    )
    .metadata(json!({
        "source_domain": "calendar",
        "source_entity_kind": "meeting_outcome",
        "meeting_outcome_id": outcome.id,
        "event_id": outcome.event_id,
    }));
    if let Some(due_date) = outcome.due_date {
        obligation = obligation.due_at(due_date);
    }
    let evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Event,
        outcome.event_id.clone(),
    )
    .with_observation_id(Some(observation_id.to_owned()))
    .quote(
        outcome
            .description
            .clone()
            .unwrap_or_else(|| outcome.title.clone()),
    )
    .metadata(json!({
        "source_domain": "calendar",
        "meeting_outcome_id": outcome.id,
    }));
    let stored = ObligationReviewPort::upsert_with_evidence_in_transaction(
        transaction,
        &obligation,
        &[evidence],
    )
    .await?;
    sync_obligation_review_state_in_transaction(transaction, &stored).await?;
    Ok(Some(stored.obligation_id))
}

#[derive(Debug, Error)]
pub enum CalendarMeetingOutcomeApplicationError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Meetings(#[from] MeetingsError),

    #[error(transparent)]
    Decision(#[from] DecisionStoreError),

    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}
use crate::domains::decisions::models::entity_kind::DecisionEntityKind;
use crate::domains::decisions::models::evidence::NewDecisionEvidence;
use crate::domains::decisions::models::impacted_entity::NewDecisionImpactedEntity;
use crate::domains::decisions::models::source_kind::DecisionEvidenceSourceKind;
use crate::domains::decisions::models::states::DecisionReviewState;
use crate::domains::obligations::models::states::ObligationReviewState;
