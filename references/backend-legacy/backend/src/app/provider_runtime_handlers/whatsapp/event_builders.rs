use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppProviderCommand, WhatsAppProviderCommandResponse,
};
use crate::platform::events::bus::whatsapp_event_types;
use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

const AUDIT_ACTOR_ID: &str = "makosh-frontend";
static EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(super) fn command_event(response: &WhatsAppProviderCommandResponse) -> NewEventEnvelope {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}",
        response.command_id,
        response.command_kind,
        response.status,
        response.updated_at.timestamp_micros()
    );
    NewEventEnvelope::builder(event_id("command_response", &response.command_id, now), whatsapp_event_types::COMMAND_STATUS_CHANGED.to_owned(), now,
        json!({"channel":"whatsapp","account_id":response.account_id,"actor_id":AUDIT_ACTOR_ID,"kind":"whatsapp_provider_commands","source_id":source_id}),
        json!({"id":response.command_id,"entity_id":response.command_id,"kind":"whatsapp_provider_command"}))
    .payload(json!({"account_id":response.account_id,"command_id":response.command_id,"idempotency_key":response.idempotency_key,"command_kind":response.command_kind,"action":response.command_kind,"provider_chat_id":response.provider_chat_id,"provider_message_id":response.provider_message_id,"status":response.status,"durable_status":response.durable_status,"delivery_state":response.delivery_state,"runtime_kind":response.runtime_kind,"provider_shape":response.provider_shape,"session_restore_available":response.session_restore_available,"runtime_blockers":response.runtime_blockers,"rendered_preview_hash":response.rendered_preview_hash}))
    .build().expect("WhatsApp command event envelope must be valid")
}

pub(super) fn command_record_event(
    command: &WhatsAppProviderCommand,
    source: &str,
) -> NewEventEnvelope {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}:{}",
        command.command_id,
        command.command_kind,
        command.status,
        source,
        command.updated_at.timestamp_micros()
    );
    NewEventEnvelope::builder(event_id("command_record", &command.command_id, now), whatsapp_event_types::COMMAND_STATUS_CHANGED.to_owned(), now,
        json!({"channel":"whatsapp","account_id":command.account_id,"actor_id":AUDIT_ACTOR_ID,"kind":"whatsapp_provider_commands","source_id":source_id}),
        json!({"id":command.command_id,"entity_id":command.command_id,"kind":"whatsapp_provider_command"}))
    .payload(json!({"account_id":command.account_id,"command_id":command.command_id,"idempotency_key":command.idempotency_key,"command_kind":command.command_kind,"action":command.command_kind,"provider_chat_id":command.provider_chat_id,"provider_message_id":command.provider_message_id,"capability_state":command.capability_state,"action_class":command.action_class,"confirmation_decision":command.confirmation_decision,"status":command.status,"retry_count":command.retry_count,"max_retries":command.max_retries,"last_error":command.last_error,"result_payload":command.result_payload,"audit_metadata":command.audit_metadata,"provider_state":command.provider_state,"reconciliation_status":command.reconciliation_status,"next_attempt_at":command.next_attempt_at,"last_attempt_at":command.last_attempt_at,"provider_observed_at":command.provider_observed_at,"reconciled_at":command.reconciled_at,"dead_lettered_at":command.dead_lettered_at,"completed_at":command.completed_at,"source":source}))
    .build().expect("WhatsApp command record event envelope must be valid")
}

pub(super) fn event_id(scope: &str, subject: &str, now: chrono::DateTime<chrono::Utc>) -> String {
    let seq = EVENT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "evt_whatsapp_{}_{}_{}_{}",
        scope,
        subject.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        now.timestamp_nanos_opt().unwrap_or_default(),
        seq
    )
}
