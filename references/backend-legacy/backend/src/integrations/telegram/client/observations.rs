use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::errors::TelegramError;
use super::models::messages::TelegramMessage;
use super::store::TelegramStore;
use makosh_communications_api::provider_messages::ProviderMessageObservationEvent;

pub(in crate::integrations::telegram) struct TelegramAttachmentDownloadObservation<'a> {
    pub(in crate::integrations::telegram) provider_attachment_id: &'a str,
    pub(in crate::integrations::telegram) communication_attachment_id: Option<&'a str>,
    pub(in crate::integrations::telegram) tdlib_file_id: i64,
    pub(in crate::integrations::telegram) download_state: &'a str,
    pub(in crate::integrations::telegram) local_path: Option<&'a str>,
    pub(in crate::integrations::telegram) size_bytes: Option<i64>,
    pub(in crate::integrations::telegram) content_type: &'a str,
    pub(in crate::integrations::telegram) filename: Option<&'a str>,
    pub(in crate::integrations::telegram) observed_at: DateTime<Utc>,
}

impl TelegramStore {
    pub(in crate::integrations::telegram) async fn append_message_metadata_observation(
        &self,
        message: &TelegramMessage,
        metadata: &Value,
    ) -> Result<Option<i64>, TelegramError> {
        self.append_message_observation_event(
            message,
            "metadata_observed",
            Utc::now(),
            &json!({ "message_metadata": metadata }),
        )
        .await
    }

    pub(in crate::integrations::telegram) async fn append_message_content_observation(
        &self,
        message: &TelegramMessage,
        body_text: &str,
        metadata: &Value,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<i64>, TelegramError> {
        self.append_message_observation_event(
            message,
            "content_observed",
            observed_at,
            &json!({
                "body_text": body_text,
                "message_metadata": metadata,
                "observed_at": observed_at,
            }),
        )
        .await
    }

    pub(in crate::integrations::telegram) async fn append_message_pin_observation(
        &self,
        message: &TelegramMessage,
        is_pinned: bool,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<i64>, TelegramError> {
        self.append_message_observation_event(
            message,
            "pinned_state_observed",
            observed_at,
            &json!({
                "is_pinned": is_pinned,
                "observed_at": observed_at,
            }),
        )
        .await
    }

    pub(in crate::integrations::telegram) async fn append_message_delivery_state_observation(
        &self,
        message: &TelegramMessage,
        delivery_state: &str,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<i64>, TelegramError> {
        self.append_message_observation_event(
            message,
            "delivery_state_observed",
            observed_at,
            &json!({
                "delivery_state": delivery_state,
                "observed_at": observed_at,
            }),
        )
        .await
    }

    pub(in crate::integrations::telegram) async fn append_message_provider_identity_observation(
        &self,
        message: &TelegramMessage,
        provider_record_id: &str,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<i64>, TelegramError> {
        self.append_message_observation_event(
            message,
            "provider_identity_observed",
            observed_at,
            &json!({
                "provider_record_id": provider_record_id,
                "observed_at": observed_at,
            }),
        )
        .await
    }

    pub(in crate::integrations::telegram) async fn append_attachment_download_observation(
        &self,
        message: &TelegramMessage,
        observation: TelegramAttachmentDownloadObservation<'_>,
    ) -> Result<Option<i64>, TelegramError> {
        self.append_message_observation_event(
            message,
            "attachment_download_state_observed",
            observation.observed_at,
            &json!({
                "provider_attachment_id": observation.provider_attachment_id,
                "communication_attachment_id": observation.communication_attachment_id,
                "provider_file_id": observation.tdlib_file_id,
                "download_state": observation.download_state,
                "local_path": observation.local_path,
                "size_bytes": observation.size_bytes,
                "content_type": observation.content_type,
                "filename": observation.filename,
                "observed_at": observation.observed_at,
            }),
        )
        .await
    }

    async fn append_message_observation_event(
        &self,
        message: &TelegramMessage,
        event_kind: &str,
        observed_at: DateTime<Utc>,
        payload: &Value,
    ) -> Result<Option<i64>, TelegramError> {
        self.provider_observation_events()
            .append_provider_message_observation(ProviderMessageObservationEvent {
                provider: "telegram",
                account_id: &message.account_id,
                channel_kind: message.channel_kind.as_str(),
                message_id: &message.message_id,
                external_message_id: &message.provider_message_id,
                event_kind,
                observed_at,
                external_event_id: None,
                payload,
                causation_id: None,
                correlation_id: None,
            })
            .await
            .map_err(Into::into)
    }
}
