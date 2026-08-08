//! Exact signed admission, Storage and private host-route lifecycle for desktop recording.

use super::*;

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use makosh_desktop_call_recording_api::{MODULE_ID_V1, OWNER_ID_V1};
use makosh_desktop_call_recording_persistence::{
    STORAGE_BUNDLE_REVISION_V1, desktop_call_recording_storage_bundle_v1,
};
use makosh_desktop_call_recording_runtime::{
    admission::{STORAGE_CAPABILITY_ID_V1, module_descriptor_v1},
    settings::settings_schema_bytes_v1,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::{
    ManagedIntegrationHostBridgeConfigurationV1, ManagedIntegrationRuntimeConfigurationV1,
    SettingsSnapshotV1,
};

const RELEASE_ARTIFACT_ID_V1: &str = "desktop_call_recording.runtime.v1";
const BUILD_ID_V1: &str = "managed-desktop-call-recording-live";
pub(super) const DESKTOP_RECORDING_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedDesktopRecordingRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

#[derive(Clone)]
pub(super) struct StartedDesktopRecordingRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    pub(super) host_bridge_socket_path: PathBuf,
    pub(super) route_binding_sha256: [u8; 32],
    capability_ids: Vec<String>,
}

pub(super) fn desktop_recording_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        RELEASE_ARTIFACT_ID_V1,
        desktop_recording_binary(),
        module_descriptor_v1(BUILD_ID_V1).encode_to_vec(),
    )
    .with_settings_schema(settings_schema_bytes_v1())
}

pub(super) fn installed_desktop_recording_release_v1(root: &Path) -> InstalledSignedBundle {
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
            blob_release_artifact(),
            desktop_recording_release_artifact_v1(),
        ],
    )
    .expect("install signed desktop recording release")
}

pub(super) fn admit_desktop_recording_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedDesktopRecordingRuntimeV1 {
    let descriptor = module_descriptor_v1(BUILD_ID_V1);
    assert_eq!(descriptor.module_id, MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact desktop recording descriptor");
    let capability_ids = descriptor
        .capabilities
        .iter()
        .map(|capability| capability.capability_id.clone())
        .collect::<Vec<_>>();
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact desktop recording capabilities");
    let settings = settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(desktop_recording_binary())
                    .expect("desktop recording runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record desktop recording release binding");
    let bundle = desktop_call_recording_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                OWNER_ID_V1,
                u64::from(STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("desktop recording Storage bundle"),
        )
        .expect("persist desktop recording Storage bundle");
    AdmittedDesktopRecordingRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn configure_desktop_recording_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            realtime,
        )))
        .expect("configure desktop recording client realtime");
}

pub(super) fn prepare_desktop_recording_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedDesktopRecordingRuntimeV1,
) -> AdmittedDesktopRecordingRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve desktop recording launch");
    let bundle = store
        .platform_storage_bundle(OWNER_ID_V1, u64::from(STORAGE_BUNDLE_REVISION_V1))
        .expect("read desktop recording Storage bundle")
        .expect("desktop recording Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("desktop recording Storage binding issue"),
    )
    .expect("issue desktop recording Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision desktop recording Storage binding");
    admitted
}

pub(super) fn start_desktop_recording_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedDesktopRecordingRuntimeV1,
) -> StartedDesktopRecordingRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load desktop recording launch reservation");
    start_reserved_desktop_recording_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        admitted,
    )
}

pub(super) fn restart_desktop_recording_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedDesktopRecordingRuntimeV1,
) -> StartedDesktopRecordingRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = desktop_recording_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding)
        .expect("derive desktop recording Storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve desktop recording successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision desktop recording successor");
    let successor = start_reserved_desktop_recording_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        AdmittedDesktopRecordingRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_desktop_recording_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedDesktopRecordingRuntimeV1,
) -> StartedDesktopRecordingRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = desktop_recording_storage_binding_v1(store, &admitted.registration_id);
    let topology = crate::platform::storage::topology::current(store)
        .expect("desktop recording Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("desktop recording Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let configuration = ManagedIntegrationRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: OWNER_ID_V1.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: Some(storage),
        event_hub_endpoint: events.nats_endpoint().to_owned(),
        event_credential_revision: events.credential_revision(),
        configuration_instance_id: admitted.registration_id.clone(),
        runtime_artifacts: Vec::new(),
        integration_state_root: None,
        configuration_instances: Vec::new(),
        logical_human_owner_id: DESKTOP_RECORDING_LOGICAL_OWNER_ID_V1.to_owned(),
    };
    let host_bridge_configuration =
        desktop_recording_host_bridge_configuration_v1(runtime_dir, store, &reservation);
    let host_bridge_socket_path = PathBuf::from(&host_bridge_configuration.socket_path);
    let route_binding_sha256 = host_bridge_configuration
        .route_binding_sha256
        .as_slice()
        .try_into()
        .expect("exact desktop recording host route binding");
    managed_launch::start_staged_with_host_bridge_configuration(
        supervisor,
        kernel_data,
        runtime_dir,
        reservation,
        managed_launch::ManagedIntegrationLaunchConfiguration {
            runtime: configuration,
            settings_snapshot_bytes: SettingsSnapshotV1 {
                target_id: admitted.registration_id.clone(),
                revision: 1,
                values: Vec::new(),
            }
            .encode_to_vec(),
            granted_capability_ids: &admitted.capability_ids,
        },
        host_bridge_configuration,
    )
    .expect("start managed desktop recording integration");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "desktop recording readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedDesktopRecordingRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        host_bridge_socket_path,
        route_binding_sha256,
        capability_ids: admitted.capability_ids,
    }
}

fn desktop_recording_host_bridge_configuration_v1(
    runtime_dir: &Path,
    store: &SqliteControlStore,
    reservation: &managed_launch::ManagedLaunchReservation,
) -> ManagedIntegrationHostBridgeConfigurationV1 {
    let route_directory = private_directory(runtime_dir.join("host-bridges"));
    let socket_path = route_directory.join(format!(
        "r-{}-{}.sock",
        reservation.runtime_generation(),
        reservation.grant_epoch(),
    ));
    let socket_path = socket_path
        .to_str()
        .filter(|path| path.len() <= 100)
        .expect("bounded UTF-8 desktop recording host socket path")
        .to_owned();
    assert!(
        std::fs::symlink_metadata(&socket_path).is_err(),
        "desktop recording host socket path starts absent",
    );
    let kernel_instance_id = store.snapshot().instance_id();
    let mut binding = Sha256::new();
    for field in [
        kernel_instance_id,
        OWNER_ID_V1,
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
        owner_id: OWNER_ID_V1.to_owned(),
        registration_id: reservation.registration_id().to_owned(),
        runtime_instance_id: reservation.runtime_instance_id().to_owned(),
        runtime_generation: reservation.runtime_generation(),
        grant_epoch: reservation.grant_epoch(),
        socket_path,
        route_binding_sha256: binding.finalize().to_vec(),
    }
}

fn desktop_recording_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, STORAGE_CAPABILITY_ID_V1)
        .expect("read desktop recording Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active desktop recording Storage binding")
}

fn desktop_recording_binary() -> PathBuf {
    binary("MAKOSH_DESKTOP_CALL_RECORDING_RUNTIME_BIN")
}
