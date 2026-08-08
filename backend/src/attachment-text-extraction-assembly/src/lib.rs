#![forbid(unsafe_code)]
//! Unsigned release materialization for attachment text extraction.

use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

use makosh_attachment_text_extraction_api::{
    ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1, ATTACHMENT_TEXT_EXTRACTION_OWNER_V1,
};
use makosh_attachment_text_extraction_persistence::attachment_text_extraction_storage_bundle_v1;
use makosh_attachment_text_extraction_runtime::{
    ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1,
    attachment_text_extraction_module_descriptor_v1, attachment_text_extraction_settings_schema_v1,
};
use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::validation::validate_storage_bundle;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const PACKAGE: &str = "makosh-attachment-text-extraction-assembly";
pub const ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT_ID_V1: &str =
    "attachment_text_extraction.runtime.v1";
pub const ATTACHMENT_TEXT_EXTRACTION_STORAGE_ARTIFACT_ID_V1: &str =
    "attachment_text_extraction.storage.v1";
pub const ATTACHMENT_TEXT_EXTRACTION_DESCRIPTOR_FILE_V1: &str =
    "attachment_text_extraction.runtime.descriptor.pb";
pub const ATTACHMENT_TEXT_EXTRACTION_SETTINGS_FILE_V1: &str =
    "attachment_text_extraction.runtime.settings.pb";
pub const ATTACHMENT_TEXT_EXTRACTION_STORAGE_FILE_V1: &str =
    "attachment_text_extraction.storage.bundle.pb";
pub const ATTACHMENT_TEXT_EXTRACTION_FRAGMENT_FILE_V1: &str =
    "attachment_text_extraction.release-artifacts.json";

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
pub enum AttachmentTextExtractionReleaseArtifactInputV1 {
    BoundRuntimeArtifact(BoundRuntimeArtifactInputV1),
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl AttachmentTextExtractionReleaseArtifactInputV1 {
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
pub struct AttachmentTextExtractionReleaseArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<AttachmentTextExtractionReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTextExtractionReleaseAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionReleaseAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

pub fn materialize_attachment_text_extraction_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
    ocr_runner_source: &Path,
    ocr_english_model_source: &Path,
    ocr_russian_model_source: &Path,
) -> Result<
    AttachmentTextExtractionReleaseAssemblyPathsV1,
    AttachmentTextExtractionReleaseAssemblyErrorV1,
> {
    validate_inputs(
        output_directory,
        build_id,
        runtime_source,
        ocr_runner_source,
        ocr_english_model_source,
        ocr_russian_model_source,
    )?;
    let descriptor = attachment_text_extraction_module_descriptor_v1(build_id);
    let settings = attachment_text_extraction_settings_schema_v1();
    let storage = attachment_text_extraction_storage_bundle_v1();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings).is_err()
        || validate_storage_bundle(&storage).is_err()
    {
        return Err(AttachmentTextExtractionReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    let paths = AttachmentTextExtractionReleaseAssemblyPathsV1 {
        descriptor: output_directory.join(ATTACHMENT_TEXT_EXTRACTION_DESCRIPTOR_FILE_V1),
        settings_schema: output_directory.join(ATTACHMENT_TEXT_EXTRACTION_SETTINGS_FILE_V1),
        storage_bundle: output_directory.join(ATTACHMENT_TEXT_EXTRACTION_STORAGE_FILE_V1),
        artifact_fragment: output_directory.join(ATTACHMENT_TEXT_EXTRACTION_FRAGMENT_FILE_V1),
    };
    let fragment = artifact_fragment(
        runtime_source,
        ocr_runner_source,
        ocr_english_model_source,
        ocr_russian_model_source,
        &paths,
    )?;
    let writes = [
        (paths.descriptor.as_path(), descriptor.encode_to_vec()),
        (paths.settings_schema.as_path(), settings.encode_to_vec()),
        (paths.storage_bundle.as_path(), storage.encode_to_vec()),
        (
            paths.artifact_fragment.as_path(),
            serde_json::to_vec_pretty(&fragment).map_err(|_| {
                AttachmentTextExtractionReleaseAssemblyErrorV1::FragmentEncodingFailed
            })?,
        ),
    ];
    let mut directory = DirBuilder::new();
    directory.mode(0o700);
    directory
        .create(output_directory)
        .map_err(|_| AttachmentTextExtractionReleaseAssemblyErrorV1::OutputUnavailable)?;
    for (path, bytes) in writes {
        if write_new_private_file(path, &bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(AttachmentTextExtractionReleaseAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
    ocr_runner_source: &Path,
    ocr_english_model_source: &Path,
    ocr_russian_model_source: &Path,
) -> Result<(), AttachmentTextExtractionReleaseAssemblyErrorV1> {
    if !output_directory.is_absolute()
        || output_directory.parent().is_none()
        || output_directory.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
        || !regular_non_symlink_file(runtime_source)
        || !ocr_runner_source.is_absolute()
        || !regular_non_symlink_file(ocr_runner_source)
        || !ocr_english_model_source.is_absolute()
        || !regular_non_symlink_file(ocr_english_model_source)
        || !ocr_russian_model_source.is_absolute()
        || !regular_non_symlink_file(ocr_russian_model_source)
    {
        return Err(AttachmentTextExtractionReleaseAssemblyErrorV1::InvalidInput);
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
    ocr_runner_source: &Path,
    ocr_english_model_source: &Path,
    ocr_russian_model_source: &Path,
    paths: &AttachmentTextExtractionReleaseAssemblyPathsV1,
) -> Result<
    AttachmentTextExtractionReleaseArtifactFragmentV1,
    AttachmentTextExtractionReleaseAssemblyErrorV1,
> {
    let artifacts = vec![
        AttachmentTextExtractionReleaseArtifactInputV1::BoundRuntimeArtifact(
            BoundRuntimeArtifactInputV1 {
                artifact_kind: "module_runtime_read_only_data".to_owned(),
                artifact_id: ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1.to_owned(),
                relative_path: "runtime-resources/attachment-text-extraction/eng.traineddata"
                    .to_owned(),
                source_path: utf8_path(ocr_english_model_source)?,
                required: true,
                bound_module_id: ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1.to_owned(),
            },
        ),
        AttachmentTextExtractionReleaseArtifactInputV1::BoundRuntimeArtifact(
            BoundRuntimeArtifactInputV1 {
                artifact_kind: "module_runtime_native_executable".to_owned(),
                artifact_id: ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1.to_owned(),
                relative_path: "runtime-resources/attachment-text-extraction/tesseract-runner"
                    .to_owned(),
                source_path: utf8_path(ocr_runner_source)?,
                required: true,
                bound_module_id: ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1.to_owned(),
            },
        ),
        AttachmentTextExtractionReleaseArtifactInputV1::BoundRuntimeArtifact(
            BoundRuntimeArtifactInputV1 {
                artifact_kind: "module_runtime_read_only_data".to_owned(),
                artifact_id: ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1.to_owned(),
                relative_path: "runtime-resources/attachment-text-extraction/rus.traineddata"
                    .to_owned(),
                source_path: utf8_path(ocr_russian_model_source)?,
                required: true,
                bound_module_id: ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1.to_owned(),
            },
        ),
        AttachmentTextExtractionReleaseArtifactInputV1::ModuleRuntime(
            ModuleRuntimeArtifactInputV1 {
                artifact_kind: "module_runtime".to_owned(),
                artifact_id: ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT_ID_V1.to_owned(),
                relative_path: "bin/makosh-attachment-text-extraction-runtime".to_owned(),
                source_path: utf8_path(runtime_source)?,
                required: true,
                descriptor: ReleaseContractInputV1 {
                    relative_path: "contracts/attachment_text_extraction.runtime.descriptor.pb"
                        .to_owned(),
                    source_path: utf8_path(&paths.descriptor)?,
                },
                settings_schema: ReleaseContractInputV1 {
                    relative_path: "contracts/attachment_text_extraction.runtime.settings.pb"
                        .to_owned(),
                    source_path: utf8_path(&paths.settings_schema)?,
                },
            },
        ),
        AttachmentTextExtractionReleaseArtifactInputV1::StorageBundle(
            StorageBundleArtifactInputV1 {
                artifact_kind: "storage_bundle".to_owned(),
                artifact_id: ATTACHMENT_TEXT_EXTRACTION_STORAGE_ARTIFACT_ID_V1.to_owned(),
                relative_path: "storage/attachment_text_extraction.storage.bundle.pb".to_owned(),
                source_path: utf8_path(&paths.storage_bundle)?,
                required: true,
            },
        ),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(AttachmentTextExtractionReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    Ok(AttachmentTextExtractionReleaseArtifactFragmentV1 {
        version: 1,
        owner_id: ATTACHMENT_TEXT_EXTRACTION_OWNER_V1.to_owned(),
        module_id: ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1.to_owned(),
        artifacts,
    })
}

fn utf8_path(path: &Path) -> Result<String, AttachmentTextExtractionReleaseAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(AttachmentTextExtractionReleaseAssemblyErrorV1::InvalidInput)
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
    fn emits_valid_unsigned_runtime_and_storage_inputs_without_overwrite() {
        let root = std::env::temp_dir().join(format!(
            "makosh-text-extraction-assembly-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("test root");
        let runtime = root.join("runtime");
        let ocr_runner = root.join("tesseract-runner");
        let ocr_english = root.join("eng.traineddata");
        let ocr_russian = root.join("rus.traineddata");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        fs::write(&ocr_runner, b"runner").expect("runner fixture");
        fs::write(&ocr_english, b"english").expect("english fixture");
        fs::write(&ocr_russian, b"russian").expect("russian fixture");
        let output = root.join("assembly");
        let paths = materialize_attachment_text_extraction_release_assembly_v1(
            &output,
            "build-1",
            &runtime,
            &ocr_runner,
            &ocr_english,
            &ocr_russian,
        )
        .expect("materialize");
        let fragment: AttachmentTextExtractionReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).expect("fragment"))
                .expect("typed fragment");
        assert_eq!(fragment.owner_id, ATTACHMENT_TEXT_EXTRACTION_OWNER_V1);
        assert_eq!(fragment.module_id, ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1);
        assert_eq!(fragment.artifacts.len(), 5);
        assert!(matches!(
            &fragment.artifacts[0],
            AttachmentTextExtractionReleaseArtifactInputV1::BoundRuntimeArtifact(value)
                if value.artifact_id == ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1
                    && value.artifact_kind == "module_runtime_read_only_data"
                    && value.bound_module_id == ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1
        ));
        assert_eq!(
            materialize_attachment_text_extraction_release_assembly_v1(
                &output,
                "build-1",
                &runtime,
                &ocr_runner,
                &ocr_english,
                &ocr_russian,
            ),
            Err(AttachmentTextExtractionReleaseAssemblyErrorV1::InvalidInput)
        );
        fs::remove_dir_all(root).expect("cleanup");
    }
}
