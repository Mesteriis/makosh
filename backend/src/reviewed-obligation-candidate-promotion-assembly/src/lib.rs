//! Unsigned Reviewed Obligation Candidate Promotion workflow release assembly.
//!
//! This build unit materializes deterministic runtime contracts and the
//! workflow-owned Storage bundle. It never executes the workflow or signs a
//! distribution.

#![forbid(unsafe_code)]

use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

use makosh_reviewed_obligation_candidate_promotion_persistence::schema::reviewed_obligation_candidate_promotion_storage_bundle_v1;
use makosh_reviewed_obligation_candidate_promotion_runtime::{
    reviewed_obligation_candidate_promotion_module_descriptor_v1,
    reviewed_obligation_candidate_promotion_settings_schema_v1,
};
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const ASSEMBLY_FRAGMENT_VERSION_V1: u32 = 1;
pub const RUNTIME_ARTIFACT_ID_V1: &str = "reviewed_obligation_candidate_promotion.runtime.v1";
pub const STORAGE_ARTIFACT_ID_V1: &str = "reviewed_obligation_candidate_promotion.storage.v1";
pub const DESCRIPTOR_FILE_V1: &str =
    "reviewed_obligation_candidate_promotion.runtime.descriptor.pb";
pub const SETTINGS_FILE_V1: &str = "reviewed_obligation_candidate_promotion.runtime.settings.pb";
pub const STORAGE_BUNDLE_FILE_V1: &str =
    "reviewed_obligation_candidate_promotion.storage.bundle.pb";
pub const ARTIFACT_FRAGMENT_FILE_V1: &str =
    "reviewed_obligation_candidate_promotion.release-artifacts.json";

const RUNTIME_RELATIVE_PATH_V1: &str = "bin/makosh-reviewed-obligation-candidate-promotion-runtime";
const DESCRIPTOR_RELATIVE_PATH_V1: &str =
    "contracts/reviewed_obligation_candidate_promotion.runtime.descriptor.pb";
const SETTINGS_RELATIVE_PATH_V1: &str =
    "contracts/reviewed_obligation_candidate_promotion.runtime.settings.pb";
const STORAGE_RELATIVE_PATH_V1: &str =
    "storage/reviewed_obligation_candidate_promotion.storage.bundle.pb";

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

pub fn materialize_reviewed_obligation_candidate_promotion_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<ReleaseAssemblyPathsV1, ReleaseAssemblyErrorV1> {
    validate_inputs(output_directory, build_id, runtime_source)?;
    let descriptor = reviewed_obligation_candidate_promotion_module_descriptor_v1(build_id);
    let settings = reviewed_obligation_candidate_promotion_settings_schema_v1();
    let storage = reviewed_obligation_candidate_promotion_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(ReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    let paths = ReleaseAssemblyPathsV1 {
        descriptor: output_directory.join(DESCRIPTOR_FILE_V1),
        settings_schema: output_directory.join(SETTINGS_FILE_V1),
        storage_bundle: output_directory.join(STORAGE_BUNDLE_FILE_V1),
        artifact_fragment: output_directory.join(ARTIFACT_FRAGMENT_FILE_V1),
    };
    let fragment = artifact_fragment(
        &descriptor.owner_id,
        &descriptor.module_id,
        runtime_source,
        &paths,
    )?;
    let fragment_bytes = serde_json::to_vec_pretty(&fragment)
        .map_err(|_| ReleaseAssemblyErrorV1::FragmentEncodingFailed)?;
    let mut directory = DirBuilder::new();
    directory.mode(0o700);
    directory
        .create(output_directory)
        .map_err(|_| ReleaseAssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in [
        (&paths.descriptor, descriptor.encode_to_vec()),
        (&paths.settings_schema, settings.encode_to_vec()),
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
        return Err(ReleaseAssemblyErrorV1::InvalidInput);
    }
    Ok(())
}

fn artifact_fragment(
    owner_id: &str,
    module_id: &str,
    runtime_source: &Path,
    paths: &ReleaseAssemblyPathsV1,
) -> Result<ReleaseArtifactFragmentV1, ReleaseAssemblyErrorV1> {
    let artifacts = vec![
        ReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime".to_owned(),
            artifact_id: RUNTIME_ARTIFACT_ID_V1.to_owned(),
            relative_path: RUNTIME_RELATIVE_PATH_V1.to_owned(),
            source_path: utf8_path(runtime_source)?,
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: DESCRIPTOR_RELATIVE_PATH_V1.to_owned(),
                source_path: utf8_path(&paths.descriptor)?,
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: SETTINGS_RELATIVE_PATH_V1.to_owned(),
                source_path: utf8_path(&paths.settings_schema)?,
            },
        }),
        ReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_kind: "storage_bundle".to_owned(),
            artifact_id: STORAGE_ARTIFACT_ID_V1.to_owned(),
            relative_path: STORAGE_RELATIVE_PATH_V1.to_owned(),
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
        version: ASSEMBLY_FRAGMENT_VERSION_V1,
        owner_id: owner_id.to_owned(),
        module_id: module_id.to_owned(),
        artifacts,
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

    use makosh_runtime_protocol::validation::descriptor::decode_descriptor_v1;
    use makosh_storage_protocol::v1::StorageBundleV1;

    use super::*;

    static NEXT_TEMPORARY_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn materializes_exact_unsigned_workflow_artifacts() {
        let root = temporary_directory();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        let output = root.join("assembly");
        let paths = materialize_reviewed_obligation_candidate_promotion_release_assembly_v1(
            &output, "build-1", &runtime,
        )
        .expect("assembly");
        let descriptor = decode_descriptor_v1(&fs::read(paths.descriptor).expect("descriptor"))
            .expect("canonical descriptor");
        let storage =
            StorageBundleV1::decode(fs::read(paths.storage_bundle).expect("storage").as_slice())
                .expect("canonical storage");
        let fragment: ReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).expect("fragment"))
                .expect("typed fragment");
        assert_eq!(
            descriptor.owner_id,
            "reviewed_obligation_candidate_promotion"
        );
        assert_eq!(
            descriptor.module_kind,
            makosh_runtime_protocol::v1::ModuleKindV1::Workflow as i32
        );
        assert_eq!(storage.owner_id, "reviewed_obligation_candidate_promotion");
        assert_eq!(fragment.owner_id, "reviewed_obligation_candidate_promotion");
        assert_eq!(fragment.artifacts.len(), 2);
        assert!(
            fragment
                .artifacts
                .windows(2)
                .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_missing_and_symlink_runtime_inputs() {
        let root = temporary_directory();
        let output = root.join("missing-output");
        assert_eq!(
            materialize_reviewed_obligation_candidate_promotion_release_assembly_v1(
                &output,
                "build-1",
                &root.join("missing"),
            ),
            Err(ReleaseAssemblyErrorV1::InvalidInput)
        );
        let runtime = root.join("runtime");
        let link = root.join("runtime-link");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        std::os::unix::fs::symlink(&runtime, &link).expect("symlink fixture");
        assert_eq!(
            materialize_reviewed_obligation_candidate_promotion_release_assembly_v1(
                &output, "build-1", &link,
            ),
            Err(ReleaseAssemblyErrorV1::InvalidInput)
        );
        let _ = fs::remove_dir_all(root);
    }

    fn temporary_directory() -> PathBuf {
        let id = NEXT_TEMPORARY_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "makosh-reviewed-obligation-candidate-promotion-assembly-{}-{id}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temporary root");
        path
    }
}
