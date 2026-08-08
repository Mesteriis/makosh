#![forbid(unsafe_code)]
//! Unsigned release materialization for the call transcription workflow.

use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

use makosh_call_transcription_persistence::call_transcription_storage_bundle_v1;
use makosh_call_transcription_runtime::{module_descriptor_v1, settings_schema_bytes_v1};
use makosh_runtime_protocol::validation::descriptor::{
    decode_settings_schema_v1, validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const PACKAGE: &str = "makosh-call-transcription-assembly";
pub const OWNER_ID_V1: &str = "call_transcription";
pub const MODULE_ID_V1: &str = "makosh-call-transcription-runtime";
pub const RUNTIME_ARTIFACT_ID_V1: &str = "call_transcription.runtime.v1";
pub const STORAGE_ARTIFACT_ID_V1: &str = "call_transcription.storage.v1";

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
pub enum ReleaseArtifactInputV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl ReleaseArtifactInputV1 {
    fn artifact_id(&self) -> &str {
        match self {
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<ReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

pub fn materialize_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<ReleaseAssemblyPathsV1, ReleaseAssemblyErrorV1> {
    validate_inputs(output_directory, build_id, runtime_source)?;
    let descriptor = module_descriptor_v1(build_id);
    let settings_bytes = settings_schema_bytes_v1();
    let settings = decode_settings_schema_v1(&settings_bytes)
        .map_err(|_| ReleaseAssemblyErrorV1::InvalidCanonicalArtifact)?;
    let storage = call_transcription_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(ReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    let paths = ReleaseAssemblyPathsV1 {
        descriptor: output_directory.join("call_transcription.runtime.descriptor.pb"),
        settings_schema: output_directory.join("call_transcription.runtime.settings.pb"),
        storage_bundle: output_directory.join("call_transcription.storage.bundle.pb"),
        artifact_fragment: output_directory.join("call-transcription.release-artifacts.json"),
    };
    let fragment = artifact_fragment(runtime_source, &paths)?;
    let fragment_bytes = serde_json::to_vec_pretty(&fragment)
        .map_err(|_| ReleaseAssemblyErrorV1::FragmentEncodingFailed)?;
    let mut directory = DirBuilder::new();
    directory.mode(0o700);
    directory
        .create(output_directory)
        .map_err(|_| ReleaseAssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in [
        (&paths.descriptor, descriptor.encode_to_vec()),
        (&paths.settings_schema, settings_bytes),
        (&paths.storage_bundle, storage.encode_to_vec()),
        (&paths.artifact_fragment, fragment_bytes),
    ] {
        if write_new_private_file(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(ReleaseAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<(), ReleaseAssemblyErrorV1> {
    if !output_directory.is_absolute()
        || output_directory.parent().is_none()
        || output_directory.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
        || !regular_non_symlink_file(runtime_source)
    {
        Err(ReleaseAssemblyErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}

fn artifact_fragment(
    runtime_source: &Path,
    paths: &ReleaseAssemblyPathsV1,
) -> Result<ReleaseArtifactFragmentV1, ReleaseAssemblyErrorV1> {
    let artifacts = vec![
        ReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime".to_owned(),
            artifact_id: RUNTIME_ARTIFACT_ID_V1.to_owned(),
            relative_path: "bin/makosh-call-transcription-runtime".to_owned(),
            source_path: utf8_path(runtime_source)?,
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: "contracts/call_transcription.runtime.descriptor.pb".to_owned(),
                source_path: utf8_path(&paths.descriptor)?,
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: "contracts/call_transcription.runtime.settings.pb".to_owned(),
                source_path: utf8_path(&paths.settings_schema)?,
            },
        }),
        ReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_kind: "storage_bundle".to_owned(),
            artifact_id: STORAGE_ARTIFACT_ID_V1.to_owned(),
            relative_path: "storage/call_transcription.storage.bundle.pb".to_owned(),
            source_path: utf8_path(&paths.storage_bundle)?,
            required: true,
        }),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(ReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    Ok(ReleaseArtifactFragmentV1 {
        version: 1,
        owner_id: OWNER_ID_V1.to_owned(),
        module_id: MODULE_ID_V1.to_owned(),
        artifacts,
    })
}

fn regular_non_symlink_file(path: &Path) -> bool {
    fs::symlink_metadata(path).ok().is_some_and(|metadata| {
        metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0
    })
}

fn utf8_path(path: &Path) -> Result<String, ReleaseAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(ReleaseAssemblyErrorV1::InvalidInput)
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn emits_exact_unsigned_runtime_and_storage_inputs() {
        let root = std::env::temp_dir().join(format!(
            "makosh-call-transcription-assembly-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("root");
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        let paths = materialize_release_assembly_v1(&root.join("assembly"), "test-build", &runtime)
            .expect("assembly");
        let fragment: ReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).expect("fragment"))
                .expect("fragment json");
        assert_eq!(fragment.owner_id, OWNER_ID_V1);
        assert_eq!(fragment.module_id, MODULE_ID_V1);
        assert_eq!(fragment.artifacts.len(), 2);
        fs::remove_dir_all(root).expect("cleanup");
    }
}
