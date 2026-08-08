use makosh_events_api::StoredEventEnvelope;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;
use tokio::task::spawn_blocking;

use crate::application::realtime_conversation_transcript_execution::{
    RealtimeConversationTranscriptBridgeError, RealtimeConversationTranscriptBridgeRequest,
    complete_realtime_conversation_transcript_bridge,
};
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::realtime_conversation::events::REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED;
use crate::platform::realtime_conversation::models::CallBundleManifest;
use makosh_events_postgres::errors::EventStoreError;

pub const REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_CONSUMER: &str =
    "realtime_conversation_transcript_execution";
const TRANSCRIBER_PATH_ENV: &str = "MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER";
const TRANSCRIBER_ARGS_JSON_ENV: &str = "MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER_ARGS_JSON";
const TRANSCRIBER_TIMEOUT_SECONDS_ENV: &str =
    "MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER_TIMEOUT_SECONDS";
const DEFAULT_TRANSCRIBER_TIMEOUT_SECONDS: u64 = 900;

#[derive(Debug, Error)]
pub enum RealtimeConversationTranscriptExecutionError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error(transparent)]
    TranscriptBridge(#[from] RealtimeConversationTranscriptBridgeError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),

    #[error("event payload field {field} is invalid: {value}")]
    InvalidPayloadField { field: &'static str, value: String },

    #[error("{0} is not configured")]
    MissingConfiguration(&'static str),

    #[error("{0} must be a JSON string array or integer seconds")]
    InvalidConfiguration(&'static str),

    #[error("transcript execution only supports provider `{expected}`, got `{actual}`")]
    UnsupportedProvider {
        expected: &'static str,
        actual: String,
    },

    #[error("transcriber command timed out after {0} seconds")]
    CommandTimeout(u64),

    #[error("transcriber command failed with status {status}: {stderr}")]
    CommandFailed { status: i32, stderr: String },
}

pub async fn execute_realtime_conversation_transcript_request_event(
    pool: PgPool,
    event_bus: InMemoryEventBus,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    execute_realtime_conversation_transcript_request_event_inner(&pool, &event_bus, event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub fn realtime_conversation_transcriber_is_configured() -> bool {
    std::env::var(TRANSCRIBER_PATH_ENV)
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

async fn execute_realtime_conversation_transcript_request_event_inner(
    pool: &PgPool,
    event_bus: &InMemoryEventBus,
    event: StoredEventEnvelope,
) -> Result<(), RealtimeConversationTranscriptExecutionError> {
    if event.event.event_type != REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED {
        return Ok(());
    }

    let payload = TranscriptExecutionPayload::from_payload(&event.event.payload)?;
    if payload.provider_kind != "yandex_telemost" {
        return Err(
            RealtimeConversationTranscriptExecutionError::UnsupportedProvider {
                expected: "yandex_telemost",
                actual: payload.provider_kind,
            },
        );
    }

    let manifest: CallBundleManifest =
        serde_json::from_str(&fs::read_to_string(&payload.manifest_path)?)?;
    let output = run_transcriber_command(&payload, &manifest).await?;
    let request = RealtimeConversationTranscriptBridgeRequest {
        account_id: payload.account_id,
        conference_id: payload.conference_id,
        bundle_id: payload.bundle_id,
        bundle_root: payload.bundle_root,
        transcript_text: output.transcript_text,
        segments: output.segments,
        language_code: output.language_code,
        stt_provider: output.stt_provider,
        summary: output.summary,
        confidence: output.confidence,
        metadata: output.metadata,
    };
    let event_store = makosh_events_postgres::store::EventStore::new(pool.clone());
    complete_realtime_conversation_transcript_bridge(&event_store, Some(event_bus), &request)
        .await?;
    Ok(())
}

#[derive(Clone, Debug)]
struct TranscriptExecutionPayload {
    bundle_id: String,
    account_id: String,
    conference_id: Option<String>,
    provider_kind: String,
    bundle_root: String,
    manifest_path: String,
    audio_path: String,
}

impl TranscriptExecutionPayload {
    fn from_payload(payload: &Value) -> Result<Self, RealtimeConversationTranscriptExecutionError> {
        Ok(Self {
            bundle_id: required_string(payload, "bundle_id")?,
            account_id: required_string(payload, "account_id")?,
            conference_id: optional_string(payload, "conference_id"),
            provider_kind: required_string(payload, "provider_kind")?,
            bundle_root: required_absolute_path(payload, "bundle_root")?,
            manifest_path: required_absolute_path(payload, "manifest_path")?,
            audio_path: required_absolute_path(payload, "audio_path")?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
struct LocalTranscriptCommandOutput {
    transcript_text: String,
    #[serde(default = "default_json_array")]
    segments: Value,
    #[serde(default)]
    language_code: Option<String>,
    stt_provider: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    confidence: Option<f64>,
    #[serde(default = "default_json_object")]
    metadata: Value,
}

#[derive(Clone, Debug)]
struct LocalTranscriberConfig {
    executable: String,
    args: Vec<String>,
    timeout_seconds: u64,
}

impl LocalTranscriberConfig {
    fn from_env() -> Result<Self, RealtimeConversationTranscriptExecutionError> {
        let executable = std::env::var(TRANSCRIBER_PATH_ENV).map_err(|_| {
            RealtimeConversationTranscriptExecutionError::MissingConfiguration(TRANSCRIBER_PATH_ENV)
        })?;
        let executable = executable.trim().to_owned();
        if executable.is_empty() {
            return Err(
                RealtimeConversationTranscriptExecutionError::MissingConfiguration(
                    TRANSCRIBER_PATH_ENV,
                ),
            );
        }

        let args = match std::env::var(TRANSCRIBER_ARGS_JSON_ENV) {
            Ok(value) if !value.trim().is_empty() => {
                let parsed: Value = serde_json::from_str(&value)?;
                let Some(items) = parsed.as_array() else {
                    return Err(
                        RealtimeConversationTranscriptExecutionError::InvalidConfiguration(
                            TRANSCRIBER_ARGS_JSON_ENV,
                        ),
                    );
                };
                let mut args = Vec::with_capacity(items.len());
                for item in items {
                    let Some(item) = item.as_str() else {
                        return Err(
                            RealtimeConversationTranscriptExecutionError::InvalidConfiguration(
                                TRANSCRIBER_ARGS_JSON_ENV,
                            ),
                        );
                    };
                    args.push(item.to_owned());
                }
                args
            }
            _ => Vec::new(),
        };

        let timeout_seconds = match std::env::var(TRANSCRIBER_TIMEOUT_SECONDS_ENV) {
            Ok(value) if !value.trim().is_empty() => value.trim().parse().map_err(|_| {
                RealtimeConversationTranscriptExecutionError::InvalidConfiguration(
                    TRANSCRIBER_TIMEOUT_SECONDS_ENV,
                )
            })?,
            _ => DEFAULT_TRANSCRIBER_TIMEOUT_SECONDS,
        };

        Ok(Self {
            executable,
            args,
            timeout_seconds,
        })
    }
}

async fn run_transcriber_command(
    payload: &TranscriptExecutionPayload,
    manifest: &CallBundleManifest,
) -> Result<LocalTranscriptCommandOutput, RealtimeConversationTranscriptExecutionError> {
    let config = LocalTranscriberConfig::from_env()?;
    let payload = payload.clone();
    let manifest_json = serde_json::to_string(manifest)?;
    let timeout_seconds = config.timeout_seconds;
    let output = tokio::time::timeout(
        Duration::from_secs(timeout_seconds),
        spawn_blocking(move || {
            let mut command = Command::new(&config.executable);
            command.args(&config.args);
            command.env("MAKOSH_TRANSCRIPT_BUNDLE_ID", &payload.bundle_id);
            command.env("MAKOSH_TRANSCRIPT_ACCOUNT_ID", &payload.account_id);
            command.env(
                "MAKOSH_TRANSCRIPT_CONFERENCE_ID",
                payload.conference_id.as_deref().unwrap_or(""),
            );
            command.env("MAKOSH_TRANSCRIPT_PROVIDER_KIND", &payload.provider_kind);
            command.env("MAKOSH_TRANSCRIPT_BUNDLE_ROOT", &payload.bundle_root);
            command.env("MAKOSH_TRANSCRIPT_MANIFEST_PATH", &payload.manifest_path);
            command.env("MAKOSH_TRANSCRIPT_AUDIO_PATH", &payload.audio_path);
            command.env("MAKOSH_TRANSCRIPT_MANIFEST_JSON", manifest_json);
            command.output()
        }),
    )
    .await
    .map_err(|_| RealtimeConversationTranscriptExecutionError::CommandTimeout(timeout_seconds))??;
    let output = output?;

    if !output.status.success() {
        return Err(
            RealtimeConversationTranscriptExecutionError::CommandFailed {
                status: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            },
        );
    }

    Ok(serde_json::from_slice(&output.stdout)?)
}

fn required_string(
    payload: &Value,
    field: &'static str,
) -> Result<String, RealtimeConversationTranscriptExecutionError> {
    let value = payload
        .get(field)
        .ok_or(RealtimeConversationTranscriptExecutionError::MissingPayloadField(field))?;
    let value = value.as_str().ok_or_else(|| {
        RealtimeConversationTranscriptExecutionError::InvalidPayloadField {
            field,
            value: value.to_string(),
        }
    })?;
    let value = value.trim();
    if value.is_empty() {
        return Err(
            RealtimeConversationTranscriptExecutionError::InvalidPayloadField {
                field,
                value: String::new(),
            },
        );
    }
    Ok(value.to_owned())
}

fn optional_string(payload: &Value, field: &'static str) -> Option<String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn required_absolute_path(
    payload: &Value,
    field: &'static str,
) -> Result<String, RealtimeConversationTranscriptExecutionError> {
    let value = required_string(payload, field)?;
    if !Path::new(&value).is_absolute() {
        return Err(
            RealtimeConversationTranscriptExecutionError::InvalidPayloadField { field, value },
        );
    }
    Ok(value)
}

fn default_json_array() -> Value {
    json!([])
}

fn default_json_object() -> Value {
    json!({})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_execution_payload_requires_absolute_paths() {
        let error = TranscriptExecutionPayload::from_payload(&json!({
            "bundle_id": "bundle-1",
            "account_id": "telemost-main",
            "provider_kind": "yandex_telemost",
            "bundle_root": "relative/root",
            "manifest_path": "/tmp/manifest.json",
            "audio_path": "/tmp/audio.mp3"
        }))
        .expect_err("relative bundle_root must fail");

        assert!(error.to_string().contains("bundle_root"));
    }
}
