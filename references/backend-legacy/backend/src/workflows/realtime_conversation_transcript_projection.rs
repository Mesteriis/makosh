use makosh_events_api::StoredEventEnvelope;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::calendar::ports::{EventRecordingPort, EventTranscriptPort};
use crate::domains::documents::core::models::NewDocumentImport;
use crate::domains::documents::ports::DocumentImportPort;
use crate::platform::realtime_conversation::events::REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED;
use crate::platform::realtime_conversation::models::CallBundleManifest;
use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

pub const REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER: &str =
    "realtime_conversation_transcript_projection";

#[derive(Debug, Error)]
pub enum RealtimeConversationTranscriptProjectionError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    DocumentImport(#[from] crate::domains::documents::core::errors::DocumentImportError),

    #[error(transparent)]
    Observation(#[from] makosh_observations_postgres::errors::ObservationStoreError),

    #[error(transparent)]
    Meetings(#[from] crate::domains::calendar::meetings::errors::MeetingsError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),

    #[error("event payload field {field} is invalid: {value}")]
    InvalidPayloadField { field: &'static str, value: String },
}

pub async fn project_realtime_conversation_transcript_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_realtime_conversation_transcript_event_inner(&pool, event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

async fn project_realtime_conversation_transcript_event_inner(
    pool: &PgPool,
    event: StoredEventEnvelope,
) -> Result<(), RealtimeConversationTranscriptProjectionError> {
    if event.event.event_type != REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED {
        return Ok(());
    }

    let projection = TranscriptProjectionPayload::from_payload(&event.event.payload)?;
    let transcript_markdown = fs::read_to_string(&projection.transcript_markdown_path)?;
    let transcript_json: Value =
        serde_json::from_str(&fs::read_to_string(&projection.transcript_json_path)?)?;
    let manifest: CallBundleManifest =
        serde_json::from_str(&fs::read_to_string(&projection.manifest_path)?)?;

    let mut transaction = pool.begin().await?;
    let observation = ObservationStore::capture_in_transaction(
        &mut transaction,
        &NewObservation::new(
            "MEETING_TRANSCRIPT",
            ObservationOriginKind::LocalRuntime,
            event.event.occurred_at,
            json!({
                "bundle_id": projection.bundle_id,
                "account_id": projection.account_id,
                "conference_id": projection.conference_id,
                "calendar_event_id": manifest.calendar_event_id,
                "provider_kind": manifest.provider_kind.as_str(),
                "language_code": projection.language_code,
                "stt_provider": projection.stt_provider,
                "confidence": projection.confidence,
                "summary": projection.summary,
                "segment_count": projection.segment_count,
                "transcript": transcript_json,
            }),
            format!("call-bundle://{}/transcript", projection.bundle_id),
        )
        .confidence(projection.confidence)
        .provenance(json!({
            "captured_by": "realtime_conversation_transcript_projection",
            "event_id": event.event.event_id,
            "transcript_json_path": projection.transcript_json_path,
            "transcript_markdown_path": projection.transcript_markdown_path,
        })),
    )
    .await?;

    let document_id = transcript_document_id(&projection.bundle_id);
    let document = NewDocumentImport::markdown(
        &document_id,
        transcript_document_title(&manifest, &projection),
        transcript_markdown,
    );
    let _ = DocumentImportPort::import_document_manual_with_observation_in_transaction(
        &mut transaction,
        &document,
        format!("call-bundle://{}/transcript-document", projection.bundle_id),
        json!({
            "captured_by": "realtime_conversation_transcript_projection",
            "event_id": event.event.event_id,
            "bundle_id": projection.bundle_id,
        }),
        Some(&observation.observation_id),
        Some("transcript_projection"),
        Some(json!({
            "bundle_id": projection.bundle_id,
            "document_role": "meeting_transcript",
        })),
    )
    .await?;

    if let Some(calendar_event_id) = manifest
        .calendar_event_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let audio_path = bundle_audio_path(&manifest);
        let recording = match EventRecordingPort::find_by_file_path_in_transaction(
            &mut transaction,
            calendar_event_id,
            &audio_path,
        )
        .await?
        {
            Some(recording) => recording,
            None => {
                EventRecordingPort::add_with_observation_in_transaction(
                    &mut transaction,
                    calendar_event_id,
                    Some(&audio_path),
                    Some("yandex_telemost_local_recording"),
                    None,
                    Some(&observation.observation_id),
                )
                .await?
            }
        };
        let event_transcript = EventTranscriptPort::add_with_observation_in_transaction(
            &mut transaction,
            calendar_event_id,
            &projection.transcript_text,
            projection.language_code.as_deref(),
            projection.summary.as_deref(),
            Some(&projection.stt_provider),
            Some(&observation.observation_id),
        )
        .await?;
        let _ = EventRecordingPort::attach_transcript_in_transaction(
            &mut transaction,
            &recording.id,
            &event_transcript.id,
            Some(&observation.observation_id),
        )
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

#[derive(Clone, Debug)]
struct TranscriptProjectionPayload {
    bundle_id: String,
    account_id: String,
    conference_id: Option<String>,
    manifest_path: String,
    transcript_json_path: String,
    transcript_markdown_path: String,
    transcript_text: String,
    language_code: Option<String>,
    stt_provider: String,
    confidence: f64,
    summary: Option<String>,
    segment_count: usize,
}

impl TranscriptProjectionPayload {
    fn from_payload(
        payload: &Value,
    ) -> Result<Self, RealtimeConversationTranscriptProjectionError> {
        Ok(Self {
            bundle_id: required_string(payload, "bundle_id")?,
            account_id: required_string(payload, "account_id")?,
            conference_id: optional_string(payload, "conference_id"),
            manifest_path: required_string(payload, "manifest_path")?,
            transcript_json_path: required_string(payload, "transcript_json_path")?,
            transcript_markdown_path: required_string(payload, "transcript_markdown_path")?,
            transcript_text: required_string(payload, "transcript_text")?,
            language_code: optional_string(payload, "language_code"),
            stt_provider: required_string(payload, "stt_provider")?,
            confidence: optional_f64(payload, "confidence").unwrap_or(0.82),
            summary: optional_string(payload, "summary"),
            segment_count: payload
                .get("segment_count")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize,
        })
    }
}

fn transcript_document_id(bundle_id: &str) -> String {
    format!("realtime-conversation-transcript:{bundle_id}")
}

fn transcript_document_title(
    manifest: &CallBundleManifest,
    projection: &TranscriptProjectionPayload,
) -> String {
    if let Some(conference_id) = projection
        .conference_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("Telemost transcript {conference_id}");
    }
    if let Some(provider_conference_id) = manifest
        .provider_conference_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("Telemost transcript {provider_conference_id}");
    }
    format!("Telemost transcript {}", projection.bundle_id)
}

fn bundle_audio_path(manifest: &CallBundleManifest) -> String {
    Path::new(&manifest.layout.root)
        .join(&manifest.layout.audio_mp3)
        .to_string_lossy()
        .into_owned()
}

fn required_string(
    payload: &Value,
    field: &'static str,
) -> Result<String, RealtimeConversationTranscriptProjectionError> {
    let value = payload
        .get(field)
        .ok_or(RealtimeConversationTranscriptProjectionError::MissingPayloadField(field))?;
    let value = value.as_str().ok_or_else(|| {
        RealtimeConversationTranscriptProjectionError::InvalidPayloadField {
            field,
            value: value.to_string(),
        }
    })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(
            RealtimeConversationTranscriptProjectionError::InvalidPayloadField {
                field,
                value: value.to_owned(),
            },
        );
    }
    if !Path::new(trimmed).is_absolute()
        && matches!(
            field,
            "manifest_path" | "transcript_json_path" | "transcript_markdown_path"
        )
    {
        return Err(
            RealtimeConversationTranscriptProjectionError::InvalidPayloadField {
                field,
                value: value.to_owned(),
            },
        );
    }
    Ok(trimmed.to_owned())
}

fn optional_string(payload: &Value, field: &'static str) -> Option<String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn optional_f64(payload: &Value, field: &'static str) -> Option<f64> {
    payload.get(field).and_then(Value::as_f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_document_id_is_stable() {
        assert_eq!(
            transcript_document_id("bundle-1"),
            "realtime-conversation-transcript:bundle-1"
        );
    }

    #[test]
    fn transcript_projection_payload_requires_absolute_paths() {
        let payload = json!({
            "bundle_id": "bundle-1",
            "account_id": "telemost-main",
            "manifest_path": "manifest.json",
            "transcript_json_path": "/tmp/transcript.json",
            "transcript_markdown_path": "/tmp/transcript.md",
            "stt_provider": "whisper-local"
        });

        let error = TranscriptProjectionPayload::from_payload(&payload).expect_err("invalid path");

        assert!(error.to_string().contains("manifest_path"));
    }
}
