use super::common::*;
use makosh_kernel_control_store::{
    PlatformStorageEndpointV1, PlatformStorageTopology, StorageDeploymentProfileV1,
};
use makosh_runtime_protocol::v1::{
    GetVaultRuntimeStatusRequestV1, ManagedVaultRuntimeControlRequestV1,
    ManagedVaultRuntimeControlResponseV1, VaultRuntimeStateV1, VaultRuntimeStatusV1,
    managed_vault_runtime_control_request_v1::Operation as VaultOperation,
    managed_vault_runtime_control_response_v1::Result as VaultResult,
};
use makosh_storage_protocol::v1::{
    GetStorageRuntimeStatusRequestV1, StorageRuntimeControlRequestV1,
    StorageRuntimeControlResponseV1, StorageRuntimeStateV1, StorageRuntimeStatusV1,
    storage_runtime_control_request_v1::Operation,
    storage_runtime_control_response_v1::Result as ResponseResult,
};

use crate::platform::storage::status::parse_current;

#[test]
fn managed_storage_status_requires_the_exact_live_runtime_generation() {
    let status =
        parse_current(reconciling_response(8, 3), 8, 1, 1, 3).expect("current Storage status");

    assert_eq!(status.runtime_generation(), 8);
    assert_eq!(status.state(), StorageRuntimeStateV1::Reconciling);
    assert_eq!(status.topology_revision(), 1);
    assert!(parse_current(reconciling_response(7, 3), 8, 1, 1, 3).is_err());
    assert!(parse_current(reconciling_response(8, 2), 8, 1, 1, 3).is_err());
}

#[test]
fn managed_storage_status_refuses_failed_or_error_responses() {
    let failed = StorageRuntimeControlResponseV1 {
        result: Some(ResponseResult::Status(StorageRuntimeStatusV1 {
            state: StorageRuntimeStateV1::Failed as i32,
            runtime_generation: 8,
            vault_runtime_generation: 3,
            blocker_code: "postgres_unavailable".to_owned(),
            ..Default::default()
        })),
        error_code: String::new(),
    };
    assert!(parse_current(failed, 8, 1, 1, 3).is_err());

    let error = StorageRuntimeControlResponseV1 {
        result: None,
        error_code: "operation_not_available".to_owned(),
    };
    assert!(parse_current(error, 8, 1, 1, 3).is_err());
}

#[test]
fn kernel_reads_storage_status_only_from_the_current_durable_launch() {
    let fixture = StorageStatusFixture::start();
    let status = fixture.current();

    assert_eq!(status.runtime_generation(), 5);
    assert_eq!(status.state(), StorageRuntimeStateV1::Reconciling);
    fixture.stop();
}

struct StorageStatusFixture {
    root: std::path::PathBuf,
    store: SqliteControlStore,
    supervisor: ManagedRuntimeSupervisor,
    shutdown_requested: Arc<AtomicBool>,
}

impl StorageStatusFixture {
    fn start() -> Self {
        let root = unique_target_root("makosh-managed-storage-status");
        std::fs::create_dir_all(&root).expect("create fixture root");
        let store = configured_store(&root);
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
        start_status_children(&root, &supervisor);
        Self {
            root,
            store,
            supervisor,
            shutdown_requested,
        }
    }

    fn current(&self) -> crate::platform::storage::status::ManagedStorageStatus {
        crate::platform::storage::status::wait_current(&self.store, &self.supervisor.relay_port())
            .expect("current storage status")
    }

    fn stop(self) {
        self.shutdown_requested.store(true, Ordering::Release);
        self.supervisor
            .shutdown()
            .expect("stop managed Storage child");
        std::fs::remove_dir_all(self.root).expect("remove fixture");
    }
}

fn configured_store(root: &std::path::Path) -> SqliteControlStore {
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let binding = PlatformManagedProcessBinding::new(
        "storage",
        1,
        "distribution",
        "artifact",
        [1; 32],
        [2; 32],
        None,
    );
    store
        .record_platform_managed_process_binding(&binding)
        .expect("record binding");
    store
        .record_platform_managed_process_launch(&PlatformManagedProcessLaunch::new(
            "storage", 1, 1, 5, 1,
        ))
        .expect("record launch");
    record_vault_launch(&store);
    store
        .record_platform_storage_topology(&PlatformStorageTopology::new(
            makosh_kernel_control_store::PlatformStorageTopologyInputV1 {
                revision: 1,
                storage_generation: 1,
                storage_instance_id: "storage_main".to_owned(),
                database_id: "makosh".to_owned(),
                deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded,
                postgres_endpoint: endpoint(5_432),
                pgbouncer_endpoint: endpoint(6_432),
                postgres_artifact_sha256: [1; 32],
                pgbouncer_artifact_sha256: [2; 32],
            },
        ))
        .expect("record topology");
    store
}

fn start_status_children(root: &std::path::Path, supervisor: &ManagedRuntimeSupervisor) {
    let (staged, expectation) = status_child(root);
    let (vault_staged, vault_expectation) = vault_status_child(root);
    supervisor
        .start(
            "vault".to_owned(),
            vault_staged,
            vault_expectation,
            ManagedChildExecutionPolicy::new(1, Duration::from_secs(30))
                .expect("managed execution policy"),
        )
        .expect("start managed Vault child");
    supervisor
        .start(
            "storage".to_owned(),
            staged,
            expectation,
            ManagedChildExecutionPolicy::new(1, Duration::from_secs(30))
                .expect("managed execution policy"),
        )
        .expect("start managed Storage child");
}

fn endpoint(port: u16) -> PlatformStorageEndpointV1 {
    PlatformStorageEndpointV1::new("127.0.0.1", port)
}

fn record_vault_launch(store: &SqliteControlStore) {
    store
        .record_platform_managed_process_binding(&PlatformManagedProcessBinding::new(
            "vault",
            1,
            "distribution",
            "artifact",
            [3; 32],
            [4; 32],
            None,
        ))
        .expect("record Vault binding");
    store
        .record_platform_managed_process_launch(&PlatformManagedProcessLaunch::new(
            "vault", 1, 1, 3, 1,
        ))
        .expect("record Vault launch");
}

fn reconciling_response(
    generation: u64,
    vault_runtime_generation: u64,
) -> StorageRuntimeControlResponseV1 {
    StorageRuntimeControlResponseV1 {
        result: Some(ResponseResult::Status(StorageRuntimeStatusV1 {
            state: StorageRuntimeStateV1::Reconciling as i32,
            runtime_generation: generation,
            storage_generation: 1,
            topology_revision: 1,
            vault_runtime_generation,
            ..Default::default()
        })),
        error_code: String::new(),
    }
}

fn status_child(
    root: &std::path::Path,
) -> (
    staged_native_artifact::StagedNativeArtifact,
    ManagedRuntimeExpectation,
) {
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "storage".into(),
        owner_id: "storage".into(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let expectation = ManagedRuntimeExpectation::new(
        "storage",
        "storage-runtime",
        "storage",
        5,
        1,
        Sha256::digest(&descriptor_bytes).into(),
        None,
    );
    let source = root.join("managed-status-child.sh");
    let describe = ManagedRuntimeControlRequestV1 {
        operation: Some(
            makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes,
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    let status_request = StorageRuntimeControlRequestV1 {
        operation: Some(Operation::GetStatus(GetStorageRuntimeStatusRequestV1 {})),
    };
    let response = reconciling_response(5, 3);
    let describe_response_length = frame(&describe_response("storage", 5, 1).encode_to_vec()).len();
    let request_length = frame(&status_request.encode_to_vec()).len();
    std::fs::write(
        &source,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\ndd bs=1 count={describe_response_length} of=/dev/null 2>/dev/null\nwhile true; do dd bs=1 count={request_length} of=/dev/null 2>/dev/null; printf '{}' >&0; done\n",
            shell_binary_literal(&frame(&describe.encode_to_vec())),
            shell_binary_literal(&frame(&response.encode_to_vec())),
        ),
    )
    .expect("write status child");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&source).expect("read status child")).into();
    let staged =
        staged_native_artifact::stage(&source, &root.join("launch"), "status-child", &digest)
            .expect("stage status child");
    (staged, expectation)
}

fn vault_status_child(
    root: &std::path::Path,
) -> (
    staged_native_artifact::StagedNativeArtifact,
    ManagedRuntimeExpectation,
) {
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "vault".into(),
        owner_id: "vault".into(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let expectation = ManagedRuntimeExpectation::new(
        "vault",
        "vault-runtime",
        "vault",
        3,
        1,
        Sha256::digest(&descriptor_bytes).into(),
        None,
    );
    let source = root.join("managed-vault-status-child.sh");
    let describe = ManagedRuntimeControlRequestV1 {
        operation: Some(
            makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes,
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    let describe_response_length = frame(&describe_response("vault", 3, 1).encode_to_vec()).len();
    let request_length = frame(&vault_status_request().encode_to_vec()).len();
    let response = vault_status_response();
    std::fs::write(
        &source,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\ndd bs=1 count={describe_response_length} of=/dev/null 2>/dev/null\ndd bs=1 count={request_length} of=/dev/null 2>/dev/null\nprintf '{}' >&0\nsleep 30\n",
            shell_binary_literal(&frame(&describe.encode_to_vec())),
            shell_binary_literal(&frame(&response.encode_to_vec())),
        ),
    )
    .expect("write Vault status child");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&source).expect("read Vault status child")).into();
    let staged = staged_native_artifact::stage(
        &source,
        &root.join("launch-vault"),
        "vault-status-child",
        &digest,
    )
    .expect("stage Vault status child");
    (staged, expectation)
}

fn vault_status_request() -> ManagedVaultRuntimeControlRequestV1 {
    ManagedVaultRuntimeControlRequestV1 {
        operation: Some(VaultOperation::GetStatus(GetVaultRuntimeStatusRequestV1 {})),
    }
}

fn describe_response(
    registration_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
) -> ManagedRuntimeControlResponseV1 {
    ManagedRuntimeControlResponseV1 {
        result: Some(
            makosh_runtime_protocol::v1::managed_runtime_control_response_v1::Result::Describe(
                makosh_runtime_protocol::v1::DescribeManagedRuntimeResponseV1 {
                    registration_id: registration_id.to_owned(),
                    runtime_generation,
                    grant_epoch,
                },
            ),
        ),
        error_code: String::new(),
    }
}

fn vault_status_response() -> ManagedVaultRuntimeControlResponseV1 {
    ManagedVaultRuntimeControlResponseV1 {
        result: Some(VaultResult::Status(VaultRuntimeStatusV1 {
            state: VaultRuntimeStateV1::Ready as i32,
            vault_runtime_generation: 3,
            hpke_public_key_x25519: vec![3; 32],
            blocker_code: String::new(),
        })),
        error_code: String::new(),
    }
}

fn frame(bytes: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(bytes.len() + 5);
    let mut length = u32::try_from(bytes.len()).expect("bounded status frame");
    while length >= 0x80 {
        frame.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    frame.push(length as u8);
    frame.extend_from_slice(bytes);
    frame
}
