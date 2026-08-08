//! Owner-private native-host route descriptor with managed-runtime lifetime.

use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use makosh_runtime_protocol::{
    v1::ManagedIntegrationHostBridgeConfigurationV1,
    validation::integration_host_bridge::validate_managed_integration_host_bridge_configuration,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub(crate) struct PublishedHostBridgeDescriptor {
    path: PathBuf,
}

impl PublishedHostBridgeDescriptor {
    pub(crate) fn remove(self, socket_path: &Path) {
        if let Ok(metadata) = std::fs::symlink_metadata(&self.path)
            && metadata.file_type().is_file()
            && !metadata.file_type().is_symlink()
            && metadata.uid() == current_uid()
        {
            let _ = std::fs::remove_file(self.path);
        }
        remove_owned_socket(socket_path);
    }
}

fn remove_owned_socket(path: &Path) {
    if let Ok(metadata) = std::fs::symlink_metadata(path)
        && metadata.file_type().is_socket()
        && !metadata.file_type().is_symlink()
        && metadata.uid() == current_uid()
    {
        let _ = std::fs::remove_file(path);
    }
}

pub(crate) fn publish(
    runtime_dir: &Path,
    configuration: &ManagedIntegrationHostBridgeConfigurationV1,
) -> Result<PublishedHostBridgeDescriptor, String> {
    validate_managed_integration_host_bridge_configuration(configuration)
        .map_err(|_| "managed host route descriptor is invalid".to_owned())?;
    let directory = runtime_dir.join("host-bridges");
    crate::infrastructure::filesystem::ensure_owner_private_directory(&directory)
        .map_err(|_| "managed host route descriptor directory is invalid".to_owned())?;
    let path = directory.join(descriptor_file_name(&configuration.registration_id));
    validate_target(&path)?;
    let temporary = directory.join(format!(
        ".{}-{}-{}.tmp",
        descriptor_file_name(&configuration.registration_id),
        configuration.runtime_generation,
        std::process::id(),
    ));
    if std::fs::symlink_metadata(&temporary).is_ok() {
        return Err("managed host route descriptor temporary path is unavailable".to_owned());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(&temporary)
        .map_err(|_| "managed host route descriptor is unavailable".to_owned())?;
    if file
        .write_all(&configuration.encode_to_vec())
        .and_then(|_| file.sync_all())
        .is_err()
    {
        let _ = std::fs::remove_file(&temporary);
        return Err("managed host route descriptor is unavailable".to_owned());
    }
    std::fs::rename(&temporary, &path).map_err(|_| {
        let _ = std::fs::remove_file(&temporary);
        "managed host route descriptor is unavailable".to_owned()
    })?;
    Ok(PublishedHostBridgeDescriptor { path })
}

fn descriptor_file_name(registration_id: &str) -> String {
    let digest = Sha256::digest(registration_id.as_bytes());
    format!(
        "route-{}.bin",
        digest[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
    )
}

fn validate_target(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.file_type().is_symlink()
                || !metadata.file_type().is_file()
                || metadata.uid() != current_uid()
                || metadata.mode() & 0o077 != 0 =>
        {
            Err("managed host route descriptor target is invalid".to_owned())
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("managed host route descriptor target is invalid".to_owned()),
    }
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn publishes_and_removes_only_the_exact_private_route_descriptor() {
        let root = std::env::temp_dir().join(format!(
            "hhr-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .subsec_nanos(),
        ));
        let directory = root.join("host-bridges");
        std::fs::create_dir_all(&directory).expect("directory");
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))
            .expect("root permissions");
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))
            .expect("directory permissions");
        let configuration = ManagedIntegrationHostBridgeConfigurationV1 {
            major: 1,
            kernel_instance_id: "kernel_1".to_owned(),
            owner_id: "whatsapp".to_owned(),
            registration_id: "whatsapp_runtime".to_owned(),
            runtime_instance_id: "whatsapp_runtime_1".to_owned(),
            runtime_generation: 2,
            grant_epoch: 3,
            socket_path: "/private/tmp/makosh/whatsapp.sock".to_owned(),
            route_binding_sha256: vec![9; 32],
        };

        let descriptor = publish(&root, &configuration).expect("descriptor");
        let path = root
            .join("host-bridges")
            .join(descriptor_file_name(&configuration.registration_id));
        let metadata = std::fs::metadata(&path).expect("metadata");

        assert_eq!(metadata.permissions().mode() & 0o777, 0o400);
        assert_eq!(
            ManagedIntegrationHostBridgeConfigurationV1::decode(
                std::fs::read(&path).expect("descriptor bytes").as_slice()
            )
            .expect("decoded descriptor"),
            configuration
        );
        let socket_path = root.join("whatsapp.sock");
        let listener = std::os::unix::net::UnixListener::bind(&socket_path).expect("host socket");
        descriptor.remove(&socket_path);
        assert!(!path.exists());
        assert!(!socket_path.exists());
        drop(listener);
        std::fs::remove_dir_all(root).expect("cleanup root");
    }
}
