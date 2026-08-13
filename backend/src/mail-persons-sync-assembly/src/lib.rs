//! Unsigned dormant Mail-to-Person workflow assembly.
#![forbid(unsafe_code)]

use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Component, Path, PathBuf},
};

use makosh_mail_persons_sync_persistence::mail_persons_sync_storage_bundle_v1;
use makosh_mail_persons_sync_runtime::{
    mail_persons_sync_module_descriptor_v1, mail_persons_sync_settings_schema_v1,
};
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT_ID_V1: &str = "mail_persons_sync.runtime.v1";
pub const MAIL_PERSONS_SYNC_STORAGE_ARTIFACT_ID_V1: &str = "mail_persons_sync.storage.v1";

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
pub enum MailPersonsSyncReleaseArtifactInputV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl MailPersonsSyncReleaseArtifactInputV1 {
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
pub struct MailPersonsSyncArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<MailPersonsSyncReleaseArtifactInputV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

pub fn mail_persons_sync_artifact_fragment_v1(
    runtime_source: &str,
) -> Result<MailPersonsSyncArtifactFragmentV1, MailPersonsSyncAssemblyErrorV1> {
    let runtime = Path::new(runtime_source);
    if !runtime.is_absolute()
        || runtime
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        || runtime_source.as_bytes().contains(&0)
    {
        return Err(MailPersonsSyncAssemblyErrorV1::InvalidInput);
    }
    let mut artifacts = vec![
        MailPersonsSyncReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_id: MAIL_PERSONS_SYNC_STORAGE_ARTIFACT_ID_V1.to_owned(),
            artifact_kind: "storage_bundle".to_owned(),
            relative_path: "storage/mail_persons_sync.storage.bundle.pb".to_owned(),
            source_path: "generated:mail_persons_sync.storage.bundle.pb".to_owned(),
            required: true,
        }),
        MailPersonsSyncReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_id: MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT_ID_V1.to_owned(),
            artifact_kind: "module_runtime".to_owned(),
            relative_path: "bin/makosh-mail-persons-sync-runtime".to_owned(),
            source_path: runtime_source.to_owned(),
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: "contracts/mail_persons_sync.runtime.descriptor.pb".to_owned(),
                source_path: "generated:mail_persons_sync.runtime.descriptor.pb".to_owned(),
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: "contracts/mail_persons_sync.runtime.settings.pb".to_owned(),
                source_path: "generated:mail_persons_sync.runtime.settings.pb".to_owned(),
            },
        }),
    ];
    artifacts.sort_by(|left, right| left.artifact_id().cmp(right.artifact_id()));
    Ok(MailPersonsSyncArtifactFragmentV1 {
        version: 1,
        owner_id: "mail_persons_sync".to_owned(),
        module_id: "makosh-mail-persons-sync-runtime".to_owned(),
        artifacts,
    })
}

pub fn materialize_mail_persons_sync_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<MailPersonsSyncAssemblyPathsV1, MailPersonsSyncAssemblyErrorV1> {
    materialize_mail_persons_sync_assembly_inner_v1(
        output_directory,
        build_id,
        runtime_source,
        None,
    )
}

fn materialize_mail_persons_sync_assembly_inner_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
    fail_before_write: Option<usize>,
) -> Result<MailPersonsSyncAssemblyPathsV1, MailPersonsSyncAssemblyErrorV1> {
    if !output_directory.is_absolute()
        || output_directory.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
    {
        return Err(MailPersonsSyncAssemblyErrorV1::InvalidInput);
    }
    let metadata = fs::symlink_metadata(runtime_source)
        .map_err(|_| MailPersonsSyncAssemblyErrorV1::InvalidInput)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() == 0 {
        return Err(MailPersonsSyncAssemblyErrorV1::InvalidInput);
    }
    let descriptor = mail_persons_sync_module_descriptor_v1(build_id);
    let settings = mail_persons_sync_settings_schema_v1();
    let storage = mail_persons_sync_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(MailPersonsSyncAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    DirBuilder::new()
        .mode(0o700)
        .create(output_directory)
        .map_err(|_| MailPersonsSyncAssemblyErrorV1::OutputUnavailable)?;
    let paths = MailPersonsSyncAssemblyPathsV1 {
        descriptor: output_directory.join("mail_persons_sync.runtime.descriptor.pb"),
        settings_schema: output_directory.join("mail_persons_sync.runtime.settings.pb"),
        storage_bundle: output_directory.join("mail_persons_sync.storage.bundle.pb"),
        artifact_fragment: output_directory.join("mail_persons_sync.release-artifacts.json"),
    };
    let mut fragment = mail_persons_sync_artifact_fragment_v1(
        runtime_source
            .to_str()
            .ok_or(MailPersonsSyncAssemblyErrorV1::InvalidInput)?,
    )?;
    for artifact in &mut fragment.artifacts {
        match artifact {
            MailPersonsSyncReleaseArtifactInputV1::ModuleRuntime(value) => {
                value.descriptor.source_path = absolute_utf8(&paths.descriptor)?;
                value.settings_schema.source_path = absolute_utf8(&paths.settings_schema)?;
            }
            MailPersonsSyncReleaseArtifactInputV1::StorageBundle(value) => {
                value.source_path = absolute_utf8(&paths.storage_bundle)?;
            }
        }
    }
    let fragment = serde_json::to_vec(&fragment)
        .map_err(|_| MailPersonsSyncAssemblyErrorV1::FragmentEncodingFailed)?;
    let writes = [
        (&paths.descriptor, descriptor.encode_to_vec()),
        (&paths.settings_schema, settings.encode_to_vec()),
        (&paths.storage_bundle, storage.encode_to_vec()),
        (&paths.artifact_fragment, fragment),
    ];
    for (index, (path, bytes)) in writes.into_iter().enumerate() {
        if fail_before_write == Some(index) || write_private(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(MailPersonsSyncAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn absolute_utf8(path: &Path) -> Result<String, MailPersonsSyncAssemblyErrorV1> {
    if !path.is_absolute() {
        return Err(MailPersonsSyncAssemblyErrorV1::InvalidInput);
    }
    path.to_str()
        .map(str::to_owned)
        .ok_or(MailPersonsSyncAssemblyErrorV1::InvalidInput)
}

fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

pub const PACKAGE: &str = "makosh-mail-persons-sync-assembly";

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn injected_partial_write_is_removed_and_existing_output_is_preserved() {
        let root = temporary();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        let failed = root.join("failed");
        assert_eq!(
            materialize_mail_persons_sync_assembly_inner_v1(&failed, "build-1", &runtime, Some(2),),
            Err(MailPersonsSyncAssemblyErrorV1::ArtifactWriteFailed)
        );
        assert!(!failed.exists());
        let existing = root.join("existing");
        fs::create_dir(&existing).expect("existing output");
        fs::write(existing.join("sentinel"), b"preserve").expect("sentinel");
        assert_eq!(
            materialize_mail_persons_sync_assembly_v1(&existing, "build-1", &runtime),
            Err(MailPersonsSyncAssemblyErrorV1::InvalidInput)
        );
        assert_eq!(
            fs::read(existing.join("sentinel")).expect("preserved sentinel"),
            b"preserve"
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    fn temporary() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "makosh-mail-persons-sync-assembly-unit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("temporary root");
        path
    }
}
