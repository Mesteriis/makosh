use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::tdjson::client::TdJsonClient;
use crate::integrations::telegram::tdjson::{self, snapshots::TelegramTdlibMessageSnapshot};
use makosh_provider_telegram::tdlib::chats::get_chat_history;

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::super::models::TelegramHistorySyncMode;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id};
use super::support::oldest_tdlib_message_id;

pub(super) fn actor_sync_history(
    client: &TdJsonClient,
    provider_chat_id: &str,
    from_message_id: Option<i64>,
    limit: i32,
    mode: TelegramHistorySyncMode,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let page_limit = limit.clamp(1, 100);
    let mut cursor = from_message_id;
    let mut snapshots = Vec::new();
    let mut page_index = 0;

    loop {
        let extra = format!(
            "makosh-runtime-history-{chat_id}-{}-{page_index}",
            cursor.unwrap_or(0)
        );
        client.send_json(&get_chat_history(
            chat_id, cursor, page_limit, false, &extra,
        ))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        let page = tdjson::parsing::messages::parse_tdlib_message_list(&response)?;
        if page.is_empty() {
            break;
        }

        let page_len = page.len();
        let next_cursor = oldest_tdlib_message_id(&page);
        snapshots.extend(page);
        if mode != TelegramHistorySyncMode::Full || page_len < page_limit as usize {
            break;
        }
        if next_cursor.is_none() || next_cursor == cursor {
            break;
        }
        cursor = next_cursor;
        page_index += 1;
    }

    Ok(snapshots)
}
