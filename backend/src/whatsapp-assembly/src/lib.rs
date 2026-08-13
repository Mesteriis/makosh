//! WhatsApp-owned release assembly artifact materialization.
//!
//! This package emits unsigned inputs for the generic distribution compiler.
//! It never receives release signing authority and is not a managed runtime.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use makosh_runtime_protocol::validation::descriptor::{
    validate_descriptor_v1, validate_settings_schema_v1,
};
use makosh_storage_protocol::{v1::StorageBundleV1, validation::validate_storage_bundle};
use makosh_whatsapp_persistence::{
    WHATSAPP_OWNER_RLS_STORAGE_REVISION_V1, whatsapp_owner_rls_storage_migration_v1,
    whatsapp_storage_bundle_v1,
};
use makosh_whatsapp_runtime::admission::whatsapp_module_descriptor_v1;
use makosh_whatsapp_runtime::settings::whatsapp_settings_schema_v1;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const WHATSAPP_ASSEMBLY_FRAGMENT_VERSION_V1: u32 = 1;
pub const WHATSAPP_ASSEMBLY_OWNER_ID: &str = makosh_whatsapp_runtime::admission::WHATSAPP_OWNER_ID;
pub const WHATSAPP_ASSEMBLY_MODULE_ID: &str =
    makosh_whatsapp_runtime::admission::WHATSAPP_MODULE_ID;
pub const WHATSAPP_RUNTIME_ARTIFACT_ID: &str = "whatsapp.runtime.v1";
pub const WHATSAPP_STORAGE_ARTIFACT_ID: &str = "whatsapp.storage.v1";
pub const WHATSAPP_STORAGE_BUNDLE_REVISION_V5: u32 = WHATSAPP_OWNER_RLS_STORAGE_REVISION_V1;
pub const WHATSAPP_DESCRIPTOR_FILE: &str = "whatsapp.runtime.descriptor.pb";
pub const WHATSAPP_SETTINGS_FILE: &str = "whatsapp.runtime.settings.pb";
pub const WHATSAPP_STORAGE_BUNDLE_FILE: &str = "whatsapp.storage.bundle.pb";
pub const WHATSAPP_ARTIFACT_FRAGMENT_FILE: &str = "whatsapp.release-artifacts.json";

const WHATSAPP_RUNTIME_RELATIVE_PATH: &str = "bin/makosh-whatsapp-runtime";
const WHATSAPP_DESCRIPTOR_RELATIVE_PATH: &str = "contracts/whatsapp.runtime.descriptor.pb";
const WHATSAPP_SETTINGS_RELATIVE_PATH: &str = "contracts/whatsapp.runtime.settings.pb";
const WHATSAPP_STORAGE_RELATIVE_PATH: &str = "storage/whatsapp.storage.bundle.pb";

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
pub enum WhatsAppReleaseArtifactInputV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
}

impl WhatsAppReleaseArtifactInputV1 {
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
pub struct WhatsAppReleaseArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<WhatsAppReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhatsAppReleaseAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhatsAppReleaseAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

#[must_use]
pub fn whatsapp_storage_bundle_with_owner_rls_v5() -> StorageBundleV1 {
    let mut bundle = whatsapp_storage_bundle_v1();
    bundle.revision = WHATSAPP_STORAGE_BUNDLE_REVISION_V5;
    bundle.steps.push(whatsapp_owner_rls_storage_migration_v1());
    bundle
}

/// Materializes one unsigned, exact WhatsApp release artifact set.
///
/// The output directory must be an absolute path that does not exist. The
/// runtime path is a reference for the generic release compiler; that compiler
/// independently reopens, validates and digests the source file.
pub fn materialize_whatsapp_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<WhatsAppReleaseAssemblyPathsV1, WhatsAppReleaseAssemblyErrorV1> {
    validate_inputs(output_directory, build_id, runtime_source)?;

    let descriptor = whatsapp_module_descriptor_v1(build_id);
    let settings_schema = whatsapp_settings_schema_v1();
    let storage_bundle = whatsapp_storage_bundle_with_owner_rls_v5();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings_schema).is_err()
        || validate_storage_bundle(&storage_bundle).is_err()
    {
        return Err(WhatsAppReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }

    let descriptor_bytes = descriptor.encode_to_vec();
    let settings_bytes = settings_schema.encode_to_vec();
    let storage_bytes = storage_bundle.encode_to_vec();
    let paths = WhatsAppReleaseAssemblyPathsV1 {
        descriptor: output_directory.join(WHATSAPP_DESCRIPTOR_FILE),
        settings_schema: output_directory.join(WHATSAPP_SETTINGS_FILE),
        storage_bundle: output_directory.join(WHATSAPP_STORAGE_BUNDLE_FILE),
        artifact_fragment: output_directory.join(WHATSAPP_ARTIFACT_FRAGMENT_FILE),
    };
    let fragment = artifact_fragment(
        runtime_source,
        &paths.descriptor,
        &paths.settings_schema,
        &paths.storage_bundle,
    )?;
    let fragment_bytes = serde_json::to_vec_pretty(&fragment)
        .map_err(|_| WhatsAppReleaseAssemblyErrorV1::FragmentEncodingFailed)?;

    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(output_directory)
        .map_err(|_| WhatsAppReleaseAssemblyErrorV1::OutputUnavailable)?;

    let writes = [
        (&paths.descriptor, descriptor_bytes.as_slice()),
        (&paths.settings_schema, settings_bytes.as_slice()),
        (&paths.storage_bundle, storage_bytes.as_slice()),
        (&paths.artifact_fragment, fragment_bytes.as_slice()),
    ];
    for (path, bytes) in writes {
        if write_new_private_file(path, bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(WhatsAppReleaseAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
) -> Result<(), WhatsAppReleaseAssemblyErrorV1> {
    if !output_directory.is_absolute()
        || output_directory.parent().is_none()
        || output_directory.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
        || !regular_non_symlink_file(runtime_source)
    {
        return Err(WhatsAppReleaseAssemblyErrorV1::InvalidInput);
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
) -> Result<WhatsAppReleaseArtifactFragmentV1, WhatsAppReleaseAssemblyErrorV1> {
    let artifacts = vec![
        WhatsAppReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime".to_owned(),
            artifact_id: WHATSAPP_RUNTIME_ARTIFACT_ID.to_owned(),
            relative_path: WHATSAPP_RUNTIME_RELATIVE_PATH.to_owned(),
            source_path: utf8_path(runtime_source)?,
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: WHATSAPP_DESCRIPTOR_RELATIVE_PATH.to_owned(),
                source_path: utf8_path(descriptor)?,
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: WHATSAPP_SETTINGS_RELATIVE_PATH.to_owned(),
                source_path: utf8_path(settings_schema)?,
            },
        }),
        WhatsAppReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_kind: "storage_bundle".to_owned(),
            artifact_id: WHATSAPP_STORAGE_ARTIFACT_ID.to_owned(),
            relative_path: WHATSAPP_STORAGE_RELATIVE_PATH.to_owned(),
            source_path: utf8_path(storage_bundle)?,
            required: true,
        }),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(WhatsAppReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    Ok(WhatsAppReleaseArtifactFragmentV1 {
        version: WHATSAPP_ASSEMBLY_FRAGMENT_VERSION_V1,
        owner_id: WHATSAPP_ASSEMBLY_OWNER_ID.to_owned(),
        module_id: WHATSAPP_ASSEMBLY_MODULE_ID.to_owned(),
        artifacts,
    })
}

fn utf8_path(path: &Path) -> Result<String, WhatsAppReleaseAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(WhatsAppReleaseAssemblyErrorV1::InvalidInput)
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
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use makosh_runtime_protocol::validation::descriptor::{
        decode_descriptor_v1, decode_settings_schema_v1,
    };
    use makosh_storage_protocol::v1::StorageBundleV1;

    use super::*;

    static NEXT_TEMPORARY_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn materializes_exact_canonical_artifacts_and_sorted_unsigned_fragment() {
        let root = temporary_directory();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        let output = root.join("assembly");

        let paths = materialize_whatsapp_release_assembly_v1(&output, "build-1", &runtime)
            .expect("materialize WhatsApp assembly");
        let descriptor_bytes = fs::read(&paths.descriptor).expect("descriptor bytes");
        let settings_bytes = fs::read(&paths.settings_schema).expect("settings bytes");
        let storage_bytes = fs::read(&paths.storage_bundle).expect("storage bytes");
        let descriptor =
            decode_descriptor_v1(&descriptor_bytes).expect("valid descriptor artifact");
        let settings = decode_settings_schema_v1(&settings_bytes).expect("valid settings artifact");
        let storage =
            StorageBundleV1::decode(storage_bytes.as_slice()).expect("valid storage artifact");
        let fragment_bytes = fs::read(paths.artifact_fragment).expect("fragment bytes");
        let fragment: WhatsAppReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fragment_bytes).expect("typed fragment");

        assert_eq!(
            descriptor_bytes,
            whatsapp_module_descriptor_v1("build-1").encode_to_vec()
        );
        assert_eq!(
            settings_bytes,
            whatsapp_settings_schema_v1().encode_to_vec()
        );
        assert_eq!(
            storage_bytes,
            whatsapp_storage_bundle_with_owner_rls_v5().encode_to_vec()
        );
        assert_eq!(storage.revision, WHATSAPP_STORAGE_BUNDLE_REVISION_V5);
        assert_eq!(descriptor.module_id, WHATSAPP_ASSEMBLY_MODULE_ID);
        assert_eq!(settings.major, 1);
        assert_eq!(storage.owner_id, WHATSAPP_ASSEMBLY_OWNER_ID);
        assert_eq!(
            fragment
                .artifacts
                .iter()
                .map(WhatsAppReleaseArtifactInputV1::artifact_id)
                .collect::<Vec<_>>(),
            vec![WHATSAPP_RUNTIME_ARTIFACT_ID, WHATSAPP_STORAGE_ARTIFACT_ID]
        );
        assert_eq!(fragment.version, WHATSAPP_ASSEMBLY_FRAGMENT_VERSION_V1);
        assert_eq!(fragment.owner_id, WHATSAPP_ASSEMBLY_OWNER_ID);
        assert_eq!(fragment.module_id, WHATSAPP_ASSEMBLY_MODULE_ID);
        let fragment_text = String::from_utf8(fragment_bytes).expect("UTF-8 fragment");
        for forbidden in ["signature", "sha256", "grant", "secret", "credential"] {
            assert!(!fragment_text.contains(forbidden));
        }

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn canonical_contract_bytes_are_deterministic_and_output_never_overwrites() {
        let root = temporary_directory();
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        let first =
            materialize_whatsapp_release_assembly_v1(&root.join("first"), "build-1", &runtime)
                .expect("first assembly");
        let second =
            materialize_whatsapp_release_assembly_v1(&root.join("second"), "build-1", &runtime)
                .expect("second assembly");

        assert_eq!(
            fs::read(first.descriptor).expect("first descriptor"),
            fs::read(second.descriptor).expect("second descriptor")
        );
        assert_eq!(
            fs::read(first.settings_schema).expect("first settings"),
            fs::read(second.settings_schema).expect("second settings")
        );
        assert_eq!(
            fs::read(first.storage_bundle).expect("first storage"),
            fs::read(second.storage_bundle).expect("second storage")
        );
        assert_eq!(
            materialize_whatsapp_release_assembly_v1(&root.join("first"), "build-1", &runtime),
            Err(WhatsAppReleaseAssemblyErrorV1::InvalidInput)
        );

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_relative_missing_empty_and_symlinked_release_inputs() {
        use std::os::unix::fs::symlink;

        let root = temporary_directory();
        let runtime = root.join("runtime");
        let empty_runtime = root.join("empty-runtime");
        let runtime_link = root.join("runtime-link");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        fs::write(&empty_runtime, b"").expect("empty runtime fixture");
        symlink(&runtime, &runtime_link).expect("runtime symlink");

        for (output, build_id, source) in [
            (PathBuf::from("relative"), "build-1", runtime.clone()),
            (root.join("missing"), "build-1", root.join("absent")),
            (root.join("empty"), "build-1", empty_runtime),
            (root.join("symlink"), "build-1", runtime_link),
            (root.join("empty-build"), "", runtime.clone()),
            (root.join("non-ascii-build"), "сборка", runtime.clone()),
        ] {
            assert_eq!(
                materialize_whatsapp_release_assembly_v1(&output, build_id, &source),
                Err(WhatsAppReleaseAssemblyErrorV1::InvalidInput)
            );
        }

        let existing_file = root.join("existing-output");
        fs::write(&existing_file, b"occupied").expect("existing output fixture");
        assert_eq!(
            materialize_whatsapp_release_assembly_v1(&existing_file, "build-1", &runtime),
            Err(WhatsAppReleaseAssemblyErrorV1::InvalidInput)
        );

        fs::remove_dir_all(root).expect("remove fixture");
    }

    fn temporary_directory() -> PathBuf {
        let id = NEXT_TEMPORARY_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "makosh-whatsapp-assembly-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create fixture root");
        path
    }
}
