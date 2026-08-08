use super::*;

pub(super) fn configured_store(root: &Path, kernel: &Path) -> SqliteControlStore {
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "kernel-main", 1)
        .expect("Control Store");
    vault_binding::bind_installed_release(&store, kernel).expect("bind signed Vault release");
    storage_binding::bind_installed_release(&store, kernel).expect("bind signed Storage release");
    store
        .record_platform_storage_topology(
            &PlatformStorageTopology::new(
                makosh_kernel_control_store::PlatformStorageTopologyInputV1 {
                    revision: 1,
                    storage_generation: 1,
                    storage_instance_id: "storage_main".to_owned(),
                    database_id: "makosh_storage_authenticated".to_owned(),
                    deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded,
                    postgres_endpoint: endpoint(
                        "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST",
                        "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT",
                    ),
                    pgbouncer_endpoint: endpoint(
                        "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_HOST",
                        "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT",
                    ),
                    postgres_artifact_sha256: [1; 32],
                    pgbouncer_artifact_sha256: [2; 32],
                },
            )
            .with_pgbouncer_backend_endpoint(PlatformStorageEndpointV1::new("postgres", 5_432)),
        )
        .expect("record Storage topology");
    store
}

pub(super) fn start_vault(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    data: &Path,
    kernel: &Path,
) -> u64 {
    vault_launch::start_from_kernel(supervisor, store, data, kernel, &data.join("runtime"))
        .expect("start signed Vault")
}

pub(super) fn start_storage(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel: &Path,
    runtime: &Path,
) -> u64 {
    storage_launch::start_from_kernel(supervisor, store, kernel, runtime)
        .expect("start signed Storage")
}

pub(super) fn storage_runtime_directory() -> PathBuf {
    let databases = PathBuf::from(required(
        "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_DATABASES_FILE",
    ));
    let pgbouncer = databases.parent().expect("PgBouncer config parent");
    pgbouncer
        .parent()
        .expect("Storage runtime parent")
        .parent()
        .expect("Storage runtime directory")
        .to_path_buf()
}

pub(super) fn assert_reconciling_status(
    supervisor: &ManagedRuntimeSupervisor,
    expected_generation: u64,
) {
    let request = StorageRuntimeControlRequestV1 {
        operation: Some(Operation::GetStatus(GetStorageRuntimeStatusRequestV1 {})),
    };
    for _ in 0..40 {
        if let Ok(bytes) = supervisor.relay("storage", request.encode_to_vec())
            && let Ok(response) = StorageRuntimeControlResponseV1::decode(bytes.as_slice())
        {
            assert!(
                matches!(response.result, Some(StorageResult::Status(status)) if status.state == StorageRuntimeStateV1::Reconciling as i32 && status.runtime_generation == expected_generation && status.vault_runtime_generation == 1)
            );
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!(
        "managed Storage status is unavailable: {:?}",
        supervisor.last_failure("storage")
    );
}

pub(super) fn initialize_vault(data: &Path, source: &Path) {
    let output = std::process::Command::new(vault_binary())
        .args(["initialize", "--data-dir"])
        .arg(data)
        .args(["--instance-id", "kernel-main", "--platform-credential-dir"])
        .arg(source)
        .output()
        .expect("Vault initializer");
    assert!(output.status.success(), "Vault initialization failed");
}

pub(super) fn credential_directory() -> PathBuf {
    let pgbouncer = PathBuf::from(required(
        "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE",
    ));
    let postgres = PathBuf::from(required(
        "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
    ));
    assert_eq!(pgbouncer.parent(), postgres.parent());
    pgbouncer.parent().expect("credential parent").to_owned()
}

pub(super) fn descriptor(id: &str) -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: id.to_owned(),
        owner_id: id.to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "managed-process-test".to_owned(),
        ..Default::default()
    }
}

pub(super) fn installed_release(root: &Path) -> InstalledSignedBundle {
    InstalledSignedBundle::install(
        root,
        &[
            SignedRuntimeArtifact::new(
                "platform.storage",
                storage_binary(),
                descriptor("storage").encode_to_vec(),
            ),
            SignedRuntimeArtifact::new(
                "platform.vault",
                vault_binary(),
                descriptor("vault").encode_to_vec(),
            ),
        ],
    )
    .expect("install signed managed release")
}

pub(super) fn vault_binary() -> PathBuf {
    binary("MAKOSH_VAULT_RUNTIME_BIN")
}
pub(super) fn storage_binary() -> PathBuf {
    binary("MAKOSH_STORAGE_RUNTIME_BIN")
}
pub(super) fn binary(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| panic!("{name} must name a regular binary"))
}
pub(super) fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("missing {name}"))
}
fn port(name: &str) -> u32 {
    required(name)
        .parse()
        .unwrap_or_else(|_| panic!("invalid {name}"))
}
fn endpoint(host: &str, port_name: &str) -> PlatformStorageEndpointV1 {
    PlatformStorageEndpointV1::new(required(host), port(port_name).try_into().expect("port"))
}
