use std::path::Path;
use std::sync::mpsc::{Receiver, TryRecvError};

use serde_json::json;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::qr_login::TelegramQrLoginStartRequest;

use super::super::client::TdJsonClient;
use super::super::qr_login_support::types::{DrainedQrLoginCommand, TelegramQrLoginCommand};
use super::super::requests::set_tdlib_parameters_request;

pub(super) fn drain_qr_login_commands(
    client: &TdJsonClient,
    command_rx: &Receiver<TelegramQrLoginCommand>,
) -> Result<DrainedQrLoginCommand, TelegramError> {
    let mut password_submitted = false;
    loop {
        match command_rx.try_recv() {
            Ok(TelegramQrLoginCommand::CheckPassword(password)) => {
                client.send_json(&json!({
                    "@type": "checkAuthenticationPassword",
                    "password": password,
                    "@extra": "makosh-check-authentication-password"
                }))?;
                password_submitted = true;
            }
            Ok(TelegramQrLoginCommand::Cancel) => {
                client.send_json(&json!({ "@type": "close" }))?;
                return Ok(DrainedQrLoginCommand::Cancelled);
            }
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => {
                return Ok(if password_submitted {
                    DrainedQrLoginCommand::PasswordSubmitted
                } else {
                    DrainedQrLoginCommand::None
                });
            }
        }
    }
}

pub(super) fn send_tdlib_parameters(
    client: &TdJsonClient,
    request: &TelegramQrLoginStartRequest,
    database_directory: &Path,
) -> Result<(), TelegramError> {
    client.send_json(&set_tdlib_parameters_request(request, database_directory)?)
}

pub(super) fn close_tdlib_session(client: &TdJsonClient) {
    let _ = client.send_json(&json!({ "@type": "close" }));
}
