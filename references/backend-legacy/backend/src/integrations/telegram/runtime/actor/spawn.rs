use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::tdjson;
use crate::platform::config::app_config::AppConfig;
use makosh_communications_api::accounts::ProviderAccount;
use tokio::sync::mpsc::UnboundedSender;

use super::super::state::{TelegramRuntimeCommand, TelegramRuntimeEvent};
use super::driver::drive_tdlib_actor;
use super::start_request::tdlib_start_request_from_account;
use super::support::short_thread_suffix;

pub(in crate::integrations::telegram::runtime) fn spawn_tdlib_actor(
    config: AppConfig,
    account: ProviderAccount,
    session_encryption_key: Option<String>,
    runtime_event_tx: Option<UnboundedSender<TelegramRuntimeEvent>>,
) -> Result<Sender<TelegramRuntimeCommand>, TelegramError> {
    if !tdjson::client::runtime_available(config.tdjson_path()) {
        return Err(TelegramError::TdlibRuntimeUnavailable(
            "libtdjson is not available for Telegram live runtime".to_owned(),
        ));
    }
    let start_request =
        tdlib_start_request_from_account(&config, &account, session_encryption_key)?;
    let (command_tx, command_rx) = mpsc::channel();
    let thread_name = format!(
        "telegram-tdlib-{}",
        short_thread_suffix(&account.account_id)
    );
    thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            if let Err(error) =
                drive_tdlib_actor(config, start_request, command_rx, runtime_event_tx)
            {
                tracing::warn!(error = %error, "Telegram TDLib actor stopped");
            }
        })
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("failed to spawn Telegram TDLib actor: {error}"))
        })?;

    Ok(command_tx)
}
