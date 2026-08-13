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

struct KernelStorageCredentialObserverRouteV1 {
    handler: KernelManagedVaultRouteHandler,
    storage_expectation: ManagedRuntimeExpectation,
}

impl crate::vault::StorageVaultRoutePortV1 for KernelStorageCredentialObserverRouteV1 {
    #[allow(clippy::manual_async_fn)]
    fn route_vault_ciphertext(
        &mut self,
        route: makosh_runtime_protocol::v1::VaultCiphertextRouteV1,
    ) -> impl std::future::Future<
        Output = Result<
            makosh_runtime_protocol::v1::VaultCiphertextResponseV1,
            crate::vault::StorageVaultRouteFailureV1,
        >,
    > + Send {
        async move {
            self.handler
                .route_vault_ciphertext(&self.storage_expectation, route)
                .map_err(|_| crate::vault::StorageVaultRouteFailureV1::Rejected)
        }
    }
}

pub(super) fn runtime_storage_credential_for_registration_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    data: &Path,
    registration_id: &str,
    storage_capability_id: &str,
) -> zeroize::Zeroizing<Vec<u8>> {
    let platform_binding = store
        .platform_storage_binding(registration_id, storage_capability_id)
        .expect("read target Storage binding")
        .filter(|binding| {
            binding.state() == makosh_kernel_control_store::PlatformStorageBindingStateV1::Active
        })
        .expect("active target Storage binding");
    let topology = crate::platform::storage::topology::current(store)
        .expect("current target Storage topology");
    let runtime_topology = crate::platform::storage::topology::to_runtime(&topology)
        .expect("typed target Storage topology");
    let binding_message = crate::platform::storage::topology::to_runtime_binding(
        &runtime_topology,
        &platform_binding,
    )
    .expect("typed target Storage binding");
    let binding =
        makosh_storage_protocol::validation::storage_binding_from_message(&binding_message)
            .expect("validated target Storage binding");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("current Vault route context");
    let context = crate::vault::StorageVaultRouteContextV1::new(
        store.snapshot().instance_id().to_owned(),
        vault.runtime_generation(),
        *vault.hpke_public_key_x25519(),
    )
    .expect("typed Vault route context");
    let storage_binding = store
        .platform_managed_process_binding(crate::platform::storage::binding::STORAGE_PROCESS_ID)
        .expect("read Storage process binding")
        .expect("bound Storage process");
    let storage_launch = store
        .platform_managed_process_launch(crate::platform::storage::binding::STORAGE_PROCESS_ID)
        .expect("read Storage process launch")
        .expect("launched Storage process");
    let storage_expectation = ManagedRuntimeExpectation::from_platform_fenced_launch(
        crate::platform::storage::binding::STORAGE_PROCESS_ID,
        "storage",
        &storage_binding,
        &storage_launch,
    )
    .expect("current Storage process expectation");
    let route = KernelStorageCredentialObserverRouteV1 {
        handler: KernelManagedVaultRouteHandler::new(
            Arc::clone(store),
            data,
            Arc::new(supervisor.relay_port()),
        ),
        storage_expectation,
    };
    let mut adapter = crate::vault::StorageVaultLeaseAdapterV1::new(route, context);
    let lease_id = crate::vault::complete_immediately(adapter.issue_runtime_credential(&binding))
        .expect("synchronous Vault credential observer")
        .expect("issue exact target runtime credential lease");
    let credential =
        crate::vault::complete_immediately(adapter.resolve_runtime_credential(&binding, lease_id))
            .expect("synchronous Vault credential observer")
            .expect("resolve exact target runtime Storage credential");
    assert_eq!(
        credential.len(),
        64,
        "runtime credential has canonical length"
    );
    credential
}

pub(super) async fn authenticated_storage_admin_pool_v1() -> sqlx::PgPool {
    let password = zeroize::Zeroizing::new(
        std::fs::read_to_string(required(
            "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .expect("read disposable PostgreSQL credential")
        .trim()
        .to_owned(),
    );
    let options = sqlx::postgres::PgConnectOptions::new()
        .host(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
        .port(
            required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
                .parse()
                .expect("valid PostgreSQL port"),
        )
        .username("makosh_postgres_admin")
        .password(password.as_str())
        .database("makosh_storage_authenticated")
        .ssl_mode(sqlx::postgres::PgSslMode::Disable);
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect authenticated Storage conformance database")
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
