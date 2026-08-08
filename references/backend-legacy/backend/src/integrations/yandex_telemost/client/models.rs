use chrono::{DateTime, Utc};
use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use makosh_provider_telemost::protocol::{
    YANDEX_TELEMOST_API_BASE_URL, YANDEX_TELEMOST_PROVIDER_KIND_STR,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::integrations::yandex_telemost::client::errors::YandexTelemostError;

pub const YANDEX_TELEMOST_RUNTIME_KIND: &str = "yandex_telemost_webview_runtime";
pub const YANDEX_TELEMOST_LIVE_RUNTIME_KIND: &str = "yandex_telemost_live_authorized_runtime";
pub const YANDEX_TELEMOST_WEB_ORIGIN: &str = "https://telemost.yandex.ru";

pub const YANDEX_TELEMOST_CAP_CONFERENCE_CREATE: &str = "telemost.conferences.create";
pub const YANDEX_TELEMOST_CAP_CONFERENCE_READ: &str = "telemost.conferences.read";
pub const YANDEX_TELEMOST_CAP_CONFERENCE_UPDATE: &str = "telemost.conferences.update";
pub const YANDEX_TELEMOST_CAP_COHOSTS_READ: &str = "telemost.cohosts.read";
pub const YANDEX_TELEMOST_CAP_WEBVIEW_OPEN: &str = "telemost.webview.open";
pub const YANDEX_TELEMOST_CAP_LOCAL_RECORDING: &str = "telemost.local_recording.mp3";
pub const YANDEX_TELEMOST_CAP_SPEAKER_TIMELINE_HINTS: &str =
    "telemost.speaker_timeline.webview_hints";

fn default_json_object() -> Value {
    json!({})
}

fn default_json_array() -> Value {
    json!([])
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostAccount {
    pub account_id: String,
    pub provider_kind: String,
    pub display_name: String,
    pub external_account_id: String,
    pub lifecycle_state: String,
    pub runtime_kind: String,
    pub api_base_url: String,
    pub token_secret_ref: Option<String>,
    pub join_webview_available: bool,
    pub local_recorder_available: bool,
    pub config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProviderAccount> for YandexTelemostAccount {
    fn from(account: ProviderAccount) -> Self {
        let config = account.config.clone();
        Self {
            account_id: account.account_id,
            provider_kind: account.provider_kind.as_str().to_owned(),
            display_name: account.display_name,
            external_account_id: account.external_account_id,
            lifecycle_state: config
                .get("lifecycle_state")
                .and_then(Value::as_str)
                .unwrap_or("blocked")
                .to_owned(),
            runtime_kind: config
                .get("runtime_kind")
                .and_then(Value::as_str)
                .unwrap_or(YANDEX_TELEMOST_RUNTIME_KIND)
                .to_owned(),
            api_base_url: config
                .get("api_base_url")
                .and_then(Value::as_str)
                .unwrap_or(YANDEX_TELEMOST_API_BASE_URL)
                .to_owned(),
            token_secret_ref: config
                .get("token_secret_ref")
                .and_then(Value::as_str)
                .map(str::to_owned),
            join_webview_available: config
                .get("join_webview_available")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            local_recorder_available: config
                .get("local_recorder_available")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            config,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostAccountSetupRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    #[serde(default)]
    pub oauth_token: Option<String>,
    #[serde(default)]
    pub oauth_token_ref: Option<String>,
    #[serde(default)]
    pub api_base_url: Option<String>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostAccountSetupResponse {
    pub account: YandexTelemostAccount,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostAccountListResponse {
    pub items: Vec<YandexTelemostAccount>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostRuntimeStatus {
    pub account_id: String,
    pub provider_kind: String,
    pub lifecycle_state: String,
    pub runtime_kind: String,
    pub checked_at: DateTime<Utc>,
    pub api_base_url: String,
    pub authorized: bool,
    pub blockers: Vec<String>,
    pub capabilities: Vec<YandexTelemostCapabilityState>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostCapabilityState {
    pub capability: String,
    pub status: String,
    pub source: String,
    pub confidence: f32,
    pub evidence: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostConferenceOpenRequest {
    pub account_id: String,
    #[serde(default)]
    pub conference_id: Option<String>,
    pub join_url: String,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YandexTelemostConferenceWebviewManifest {
    pub account_id: String,
    pub conference_id: Option<String>,
    pub join_url: String,
    pub target_origin: &'static str,
    pub provider_shape: &'static str,
    pub runtime_kind: &'static str,
    pub window_label: String,
    pub opened_window: bool,
    pub focused_existing_window: bool,
    pub owner_visible: bool,
    pub hidden_headless_mode: &'static str,
    pub local_recording: YandexTelemostLocalRecordingManifest,
    pub speaker_timeline: YandexTelemostSpeakerTimelinePolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YandexTelemostLocalRecordingManifest {
    pub state: &'static str,
    pub audio_format: &'static str,
    pub recorder_boundary: &'static str,
    pub consent_required: bool,
    pub default_output_policy: &'static str,
    pub audio_device_policy: YandexTelemostLocalRecordingPolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YandexTelemostLocalRecordingPolicy {
    pub macos: &'static str,
    pub linux: &'static str,
    pub windows: &'static str,
    pub ffmpeg_path_env: &'static str,
    pub ffmpeg_input_env: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YandexTelemostSpeakerTimelinePolicy {
    pub state: &'static str,
    pub source: &'static str,
    pub reliability: &'static str,
    pub output_files: Vec<&'static str>,
    pub role_in_transcription: &'static str,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostRecordingBridgeRequest {
    pub account_id: String,
    #[serde(default)]
    pub conference_id: Option<String>,
    pub join_url: String,
    pub recording_session_id: String,
    pub output_dir: String,
    pub audio_path: String,
    pub speaker_jsonl_path: String,
    pub speaker_txt_path: String,
    pub started_at_epoch_ms: u128,
    pub stopped_at_epoch_ms: u128,
    pub consent_attested: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostRecordingBridgeResponse {
    pub account_id: String,
    pub conference_id: Option<String>,
    pub recording_session_id: String,
    pub bundle_id: String,
    pub bundle_root: String,
    pub manifest_path: String,
    pub follow_up_events: Vec<String>,
    pub radar_signal_kinds: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostTranscriptBridgeRequest {
    pub account_id: String,
    #[serde(default)]
    pub conference_id: Option<String>,
    pub bundle_id: String,
    pub bundle_root: String,
    pub transcript_text: String,
    #[serde(default = "default_json_array")]
    pub segments: Value,
    #[serde(default)]
    pub language_code: Option<String>,
    pub stt_provider: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostTranscriptBridgeResponse {
    pub account_id: String,
    pub conference_id: Option<String>,
    pub bundle_id: String,
    pub manifest_path: String,
    pub transcript_json_path: String,
    pub transcript_markdown_path: String,
    pub summary_markdown_path: Option<String>,
    pub follow_up_events: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct YandexTelemostRetentionCleanupRequest {
    #[serde(default = "default_true")]
    pub remove_audio: bool,
    #[serde(default = "default_true")]
    pub remove_speaker_hints: bool,
    #[serde(default = "default_yandex_telemost_retention_cleanup_limit")]
    pub limit: i64,
}

impl YandexTelemostRetentionCleanupRequest {
    pub fn validate(&self) -> Result<(), YandexTelemostError> {
        if !self.remove_audio && !self.remove_speaker_hints {
            return Err(YandexTelemostError::InvalidRequest(
                "Yandex Telemost retention cleanup must target audio, speaker hints or both"
                    .to_owned(),
            ));
        }
        if self.limit <= 0 {
            return Err(YandexTelemostError::InvalidRequest(
                "Yandex Telemost retention cleanup limit must be positive".to_owned(),
            ));
        }
        Ok(())
    }

    pub fn limit(&self) -> i64 {
        self.limit.clamp(1, 500)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct YandexTelemostRetentionCleanupItem {
    pub bundle_id: String,
    pub conference_id: Option<String>,
    pub bundle_root: String,
    pub audio_removed: bool,
    pub speaker_jsonl_removed: bool,
    pub speaker_txt_removed: bool,
    pub speaker_hints_removed: bool,
    pub audio_expires_at: Option<DateTime<Utc>>,
    pub speaker_hints_expires_at: Option<DateTime<Utc>>,
    pub removed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct YandexTelemostRetentionCleanupResponse {
    pub account_id: String,
    pub checked_at: DateTime<Utc>,
    pub audio_files_removed: usize,
    pub speaker_hint_files_removed: usize,
    pub bundles_cleaned: usize,
    pub items: Vec<YandexTelemostRetentionCleanupItem>,
}

pub fn yandex_telemost_default_config(token_secret_ref: Option<&str>, api_base_url: &str) -> Value {
    json!({
        "provider_shape": YANDEX_TELEMOST_PROVIDER_KIND_STR,
        "runtime_kind": YANDEX_TELEMOST_RUNTIME_KIND,
        "lifecycle_state": if token_secret_ref.is_some() { "authorized" } else { "blocked" },
        "api_base_url": api_base_url,
        "token_secret_ref": token_secret_ref,
        "join_webview_available": true,
        "local_recorder_available": true,
        "recording_policy": {
            "owner_visible": true,
            "consent_required": true,
            "audio_format": "mp3",
            "speaker_timeline_source": "webview_dom_heuristic_hint_not_truth"
        }
    })
}

pub fn yandex_telemost_capabilities(authorized: bool) -> Vec<YandexTelemostCapabilityState> {
    let provider = "yandex_telemost_api_docs".to_owned();
    let status = if authorized {
        "available"
    } else {
        "blocked_missing_oauth_token"
    }
    .to_owned();
    let confidence = if authorized { 0.85 } else { 0.45 };
    [
        YANDEX_TELEMOST_CAP_CONFERENCE_CREATE,
        YANDEX_TELEMOST_CAP_CONFERENCE_READ,
        YANDEX_TELEMOST_CAP_CONFERENCE_UPDATE,
        YANDEX_TELEMOST_CAP_COHOSTS_READ,
    ]
    .into_iter()
    .map(|capability| YandexTelemostCapabilityState {
        capability: capability.to_owned(),
        status: status.clone(),
        source: provider.clone(),
        confidence,
        evidence: json!({ "requires": "OAuth token with telemost-api scope" }),
    })
    .chain([
        YandexTelemostCapabilityState {
            capability: YANDEX_TELEMOST_CAP_WEBVIEW_OPEN.to_owned(),
            status: "available_desktop_only".to_owned(),
            source: "tauri_visible_webview_contract".to_owned(),
            confidence: 0.7,
            evidence: json!({ "owner_visible": true, "hidden_headless_mode": "forbidden" }),
        },
        YandexTelemostCapabilityState {
            capability: YANDEX_TELEMOST_CAP_LOCAL_RECORDING.to_owned(),
            status: "available_after_explicit_consent".to_owned(),
            source: "tauri_ffmpeg_local_recorder_contract".to_owned(),
            confidence: 0.65,
            evidence: json!({ "format": "mp3", "transcription_source": "local_audio_file" }),
        },
        YandexTelemostCapabilityState {
            capability: YANDEX_TELEMOST_CAP_SPEAKER_TIMELINE_HINTS.to_owned(),
            status: "heuristic_hint_only".to_owned(),
            source: "visible_webview_dom_heuristic".to_owned(),
            confidence: 0.35,
            evidence: json!({ "truth_source": false, "role": "whisper diarization hint" }),
        },
    ])
    .collect()
}

pub fn webview_manifest_for_request(
    request: &YandexTelemostConferenceOpenRequest,
    window_label: String,
    opened_window: bool,
    focused_existing_window: bool,
) -> YandexTelemostConferenceWebviewManifest {
    YandexTelemostConferenceWebviewManifest {
        account_id: request.account_id.trim().to_owned(),
        conference_id: request.conference_id.clone(),
        join_url: request.join_url.trim().to_owned(),
        target_origin: YANDEX_TELEMOST_WEB_ORIGIN,
        provider_shape: YANDEX_TELEMOST_PROVIDER_KIND_STR,
        runtime_kind: YANDEX_TELEMOST_RUNTIME_KIND,
        window_label,
        opened_window,
        focused_existing_window,
        owner_visible: true,
        hidden_headless_mode: "forbidden",
        local_recording: YandexTelemostLocalRecordingManifest {
            state: "implemented_as_tauri_local_ffmpeg_controller",
            audio_format: "mp3",
            recorder_boundary: "local_desktop_only_no_backend_secret_access",
            consent_required: true,
            default_output_policy: "app_data_dir/telemost-recordings/{account_id}/{recording_session_id}",
            audio_device_policy: YandexTelemostLocalRecordingPolicy {
                macos: "use explicit loopback input such as BlackHole 2ch; Макошь does not install kernel audio drivers silently",
                linux: "prepare command can create a PulseAudio/PipeWire null sink named makosh_telemost and record makosh_telemost.monitor",
                windows: "use WASAPI loopback or an explicitly configured virtual input",
                ffmpeg_path_env: "MAKOSH_TELEMOST_FFMPEG_PATH",
                ffmpeg_input_env: "MAKOSH_TELEMOST_FFMPEG_INPUT",
            },
        },
        speaker_timeline: YandexTelemostSpeakerTimelinePolicy {
            state: "implemented_as_webview_dom_heuristic_jsonl_and_txt",
            source: "visible_webview_active_speaker_dom_observation",
            reliability: "hint_not_source_of_truth",
            output_files: vec!["speaker-timeline.jsonl", "speaker-timeline.txt"],
            role_in_transcription: "warm-start diarization/person-count hints for Whisper-side processing",
        },
    }
}

pub fn telemost_provider_kind() -> CommunicationProviderKind {
    CommunicationProviderKind::YandexTelemostUser
}

fn default_true() -> bool {
    true
}

fn default_yandex_telemost_retention_cleanup_limit() -> i64 {
    100
}
