use std::sync::Arc;

use serde_json::Value;
use tokio::sync::broadcast;

use makosh_events_api::{EventEnvelope, NewEventEnvelope};

/// Max events in the broadcast ring buffer before oldest are dropped.
const BUS_CAPACITY: usize = 4096;

/// Shared event bus for realtime dispatch.
#[derive(Clone)]
pub struct InMemoryEventBus {
    tx: broadcast::Sender<Arc<NewEventEnvelope>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BUS_CAPACITY);
        Self { tx }
    }

    pub fn broadcast(&self, event: NewEventEnvelope) -> usize {
        self.tx.send(Arc::new(event)).unwrap_or(0)
    }

    pub fn broadcast_stored(&self, event: &EventEnvelope) -> usize {
        self.broadcast(NewEventEnvelope {
            event_id: event.event_id.clone(),
            event_type: event.event_type.clone(),
            schema_version: event.schema_version,
            occurred_at: event.occurred_at,
            source: event.source.clone(),
            actor: event.actor.clone(),
            subject: event.subject.clone(),
            payload: event.payload.clone(),
            provenance: event.provenance.clone(),
            causation_id: event.causation_id.clone(),
            correlation_id: event.correlation_id.clone(),
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<NewEventEnvelope>> {
        self.tx.subscribe()
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Telegram-specific event type constants (ADR-0091)
// ---------------------------------------------------------------------------

pub mod telegram_event_types {
    pub const SYNC_STARTED: &str = "telegram.sync.started";
    pub const SYNC_PROGRESS: &str = "telegram.sync.progress";
    pub const SYNC_COMPLETED: &str = "telegram.sync.completed";
    pub const SYNC_FAILED: &str = "telegram.sync.failed";

    pub const MESSAGE_CREATED: &str = "telegram.message.created";
    pub const MESSAGE_UPDATED: &str = "telegram.message.updated";
    pub const MESSAGE_DELETED: &str = "telegram.message.deleted";
    pub const MESSAGE_TOMBSTONED: &str = "telegram.message.tombstoned";
    pub const MESSAGE_VISIBILITY_RESTORED: &str = "telegram.message.visibility_restored";

    pub const REACTION_CHANGED: &str = "telegram.reaction.changed";

    pub const CHAT_UPDATED: &str = "telegram.chat.updated";
    pub const CHAT_PINNED: &str = "telegram.chat.pinned";
    pub const CHAT_ARCHIVED: &str = "telegram.chat.archived";
    pub const CHAT_MUTED: &str = "telegram.chat.muted";
    pub const FOLDERS_UPDATED: &str = "telegram.folders.updated";

    pub const TYPING_CHANGED: &str = "telegram.typing.changed";

    pub const TOPIC_UPDATED: &str = "telegram.topic.updated";

    pub const PARTICIPANT_UPDATED: &str = "telegram.participant.updated";

    pub const MEDIA_DOWNLOAD_STARTED: &str = "telegram.media.download.started";
    pub const MEDIA_DOWNLOAD_PROGRESS: &str = "telegram.media.download.progress";
    pub const MEDIA_DOWNLOAD_FAILED: &str = "telegram.media.download.failed";
    pub const MEDIA_DOWNLOADED: &str = "telegram.media.downloaded";
    pub const MEDIA_UPLOAD_STARTED: &str = "telegram.media.upload.started";
    pub const MEDIA_UPLOAD_PROGRESS: &str = "telegram.media.upload.progress";
    pub const MEDIA_UPLOAD_FAILED: &str = "telegram.media.upload.failed";
    pub const MEDIA_UPLOAD_COMPLETED: &str = "telegram.media.upload.completed";

    pub const COMMAND_STATUS_CHANGED: &str = "telegram.command.status_changed";
    pub const COMMAND_RECONCILED: &str = "telegram.command.reconciled";
}

pub mod whatsapp_event_types {
    pub const SYNC_STARTED: &str = "whatsapp.sync.started";
    pub const SYNC_PROGRESS: &str = "whatsapp.sync.progress";
    pub const SYNC_COMPLETED: &str = "whatsapp.sync.completed";
    pub const SYNC_FAILED: &str = "whatsapp.sync.failed";
    pub const RUNTIME_STATUS_CHANGED: &str = "whatsapp.runtime.status_changed";
    pub const RUNTIME_EVENT: &str = "whatsapp.runtime.event";
    pub const SESSION_LINK_STATE_CHANGED: &str = "whatsapp.session.link_state_changed";
    pub const DIALOG_UPDATED: &str = "whatsapp.dialog.updated";
    pub const MESSAGE_CREATED: &str = "whatsapp.message.created";
    pub const MESSAGE_UPDATED: &str = "whatsapp.message.updated";
    pub const MESSAGE_DELETED: &str = "whatsapp.message.deleted";
    pub const REACTION_CHANGED: &str = "whatsapp.reaction.changed";
    pub const RECEIPT_CHANGED: &str = "whatsapp.receipt.changed";
    pub const PARTICIPANT_CHANGED: &str = "whatsapp.participant.changed";
    pub const PRESENCE_CHANGED: &str = "whatsapp.presence.changed";
    pub const CALL_UPDATED: &str = "whatsapp.call.updated";
    pub const STATUS_UPDATED: &str = "whatsapp.status.updated";
    pub const STATUS_DELETED: &str = "whatsapp.status.deleted";
    pub const COMMAND_STATUS_CHANGED: &str = "whatsapp.command.status_changed";
    pub const COMMAND_RECONCILED: &str = "whatsapp.command.reconciled";
    pub const MEDIA_DOWNLOAD_REQUESTED: &str = "whatsapp.media.download.requested";
    pub const MEDIA_DOWNLOAD_STARTED: &str = "whatsapp.media.download.started";
    pub const MEDIA_DOWNLOAD_PROGRESS: &str = "whatsapp.media.download.progress";
    pub const MEDIA_DOWNLOAD_COMPLETED: &str = "whatsapp.media.download.completed";
    pub const MEDIA_DOWNLOAD_FAILED: &str = "whatsapp.media.download.failed";
    pub const MEDIA_UPLOAD_REQUESTED: &str = "whatsapp.media.upload.requested";
    pub const MEDIA_UPLOAD_STARTED: &str = "whatsapp.media.upload.started";
    pub const MEDIA_UPLOAD_PROGRESS: &str = "whatsapp.media.upload.progress";
    pub const MEDIA_UPLOAD_COMPLETED: &str = "whatsapp.media.upload.completed";
    pub const MEDIA_UPLOAD_FAILED: &str = "whatsapp.media.upload.failed";
}

pub mod zulip_event_types {
    pub const COMMAND_STATUS_CHANGED: &str = "zulip.command.status_changed";
    pub const COMMAND_RECONCILED: &str = "zulip.command.reconciled";
}

pub mod zoom_event_types {
    pub const AUTHORIZATION_COMPLETED: &str = "zoom.authorization.completed";
    pub const RUNTIME_STATUS_CHANGED: &str = "zoom.runtime.status_changed";
    pub const TOKEN_REFRESHED: &str = "zoom.token.refreshed";
    pub const TOKEN_REFRESH_SKIPPED: &str = "zoom.token.refresh.skipped";
    pub const TOKEN_REFRESH_FAILED: &str = "zoom.token.refresh.failed";
    pub const MEETING_OBSERVED: &str = "zoom.meeting.observed";
    pub const RECORDING_OBSERVED: &str = "zoom.recording.observed";
    pub const TRANSCRIPT_OBSERVED: &str = "zoom.transcript.observed";
    pub const TRANSCRIPT_REMOVED: &str = "zoom.transcript.removed";
    pub const RECORDING_IMPORT_REMOVED: &str = "zoom.recording.import.removed";
    pub const RETENTION_CLEANUP_COMPLETED: &str = "zoom.retention.cleanup.completed";
}

pub mod yandex_telemost_event_types {
    pub const ACCOUNT_CONFIGURED: &str = "integration.yandex_telemost.account.configured";
    pub const AUTHORIZATION_COMPLETED: &str = "integration.yandex_telemost.authorization.completed";
    pub const RUNTIME_STATUS_CHANGED: &str = "integration.yandex_telemost.runtime.status_changed";
    pub const CONFERENCE_CREATED: &str = "integration.yandex_telemost.conference.created";
    pub const CONFERENCE_OBSERVED: &str = "integration.yandex_telemost.conference.observed";
    pub const CONFERENCE_UPDATED: &str = "integration.yandex_telemost.conference.updated";
    pub const COHOSTS_OBSERVED: &str = "integration.yandex_telemost.cohosts.observed";
    pub const WEBVIEW_OPEN_REQUESTED: &str = "integration.yandex_telemost.webview.open_requested";
    pub const SPEAKER_HINT_OBSERVED: &str = "integration.yandex_telemost.speaker_hint.observed";
    pub const LOCAL_CAPTURE_OBSERVED: &str = "integration.yandex_telemost.local_capture.observed";
    pub const LOCAL_RECORDING_REQUESTED: &str =
        "integration.yandex_telemost.local_recording.requested";
    pub const LOCAL_RECORDING_COMPLETED: &str =
        "integration.yandex_telemost.local_recording.completed";
    pub const RETENTION_CLEANUP_COMPLETED: &str =
        "integration.yandex_telemost.retention.cleanup.completed";
}

/// Sanitize an event payload to never include secrets or raw message bodies.
pub fn sanitize_event_payload(mut payload: Value) -> Value {
    if let Some(obj) = payload.as_object_mut() {
        obj.remove("raw_body");
        obj.remove("tdlib_raw");
        obj.remove("access_token");
        obj.remove("api_hash");
        obj.remove("session_key");
        obj.remove("bot_token");
        obj.remove("proxy_password");
        obj.remove("password");
    }
    payload
}
