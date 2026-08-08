use serde_json::{Value, json};

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::qr_login::TelegramQrLoginStatus;

use super::super::parsing::events::{authorization_state, tdlib_error_message};
use super::super::qr_login_support::authorization::{password_hint, state_allows_qr_request};
use super::super::qr_login_support::constants::QR_POLL_AFTER_MS;
use super::super::qr_login_support::identity::fetch_authorized_user_identity;
use super::super::qr_login_support::pending::{
    mark_pending_ready_status, mark_pending_status, upsert_pending_response,
};
use super::super::qr_login_support::responses::qr_waiting_response;
use super::tdlib_commands::{close_tdlib_session, send_tdlib_parameters};
use super::worker::handle_tdlib_setup_event;
use super::worker_state::{QrLoginEventOutcome, QrLoginRuntimeState, QrLoginWorkerContext};

pub(super) fn handle_qr_login_event(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    event: Value,
) -> Result<QrLoginEventOutcome, TelegramError> {
    if handle_tdlib_setup_event(context, state, &event)? {
        return Ok(QrLoginEventOutcome::Continue);
    }
    if handle_tdlib_error(context, state, &event)? {
        return Ok(QrLoginEventOutcome::Continue);
    }

    let Some(authorization_state) = authorization_state(&event) else {
        return Ok(QrLoginEventOutcome::Continue);
    };
    let Some(state_type) = authorization_state.get("@type").and_then(Value::as_str) else {
        return Ok(QrLoginEventOutcome::Continue);
    };

    handle_authorization_state(context, state, authorization_state, state_type)
}

fn handle_tdlib_error(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    event: &Value,
) -> Result<bool, TelegramError> {
    let Some(message) = tdlib_error_message(event) else {
        return Ok(false);
    };
    if state.password_check_in_flight {
        state.password_check_in_flight = false;
        mark_pending_status(
            context.pending_logins,
            context.setup_id,
            TelegramQrLoginStatus::WaitingPassword,
            "Telegram password was rejected. Try again.",
            QR_POLL_AFTER_MS,
        )?;
        return Ok(true);
    }
    Err(TelegramError::TdlibRuntime(message))
}

fn handle_authorization_state(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    authorization_state: &Value,
    state_type: &str,
) -> Result<QrLoginEventOutcome, TelegramError> {
    match state_type {
        "authorizationStateWaitTdlibParameters" => {
            send_tdlib_parameters(context.client, context.request, context.database_directory)?;
            state.tdlib_parameters_sent = true;
        }
        "authorizationStateWaitEncryptionKey" => {
            context.client.send_json(
                &super::super::requests::check_database_encryption_key_request(context.request),
            )?;
            state.database_encryption_key_checked = true;
        }
        state_name if state_allows_qr_request(state_name) && !state.qr_requested => {
            context.client.send_json(&json!({
                "@type": "requestQrCodeAuthentication",
                "other_user_ids": [],
                "@extra": "makosh-request-qr-code-authentication"
            }))?;
            state.qr_requested = true;
        }
        "authorizationStateWaitOtherDeviceConfirmation" => {
            handle_wait_other_device_confirmation(context, state, authorization_state)?;
        }
        "authorizationStateWaitPassword" => {
            handle_wait_password(context, state, authorization_state)?
        }
        "authorizationStateReady" => return handle_ready(context, state),
        "authorizationStateClosed" => {
            mark_pending_status(
                context.pending_logins,
                context.setup_id,
                TelegramQrLoginStatus::Failed,
                "Telegram TDLib authorization session closed before QR login completed.",
                0,
            )?;
            return Ok(QrLoginEventOutcome::Complete);
        }
        "authorizationStateClosing" | "authorizationStateLoggingOut" => {
            mark_pending_status(
                context.pending_logins,
                context.setup_id,
                TelegramQrLoginStatus::Failed,
                "Telegram TDLib authorization session is closing.",
                0,
            )?;
            return Ok(QrLoginEventOutcome::Complete);
        }
        unsupported if state.qr_requested => {
            mark_pending_status(
                context.pending_logins,
                context.setup_id,
                TelegramQrLoginStatus::Failed,
                &format!(
                    "Telegram QR login requires unsupported authorization state `{unsupported}`."
                ),
                0,
            )?;
            return Ok(QrLoginEventOutcome::Complete);
        }
        _ => {}
    }
    Ok(QrLoginEventOutcome::Continue)
}

fn handle_wait_other_device_confirmation(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    authorization_state: &Value,
) -> Result<(), TelegramError> {
    state.qr_link_issued = true;
    let link = authorization_state
        .get("link")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "TDLib QR authorization state did not include a link".to_owned(),
            )
        })?;
    let response = qr_waiting_response(context.setup_id, &context.request.account_id, link)?;
    upsert_pending_response(
        context.pending_logins,
        context.request.clone(),
        response,
        context.command_tx.clone(),
        std::sync::Arc::clone(context.worker_completion),
    )
}

fn handle_wait_password(
    context: &QrLoginWorkerContext<'_>,
    state: &mut QrLoginRuntimeState,
    authorization_state: &Value,
) -> Result<(), TelegramError> {
    state.password_check_in_flight = false;
    let password_hint = password_hint(authorization_state);
    let message = password_hint
        .as_deref()
        .map(|hint| format!("Telegram requires your 2-step verification password. Hint: {hint}"))
        .unwrap_or_else(|| "Telegram requires your 2-step verification password.".to_owned());
    mark_pending_status(
        context.pending_logins,
        context.setup_id,
        TelegramQrLoginStatus::WaitingPassword,
        &message,
        QR_POLL_AFTER_MS,
    )
}

fn handle_ready(
    context: &QrLoginWorkerContext<'_>,
    state: &QrLoginRuntimeState,
) -> Result<QrLoginEventOutcome, TelegramError> {
    let identity = match fetch_authorized_user_identity(context.client) {
        Ok(identity) => identity,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "Telegram QR login completed, but TDLib user identity lookup failed"
            );
            None
        }
    };
    let message = if state.qr_requested {
        "Telegram QR login confirmed on the other device."
    } else {
        "Telegram TDLib session is already authorized."
    };
    mark_pending_ready_status(
        context.pending_logins,
        context.setup_id,
        message,
        identity.as_ref(),
    )?;
    close_tdlib_session(context.client);
    Ok(QrLoginEventOutcome::Complete)
}
