//! Telegram-owned release assembly artifact materialization.
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
use makosh_storage_protocol::v1::StorageBundleV1;
use makosh_storage_protocol::validation::validate_storage_bundle;
use makosh_telegram_automation_persistence::schema::{
    TELEGRAM_AUTOMATION_STORAGE_REVISION_V1, telegram_automation_storage_migration_v1,
};
use makosh_telegram_calls_persistence::{
    TELEGRAM_CALLS_STORAGE_REVISION_V1, TELEGRAM_CALLS_STORAGE_REVISION_V2,
    TELEGRAM_CALLS_STORAGE_REVISION_V3, TELEGRAM_CALLS_STORAGE_REVISION_V4,
    TELEGRAM_CALLS_STORAGE_REVISION_V5, telegram_calls_storage_migration_v1,
    telegram_calls_storage_migration_v2, telegram_calls_storage_migration_v3,
    telegram_calls_storage_migration_v4, telegram_calls_storage_migration_v5,
};
use makosh_telegram_persistence::{
    TELEGRAM_DELIVERY_INTENT_STORAGE_REVISION_V1, TELEGRAM_DELIVERY_ROUTE_STORAGE_REVISION_V1,
    TELEGRAM_OWNER_RLS_STORAGE_REVISION_V1, telegram_delivery_intent_storage_migration_v1,
    telegram_delivery_route_storage_migration_v1, telegram_owner_rls_storage_migration_v1,
    telegram_storage_bundle_v1,
};
use makosh_telegram_runtime::admission::telegram_module_descriptor_v1;
use makosh_telegram_runtime::settings::telegram_settings_schema_v1;
use prost::Message;
use serde::{Deserialize, Serialize};

pub const TELEGRAM_ASSEMBLY_FRAGMENT_VERSION_V1: u32 = 1;
pub const TELEGRAM_ASSEMBLY_OWNER_ID: &str = "telegram";
pub const TELEGRAM_ASSEMBLY_MODULE_ID: &str = "makosh-telegram-runtime";
pub const TELEGRAM_RUNTIME_ARTIFACT_ID: &str = "telegram.runtime.v1";
pub const TELEGRAM_STORAGE_ARTIFACT_ID: &str = "telegram.storage.v1";
pub const TELEGRAM_TDJSON_ARTIFACT_ID: &str = "telegram.tdjson.v1";
pub const TELEGRAM_TGCALLS_ARTIFACT_ID: &str = "telegram.tgcalls.v1";
pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V2: u32 = TELEGRAM_AUTOMATION_STORAGE_REVISION_V1;
pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V3: u32 = TELEGRAM_CALLS_STORAGE_REVISION_V1;
pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V4: u32 = TELEGRAM_CALLS_STORAGE_REVISION_V2;
pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V5: u32 = TELEGRAM_CALLS_STORAGE_REVISION_V3;
pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V6: u32 = TELEGRAM_CALLS_STORAGE_REVISION_V4;
pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V7: u32 = TELEGRAM_DELIVERY_ROUTE_STORAGE_REVISION_V1;
pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V8: u32 = TELEGRAM_DELIVERY_INTENT_STORAGE_REVISION_V1;
pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V9: u32 = TELEGRAM_CALLS_STORAGE_REVISION_V5;
pub const TELEGRAM_STORAGE_BUNDLE_REVISION_V10: u32 = TELEGRAM_OWNER_RLS_STORAGE_REVISION_V1;
pub const TELEGRAM_DESCRIPTOR_FILE: &str = "telegram.runtime.descriptor.pb";
pub const TELEGRAM_SETTINGS_FILE: &str = "telegram.runtime.settings.pb";
pub const TELEGRAM_STORAGE_BUNDLE_FILE: &str = "telegram.storage.bundle.pb";
pub const TELEGRAM_ARTIFACT_FRAGMENT_FILE: &str = "telegram.release-artifacts.json";

const TELEGRAM_RUNTIME_RELATIVE_PATH: &str = "bin/makosh-telegram-runtime";
const TELEGRAM_DESCRIPTOR_RELATIVE_PATH: &str = "contracts/telegram.runtime.descriptor.pb";
const TELEGRAM_SETTINGS_RELATIVE_PATH: &str = "contracts/telegram.runtime.settings.pb";
const TELEGRAM_STORAGE_RELATIVE_PATH: &str = "storage/telegram.storage.bundle.pb";
const TELEGRAM_TDJSON_RELATIVE_PATH: &str = "lib/libtdjson.dylib";
const TELEGRAM_TGCALLS_RELATIVE_PATH: &str = "lib/libmakosh_tgcalls_bridge.dylib";

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
#[serde(deny_unknown_fields)]
pub struct NativeDependencyArtifactInputV1 {
    pub artifact_kind: String,
    pub artifact_id: String,
    pub relative_path: String,
    pub source_path: String,
    pub required: bool,
    pub bound_module_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TelegramReleaseArtifactInputV1 {
    ModuleRuntime(ModuleRuntimeArtifactInputV1),
    StorageBundle(StorageBundleArtifactInputV1),
    NativeDependency(NativeDependencyArtifactInputV1),
}

impl TelegramReleaseArtifactInputV1 {
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        match self {
            Self::ModuleRuntime(value) => &value.artifact_id,
            Self::StorageBundle(value) => &value.artifact_id,
            Self::NativeDependency(value) => &value.artifact_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelegramReleaseArtifactFragmentV1 {
    pub version: u32,
    pub owner_id: String,
    pub module_id: String,
    pub artifacts: Vec<TelegramReleaseArtifactInputV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramReleaseAssemblyPathsV1 {
    pub descriptor: PathBuf,
    pub settings_schema: PathBuf,
    pub storage_bundle: PathBuf,
    pub artifact_fragment: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramReleaseAssemblyErrorV1 {
    InvalidInput,
    InvalidCanonicalArtifact,
    OutputUnavailable,
    ArtifactWriteFailed,
    FragmentEncodingFailed,
}

#[must_use]
pub fn telegram_storage_bundle_with_automation_v2() -> StorageBundleV1 {
    let mut bundle = telegram_storage_bundle_v1();
    bundle.revision = TELEGRAM_STORAGE_BUNDLE_REVISION_V2;
    bundle
        .steps
        .push(telegram_automation_storage_migration_v1());
    bundle
}

#[must_use]
pub fn telegram_storage_bundle_with_calls_v3() -> StorageBundleV1 {
    let mut bundle = telegram_storage_bundle_with_automation_v2();
    bundle.revision = TELEGRAM_STORAGE_BUNDLE_REVISION_V3;
    bundle.steps.push(telegram_calls_storage_migration_v1());
    bundle
}

#[must_use]
pub fn telegram_storage_bundle_with_call_signaling_v4() -> StorageBundleV1 {
    let mut bundle = telegram_storage_bundle_with_calls_v3();
    bundle.revision = TELEGRAM_STORAGE_BUNDLE_REVISION_V4;
    bundle.steps.push(telegram_calls_storage_migration_v2());
    bundle
}

#[must_use]
pub fn telegram_storage_bundle_with_call_media_v5() -> StorageBundleV1 {
    let mut bundle = telegram_storage_bundle_with_call_signaling_v4();
    bundle.revision = TELEGRAM_STORAGE_BUNDLE_REVISION_V5;
    bundle.steps.push(telegram_calls_storage_migration_v3());
    bundle
}

#[must_use]
pub fn telegram_storage_bundle_with_calls_backfill_v6() -> StorageBundleV1 {
    let mut bundle = telegram_storage_bundle_with_call_media_v5();
    bundle.revision = TELEGRAM_STORAGE_BUNDLE_REVISION_V6;
    bundle.steps.push(telegram_calls_storage_migration_v4());
    bundle
}

#[must_use]
pub fn telegram_storage_bundle_with_delivery_route_v7() -> StorageBundleV1 {
    let mut bundle = telegram_storage_bundle_with_calls_backfill_v6();
    bundle.revision = TELEGRAM_STORAGE_BUNDLE_REVISION_V7;
    bundle
        .steps
        .push(telegram_delivery_route_storage_migration_v1());
    bundle
}

#[must_use]
pub fn telegram_storage_bundle_with_delivery_intent_v8() -> StorageBundleV1 {
    let mut bundle = telegram_storage_bundle_with_delivery_route_v7();
    bundle.revision = TELEGRAM_STORAGE_BUNDLE_REVISION_V8;
    bundle
        .steps
        .push(telegram_delivery_intent_storage_migration_v1());
    bundle
}

#[must_use]
pub fn telegram_storage_bundle_with_call_evidence_v9() -> StorageBundleV1 {
    let mut bundle = telegram_storage_bundle_with_delivery_intent_v8();
    bundle.revision = TELEGRAM_STORAGE_BUNDLE_REVISION_V9;
    bundle.steps.push(telegram_calls_storage_migration_v5());
    bundle
}

#[must_use]
pub fn telegram_storage_bundle_with_owner_rls_v10() -> StorageBundleV1 {
    let mut bundle = telegram_storage_bundle_with_call_evidence_v9();
    bundle.revision = TELEGRAM_STORAGE_BUNDLE_REVISION_V10;
    bundle.steps.push(telegram_owner_rls_storage_migration_v1());
    bundle
}

/// Materializes one unsigned, exact Telegram release artifact set.
///
/// The output directory must be an absolute path that does not exist. Runtime
/// and TDJson paths are references for the generic release compiler; that
/// compiler independently reopens, validates and digests the source files.
pub fn materialize_telegram_release_assembly_v1(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
    tdjson_source: &Path,
    tgcalls_source: &Path,
) -> Result<TelegramReleaseAssemblyPathsV1, TelegramReleaseAssemblyErrorV1> {
    validate_inputs(
        output_directory,
        build_id,
        runtime_source,
        tdjson_source,
        tgcalls_source,
    )?;

    let descriptor = telegram_module_descriptor_v1(build_id);
    let settings_schema = telegram_settings_schema_v1();
    let storage_bundle = telegram_storage_bundle_with_owner_rls_v10();
    if validate_descriptor_v1(&descriptor).is_err()
        || validate_settings_schema_v1(&settings_schema).is_err()
        || validate_storage_bundle(&storage_bundle).is_err()
    {
        return Err(TelegramReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }

    let descriptor_bytes = descriptor.encode_to_vec();
    let settings_bytes = settings_schema.encode_to_vec();
    let storage_bytes = storage_bundle.encode_to_vec();
    let paths = TelegramReleaseAssemblyPathsV1 {
        descriptor: output_directory.join(TELEGRAM_DESCRIPTOR_FILE),
        settings_schema: output_directory.join(TELEGRAM_SETTINGS_FILE),
        storage_bundle: output_directory.join(TELEGRAM_STORAGE_BUNDLE_FILE),
        artifact_fragment: output_directory.join(TELEGRAM_ARTIFACT_FRAGMENT_FILE),
    };
    let fragment = artifact_fragment(
        runtime_source,
        tdjson_source,
        tgcalls_source,
        &paths.descriptor,
        &paths.settings_schema,
        &paths.storage_bundle,
    )?;
    let fragment_bytes = serde_json::to_vec_pretty(&fragment)
        .map_err(|_| TelegramReleaseAssemblyErrorV1::FragmentEncodingFailed)?;

    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(output_directory)
        .map_err(|_| TelegramReleaseAssemblyErrorV1::OutputUnavailable)?;

    let writes = [
        (&paths.descriptor, descriptor_bytes.as_slice()),
        (&paths.settings_schema, settings_bytes.as_slice()),
        (&paths.storage_bundle, storage_bytes.as_slice()),
        (&paths.artifact_fragment, fragment_bytes.as_slice()),
    ];
    for (path, bytes) in writes {
        if write_new_private_file(path, bytes).is_err() {
            let _ = fs::remove_dir_all(output_directory);
            return Err(TelegramReleaseAssemblyErrorV1::ArtifactWriteFailed);
        }
    }
    Ok(paths)
}

fn validate_inputs(
    output_directory: &Path,
    build_id: &str,
    runtime_source: &Path,
    tdjson_source: &Path,
    tgcalls_source: &Path,
) -> Result<(), TelegramReleaseAssemblyErrorV1> {
    if !output_directory.is_absolute()
        || output_directory.parent().is_none()
        || output_directory.exists()
        || build_id.is_empty()
        || build_id.len() > 128
        || !build_id.is_ascii()
        || !runtime_source.is_absolute()
        || !tdjson_source.is_absolute()
        || !tgcalls_source.is_absolute()
        || !regular_non_symlink_file(runtime_source)
        || !regular_non_symlink_file(tdjson_source)
        || !regular_non_symlink_file(tgcalls_source)
    {
        return Err(TelegramReleaseAssemblyErrorV1::InvalidInput);
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
    tdjson_source: &Path,
    tgcalls_source: &Path,
    descriptor: &Path,
    settings_schema: &Path,
    storage_bundle: &Path,
) -> Result<TelegramReleaseArtifactFragmentV1, TelegramReleaseAssemblyErrorV1> {
    let runtime_source = utf8_path(runtime_source)?;
    let tdjson_source = utf8_path(tdjson_source)?;
    let tgcalls_source = utf8_path(tgcalls_source)?;
    let descriptor = utf8_path(descriptor)?;
    let settings_schema = utf8_path(settings_schema)?;
    let storage_bundle = utf8_path(storage_bundle)?;
    let artifacts = vec![
        TelegramReleaseArtifactInputV1::ModuleRuntime(ModuleRuntimeArtifactInputV1 {
            artifact_kind: "module_runtime".to_owned(),
            artifact_id: TELEGRAM_RUNTIME_ARTIFACT_ID.to_owned(),
            relative_path: TELEGRAM_RUNTIME_RELATIVE_PATH.to_owned(),
            source_path: runtime_source,
            required: true,
            descriptor: ReleaseContractInputV1 {
                relative_path: TELEGRAM_DESCRIPTOR_RELATIVE_PATH.to_owned(),
                source_path: descriptor,
            },
            settings_schema: ReleaseContractInputV1 {
                relative_path: TELEGRAM_SETTINGS_RELATIVE_PATH.to_owned(),
                source_path: settings_schema,
            },
        }),
        TelegramReleaseArtifactInputV1::StorageBundle(StorageBundleArtifactInputV1 {
            artifact_kind: "storage_bundle".to_owned(),
            artifact_id: TELEGRAM_STORAGE_ARTIFACT_ID.to_owned(),
            relative_path: TELEGRAM_STORAGE_RELATIVE_PATH.to_owned(),
            source_path: storage_bundle,
            required: true,
        }),
        TelegramReleaseArtifactInputV1::NativeDependency(NativeDependencyArtifactInputV1 {
            artifact_kind: "module_runtime_native_dependency".to_owned(),
            artifact_id: TELEGRAM_TDJSON_ARTIFACT_ID.to_owned(),
            relative_path: TELEGRAM_TDJSON_RELATIVE_PATH.to_owned(),
            source_path: tdjson_source,
            required: true,
            bound_module_id: TELEGRAM_ASSEMBLY_MODULE_ID.to_owned(),
        }),
        TelegramReleaseArtifactInputV1::NativeDependency(NativeDependencyArtifactInputV1 {
            artifact_kind: "module_runtime_native_dependency".to_owned(),
            artifact_id: TELEGRAM_TGCALLS_ARTIFACT_ID.to_owned(),
            relative_path: TELEGRAM_TGCALLS_RELATIVE_PATH.to_owned(),
            source_path: tgcalls_source,
            required: true,
            bound_module_id: TELEGRAM_ASSEMBLY_MODULE_ID.to_owned(),
        }),
    ];
    if !artifacts
        .windows(2)
        .all(|pair| pair[0].artifact_id() < pair[1].artifact_id())
    {
        return Err(TelegramReleaseAssemblyErrorV1::InvalidCanonicalArtifact);
    }
    Ok(TelegramReleaseArtifactFragmentV1 {
        version: TELEGRAM_ASSEMBLY_FRAGMENT_VERSION_V1,
        owner_id: TELEGRAM_ASSEMBLY_OWNER_ID.to_owned(),
        module_id: TELEGRAM_ASSEMBLY_MODULE_ID.to_owned(),
        artifacts,
    })
}

fn utf8_path(path: &Path) -> Result<String, TelegramReleaseAssemblyErrorV1> {
    path.to_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(TelegramReleaseAssemblyErrorV1::InvalidInput)
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

    use super::*;
    use makosh_runtime_protocol::validation::descriptor::{
        decode_descriptor_v1, decode_settings_schema_v1,
    };

    static NEXT_TEMPORARY_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn materializes_valid_exact_artifacts_and_sorted_fragment() {
        let root = temporary_directory();
        let runtime = root.join("runtime");
        let tdjson = root.join("libtdjson.dylib");
        let tgcalls = root.join("libmakosh_tgcalls_bridge.dylib");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        fs::write(&tdjson, b"tdjson").expect("TDJson fixture");
        fs::write(&tgcalls, b"tgcalls").expect("tgcalls fixture");
        let output = root.join("assembly");

        let paths = materialize_telegram_release_assembly_v1(
            &output, "build-1", &runtime, &tdjson, &tgcalls,
        )
        .expect("materialize Telegram assembly");
        let descriptor =
            decode_descriptor_v1(&fs::read(paths.descriptor).expect("descriptor bytes"))
                .expect("valid descriptor");
        let settings =
            decode_settings_schema_v1(&fs::read(paths.settings_schema).expect("settings bytes"))
                .expect("valid settings");
        let storage = StorageBundleV1::decode(
            fs::read(paths.storage_bundle)
                .expect("storage bytes")
                .as_slice(),
        )
        .expect("valid storage bundle");
        let fragment: TelegramReleaseArtifactFragmentV1 =
            serde_json::from_slice(&fs::read(paths.artifact_fragment).expect("fragment bytes"))
                .expect("typed fragment");

        assert_eq!(descriptor.module_id, TELEGRAM_ASSEMBLY_MODULE_ID);
        assert_eq!(settings.major, 1);
        assert_eq!(storage.owner_id, TELEGRAM_ASSEMBLY_OWNER_ID);
        assert_eq!(storage.revision, TELEGRAM_STORAGE_BUNDLE_REVISION_V10);
        assert_eq!(
            storage
                .steps
                .iter()
                .map(|step| step.migration_id.as_str())
                .collect::<Vec<_>>(),
            [
                "telegram_state_initial",
                "telegram_automation_management_preview",
                "telegram_call_history",
                "telegram_call_signaling",
                "telegram_call_media_projection",
                "telegram_call_realtime_backfill_job",
                "telegram_delivery_route_locators",
                "telegram_delivery_intent_inbox_jobs_and_result_outbox",
                "telegram_call_evidence_outbox",
                "telegram_owner_scope_and_rls"
            ]
        );
        assert_eq!(
            fragment
                .artifacts
                .iter()
                .map(TelegramReleaseArtifactInputV1::artifact_id)
                .collect::<Vec<_>>(),
            vec![
                TELEGRAM_RUNTIME_ARTIFACT_ID,
                TELEGRAM_STORAGE_ARTIFACT_ID,
                TELEGRAM_TDJSON_ARTIFACT_ID,
                TELEGRAM_TGCALLS_ARTIFACT_ID,
            ]
        );
        assert_eq!(fragment.owner_id, TELEGRAM_ASSEMBLY_OWNER_ID);
        assert_eq!(fragment.module_id, TELEGRAM_ASSEMBLY_MODULE_ID);
        assert!(matches!(
            &fragment.artifacts[2],
            TelegramReleaseArtifactInputV1::NativeDependency(value)
                if value.bound_module_id == TELEGRAM_ASSEMBLY_MODULE_ID
        ));
        assert!(matches!(
            &fragment.artifacts[3],
            TelegramReleaseArtifactInputV1::NativeDependency(value)
                if value.bound_module_id == TELEGRAM_ASSEMBLY_MODULE_ID
        ));

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn canonical_contract_bytes_are_deterministic_and_output_never_overwrites() {
        let root = temporary_directory();
        let runtime = root.join("runtime");
        let tdjson = root.join("libtdjson.dylib");
        let tgcalls = root.join("libmakosh_tgcalls_bridge.dylib");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        fs::write(&tdjson, b"tdjson").expect("TDJson fixture");
        fs::write(&tgcalls, b"tgcalls").expect("tgcalls fixture");
        let first = materialize_telegram_release_assembly_v1(
            &root.join("first"),
            "build-1",
            &runtime,
            &tdjson,
            &tgcalls,
        )
        .expect("first assembly");
        let second = materialize_telegram_release_assembly_v1(
            &root.join("second"),
            "build-1",
            &runtime,
            &tdjson,
            &tgcalls,
        )
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
            materialize_telegram_release_assembly_v1(
                &root.join("first"),
                "build-1",
                &runtime,
                &tdjson,
                &tgcalls,
            ),
            Err(TelegramReleaseAssemblyErrorV1::InvalidInput)
        );

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_relative_missing_and_symlinked_release_sources() {
        use std::os::unix::fs::symlink;

        let root = temporary_directory();
        let runtime = root.join("runtime");
        let tdjson = root.join("libtdjson.dylib");
        let tgcalls = root.join("libmakosh_tgcalls_bridge.dylib");
        let empty_tdjson = root.join("empty-libtdjson.dylib");
        let runtime_link = root.join("runtime-link");
        fs::write(&runtime, b"runtime").expect("runtime fixture");
        fs::write(&tdjson, b"tdjson").expect("TDJson fixture");
        fs::write(&tgcalls, b"tgcalls").expect("tgcalls fixture");
        fs::write(&empty_tdjson, b"").expect("empty TDJson fixture");
        symlink(&runtime, &runtime_link).expect("runtime symlink");

        assert_eq!(
            materialize_telegram_release_assembly_v1(
                Path::new("relative"),
                "build-1",
                &runtime,
                &tdjson,
                &tgcalls,
            ),
            Err(TelegramReleaseAssemblyErrorV1::InvalidInput)
        );
        assert_eq!(
            materialize_telegram_release_assembly_v1(
                &root.join("missing"),
                "build-1",
                &runtime,
                &root.join("absent"),
                &tgcalls,
            ),
            Err(TelegramReleaseAssemblyErrorV1::InvalidInput)
        );
        assert_eq!(
            materialize_telegram_release_assembly_v1(
                &root.join("symlink"),
                "build-1",
                &runtime_link,
                &tdjson,
                &tgcalls,
            ),
            Err(TelegramReleaseAssemblyErrorV1::InvalidInput)
        );
        assert_eq!(
            materialize_telegram_release_assembly_v1(
                &root.join("empty"),
                "build-1",
                &runtime,
                &empty_tdjson,
                &tgcalls,
            ),
            Err(TelegramReleaseAssemblyErrorV1::InvalidInput)
        );

        fs::remove_dir_all(root).expect("remove fixture");
    }

    fn temporary_directory() -> PathBuf {
        let id = NEXT_TEMPORARY_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "makosh-telegram-assembly-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create fixture root");
        path
    }
}
