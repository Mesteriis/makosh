use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::tdjson::client::TdJsonClient;
use crate::integrations::telegram::tdjson::{self, snapshots::TelegramTdlibFileSnapshot};
use makosh_provider_telegram::tdlib::chats::download_file;

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::receive_tdlib_extra;

pub(super) fn actor_download_file(
    client: &TdJsonClient,
    file_id: i64,
    priority: i32,
) -> Result<TelegramTdlibFileSnapshot, TelegramError> {
    let extra = format!("makosh-runtime-download-file-{file_id}");
    client.send_json(&download_file(file_id, priority, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::parsing::events::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parsing::files::parse_tdlib_file_snapshot(&response)
}
