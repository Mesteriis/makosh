use serde_json::Value;

use super::errors::TimelineEngineError;

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), TimelineEngineError> {
    if value.trim().is_empty() {
        return Err(TimelineEngineError::EmptyField(field));
    }
    Ok(())
}

pub(super) fn required_json_string(
    value: &Value,
    object_name: &'static str,
    field_name: &'static str,
    event_id: &str,
) -> Result<String, TimelineEngineError> {
    let field_value = value
        .get(field_name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| TimelineEngineError::InvalidEventLogField {
            event_id: event_id.to_owned(),
            object_name,
            field_name,
        })?;

    Ok(field_value.to_owned())
}

pub(super) fn optional_json_string(value: &Value, field_name: &str) -> Option<String> {
    value
        .get(field_name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn event_log_source_ref(event: &makosh_events_api::EventEnvelope) -> String {
    let Some(kind) = optional_json_string(&event.source, "kind") else {
        return event.event_id.clone();
    };
    let Some(source_id) = optional_json_string(&event.source, "source_id") else {
        return event.event_id.clone();
    };

    format!("{kind}:{source_id}")
}
