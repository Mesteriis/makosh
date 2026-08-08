use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::messages::TelegramManualSendRequest;
use crate::integrations::telegram::runtime::models::TelegramMediaSendRequest;
use crate::integrations::telegram::tdjson::client::TdJsonClient;
use crate::integrations::telegram::tdjson::{self, snapshots::TelegramTdlibMessageSnapshot};
use makosh_provider_telegram::tdlib::messages::{self, send_media, send_reply, send_text};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id, tdlib_provider_message_id};

pub(super) fn actor_send_text(
    client: &TdJsonClient,
    request: &TelegramManualSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(&request.provider_chat_id)?;
    let extra = format!("makosh-runtime-send-{}", request.command_id.trim());
    client.send_json(
        &send_text(chat_id, &request.text, &extra)
            .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?,
    )?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parsing::messages::parse_tdlib_message_snapshot(&response)
}

pub(super) fn actor_send_media(
    client: &TdJsonClient,
    request: &TelegramMediaSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    request.validate()?;
    let chat_id = tdlib_provider_chat_id(&request.provider_chat_id)?;
    let extra = format!("makosh-media-send-{}", request.command_id.trim());
    client.send_json(
        &send_media(
            chat_id,
            request.media_type,
            &request.local_path,
            request.caption.as_deref(),
            request.filename.as_deref(),
            &extra,
        )
        .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?,
    )?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parsing::messages::parse_tdlib_message_snapshot(&response)
}

pub(super) fn actor_send_reply(
    client: &TdJsonClient,
    provider_chat_id: &str,
    reply_to_provider_message_id: &str,
    text: &str,
    command_id: &str,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let reply_to_message_id = tdlib_provider_message_id(reply_to_provider_message_id)?;
    let extra = format!("makosh-reply-{}", command_id.trim());
    client.send_json(
        &send_reply(chat_id, reply_to_message_id, text, &extra)
            .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?,
    )?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parsing::messages::parse_tdlib_message_snapshot(&response)
}

pub(super) fn actor_send_forward(
    client: &TdJsonClient,
    provider_chat_id: &str,
    from_provider_chat_id: &str,
    from_provider_message_id: &str,
    command_id: &str,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let from_chat_id = tdlib_provider_chat_id(from_provider_chat_id)?;
    let message_id = tdlib_provider_message_id(from_provider_message_id)?;
    let extra = format!("makosh-forward-{}", command_id.trim());
    client.send_json(&messages::forward_message(
        chat_id,
        from_chat_id,
        message_id,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parsing::messages::parse_tdlib_message_snapshot(&response)
}
