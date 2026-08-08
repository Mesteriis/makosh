//! Owner-private Unix IPC transport for Module Registry control operations.

pub(crate) mod cli;
mod dispatch;
pub(crate) mod sessions;

use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use makosh_gateway_protocol::v1::{OwnerControlRequestV1, OwnerControlResponseV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use prost::Message;

use crate::identity::owner_control::sessions::OwnerControlSessions;
use crate::infrastructure::filesystem::remove_stale_owner_unix_socket;
use crate::platform::gateway::BrowserPairingAdmissionV1;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const MAX_FRAME_BYTES: usize = 64 * 1024;
const IPC_TIMEOUT: Duration = Duration::from_secs(5);
const SHUTDOWN_POLL: Duration = Duration::from_millis(25);

pub fn serve(
    store: Arc<SqliteControlStore>,
    data_dir: &Path,
    runtime_dir: &Path,
    shutdown_requested: Arc<AtomicBool>,
    managed_runtime_supervisor: ManagedRuntimeSupervisor,
    browser_pairing: Option<Arc<BrowserPairingAdmissionV1>>,
) -> Result<(), String> {
    let socket_path = runtime_dir.join("owner.sock");
    remove_stale_owner_unix_socket(&socket_path, "owner control socket")?;
    let listener = UnixListener::bind(&socket_path).map_err(|error| error.to_string())?;
    let cleanup = SocketCleanup(socket_path.clone());
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;
    println!("owner_control_socket={}", socket_path.display());
    let mut sessions = OwnerControlSessions::default();
    let result = accept_connections(OwnerControlConnectionLoopV1 {
        store: &store,
        data_dir,
        runtime_dir,
        shutdown_requested,
        supervisor: &managed_runtime_supervisor,
        browser_pairing: browser_pairing.as_deref(),
        listener: &listener,
        sessions: &mut sessions,
    });
    drop(cleanup);
    result
}

struct OwnerControlConnectionLoopV1<'a> {
    store: &'a SqliteControlStore,
    data_dir: &'a Path,
    runtime_dir: &'a Path,
    shutdown_requested: Arc<AtomicBool>,
    supervisor: &'a ManagedRuntimeSupervisor,
    browser_pairing: Option<&'a BrowserPairingAdmissionV1>,
    listener: &'a UnixListener,
    sessions: &'a mut OwnerControlSessions,
}

fn accept_connections(input: OwnerControlConnectionLoopV1<'_>) -> Result<(), String> {
    loop {
        if input.shutdown_requested.load(Ordering::Acquire) {
            return Ok(());
        }
        let mut stream = match input.listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(SHUTDOWN_POLL);
                continue;
            }
            Err(error) => return Err(error.to_string()),
        };
        let _ = handle_connection(
            input.store,
            input.data_dir,
            input.runtime_dir,
            input.supervisor,
            input.browser_pairing,
            input.sessions,
            &mut stream,
        );
    }
}

struct SocketCleanup(PathBuf);

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn handle_connection(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    browser_pairing: Option<&BrowserPairingAdmissionV1>,
    sessions: &mut OwnerControlSessions,
    stream: &mut UnixStream,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(IPC_TIMEOUT)))
        .map_err(|error| error.to_string())?;
    let response = match decode_request(stream) {
        Ok(request) => dispatch::handle(
            store,
            data_dir,
            runtime_dir,
            supervisor,
            browser_pairing,
            sessions,
            request,
        ),
        Err(_) => OwnerControlResponseV1 {
            result: None,
            error_code: "invalid_request".to_owned(),
        },
    };
    write_frame(stream, &response.encode_to_vec())
}

fn decode_request(stream: &mut UnixStream) -> Result<OwnerControlRequestV1, String> {
    let bytes = read_frame(stream)?;
    OwnerControlRequestV1::decode(bytes.as_slice())
        .map_err(|_| "invalid owner control request".to_owned())
}

pub(super) fn read_frame(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(stream)?)
        .map_err(|_| "owner control frame is too large".to_owned())?;
    if length > MAX_FRAME_BYTES {
        return Err("owner control frame is too large".to_owned());
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
    Err("owner control frame length is invalid".to_owned())
}

pub(super) fn write_frame(stream: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let mut length =
        u32::try_from(bytes.len()).map_err(|_| "owner control response is too large".to_owned())?;
    let mut prefix = [0_u8; 5];
    let mut index = 0;
    while length >= 0x80 {
        prefix[index] = (length as u8) | 0x80;
        length >>= 7;
        index += 1;
    }
    prefix[index] = length as u8;
    stream
        .write_all(&prefix[..=index])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}
