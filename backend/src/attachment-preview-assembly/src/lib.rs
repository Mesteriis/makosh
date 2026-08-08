//! Attachment Preview unsigned release artifact materialization.
//!
//! This assembly owns descriptor, settings and Storage artifact composition.
//! It never launches the runtime and never receives signing authority.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use makosh_attachment_preview_persistence::attachment_preview_storage_bundle_v1;
use makosh_attachment_preview_runtime::{
    admission::attachment_preview_module_descriptor_v1, attachment_preview_settings_schema_v1,
};
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const ATTACHMENT_PREVIEW_ASSEMBLY_FRAGMENT_VERSION_V1: u32 = 1;
pub const ATTACHMENT_PREVIEW_ASSEMBLY_OWNER_ID: &str =
    makosh_attachment_preview_api::ATTACHMENT_PREVIEW_OWNER_V1;
pub const ATTACHMENT_PREVIEW_ASSEMBLY_MODULE_ID: &str =
    makosh_attachment_preview_api::ATTACHMENT_PREVIEW_MODULE_ID_V1;
pub const ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT_ID: &str = "attachment_preview.runtime.v1";
pub const ATTACHMENT_PREVIEW_STORAGE_ARTIFACT_ID: &str = "attachment_preview.storage.v1";
pub const ATTACHMENT_PREVIEW_DESCRIPTOR_FILE: &str = "attachment-preview.runtime.descriptor.pb";
pub const ATTACHMENT_PREVIEW_SETTINGS_FILE: &str = "attachment-preview.runtime.settings.pb";
pub const ATTACHMENT_PREVIEW_STORAGE_BUNDLE_FILE: &str = "attachment-preview.storage.bundle.pb";
pub const ATTACHMENT_PREVIEW_ARTIFACT_FRAGMENT_FILE: &str =
    "attachment-preview.release-artifacts.json";

const RUNTIME_RELATIVE_PATH: &str = "bin/makosh-attachment-preview-runtime";
const DESCRIPTOR_RELATIVE_PATH: &str = "contracts/attachment-preview.runtime.descriptor.pb";
const SETTINGS_RELATIVE_PATH: &str = "contracts/attachment-preview.runtime.settings.pb";
const STORAGE_RELATIVE_PATH: &str = "storage/attachment-preview.storage.bundle.pb";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseContractInputV1 {
    pub relative_path: String,
    pub source_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageBundleArtifactInputV1 {
    pub artifact_kind: String,
    pub artifact_id: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttachmentPreviewReleaseArtifactInputV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl AttachmentPreviewReleaseArtifactInputV1 {
    fn artifact_id(&self) -> &str {
        match self {
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachmentPreviewReleaseArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<AttachmentPreviewReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewReleaseAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewReleaseAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

pub fn materialize_attachment_preview_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<AttachmentPreviewReleaseAssemblyPathsV1, AttachmentPreviewReleaseAssemblyErrorV1> {
    validate_inputs(output_directory, build_id, runtime_source)?;

    let descriptor = attachment_preview_module_descriptor_v1(build_id);
    let settings = attachment_preview_settings_schema_v1();
    let storage = attachment_preview_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(AttachmentPreviewReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }

    let paths = AttachmentPreviewReleaseAssemblyPathsV1 {
        descriptor: output_directory.join(ATTACHMENT_PREVIEW_DESCRIPTOR_FILE),
        settings_schema: output_directory.join(ATTACHMENT_PREVIEW_SETTINGS_FILE),
        storage_bundle: output_directory.join(ATTACHMENT_PREVIEW_STORAGE_BUNDLE_FILE),
        artifact_fragment: output_directory.join(ATTACHMENT_PREVIEW_ARTIFACT_FRAGMENT_FILE),
    };
    let fragment = artifact_fragment(
        runtime_source,
        &paths.descriptor,
        &paths.settings_schema,
        &paths.storage_bundle,
    )?;
    let fragment = serde_json::to_vec_pretty(&fragment)
        .map_err(|_| AttachmentPreviewReleaseAssemblyErrorV1::FragmentEncodingFailed)?;

    let mut directory = DirBuilder::new();
    directory.mode(0o700);
    directory
        .create(output_directory)
        .map_err(|_| AttachmentPreviewReleaseAssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in [
        (paths.descriptor.as_path(), descriptor.encode_to_vec()),
        (paths.settings_schema.as_path(), settings.encode_to_vec()),
        (paths.storage_bundle.as_path(), storage.encode_to_vec()),
        (paths.artifact_fragment.as_path(), fragment),
    ] {
        if write_new_private_file(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(AttachmentPreviewReleaseAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<(), AttachmentPreviewReleaseAssemblyErrorV1> {
    if !output_directory.is_absolute()
        || output_directory.parent().is_none()
        || output_directory.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
        || !regular_non_symlink_file(runtime_source)
    {
        return Err(AttachmentPreviewReleaseAssemblyErrorV1::InvalidInput);
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
    settings: &Path,
    storage: &Path,
) -> Result<AttachmentPreviewReleaseArtifactFragmentV1, AttachmentPreviewReleaseAssemblyErrorV1> {
    let artifacts = vec![
        AttachmentPreviewReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime".to_owned(),
            artifact_id: ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT_ID.to_owned(),
            relative_path: RUNTIME_RELATIVE_PATH.to_owned(),
            source_path: utf8_path(runtime_source)?,
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: DESCRIPTOR_RELATIVE_PATH.to_owned(),
                source_path: utf8_path(descriptor)?,
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: SETTINGS_RELATIVE_PATH.to_owned(),
                source_path: utf8_path(settings)?,
            },
        }),
        AttachmentPreviewReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_kind: "storage_bundle".to_owned(),
            artifact_id: ATTACHMENT_PREVIEW_STORAGE_ARTIFACT_ID.to_owned(),
            relative_path: STORAGE_RELATIVE_PATH.to_owned(),
            source_path: utf8_path(storage)?,
            required: true,
        }),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(AttachmentPreviewReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    Ok(AttachmentPreviewReleaseArtifactFragmentV1 {
        version: ATTACHMENT_PREVIEW_ASSEMBLY_FRAGMENT_VERSION_V1,
        owner_id: ATTACHMENT_PREVIEW_ASSEMBLY_OWNER_ID.to_owned(),
        module_id: ATTACHMENT_PREVIEW_ASSEMBLY_MODULE_ID.to_owned(),
        artifacts,
    })
}

fn utf8_path(path: &Path) -> Result<String, AttachmentPreviewReleaseAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(AttachmentPreviewReleaseAssemblyErrorV1::InvalidInput)
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

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn emits_exact_sorted_unsigned_artifacts_without_overwrite() {
        let root = temporary_directory();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        let output = root.join("assembly");
        let paths =
            materialize_attachment_preview_release_assembly_v1(&output, "build-1", &runtime)
                .expect("materialize assembly");

        let descriptor =
            decode_descriptor_v1(&fs::read(&paths.descriptor).expect("descriptor bytes"))
                .expect("descriptor");
        let settings =
            decode_settings_schema_v1(&fs::read(&paths.settings_schema).expect("settings bytes"))
                .expect("settings");
        let storage_bytes = fs::read(&paths.storage_bundle).expect("storage bytes");
        let storage = StorageBundleV1::decode(storage_bytes.as_slice()).expect("storage bundle");
        let fragment_bytes = fs::read(&paths.artifact_fragment).expect("fragment bytes");
        let fragment: AttachmentPreviewReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fragment_bytes).expect("typed fragment");

        assert_eq!(descriptor.module_id, ATTACHMENT_PREVIEW_ASSEMBLY_MODULE_ID);
        assert_eq!(
            descriptor.module_kind,
            makosh_runtime_protocol::v1::ModuleKindV1::Workflow as i32
        );
        assert_eq!(settings.major, 1);
        assert_eq!(storage.owner_id, ATTACHMENT_PREVIEW_ASSEMBLY_OWNER_ID);
        assert_eq!(
            fragment
                .artifacts
                .iter()
                .map(AttachmentPreviewReleaseArtifactInputV1::artifact_id)
                .collect::<Vec<_>>(),
            vec![
                ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT_ID,
                ATTACHMENT_PREVIEW_STORAGE_ARTIFACT_ID,
            ]
        );
        let fragment_text = String::from_utf8(fragment_bytes).expect("UTF-8 fragment");
        for forbidden in ["signature", "sha256", "grant", "secret", "credential"] {
            assert!(!fragment_text.contains(forbidden));
        }
        assert_eq!(
            materialize_attachment_preview_release_assembly_v1(&output, "build-1", &runtime),
            Err(AttachmentPreviewReleaseAssemblyErrorV1::InvalidInput)
        );
        fs::remove_dir_all(root).expect("remove fixture");
    }

    fn temporary_directory() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "makosh-attachment-preview-assembly-{}-{id}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).expect("clear fixture");
        }
        fs::create_dir(&path).expect("fixture root");
        path
    }
}
