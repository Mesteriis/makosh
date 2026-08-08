//! Communication Reply Suggestion workflow release assembly.
//!
//! This build unit emits unsigned deterministic inputs for the generic
//! distribution compiler. It does not execute workflow or provider behavior.

use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

use makosh_communication_reply_suggestion_persistence::communication_reply_suggestion_storage_bundle_v1;
use makosh_communication_reply_suggestion_runtime::{
    communication_reply_suggestion_module_descriptor_v1,
    communication_reply_suggestion_settings_schema_v1,
};
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const REPLY_SUGGESTION_ASSEMBLY_FRAGMENT_VERSION_V1: u32 = 1;
pub const REPLY_SUGGESTION_ASSEMBLY_OWNER_ID: &str = "communication_reply_suggestion";
pub const REPLY_SUGGESTION_ASSEMBLY_MODULE_ID: &str =
    "makosh-communication-reply-suggestion-runtime";
pub const REPLY_SUGGESTION_RUNTIME_ARTIFACT_ID: &str = "communication_reply_suggestion.runtime.v1";
pub const REPLY_SUGGESTION_STORAGE_ARTIFACT_ID: &str = "communication_reply_suggestion.storage.v1";
pub const REPLY_SUGGESTION_DESCRIPTOR_FILE: &str =
    "communication_reply_suggestion.runtime.descriptor.pb";
pub const REPLY_SUGGESTION_SETTINGS_FILE: &str =
    "communication_reply_suggestion.runtime.settings.pb";
pub const REPLY_SUGGESTION_STORAGE_BUNDLE_FILE: &str =
    "communication_reply_suggestion.storage.bundle.pb";
pub const REPLY_SUGGESTION_ARTIFACT_FRAGMENT_FILE: &str =
    "communication_reply_suggestion.release-artifacts.json";

const RUNTIME_RELATIVE_PATH: &str = "bin/makosh-communication-reply-suggestion-runtime";
const DESCRIPTOR_RELATIVE_PATH: &str =
    "contracts/communication_reply_suggestion.runtime.descriptor.pb";
const SETTINGS_RELATIVE_PATH: &str = "contracts/communication_reply_suggestion.runtime.settings.pb";
const STORAGE_RELATIVE_PATH: &str = "storage/communication_reply_suggestion.storage.bundle.pb";

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
pub enum ReplySuggestionReleaseArtifactInputV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl ReplySuggestionReleaseArtifactInputV1 {
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        match self {
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReplySuggestionReleaseArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<ReplySuggestionReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplySuggestionReleaseAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplySuggestionReleaseAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

pub fn materialize_reply_suggestion_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<ReplySuggestionReleaseAssemblyPathsV1, ReplySuggestionReleaseAssemblyErrorV1> {
    validate_inputs(output_directory, build_id, runtime_source)?;
    let descriptor = communication_reply_suggestion_module_descriptor_v1(build_id);
    let settings_schema = communication_reply_suggestion_settings_schema_v1();
    let storage_bundle = communication_reply_suggestion_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings_schema).is_err()
        || validate_storage_bundle(&storage_bundle).is_err()
    {
        return Err(ReplySuggestionReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }

    let paths = ReplySuggestionReleaseAssemblyPathsV1 {
        descriptor: output_directory.join(REPLY_SUGGESTION_DESCRIPTOR_FILE),
        settings_schema: output_directory.join(REPLY_SUGGESTION_SETTINGS_FILE),
        storage_bundle: output_directory.join(REPLY_SUGGESTION_STORAGE_BUNDLE_FILE),
        artifact_fragment: output_directory.join(REPLY_SUGGESTION_ARTIFACT_FRAGMENT_FILE),
    };
    let fragment = artifact_fragment(
        runtime_source,
        &paths.descriptor,
        &paths.settings_schema,
        &paths.storage_bundle,
    )?;
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
        (
            paths.artifact_fragment.as_path(),
            serde_json::to_vec_pretty(&fragment)
                .map_err(|_| ReplySuggestionReleaseAssemblyErrorV1::FragmentEncodingFailed)?,
        ),
    ];

    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(output_directory)
        .map_err(|_| ReplySuggestionReleaseAssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in &writes {
        if write_new_private_file(path, bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(ReplySuggestionReleaseAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<(), ReplySuggestionReleaseAssemblyErrorV1> {
    if !output_directory.is_absolute()
        || output_directory.parent().is_none()
        || output_directory.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
        || !regular_non_symlink_file(runtime_source)
    {
        return Err(ReplySuggestionReleaseAssemblyErrorV1::InvalidInput);
    }
    Ok(())
}

fn artifact_fragment(
    runtime_source: &Path,
    descriptor: &Path,
    settings_schema: &Path,
    storage_bundle: &Path,
) -> Result<ReplySuggestionReleaseArtifactFragmentV1, ReplySuggestionReleaseAssemblyErrorV1> {
    let artifacts = vec![
        ReplySuggestionReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime".to_owned(),
            artifact_id: REPLY_SUGGESTION_RUNTIME_ARTIFACT_ID.to_owned(),
            relative_path: RUNTIME_RELATIVE_PATH.to_owned(),
            source_path: utf8_path(runtime_source)?,
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: DESCRIPTOR_RELATIVE_PATH.to_owned(),
                source_path: utf8_path(descriptor)?,
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: SETTINGS_RELATIVE_PATH.to_owned(),
                source_path: utf8_path(settings_schema)?,
            },
        }),
        ReplySuggestionReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_kind: "storage_bundle".to_owned(),
            artifact_id: REPLY_SUGGESTION_STORAGE_ARTIFACT_ID.to_owned(),
            relative_path: STORAGE_RELATIVE_PATH.to_owned(),
            source_path: utf8_path(storage_bundle)?,
            required: true,
        }),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(ReplySuggestionReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    Ok(ReplySuggestionReleaseArtifactFragmentV1 {
        version: REPLY_SUGGESTION_ASSEMBLY_FRAGMENT_VERSION_V1,
        owner_id: REPLY_SUGGESTION_ASSEMBLY_OWNER_ID.to_owned(),
        module_id: REPLY_SUGGESTION_ASSEMBLY_MODULE_ID.to_owned(),
        artifacts,
    })
}

fn regular_non_symlink_file(path: &Path) -> bool {
    fs::symlink_metadata(path).ok().is_some_and(|metadata| {
        metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0
    })
}

fn utf8_path(path: &Path) -> Result<String, ReplySuggestionReleaseAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(ReplySuggestionReleaseAssemblyErrorV1::InvalidInput)
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

    use makosh_runtime_protocol::v1::ModuleDescriptorV1;
    use makosh_storage_protocol::v1::StorageBundleV1;

    use super::*;

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn materializes_exact_unsigned_runtime_and_storage_fragment() {
        let root = temporary_directory();
        let runtime = root.join("makosh-communication-reply-suggestion-runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        let paths = materialize_reply_suggestion_release_assembly_v1(
            &root.join("assembly"),
            "build-1",
            &runtime,
        )
        .expect("assembly");
        let descriptor =
            ModuleDescriptorV1::decode(fs::read(paths.descriptor).expect("descriptor").as_slice())
                .expect("descriptor");
        let storage =
            StorageBundleV1::decode(fs::read(paths.storage_bundle).expect("storage").as_slice())
                .expect("storage");
        let fragment: ReplySuggestionReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).expect("fragment"))
                .expect("fragment");
        assert_eq!(descriptor.module_id, REPLY_SUGGESTION_ASSEMBLY_MODULE_ID);
        assert_eq!(storage.owner_id, REPLY_SUGGESTION_ASSEMBLY_OWNER_ID);
        assert_eq!(
            fragment
                .artifacts
                .iter()
                .map(ReplySuggestionReleaseArtifactInputV1::artifact_id)
                .collect::<Vec<_>>(),
            [
                REPLY_SUGGESTION_RUNTIME_ARTIFACT_ID,
                REPLY_SUGGESTION_STORAGE_ARTIFACT_ID,
            ]
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn rejects_missing_runtime_and_existing_output() {
        let root = temporary_directory();
        let output = root.join("assembly");
        assert_eq!(
            materialize_reply_suggestion_release_assembly_v1(
                &output,
                "build-1",
                &root.join("missing"),
            ),
            Err(ReplySuggestionReleaseAssemblyErrorV1::InvalidInput)
        );
        fs::create_dir(&output).expect("existing output");
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        assert_eq!(
            materialize_reply_suggestion_release_assembly_v1(&output, "build-1", &runtime),
            Err(ReplySuggestionReleaseAssemblyErrorV1::InvalidInput)
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    fn temporary_directory() -> PathBuf {
        let root = std::env::current_dir()
            .expect("cwd")
            .join("target")
            .join(format!(
                "communication-reply-suggestion-assembly-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("fixture");
        root
    }
}
