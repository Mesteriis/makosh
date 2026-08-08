//! Private, status-only Unix socket for the Vault runtime process.

use std::io::{Read, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use makosh_runtime_protocol::v1::{
    GetVaultRuntimeStatusRequestV1, VaultRuntimeControlRequestV1, VaultRuntimeControlResponseV1,
    VaultRuntimeStateV1, VaultRuntimeStatusV1, vault_runtime_control_request_v1::Operation,
    vault_runtime_control_response_v1::Result as ResponseResult,
};
use prost::Message;

use crate::transport::keys::VaultTransportKeyPair;

const MAX_FRAME_BYTES: usize = 64 * 1024;
const IPC_TIMEOUT: Duration = Duration::from_secs(5);

pub fn serve(
    runtime_dir: &Path,
    runtime_generation: u64,
    transport_keys: &VaultTransportKeyPair,
    shutdown_requested: &AtomicBool,
) -> Result<(), String> {
    validate_private_directory(runtime_dir)?;
    let socket_path = runtime_dir.join("vault.sock");
    ensure_socket_path(&socket_path)?;
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).map_err(|_| "Vault IPC is unavailable")?;
    let cleanup = SocketCleanup(socket_path);
    listener
        .set_nonblocking(true)
        .map_err(|_| "Vault IPC is unavailable")?;
    std::fs::set_permissions(cleanup.path(), std::fs::Permissions::from_mode(0o600))
        .map_err(|_| "Vault IPC is unavailable")?;

    loop {
        if shutdown_requested.load(Ordering::Acquire) {
            return Ok(());
        }
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(25));
                continue;
            }
            Err(_) => return Err("Vault IPC is unavailable".to_owned()),
        };
        let _ = handle_connection(&mut stream, runtime_generation, transport_keys);
    }
}

struct SocketCleanup(PathBuf);

impl SocketCleanup {
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn handle_connection(
    stream: &mut UnixStream,
    runtime_generation: u64,
    transport_keys: &VaultTransportKeyPair,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(IPC_TIMEOUT)))
        .map_err(|_| "Vault IPC is unavailable".to_owned())?;
    let response = read_frame(stream)
        .and_then(|bytes| {
            VaultRuntimeControlRequestV1::decode(bytes.as_slice())
                .map_err(|_| "invalid_request".to_owned())
        })
        .map(|request| response_for(request, runtime_generation, transport_keys))
        .unwrap_or_else(|_| error_response("invalid_request"));
    write_frame(stream, &response.encode_to_vec())
}

fn response_for(
    request: VaultRuntimeControlRequestV1,
    runtime_generation: u64,
    transport_keys: &VaultTransportKeyPair,
) -> VaultRuntimeControlResponseV1 {
    match request.operation {
        Some(Operation::GetStatus(GetVaultRuntimeStatusRequestV1 {})) => {
            VaultRuntimeControlResponseV1 {
                result: Some(ResponseResult::Status(VaultRuntimeStatusV1 {
                    state: VaultRuntimeStateV1::Ready as i32,
                    vault_runtime_generation: runtime_generation,
                    hpke_public_key_x25519: transport_keys.public_key().as_bytes().to_vec(),
                    blocker_code: String::new(),
                })),
                error_code: String::new(),
            }
        }
        None => error_response("operation_not_available"),
    }
}

fn error_response(error_code: &str) -> VaultRuntimeControlResponseV1 {
    VaultRuntimeControlResponseV1 {
        result: None,
        error_code: error_code.to_owned(),
    }
}

fn validate_private_directory(path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| "Vault IPC is unavailable")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.mode() & 0o077 != 0
        || metadata.uid() != unsafe { libc::geteuid() }
    {
        return Err("Vault IPC is unavailable".to_owned());
    }
    Ok(())
}

fn ensure_socket_path(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.file_type().is_symlink()
                || !(metadata.is_file() || metadata.file_type().is_socket())
                || metadata.mode() & 0o077 != 0
                || metadata.uid() != unsafe { libc::geteuid() } =>
        {
            Err("Vault IPC is unavailable".to_owned())
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Vault IPC is unavailable".to_owned()),
    }
}

fn read_frame(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(stream)?).map_err(|_| "invalid_request".to_owned())?;
    if length > MAX_FRAME_BYTES {
        return Err("invalid_request".to_owned());
    }
    let mut bytes = vec![0; length];
    stream
        .read_exact(&mut bytes)
        .map_err(|_| "invalid_request".to_owned())?;
    Ok(bytes)
}

fn read_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|_| "invalid_request".to_owned())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("invalid_request".to_owned())
}

fn write_frame(stream: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let mut length =
        u32::try_from(bytes.len()).map_err(|_| "Vault IPC is unavailable".to_owned())?;
    while length >= 0x80 {
        stream
            .write_all(&[(length as u8 & 0x7f) | 0x80])
            .map_err(|_| "Vault IPC is unavailable".to_owned())?;
        length >>= 7;
    }
    stream
        .write_all(&[length as u8])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|_| "Vault IPC is unavailable".to_owned())
}
