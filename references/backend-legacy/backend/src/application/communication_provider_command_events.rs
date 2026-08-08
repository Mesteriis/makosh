use makosh_events_api::NewEventEnvelope;
use serde_json::json;

use crate::application::communication_fixture_event::build as build_event;
use crate::platform::events::bus::telegram_event_types;

pub(crate) struct CommandEventInput<'a> {
    pub(crate) account_id: &'a str,
    pub(crate) command_id: &'a str,
    pub(crate) command_kind: &'a str,
    pub(crate) provider_chat_id: &'a str,
    pub(crate) message_id: Option<&'a str>,
    pub(crate) provider_message_id: Option<&'a str>,
    pub(crate) status: &'a str,
    pub(crate) extra_payload: serde_json::Value,
}

pub(crate) fn build_command_event(input: CommandEventInput<'_>) -> NewEventEnvelope {
    let mut payload = json!({
        "account_id": input.account_id,
        "command_id": input.command_id,
        "command_kind": input.command_kind,
        "action": input.command_kind,
        "provider_chat_id": input.provider_chat_id,
        "message_id": input.message_id,
        "provider_message_id": input.provider_message_id,
        "status": input.status,
    });
    if let (Some(base), Some(extra)) = (payload.as_object_mut(), input.extra_payload.as_object()) {
        for (key, value) in extra {
            base.insert(key.clone(), value.clone());
        }
    }
    if let Some(payload_obj) = payload.as_object_mut() {
        payload_obj.insert("payload".to_owned(), input.extra_payload);
    }
    build_event(
        telegram_event_types::COMMAND_STATUS_CHANGED,
        input.account_id,
        input.command_id,
        payload,
    )
}

pub(crate) fn merge_json_objects(
    mut base: serde_json::Value,
    extra: serde_json::Value,
) -> serde_json::Value {
    if let (Some(base_obj), Some(extra_obj)) = (base.as_object_mut(), extra.as_object()) {
        for (key, value) in extra_obj {
            base_obj.insert(key.clone(), value.clone());
        }
    }
    base
}
