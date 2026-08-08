//! Kernel-owned topology for durable, integration-owned provider state roots.

use std::path::{Path, PathBuf};

use makosh_runtime_protocol::v1::IntegrationStateRootV1;

use crate::infrastructure::filesystem::{
    ensure_owner_private_directory, prepare_owner_private_directory,
};

const STATE_GENERATION_V1: u64 = 1;
const MAX_IDENTIFIER_BYTES: usize = 128;

pub fn prepare(
    data_dir: &Path,
    owner_id: &str,
    registration_id: &str,
    configuration_instance_id: &str,
    state_layout_revision: u32,
) -> Result<IntegrationStateRootV1, String> {
    if state_layout_revision == 0
        || !valid_identifier(owner_id)
        || !valid_identifier(registration_id)
        || !valid_identifier(configuration_instance_id)
    {
        return Err("integration state root identity is invalid".to_owned());
    }
    ensure_owner_private_directory(data_dir)?;
    let canonical_data_dir = std::fs::canonicalize(data_dir).map_err(|error| error.to_string())?;
    let mut current = canonical_data_dir.clone();
    for component in [
        "integration-state",
        owner_id,
        registration_id,
        configuration_instance_id,
    ] {
        current.push(component);
        prepare_owner_private_directory(&current)?;
        current = canonical_private_child(&canonical_data_dir, &current)?;
    }
    let root_path = path_string(&current)?;
    Ok(IntegrationStateRootV1 {
        root_path,
        state_generation: STATE_GENERATION_V1,
        state_layout_revision,
    })
}

fn canonical_private_child(data_dir: &Path, child: &Path) -> Result<PathBuf, String> {
    ensure_owner_private_directory(child)?;
    let canonical = std::fs::canonicalize(child).map_err(|error| error.to_string())?;
    if !canonical.starts_with(data_dir) || canonical == data_dir {
        return Err("integration state root escapes the Макошь data directory".to_owned());
    }
    Ok(canonical)
}

fn path_string(path: &Path) -> Result<String, String> {
    path.to_str()
        .filter(|value| !value.contains(['\0', '\n', '\r']))
        .map(ToOwned::to_owned)
        .ok_or_else(|| "integration state root path is invalid".to_owned())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::prepare;

    #[test]
    fn creates_one_owner_private_root_without_client_path_input() {
        let data_dir = test_directory("private");
        std::fs::create_dir(&data_dir).expect("data dir");
        std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))
            .expect("permissions");

        let root =
            prepare(&data_dir, "telegram", "registration-1", "account-1", 3).expect("state root");
        assert!(
            std::path::Path::new(&root.root_path)
                .starts_with(std::fs::canonicalize(&data_dir).expect("canonical data dir"))
        );
        assert_eq!(root.state_generation, 1);
        assert_eq!(root.state_layout_revision, 3);
        let mode = std::fs::symlink_metadata(&root.root_path)
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(mode & 0o077, 0);

        std::fs::remove_dir_all(data_dir).expect("cleanup");
    }

    #[test]
    fn rejects_a_symlinked_state_parent() {
        let data_dir = test_directory("symlink");
        let outside = test_directory("outside");
        std::fs::create_dir(&data_dir).expect("data dir");
        std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))
            .expect("permissions");
        std::fs::create_dir(&outside).expect("outside");
        symlink(&outside, data_dir.join("integration-state")).expect("symlink");

        assert!(prepare(&data_dir, "telegram", "registration-1", "account-1", 1,).is_err());

        std::fs::remove_file(data_dir.join("integration-state")).expect("remove symlink");
        std::fs::remove_dir(data_dir).expect("cleanup data");
        std::fs::remove_dir(outside).expect("cleanup outside");
    }

    fn test_directory(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "makosh-integration-state-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
