use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use sqlx::postgres::PgPool;

use makosh_events_postgres::store::EventStore;

use super::constants::{
    AUTOMATION_SEND_DRY_RUN_EVENT_TYPE, AUTOMATION_SOURCE_KIND, AUTOMATION_SOURCE_PROVIDER,
};
use super::errors::AutomationError;
use super::evidence::capture_dry_run_observation;
use super::ids::sha256_hex;
use super::models::{TelegramSendDryRunRequest, TelegramSendDryRunResponse};
use super::policy::evaluate_policy;
use super::store::AutomationStore;
use super::validation::validate_non_empty;

pub(super) async fn dry_run_send(
    pool: &PgPool,
    request: &TelegramSendDryRunRequest,
    actor_id: &str,
) -> Result<TelegramSendDryRunResponse, AutomationError> {
    request.validate()?;
    let actor_id = validate_non_empty("actor_id", actor_id)?;
    let (policy, template) =
        AutomationStore::policy_with_template(pool, &request.policy_id).await?;
    let rendered_text = evaluate_policy(&policy, &template, request)?;
    let rendered_preview_hash = sha256_hex(rendered_text.as_bytes());
    let outbound_message_id = format!(
        "telegram_outbound:v4:{}",
        sha256_hex(
            [
                request.command_id.as_str(),
                request.policy_id.as_str(),
                request.provider_chat_id.as_str(),
                rendered_preview_hash.as_str(),
            ]
            .join("\0")
            .as_bytes()
        )
    );
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO telegram_outbound_messages (
            outbound_message_id,
            policy_id,
            template_id,
            account_id,
            provider_chat_id,
            send_mode,
            status,
            rendered_preview_hash,
            variables,
            source_context,
            actor_id
        )
        VALUES ($1, $2, $3, $4, $5, 'dry_run', 'allowed', $6, $7, $8, $9)
        ON CONFLICT (outbound_message_id)
        DO NOTHING
        "#,
    )
    .bind(&outbound_message_id)
    .bind(&policy.policy_id)
    .bind(&template.template_id)
    .bind(&policy.account_id)
    .bind(&request.provider_chat_id)
    .bind(&rendered_preview_hash)
    .bind(&request.variables)
    .bind(&request.source_context)
    .bind(&actor_id)
    .execute(&mut *transaction)
    .await?;

    let event_id = format!(
        "automation_telegram_send_dry_run:{}",
        request.command_id.trim()
    );
    let event = NewEventEnvelope::builder(
        event_id.clone(),
        AUTOMATION_SEND_DRY_RUN_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": AUTOMATION_SOURCE_KIND,
            "provider": AUTOMATION_SOURCE_PROVIDER,
            "policy_id": policy.policy_id,
        }),
        json!({
            "kind": "telegram_outbound_message",
            "id": outbound_message_id,
        }),
    )
    .actor(json!({"actor_id": actor_id}))
    .payload(json!({
        "command_id": request.command_id,
        "outbound_message_id": outbound_message_id,
        "policy_id": policy.policy_id,
        "template_id": template.template_id,
        "account_id": policy.account_id,
        "provider_chat_id": request.provider_chat_id,
        "rendered_preview_hash": rendered_preview_hash,
        "send_mode": "dry_run",
        "status": "allowed",
    }))
    .build()?;
    EventStore::append_in_transaction(&mut transaction, &event).await?;
    let response = TelegramSendDryRunResponse {
        outbound_message_id,
        policy_id: policy.policy_id,
        template_id: template.template_id,
        account_id: policy.account_id,
        provider_chat_id: request.provider_chat_id.clone(),
        rendered_text,
        rendered_preview_hash,
        status: "allowed".to_owned(),
        event_id,
    };
    capture_dry_run_observation(&mut transaction, request, &response, &actor_id, Utc::now())
        .await?;
    transaction.commit().await?;

    Ok(response)
}
