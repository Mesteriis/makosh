//! Isolated release, Control Store, and Vault-status fixture for Blob launch.

use std::path::{Path, PathBuf};

use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeResponseV1, GetVaultRuntimeStatusRequestV1,
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedVaultRuntimeControlRequestV1, ManagedVaultRuntimeControlResponseV1, ModuleDescriptorV1,
    ModuleKindV1, SettingsSchemaRefV1, SettingsSchemaV1, VaultRuntimeStateV1, VaultRuntimeStatusV1,
    managed_runtime_control_request_v1::Operation as ManagedOperation,
    managed_runtime_control_response_v1::Result as ManagedResult,
    managed_vault_runtime_control_request_v1::Operation as VaultOperation,
    managed_vault_runtime_control_response_v1::Result as VaultResult,
};
use prost::Message;

use super::super::common::*;
use crate::identity::device::signer::FileDeviceSigner;
use crate::platform::blob::launch;
use crate::platform::managed::signed_bundle::{InstalledSignedBundle, SignedRuntimeArtifact};

const BLOB_ARTIFACT_ID: &str = "platform.blob";

pub(super) struct BlobServiceFixture {
    root: PathBuf,
    runtime_dir: PathBuf,
    release: InstalledSignedBundle,
    vault_child: PathBuf,
}

impl BlobServiceFixture {
    pub(super) fn new() -> Self {
        let root = unique_target_root("makosh-blob-managed-service");
        std::fs::create_dir_all(&root).expect("create fixture directory");
        let runtime_dir = short_runtime_directory();
        std::fs::create_dir_all(&runtime_dir).expect("create short runtime directory");
        let data_dir = root.join("data");
        std::fs::create_dir_all(&data_dir).expect("create Blob data directory");
        std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))
            .expect("private Blob data directory");
        let _ =
            FileDeviceSigner::open_or_create_for_instance(&data_dir).expect("Kernel file signer");
        let descriptor = blob_descriptor();
        let schema = blob_schema();
        let release = InstalledSignedBundle::install(
            &root,
            &[
                SignedRuntimeArtifact::new(BLOB_ARTIFACT_ID, blob_binary(), descriptor)
                    .with_settings_schema(schema),
            ],
        )
        .expect("install signed Blob release");
        Self {
            vault_child: write_vault_child(&root),
            root,
            runtime_dir,
            release,
        }
    }

    pub(super) fn store(&self) -> SqliteControlStore {
        let store = SqliteControlStore::create(&self.root.join("control.sqlite"), "instance-1", 1)
            .expect("create Control Store");
        store
            .record_platform_managed_process_binding(&PlatformManagedProcessBinding::new(
                "vault",
                1,
                "fixture",
                "platform.vault",
                [1; 32],
                [2; 32],
                None,
            ))
            .expect("record Vault binding");
        store
            .record_platform_managed_process_launch(&PlatformManagedProcessLaunch::new(
                "vault", 1, 1, 3, 1,
            ))
            .expect("record Vault launch");
        store
    }

    pub(super) fn release_kernel(&self) -> &Path {
        self.release.kernel()
    }

    pub(super) fn start_vault(&self, supervisor: &ManagedRuntimeSupervisor) {
        let descriptor = vault_descriptor().encode_to_vec();
        let digest: [u8; 32] =
            Sha256::digest(std::fs::read(&self.vault_child).expect("read Vault fixture child"))
                .into();
        let staged = staged_native_artifact::stage(
            &self.vault_child,
            &self.root.join("vault-launch"),
            "vault-fixture",
            &digest,
        )
        .expect("stage Vault fixture child");
        supervisor
            .start(
                "vault".to_owned(),
                staged,
                ManagedRuntimeExpectation::new(
                    "vault",
                    "vault",
                    "vault",
                    3,
                    1,
                    Sha256::digest(descriptor).into(),
                    None,
                ),
                ManagedChildExecutionPolicy::new(1, Duration::from_secs(30))
                    .expect("Vault fixture execution policy"),
            )
            .expect("start Vault fixture child");
    }

    pub(super) fn start_blob(
        &self,
        supervisor: &ManagedRuntimeSupervisor,
        store: &SqliteControlStore,
    ) -> u64 {
        launch::start_from_kernel(
            supervisor,
            store,
            self.release.kernel(),
            &self.root.join("data"),
            &self.runtime_dir,
        )
        .expect("start signed Blob service")
    }
}

impl Drop for BlobServiceFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::remove_dir_all(&self.runtime_dir);
    }
}

fn short_runtime_directory() -> PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::path::PathBuf::from("/tmp").join(format!("makosh-blob-{}-{suffix}", std::process::id()))
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

fn blob_descriptor() -> Vec<u8> {
    let schema = blob_schema();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "blob".to_owned(),
        owner_id: "blob".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "blob-service-test".to_owned(),
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

fn vault_descriptor() -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "vault".to_owned(),
        owner_id: "vault".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "vault-fixture".to_owned(),
        ..Default::default()
    }
}

fn write_vault_child(root: &Path) -> PathBuf {
    let descriptor = vault_descriptor().encode_to_vec();
    let describe = ManagedRuntimeControlRequestV1 {
        operation: Some(ManagedOperation::Describe(
            DescribeManagedRuntimeRequestV1 {
                descriptor_bytes: descriptor,
                settings_schema_bytes: Vec::new(),
            },
        )),
    };
    let status = ManagedVaultRuntimeControlResponseV1 {
        result: Some(VaultResult::Status(VaultRuntimeStatusV1 {
            state: VaultRuntimeStateV1::Ready as i32,
            vault_runtime_generation: 3,
            hpke_public_key_x25519: vec![9; 32],
            blocker_code: String::new(),
        })),
        error_code: String::new(),
    };
    let request = ManagedVaultRuntimeControlRequestV1 {
        operation: Some(VaultOperation::GetStatus(GetVaultRuntimeStatusRequestV1 {})),
    };
    write_child_script(
        root.join("vault-child.sh"),
        &describe.encode_to_vec(),
        &status.encode_to_vec(),
        request.encode_to_vec().len() + 1,
    )
}

fn write_child_script(
    path: PathBuf,
    describe: &[u8],
    status: &[u8],
    request_length: usize,
) -> PathBuf {
    let describe_frame = frame(describe);
    let status_frame = frame(status);
    let response_length = describe_response_length();
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\ndd bs=1 count={response_length} of=/dev/null 2>/dev/null\nwhile dd bs=1 count={request_length} of=/dev/null 2>/dev/null; do\n  printf '{}' >&0\ndone\n",
            shell_binary_literal(&describe_frame),
            shell_binary_literal(&status_frame),
        ),
    )
    .expect("write Vault fixture child");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .expect("make Vault fixture child executable");
    path
}

fn describe_response_length() -> usize {
    let response = ManagedRuntimeControlResponseV1 {
        result: Some(ManagedResult::Describe(DescribeManagedRuntimeResponseV1 {
            registration_id: "vault".to_owned(),
            runtime_generation: 3,
            grant_epoch: 1,
        })),
        error_code: String::new(),
    };
    frame(&response.encode_to_vec()).len()
}

fn frame(bytes: &[u8]) -> Vec<u8> {
    assert!(bytes.len() < 128, "fixture frame stays single-byte");
    [vec![bytes.len() as u8], bytes.to_vec()].concat()
}
