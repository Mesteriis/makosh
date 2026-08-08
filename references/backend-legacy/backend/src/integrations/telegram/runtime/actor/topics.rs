use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::tdjson::client::TdJsonClient;
use crate::integrations::telegram::tdjson::{self, snapshots::TelegramTdlibTopicSnapshot};
use makosh_provider_telegram::tdlib::topics;

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id};

pub(super) fn actor_get_forum_topics(
    client: &TdJsonClient,
    provider_chat_id: &str,
    limit: i32,
) -> Result<Vec<TelegramTdlibTopicSnapshot>, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-forum-topics-{provider_chat_id}");
    client.send_json(&topics::get_forum_topics(chat_id, limit, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parsing::topics::parse_tdlib_topic_list(&response)
}

pub(super) fn actor_create_forum_topic(
    client: &TdJsonClient,
    provider_chat_id: &str,
    title: &str,
    command_id: &str,
) -> Result<TelegramTdlibTopicSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-forum-topic-create-{command_id}");
    client.send_json(
        &topics::create_forum_topic(chat_id, title, &extra)
            .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?,
    )?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parsing::topics::parse_tdlib_created_forum_topic(&response)
}

pub(super) fn actor_toggle_forum_topic_closed(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_topic_id: i64,
    is_closed: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-forum-topic-closed-{command_id}");
    client.send_json(&topics::toggle_forum_topic_closed(
        chat_id,
        provider_topic_id,
        is_closed,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}
