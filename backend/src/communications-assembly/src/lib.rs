//! Communications-owned release assembly artifact materialization.
//!
//! This package emits unsigned inputs for the generic distribution compiler.
//! It never receives release signing authority and is not a managed runtime.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use makosh_communications_runtime::admission::{
    communications_module_descriptor_v1, communications_settings_schema_v1,
};
use makosh_communications_runtime::storage_bundle::communications_runtime_storage_bundle_v1;
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const COMMUNICATIONS_ASSEMBLY_FRAGMENT_VERSION_V1: u32 = 1;
pub const COMMUNICATIONS_ASSEMBLY_OWNER_ID: &str = "communications";
pub const COMMUNICATIONS_ASSEMBLY_MODULE_ID: &str = "makosh-communications-runtime";
pub const COMMUNICATIONS_RUNTIME_ARTIFACT_ID: &str = "communications.runtime.v1";
pub const COMMUNICATIONS_STORAGE_ARTIFACT_ID: &str = "communications.storage.v1";
pub const COMMUNICATIONS_DESCRIPTOR_FILE: &str = "communications.runtime.descriptor.pb";
pub const COMMUNICATIONS_SETTINGS_FILE: &str = "communications.runtime.settings.pb";
pub const COMMUNICATIONS_STORAGE_BUNDLE_FILE: &str = "communications.storage.bundle.pb";
pub const COMMUNICATIONS_ARTIFACT_FRAGMENT_FILE: &str = "communications.release-artifacts.json";

const COMMUNICATIONS_RUNTIME_RELATIVE_PATH: &str = "bin/makosh-communications-runtime";
const COMMUNICATIONS_DESCRIPTOR_RELATIVE_PATH: &str =
    "contracts/communications.runtime.descriptor.pb";
const COMMUNICATIONS_SETTINGS_RELATIVE_PATH: &str = "contracts/communications.runtime.settings.pb";
const COMMUNICATIONS_STORAGE_RELATIVE_PATH: &str = "storage/communications.storage.bundle.pb";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseContractInputV1 {
    pub relative_path: String,
    pub source_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageBundleArtifactInputV1 {
    pub artifact_kind: String,
    pub artifact_id: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommunicationsReleaseArtifactInputV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl CommunicationsReleaseArtifactInputV1 {
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        match self {
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommunicationsReleaseArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<CommunicationsReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommunicationsReleaseAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommunicationsReleaseAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

pub fn materialize_communications_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<CommunicationsReleaseAssemblyPathsV1, CommunicationsReleaseAssemblyErrorV1> {
    validate_inputs(output_directory, build_id, runtime_source)?;
    let descriptor = communications_module_descriptor_v1(build_id);
    let settings_schema = communications_settings_schema_v1();
    let storage_bundle = communications_runtime_storage_bundle_v1()
        .map_err(|_| CommunicationsReleaseAssemblyErrorV1::InvalidCanonicalArtifact)?;
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings_schema).is_err()
        || validate_storage_bundle(&storage_bundle).is_err()
    {
        return Err(CommunicationsReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }

    let paths = CommunicationsReleaseAssemblyPathsV1 {
        descriptor: output_directory.join(COMMUNICATIONS_DESCRIPTOR_FILE),
        settings_schema: output_directory.join(COMMUNICATIONS_SETTINGS_FILE),
        storage_bundle: output_directory.join(COMMUNICATIONS_STORAGE_BUNDLE_FILE),
        artifact_fragment: output_directory.join(COMMUNICATIONS_ARTIFACT_FRAGMENT_FILE),
    };
    let fragment = artifact_fragment(
        runtime_source,
        &paths.descriptor,
        &paths.settings_schema,
        &paths.storage_bundle,
    )?;
    let fragment_bytes = serde_json::to_vec_pretty(&fragment)
        .map_err(|_| CommunicationsReleaseAssemblyErrorV1::FragmentEncodingFailed)?;
    let writes = [
        (paths.descriptor.as_path(), descriptor.encode_to_vec()),
        (
            paths.settings_schema.as_path(),
            settings_schema.encode_to_vec(),
        ),
        (
            paths.storage_bundle.as_path(),
            storage_bundle.encode_to_vec(),
        ),
        (paths.artifact_fragment.as_path(), fragment_bytes),
    ];

    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(output_directory)
        .map_err(|_| CommunicationsReleaseAssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in &writes {
        if write_new_private_file(path, bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(CommunicationsReleaseAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<(), CommunicationsReleaseAssemblyErrorV1> {
    if !output_directory.is_absolute()
        || output_directory.parent().is_none()
        || output_directory.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
        || !regular_non_symlink_file(runtime_source)
    {
        return Err(CommunicationsReleaseAssemblyErrorV1::InvalidInput);
    }
    Ok(())
}

fn regular_non_symlink_file(path: &Path) -> bool {
    fs::symlink_metadata(path).ok().is_some_and(|metadata| {
        metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0
    })
}

fn artifact_fragment(
    runtime_source: &Path,
    descriptor: &Path,
    settings_schema: &Path,
    storage_bundle: &Path,
) -> Result<CommunicationsReleaseArtifactFragmentV1, CommunicationsReleaseAssemblyErrorV1> {
    let artifacts = vec![
        CommunicationsReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime".to_owned(),
            artifact_id: COMMUNICATIONS_RUNTIME_ARTIFACT_ID.to_owned(),
            relative_path: COMMUNICATIONS_RUNTIME_RELATIVE_PATH.to_owned(),
            source_path: utf8_path(runtime_source)?,
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: COMMUNICATIONS_DESCRIPTOR_RELATIVE_PATH.to_owned(),
                source_path: utf8_path(descriptor)?,
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: COMMUNICATIONS_SETTINGS_RELATIVE_PATH.to_owned(),
                source_path: utf8_path(settings_schema)?,
            },
        }),
        CommunicationsReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_kind: "storage_bundle".to_owned(),
            artifact_id: COMMUNICATIONS_STORAGE_ARTIFACT_ID.to_owned(),
            relative_path: COMMUNICATIONS_STORAGE_RELATIVE_PATH.to_owned(),
            source_path: utf8_path(storage_bundle)?,
            required: true,
        }),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(CommunicationsReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    Ok(CommunicationsReleaseArtifactFragmentV1 {
        version: COMMUNICATIONS_ASSEMBLY_FRAGMENT_VERSION_V1,
        owner_id: COMMUNICATIONS_ASSEMBLY_OWNER_ID.to_owned(),
        module_id: COMMUNICATIONS_ASSEMBLY_MODULE_ID.to_owned(),
        artifacts,
    })
}

fn utf8_path(path: &Path) -> Result<String, CommunicationsReleaseAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(CommunicationsReleaseAssemblyErrorV1::InvalidInput)
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
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn materializes_exact_unsigned_runtime_and_storage_fragment() {
        let root = temporary_directory();
        let runtime = root.join("makosh-communications-runtime");
        fs::write(&runtime, b"runtime").expect("write runtime");
        let output = root.join("assembly");
        let paths = materialize_communications_release_assembly_v1(&output, "build-1", &runtime)
            .expect("materialize assembly");
        let fragment: CommunicationsReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).expect("read fragment"))
                .expect("decode fragment");
        assert_eq!(fragment.owner_id, "communications");
        assert_eq!(fragment.module_id, "makosh-communications-runtime");
        assert_eq!(
            fragment
                .artifacts
                .iter()
                .map(CommunicationsReleaseArtifactInputV1::artifact_id)
                .collect::<Vec<_>>(),
            ["communications.runtime.v1", "communications.storage.v1"]
        );
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn rejects_missing_relative_and_existing_outputs() {
        let root = temporary_directory();
        let missing = root.join("missing");
        assert_eq!(
            materialize_communications_release_assembly_v1(
                &root.join("assembly"),
                "build-1",
                &missing
            ),
            Err(CommunicationsReleaseAssemblyErrorV1::InvalidInput)
        );
        fs::remove_dir_all(root).expect("remove fixture");
    }

    fn temporary_directory() -> PathBuf {
        let root = std::env::current_dir()
            .expect("cwd")
            .join("target")
            .join(format!(
                "communications-assembly-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&root).expect("create fixture");
        root
    }
}
