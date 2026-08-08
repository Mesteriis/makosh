//! Unsigned Ollama AI integration release assembly.

#![forbid(unsafe_code)]

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use makosh_ollama_ai_api::ollama_ai_settings_schema_v1;
use makosh_ollama_ai_persistence::schema::ollama_ai_storage_bundle_v1;
use makosh_ollama_ai_runtime::ollama_ai_module_descriptor_v1;
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const OLLAMA_AI_ASSEMBLY_FRAGMENT_VERSION_V1: u32 = 1;
pub const OLLAMA_AI_RUNTIME_ARTIFACT_ID_V1: &str = "ollama_ai.runtime.v1";
pub const OLLAMA_AI_STORAGE_ARTIFACT_ID_V1: &str = "ollama_ai.storage.v1";
pub const OLLAMA_AI_DESCRIPTOR_FILE_V1: &str = "ollama-ai.runtime.descriptor.pb";
pub const OLLAMA_AI_SETTINGS_FILE_V1: &str = "ollama-ai.runtime.settings.pb";
pub const OLLAMA_AI_STORAGE_BUNDLE_FILE_V1: &str = "ollama-ai.storage.bundle.pb";
pub const OLLAMA_AI_ARTIFACT_FRAGMENT_FILE_V1: &str = "ollama-ai.release-artifacts.json";

const RUNTIME_RELATIVE_PATH_V1: &str = "bin/makosh-ollama-ai-runtime";
const DESCRIPTOR_RELATIVE_PATH_V1: &str = "contracts/ollama-ai.runtime.descriptor.pb";
const SETTINGS_RELATIVE_PATH_V1: &str = "contracts/ollama-ai.runtime.settings.pb";
const STORAGE_RELATIVE_PATH_V1: &str = "storage/ollama-ai.storage.bundle.pb";

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
#[serde(untagged)]
pub enum OllamaAiReleaseArtifactInputV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl OllamaAiReleaseArtifactInputV1 {
    fn artifact_id(&self) -> &str {
        match self {
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OllamaAiReleaseArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<OllamaAiReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaAiReleaseAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OllamaAiReleaseAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

pub fn materialize_ollama_ai_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<OllamaAiReleaseAssemblyPathsV1, OllamaAiReleaseAssemblyErrorV1> {
    validate_inputs_v1(output_directory, build_id, runtime_source)?;
    let descriptor = ollama_ai_module_descriptor_v1(build_id);
    let settings = ollama_ai_settings_schema_v1();
    let storage = ollama_ai_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(OllamaAiReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    let paths = OllamaAiReleaseAssemblyPathsV1 {
        descriptor: output_directory.join(OLLAMA_AI_DESCRIPTOR_FILE_V1),
        settings_schema: output_directory.join(OLLAMA_AI_SETTINGS_FILE_V1),
        storage_bundle: output_directory.join(OLLAMA_AI_STORAGE_BUNDLE_FILE_V1),
        artifact_fragment: output_directory.join(OLLAMA_AI_ARTIFACT_FRAGMENT_FILE_V1),
    };
    let fragment = artifact_fragment_v1(
        &descriptor.owner_id,
        &descriptor.module_id,
        runtime_source,
        &paths,
    )?;
    let fragment_bytes = serde_json::to_vec_pretty(&fragment)
        .map_err(|_| OllamaAiReleaseAssemblyErrorV1::FragmentEncodingFailed)?;
    let mut directory = DirBuilder::new();
    directory.mode(0o700);
    directory
        .create(output_directory)
        .map_err(|_| OllamaAiReleaseAssemblyErrorV1::OutputUnavailable)?;
    let writes = [
        (&paths.descriptor, descriptor.encode_to_vec()),
        (&paths.settings_schema, settings.encode_to_vec()),
        (&paths.storage_bundle, storage.encode_to_vec()),
        (&paths.artifact_fragment, fragment_bytes),
    ];
    for (path, bytes) in writes {
        if write_new_private_file_v1(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(OllamaAiReleaseAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<(), OllamaAiReleaseAssemblyErrorV1> {
    let valid_runtime = fs::symlink_metadata(runtime_source)
        .ok()
        .is_some_and(|metadata| {
            metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0
        });
    if !output_directory.is_absolute()
        || output_directory.parent().is_none()
        || output_directory.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
        || !valid_runtime
    {
        return Err(OllamaAiReleaseAssemblyErrorV1::InvalidInput);
    }
    Ok(())
}

fn artifact_fragment_v1(
    owner_id: &str,
    module_id: &str,
    runtime_source: &Path,
    paths: &OllamaAiReleaseAssemblyPathsV1,
) -> Result<OllamaAiReleaseArtifactFragmentV1, OllamaAiReleaseAssemblyErrorV1> {
    let artifacts = vec![
        OllamaAiReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime".to_owned(),
            artifact_id: OLLAMA_AI_RUNTIME_ARTIFACT_ID_V1.to_owned(),
            relative_path: RUNTIME_RELATIVE_PATH_V1.to_owned(),
            source_path: utf8_path_v1(runtime_source)?,
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: DESCRIPTOR_RELATIVE_PATH_V1.to_owned(),
                source_path: utf8_path_v1(&paths.descriptor)?,
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: SETTINGS_RELATIVE_PATH_V1.to_owned(),
                source_path: utf8_path_v1(&paths.settings_schema)?,
            },
        }),
        OllamaAiReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_kind: "storage_bundle".to_owned(),
            artifact_id: OLLAMA_AI_STORAGE_ARTIFACT_ID_V1.to_owned(),
            relative_path: STORAGE_RELATIVE_PATH_V1.to_owned(),
            source_path: utf8_path_v1(&paths.storage_bundle)?,
            required: true,
        }),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(OllamaAiReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    Ok(OllamaAiReleaseArtifactFragmentV1 {
        version: OLLAMA_AI_ASSEMBLY_FRAGMENT_VERSION_V1,
        owner_id: owner_id.to_owned(),
        module_id: module_id.to_owned(),
        artifacts,
    })
}

fn utf8_path_v1(path: &Path) -> Result<String, OllamaAiReleaseAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(OllamaAiReleaseAssemblyErrorV1::InvalidInput)
}

fn write_new_private_file_v1(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use makosh_runtime_protocol::validation::descriptor::decode_descriptor_v1;

    use super::*;

    static NEXT_TEMPORARY_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn materializes_exact_unsigned_ollama_artifacts() {
        let root = temporary_directory();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        let paths =
            materialize_ollama_ai_release_assembly_v1(&root.join("assembly"), "build-1", &runtime)
                .expect("assembly");
        let descriptor = decode_descriptor_v1(&fs::read(paths.descriptor).expect("descriptor"))
            .expect("descriptor");
        let fragment: OllamaAiReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).expect("fragment"))
                .expect("fragment");
        assert_eq!(descriptor.owner_id, "ollama");
        assert_eq!(fragment.module_id, "makosh-ollama-ai-runtime");
        assert_eq!(fragment.artifacts.len(), 2);
        fs::remove_dir_all(root).expect("cleanup");
    }

    fn temporary_directory() -> PathBuf {
        let id = NEXT_TEMPORARY_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "makosh-ollama-ai-assembly-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("temporary root");
        path
    }
}
