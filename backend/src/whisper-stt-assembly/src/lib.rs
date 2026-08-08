#![forbid(unsafe_code)]
//! Unsigned release materialization for the Whisper provider integration.

use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use makosh_whisper_stt_persistence::schema::whisper_stt_storage_bundle_v1;
use makosh_whisper_stt_runtime::{
    WHISPER_STT_MODEL_ARTIFACT_ID_V1, WHISPER_STT_MODULE_ID_V1, WHISPER_STT_OWNER_ID_V1,
    WHISPER_STT_RUNNER_ARTIFACT_ID_V1, whisper_stt_module_descriptor_v1,
    whisper_stt_settings_schema_v1,
};
use prost::Message;
use serde::{Deserialize, Serialize};

pub const PACKAGE: &str = "makosh-whisper-stt-assembly";
pub const WHISPER_STT_RUNTIME_ARTIFACT_ID_V1: &str = "whisper_stt.runtime.v1";
pub const WHISPER_STT_STORAGE_ARTIFACT_ID_V1: &str = "whisper_stt.storage.v1";
pub const WHISPER_STT_DESCRIPTOR_FILE_V1: &str = "whisper_stt.runtime.descriptor.pb";
pub const WHISPER_STT_SETTINGS_FILE_V1: &str = "whisper_stt.runtime.settings.pb";
pub const WHISPER_STT_STORAGE_FILE_V1: &str = "whisper_stt.storage.bundle.pb";
pub const WHISPER_STT_FRAGMENT_FILE_V1: &str = "whisper-stt.release-artifacts.json";

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseContractInputV1 {
    pub relative_path: String,
    pub source_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleRuntimeArtifactInputV1 {
    pub artifact_kind: String,
    pub artifact_id: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
    pub descriptor: ReleaseContractInputV1,
    pub settings_schema: ReleaseContractInputV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StorageBundleArtifactInputV1 {
    pub artifact_kind: String,
    pub artifact_id: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BoundRuntimeArtifactInputV1 {
    pub artifact_kind: String,
    pub artifact_id: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
    pub bound_module_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum WhisperSttReleaseArtifactInputV1 {
    BoundRuntimeArtifact(BoundRuntimeArtifactInputV1),
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl WhisperSttReleaseArtifactInputV1 {
    fn artifact_id(&self) -> &str {
        match self {
            Self::BoundRuntimeArtifact(value) => &value.artifact_id,
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WhisperSttReleaseArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<WhisperSttReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttReleaseAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhisperSttReleaseAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

pub fn materialize_whisper_stt_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
    runner_source: &Path,
    model_source: &Path,
) -> Result<WhisperSttReleaseAssemblyPathsV1, WhisperSttReleaseAssemblyErrorV1> {
    validate_inputs(
        output_directory,
        build_id,
        runtime_source,
        runner_source,
        model_source,
    )?;
    let descriptor = whisper_stt_module_descriptor_v1(build_id);
    let settings = whisper_stt_settings_schema_v1();
    let storage = whisper_stt_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(WhisperSttReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    let paths = WhisperSttReleaseAssemblyPathsV1 {
        descriptor: output_directory.join(WHISPER_STT_DESCRIPTOR_FILE_V1),
        settings_schema: output_directory.join(WHISPER_STT_SETTINGS_FILE_V1),
        storage_bundle: output_directory.join(WHISPER_STT_STORAGE_FILE_V1),
        artifact_fragment: output_directory.join(WHISPER_STT_FRAGMENT_FILE_V1),
    };
    let fragment = artifact_fragment(runtime_source, runner_source, model_source, &paths)?;
    let writes = [
        (paths.descriptor.as_path(), descriptor.encode_to_vec()),
        (paths.settings_schema.as_path(), settings.encode_to_vec()),
        (paths.storage_bundle.as_path(), storage.encode_to_vec()),
        (
            paths.artifact_fragment.as_path(),
            serde_json::to_vec_pretty(&fragment)
                .map_err(|_| WhisperSttReleaseAssemblyErrorV1::FragmentEncodingFailed)?,
        ),
    ];
    let mut directory = DirBuilder::new();
    directory
        .mode(0o700)
        .create(output_directory)
        .map_err(|_| WhisperSttReleaseAssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in writes {
        if write_new_private_file(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(WhisperSttReleaseAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
    runner_source: &Path,
    model_source: &Path,
) -> Result<(), WhisperSttReleaseAssemblyErrorV1> {
    if !output_directory.is_absolute()
        || output_directory.parent().is_none()
        || output_directory.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !regular_non_symlink_file(runtime_source)
        || !regular_non_symlink_file(runner_source)
        || !regular_non_symlink_file(model_source)
    {
        return Err(WhisperSttReleaseAssemblyErrorV1::InvalidInput);
    }
    Ok(())
}

fn regular_non_symlink_file(path: &Path) -> bool {
    path.is_absolute()
        && fs::symlink_metadata(path).ok().is_some_and(|metadata| {
            metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0
        })
}

fn artifact_fragment(
    runtime_source: &Path,
    runner_source: &Path,
    model_source: &Path,
    paths: &WhisperSttReleaseAssemblyPathsV1,
) -> Result<WhisperSttReleaseArtifactFragmentV1, WhisperSttReleaseAssemblyErrorV1> {
    let artifacts = vec![
        WhisperSttReleaseArtifactInputV1::BoundRuntimeArtifact(BoundRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime_read_only_data".to_owned(),
            artifact_id: WHISPER_STT_MODEL_ARTIFACT_ID_V1.to_owned(),
            relative_path: "runtime-resources/whisper-stt/model.bin".to_owned(),
            source_path: utf8_path(model_source)?,
            required: true,
            bound_module_id: WHISPER_STT_MODULE_ID_V1.to_owned(),
        }),
        WhisperSttReleaseArtifactInputV1::BoundRuntimeArtifact(BoundRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime_native_executable".to_owned(),
            artifact_id: WHISPER_STT_RUNNER_ARTIFACT_ID_V1.to_owned(),
            relative_path: "runtime-resources/whisper-stt/whisper-cli".to_owned(),
            source_path: utf8_path(runner_source)?,
            required: true,
            bound_module_id: WHISPER_STT_MODULE_ID_V1.to_owned(),
        }),
        WhisperSttReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime".to_owned(),
            artifact_id: WHISPER_STT_RUNTIME_ARTIFACT_ID_V1.to_owned(),
            relative_path: "bin/makosh-whisper-stt-runtime".to_owned(),
            source_path: utf8_path(runtime_source)?,
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: "contracts/whisper_stt.runtime.descriptor.pb".to_owned(),
                source_path: utf8_path(&paths.descriptor)?,
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: "contracts/whisper_stt.runtime.settings.pb".to_owned(),
                source_path: utf8_path(&paths.settings_schema)?,
            },
        }),
        WhisperSttReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_kind: "storage_bundle".to_owned(),
            artifact_id: WHISPER_STT_STORAGE_ARTIFACT_ID_V1.to_owned(),
            relative_path: "storage/whisper_stt.storage.bundle.pb".to_owned(),
            source_path: utf8_path(&paths.storage_bundle)?,
            required: true,
        }),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(WhisperSttReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    Ok(WhisperSttReleaseArtifactFragmentV1 {
        version: 1,
        owner_id: WHISPER_STT_OWNER_ID_V1.to_owned(),
        module_id: WHISPER_STT_MODULE_ID_V1.to_owned(),
        artifacts,
    })
}

fn utf8_path(path: &Path) -> Result<String, WhisperSttReleaseAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(WhisperSttReleaseAssemblyErrorV1::InvalidInput)
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn emits_unsigned_runtime_storage_runner_and_model_inputs() {
        let root = std::env::temp_dir().join(format!(
            "makosh-whisper-assembly-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("root");
        let runtime = file(&root, "runtime", b"runtime");
        let runner = file(&root, "runner", b"runner");
        let model = file(&root, "model", b"model");
        let output = root.join("output");
        let paths = materialize_whisper_stt_release_assembly_v1(
            &output,
            "test-build",
            &runtime,
            &runner,
            &model,
        )
        .expect("assembly");
        let fragment: WhisperSttReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).expect("fragment"))
                .expect("fragment json");
        assert_eq!(fragment.owner_id, WHISPER_STT_OWNER_ID_V1);
        assert_eq!(fragment.artifacts.len(), 4);
        assert!(
            fragment
                .artifacts
                .windows(2)
                .all(|pair| { pair[0].artifact_id() < pair[1].artifact_id() })
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    fn file(root: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let path = root.join(name);
        fs::write(&path, bytes).expect("file");
        path
    }
}
