use serde_json::Value;
use thiserror::Error;

use crate::platform::events::bus::InMemoryEventBus;
use makosh_events_postgres::store::EventStore;

#[derive(Clone, Debug)]
pub(crate) struct RealtimeConversationTranscriptBridgeRequest {
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) bundle_id: String,
    pub(crate) bundle_root: String,
    pub(crate) transcript_text: String,
    pub(crate) segments: Value,
    pub(crate) language_code: Option<String>,
    pub(crate) stt_provider: String,
    pub(crate) summary: Option<String>,
    pub(crate) confidence: Option<f64>,
    pub(crate) metadata: Value,
}

#[derive(Debug, Error)]
pub enum RealtimeConversationTranscriptBridgeError {
    #[error(transparent)]
    Provider(#[from] crate::integrations::yandex_telemost::client::errors::YandexTelemostError),
}

pub(crate) async fn complete_realtime_conversation_transcript_bridge(
    event_store: &EventStore,
    event_bus: Option<&InMemoryEventBus>,
    request: &RealtimeConversationTranscriptBridgeRequest,
) -> Result<(), RealtimeConversationTranscriptBridgeError> {
    crate::integrations::yandex_telemost::runtime_bridge::complete_yandex_telemost_transcript_bridge(
        event_store,
        event_bus,
        &crate::integrations::yandex_telemost::client::models::YandexTelemostTranscriptBridgeRequest {
            account_id: request.account_id.clone(),
            conference_id: request.conference_id.clone(),
            bundle_id: request.bundle_id.clone(),
            bundle_root: request.bundle_root.clone(),
            transcript_text: request.transcript_text.clone(),
            segments: request.segments.clone(),
            language_code: request.language_code.clone(),
            stt_provider: request.stt_provider.clone(),
            summary: request.summary.clone(),
            confidence: request.confidence,
            metadata: request.metadata.clone(),
        },
    )
    .await?;
    Ok(())
}
