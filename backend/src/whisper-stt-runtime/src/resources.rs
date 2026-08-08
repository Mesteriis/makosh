use std::{
    fs::{self, DirBuilder, File},
    io::Read,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

use makosh_runtime_protocol::v1::{ManagedRuntimeArtifactBindingV1, RuntimeArtifactUseV1};
use makosh_whisper_stt_process::WhisperSttProcessConfigurationV1;
use sha2::{Digest, Sha256};

use crate::admission::{WHISPER_STT_MODEL_ARTIFACT_ID_V1, WHISPER_STT_RUNNER_ARTIFACT_ID_V1};

const WORK_ROOT_V1: &str = "whisper-work-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhisperSttResourcesErrorV1 {
    InvalidBindings,
    ArtifactUnavailable,
    WorkRootUnavailable,
}

pub struct PreparedWhisperSttResourcesV1 {
    configuration: WhisperSttProcessConfigurationV1,
    model_sha256: [u8; 32],
    work_root: PathBuf,
}

impl PreparedWhisperSttResourcesV1 {
    #[must_use]
    pub fn configuration(&self) -> &WhisperSttProcessConfigurationV1 {
        &self.configuration
    }

    #[must_use]
    pub fn model_sha256(&self) -> [u8; 32] {
        self.model_sha256
    }
}

impl Drop for PreparedWhisperSttResourcesV1 {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.work_root);
    }
}

pub fn prepare_whisper_stt_resources_v1(
    bindings: &[ManagedRuntimeArtifactBindingV1],
) -> Result<PreparedWhisperSttResourcesV1, WhisperSttResourcesErrorV1> {
    let [model, runner] = bindings else {
        return Err(WhisperSttResourcesErrorV1::InvalidBindings);
    };
    validate_binding(
        model,
        WHISPER_STT_MODEL_ARTIFACT_ID_V1,
        RuntimeArtifactUseV1::ReadOnlyData,
    )?;
    validate_binding(
        runner,
        WHISPER_STT_RUNNER_ARTIFACT_ID_V1,
        RuntimeArtifactUseV1::NativeExecutable,
    )?;
    let model_path = Path::new(&model.staged_path);
    let runner_path = Path::new(&runner.staged_path);
    let artifact_root = common_parent(model_path, runner_path)?;
    verify_staged_file(model_path, model, 0o400)?;
    verify_staged_file(runner_path, runner, 0o500)?;
    let work_root = artifact_root.join(WORK_ROOT_V1);
    let mut builder = DirBuilder::new();
    builder
        .mode(0o700)
        .create(&work_root)
        .map_err(|_| WhisperSttResourcesErrorV1::WorkRootUnavailable)?;
    Ok(PreparedWhisperSttResourcesV1 {
        configuration: WhisperSttProcessConfigurationV1 {
            executable: runner_path.to_owned(),
            model: model_path.to_owned(),
            private_work_root: work_root.clone(),
        },
        model_sha256: digest_array(model)?,
        work_root,
    })
}

fn validate_binding(
    binding: &ManagedRuntimeArtifactBindingV1,
    artifact_id: &str,
    use_kind: RuntimeArtifactUseV1,
) -> Result<(), WhisperSttResourcesErrorV1> {
    if binding.artifact_id != artifact_id
        || binding.r#use != use_kind as i32
        || binding.size_bytes == 0
        || binding.sha256.len() != 32
        || !binding.sha256.iter().any(|byte| *byte != 0)
        || !Path::new(&binding.staged_path).is_absolute()
    {
        return Err(WhisperSttResourcesErrorV1::InvalidBindings);
    }
    Ok(())
}

fn common_parent(left: &Path, right: &Path) -> Result<PathBuf, WhisperSttResourcesErrorV1> {
    let left_parent = left
        .parent()
        .ok_or(WhisperSttResourcesErrorV1::InvalidBindings)?
        .canonicalize()
        .map_err(|_| WhisperSttResourcesErrorV1::ArtifactUnavailable)?;
    let right_parent = right
        .parent()
        .ok_or(WhisperSttResourcesErrorV1::InvalidBindings)?
        .canonicalize()
        .map_err(|_| WhisperSttResourcesErrorV1::ArtifactUnavailable)?;
    if left_parent != right_parent {
        return Err(WhisperSttResourcesErrorV1::InvalidBindings);
    }
    Ok(left_parent)
}

fn verify_staged_file(
    path: &Path,
    binding: &ManagedRuntimeArtifactBindingV1,
    expected_mode: u32,
) -> Result<(), WhisperSttResourcesErrorV1> {
    let path_before =
        fs::symlink_metadata(path).map_err(|_| WhisperSttResourcesErrorV1::ArtifactUnavailable)?;
    if path_before.file_type().is_symlink()
        || !path_before.is_file()
        || path_before.len() != binding.size_bytes
        || path_before.permissions().mode() & 0o777 != expected_mode
    {
        return Err(WhisperSttResourcesErrorV1::ArtifactUnavailable);
    }
    let mut file = File::open(path).map_err(|_| WhisperSttResourcesErrorV1::ArtifactUnavailable)?;
    let opened = file
        .metadata()
        .map_err(|_| WhisperSttResourcesErrorV1::ArtifactUnavailable)?;
    if !same_file(&path_before, &opened) {
        return Err(WhisperSttResourcesErrorV1::ArtifactUnavailable);
    }
    let observed = digest_reader(&mut file)?;
    let opened_after = file
        .metadata()
        .map_err(|_| WhisperSttResourcesErrorV1::ArtifactUnavailable)?;
    let path_after =
        fs::symlink_metadata(path).map_err(|_| WhisperSttResourcesErrorV1::ArtifactUnavailable)?;
    if !same_file(&opened, &opened_after)
        || !same_file(&opened, &path_after)
        || observed.as_slice() != binding.sha256.as_slice()
    {
        return Err(WhisperSttResourcesErrorV1::ArtifactUnavailable);
    }
    Ok(())
}

fn digest_reader(file: &mut File) -> Result<[u8; 32], WhisperSttResourcesErrorV1> {
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| WhisperSttResourcesErrorV1::ArtifactUnavailable)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest.finalize().into())
}

fn digest_array(
    binding: &ManagedRuntimeArtifactBindingV1,
) -> Result<[u8; 32], WhisperSttResourcesErrorV1> {
    binding
        .sha256
        .as_slice()
        .try_into()
        .map_err(|_| WhisperSttResourcesErrorV1::InvalidBindings)
}

fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}

#[cfg(test)]
mod tests {
    use std::{
        os::unix::fs::symlink,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn accepts_only_exact_digest_mode_inode_and_artifact_use() {
        let root = std::env::temp_dir().join(format!(
            "makosh-whisper-resources-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("root");
        let bindings = [
            staged(
                &root,
                WHISPER_STT_MODEL_ARTIFACT_ID_V1,
                b"model",
                RuntimeArtifactUseV1::ReadOnlyData,
                0o400,
            ),
            staged(
                &root,
                WHISPER_STT_RUNNER_ARTIFACT_ID_V1,
                b"runner",
                RuntimeArtifactUseV1::NativeExecutable,
                0o500,
            ),
        ];
        let prepared = prepare_whisper_stt_resources_v1(&bindings).expect("resources");
        assert_eq!(prepared.model_sha256(), Sha256::digest(b"model").as_slice());
        let work = prepared.work_root.clone();
        drop(prepared);
        assert!(!work.exists());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn rejects_digest_drift_and_symlink_substitution() {
        let root = std::env::temp_dir().join(format!(
            "makosh-whisper-resources-negative-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("root");
        let mut bindings = [
            staged(
                &root,
                WHISPER_STT_MODEL_ARTIFACT_ID_V1,
                b"model",
                RuntimeArtifactUseV1::ReadOnlyData,
                0o400,
            ),
            staged(
                &root,
                WHISPER_STT_RUNNER_ARTIFACT_ID_V1,
                b"runner",
                RuntimeArtifactUseV1::NativeExecutable,
                0o500,
            ),
        ];
        bindings[0].sha256 = vec![9; 32];
        assert_eq!(
            prepare_whisper_stt_resources_v1(&bindings).err(),
            Some(WhisperSttResourcesErrorV1::ArtifactUnavailable)
        );
        bindings[0].sha256 = Sha256::digest(b"model").to_vec();
        let linked = root.join("linked-model");
        symlink(&bindings[0].staged_path, &linked).expect("symlink");
        bindings[0].staged_path = linked.display().to_string();
        assert!(prepare_whisper_stt_resources_v1(&bindings).is_err());
        fs::remove_dir_all(root).expect("cleanup");
    }

    fn staged(
        root: &Path,
        artifact_id: &str,
        bytes: &[u8],
        use_kind: RuntimeArtifactUseV1,
        mode: u32,
    ) -> ManagedRuntimeArtifactBindingV1 {
        let path = root.join(artifact_id);
        fs::write(&path, bytes).expect("bytes");
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("mode");
        ManagedRuntimeArtifactBindingV1 {
            artifact_id: artifact_id.to_owned(),
            r#use: use_kind as i32,
            staged_path: path.display().to_string(),
            size_bytes: bytes.len() as u64,
            sha256: Sha256::digest(bytes).to_vec(),
        }
    }
}
