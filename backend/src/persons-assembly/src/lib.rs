//! Unsigned Persons release assembly. This package never receives signing authority.
#![forbid(unsafe_code)]

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use makosh_persons_persistence::persons_storage_bundle_v1;
use makosh_persons_runtime::{persons_module_descriptor_v1, persons_settings_schema_v1};
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const PERSONS_DESCRIPTOR_FILE_V1: &str = "persons.runtime.descriptor.pb";
pub const PERSONS_SETTINGS_FILE_V1: &str = "persons.runtime.settings.pb";
pub const PERSONS_STORAGE_BUNDLE_FILE_V1: &str = "persons.storage.bundle.pb";
pub const PERSONS_ARTIFACT_FRAGMENT_FILE_V1: &str = "persons.release-artifacts.json";
pub const PERSONS_RUNTIME_ARTIFACT_ID_V1: &str = "persons.runtime.v1";
pub const PERSONS_STORAGE_ARTIFACT_ID_V1: &str = "persons.storage.v1";

const RUNTIME_RELATIVE_PATH_V1: &str = "bin/makosh-persons-runtime";
const DESCRIPTOR_RELATIVE_PATH_V1: &str = "contracts/persons.runtime.descriptor.pb";
const SETTINGS_RELATIVE_PATH_V1: &str = "contracts/persons.runtime.settings.pb";
const STORAGE_RELATIVE_PATH_V1: &str = "storage/persons.storage.bundle.pb";

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseContractInputV1 {
    pub relative_path: String,
    pub source_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleRuntimeArtifactInputV1 {
    pub artifact_id: String,
    pub artifact_kind: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
    pub descriptor: ReleaseContractInputV1,
    pub settings_schema: ReleaseContractInputV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StorageBundleArtifactInputV1 {
    pub artifact_id: String,
    pub artifact_kind: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PersonsReleaseArtifactInputV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl PersonsReleaseArtifactInputV1 {
    fn artifact_id(&self) -> &str {
        match self {
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersonsArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<PersonsReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonsAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

pub fn materialize_persons_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<PersonsAssemblyPathsV1, PersonsAssemblyErrorV1> {
    materialize_persons_release_assembly_inner_v1(output_directory, build_id, runtime_source, None)
}

fn materialize_persons_release_assembly_inner_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
    fail_before_write: Option<usize>,
) -> Result<PersonsAssemblyPathsV1, PersonsAssemblyErrorV1> {
    validate_inputs(output_directory, build_id, runtime_source)?;
    let descriptor = persons_module_descriptor_v1(build_id);
    let settings = persons_settings_schema_v1();
    let storage = persons_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(PersonsAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    let paths = PersonsAssemblyPathsV1 {
        descriptor: output_directory.join(PERSONS_DESCRIPTOR_FILE_V1),
        settings_schema: output_directory.join(PERSONS_SETTINGS_FILE_V1),
        storage_bundle: output_directory.join(PERSONS_STORAGE_BUNDLE_FILE_V1),
        artifact_fragment: output_directory.join(PERSONS_ARTIFACT_FRAGMENT_FILE_V1),
    };
    let artifacts = vec![
        PersonsReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_id: PERSONS_RUNTIME_ARTIFACT_ID_V1.to_owned(),
            artifact_kind: "module_runtime".to_owned(),
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
        PersonsReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_id: PERSONS_STORAGE_ARTIFACT_ID_V1.to_owned(),
            artifact_kind: "storage_bundle".to_owned(),
            relative_path: STORAGE_RELATIVE_PATH_V1.to_owned(),
            source_path: utf8_path(&paths.storage_bundle)?,
            required: true,
        }),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(PersonsAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    let fragment = PersonsArtifactFragmentV1 {
        version: 1,
        owner_id: descriptor.owner_id.clone(),
        module_id: descriptor.module_id.clone(),
        artifacts,
    };
    let fragment_bytes = serde_json::to_vec_pretty(&fragment)
        .map_err(|_| PersonsAssemblyErrorV1::FragmentEncodingFailed)?;
    let mut directory = DirBuilder::new();
    directory.mode(0o700);
    directory
        .create(output_directory)
        .map_err(|_| PersonsAssemblyErrorV1::OutputUnavailable)?;
    for (index, (path, bytes)) in [
        (&paths.descriptor, descriptor.encode_to_vec()),
        (&paths.settings_schema, settings.encode_to_vec()),
        (&paths.storage_bundle, storage.encode_to_vec()),
        (&paths.artifact_fragment, fragment_bytes),
    ]
    .into_iter()
    .enumerate()
    {
        if fail_before_write == Some(index) || write_private_new(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(PersonsAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs(
    output: &Path,
    build_id: &str,
    runtime: &Path,
) -> Result<(), PersonsAssemblyErrorV1> {
    let runtime_valid = fs::symlink_metadata(runtime).ok().is_some_and(|metadata| {
        metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0
    });
    if !output.is_absolute()
        || output.parent().is_none()
        || output.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime.is_absolute()
        || !runtime_valid
    {
        return Err(PersonsAssemblyErrorV1::InvalidInput);
    }
    Ok(())
}

fn utf8_path(source: &Path) -> Result<String, PersonsAssemblyErrorV1> {
    source
        .to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(PersonsAssemblyErrorV1::InvalidInput)
}

fn write_private_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_runtime_protocol::validation::descriptor::{
        decode_descriptor_v1, decode_settings_schema_v1,
    };
    use makosh_storage_protocol::v1::StorageBundleV1;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn output_is_private_exact_and_deterministic() {
        let root = temporary();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        let first = materialize_persons_release_assembly_v1(&root.join("one"), "build-1", &runtime)
            .expect("first");
        let descriptor = decode_descriptor_v1(&fs::read(&first.descriptor).expect("descriptor"))
            .expect("canonical descriptor");
        assert_eq!(descriptor.owner_id, "persons");
        assert_eq!(descriptor.module_id, "makosh-persons-runtime");
        decode_settings_schema_v1(&fs::read(&first.settings_schema).expect("settings"))
            .expect("canonical settings");
        StorageBundleV1::decode(fs::read(&first.storage_bundle).expect("storage").as_slice())
            .expect("canonical storage");
        let first_fragment_bytes = fs::read(&first.artifact_fragment).expect("fragment");
        let fragment: PersonsArtifactFragmentV1 =
            serde_json::from_slice(&first_fragment_bytes).expect("typed fragment");
        assert_eq!(fragment.owner_id, "persons");
        assert_eq!(fragment.module_id, "makosh-persons-runtime");
        assert_eq!(fragment.artifacts.len(), 2);
        assert!(
            fragment
                .artifacts
                .windows(2)
                .all(|pair| { pair[0].artifact_id() < pair[1].artifact_id() })
        );
        let PersonsReleaseArtifactInputV1::ModuleRuntime(module) = &fragment.artifacts[0] else {
            panic!("runtime artifact must be first");
        };
        assert_eq!(module.relative_path, RUNTIME_RELATIVE_PATH_V1);
        assert_eq!(module.descriptor.relative_path, DESCRIPTOR_RELATIVE_PATH_V1);
        assert_eq!(
            module.settings_schema.relative_path,
            SETTINGS_RELATIVE_PATH_V1
        );
        let PersonsReleaseArtifactInputV1::StorageBundle(storage) = &fragment.artifacts[1] else {
            panic!("storage artifact must be second");
        };
        assert_eq!(storage.relative_path, STORAGE_RELATIVE_PATH_V1);
        assert_eq!(
            fs::metadata(root.join("one"))
                .expect("dir")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(first.settings_schema)
                .expect("file")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        fs::remove_dir_all(root.join("one")).expect("remove first assembly");
        let second =
            materialize_persons_release_assembly_v1(&root.join("one"), "build-1", &runtime)
                .expect("second identical assembly");
        assert_eq!(
            first_fragment_bytes,
            fs::read(second.artifact_fragment).expect("second fragment")
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn refuses_overwrite_symlink_and_empty_runtime() {
        let root = temporary();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        let output = root.join("existing");
        fs::create_dir(&output).expect("existing");
        assert_eq!(
            materialize_persons_release_assembly_v1(&output, "build", &runtime),
            Err(PersonsAssemblyErrorV1::InvalidInput)
        );
        let link = root.join("link");
        std::os::unix::fs::symlink(&runtime, &link).expect("link");
        assert_eq!(
            materialize_persons_release_assembly_v1(&root.join("linked"), "build", &link),
            Err(PersonsAssemblyErrorV1::InvalidInput)
        );
        let partial = root.join("partial");
        fs::create_dir(&partial).expect("partial directory");
        fs::write(partial.join(PERSONS_DESCRIPTOR_FILE_V1), b"partial")
            .expect("partial descriptor");
        assert_eq!(
            materialize_persons_release_assembly_v1(&partial, "build", &runtime),
            Err(PersonsAssemblyErrorV1::InvalidInput)
        );
        assert_eq!(
            fs::read(partial.join(PERSONS_DESCRIPTOR_FILE_V1)).expect("partial retained"),
            b"partial"
        );
        let failed = root.join("failed-write");
        assert_eq!(
            materialize_persons_release_assembly_inner_v1(&failed, "build", &runtime, Some(1),),
            Err(PersonsAssemblyErrorV1::ArtifactWriteFailed)
        );
        assert!(!failed.exists(), "partial new output must be cleaned");
        fs::remove_dir_all(root).expect("cleanup");
    }

    fn temporary() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "makosh-persons-assembly-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("root");
        path
    }

    use std::os::unix::fs::PermissionsExt;
}
