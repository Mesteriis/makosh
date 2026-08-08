use sqlx::{Postgres, Transaction};

use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::ports::PersonaProjectionPort;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

pub(crate) async fn create_persona_projection_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    email: Option<&str>,
    display_name: Option<&str>,
    message: Option<&ProjectedMessage>,
) -> Result<String, PersonaProjectionError> {
    let projection_observation_id = if let Some(message) = message {
        message.observation_id.clone()
    } else {
        ObservationStore::capture_in_transaction(
            transaction,
            &NewObservation::new(
                "PERSONA_MUTATION",
                ObservationOriginKind::Manual,
                chrono::Utc::now(),
                serde_json::json!({
                    "command_id": command_id,
                    "event_id": event_id,
                    "email": email,
                    "display_name": display_name,
                    "operation": "workflow_action_create_persona",
                }),
                format!("workflow-action://create-persona/{command_id}"),
            )
            .provenance(serde_json::json!({
                "captured_by": "workflows.create_persona_projection_in_transaction",
                "workflow_action": "create_persona",
            })),
        )
        .await?
        .observation_id
    };

    if let Some(email) = email {
        let (person, identity_id) =
            PersonaProjectionPort::upsert_email_persona_in_transaction(transaction, email).await?;
        PersonaProjectionPort::link_email_persona_projection_in_transaction(
            transaction,
            &projection_observation_id,
            &person,
            &identity_id,
            email,
            "workflow_action_projection",
        )
        .await?;
        return Ok(person.persona_id);
    }

    let fallback_persona_id = workflow_action_persona_id(command_id);
    let person = PersonaProjectionPort::upsert_persona_without_email_in_transaction(
        transaction,
        display_name,
        &fallback_persona_id,
        false,
    )
    .await?;
    PersonaProjectionPort::link_persona_projection_in_transaction(
        transaction,
        &projection_observation_id,
        &person,
        "workflow_action_projection",
    )
    .await?;
    Ok(person.persona_id)
}

fn workflow_action_persona_id(command_id: &str) -> String {
    let normalized = command_id
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    let normalized = if normalized.is_empty() {
        "manual".to_owned()
    } else {
        normalized
    };
    format!("persona:v1:workflow_action:{normalized}")
}
