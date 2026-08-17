use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};

use makosh_runtime_protocol::v1::{
    ManagedIntegrationRuntimeConfigurationV1, ManagedRuntimeArtifactBindingV1, RuntimeArtifactUseV1,
};
use makosh_secure_file::{SecureReadPolicy, read};
use makosh_telegram_runtime::admission::{
    TELEGRAM_STATE_LAYOUT_REVISION_V1, TELEGRAM_TDJSON_ARTIFACT_ID, TELEGRAM_TGCALLS_ARTIFACT_ID,
};
use sha2::{Digest, Sha256};

const TDLIB_STATE_DIRECTORY_V1: &str = "tdlib-v1";
const MAX_TDJSON_ARTIFACT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_TGCALLS_ARTIFACT_BYTES: u64 = 128 * 1024 * 1024;

pub(crate) struct TelegramRuntimeBindingsV1 {
    tdjson_artifact_path: PathBuf,
    tgcalls_artifact_path: PathBuf,
    database_directory: PathBuf,
}

impl TelegramRuntimeBindingsV1 {
    pub(crate) fn tdjson_artifact_path(&self) -> &Path {
        &self.tdjson_artifact_path
    }

    pub(crate) fn tgcalls_artifact_path(&self) -> &Path {
        &self.tgcalls_artifact_path
    }

    #[cfg(test)]
    pub(crate) fn database_directory(&self) -> &Path {
        &self.database_directory
    }

    pub(crate) fn into_database_directory(self) -> PathBuf {
        self.database_directory
    }
}

pub(crate) fn resolve(
    configuration: &ManagedIntegrationRuntimeConfigurationV1,
) -> Result<TelegramRuntimeBindingsV1, String> {
    if configuration.runtime_artifacts.len() != 2
        || configuration.runtime_artifacts[0].artifact_id != TELEGRAM_TDJSON_ARTIFACT_ID
        || configuration.runtime_artifacts[1].artifact_id != TELEGRAM_TGCALLS_ARTIFACT_ID
    {
        return Err(invalid_bindings("artifact_contract"));
    }
    let tdjson_artifact_path = resolve_native_artifact(
        &configuration.runtime_artifacts[0],
        MAX_TDJSON_ARTIFACT_BYTES,
    )?;
    let tgcalls_artifact_path = resolve_native_artifact(
        &configuration.runtime_artifacts[1],
        MAX_TGCALLS_ARTIFACT_BYTES,
    )?;

    let state_root = configuration
        .integration_state_root
        .as_ref()
        .filter(|root| {
            root.state_generation != 0
                && root.state_layout_revision == TELEGRAM_STATE_LAYOUT_REVISION_V1
        })
        .ok_or_else(|| invalid_bindings("state_contract"))?;
    let database_directory = prepare_database_directory(Path::new(&state_root.root_path))?;
    Ok(TelegramRuntimeBindingsV1 {
        tdjson_artifact_path,
        tgcalls_artifact_path,
        database_directory,
    })
}

fn resolve_native_artifact(
    artifact: &ManagedRuntimeArtifactBindingV1,
    maximum_size_bytes: u64,
) -> Result<PathBuf, String> {
    if RuntimeArtifactUseV1::try_from(artifact.r#use)
        .ok()
        .is_none_or(|value| value != RuntimeArtifactUseV1::NativeDynamicLibrary)
        || artifact.size_bytes == 0
        || artifact.size_bytes > maximum_size_bytes
        || artifact.sha256.len() != 32
    {
        return Err(invalid_bindings("artifact_metadata"));
    }
    let artifact_path = PathBuf::from(&artifact.staged_path);
    let bytes = read(
        &artifact_path,
        SecureReadPolicy::owner_private(artifact.size_bytes),
    )
    .map_err(|_| invalid_bindings("artifact_read"))?;
    if bytes.len() as u64 != artifact.size_bytes
        || Sha256::digest(&bytes).as_slice() != artifact.sha256.as_slice()
    {
        return Err(invalid_bindings("artifact_digest"));
    }
    Ok(artifact_path)
}

fn prepare_database_directory(root: &Path) -> Result<PathBuf, String> {
    if !root.is_absolute() {
        return Err(invalid_bindings("state_root_path"));
    }
    validate_private_directory(root)?;
    let canonical_root =
        fs::canonicalize(root).map_err(|_| invalid_bindings("state_root_canonical"))?;
    let database_directory = canonical_root.join(TDLIB_STATE_DIRECTORY_V1);
    match DirBuilder::new().mode(0o700).create(&database_directory) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err(invalid_bindings("state_database_create")),
    }
    validate_private_directory(&database_directory)?;
    let canonical_database = fs::canonicalize(&database_directory)
        .map_err(|_| invalid_bindings("state_database_canonical"))?;
    if !canonical_database.starts_with(&canonical_root) || canonical_database == canonical_root {
        return Err(invalid_bindings("state_database_boundary"));
    }
    Ok(canonical_database)
}

fn validate_private_directory(path: &Path) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| invalid_bindings("state_directory_metadata"))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o077 != 0
    {
        return Err(invalid_bindings("state_directory_permissions"));
    }
    Ok(())
}

fn invalid_bindings(stage: &'static str) -> String {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_telegram_runtime_binding_error stage={stage}");
    }
    "Telegram runtime platform bindings are invalid".to_owned()
}

#[cfg(test)]
mod tests {
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use makosh_runtime_protocol::v1::{
        IntegrationStateRootV1, ManagedIntegrationRuntimeConfigurationV1,
        ManagedRuntimeArtifactBindingV1, RuntimeArtifactUseV1,
    };
    use makosh_telegram_runtime::admission::{
        TELEGRAM_TDJSON_ARTIFACT_ID, TELEGRAM_TGCALLS_ARTIFACT_ID,
    };
    use sha2::{Digest, Sha256};

    use super::resolve;

    #[test]
    fn resolves_only_exact_staged_native_artifacts_and_private_state_root() {
        let directory = test_directory("exact");
        let tdjson = write_artifact(&directory, "libtdjson.dylib", b"exact-tdlib");
        let tgcalls = write_artifact(
            &directory,
            "libmakosh_tgcalls_bridge.dylib",
            b"exact-tgcalls",
        );
        let state_root = directory.join("state");
        fs::create_dir(&state_root).expect("create state root");
        fs::set_permissions(&state_root, fs::Permissions::from_mode(0o700))
            .expect("protect state root");
        let configuration = configuration(
            &tdjson,
            b"exact-tdlib",
            &tgcalls,
            b"exact-tgcalls",
            &state_root,
        );

        let bindings = resolve(&configuration).expect("resolve exact runtime bindings");

        assert_eq!(bindings.tdjson_artifact_path(), tdjson);
        assert_eq!(bindings.tgcalls_artifact_path(), tgcalls);
        assert_eq!(
            bindings.database_directory(),
            fs::canonicalize(&state_root)
                .expect("canonical state root")
                .join("tdlib-v1")
        );
        assert!(
            bindings
                .database_directory()
                .metadata()
                .expect("state metadata")
                .is_dir()
        );
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn rejects_tampered_artifact_and_symlinked_state_child() {
        let directory = test_directory("tampered");
        let tdjson = write_artifact(&directory, "libtdjson.dylib", b"exact-tdlib");
        let tgcalls = write_artifact(
            &directory,
            "libmakosh_tgcalls_bridge.dylib",
            b"exact-tgcalls",
        );
        let state_root = directory.join("state");
        fs::create_dir(&state_root).expect("create state root");
        fs::set_permissions(&state_root, fs::Permissions::from_mode(0o700))
            .expect("protect state root");
        let configuration = configuration(
            &tdjson,
            b"exact-tdlib",
            &tgcalls,
            b"exact-tgcalls",
            &state_root,
        );
        let mut missing_artifact = configuration.clone();
        missing_artifact.runtime_artifacts.pop();
        assert!(resolve(&missing_artifact).is_err());
        let mut wrong_order = configuration.clone();
        wrong_order.runtime_artifacts.swap(0, 1);
        assert!(resolve(&wrong_order).is_err());
        fs::set_permissions(&tgcalls, fs::Permissions::from_mode(0o700))
            .expect("make test artifact writable");
        fs::write(&tgcalls, b"tampered-tgcalls").expect("tamper staged artifact");

        assert!(resolve(&configuration).is_err());

        fs::write(&tgcalls, b"exact-tgcalls").expect("restore staged artifact");
        fs::set_permissions(&tgcalls, fs::Permissions::from_mode(0o500))
            .expect("protect staged artifact");
        let outside = directory.join("outside");
        fs::create_dir(&outside).expect("create outside state");
        symlink(&outside, state_root.join("tdlib-v1")).expect("symlink state child");
        assert!(resolve(&configuration).is_err());
        let _ = fs::remove_dir_all(directory);
    }

    fn configuration(
        tdjson: &Path,
        tdjson_bytes: &[u8],
        tgcalls: &Path,
        tgcalls_bytes: &[u8],
        state_root: &Path,
    ) -> ManagedIntegrationRuntimeConfigurationV1 {
        ManagedIntegrationRuntimeConfigurationV1 {
            runtime_artifacts: vec![
                ManagedRuntimeArtifactBindingV1 {
                    artifact_id: TELEGRAM_TDJSON_ARTIFACT_ID.to_owned(),
                    r#use: RuntimeArtifactUseV1::NativeDynamicLibrary as i32,
                    staged_path: tdjson.display().to_string(),
                    size_bytes: tdjson_bytes.len() as u64,
                    sha256: Sha256::digest(tdjson_bytes).to_vec(),
                },
                ManagedRuntimeArtifactBindingV1 {
                    artifact_id: TELEGRAM_TGCALLS_ARTIFACT_ID.to_owned(),
                    r#use: RuntimeArtifactUseV1::NativeDynamicLibrary as i32,
                    staged_path: tgcalls.display().to_string(),
                    size_bytes: tgcalls_bytes.len() as u64,
                    sha256: Sha256::digest(tgcalls_bytes).to_vec(),
                },
            ],
            integration_state_root: Some(IntegrationStateRootV1 {
                root_path: state_root.display().to_string(),
                state_generation: 1,
                state_layout_revision: 1,
            }),
            ..Default::default()
        }
    }

    fn write_artifact(directory: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        fs::create_dir_all(directory).expect("create test directory");
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
            .expect("protect test directory");
        let path = directory.join(name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o500)
            .open(&path)
            .expect("create staged artifact");
        file.write_all(bytes).expect("write staged artifact");
        path
    }

    fn test_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "makosh-telegram-runtime-bindings-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
