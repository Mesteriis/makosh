use chrono::{DateTime, Utc};
use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use makosh_provider_zoom::protocol::{
    DEFAULT_ZOOM_API_BASE_URL, DEFAULT_ZOOM_AUTHORIZATION_ENDPOINT, DEFAULT_ZOOM_TOKEN_ENDPOINT,
    ZOOM_DEFAULT_WEBHOOK_EVENT_TYPES, ZOOM_DEFAULT_WEBHOOK_SUBSCRIPTION_NAME,
    ZOOM_EXPLICIT_TOKEN_REFRESH_THRESHOLD_SECONDS, ZOOM_MAX_TOKEN_REFRESH_THRESHOLD_SECONDS,
    ZOOM_PROVIDER_KIND_STR, ZOOM_PROVIDER_SYNC_DEFAULT_MAX_MEETINGS,
    ZOOM_PROVIDER_SYNC_DEFAULT_PAGE_SIZE, ZOOM_PROVIDER_SYNC_MAX_MEETINGS,
    ZOOM_PROVIDER_SYNC_MAX_PAGE_SIZE, ZOOM_RUNTIME_KIND,
    ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS, ZoomAuthShape, normalized_scopes,
    sanitize_zoom_payload, validate_array, validate_non_empty, validate_object,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::platform::calls::models::{CallDirection, CallState, NewProviderCall};

use super::errors::ZoomError;

#[path = "oauth_models.rs"]
pub(crate) mod oauth_models;
#[path = "transcript.rs"]
mod transcript;
use transcript::{
    ZoomParsedTranscriptFile, empty_json_array, empty_json_object, has_optional_ref,
    parse_zoom_transcript_file, trimmed_optional, validate_optional_ref,
    validate_refresh_threshold,
};

pub const MAX_TRANSCRIPT_FILE_TEXT_BYTES: usize = 10 * 1024 * 1024;

pub const ZOOM_PROVIDER_KIND: CommunicationProviderKind = CommunicationProviderKind::ZoomUser;
#[derive(Clone, Debug, Deserialize)]
pub struct ZoomAccountSetupRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub account_email: Option<String>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

impl ZoomAccountSetupRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }

    pub fn account_config(&self) -> Value {
        json!({
            "provider": "zoom",
            "provider_kind": ZOOM_PROVIDER_KIND_STR,
            "runtime_kind": ZOOM_RUNTIME_KIND,
            "auth_shape": ZoomAuthShape::Fixture.as_str(),
            "lifecycle_state": "fixture_ready",
            "account_email": trimmed_optional(&self.account_email),
            "metadata": &self.metadata,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomLiveAccountSetupRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub account_email: Option<String>,
    #[serde(default)]
    pub auth_shape: ZoomAuthShape,
    pub client_id: String,
    pub token_secret_ref: Option<String>,
    pub client_secret_ref: Option<String>,
    pub webhook_secret_ref: Option<String>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

impl ZoomLiveAccountSetupRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_non_empty("client_id", &self.client_id)?;
        if self.auth_shape == ZoomAuthShape::Fixture {
            return Err(ZoomError::InvalidRequest(
                "auth_shape must be oauth_user or server_to_server for live account metadata"
                    .to_owned(),
            ));
        }
        validate_optional_ref("token_secret_ref", &self.token_secret_ref)?;
        validate_optional_ref("client_secret_ref", &self.client_secret_ref)?;
        validate_optional_ref("webhook_secret_ref", &self.webhook_secret_ref)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }

    pub fn provider_kind(&self) -> CommunicationProviderKind {
        match self.auth_shape {
            ZoomAuthShape::ServerToServer => CommunicationProviderKind::ZoomServerToServer,
            ZoomAuthShape::Fixture | ZoomAuthShape::OAuthUser => {
                CommunicationProviderKind::ZoomUser
            }
        }
    }

    pub fn account_config(&self) -> Value {
        let provider_kind = self.provider_kind();
        json!({
            "provider": "zoom",
            "provider_kind": provider_kind.as_str(),
            "runtime_kind": "zoom_live_blocked_runtime",
            "auth_shape": self.auth_shape.as_str(),
            "lifecycle_state": "blocked",
            "account_email": trimmed_optional(&self.account_email),
            "client_id": self.client_id.trim(),
            "credential_refs_bound": {
                "zoom_oauth_token": has_optional_ref(&self.token_secret_ref),
                "zoom_client_secret": has_optional_ref(&self.client_secret_ref),
                "zoom_webhook_secret": has_optional_ref(&self.webhook_secret_ref),
            },
            "runtime_blockers": ["zoom_live_authorization_required"],
            "metadata": &self.metadata,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomAccount {
    pub account_id: String,
    pub provider_kind: String,
    pub display_name: String,
    pub external_account_id: String,
    pub auth_shape: String,
    pub lifecycle_state: String,
    pub runtime_kind: String,
    pub account_email: Option<String>,
    pub config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProviderAccount> for ZoomAccount {
    fn from(account: ProviderAccount) -> Self {
        let auth_shape = account
            .config
            .get("auth_shape")
            .and_then(Value::as_str)
            .unwrap_or("fixture")
            .to_owned();
        let lifecycle_state = account
            .config
            .get("lifecycle_state")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let runtime_kind = account
            .config
            .get("runtime_kind")
            .and_then(Value::as_str)
            .unwrap_or(ZOOM_RUNTIME_KIND)
            .to_owned();
        let account_email = account
            .config
            .get("account_email")
            .and_then(Value::as_str)
            .map(str::to_owned);

        Self {
            account_id: account.account_id,
            provider_kind: account.provider_kind.as_str().to_owned(),
            display_name: account.display_name,
            external_account_id: account.external_account_id,
            auth_shape,
            lifecycle_state,
            runtime_kind,
            account_email,
            config: account.config,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomAccountSetupResponse {
    pub account: ZoomAccount,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomAccountListResponse {
    pub items: Vec<ZoomAccount>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRuntimeStatus {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime_kind: String,
    pub status: String,
    pub healthy: bool,
    pub auth_shape: String,
    pub live_runtime_available: bool,
    pub recording_ingest_available: bool,
    pub transcript_ingest_available: bool,
    pub runtime_blockers: Vec<String>,
    pub last_error: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRuntimeStartRequest {
    pub account_id: String,
    #[serde(default)]
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRuntimeStopRequest {
    pub account_id: String,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRuntimeRemoveRequest {
    pub account_id: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRuntimeRemoveResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub removed: bool,
    pub removed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZoomParticipantSnapshot {
    pub participant_id: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub joined_at: Option<DateTime<Utc>>,
    pub left_at: Option<DateTime<Utc>>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZoomRecordingRef {
    pub recording_id: String,
    pub recording_type: Option<String>,
    pub download_ref: Option<String>,
    pub file_extension: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub recorded_at: Option<DateTime<Utc>>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomMeetingObservationRequest {
    pub observation_id: Option<String>,
    pub account_id: String,
    pub meeting_id: String,
    pub meeting_uuid: Option<String>,
    pub topic: Option<String>,
    pub host_email: Option<String>,
    pub join_url: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    #[serde(default)]
    pub participants: Vec<ZoomParticipantSnapshot>,
    #[serde(default)]
    pub recording_refs: Vec<ZoomRecordingRef>,
    pub transcript_ref: Option<String>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl ZoomMeetingObservationRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("meeting_id", &self.meeting_id)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }

    pub fn provider_chat_id(&self) -> String {
        format!("zoom:meeting:{}", self.meeting_id.trim())
    }

    pub fn event_subject_id(&self) -> String {
        self.meeting_uuid
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(self.meeting_id.trim())
            .to_owned()
    }

    pub fn into_call(&self, call_id: String, observed_at: DateTime<Utc>) -> NewProviderCall {
        let participants = sanitize_zoom_payload(json!(&self.participants));
        let recording_refs = sanitize_zoom_payload(json!(&self.recording_refs));
        let metadata = sanitize_zoom_payload(self.metadata.clone());
        NewProviderCall {
            call_id,
            account_id: self.account_id.trim().to_owned(),
            provider_call_id: self.meeting_id.trim().to_owned(),
            provider_chat_id: self.provider_chat_id(),
            direction: CallDirection::Outgoing,
            call_state: if self.ended_at.is_some() {
                CallState::Ended
            } else {
                CallState::Active
            },
            started_at: self.started_at,
            ended_at: self.ended_at,
            transcription_policy_id: None,
            metadata: json!({
                "provider": "zoom",
                "provider_kind": ZOOM_PROVIDER_KIND_STR,
                "meeting_id": &self.meeting_id,
                "meeting_uuid": &self.meeting_uuid,
                "topic": &self.topic,
                "host_email": &self.host_email,
                "join_url": &self.join_url,
                "duration_seconds": &self.duration_seconds,
                "participants": participants,
                "recording_refs": recording_refs,
                "transcript_ref": &self.transcript_ref,
                "observed_at": observed_at,
                "metadata": metadata,
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ZoomMeetingIngestResult {
    pub call_id: String,
    pub account_id: String,
    pub meeting_id: String,
    pub event_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRecordingObservationRequest {
    pub observation_id: Option<String>,
    pub account_id: String,
    pub meeting_id: String,
    pub recording: ZoomRecordingRef,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl ZoomRecordingObservationRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("meeting_id", &self.meeting_id)?;
        validate_non_empty("recording.recording_id", &self.recording.recording_id)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ZoomRecordingIngestResult {
    pub account_id: String,
    pub meeting_id: String,
    pub recording_id: String,
    pub event_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRecordingMediaDownloadRequest {
    pub observation_id: Option<String>,
    pub account_id: String,
    pub meeting_id: String,
    pub meeting_uuid: Option<String>,
    pub recording: ZoomRecordingRef,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub download_url: String,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl ZoomRecordingMediaDownloadRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("meeting_id", &self.meeting_id)?;
        validate_non_empty("recording.recording_id", &self.recording.recording_id)?;
        validate_non_empty("download_url", &self.download_url)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRecordingMediaImportResult {
    pub attachment_id: String,
    pub blob_id: String,
    pub account_id: String,
    pub meeting_id: String,
    pub recording_id: String,
    pub content_type: String,
    pub scan_status: String,
    pub storage_kind: String,
    pub storage_path: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomTranscriptObservationRequest {
    pub observation_id: Option<String>,
    pub transcript_id: String,
    pub account_id: String,
    pub meeting_id: String,
    pub meeting_uuid: Option<String>,
    pub source_recording_ref: Option<String>,
    pub language_code: Option<String>,
    pub transcript_text: String,
    #[serde(default = "empty_json_array")]
    pub segments: Value,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl ZoomTranscriptObservationRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("transcript_id", &self.transcript_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("meeting_id", &self.meeting_id)?;
        validate_non_empty("transcript_text", &self.transcript_text)?;
        validate_array("segments", &self.segments)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }

    pub fn provider_chat_id(&self) -> String {
        format!("zoom:meeting:{}", self.meeting_id.trim())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ZoomTranscriptIngestResult {
    pub transcript_id: String,
    pub call_id: String,
    pub account_id: String,
    pub meeting_id: String,
    pub event_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomTranscriptFileImportRequest {
    pub observation_id: Option<String>,
    pub transcript_id: String,
    pub account_id: String,
    pub meeting_id: String,
    pub meeting_uuid: Option<String>,
    pub source_recording_ref: Option<String>,
    pub language_code: Option<String>,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub file_text: String,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl ZoomTranscriptFileImportRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("transcript_id", &self.transcript_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("meeting_id", &self.meeting_id)?;
        validate_non_empty("file_text", &self.file_text)?;
        if self.file_text.len() > MAX_TRANSCRIPT_FILE_TEXT_BYTES {
            return Err(ZoomError::InvalidRequest(format!(
                "file_text must be at most {MAX_TRANSCRIPT_FILE_TEXT_BYTES} bytes"
            )));
        }
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }

    pub(crate) fn parse_file(&self) -> Result<ZoomParsedTranscriptFile, ZoomError> {
        parse_zoom_transcript_file(
            &self.file_text,
            self.file_name.as_deref(),
            self.content_type.as_deref(),
        )
    }

    pub(crate) fn to_transcript_observation(
        &self,
        parsed: &ZoomParsedTranscriptFile,
    ) -> ZoomTranscriptObservationRequest {
        ZoomTranscriptObservationRequest {
            observation_id: self.observation_id.clone(),
            transcript_id: self.transcript_id.trim().to_owned(),
            account_id: self.account_id.trim().to_owned(),
            meeting_id: self.meeting_id.trim().to_owned(),
            meeting_uuid: self.meeting_uuid.clone(),
            source_recording_ref: self.source_recording_ref.clone(),
            language_code: self.language_code.clone(),
            transcript_text: parsed.transcript_text.clone(),
            segments: parsed.segments.clone(),
            metadata: json!({
                "transcript_file_import": {
                    "format": &parsed.format,
                    "file_name": trimmed_optional(&self.file_name),
                    "content_type": trimmed_optional(&self.content_type),
                    "parsed_segment_count": parsed.parsed_segment_count,
                },
                "metadata": &self.metadata,
            }),
            causation_id: self.causation_id.clone(),
            correlation_id: self.correlation_id.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ZoomTranscriptFileImportResult {
    pub transcript_id: String,
    pub call_id: String,
    pub account_id: String,
    pub meeting_id: String,
    pub event_id: String,
    pub status: String,
    pub import_format: String,
    pub parsed_segment_count: usize,
}

impl ZoomTranscriptFileImportResult {
    pub(crate) fn from_ingest(
        result: ZoomTranscriptIngestResult,
        import_format: String,
        parsed_segment_count: usize,
    ) -> Self {
        Self {
            transcript_id: result.transcript_id,
            call_id: result.call_id,
            account_id: result.account_id,
            meeting_id: result.meeting_id,
            event_id: result.event_id,
            status: result.status,
            import_format,
            parsed_segment_count,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRecordingSyncRequest {
    pub account_id: String,
    pub user_id: Option<String>,
    pub from: String,
    pub to: String,
    pub page_size: Option<usize>,
    pub max_meetings: Option<usize>,
    pub api_base_url: Option<String>,
}

impl ZoomRecordingSyncRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("from", &self.from)?;
        validate_non_empty("to", &self.to)?;
        if let Some(user_id) = &self.user_id {
            validate_non_empty("user_id", user_id)?;
        }
        let from = chrono::NaiveDate::parse_from_str(self.from.trim(), "%Y-%m-%d")
            .map_err(|_| ZoomError::InvalidRequest("from must use YYYY-MM-DD format".to_owned()))?;
        let to = chrono::NaiveDate::parse_from_str(self.to.trim(), "%Y-%m-%d")
            .map_err(|_| ZoomError::InvalidRequest("to must use YYYY-MM-DD format".to_owned()))?;
        if from > to {
            return Err(ZoomError::InvalidRequest(
                "from must be earlier than or equal to to".to_owned(),
            ));
        }
        if let Some(page_size) = self.page_size
            && !(1..=ZOOM_PROVIDER_SYNC_MAX_PAGE_SIZE).contains(&page_size)
        {
            return Err(ZoomError::InvalidRequest(format!(
                "page_size must be between 1 and {ZOOM_PROVIDER_SYNC_MAX_PAGE_SIZE}"
            )));
        }
        if let Some(max_meetings) = self.max_meetings
            && !(1..=ZOOM_PROVIDER_SYNC_MAX_MEETINGS).contains(&max_meetings)
        {
            return Err(ZoomError::InvalidRequest(format!(
                "max_meetings must be between 1 and {ZOOM_PROVIDER_SYNC_MAX_MEETINGS}"
            )));
        }
        if let Some(api_base_url) = &self.api_base_url {
            validate_non_empty("api_base_url", api_base_url)?;
        }
        Ok(())
    }

    pub(crate) fn page_size(&self) -> usize {
        self.page_size
            .unwrap_or(ZOOM_PROVIDER_SYNC_DEFAULT_PAGE_SIZE)
    }

    pub(crate) fn max_meetings(&self) -> usize {
        self.max_meetings
            .unwrap_or(ZOOM_PROVIDER_SYNC_DEFAULT_MAX_MEETINGS)
    }

    pub(crate) fn api_base_url(&self) -> String {
        self.api_base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_ZOOM_API_BASE_URL)
            .trim_end_matches('/')
            .to_owned()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRecordingSyncFailure {
    pub meeting_id: String,
    pub step: String,
    pub error: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRecordingSyncResult {
    pub account_id: String,
    pub user_id: String,
    pub from: String,
    pub to: String,
    pub meetings_seen: usize,
    pub meetings_recorded: usize,
    pub recordings_recorded: usize,
    pub media_downloads_recorded: usize,
    pub transcripts_recorded: usize,
    pub failures: Vec<ZoomRecordingSyncFailure>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRecordingImportAuditItem {
    pub attachment_id: String,
    pub account_id: String,
    pub meeting_id: Option<String>,
    pub meeting_uuid: Option<String>,
    pub recording_id: Option<String>,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub source: Option<String>,
    pub scan_status: String,
    pub scan_summary: Option<String>,
    pub storage_kind: String,
    pub storage_path: String,
    pub retention_mode: String,
    pub retention_days: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRecordingImportAuditResponse {
    pub account_id: String,
    pub items: Vec<ZoomRecordingImportAuditItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRecordingImportRemoveRequest {
    #[serde(default)]
    pub reason: Option<String>,
}

impl ZoomRecordingImportRemoveRequest {
    pub fn reason(&self) -> Option<String> {
        self.reason
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRecordingImportRemoveResponse {
    pub account_id: String,
    pub attachment_id: String,
    pub blob_id: String,
    pub recording_id: Option<String>,
    pub removed: bool,
    pub blob_metadata_removed: bool,
    pub blob_file_removed: bool,
    pub removed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRetentionCleanupRequest {
    #[serde(default = "default_true")]
    pub remove_recordings: bool,
    #[serde(default = "default_true")]
    pub remove_transcripts: bool,
    #[serde(default = "default_zoom_retention_cleanup_limit")]
    pub limit: i64,
}

impl ZoomRetentionCleanupRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        if !self.remove_recordings && !self.remove_transcripts {
            return Err(ZoomError::InvalidRequest(
                "Zoom retention cleanup must target recordings, transcripts or both".to_owned(),
            ));
        }
        if self.limit <= 0 {
            return Err(ZoomError::InvalidRequest(
                "Zoom retention cleanup limit must be positive".to_owned(),
            ));
        }
        Ok(())
    }

    pub fn limit(&self) -> i64 {
        self.limit.clamp(1, 500)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRetentionCleanupItem {
    pub evidence_kind: String,
    pub entity_id: String,
    pub call_id: Option<String>,
    pub meeting_id: Option<String>,
    pub recording_id: Option<String>,
    pub transcript_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub removed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRetentionCleanupResponse {
    pub account_id: String,
    pub checked_at: DateTime<Utc>,
    pub recordings_removed: usize,
    pub transcripts_removed: usize,
    pub items: Vec<ZoomRetentionCleanupItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomAuditEventItem {
    pub position: i64,
    pub event_id: String,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub subject_kind: Option<String>,
    pub subject_entity_id: Option<String>,
    pub correlation_id: Option<String>,
    pub source: Value,
    pub subject: Value,
    pub payload: Value,
    pub provenance: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomAuditEventResponse {
    pub account_id: String,
    pub items: Vec<ZoomAuditEventItem>,
}

fn default_zoom_retention_cleanup_limit() -> i64 {
    100
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomWebhookSubscriptionStatusRequest {
    pub account_id: String,
    pub api_base_url: Option<String>,
}

impl ZoomWebhookSubscriptionStatusRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        if let Some(api_base_url) = &self.api_base_url {
            validate_non_empty("api_base_url", api_base_url)?;
        }
        Ok(())
    }

    pub(crate) fn api_base_url(&self) -> String {
        self.api_base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_ZOOM_API_BASE_URL)
            .trim_end_matches('/')
            .to_owned()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomWebhookSubscriptionReconcileRequest {
    pub account_id: String,
    pub endpoint_url: String,
    pub subscription_name: Option<String>,
    #[serde(default)]
    pub event_types: Vec<String>,
    pub api_base_url: Option<String>,
}

impl ZoomWebhookSubscriptionReconcileRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("endpoint_url", &self.endpoint_url)?;
        if let Some(subscription_name) = &self.subscription_name {
            validate_non_empty("subscription_name", subscription_name)?;
        }
        if let Some(api_base_url) = &self.api_base_url {
            validate_non_empty("api_base_url", api_base_url)?;
        }
        for event_type in self.resolved_event_types() {
            validate_non_empty("event_types[]", &event_type)?;
        }
        Ok(())
    }

    pub(crate) fn api_base_url(&self) -> String {
        self.api_base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_ZOOM_API_BASE_URL)
            .trim_end_matches('/')
            .to_owned()
    }

    pub(crate) fn resolved_subscription_name(&self) -> String {
        self.subscription_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(ZOOM_DEFAULT_WEBHOOK_SUBSCRIPTION_NAME)
            .to_owned()
    }

    pub(crate) fn resolved_event_types(&self) -> Vec<String> {
        let mut event_types = if self.event_types.is_empty() {
            ZOOM_DEFAULT_WEBHOOK_EVENT_TYPES
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        } else {
            self.event_types
                .iter()
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
        };
        event_types.sort();
        event_types.dedup();
        event_types
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomWebhookSubscriptionRemoveRequest {
    pub account_id: String,
    pub subscription_id: Option<String>,
    pub api_base_url: Option<String>,
}

impl ZoomWebhookSubscriptionRemoveRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        if let Some(subscription_id) = &self.subscription_id {
            validate_non_empty("subscription_id", subscription_id)?;
        }
        if let Some(api_base_url) = &self.api_base_url {
            validate_non_empty("api_base_url", api_base_url)?;
        }
        Ok(())
    }

    pub(crate) fn api_base_url(&self) -> String {
        self.api_base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_ZOOM_API_BASE_URL)
            .trim_end_matches('/')
            .to_owned()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ZoomWebhookSubscription {
    pub subscription_id: String,
    pub subscription_name: String,
    pub endpoint_url: String,
    pub event_types: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomWebhookSubscriptionStatusResult {
    pub account_id: String,
    pub provider_kind: String,
    pub auth_shape: String,
    pub checked_at: DateTime<Utc>,
    pub managed_subscription_id: Option<String>,
    pub subscriptions: Vec<ZoomWebhookSubscription>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomWebhookSubscriptionReconcileResult {
    pub account_id: String,
    pub provider_kind: String,
    pub auth_shape: String,
    pub status: String,
    pub checked_at: DateTime<Utc>,
    pub subscription: ZoomWebhookSubscription,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomWebhookSubscriptionRemoveResult {
    pub account_id: String,
    pub provider_kind: String,
    pub auth_shape: String,
    pub removed: bool,
    pub checked_at: DateTime<Utc>,
    pub subscription_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ZoomAuthorizationResult {
    pub account_id: String,
    pub provider_kind: String,
    pub auth_shape: String,
    pub lifecycle_state: String,
    pub runtime_kind: String,
    pub token_secret_ref: String,
    pub client_secret_ref: Option<String>,
    pub secret_kind: String,
    pub store_kind: String,
    pub authorized_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZoomOAuthTokenBundle {
    pub token_url: String,
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret_ref: Option<String>,
    pub auth_shape: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoom_account_id: Option<String>,
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ZoomOAuthTokenResponse {
    pub(crate) access_token: String,
    pub(crate) refresh_token: Option<String>,
    pub(crate) expires_in: Option<i64>,
    pub(crate) token_type: Option<String>,
    pub(crate) scope: Option<String>,
}
