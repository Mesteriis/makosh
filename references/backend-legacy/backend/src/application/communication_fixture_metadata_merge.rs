use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::communication_fixture_error::CommunicationFixtureIngestError;

pub(super) fn merge_object(
    current: &Value,
    patch: Value,
) -> Result<Value, CommunicationFixtureIngestError> {
    let Value::Object(mut current_map) = current.clone() else {
        return Err(CommunicationFixtureIngestError::SignalControlBlocked(
            "metadata is not a JSON object".to_owned(),
        ));
    };
    let Value::Object(patch_map) = patch else {
        return Err(CommunicationFixtureIngestError::SignalControlBlocked(
            "metadata patch is not a JSON object".to_owned(),
        ));
    };
    current_map.extend(patch_map);
    Ok(Value::Object(current_map))
}

pub(super) fn annotate_observed_source(
    raw: &makosh_communications_api::evidence::NewRawCommunicationRecord,
    observed_source: &str,
) -> Result<
    makosh_communications_api::evidence::NewRawCommunicationRecord,
    CommunicationFixtureIngestError,
> {
    let mut observed_raw = raw.clone();
    observed_raw.provenance = merge_object(
        &observed_raw.provenance,
        json!({"observed_source": observed_source}),
    )?;
    Ok(observed_raw)
}

pub(super) fn merge_identity_display_name(
    current_display_name: Option<&str>,
    current_metadata: &Value,
    new_display_name: Option<&str>,
    patch: Value,
    observed_at: DateTime<Utc>,
) -> Result<Value, CommunicationFixtureIngestError> {
    let mut merged = merge_object(current_metadata, patch)?;
    let Some(new_display_name) = new_display_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(merged);
    };
    let mut history = merged
        .get("display_name_history")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(previous_display_name) = current_display_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        crate::application::communication_fixture_metadata::push_json_string_once(
            &mut history,
            previous_display_name,
        );
        if previous_display_name != new_display_name {
            merged["previous_display_name"] = json!(previous_display_name);
            merged["display_name_changed_at"] = json!(observed_at);
        }
    }
    crate::application::communication_fixture_metadata::push_json_string_once(
        &mut history,
        new_display_name,
    );
    merged["display_name_history"] = Value::Array(history);
    merged["display_name_observed_at"] = json!(observed_at);
    Ok(merged)
}
