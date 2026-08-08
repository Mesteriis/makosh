use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use serde_json::json;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::qr_login::{
    TelegramQrLoginStartRequest, TelegramQrLoginStatus,
};
use crate::platform::config::app_config::AppConfig;

use super::super::client::TdJsonLibrary;
use super::super::qr_login_support::constants::{
    QR_FIRST_LINK_TIMEOUT, QR_POLL_AFTER_MS, QR_SESSION_LIFETIME,
};
use super::super::qr_login_support::pending::mark_pending_status;
use super::super::qr_login_support::types::{
    DrainedQrLoginCommand, PendingQrLoginMap, QrLoginWorkerCompletion, TelegramQrLoginCommand,
};
use super::super::requests::{check_database_encryption_key_request, tdlib_database_directory};
use super::authorization::handle_qr_login_event;
use super::tdlib_commands::{close_tdlib_session, drain_qr_login_commands, send_tdlib_parameters};
use super::worker_state::{QrLoginEventOutcome, QrLoginRuntimeState, QrLoginWorkerContext};

pub(super) fn drive_qr_login(
    config: AppConfig,
    pending_logins: PendingQrLoginMap,
    request: TelegramQrLoginStartRequest,
    setup_id: String,
    command_tx: Sender<TelegramQrLoginCommand>,
    command_rx: Receiver<TelegramQrLoginCommand>,
    worker_completion: QrLoginWorkerCompletion,
) -> Result<(), TelegramError> {
    let library = TdJsonLibrary::load(config.tdjson_path())?;
    let client = library.create_client()?;
    let database_directory = tdlib_database_directory(&request);
    let files_directory = database_directory.join("files");
    std::fs::create_dir_all(&files_directory).map_err(|error| {
        TelegramError::TdlibRuntime(format!(
            "failed to create TDLib data directory `{}`: {error}",
            files_directory.display()
        ))
    })?;

    let _ = client.execute_json(&json!({
        "@type": "setLogVerbosityLevel",
        "new_verbosity_level": 1
    }));
    client.send_json(&json!({
        "@type": "getAuthorizationState",
        "@extra": "makosh-initial-authorization-state"
    }))?;

    let started_at = Instant::now();
    let mut state = QrLoginRuntimeState::default();

    loop {
        if drain_worker_commands(&client, &command_rx, &pending_logins, &setup_id, &mut state)? {
            return Ok(());
        }
        if expire_stale_session(&client, &pending_logins, &setup_id, &state, started_at)? {
            return Ok(());
        }

        let Some(event) = client.receive_json(1.0)? else {
            continue;
        };
        let context = QrLoginWorkerContext {
            client: &client,
            pending_logins: &pending_logins,
            setup_id: &setup_id,
            request: &request,
            command_tx: &command_tx,
            worker_completion: &worker_completion,
            database_directory: &database_directory,
        };
        if handle_qr_login_event(&context, &mut state, event)? == QrLoginEventOutcome::Complete {
            return Ok(());
        }
    }
}

fn drain_worker_commands(
    client: &super::super::client::TdJsonClient,
    command_rx: &Receiver<TelegramQrLoginCommand>,
    pending_logins: &PendingQrLoginMap,
    setup_id: &str,
    state: &mut QrLoginRuntimeState,
) -> Result<bool, TelegramError> {
    match drain_qr_login_commands(client, command_rx)? {
        DrainedQrLoginCommand::Cancelled => Ok(true),
        DrainedQrLoginCommand::PasswordSubmitted => {
            state.password_check_in_flight = true;
            mark_pending_status(
                pending_logins,
                setup_id,
                TelegramQrLoginStatus::WaitingPassword,
                "Checking Telegram password.",
                QR_POLL_AFTER_MS,
            )?;
            Ok(false)
        }
        DrainedQrLoginCommand::None => Ok(false),
    }
}

fn expire_stale_session(
    client: &super::super::client::TdJsonClient,
    pending_logins: &PendingQrLoginMap,
    setup_id: &str,
    state: &QrLoginRuntimeState,
    started_at: Instant,
) -> Result<bool, TelegramError> {
    if !state.qr_link_issued && started_at.elapsed() > QR_FIRST_LINK_TIMEOUT {
        mark_pending_status(
            pending_logins,
            setup_id,
            TelegramQrLoginStatus::Failed,
            "Telegram TDLib did not return a QR confirmation link in time.",
            0,
        )?;
        close_tdlib_session(client);
        return Ok(true);
    }
    if started_at.elapsed() > QR_SESSION_LIFETIME {
        mark_pending_status(
            pending_logins,
            setup_id,
            TelegramQrLoginStatus::Expired,
            "Telegram QR login session expired; start a new QR login.",
            0,
        )?;
        close_tdlib_session(client);
        return Ok(true);
    }
    Ok(false)
}

pub(super) fn handle_tdlib_setup_event(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    event: &serde_json::Value,
) -> Result<bool, TelegramError> {
    if super::super::parsing::events::is_tdlib_parameters_not_specified_error(event) {
        if !state.tdlib_parameters_sent {
            send_tdlib_parameters(context.client, context.request, context.database_directory)?;
            state.tdlib_parameters_sent = true;
        }
        return Ok(true);
    }
    if super::super::parsing::events::is_tdlib_database_encryption_key_needed_error(event) {
        if !state.database_encryption_key_checked {
            context
                .client
                .send_json(&check_database_encryption_key_request(context.request))?;
            state.database_encryption_key_checked = true;
        }
        return Ok(true);
    }
    Ok(false)
}
