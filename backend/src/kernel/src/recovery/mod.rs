pub(crate) mod capture_coordinator;
pub(crate) mod control_store_media;
mod dispatch;
mod export;
pub(crate) mod fence;
mod framing;
pub(crate) mod media;
pub(crate) mod offline;
pub(crate) mod process_port;
pub(crate) mod restore_coordinator;

use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use makosh_gateway_protocol::v1::RecoveryControlRequestV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use prost::Message;

use crate::infrastructure::filesystem::remove_stale_owner_unix_socket;

use self::dispatch::recovery_response;
use self::framing::{read_recovery_frame, write_recovery_frame};

const RECOVERY_IPC_TIMEOUT: Duration = Duration::from_secs(5);
const RECOVERY_SHUTDOWN_POLL: Duration = Duration::from_millis(25);

pub fn serve_recovery_socket(
    runtime_dir: &Path,
    store_path: &Path,
    online_store: Option<Arc<SqliteControlStore>>,
    shutdown_requested: Arc<AtomicBool>,
) -> Result<(), String> {
    let socket_path = runtime_dir.join("recovery.sock");
    remove_stale_owner_unix_socket(&socket_path, "recovery socket")?;
    let listener = bind_private_listener(&socket_path)?;
    let _socket_cleanup = RecoverySocketCleanup(socket_path.clone());
    println!("recovery_socket={}", socket_path.display());

    loop {
        if shutdown_requested.load(Ordering::Acquire) {
            return Ok(());
        }
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(RECOVERY_SHUTDOWN_POLL);
                continue;
            }
            Err(error) => return Err(error.to_string()),
        };
        match handle_connection(&mut stream, store_path, online_store.as_deref()) {
            Ok(true) => {
                shutdown_requested.store(true, Ordering::Release);
                return Ok(());
            }
            Ok(false) => {}
            Err(_) => eprintln!("recovery_connection_error=invalid_request"),
        }
    }
}

fn bind_private_listener(socket_path: &Path) -> Result<UnixListener, String> {
    let listener = UnixListener::bind(socket_path).map_err(|error| error.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;
    Ok(listener)
}

fn handle_connection(
    stream: &mut UnixStream,
    store_path: &Path,
    online_store: Option<&SqliteControlStore>,
) -> Result<bool, String> {
    stream
        .set_read_timeout(Some(RECOVERY_IPC_TIMEOUT))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(RECOVERY_IPC_TIMEOUT))
        .map_err(|error| error.to_string())?;
    let bytes = read_recovery_frame(stream)?;
    let request = RecoveryControlRequestV1::decode(bytes.as_slice())
        .map_err(|_| "invalid recovery IPC request".to_owned())?;
    let action = recovery_response(request, store_path, online_store);
    write_recovery_frame(stream, &action.response.encode_to_vec())?;
    Ok(action.shutdown)
}

struct RecoverySocketCleanup(PathBuf);

impl Drop for RecoverySocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
