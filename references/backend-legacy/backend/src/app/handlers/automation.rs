use axum::Json;
use axum::extract::State;

use crate::engines::automation::{
    errors::AutomationError,
    models::{
        AutomationPolicy, AutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
    },
};
use crate::platform::audit::models::NewApiAuditRecord;
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};

use crate::app::api_support::{
    automation_calls::*,
    stores::{domain_stores::*, integration_stores::*},
};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;

pub(crate) async fn post_policy_template(
    State(state): State<AppState>,
    Json(request): Json<PolicyTemplateApiRequest>,
) -> Result<Json<AutomationTemplate>, ApiError> {
    let actor_id = "makosh-frontend";
    Ok(Json(
        automation_store(&state)?
            .upsert_template(&request.into_template(), actor_id)
            .await?,
    ))
}

pub(crate) async fn get_policy_templates(
    State(state): State<AppState>,
) -> Result<Json<PolicyTemplateListResponse>, ApiError> {
    let items = automation_store(&state)?.list_templates().await?;

    Ok(Json(PolicyTemplateListResponse { items }))
}

pub(crate) async fn post_policy(
    State(state): State<AppState>,
    Json(request): Json<PolicyApiRequest>,
) -> Result<Json<AutomationPolicy>, ApiError> {
    let actor_id = "makosh-frontend";
    Ok(Json(
        automation_store(&state)?
            .upsert_policy(&request.into_policy(), actor_id)
            .await?,
    ))
}

pub(crate) async fn get_policies(
    State(state): State<AppState>,
) -> Result<Json<PolicyListResponse>, ApiError> {
    let items = automation_store(&state)?.list_policies().await?;

    Ok(Json(PolicyListResponse { items }))
}

pub(crate) async fn post_telegram_send_dry_run(
    State(state): State<AppState>,
    Json(request): Json<TelegramSendDryRunRequest>,
) -> Result<Json<TelegramSendDryRunResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();
    let response = match automation_store(&state)?
        .dry_run_send(&request, &actor_id)
        .await
    {
        Ok(response) => response,
        Err(error) => {
            if let Some(decision) = telegram_send_dry_run_rejection_decision(&error, &request) {
                api_audit_log(&state)?
                    .record(
                        &NewApiAuditRecord::automation_telegram_send_dry_run_rejected(
                            &actor_id,
                            &request.command_id,
                            &request.policy_id,
                            &request.provider_chat_id,
                            &decision,
                        ),
                    )
                    .await?;
            }
            return Err(error.into());
        }
    };
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::automation_telegram_send_dry_run(
            &actor_id,
            &response.outbound_message_id,
            &response.policy_id,
            &response.template_id,
            &response.account_id,
            &response.provider_chat_id,
            &response.rendered_preview_hash,
        ))
        .await?;

    Ok(Json(response))
}

pub(crate) fn telegram_send_dry_run_rejection_decision(
    error: &AutomationError,
    request: &TelegramSendDryRunRequest,
) -> Option<CapabilityDecision> {
    let reason = match error {
        AutomationError::InvalidRequest(_) => "invalid_request",
        AutomationError::PolicyNotFound => "policy_not_found",
        AutomationError::PolicyDisabled => "policy_disabled",
        AutomationError::ChatNotAllowed => "provider_chat_not_allowed",
        AutomationError::MissingTemplateVariable(_) => "template_variable_missing",
        AutomationError::UndeclaredTemplateVariable(_) => "template_variable_undeclared",
        AutomationError::EventEnvelope(_)
        | AutomationError::EventStore(_)
        | AutomationError::ObservationStore(_)
        | AutomationError::Sqlx(_) => return None,
    };

    Some(CapabilityDecision::rejected_high_risk(
        CapabilityActionClass::Automation,
        "telegram.send",
        reason,
        non_empty_optional_string(&request.policy_id),
    ))
}

pub(crate) fn non_empty_optional_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
