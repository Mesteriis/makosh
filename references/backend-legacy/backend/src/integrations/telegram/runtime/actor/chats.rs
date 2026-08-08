use serde_json::Value;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::tdjson::client::TdJsonClient;
use crate::integrations::telegram::tdjson::{
    self, snapshots::TelegramTdlibChatFolderSnapshot, snapshots::TelegramTdlibChatSnapshot,
};
use makosh_provider_telegram::tdlib::chats;

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::receive_tdlib_extra;

pub(super) fn actor_load_chats(
    client: &TdJsonClient,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatSnapshot>, TelegramError> {
    let load_extra = "makosh-runtime-load-chats";
    client.send_json(&chats::load_chats(limit, load_extra))?;
    let load_response = receive_tdlib_extra(client, load_extra, TDJSON_COMMAND_TIMEOUT)?;
    if tdjson::parsing::events::tdlib_error_message(&load_response).is_some()
        && !is_tdlib_not_found(&load_response)
    {
        return Err(TelegramError::TdlibRuntime(
            tdjson::parsing::events::tdlib_error_message(&load_response)
                .unwrap_or_else(|| "TDLib loadChats failed".to_owned()),
        ));
    }

    let chats_extra = "makosh-runtime-get-chats";
    client.send_json(&chats::get_chats(limit, chats_extra))?;
    let chats_response = receive_tdlib_extra(client, chats_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&chats_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    let chat_ids = tdjson::parsing::chats::parse_tdlib_chat_ids(&chats_response)?;
    let mut snapshots = Vec::with_capacity(chat_ids.len());
    for chat_id in chat_ids {
        let extra = format!("makosh-runtime-get-chat-{chat_id}");
        client.send_json(&chats::get_chat(chat_id, &extra))?;
        let chat_response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::parsing::events::tdlib_error_message(&chat_response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        snapshots.push(tdjson::parsing::chats::parse_tdlib_chat_snapshot(
            &chat_response,
        )?);
    }
    Ok(snapshots)
}

pub(super) fn actor_get_chat_folders(
    client: &TdJsonClient,
    folder_ids: &[i64],
) -> Result<Vec<TelegramTdlibChatFolderSnapshot>, TelegramError> {
    let mut snapshots = Vec::with_capacity(folder_ids.len());
    for folder_id in folder_ids {
        let extra = format!("makosh-runtime-get-chat-folder-{folder_id}");
        client.send_json(&chats::get_chat_folder(*folder_id, &extra))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        if let Some(snapshot) =
            tdjson::parsing::events::parse_tdlib_chat_folder_snapshot(&response)?
        {
            snapshots.push(snapshot);
        }
    }
    Ok(snapshots)
}

fn is_tdlib_not_found(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(404)
}
