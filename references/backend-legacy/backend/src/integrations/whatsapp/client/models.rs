use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::evidence::NewRawCommunicationRecord;

use super::errors::WhatsappWebError;
use super::validation::{validate_non_empty, validate_object, validate_string_array};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsappWebAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub provider_shape: Option<String>,
    pub display_name: String,
    pub external_account_id: String,
    pub device_name: String,
    pub local_state_path: String,
}

#[derive(Clone, Deserialize, Eq, PartialEq)]
pub struct WhatsappLiveAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub provider_shape: String,
    pub display_name: String,
    pub external_account_id: String,
    pub device_name: Option<String>,
    pub local_state_path: Option<String>,
}

impl WhatsappLiveAccountSetupRequest {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        let provider_shape = validate_non_empty("provider_shape", &self.provider_shape)?;
        if provider_shape != "whatsapp_web_companion" {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "unsupported WhatsApp provider_shape `{provider_shape}`; only whatsapp_web_companion is available"
            )));
        }
        Ok(())
    }
}

impl fmt::Debug for WhatsappLiveAccountSetupRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WhatsappLiveAccountSetupRequest")
            .field("account_id", &self.account_id)
            .field("provider_kind", &self.provider_kind)
            .field("provider_shape", &self.provider_shape)
            .field("display_name", &self.display_name)
            .field("external_account_id", &self.external_account_id)
            .field("device_name", &self.device_name)
            .field("local_state_path", &self.local_state_path)
            .finish()
    }
}

impl WhatsappWebAccountSetupRequest {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_non_empty("device_name", &self.device_name)?;
        validate_non_empty("local_state_path", &self.local_state_path)?;
        if let Some(provider_shape) = self.provider_shape.as_deref() {
            validate_non_empty("provider_shape", provider_shape)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebAccountSetupResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime: String,
    pub session: WhatsappWebSession,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewWhatsappWebSession {
    pub session_id: String,
    pub account_id: String,
    pub device_name: String,
    pub companion_runtime: WhatsappWebCompanionRuntime,
    pub link_state: WhatsappWebLinkState,
    pub local_state_path: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewWhatsappWebSession {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("session_id", &self.session_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("device_name", &self.device_name)?;
        validate_non_empty("local_state_path", &self.local_state_path)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebSession {
    pub session_id: String,
    pub account_id: String,
    pub device_name: String,
    pub companion_runtime: String,
    pub link_state: String,
    pub local_state_path: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsappWebCompanionRuntime {
    Fixture,
    ManualWebview,
    Blocked,
}

impl WhatsappWebCompanionRuntime {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::ManualWebview => "manual_webview",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsappWebLinkState {
    Fixture,
    QrPending,
    PairCodePending,
    Linked,
    Degraded,
    Revoked,
    Blocked,
}

impl WhatsappWebLinkState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::QrPending => "qr_pending",
            Self::PairCodePending => "pair_code_pending",
            Self::Linked => "linked",
            Self::Degraded => "degraded",
            Self::Revoked => "revoked",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebMessage {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub chat_title: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub text: String,
    pub reply_to_provider_message_id: Option<String>,
    pub forward_origin_chat_id: Option<String>,
    pub forward_origin_message_id: Option<String>,
    pub forward_origin_sender_id: Option<String>,
    pub forward_origin_sender_name: Option<String>,
    pub forwarded_at: Option<DateTime<Utc>>,
    #[serde(default = "default_json_object")]
    pub message_metadata: Value,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
    pub delivery_state: WhatsappWebDeliveryState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebReaction {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub provider_actor_id: String,
    pub sender_display_name: String,
    pub reaction: String,
    pub is_active: bool,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebReaction {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("provider_actor_id", &self.provider_actor_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_non_empty("reaction", &self.reaction)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn provider_record_id(&self) -> String {
        format!(
            "{}:{}:{}",
            self.provider_message_id.trim(),
            self.provider_actor_id.trim(),
            self.reaction.trim()
        )
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_chat_id,
            &self.provider_message_id,
            &self.provider_actor_id,
            &self.reaction,
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebMedia {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub storage_kind: String,
    pub storage_path: String,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebMedia {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("provider_attachment_id", &self.provider_attachment_id)?;
        validate_non_empty("content_type", &self.content_type)?;
        validate_non_empty("sha256", &self.sha256)?;
        validate_non_empty("storage_kind", &self.storage_kind)?;
        validate_non_empty("storage_path", &self.storage_path)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn provider_record_id(&self) -> String {
        format!(
            "{}:{}",
            self.provider_message_id.trim(),
            self.provider_attachment_id.trim()
        )
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_chat_id,
            &self.provider_message_id,
            &self.provider_attachment_id,
            &self.sha256,
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebStatus {
    pub account_id: String,
    pub provider_status_id: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub sender_identity_kind: Option<String>,
    pub sender_address: Option<String>,
    pub sender_push_name: Option<String>,
    #[serde(default = "default_json_object")]
    pub sender_business_profile: Value,
    #[serde(default = "default_json_object")]
    pub sender_profile_photo_ref: Value,
    pub text: String,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
}

impl NewWhatsappWebStatus {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_status_id", &self.provider_status_id)?;
        validate_non_empty("sender_id", &self.sender_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_optional_non_empty("sender_identity_kind", self.sender_identity_kind.as_deref())?;
        validate_optional_non_empty("sender_address", self.sender_address.as_deref())?;
        validate_optional_non_empty("sender_push_name", self.sender_push_name.as_deref())?;
        validate_object("sender_business_profile", &self.sender_business_profile)?;
        validate_object("sender_profile_photo_ref", &self.sender_profile_photo_ref)?;
        validate_non_empty("text", &self.text)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_status_id,
            &self.sender_id,
            self.sender_identity_kind.as_deref().unwrap_or_default(),
            self.sender_address.as_deref().unwrap_or_default(),
            self.sender_push_name.as_deref().unwrap_or_default(),
            &self.sender_business_profile.to_string(),
            &self.sender_profile_photo_ref.to_string(),
            &self.text,
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebStatusView {
    pub account_id: String,
    pub provider_status_id: String,
    pub viewer_id: String,
    pub viewer_display_name: String,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebStatusView {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_status_id", &self.provider_status_id)?;
        validate_non_empty("viewer_id", &self.viewer_id)?;
        validate_non_empty("viewer_display_name", &self.viewer_display_name)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn provider_record_id(&self) -> String {
        format!(
            "{}:{}",
            self.provider_status_id.trim(),
            self.viewer_id.trim()
        )
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_status_id,
            &self.viewer_id,
            &self.viewer_display_name,
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebStatusDelete {
    pub account_id: String,
    pub provider_status_id: String,
    pub actor_class: String,
    pub reason_class: String,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebStatusDelete {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_status_id", &self.provider_status_id)?;
        validate_non_empty("actor_class", &self.actor_class)?;
        validate_non_empty("reason_class", &self.reason_class)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_status_id,
            &self.actor_class,
            &self.reason_class,
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebPresence {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_identity_id: String,
    pub identity_kind: String,
    pub display_name: String,
    pub presence_state: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebPresence {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_identity_id", &self.provider_identity_id)?;
        validate_non_empty("identity_kind", &self.identity_kind)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("presence_state", &self.presence_state)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn provider_record_id(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.provider_chat_id.trim(),
            self.identity_kind.trim(),
            self.provider_identity_id.trim(),
            self.presence_state.trim(),
            self.observed_at.to_rfc3339(),
        )
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_chat_id,
            &self.provider_identity_id,
            &self.identity_kind,
            &self.display_name,
            &self.presence_state,
            &self
                .last_seen_at
                .map(|value| value.to_rfc3339())
                .unwrap_or_default(),
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebCall {
    pub account_id: String,
    pub provider_call_id: String,
    pub provider_chat_id: String,
    pub direction: String,
    pub call_state: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebCall {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_call_id", &self.provider_call_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("direction", &self.direction)?;
        validate_non_empty("call_state", &self.call_state)?;
        validate_object("metadata", &self.metadata)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_call_id,
            &self.provider_chat_id,
            &self.direction,
            &self.call_state,
            &self
                .started_at
                .map(|value| value.to_rfc3339())
                .unwrap_or_default(),
            &self
                .ended_at
                .map(|value| value.to_rfc3339())
                .unwrap_or_default(),
            &self.metadata.to_string(),
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebRuntimeEvent {
    pub account_id: String,
    pub provider_event_id: String,
    pub runtime_event_kind: String,
    pub runtime_status: Option<String>,
    pub lifecycle_state: Option<String>,
    pub severity: Option<String>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebRuntimeEvent {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_event_id", &self.provider_event_id)?;
        validate_non_empty("runtime_event_kind", &self.runtime_event_kind)?;
        validate_optional_non_empty("runtime_status", self.runtime_status.as_deref())?;
        validate_optional_non_empty("lifecycle_state", self.lifecycle_state.as_deref())?;
        validate_optional_non_empty("severity", self.severity.as_deref())?;
        validate_object("metadata", &self.metadata)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    fn defaults_to_degraded(&self) -> bool {
        let kind = self.runtime_event_kind.trim().to_ascii_lowercase();
        kind.contains("unknown") || kind.contains("unsupported")
    }

    pub(crate) fn effective_runtime_status(&self) -> Option<&str> {
        self.runtime_status.as_deref().or_else(|| {
            if self.defaults_to_degraded() {
                Some("degraded")
            } else {
                None
            }
        })
    }

    pub(crate) fn effective_lifecycle_state(&self) -> Option<&str> {
        self.lifecycle_state.as_deref().or_else(|| {
            if self.defaults_to_degraded() {
                Some("degraded")
            } else {
                None
            }
        })
    }

    pub(crate) fn effective_severity(&self) -> Option<&str> {
        self.severity.as_deref().or_else(|| {
            if self.defaults_to_degraded() {
                Some("warning")
            } else {
                None
            }
        })
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_event_id,
            &self.runtime_event_kind,
            self.effective_runtime_status().unwrap_or(""),
            self.effective_lifecycle_state().unwrap_or(""),
            self.effective_severity().unwrap_or(""),
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebDialog {
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_title: String,
    pub chat_kind: String,
    pub is_archived: Option<bool>,
    pub is_pinned: Option<bool>,
    pub is_muted: Option<bool>,
    pub is_unread: Option<bool>,
    pub unread_count: Option<i64>,
    pub participant_count: Option<i64>,
    pub community_parent_chat_id: Option<String>,
    pub community_parent_title: Option<String>,
    pub invite_link: Option<String>,
    pub is_community_root: Option<bool>,
    pub is_broadcast: Option<bool>,
    pub is_newsletter: Option<bool>,
    #[serde(default = "default_json_object")]
    pub avatar_metadata: Value,
    #[serde(default)]
    pub provider_labels: Vec<String>,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebDialog {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("chat_title", &self.chat_title)?;
        validate_non_empty("chat_kind", &self.chat_kind)?;
        validate_optional_non_empty(
            "community_parent_chat_id",
            self.community_parent_chat_id.as_deref(),
        )?;
        validate_optional_non_empty(
            "community_parent_title",
            self.community_parent_title.as_deref(),
        )?;
        validate_optional_non_empty("invite_link", self.invite_link.as_deref())?;
        validate_object("avatar_metadata", &self.avatar_metadata)?;
        validate_string_array("provider_labels", &self.provider_labels)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_chat_id,
            &self.chat_kind,
            &self.chat_title,
            match self.is_archived {
                Some(true) => "archived",
                Some(false) => "active",
                None => "archive_unknown",
            },
            match self.is_pinned {
                Some(true) => "pinned",
                Some(false) => "unpinned",
                None => "pin_unknown",
            },
            match self.is_muted {
                Some(true) => "muted",
                Some(false) => "unmuted",
                None => "mute_unknown",
            },
            match self.is_unread {
                Some(true) => "unread",
                Some(false) => "read",
                None => "read_unknown",
            },
            self.community_parent_chat_id.as_deref().unwrap_or_default(),
            self.community_parent_title.as_deref().unwrap_or_default(),
            self.invite_link.as_deref().unwrap_or_default(),
            match self.is_community_root {
                Some(true) => "community_root",
                Some(false) => "not_community_root",
                None => "community_root_unknown",
            },
            match self.is_broadcast {
                Some(true) => "broadcast",
                Some(false) => "not_broadcast",
                None => "broadcast_unknown",
            },
            match self.is_newsletter {
                Some(true) => "newsletter",
                Some(false) => "not_newsletter",
                None => "newsletter_unknown",
            },
            &self
                .unread_count
                .map(|value| value.to_string())
                .unwrap_or_default(),
            &self
                .participant_count
                .map(|value| value.to_string())
                .unwrap_or_default(),
            &self.avatar_metadata.to_string(),
            &self.provider_labels.join(","),
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebParticipant {
    pub account_id: String,
    pub provider_chat_id: String,
    #[serde(default)]
    pub chat_title: String,
    #[serde(default = "default_chat_kind")]
    pub chat_kind: String,
    #[serde(default)]
    pub provider_member_id: String,
    pub provider_identity_id: String,
    pub identity_kind: String,
    pub display_name: String,
    pub push_name: Option<String>,
    pub address: Option<String>,
    #[serde(default = "default_json_object")]
    pub business_profile: Value,
    #[serde(default = "default_json_object")]
    pub profile_photo_ref: Value,
    pub role: String,
    pub status: String,
    #[serde(default)]
    pub is_self: bool,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub is_owner: bool,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebParticipant {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("chat_title", self.effective_chat_title())?;
        validate_non_empty("chat_kind", self.effective_chat_kind())?;
        validate_non_empty("provider_identity_id", &self.provider_identity_id)?;
        validate_non_empty("provider_member_id", self.effective_provider_member_id())?;
        validate_non_empty("identity_kind", &self.identity_kind)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_optional_non_empty("push_name", self.push_name.as_deref())?;
        validate_object("business_profile", &self.business_profile)?;
        validate_object("profile_photo_ref", &self.profile_photo_ref)?;
        validate_non_empty("role", &self.role)?;
        validate_non_empty("status", &self.status)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn provider_record_id(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.provider_chat_id.trim(),
            self.effective_provider_member_id(),
            self.role.trim(),
            self.status.trim()
        )
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_chat_id,
            self.effective_provider_member_id(),
            &self.provider_identity_id,
            &self.role,
            &self.status,
            self.push_name.as_deref().unwrap_or_default(),
            &self.business_profile.to_string(),
            &self.profile_photo_ref.to_string(),
        ])
    }

    pub(crate) fn effective_provider_member_id(&self) -> &str {
        let provider_member_id = self.provider_member_id.trim();
        if provider_member_id.is_empty() {
            self.provider_identity_id.trim()
        } else {
            provider_member_id
        }
    }

    pub(crate) fn effective_chat_title(&self) -> &str {
        let chat_title = self.chat_title.trim();
        if chat_title.is_empty() {
            self.provider_chat_id.trim()
        } else {
            chat_title
        }
    }

    pub(crate) fn effective_chat_kind(&self) -> &str {
        let chat_kind = self.chat_kind.trim();
        if chat_kind.is_empty() {
            "group"
        } else {
            chat_kind
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebMessageUpdate {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub text: String,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebMessageUpdate {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("text", &self.text)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_chat_id,
            &self.provider_message_id,
            &self.text,
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebMessageDelete {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub reason_class: String,
    pub actor_class: String,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebMessageDelete {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("reason_class", &self.reason_class)?;
        validate_non_empty("actor_class", &self.actor_class)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_chat_id,
            &self.provider_message_id,
            &self.reason_class,
            &self.actor_class,
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebReceipt {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub delivery_state: WhatsappWebDeliveryState,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebReceipt {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_chat_id,
            &self.provider_message_id,
            self.delivery_state.as_str(),
        ])
    }
}

impl NewWhatsappWebMessage {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("chat_title", &self.chat_title)?;
        validate_non_empty("sender_id", &self.sender_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_non_empty("text", &self.text)?;
        validate_optional_non_empty(
            "reply_to_provider_message_id",
            self.reply_to_provider_message_id.as_deref(),
        )?;
        validate_optional_non_empty(
            "forward_origin_chat_id",
            self.forward_origin_chat_id.as_deref(),
        )?;
        validate_optional_non_empty(
            "forward_origin_message_id",
            self.forward_origin_message_id.as_deref(),
        )?;
        validate_optional_non_empty(
            "forward_origin_sender_id",
            self.forward_origin_sender_id.as_deref(),
        )?;
        validate_optional_non_empty(
            "forward_origin_sender_name",
            self.forward_origin_sender_name.as_deref(),
        )?;
        validate_object("message_metadata", &self.message_metadata)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.account_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_chat_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_message_id.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }
}

fn stable_source_fingerprint(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.trim().as_bytes());
        hasher.update(b"\0");
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn default_json_object() -> Value {
    Value::Object(Default::default())
}

fn default_chat_kind() -> String {
    "group".to_owned()
}

fn validate_optional_non_empty(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), WhatsappWebError> {
    if let Some(value) = value {
        validate_non_empty(field, value)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WhatsappWebDeliveryState {
    Received,
    Sent,
    Delivered,
    Read,
    Played,
    SendDryRun,
    SendBlocked,
}

impl WhatsappWebDeliveryState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Sent => "sent",
            Self::Delivered => "delivered",
            Self::Read => "read",
            Self::Played => "played",
            Self::SendDryRun => "send_dry_run",
            Self::SendBlocked => "send_blocked",
        }
    }

    pub fn as_message_delivery_state(self) -> &'static str {
        self.as_str()
    }
}

impl TryFrom<String> for WhatsappWebDeliveryState {
    type Error = WhatsappWebError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "received" => Ok(Self::Received),
            "sent" => Ok(Self::Sent),
            "delivered" => Ok(Self::Delivered),
            "read" => Ok(Self::Read),
            "played" => Ok(Self::Played),
            "send_dry_run" => Ok(Self::SendDryRun),
            "send_blocked" => Ok(Self::SendBlocked),
            _ => Err(WhatsappWebError::InvalidRequest(format!(
                "unsupported WhatsApp Web delivery_state `{value}`"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebMessageIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebReactionIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
    pub reaction_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebMediaIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
    pub attachment_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebStatusIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebStatusViewIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebStatusDeleteIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
    pub tombstone_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebPresenceIngestResult {
    pub raw_record_id: String,
    pub identity_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebCallIngestResult {
    pub raw_record_id: String,
    pub call_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebRuntimeEventIngestResult {
    pub raw_record_id: String,
    pub accepted_event_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebDialogIngestResult {
    pub raw_record_id: String,
    pub channel_id: String,
    pub conversation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebParticipantIngestResult {
    pub raw_record_id: String,
    pub conversation_id: String,
    pub participant_id: String,
    pub identity_id: String,
    pub previous_role: Option<String>,
    pub current_role: String,
    pub previous_status: Option<String>,
    pub current_status: String,
    pub role_changed: bool,
    pub membership_changed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebMessageUpdateIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
    pub version_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebMessageDeleteIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
    pub tombstone_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebReceiptIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
    pub delivery_state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedMessage {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedReaction {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedMedia {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedStatus {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedStatusView {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedStatusDelete {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedPresence {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedCall {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedRuntimeEvent {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedDialog {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedParticipant {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedMessageUpdate {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedMessageDelete {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsappWebObservedReceipt {
    pub raw: NewRawCommunicationRecord,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: Option<String>,
    pub chat_title: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub delivery_state: String,
    pub metadata: Value,
}

#[cfg(test)]
mod tests {
    use super::WhatsappWebDeliveryState;

    #[test]
    fn whatsapp_delivery_state_accepts_observed_receipt_progression() {
        for value in ["received", "sent", "delivered", "read", "played"] {
            let parsed =
                WhatsappWebDeliveryState::try_from(value.to_owned()).expect("delivery_state");
            assert_eq!(parsed.as_str(), value);
            assert_eq!(parsed.as_message_delivery_state(), value);
        }
    }
}
