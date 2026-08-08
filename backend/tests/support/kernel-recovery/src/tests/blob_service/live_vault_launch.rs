//! Real Vault and Blob binary composition through signed Kernel bindings.

use std::path::{Path, PathBuf};

use makosh_runtime_protocol::v1::{
    ModuleDescriptorV1, ModuleKindV1, SettingsSchemaRefV1, SettingsSchemaV1,
};
use prost::Message;

use super::super::common::*;
use crate::platform::blob::{binding, launch, status};
use crate::platform::managed::signed_bundle::{InstalledSignedBundle, SignedRuntimeArtifact};
use crate::tests::platform_vault::live as vault_fixture;

const BLOB_ARTIFACT_ID: &str = "platform.blob";
const VAULT_ARTIFACT_ID: &str = "platform.vault";

#[test]
#[ignore = "builds and launches the real Blob and Vault runtime binaries"]
fn kernel_starts_signed_blob_service_with_live_file_backed_vault() {
    let root = unique_target_root("makosh-blob-live-vault");
    let data = vault_fixture::private_directory(root.join("kernel"));
    let runtime = short_runtime_directory();
    vault_fixture::initialize_vault(&data);
    let release = installed_release(&root);
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "kernel-main", 1)
            .expect("create Control Store"),
    );
    let supervisor = ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false)));

    vault_fixture::bind_and_start(&supervisor, &store, &data, release.kernel());
    binding::bind_installed_release(&store, release.kernel()).expect("bind signed Blob release");
    let generation =
        launch::start_from_kernel(&supervisor, &store, release.kernel(), &data, &runtime)
            .expect("start signed Blob service");

    assert_eq!(generation, 1);
    assert_eq!(
        status::read_current(&store, &supervisor.relay_port())
            .expect("Blob status")
            .runtime_generation(),
        generation
    );

    supervisor.shutdown().expect("stop managed children");
    std::fs::remove_dir_all(root).expect("remove fixture directory");
    std::fs::remove_dir_all(runtime).expect("remove short runtime directory");
}

pub(super) fn short_runtime_directory() -> PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    PathBuf::from("/tmp").join(format!("makosh-blob-live-{}-{suffix}", std::process::id()))
}

pub(super) fn installed_release(root: &Path) -> InstalledSignedBundle {
    let schema = blob_schema();
    InstalledSignedBundle::install(
        root,
        &[
            SignedRuntimeArtifact::new(
                VAULT_ARTIFACT_ID,
                vault_fixture::vault_binary(),
                vault_fixture::vault_descriptor(),
            ),
            SignedRuntimeArtifact::new(BLOB_ARTIFACT_ID, blob_binary(), blob_descriptor(&schema))
                .with_settings_schema(schema),
        ],
    )
    .expect("install signed platform release")
}

fn blob_binary() -> PathBuf {
    std::env::var_os("MAKOSH_BLOB_SERVICE_BIN")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .expect("MAKOSH_BLOB_SERVICE_BIN must name a regular Blob service binary")
}

fn blob_schema() -> Vec<u8> {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        ..Default::default()
    }
    .encode_to_vec()
}

fn blob_descriptor(schema: &[u8]) -> Vec<u8> {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "blob".to_owned(),
        owner_id: "blob".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "blob-live-vault-test".to_owned(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: schema.len() as u64,
            sha256: Sha256::digest(schema).to_vec(),
        }),
        ..Default::default()
    }
    .encode_to_vec()
}
