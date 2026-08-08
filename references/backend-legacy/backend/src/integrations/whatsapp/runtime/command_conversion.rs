use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::{
    CommunicationProviderCommand, NewWhatsappWebMedia, NewWhatsappWebReceipt,
    WhatsAppProviderCommand, WhatsAppProviderExecutableCommand, WhatsAppProviderWriteCommand,
    WhatsappWebError,
};

pub(super) fn provider_request_id_matches_observed_receipt(
    command: &WhatsAppProviderWriteCommand,
    receipt: &NewWhatsappWebReceipt,
) -> bool {
    let observed = receipt.provider_message_id.trim();
    !observed.is_empty()
        && (command.provider_message_id.as_deref() == Some(observed)
            || [
                json_string_at(
                    &command.result_payload,
                    &["provider_submission", "provider_request_id"],
                ),
                json_string_at(
                    &command.result_payload,
                    &[
                        "provider_submission",
                        "provider_observed_completion_target",
                        "provider_message_id",
                    ],
                ),
            ]
            .into_iter()
            .flatten()
            .any(|id| id == observed))
}

pub(super) fn provider_request_id_matches_observed_media(
    command: &WhatsAppProviderWriteCommand,
    media: &NewWhatsappWebMedia,
) -> bool {
    let observed = media.provider_message_id.trim();
    !observed.is_empty()
        && [
            json_string_at(
                &command.result_payload,
                &["provider_submission", "provider_request_id"],
            ),
            json_string_at(
                &command.result_payload,
                &[
                    "provider_submission",
                    "provider_observed_completion_target",
                    "provider_message_id",
                ],
            ),
        ]
        .into_iter()
        .flatten()
        .any(|id| id == observed)
}

fn json_string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(super) fn communication_provider_command(
    command: &WhatsAppProviderWriteCommand,
) -> CommunicationProviderCommand {
    CommunicationProviderCommand {
        command_id: command.command_id.clone(),
        account_id: command.account_id.clone(),
        channel_kind: "whatsapp".to_owned(),
        command_kind: command.command_kind.clone(),
        idempotency_key: command.idempotency_key.clone(),
        provider_conversation_id: Some(command.provider_chat_id.clone()),
        provider_message_id: command.provider_message_id.clone(),
        target_ref: command.target_ref.clone(),
        payload: command.payload.clone(),
        capability_state: command.capability_state.clone(),
        action_class: command.action_class.clone(),
        confirmation_decision: command.confirmation_decision.clone(),
        status: command.status.clone(),
        retry_count: command.retry_count,
        max_retries: command.max_retries,
        lease_epoch: 0,
        last_error: command.last_error.clone(),
        result_payload: command.result_payload.clone(),
        audit_metadata: command.audit_metadata.clone(),
        provider_state: command.provider_state.clone(),
        reconciliation_status: command.reconciliation_status.clone(),
        next_attempt_at: command.next_attempt_at,
        last_attempt_at: command.last_attempt_at,
        provider_observed_at: command.provider_observed_at,
        reconciled_at: command.reconciled_at,
        dead_lettered_at: command.dead_lettered_at,
        actor_id: "makosh-frontend".to_owned(),
        happened_at: command.created_at,
        completed_at: command.completed_at,
        created_at: command.created_at,
        updated_at: command.updated_at,
    }
}

pub(super) fn canonical_provider_chat_id(
    command: &makosh_communications_api::commands::CommunicationProviderCommand,
) -> Option<&str> {
    command
        .provider_conversation_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            command
                .target_ref
                .get("provider_chat_id")
                .and_then(Value::as_str)
        })
        .or_else(|| (command.command_kind == "publish_status").then_some("status-feed"))
}

pub(super) fn provider_message_id_from_state<'a>(
    provider_state: &'a Value,
    result_payload: &'a Value,
) -> Option<&'a str> {
    result_payload
        .get("provider_message_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            provider_state
                .get("provider_message_id")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
        })
}

pub(super) fn row_to_whatsapp_provider_write_command(
    row: PgRow,
) -> Result<WhatsAppProviderWriteCommand, WhatsappWebError> {
    Ok(WhatsAppProviderWriteCommand {
        command_id: row.try_get("command_id")?,
        account_id: row.try_get("account_id")?,
        command_kind: row.try_get("command_kind")?,
        idempotency_key: row.try_get("idempotency_key")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        capability_state: row.try_get("capability_state")?,
        action_class: row.try_get("action_class")?,
        confirmation_decision: row.try_get("confirmation_decision")?,
        status: row.try_get("status")?,
        retry_count: row.try_get("retry_count")?,
        max_retries: row.try_get("max_retries")?,
        last_error: row.try_get("last_error")?,
        payload: row.try_get("payload")?,
        target_ref: row.try_get("target_ref")?,
        result_payload: row.try_get("result_payload")?,
        audit_metadata: row.try_get("audit_metadata")?,
        provider_state: row.try_get("provider_state")?,
        reconciliation_status: row.try_get("reconciliation_status")?,
        next_attempt_at: row.try_get("next_attempt_at")?,
        last_attempt_at: row.try_get("last_attempt_at")?,
        provider_observed_at: row.try_get("provider_observed_at")?,
        reconciled_at: row.try_get("reconciled_at")?,
        dead_lettered_at: row.try_get("dead_lettered_at")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

impl From<WhatsAppProviderWriteCommand> for WhatsAppProviderCommand {
    fn from(command: WhatsAppProviderWriteCommand) -> Self {
        Self {
            command_id: command.command_id,
            account_id: command.account_id,
            command_kind: command.command_kind,
            idempotency_key: command.idempotency_key,
            provider_chat_id: command.provider_chat_id,
            provider_message_id: command.provider_message_id,
            capability_state: command.capability_state,
            action_class: command.action_class,
            confirmation_decision: command.confirmation_decision,
            status: command.status,
            retry_count: command.retry_count,
            max_retries: command.max_retries,
            last_error: command.last_error,
            result_payload: command.result_payload,
            audit_metadata: command.audit_metadata,
            provider_state: command.provider_state,
            reconciliation_status: command.reconciliation_status,
            next_attempt_at: command.next_attempt_at,
            last_attempt_at: command.last_attempt_at,
            provider_observed_at: command.provider_observed_at,
            reconciled_at: command.reconciled_at,
            dead_lettered_at: command.dead_lettered_at,
            completed_at: command.completed_at,
            created_at: command.created_at,
            updated_at: command.updated_at,
        }
    }
}

impl From<&WhatsAppProviderWriteCommand> for WhatsAppProviderExecutableCommand {
    fn from(command: &WhatsAppProviderWriteCommand) -> Self {
        Self {
            command_id: command.command_id.clone(),
            account_id: command.account_id.clone(),
            command_kind: command.command_kind.clone(),
            idempotency_key: command.idempotency_key.clone(),
            provider_chat_id: command.provider_chat_id.clone(),
            provider_message_id: command.provider_message_id.clone(),
            payload: command.payload.clone(),
            target_ref: command.target_ref.clone(),
            audit_metadata: command.audit_metadata.clone(),
            provider_state: command.provider_state.clone(),
        }
    }
}
