use chrono::Utc;
use makosh_events_api::EventEnvelope;
use serde_json::Value;

use makosh_events_postgres::store::EventStore;
use makosh_observations_postgres::review_links::materialize_review_transition_link_in_transaction;

use super::super::constants::{
    PERSONA_IDENTITY_REVIEW_PREFIX, is_persona_identity_review_event_type,
};
use super::super::errors::PersonaIdentityError;
use super::super::events::{ReviewCommandEvent, ReviewEvent};
use super::super::models::{PersonaIdentityReviewCommand, PersonaIdentityReviewCommandResult};
use super::super::validation::validate_non_empty;
use super::PersonaIdentityReviewStore;
use super::review_state::{apply_review_state_in_transaction, ensure_candidate_exists};
use super::split_candidates::materialize_split_candidate_for_confirmed_merge_in_transaction;

impl PersonaIdentityReviewStore {
    pub async fn set_review_state(
        &self,
        command: &PersonaIdentityReviewCommand,
    ) -> Result<PersonaIdentityReviewCommandResult, PersonaIdentityError> {
        self.set_review_state_with_observation(command, None, None)
            .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        command: &PersonaIdentityReviewCommand,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<PersonaIdentityReviewCommandResult, PersonaIdentityError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let identity_candidate_id =
            validate_non_empty("identity_candidate_id", &command.identity_candidate_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;

        let mut transaction = self.pool().begin().await?;
        ensure_candidate_exists(&mut transaction, &identity_candidate_id).await?;

        let event_id = format!("{PERSONA_IDENTITY_REVIEW_PREFIX}{command_id}");
        let event = ReviewCommandEvent {
            command_id,
            identity_candidate_id: identity_candidate_id.clone(),
            review_state: command.review_state,
            actor_id: actor_id.clone(),
            event_id: event_id.clone(),
            occurred_at: Utc::now(),
        }
        .to_event()?;

        EventStore::append_in_transaction(&mut transaction, &event).await?;
        apply_review_state_in_transaction(
            &mut transaction,
            &identity_candidate_id,
            command.review_state,
            &event_id,
            &actor_id,
            event.occurred_at,
        )
        .await?;
        materialize_split_candidate_for_confirmed_merge_in_transaction(
            &mut transaction,
            &identity_candidate_id,
            command.review_state,
        )
        .await?;
        materialize_review_transition_link_in_transaction(
            &mut transaction,
            observation_id,
            "personas",
            "identity_candidate",
            &identity_candidate_id,
            "review_state",
            command.review_state.as_str(),
            metadata
                .map(|extra| {
                    serde_json::json!({
                        "event_id": event_id,
                        "context": extra,
                    })
                })
                .or_else(|| {
                    Some(serde_json::json!({
                        "event_id": event_id,
                    }))
                }),
        )
        .await?;

        transaction.commit().await?;

        Ok(PersonaIdentityReviewCommandResult {
            identity_candidate_id,
            review_state: command.review_state,
            event_id,
        })
    }

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), PersonaIdentityError> {
        if !is_persona_identity_review_event_type(&event.event_type) {
            return Err(PersonaIdentityError::InvalidEventType);
        }

        let parsed = ReviewEvent::from_payload(&event.payload)?;
        let actor_id = event
            .actor
            .as_ref()
            .and_then(|value| value.get("actor_id"))
            .and_then(serde_json::Value::as_str)
            .ok_or(PersonaIdentityError::MissingActorId)?;
        let actor_id = validate_non_empty("actor_id", actor_id)?;
        let mut transaction = self.pool().begin().await?;
        ensure_candidate_exists(&mut transaction, &parsed.identity_candidate_id).await?;
        apply_review_state_in_transaction(
            &mut transaction,
            &parsed.identity_candidate_id,
            parsed.review_state,
            &event.event_id,
            &actor_id,
            event.occurred_at,
        )
        .await?;
        materialize_split_candidate_for_confirmed_merge_in_transaction(
            &mut transaction,
            &parsed.identity_candidate_id,
            parsed.review_state,
        )
        .await?;

        transaction.commit().await?;
        Ok(())
    }
}
