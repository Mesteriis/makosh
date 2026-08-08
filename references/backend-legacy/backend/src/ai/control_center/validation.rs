use chrono::Utc;
use serde_json::{Map, Value};

use super::errors::AiControlCenterError;

pub(super) const CAPABILITY_SLOTS: &[&str] = &[
    "default_chat",
    "reasoning",
    "summarization",
    "mail_intelligence",
    "reply_draft",
    "extraction",
    "embeddings",
    "meeting_prep",
];

const ENTITY_SCOPES: &[&str] = &[
    "global",
    "persona",
    "organization",
    "project",
    "document",
    "task",
    "meeting",
    "communication",
    "conversation",
];

pub(super) fn canonical_entity_scope(value: &str) -> Option<&'static str> {
    let trimmed = value.trim();
    if trimmed == "person" {
        return Some("persona");
    }
    ENTITY_SCOPES
        .iter()
        .copied()
        .find(|scope| *scope == trimmed)
}

pub(super) fn validate_provider_kind(value: &str) -> Result<(), AiControlCenterError> {
    match value.trim() {
        "built_in" | "cli" | "api" => Ok(()),
        other => Err(AiControlCenterError::InvalidRequest(format!(
            "unsupported provider_kind `{other}`"
        ))),
    }
}

pub(super) fn validate_cli_preset(value: &str) -> Result<(), AiControlCenterError> {
    match value.trim() {
        "codex" | "claude" | "makosh" => Ok(()),
        other => Err(AiControlCenterError::InvalidRequest(format!(
            "unsupported CLI command preset `{other}`"
        ))),
    }
}

pub(super) fn validate_capability_slot(value: &str) -> Result<(), AiControlCenterError> {
    if CAPABILITY_SLOTS.contains(&value.trim()) {
        Ok(())
    } else {
        Err(AiControlCenterError::InvalidRequest(format!(
            "unsupported capability slot `{}`",
            value.trim()
        )))
    }
}

pub(super) fn validate_entity_scope(value: &str) -> Result<(), AiControlCenterError> {
    if canonical_entity_scope(value).is_some() {
        Ok(())
    } else {
        Err(AiControlCenterError::InvalidRequest(format!(
            "unsupported entity scope `{}`",
            value.trim()
        )))
    }
}

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), AiControlCenterError> {
    if value.trim().is_empty() {
        return Err(AiControlCenterError::EmptyField { field });
    }
    Ok(())
}

pub(super) fn non_empty_optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub(super) fn object_value(
    value: Value,
    field: &'static str,
) -> Result<Map<String, Value>, AiControlCenterError> {
    value
        .as_object()
        .cloned()
        .ok_or_else(|| AiControlCenterError::InvalidRequest(format!("{field} must be an object")))
}

pub(super) fn string_array_value(
    values: Vec<String>,
    field: &'static str,
) -> Result<Vec<String>, AiControlCenterError> {
    let mut cleaned = Vec::new();
    for value in values {
        validate_non_empty(field, &value)?;
        let value = value.trim().to_owned();
        if !cleaned.contains(&value) {
            cleaned.push(value);
        }
    }
    Ok(cleaned)
}

pub(super) fn json_string_array(value: Value) -> Result<Vec<String>, AiControlCenterError> {
    let Some(items) = value.as_array() else {
        return Err(AiControlCenterError::InvalidRequest(
            "value must be an array".to_owned(),
        ));
    };
    items
        .iter()
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                AiControlCenterError::InvalidRequest("array item must be a string".to_owned())
            })
        })
        .collect()
}

pub(super) fn json_array(value: Value) -> Result<Vec<Value>, AiControlCenterError> {
    value
        .as_array()
        .cloned()
        .ok_or_else(|| AiControlCenterError::InvalidRequest("value must be an array".to_owned()))
}

pub(super) fn reject_secret_like_json(value: &Value) -> Result<(), AiControlCenterError> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let normalized = key.to_ascii_lowercase();
                if normalized.contains("secret")
                    || normalized.contains("password")
                    || normalized.contains("token")
                    || normalized.contains("credential")
                    || normalized.contains("private_key")
                    || normalized == "body"
                    || normalized == "html"
                    || normalized == "raw"
                {
                    return Err(AiControlCenterError::SecretLikePayload);
                }
                reject_secret_like_json(child)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                reject_secret_like_json(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn render_prompt(template: &str, variables: &Map<String, Value>) -> String {
    let mut rendered = template.to_owned();
    for (key, value) in variables {
        let replacement = value
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| value.to_string());
        rendered = rendered.replace(&format!("{{{{{key}}}}}"), &replacement);
    }
    rendered
}

pub(super) fn slug_id(value: &str) -> String {
    let mut slug = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        slug = Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .to_string();
    }
    slug
}
