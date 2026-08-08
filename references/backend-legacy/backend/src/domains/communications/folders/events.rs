use makosh_events_api::NewEventEnvelope;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use super::{
    CommunicationFolder, CommunicationFolderError, FolderMessageActionResponse,
    FolderMessageOperation,
};

pub(super) const EVENT_TYPE_FOLDER_CREATED: &str = "mail.folder.created";
pub(super) const EVENT_TYPE_FOLDER_UPDATED: &str = "mail.folder.updated";
pub(super) const EVENT_TYPE_FOLDER_DELETED: &str = "mail.folder.deleted";

const EVENT_TYPE_MESSAGE_COPIED: &str = "mail.folder_message.copied";
const EVENT_TYPE_MESSAGE_MOVED: &str = "mail.folder_message.moved";

pub(super) fn folder_event(
    event_type: &str,
    folder: &CommunicationFolder,
) -> Result<NewEventEnvelope, CommunicationFolderError> {
    Ok(NewEventEnvelope::builder(
        generate_folder_event_id(event_type, &folder.folder_id),
        event_type,
        Utc::now(),
        json!({ "kind": "mail_folder_api" }),
        json!({
            "kind": "mail_folder",
            "id": folder.folder_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-frontend" }))
    .payload(serde_json::to_value(folder)?)
    .provenance(json!({
        "source_kind": "local_api",
        "source_id": folder.folder_id,
    }))
    .correlation_id(folder.folder_id.clone())
    .build()?)
}

pub(super) fn folder_message_event(
    response: &FolderMessageActionResponse,
) -> Result<NewEventEnvelope, CommunicationFolderError> {
    let event_type = match response.operation {
        FolderMessageOperation::Copy => EVENT_TYPE_MESSAGE_COPIED,
        FolderMessageOperation::Move => EVENT_TYPE_MESSAGE_MOVED,
    };
    let subject_id = format!("{}:{}", response.folder_id, response.message_id);
    Ok(NewEventEnvelope::builder(
        generate_folder_event_id(event_type, &subject_id),
        event_type,
        Utc::now(),
        json!({ "kind": "mail_folder_api" }),
        json!({
            "kind": "mail_folder_message",
            "id": subject_id,
            "folder_id": response.folder_id,
            "message_id": response.message_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-frontend" }))
    .payload(serde_json::to_value(response)?)
    .provenance(json!({
        "source_kind": "local_api",
        "source_id": response.folder_id,
    }))
    .correlation_id(response.message_id.clone())
    .build()?)
}

fn generate_folder_event_id(event_type: &str, subject_id: &str) -> String {
    format!(
        "mail_folder_event:{event_type}:{subject_id}:{:x}",
        system_time_nanos()
    )
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
