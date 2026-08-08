use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::tdjson::client::TdJsonClient;
use crate::integrations::telegram::tdjson::{self};
use makosh_provider_telegram::tdlib::{chats, messages};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id, tdlib_provider_message_id};

pub(super) fn actor_edit_message(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    new_text: &str,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = tdlib_provider_message_id(provider_message_id)?;
    let extra = format!("makosh-edit-{}", command_id.trim());
    client.send_json(
        &messages::edit_text(chat_id, message_id, new_text, &extra)
            .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?,
    )?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_delete_message(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    revoke: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = tdlib_provider_message_id(provider_message_id)?;
    let extra = format!("makosh-delete-{}", command_id.trim());
    client.send_json(&messages::delete_messages(
        chat_id,
        &[message_id],
        revoke,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_set_reaction(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    reaction_emoji: &str,
    is_active: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = tdlib_provider_message_id(provider_message_id)?;
    let extra = format!("makosh-reaction-{}", command_id.trim());
    let request = if is_active {
        messages::add_reaction(chat_id, message_id, reaction_emoji, &extra)
    } else {
        messages::remove_reaction(chat_id, message_id, reaction_emoji, &extra)
    };
    client.send_json(&request)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_pin_message(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    pin: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = tdlib_provider_message_id(provider_message_id)?;
    let extra = format!("makosh-pin-{}", command_id.trim());
    let request = if pin {
        messages::pin_message(chat_id, message_id, false, &extra)
    } else {
        messages::unpin_message(chat_id, message_id, &extra)
    };
    client.send_json(&request)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_toggle_chat_unread(
    client: &TdJsonClient,
    provider_chat_id: &str,
    is_marked_as_unread: bool,
    read_through_provider_message_id: Option<&str>,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    if is_marked_as_unread {
        let extra = format!("makosh-chat-unread-{}", command_id.trim());
        client.send_json(&chats::toggle_marked_as_unread(chat_id, true, &extra))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        return Ok(());
    }

    if let Some(provider_message_id) = read_through_provider_message_id {
        let message_id = tdlib_provider_message_id(provider_message_id)?;
        let extra = format!("makosh-chat-read-{}", command_id.trim());
        client.send_json(&messages::view_messages(
            chat_id,
            &[message_id],
            true,
            &extra,
        ))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        return Ok(());
    }

    let extra = format!("makosh-chat-read-toggle-{}", command_id.trim());
    client.send_json(&chats::toggle_marked_as_unread(chat_id, false, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_toggle_chat_archive(
    client: &TdJsonClient,
    provider_chat_id: &str,
    archived: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-chat-archive-{}", command_id.trim());
    client.send_json(&chats::add_chat_to_list(chat_id, archived, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_toggle_chat_mute(
    client: &TdJsonClient,
    provider_chat_id: &str,
    muted: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-chat-mute-{}", command_id.trim());
    client.send_json(&chats::set_chat_mute(chat_id, muted, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_add_chat_to_folder(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_folder_id: i64,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-chat-folder-{}", command_id.trim());
    client.send_json(&chats::add_chat_to_folder(
        chat_id,
        provider_folder_id,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_remove_chat_from_folder(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_folder_id: i64,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let get_extra = format!("makosh-chat-folder-remove-get-{}", command_id.trim());
    client.send_json(&chats::get_chat_folder(provider_folder_id, &get_extra))?;
    let folder_response = receive_tdlib_extra(client, &get_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&folder_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }

    let edit_extra = format!("makosh-chat-folder-remove-{}", command_id.trim());
    client.send_json(
        &tdjson::folder_requests::tdlib_edit_chat_folder_remove_chat_request(
            provider_folder_id,
            chat_id,
            &folder_response,
            &edit_extra,
        )?,
    )?;
    let response = receive_tdlib_extra(client, &edit_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_join_chat(
    client: &TdJsonClient,
    provider_chat_id: &str,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-chat-join-{}", command_id.trim());
    client.send_json(&chats::join_chat(chat_id, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_leave_chat(
    client: &TdJsonClient,
    provider_chat_id: &str,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-chat-leave-{}", command_id.trim());
    client.send_json(&chats::leave_chat(chat_id, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}
