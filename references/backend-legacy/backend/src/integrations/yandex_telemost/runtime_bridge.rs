use makosh_events_api::NewEventEnvelope;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use makosh_provider_telemost::protocol::validate_required;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::integrations::yandex_telemost::client::errors::YandexTelemostError;
use crate::integrations::yandex_telemost::client::models::{
    YandexTelemostTranscriptBridgeRequest, YandexTelemostTranscriptBridgeResponse,
};
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::realtime_conversation::events::REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED;
use crate::platform::realtime_conversation::models::{CallBundleArtifact, CallBundleManifest};
use makosh_events_postgres::store::EventStore;

pub(crate) struct MaterializedTelemostTranscriptBundle {
    pub(crate) bundle_root: PathBuf,
    pub(crate) manifest_path: PathBuf,
    pub(crate) manifest: CallBundleManifest,
    pub(crate) transcript_json_path: PathBuf,
    pub(crate) transcript_markdown_path: PathBuf,
    pub(crate) summary_markdown_path: Option<PathBuf>,
}

pub(crate) async fn complete_yandex_telemost_transcript_bridge(
    event_store: &EventStore,
    event_bus: Option<&InMemoryEventBus>,
    request: &YandexTelemostTranscriptBridgeRequest,
) -> Result<YandexTelemostTranscriptBridgeResponse, YandexTelemostError> {
    let materialized = materialize_yandex_telemost_transcript_artifacts(request)?;
    publish_realtime_conversation_transcript_completed_event(
        event_store,
        event_bus,
        request,
        &materialized,
    )
    .await?;
    Ok(YandexTelemostTranscriptBridgeResponse {
        account_id: request.account_id.clone(),
        conference_id: request.conference_id.clone(),
        bundle_id: materialized.manifest.bundle_id.clone(),
        manifest_path: materialized.manifest_path.to_string_lossy().into_owned(),
        transcript_json_path: materialized
            .transcript_json_path
            .to_string_lossy()
            .into_owned(),
        transcript_markdown_path: materialized
            .transcript_markdown_path
            .to_string_lossy()
            .into_owned(),
        summary_markdown_path: materialized
            .summary_markdown_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        follow_up_events: vec![REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED.to_owned()],
    })
}

pub(crate) fn materialize_yandex_telemost_transcript_artifacts(
    request: &YandexTelemostTranscriptBridgeRequest,
) -> Result<MaterializedTelemostTranscriptBundle, YandexTelemostError> {
    validate_yandex_telemost_transcript_bridge_request(request)?;
    let bundle_root = canonical_existing_dir("bundle_root", &request.bundle_root)?;
    let manifest_path = canonical_existing_file(
        "manifest_path",
        &bundle_root.join("manifest.json").to_string_lossy(),
        &bundle_root,
    )?;
    let mut manifest: CallBundleManifest =
        serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    if manifest.bundle_id.trim() != request.bundle_id.trim() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "bundle_id `{}` does not match manifest bundle_id `{}`",
            request.bundle_id, manifest.bundle_id
        )));
    }
    if manifest.account_id.trim() != request.account_id.trim() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "account_id `{}` does not match manifest account_id `{}`",
            request.account_id, manifest.account_id
        )));
    }

    let transcript_json_path = bundle_root.join(&manifest.layout.transcript_json);
    let transcript_markdown_path = bundle_root.join(&manifest.layout.transcript_markdown);
    let summary_markdown_path = request
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|_| bundle_root.join(&manifest.layout.summary_markdown));
    let transcript_json_relative_path = manifest.layout.transcript_json.clone();
    let transcript_markdown_relative_path = manifest.layout.transcript_markdown.clone();
    let summary_markdown_relative_path = manifest.layout.summary_markdown.clone();
    let transcript_json = json!({
        "bundle_id": manifest.bundle_id,
        "provider_kind": manifest.provider_kind.as_str(),
        "conference_id": request.conference_id,
        "language_code": request.language_code,
        "stt_provider": request.stt_provider,
        "confidence": normalized_transcript_confidence(request),
        "summary": request.summary,
        "segments": request.segments,
        "metadata": request.metadata,
        "transcript_text": request.transcript_text,
    });
    fs::write(
        &transcript_json_path,
        serde_json::to_string_pretty(&transcript_json)?,
    )?;
    fs::write(
        &transcript_markdown_path,
        render_transcript_markdown(request),
    )?;
    if let Some(path) = &summary_markdown_path {
        fs::write(path, render_summary_markdown(request))?;
    }

    upsert_bundle_artifact(
        &mut manifest,
        CallBundleArtifact {
            kind: "transcript".to_owned(),
            relative_path: transcript_json_relative_path,
            source: request.stt_provider.trim().to_owned(),
            truth_status: "model_output".to_owned(),
            media_type: Some("application/json".to_owned()),
            description: Some("Structured transcript with evidence metadata".to_owned()),
        },
    );
    upsert_bundle_artifact(
        &mut manifest,
        CallBundleArtifact {
            kind: "transcript_markdown".to_owned(),
            relative_path: transcript_markdown_relative_path,
            source: request.stt_provider.trim().to_owned(),
            truth_status: "model_output".to_owned(),
            media_type: Some("text/markdown".to_owned()),
            description: Some("Owner-readable transcript markdown projection".to_owned()),
        },
    );
    if summary_markdown_path.is_some() {
        upsert_bundle_artifact(
            &mut manifest,
            CallBundleArtifact {
                kind: "summary_markdown".to_owned(),
                relative_path: summary_markdown_relative_path,
                source: request.stt_provider.trim().to_owned(),
                truth_status: "model_output".to_owned(),
                media_type: Some("text/markdown".to_owned()),
                description: Some("Owner-readable transcript summary".to_owned()),
            },
        );
    }
    manifest.pipeline_state.transcription = "completed".to_owned();
    manifest.pipeline_state.diarization =
        if transcript_segments_have_speaker_labels(&request.segments) {
            "completed_with_speaker_segments".to_owned()
        } else {
            "completed_without_speaker_labels".to_owned()
        };
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    Ok(MaterializedTelemostTranscriptBundle {
        bundle_root,
        manifest_path,
        manifest,
        transcript_json_path,
        transcript_markdown_path,
        summary_markdown_path,
    })
}

async fn publish_realtime_conversation_transcript_completed_event(
    event_store: &EventStore,
    event_bus: Option<&InMemoryEventBus>,
    request: &YandexTelemostTranscriptBridgeRequest,
    materialized: &MaterializedTelemostTranscriptBundle,
) -> Result<(), YandexTelemostError> {
    let event = NewEventEnvelope::builder(
        format!(
            "realtime-conversation-transcript-completed-{}-{}",
            materialized.manifest.bundle_id,
            Uuid::new_v4()
        ),
        REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED,
        Utc::now(),
        json!({ "source_code": "workflow.realtime_conversation", "provider": "yandex_telemost" }),
        json!({ "kind": "call_bundle", "entity_id": materialized.manifest.bundle_id }),
    )
    .payload(json!({
        "bundle_id": materialized.manifest.bundle_id,
        "account_id": request.account_id,
        "conference_id": request.conference_id,
        "calendar_event_id": materialized.manifest.calendar_event_id,
        "manifest_path": materialized.manifest_path.to_string_lossy(),
        "bundle_root": materialized.bundle_root.to_string_lossy(),
        "transcript_json_path": materialized.transcript_json_path.to_string_lossy(),
        "transcript_markdown_path": materialized.transcript_markdown_path.to_string_lossy(),
        "summary_markdown_path": materialized.summary_markdown_path.as_ref().map(|path| path.to_string_lossy().into_owned()),
        "language_code": request.language_code,
        "stt_provider": request.stt_provider,
        "confidence": normalized_transcript_confidence(request),
        "summary": request.summary,
        "transcript_text": request.transcript_text,
        "segment_count": request.segments.as_array().map(|value| value.len()).unwrap_or(0),
        "metadata": request.metadata,
    }))
    .provenance(json!({ "origin": "telemost_transcript_runtime_bridge" }))
    .correlation_id(format!(
        "realtime-conversation:{}",
        materialized.manifest.bundle_id
    ))
    .build()?;
    if event_store
        .append_for_dispatch_idempotent(&event)
        .await?
        .is_some()
        && let Some(event_bus) = event_bus
    {
        event_bus.broadcast(event);
    }
    Ok(())
}

fn validate_yandex_telemost_transcript_bridge_request(
    request: &YandexTelemostTranscriptBridgeRequest,
) -> Result<(), YandexTelemostError> {
    validate_required("account_id", &request.account_id)?;
    validate_required("bundle_id", &request.bundle_id)?;
    validate_required("transcript_text", &request.transcript_text)?;
    validate_required("stt_provider", &request.stt_provider)?;
    if !request.segments.is_array() {
        return Err(YandexTelemostError::InvalidRequest(
            "segments must be a JSON array".to_owned(),
        ));
    }
    if !request.metadata.is_object() {
        return Err(YandexTelemostError::InvalidRequest(
            "metadata must be a JSON object".to_owned(),
        ));
    }
    if let Some(confidence) = request.confidence
        && !(0.0..=1.0).contains(&confidence)
    {
        return Err(YandexTelemostError::InvalidRequest(
            "confidence must be between 0.0 and 1.0".to_owned(),
        ));
    }
    Ok(())
}

fn upsert_bundle_artifact(manifest: &mut CallBundleManifest, artifact: CallBundleArtifact) {
    if let Some(existing) = manifest
        .artifacts
        .iter_mut()
        .find(|item| item.kind == artifact.kind)
    {
        *existing = artifact;
        return;
    }
    manifest.artifacts.push(artifact);
}

fn render_transcript_markdown(request: &YandexTelemostTranscriptBridgeRequest) -> String {
    let mut lines = vec![
        "# Transcript".to_owned(),
        String::new(),
        format!("- Bundle: `{}`", request.bundle_id),
        format!("- Account: `{}`", request.account_id),
        format!("- STT provider: `{}`", request.stt_provider.trim()),
    ];
    if let Some(conference_id) = request
        .conference_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        lines.push(format!("- Conference: `{conference_id}`"));
    }
    if let Some(language_code) = request
        .language_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        lines.push(format!("- Language: `{language_code}`"));
    }
    lines.push(String::new());
    lines.push("## Full text".to_owned());
    lines.push(String::new());
    lines.push(request.transcript_text.trim().to_owned());
    lines.join("\n")
}

fn render_summary_markdown(request: &YandexTelemostTranscriptBridgeRequest) -> String {
    let summary = request
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_default();
    format!("# Summary\n\n{summary}\n")
}

fn transcript_segments_have_speaker_labels(segments: &Value) -> bool {
    segments.as_array().is_some_and(|items| {
        items.iter().any(|segment| {
            segment
                .get("speaker")
                .and_then(Value::as_str)
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
        })
    })
}

fn normalized_transcript_confidence(request: &YandexTelemostTranscriptBridgeRequest) -> f64 {
    request.confidence.unwrap_or(0.82).clamp(0.0, 1.0)
}

fn canonical_existing_dir(name: &str, value: &str) -> Result<PathBuf, YandexTelemostError> {
    let required = require_non_empty_path(name, value)?;
    let path = Path::new(&required);
    if !path.exists() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} `{}` does not exist",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} `{}` must be a directory",
            path.display()
        )));
    }
    Ok(path.canonicalize()?)
}

fn canonical_existing_file(
    name: &str,
    value: &str,
    expected_root: &Path,
) -> Result<PathBuf, YandexTelemostError> {
    let required = require_non_empty_path(name, value)?;
    let path = Path::new(&required);
    if !path.exists() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} `{}` does not exist",
            path.display()
        )));
    }
    if !path.is_file() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} `{}` must be a file",
            path.display()
        )));
    }
    let canonical = path.canonicalize()?;
    if !canonical.starts_with(expected_root) {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} `{}` must stay under output_dir `{}`",
            canonical.display(),
            expected_root.display()
        )));
    }
    Ok(canonical)
}

fn require_non_empty_path(name: &str, value: &str) -> Result<String, YandexTelemostError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}
