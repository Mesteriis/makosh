use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use makosh_desktop_call_recording_api::{
    HOST_PROTOCOL_MAJOR_V1, HOST_PROTOCOL_REVISION_V1, MAX_AUDIO_BYTES_V1,
    wire::{
        DesktopRecordingHostCommandLeaseV1, DesktopRecordingHostHandshakeAcceptedV1,
        DesktopRecordingHostHandshakeV1, DesktopRecordingHostObservationAcceptedV1,
        DesktopRecordingHostOperationV1,
    },
};
use makosh_runtime_protocol::{
    v1::ManagedIntegrationHostBridgeConfigurationV1,
    validation::integration_host_bridge::validate_managed_integration_host_bridge_configuration,
};
use prost::Message;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

const OWNER_ID: &str = "desktop_call_recording";
const MAX_DESCRIPTOR_BYTES: u64 = 8 * 1024;
const MAX_FRAME_BYTES: usize = MAX_AUDIO_BYTES_V1 + 32 * 1024;
const BRIDGE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub(super) struct HostRouteClientV1 {
    route: ManagedIntegrationHostBridgeConfigurationV1,
}

impl HostRouteClientV1 {
    pub(super) fn load(app: &AppHandle, registration_id: &str) -> Result<Self, String> {
        validate_registration_id(registration_id)?;
        let data_dir = app.path().app_local_data_dir().map_err(|_| unavailable())?;
        let path = kernel_runtime_directory(&data_dir)?
            .join("host-bridges")
            .join(host_descriptor_file_name(registration_id));
        let metadata = fs::symlink_metadata(&path).map_err(|_| unavailable())?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.uid() != current_uid()
            || metadata.permissions().mode() & 0o077 != 0
            || metadata.len() == 0
            || metadata.len() > MAX_DESCRIPTOR_BYTES
        {
            return Err(unavailable());
        }
        let route = ManagedIntegrationHostBridgeConfigurationV1::decode(
            fs::read(path).map_err(|_| unavailable())?.as_slice(),
        )
        .map_err(|_| unavailable())?;
        validate_managed_integration_host_bridge_configuration(&route)
            .map_err(|_| unavailable())?;
        if route.owner_id != OWNER_ID || route.registration_id != registration_id {
            return Err(unavailable());
        }
        Ok(Self { route })
    }

    pub(super) fn admitted_route_exists(app: &AppHandle) -> bool {
        let Ok(data_dir) = app.path().app_local_data_dir() else {
            return false;
        };
        let Ok(directory) =
            kernel_runtime_directory(&data_dir).map(|path| path.join("host-bridges"))
        else {
            return false;
        };
        let Ok(metadata) = fs::symlink_metadata(&directory) else {
            return false;
        };
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_dir()
            || metadata.uid() != current_uid()
            || metadata.permissions().mode() & 0o077 != 0
        {
            return false;
        }
        let Ok(entries) = fs::read_dir(directory) else {
            return false;
        };
        entries.filter_map(Result::ok).any(|entry| {
            let Ok(metadata) = fs::symlink_metadata(entry.path()) else {
                return false;
            };
            if metadata.file_type().is_symlink()
                || !metadata.file_type().is_file()
                || metadata.uid() != current_uid()
                || metadata.permissions().mode() & 0o077 != 0
                || metadata.len() == 0
                || metadata.len() > MAX_DESCRIPTOR_BYTES
            {
                return false;
            }
            let Ok(bytes) = fs::read(entry.path()) else {
                return false;
            };
            let Ok(route) = ManagedIntegrationHostBridgeConfigurationV1::decode(bytes.as_slice())
            else {
                return false;
            };
            let expected_name =
                std::ffi::OsString::from(host_descriptor_file_name(&route.registration_id));
            route.owner_id == OWNER_ID
                && validate_registration_id(&route.registration_id).is_ok()
                && entry.file_name() == expected_name
                && validate_managed_integration_host_bridge_configuration(&route).is_ok()
        })
    }

    pub(super) fn claim(
        &self,
        operation: DesktopRecordingHostOperationV1,
    ) -> Result<DesktopRecordingHostCommandLeaseV1, String> {
        DesktopRecordingHostCommandLeaseV1::decode(self.exchange(operation)?.as_slice())
            .map_err(|_| unavailable())
    }

    pub(super) fn observe(
        &self,
        operation: DesktopRecordingHostOperationV1,
    ) -> Result<DesktopRecordingHostObservationAcceptedV1, String> {
        DesktopRecordingHostObservationAcceptedV1::decode(self.exchange(operation)?.as_slice())
            .map_err(|_| unavailable())
    }

    fn exchange(&self, operation: DesktopRecordingHostOperationV1) -> Result<Vec<u8>, String> {
        let route_binding: [u8; 32] = self
            .route
            .route_binding_sha256
            .as_slice()
            .try_into()
            .map_err(|_| unavailable())?;
        let mut stream = UnixStream::connect(&self.route.socket_path).map_err(|_| unavailable())?;
        stream
            .set_read_timeout(Some(BRIDGE_TIMEOUT))
            .and_then(|_| stream.set_write_timeout(Some(BRIDGE_TIMEOUT)))
            .map_err(|_| unavailable())?;
        write_frame(
            &mut stream,
            &DesktopRecordingHostHandshakeV1 {
                protocol_major: HOST_PROTOCOL_MAJOR_V1,
                protocol_revision: HOST_PROTOCOL_REVISION_V1,
                route_binding_sha256: route_binding.to_vec(),
            }
            .encode_to_vec(),
        )?;
        let accepted =
            DesktopRecordingHostHandshakeAcceptedV1::decode(read_frame(&mut stream)?.as_slice())
                .map_err(|_| unavailable())?;
        if accepted.protocol_major != HOST_PROTOCOL_MAJOR_V1
            || accepted.protocol_revision != HOST_PROTOCOL_REVISION_V1
        {
            return Err(unavailable());
        }
        write_frame(&mut stream, &operation.encode_to_vec())?;
        read_frame(&mut stream)
    }
}

fn validate_registration_id(value: &str) -> Result<(), String> {
    (!value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')))
    .then_some(())
    .ok_or_else(unavailable)
}

fn kernel_runtime_directory(data_dir: &Path) -> Result<PathBuf, String> {
    let data_dir = data_dir.canonicalize().map_err(|_| unavailable())?;
    let project =
        directories::ProjectDirs::from("dev", "Макошь", "Макошь").ok_or_else(unavailable)?;
    let digest = Sha256::digest(data_dir.as_os_str().as_encoded_bytes());
    let instance_key = digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(project.cache_dir().join("runtime").join(instance_key))
}

fn host_descriptor_file_name(registration_id: &str) -> String {
    let digest = Sha256::digest(registration_id.as_bytes());
    format!(
        "route-{}.bin",
        digest[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(unavailable());
    }
    let mut length = u32::try_from(bytes.len()).map_err(|_| unavailable())?;
    let mut prefix = [0_u8; 5];
    let mut index = 0;
    while length >= 0x80 {
        prefix[index] = (length as u8 & 0x7f) | 0x80;
        length >>= 7;
        index += 1;
    }
    prefix[index] = length as u8;
    stream
        .write_all(&prefix[..=index])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|_| unavailable())
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let mut length = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).map_err(|_| unavailable())?;
        length |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            let length = usize::try_from(length).map_err(|_| unavailable())?;
            if length == 0 || length > MAX_FRAME_BYTES {
                return Err(unavailable());
            }
            let mut bytes = vec![0_u8; length];
            stream.read_exact(&mut bytes).map_err(|_| unavailable())?;
            return Ok(bytes);
        }
    }
    Err(unavailable())
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() }
}

fn unavailable() -> String {
    "Desktop recording host route is unavailable".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_name_is_owner_scoped_and_registration_bound() {
        let first = host_descriptor_file_name("recording.runtime.v1");
        let second = host_descriptor_file_name("recording.runtime.v2");
        assert!(first.starts_with("route-"));
        assert_ne!(first, second);
    }

    #[test]
    fn registration_id_is_bounded() {
        assert!(validate_registration_id("recording.runtime.v1").is_ok());
        assert!(validate_registration_id("../runtime").is_err());
    }
}
