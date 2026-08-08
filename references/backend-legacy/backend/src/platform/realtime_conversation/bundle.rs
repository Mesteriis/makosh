use chrono::Utc;
use serde_json::json;

use super::models::{
    CallBundleArtifact, CallBundleLayout, CallBundleManifest, CallBundlePipelineState,
    CallBundlePrivacyPolicy, RealtimeConversationProviderKind,
};

pub fn default_call_bundle_layout(root: impl Into<String>) -> CallBundleLayout {
    let root = root.into();
    CallBundleLayout {
        root,
        manifest: "manifest.json".to_owned(),
        meeting_json: "meeting.json".to_owned(),
        provider_json: "provider.json".to_owned(),
        participants_json: "participants.json".to_owned(),
        audio_mp3: "audio.mp3".to_owned(),
        speaker_hints_jsonl: "speaker-hints.jsonl".to_owned(),
        speaker_timeline_txt: "speaker-timeline.txt".to_owned(),
        event_track_jsonl: "event-track.jsonl".to_owned(),
        chat_json: "chat.json".to_owned(),
        transcript_json: "transcript.json".to_owned(),
        transcript_markdown: "transcript.md".to_owned(),
        summary_markdown: "summary.md".to_owned(),
        topics_json: "topics.json".to_owned(),
        entities_json: "entities.json".to_owned(),
        decisions_json: "decisions.json".to_owned(),
        tasks_json: "tasks.json".to_owned(),
        knowledge_json: "knowledge.json".to_owned(),
        metrics_json: "metrics.json".to_owned(),
        radar_signals_json: "radar-signals.json".to_owned(),
        screenshots_dir: "screenshots".to_owned(),
        attachments_dir: "attachments".to_owned(),
        ocr_dir: "ocr".to_owned(),
    }
}

pub fn build_call_bundle_manifest(
    bundle_id: impl Into<String>,
    provider_kind: RealtimeConversationProviderKind,
    provider_shape: impl Into<String>,
    account_id: impl Into<String>,
    provider_conference_id: Option<String>,
    join_url: Option<String>,
    root: impl Into<String>,
) -> CallBundleManifest {
    let layout = default_call_bundle_layout(root);
    CallBundleManifest {
        schema_version: 1,
        bundle_id: bundle_id.into(),
        provider_kind,
        provider_shape: provider_shape.into(),
        account_id: account_id.into(),
        provider_conference_id,
        join_url,
        calendar_event_id: None,
        project_id: None,
        organization_id: None,
        created_at: Utc::now(),
        artifacts: vec![
            CallBundleArtifact {
                kind: "audio".to_owned(),
                relative_path: layout.audio_mp3.clone(),
                source: "local_audio_loopback".to_owned(),
                truth_status: "capture_artifact".to_owned(),
                media_type: Some("audio/mpeg".to_owned()),
                description: Some(
                    "Local MP3 recording used by the transcription pipeline".to_owned(),
                ),
            },
            CallBundleArtifact {
                kind: "speaker_hints".to_owned(),
                relative_path: layout.speaker_hints_jsonl.clone(),
                source: "visible_webview_dom_heuristic".to_owned(),
                truth_status: "hint_not_truth".to_owned(),
                media_type: Some("application/x-ndjson".to_owned()),
                description: Some(
                    "Warm-start hints for diarization and speaker identity merging".to_owned(),
                ),
            },
            CallBundleArtifact {
                kind: "event_track".to_owned(),
                relative_path: layout.event_track_jsonl.clone(),
                source: "local_runtime".to_owned(),
                truth_status: "observed_runtime_event".to_owned(),
                media_type: Some("application/x-ndjson".to_owned()),
                description: Some("Meeting lifecycle and capture events".to_owned()),
            },
        ],
        layout,
        pipeline_state: CallBundlePipelineState::queued_from_local_recording(),
        privacy_policy: CallBundlePrivacyPolicy::local_visible_capture(),
        provenance: json!({
            "source": "makosh_realtime_conversation_bundle_builder",
            "single_source_of_truth": false,
            "notes": "Provider DOM speaker state is only a hint for later AI processing."
        }),
    }
}
