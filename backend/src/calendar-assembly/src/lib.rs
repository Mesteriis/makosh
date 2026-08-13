//! Unsigned Calendar release assembly.

#![forbid(unsafe_code)]

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use makosh_calendar_persistence::calendar_storage_bundle_v1;
use makosh_calendar_runtime::{calendar_module_descriptor_v1, calendar_settings_schema_v1};
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const CALENDAR_ASSEMBLY_FRAGMENT_VERSION_V1: u32 = 1;
pub const CALENDAR_RUNTIME_ARTIFACT_ID_V1: &str = "calendar.runtime.v1";
pub const CALENDAR_STORAGE_ARTIFACT_ID_V1: &str = "calendar.storage.v1";
pub const CALENDAR_DESCRIPTOR_FILE_V1: &str = "calendar.runtime.descriptor.pb";
pub const CALENDAR_SETTINGS_FILE_V1: &str = "calendar.runtime.settings.pb";
pub const CALENDAR_STORAGE_BUNDLE_FILE_V1: &str = "calendar.storage.bundle.pb";
pub const CALENDAR_ARTIFACT_FRAGMENT_FILE_V1: &str = "calendar.release-artifacts.json";

const RUNTIME_RELATIVE_PATH_V1: &str = "bin/makosh-calendar-runtime";
const DESCRIPTOR_RELATIVE_PATH_V1: &str = "contracts/calendar.runtime.descriptor.pb";
const SETTINGS_RELATIVE_PATH_V1: &str = "contracts/calendar.runtime.settings.pb";
const STORAGE_RELATIVE_PATH_V1: &str = "storage/calendar.storage.bundle.pb";

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
pub enum CalendarReleaseArtifactInputV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl CalendarReleaseArtifactInputV1 {
    fn artifact_id(&self) -> &str {
        match self {
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarReleaseArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<CalendarReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarReleaseAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarReleaseAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

pub fn materialize_calendar_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<CalendarReleaseAssemblyPathsV1, CalendarReleaseAssemblyErrorV1> {
    validate_inputs(output_directory, build_id, runtime_source)?;
    let runtime_bytes =
        fs::read(runtime_source).map_err(|_| CalendarReleaseAssemblyErrorV1::InvalidInput)?;
    if runtime_bytes.is_empty() {
        return Err(CalendarReleaseAssemblyErrorV1::InvalidInput);
    }
    let descriptor = calendar_module_descriptor_v1(build_id);
    let settings = calendar_settings_schema_v1();
    let storage = calendar_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(CalendarReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    let paths = CalendarReleaseAssemblyPathsV1 {
        descriptor: output_directory.join(CALENDAR_DESCRIPTOR_FILE_V1),
        settings_schema: output_directory.join(CALENDAR_SETTINGS_FILE_V1),
        storage_bundle: output_directory.join(CALENDAR_STORAGE_BUNDLE_FILE_V1),
        artifact_fragment: output_directory.join(CALENDAR_ARTIFACT_FRAGMENT_FILE_V1),
    };
    let fragment = artifact_fragment(
        &descriptor.owner_id,
        &descriptor.module_id,
        runtime_source,
        &paths,
    )?;
    let fragment_bytes = serde_json::to_vec_pretty(&fragment)
        .map_err(|_| CalendarReleaseAssemblyErrorV1::FragmentEncodingFailed)?;
    let mut directory = DirBuilder::new();
    directory.mode(0o700);
    directory
        .create(output_directory)
        .map_err(|_| CalendarReleaseAssemblyErrorV1::OutputUnavailable)?;
    let writes = [
        (&paths.descriptor, descriptor.encode_to_vec()),
        (&paths.settings_schema, settings.encode_to_vec()),
        (&paths.storage_bundle, storage.encode_to_vec()),
        (&paths.artifact_fragment, fragment_bytes),
    ];
    for (path, bytes) in writes {
        if write_new_private_file(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(CalendarReleaseAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<(), CalendarReleaseAssemblyErrorV1> {
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
        return Err(CalendarReleaseAssemblyErrorV1::InvalidInput);
    }
    Ok(())
}

fn artifact_fragment(
    owner_id: &str,
    module_id: &str,
    runtime_source: &Path,
    paths: &CalendarReleaseAssemblyPathsV1,
) -> Result<CalendarReleaseArtifactFragmentV1, CalendarReleaseAssemblyErrorV1> {
    let artifacts = vec![
        CalendarReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime".to_owned(),
            artifact_id: CALENDAR_RUNTIME_ARTIFACT_ID_V1.to_owned(),
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
        CalendarReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_kind: "storage_bundle".to_owned(),
            artifact_id: CALENDAR_STORAGE_ARTIFACT_ID_V1.to_owned(),
            relative_path: STORAGE_RELATIVE_PATH_V1.to_owned(),
            source_path: utf8_path(&paths.storage_bundle)?,
            required: true,
        }),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(CalendarReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    Ok(CalendarReleaseArtifactFragmentV1 {
        version: CALENDAR_ASSEMBLY_FRAGMENT_VERSION_V1,
        owner_id: owner_id.to_owned(),
        module_id: module_id.to_owned(),
        artifacts,
    })
}

fn utf8_path(path: &Path) -> Result<String, CalendarReleaseAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(CalendarReleaseAssemblyErrorV1::InvalidInput)
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

    use makosh_runtime_protocol::validation::descriptor::{
        decode_descriptor_v1, decode_settings_schema_v1,
    };
    use makosh_storage_protocol::v1::StorageBundleV1;

    use super::*;

    static NEXT_TEMPORARY_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn materializes_exact_unsigned_calendar_artifacts() {
        let root = temporary_directory();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        let output = root.join("assembly");
        let paths = materialize_calendar_release_assembly_v1(&output, "build-1", &runtime)
            .expect("assembly");
        let descriptor = decode_descriptor_v1(&fs::read(paths.descriptor).expect("descriptor"))
            .expect("descriptor");
        assert_eq!(descriptor.owner_id, "calendar");
        assert_eq!(descriptor.module_id, "makosh-calendar-runtime");
        decode_settings_schema_v1(&fs::read(paths.settings_schema).expect("settings"))
            .expect("settings");
        StorageBundleV1::decode(fs::read(paths.storage_bundle).expect("storage").as_slice())
            .expect("storage");
        let fragment: CalendarReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).expect("fragment"))
                .expect("fragment");
        assert_eq!(fragment.artifacts.len(), 2);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn refuses_existing_output_and_symlink_runtime() {
        let root = temporary_directory();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        let output = root.join("existing");
        fs::create_dir(&output).expect("output fixture");
        assert_eq!(
            materialize_calendar_release_assembly_v1(&output, "build-1", &runtime),
            Err(CalendarReleaseAssemblyErrorV1::InvalidInput)
        );
        let symlink = root.join("runtime-link");
        std::os::unix::fs::symlink(&runtime, &symlink).expect("symlink fixture");
        assert_eq!(
            materialize_calendar_release_assembly_v1(&root.join("other"), "build-1", &symlink),
            Err(CalendarReleaseAssemblyErrorV1::InvalidInput)
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    fn temporary_directory() -> PathBuf {
        let id = NEXT_TEMPORARY_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "makosh-calendar-assembly-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("temporary root");
        path
    }
}
