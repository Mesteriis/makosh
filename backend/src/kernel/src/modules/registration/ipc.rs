//! Private Unix IPC transport for bounded, pending Module Registry registration.

use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use makosh_gateway_protocol::v1::{
    BeginModuleRegistrationResponseV1, DescribeModuleRegistrationResponseV1,
    GetOwnModuleRegistrationStatusResponseV1, HelloModuleRegistrationResponseV1,
    ModuleRegistrationRequestV1, ModuleRegistrationResponseV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use prost::Message;

use crate::infrastructure::filesystem::remove_stale_owner_unix_socket;
use crate::modules::registration::registry as module_registry;
use crate::modules::registration::sessions::RegistrationSessions;

const MAX_FRAME_BYTES: usize = 64 * 1024;
const IPC_TIMEOUT: Duration = Duration::from_secs(5);
const SHUTDOWN_POLL: Duration = Duration::from_millis(25);

pub fn serve(
    store: Arc<SqliteControlStore>,
    runtime_dir: &Path,
    shutdown_requested: Arc<AtomicBool>,
) -> Result<(), String> {
    let socket_path = runtime_dir.join("reg.sock");
    remove_stale_owner_unix_socket(&socket_path, "module registration socket")?;
    let listener = UnixListener::bind(&socket_path).map_err(|error| error.to_string())?;
    let cleanup = SocketCleanup(socket_path.clone());
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;
    println!("module_registration_socket={}", socket_path.display());
    let mut sessions = RegistrationSessions::default();
    let result = loop {
        if shutdown_requested.load(Ordering::Acquire) {
            break Ok(());
        }
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(SHUTDOWN_POLL);
                continue;
            }
            Err(error) => break Err(error.to_string()),
        };
        let _ = handle_connection(&store, &mut sessions, &mut stream);
    };
    drop(cleanup);
    result
}

struct SocketCleanup(PathBuf);

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn handle_connection(
    store: &SqliteControlStore,
    sessions: &mut RegistrationSessions,
    stream: &mut UnixStream,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(IPC_TIMEOUT)))
        .map_err(|error| error.to_string())?;
    let response = match read_frame(stream).and_then(|bytes| {
        ModuleRegistrationRequestV1::decode(bytes.as_slice())
            .map_err(|_| "invalid module registration request".to_owned())
    }) {
        Ok(request) => handle(store, sessions, request),
        Err(_) => error_response("invalid_request"),
    };
    write_frame(stream, &response.encode_to_vec())
}

fn handle(
    store: &SqliteControlStore,
    sessions: &mut RegistrationSessions,
    request: ModuleRegistrationRequestV1,
) -> ModuleRegistrationResponseV1 {
    use makosh_gateway_protocol::v1::module_registration_request_v1::Operation;
    use makosh_gateway_protocol::v1::module_registration_response_v1::Result;

    let result = match request.operation {
        Some(Operation::Hello(_)) => Ok(Result::Hello(HelloModuleRegistrationResponseV1 {
            protocol_major: 1,
            protocol_minor: 0,
        })),
        Some(Operation::Begin(_)) => {
            sessions
                .begin()
                .map(|(session_id, expires_at_unix_millis)| {
                    Result::Begin(BeginModuleRegistrationResponseV1 {
                        session_id,
                        expires_at_unix_millis,
                    })
                })
        }
        Some(Operation::Describe(request)) => sessions
            .start_describe(&request.session_id)
            .and_then(|_| module_registry::register(store, &request.descriptor_bytes))
            .and_then(|registration| {
                sessions.record(
                    &request.session_id,
                    registration.registration_id().to_owned(),
                )?;
                Ok(Result::Describe(DescribeModuleRegistrationResponseV1 {
                    registration_id: registration.registration_id().to_owned(),
                    registration_state: registration.state().as_str().to_owned(),
                }))
            }),
        Some(Operation::GetOwnStatus(request)) => sessions
            .registration_id(&request.session_id)
            .and_then(|registration_id| module_registry::status(store, &registration_id))
            .map(|status| {
                Result::GetOwnStatus(GetOwnModuleRegistrationStatusResponseV1 {
                    registration_id: status.registration().registration_id().to_owned(),
                    registration_state: status.registration().state().as_str().to_owned(),
                })
            }),
        None => Err("invalid_request".to_owned()),
    };
    registration_response(result)
}

fn registration_response(
    result: Result<makosh_gateway_protocol::v1::module_registration_response_v1::Result, String>,
) -> ModuleRegistrationResponseV1 {
    match result {
        Ok(result) => ModuleRegistrationResponseV1 {
            result: Some(result),
            error_code: String::new(),
        },
        Err(error) => error_response(error_code(&error)),
    }
}

fn error_response(error_code: &str) -> ModuleRegistrationResponseV1 {
    ModuleRegistrationResponseV1 {
        result: None,
        error_code: error_code.to_owned(),
    }
}

fn error_code(error: &str) -> &'static str {
    match error {
        "registration_rate_limited" => "registration_rate_limited",
        "registration_session_unavailable" => "registration_session_unavailable",
        "invalid_request" => "invalid_request",
        _ => "registration_denied",
    }
}

fn read_frame(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(stream)?)
        .map_err(|_| "module registration frame is too large".to_owned())?;
    if length > MAX_FRAME_BYTES {
        return Err("module registration frame is too large".to_owned());
    }
    let mut bytes = vec![0_u8; length];
    stream
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn read_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("module registration frame length is invalid".to_owned())
}

fn write_frame(stream: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let mut length = u32::try_from(bytes.len())
        .map_err(|_| "module registration response is too large".to_owned())?;
    while length >= 0x80 {
        stream
            .write_all(&[(length as u8 & 0x7f) | 0x80])
            .map_err(|error| error.to_string())?;
        length >>= 7;
    }
    stream
        .write_all(&[length as u8])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}
