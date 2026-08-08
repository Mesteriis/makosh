use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::tdjson::client::TdJsonClient;
use crate::integrations::telegram::tdjson::{self, snapshots::TelegramTdlibMessageSnapshot};
use makosh_provider_telegram::tdlib::chats::{search_chat_messages, search_messages};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id};

pub(super) fn actor_search_messages(
    client: &TdJsonClient,
    query: &str,
    limit: i32,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    let extra = format!("makosh-search-{}", uuid_extra(query));
    client.send_json(&search_messages(query, limit, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parsing::messages::parse_tdlib_message_list(&response)
}

pub(super) fn actor_search_chat_messages(
    client: &TdJsonClient,
    provider_chat_id: &str,
    query: &str,
    limit: i32,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("makosh-search-chat-{chat_id}-{}", uuid_extra(query));
    client.send_json(&search_chat_messages(chat_id, query, limit, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parsing::messages::parse_tdlib_message_list(&response)
}

fn uuid_extra(query: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(query.as_bytes());
    let hash = hasher.finalize();
    format!(
        "{:016x}",
        u64::from_be_bytes(hash[..8].try_into().unwrap_or([0u8; 8]))
    )
}
