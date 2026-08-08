use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{Postgres, Transaction};

use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;
use makosh_observations_postgres::store::ObservationStore;

use super::errors::AutomationError;
use super::models::{
    AutomationPolicy, AutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};

pub(super) async fn capture_template_observation(
    transaction: &mut Transaction<'_, Postgres>,
    template: &AutomationTemplate,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AUTOMATION_TEMPLATE",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "template_id": template.template_id,
                "name": template.name,
                "body_template": template.body_template,
                "required_variables": template.required_variables,
                "operation": relationship_kind,
            }),
            format!("automation-template://{}", template.template_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_domain_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "automation",
        "template",
        template.template_id.clone(),
        Some(relationship_kind),
        None,
        Some(json!({
            "name": template.name,
            "required_variables": template.required_variables,
        })),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_policy_observation(
    transaction: &mut Transaction<'_, Postgres>,
    policy: &AutomationPolicy,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AUTOMATION_POLICY",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "policy_id": policy.policy_id,
                "template_id": policy.template_id,
                "name": policy.name,
                "enabled": policy.enabled,
                "account_id": policy.account_id,
                "allowed_chat_ids": policy.allowed_chat_ids,
                "scopes": policy.scopes,
                "trigger_kind": policy.trigger_kind,
                "max_sends_per_hour": policy.max_sends_per_hour,
                "quiet_hours": policy.quiet_hours,
                "expires_at": policy.expires_at,
                "conditions": policy.conditions,
                "operation": relationship_kind,
            }),
            format!("automation-policy://{}", policy.policy_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_domain_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "automation",
        "policy",
        policy.policy_id.clone(),
        Some(relationship_kind),
        None,
        Some(json!({
            "template_id": policy.template_id,
            "enabled": policy.enabled,
            "account_id": policy.account_id,
            "trigger_kind": policy.trigger_kind,
        })),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_dry_run_observation(
    transaction: &mut Transaction<'_, Postgres>,
    request: &TelegramSendDryRunRequest,
    response: &TelegramSendDryRunResponse,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_OUTBOUND_MESSAGE",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "outbound_message_id": response.outbound_message_id,
                "command_id": request.command_id,
                "policy_id": response.policy_id,
                "template_id": response.template_id,
                "account_id": response.account_id,
                "provider_chat_id": response.provider_chat_id,
                "rendered_preview_hash": response.rendered_preview_hash,
                "status": response.status,
                "send_mode": "dry_run",
                "variables": request.variables,
                "source_context": request.source_context,
            }),
            format!(
                "telegram-outbound-message://{}",
                response.outbound_message_id
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": "dry_run_allowed",
            "event_id": response.event_id,
        })),
    )
    .await?;
    link_domain_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "automation",
        "telegram_outbound_message",
        response.outbound_message_id.clone(),
        Some("dry_run_allowed"),
        None,
        Some(json!({
            "policy_id": response.policy_id,
            "template_id": response.template_id,
            "account_id": response.account_id,
            "provider_chat_id": response.provider_chat_id,
            "status": response.status,
            "send_mode": "dry_run",
        })),
    )
    .await?;
    Ok(())
}
