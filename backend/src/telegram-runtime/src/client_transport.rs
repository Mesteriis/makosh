//! Framed local transport for the Telegram-owned module client port.

use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;

use makosh_telegram_automation_persistence::TelegramAutomationPersistence;
use makosh_telegram_calls_persistence::TelegramCallsPersistence;
use makosh_telegram_persistence::TelegramDurablePersistence;
use makosh_telegram_tdlib::TdlibTransport;

use crate::{TelegramRuntime, TelegramRuntimeComposition, client_port::TelegramClientPortError};

const MAX_CLIENT_FRAME_BYTES: usize = 512 * 1024;

pub fn serve_authorization_connection(
    mut stream: UnixStream,
    composition: &mut TelegramRuntimeComposition,
    status: Option<&makosh_telegram_api::TelegramAuthorizationStatus>,
) -> Result<(), TelegramClientTransportError> {
    loop {
        let request = match read_frame(&mut stream) {
            Ok(request) => request,
            Err(TelegramClientTransportError::Io(error)) if error == "eof" => return Ok(()),
            Err(error) => return Err(error),
        };
        let (request_id, contract, payload) =
            crate::client_port::decode_module_request_payload(&request)
                .map_err(TelegramClientTransportError::Port)?;
        if contract != makosh_telegram_api::client_contract::TelegramClientContractV1::Authorization
        {
            return Err(TelegramClientTransportError::Port(
                TelegramClientPortError::Protocol(
                    "Telegram authorization transport received another contract".to_owned(),
                ),
            ));
        }
        let request = makosh_telegram_api::client_wire::decode_request(&payload).map_err(|_| {
            TelegramClientTransportError::Port(TelegramClientPortError::Protocol(
                "Telegram authorization payload is invalid".to_owned(),
            ))
        })?;
        let response = match request {
            makosh_telegram_api::client_wire::TelegramAuthorizationRequest::Status => {
                makosh_telegram_api::client_wire::TelegramAuthorizationResponse::Status(
                    status
                        .cloned()
                        .unwrap_or(makosh_telegram_api::TelegramAuthorizationStatus {
                            state: "starting".to_owned(),
                            qr_link: None,
                            password_hint: None,
                        }),
                )
            }
            makosh_telegram_api::client_wire::TelegramAuthorizationRequest::SubmitPassword(
                password,
            ) => {
                composition.submit_password(&password).map_err(|error| {
                    TelegramClientTransportError::Port(TelegramClientPortError::Provider(error))
                })?;
                makosh_telegram_api::client_wire::TelegramAuthorizationResponse::PasswordAccepted
            }
        };
        let response_payload = makosh_telegram_api::client_wire::encode_response(&response);
        let encoded =
            crate::client_port::encode_module_response_payload(request_id, response_payload)
                .map_err(TelegramClientTransportError::Port)?;
        write_frame(&mut stream, &encoded)?;
    }
}

#[derive(Debug)]
pub enum TelegramClientTransportError {
    Port(TelegramClientPortError),
    Io(String),
    Frame(String),
    RuntimeUnavailable,
    Reconfiguration,
}

#[must_use]
pub fn module_error_code(error: &TelegramClientTransportError) -> &'static str {
    match error {
        TelegramClientTransportError::Port(TelegramClientPortError::Reconfiguration(code)) => code,
        TelegramClientTransportError::Port(
            TelegramClientPortError::Protocol(_) | TelegramClientPortError::Codec(_),
        )
        | TelegramClientTransportError::Frame(_) => "INVALID_ARGUMENT",
        TelegramClientTransportError::Port(
            TelegramClientPortError::Provider(_) | TelegramClientPortError::Persistence(_),
        )
        | TelegramClientTransportError::Io(_)
        | TelegramClientTransportError::RuntimeUnavailable
        | TelegramClientTransportError::Reconfiguration => "RUNTIME_UNAVAILABLE",
    }
}

pub fn serve_connection_durable<T: TdlibTransport>(
    mut stream: UnixStream,
    runtime: &mut TelegramRuntime<T>,
    durable: &TelegramDurablePersistence,
    automation: &TelegramAutomationPersistence,
    calls: &TelegramCallsPersistence,
    handle: &tokio::runtime::Handle,
) -> Result<(), TelegramClientTransportError> {
    loop {
        let request = match read_frame(&mut stream) {
            Ok(request) => request,
            Err(TelegramClientTransportError::Io(error)) if error == "eof" => return Ok(()),
            Err(error) => return Err(error),
        };
        let response = tokio::task::block_in_place(|| {
            handle.block_on(handle_durable_request(
                runtime, durable, automation, calls, &request,
            ))
        })?;
        write_frame(&mut stream, &response)?;
    }
}

pub async fn handle_durable_request<T: TdlibTransport>(
    runtime: &mut TelegramRuntime<T>,
    durable: &TelegramDurablePersistence,
    automation: &TelegramAutomationPersistence,
    calls: &TelegramCallsPersistence,
    request: &[u8],
) -> Result<Vec<u8>, TelegramClientTransportError> {
    if crate::calls_client_port::calls_route(request)
        .map_err(TelegramClientTransportError::Port)?
        .is_some()
    {
        return crate::calls_client_port::handle_calls_module_request(request, runtime, calls)
            .await
            .map_err(TelegramClientTransportError::Port);
    }
    if crate::automation_client_port::automation_route(request)
        .map_err(TelegramClientTransportError::Port)?
        .is_some()
    {
        return crate::automation_client_port::handle_automation_module_request(
            request, automation,
        )
        .await
        .map_err(TelegramClientTransportError::Port);
    }
    crate::client_port::TelegramClientPort::new(runtime)
        .handle_module_request_durable(request, durable)
        .await
        .map_err(TelegramClientTransportError::Port)
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, TelegramClientTransportError> {
    let length = read_length(stream)?;
    if length == 0 || length > MAX_CLIENT_FRAME_BYTES {
        return Err(TelegramClientTransportError::Frame(
            "Telegram client frame length is invalid".to_owned(),
        ));
    }
    let mut bytes = vec![0_u8; length];
    stream.read_exact(&mut bytes).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            TelegramClientTransportError::Io("eof".to_owned())
        } else {
            TelegramClientTransportError::Io("Telegram client transport is unavailable".to_owned())
        }
    })?;
    Ok(bytes)
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), TelegramClientTransportError> {
    if bytes.is_empty() || bytes.len() > MAX_CLIENT_FRAME_BYTES {
        return Err(TelegramClientTransportError::Frame(
            "Telegram client frame length is invalid".to_owned(),
        ));
    }
    let mut length = u32::try_from(bytes.len()).map_err(|_| {
        TelegramClientTransportError::Frame("Telegram client frame length is invalid".to_owned())
    })?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    stream
        .write_all(&prefix)
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|_| {
            TelegramClientTransportError::Io("Telegram client transport is unavailable".to_owned())
        })
}

fn read_length(stream: &mut UnixStream) -> Result<usize, TelegramClientTransportError> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).map_err(|error| {
            if error.kind() == ErrorKind::UnexpectedEof {
                TelegramClientTransportError::Io("eof".to_owned())
            } else {
                TelegramClientTransportError::Io(
                    "Telegram client transport is unavailable".to_owned(),
                )
            }
        })?;
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return usize::try_from(value).map_err(|_| {
                TelegramClientTransportError::Frame(
                    "Telegram client frame length is invalid".to_owned(),
                )
            });
        }
    }
    Err(TelegramClientTransportError::Frame(
        "Telegram client frame length is invalid".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use makosh_telegram_persistence::TelegramDurablePersistenceError;

    use super::*;

    #[test]
    fn module_errors_are_typed_without_exposing_internal_details() {
        assert_eq!(
            module_error_code(&TelegramClientTransportError::Port(
                TelegramClientPortError::Protocol("private detail".to_owned()),
            )),
            "INVALID_ARGUMENT"
        );
        assert_eq!(
            module_error_code(&TelegramClientTransportError::Port(
                TelegramClientPortError::Persistence(TelegramDurablePersistenceError::Database,),
            )),
            "RUNTIME_UNAVAILABLE"
        );
    }
}
