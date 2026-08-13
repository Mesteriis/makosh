//! Exact admission, Storage, signed release and host-route binding for WhatsApp.

use super::*;

use makosh_runtime_protocol::v1::{
    ManagedIntegrationHostBridgeConfigurationV1, ManagedIntegrationRuntimeConfigurationV1,
    SettingValueV1, SettingsSnapshotV1, SettingsValueEntryV1, setting_value_v1::Value,
};
use makosh_whatsapp_api::{
    client_contract::{WHATSAPP_OWNER_ID, WhatsAppClientContractV1},
    host_bridge::HOST_BRIDGE_CONTRACT_NAME,
};
use makosh_whatsapp_assembly::{
    WHATSAPP_STORAGE_BUNDLE_REVISION_V5, whatsapp_storage_bundle_with_owner_rls_v5,
};
use makosh_whatsapp_delivery_intent_contract::WHATSAPP_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1;
use makosh_whatsapp_runtime::{
    admission::{
        WHATSAPP_BLOB_CAPABILITY_ID, WHATSAPP_EVENTS_CAPABILITY_ID, WHATSAPP_STORAGE_CAPABILITY_ID,
        whatsapp_module_descriptor_v1,
    },
    settings::whatsapp_settings_schema_bytes_v1,
};

const WHATSAPP_RELEASE_ARTIFACT_ID: &str = "whatsapp.runtime.v1";
pub(super) const WHATSAPP_ACCOUNT_ID: &str = "whatsapp-account-1";

pub(super) struct AdmittedWhatsAppRuntime {
    registration_id: String,
    capability_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum WhatsAppGrantProfileV1 {
    QueryOnly,
    CommandAndQuery,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum WhatsAppBootstrapOverrideV1 {
    None,
    MissingSettings,
    InvalidSettings,
    MissingStorage,
    StaleStorageFence,
    StaleVaultFence,
    StaleEventFence,
    MissingHostRoute,
    StaleHostFence,
}

#[derive(Clone)]
pub(super) struct StartedWhatsAppRuntime {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    pub(super) host_bridge_socket_path: PathBuf,
    pub(super) route_binding_sha256: [u8; 32],
    capability_ids: Vec<String>,
}

pub(super) fn installed_communications_whatsapp_release(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(
        SignedRuntimeArtifact::new(
            WHATSAPP_RELEASE_ARTIFACT_ID,
            whatsapp_binary(),
            whatsapp_module_descriptor_v1("managed-whatsapp-live").encode_to_vec(),
        )
        .with_settings_schema(whatsapp_settings_schema_bytes_v1()),
    );
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communications and WhatsApp release")
}

pub(super) fn admit_whatsapp_runtime(
    store: &SqliteControlStore,
    grant_profile: WhatsAppGrantProfileV1,
) -> AdmittedWhatsAppRuntime {
    let descriptor = whatsapp_module_descriptor_v1("managed-whatsapp-live");
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact WhatsApp descriptor");
    let capability_ids = granted_capability_ids(grant_profile);
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact WhatsApp capabilities");
    let schema = whatsapp_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            WHATSAPP_RELEASE_ARTIFACT_ID,
            Sha256::digest(
                std::fs::read(whatsapp_binary()).expect("WhatsApp runtime binary bytes"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record WhatsApp release binding");
    let bundle = whatsapp_storage_bundle_with_owner_rls_v5().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                WHATSAPP_OWNER_ID,
                u64::from(WHATSAPP_STORAGE_BUNDLE_REVISION_V5),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record WhatsApp Storage bundle"),
        )
        .expect("persist WhatsApp Storage bundle");
    AdmittedWhatsAppRuntime {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

fn granted_capability_ids(grant_profile: WhatsAppGrantProfileV1) -> Vec<String> {
    let mut capability_ids = vec![WHATSAPP_BLOB_CAPABILITY_ID.to_owned()];
    if matches!(grant_profile, WhatsAppGrantProfileV1::CommandAndQuery) {
        capability_ids.push(WhatsAppClientContractV1::Command.capability_id().to_owned());
    }
    capability_ids.extend([
        WHATSAPP_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1.to_owned(),
        WHATSAPP_EVENTS_CAPABILITY_ID.to_owned(),
        HOST_BRIDGE_CONTRACT_NAME.to_owned(),
    ]);
    if matches!(grant_profile, WhatsAppGrantProfileV1::CommandAndQuery) {
        capability_ids.push(
            WhatsAppClientContractV1::OperationalQuery
                .capability_id()
                .to_owned(),
        );
        capability_ids.push(
            WhatsAppClientContractV1::OperationalRealtime
                .capability_id()
                .to_owned(),
        );
    }
    capability_ids.extend([
        WhatsAppClientContractV1::Query.capability_id().to_owned(),
        WHATSAPP_STORAGE_CAPABILITY_ID.to_owned(),
    ]);
    capability_ids
}

pub(super) fn prepare_whatsapp_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedWhatsAppRuntime,
) -> AdmittedWhatsAppRuntime {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve WhatsApp managed launch");
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let bundle = store
        .platform_storage_bundle(
            WHATSAPP_OWNER_ID,
            u64::from(WHATSAPP_STORAGE_BUNDLE_REVISION_V5),
        )
        .expect("read WhatsApp Storage bundle")
        .expect("WhatsApp Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        &runtime_instance_id,
        runtime_generation,
        WHATSAPP_STORAGE_CAPABILITY_ID,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(WHATSAPP_STORAGE_BUNDLE_REVISION_V5),
            *bundle.digest(),
        )
        .expect("WhatsApp Storage binding issue"),
    )
    .expect("issue WhatsApp Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision WhatsApp Storage binding");
    admitted
}

pub(super) fn start_whatsapp_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedWhatsAppRuntime,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedWhatsAppRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load WhatsApp managed launch reservation");
    launch_reserved_whatsapp_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        admitted,
        WhatsAppBootstrapOverrideV1::None,
        test_stdio_capture_directory,
    )
}

#[allow(clippy::too_many_arguments)]
fn launch_reserved_whatsapp_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedWhatsAppRuntime,
    bootstrap_override: WhatsAppBootstrapOverrideV1,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedWhatsAppRuntime {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(&admitted.registration_id, WHATSAPP_STORAGE_CAPABILITY_ID)
        .expect("read WhatsApp Storage binding")
        .expect("WhatsApp Storage binding");
    let topology =
        crate::platform::storage::topology::current(store).expect("read Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("read live Vault status");
    let mut storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build WhatsApp Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let include_storage = match bootstrap_override {
        WhatsAppBootstrapOverrideV1::MissingStorage => false,
        WhatsAppBootstrapOverrideV1::StaleStorageFence => {
            storage.credential_revision = storage.credential_revision.saturating_add(1);
            true
        }
        WhatsAppBootstrapOverrideV1::StaleVaultFence => {
            storage.vault_runtime_generation = storage.vault_runtime_generation.saturating_add(1);
            true
        }
        _ => true,
    };
    let event_credential_revision = if matches!(
        bootstrap_override,
        WhatsAppBootstrapOverrideV1::StaleEventFence
    ) {
        events.credential_revision().saturating_add(1)
    } else {
        events.credential_revision()
    };
    let configuration = ManagedIntegrationRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: WHATSAPP_OWNER_ID.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: include_storage.then_some(storage),
        event_hub_endpoint: events.nats_endpoint().to_owned(),
        event_credential_revision,
        configuration_instance_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        runtime_artifacts: Vec::new(),
        integration_state_root: None,
        configuration_instances: Vec::new(),
        logical_human_owner_id: "owner-1".to_owned(),
    };
    let mut host_bridge_configuration =
        whatsapp_host_bridge_configuration(runtime_dir, store, &reservation);
    if matches!(
        bootstrap_override,
        WhatsAppBootstrapOverrideV1::StaleHostFence
    ) {
        host_bridge_configuration.grant_epoch =
            host_bridge_configuration.grant_epoch.saturating_add(1);
    }
    let host_bridge_socket_path = PathBuf::from(&host_bridge_configuration.socket_path);
    let route_binding_sha256 = host_bridge_configuration
        .route_binding_sha256
        .as_slice()
        .try_into()
        .expect("exact WhatsApp host route binding");
    if let Some(directory) = test_stdio_capture_directory {
        unsafe {
            std::env::set_var(
                crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
                directory,
            );
        }
    }
    let mut settings_snapshot = whatsapp_settings_snapshot();
    if matches!(
        bootstrap_override,
        WhatsAppBootstrapOverrideV1::InvalidSettings
    ) {
        settings_snapshot.values[0]
            .value
            .as_mut()
            .expect("WhatsApp account setting")
            .value = Some(Value::StringValue(" ".to_owned()));
    }
    let launch = managed_launch::ManagedIntegrationLaunchConfiguration {
        runtime: configuration,
        settings_snapshot_bytes: if matches!(
            bootstrap_override,
            WhatsAppBootstrapOverrideV1::MissingSettings
        ) {
            Vec::new()
        } else {
            settings_snapshot.encode_to_vec()
        },
        granted_capability_ids: &admitted.capability_ids,
    };
    let started = if matches!(
        bootstrap_override,
        WhatsAppBootstrapOverrideV1::MissingHostRoute
    ) {
        managed_launch::start_reserved_integration(
            supervisor,
            kernel_data,
            runtime_dir,
            reservation,
            launch,
        )
    } else {
        managed_launch::start_staged_with_host_bridge_configuration(
            supervisor,
            kernel_data,
            runtime_dir,
            reservation,
            launch,
            host_bridge_configuration,
        )
    };
    if let Some(directory) = test_stdio_capture_directory {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while std::fs::read_dir(directory)
            .map(|entries| entries.flatten().count())
            .unwrap_or(0)
            < 2
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        unsafe {
            std::env::remove_var(
                crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
            );
        }
    }
    if matches!(
        bootstrap_override,
        WhatsAppBootstrapOverrideV1::MissingSettings
            | WhatsAppBootstrapOverrideV1::MissingStorage
            | WhatsAppBootstrapOverrideV1::StaleHostFence
    ) {
        assert!(
            started.is_err(),
            "Kernel must deny incomplete or stale WhatsApp bootstrap: {bootstrap_override:?}"
        );
    } else {
        started.expect("start managed WhatsApp integration");
    }
    StartedWhatsAppRuntime {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        host_bridge_socket_path,
        route_binding_sha256,
        capability_ids: admitted.capability_ids,
    }
}

pub(super) fn launch_whatsapp_successor_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: &StartedWhatsAppRuntime,
    bootstrap_override: WhatsAppBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedWhatsAppRuntime {
    let predecessor_binding = store
        .platform_storage_binding(&predecessor.registration_id, WHATSAPP_STORAGE_CAPABILITY_ID)
        .expect("read predecessor WhatsApp Storage binding")
        .expect("predecessor WhatsApp Storage binding");
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive WhatsApp successor storage fences");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        WHATSAPP_STORAGE_CAPABILITY_ID,
        issue,
    )
    .expect("reserve successor WhatsApp launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor WhatsApp Storage binding");
    launch_reserved_whatsapp_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        AdmittedWhatsAppRuntime {
            registration_id: predecessor.registration_id.clone(),
            capability_ids: predecessor.capability_ids.clone(),
        },
        bootstrap_override,
        Some(test_stdio_capture_directory),
    )
}

pub(super) fn restart_whatsapp_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: &StartedWhatsAppRuntime,
) -> StartedWhatsAppRuntime {
    let predecessor_generation = predecessor.runtime_generation;
    let capture = private_directory(runtime_dir.join("whatsapp-restart-stdio"));
    let successor = launch_whatsapp_successor_without_ready_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        predecessor,
        WhatsAppBootstrapOverrideV1::None,
        &capture,
    );
    assert_eq!(
        successor.runtime_generation,
        predecessor_generation + 1,
        "WhatsApp restart must use the next managed runtime generation",
    );
    successor
}

pub(super) fn captured_whatsapp_exit_category_v1(directory: &Path) -> String {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return "capture-unavailable".to_owned();
    };
    let mut file_count = 0usize;
    let mut nonempty_count = 0usize;
    let mut byte_count = 0usize;
    for entry in entries.flatten() {
        let Ok(bytes) = std::fs::read(entry.path()) else {
            continue;
        };
        file_count += 1;
        nonempty_count += usize::from(!bytes.is_empty());
        byte_count = byte_count.saturating_add(bytes.len());
        let output = String::from_utf8_lossy(&bytes);
        for (needle, category) in [
            ("rejected: Admission", "admission"),
            ("rejected: Control", "control"),
            ("rejected: HostBridge", "host-bridge"),
            ("rejected: Storage", "storage"),
            ("rejected: Credential", "credential"),
            ("rejected: Persistence", "persistence"),
            ("rejected: EventHub", "event-hub"),
            ("settings schema is invalid", "settings-schema"),
            ("settings snapshot is invalid", "settings-snapshot"),
            ("runtime configuration is invalid", "runtime-configuration"),
            ("runtime configuration is stale", "stale-configuration"),
            (
                "host bridge configuration is invalid",
                "host-bridge-configuration",
            ),
            (
                "host bridge listener is unavailable",
                "host-bridge-listener",
            ),
            ("client delivery failed", "client-delivery"),
            ("delivery-intent consume failed", "delivery-intent-consume"),
            ("delivery-intent worker failed", "delivery-intent-worker"),
            ("host bridge delivery failed", "host-bridge-delivery"),
            ("outbox persistence failed", "outbox-persistence"),
        ] {
            if output.contains(needle) {
                return format!(
                    "{category}; files={file_count}; nonempty={nonempty_count}; bytes={byte_count}"
                );
            }
        }
    }
    format!("unclassified; files={file_count}; nonempty={nonempty_count}; bytes={byte_count}")
}

fn whatsapp_host_bridge_configuration(
    runtime_dir: &Path,
    store: &SqliteControlStore,
    reservation: &managed_launch::ManagedLaunchReservation,
) -> ManagedIntegrationHostBridgeConfigurationV1 {
    let route_directory = private_directory(runtime_dir.join("host-bridges"));
    let socket_path = route_directory.join(format!(
        "wa-{}-{}.sock",
        reservation.runtime_generation(),
        reservation.grant_epoch(),
    ));
    let socket_path = socket_path
        .to_str()
        .filter(|path| path.len() <= 100)
        .expect("bounded UTF-8 WhatsApp host socket path")
        .to_owned();
    assert!(
        std::fs::symlink_metadata(&socket_path).is_err(),
        "WhatsApp host socket path starts absent",
    );
    let kernel_instance_id = store.snapshot().instance_id();
    let mut binding = Sha256::new();
    for field in [
        kernel_instance_id,
        WHATSAPP_OWNER_ID,
        reservation.registration_id(),
        reservation.runtime_instance_id(),
        socket_path.as_str(),
    ] {
        binding.update(field.as_bytes());
        binding.update([0]);
    }
    binding.update(reservation.runtime_generation().to_be_bytes());
    binding.update(reservation.grant_epoch().to_be_bytes());
    ManagedIntegrationHostBridgeConfigurationV1 {
        major: 1,
        kernel_instance_id: kernel_instance_id.to_owned(),
        owner_id: WHATSAPP_OWNER_ID.to_owned(),
        registration_id: reservation.registration_id().to_owned(),
        runtime_instance_id: reservation.runtime_instance_id().to_owned(),
        runtime_generation: reservation.runtime_generation(),
        grant_epoch: reservation.grant_epoch(),
        socket_path,
        route_binding_sha256: binding.finalize().to_vec(),
    }
}

fn whatsapp_settings_snapshot() -> SettingsSnapshotV1 {
    SettingsSnapshotV1 {
        target_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        revision: 1,
        values: vec![SettingsValueEntryV1 {
            setting_id: "whatsapp.account_id".to_owned(),
            value: Some(SettingValueV1 {
                value: Some(Value::StringValue(WHATSAPP_ACCOUNT_ID.to_owned())),
            }),
        }],
    }
}

fn whatsapp_binary() -> PathBuf {
    binary("MAKOSH_WHATSAPP_RUNTIME_BIN")
}
