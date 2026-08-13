//! Exact signed Search, Timeline and Graph engine admission over shared Storage/Vault/Event Hub.

use super::*;

use makosh_consistency_api::{
    CONSISTENCY_MODULE_ID_V1, CONSISTENCY_OWNER_ID_V1, CONSISTENCY_STORAGE_CAPABILITY_ID_V1,
};
use makosh_consistency_persistence::consistency_storage_bundle_v1;
use makosh_consistency_runtime::{
    consistency_module_descriptor_v1, consistency_settings_schema_bytes_v1,
};
use makosh_graph_api::{GRAPH_MODULE_ID_V1, GRAPH_OWNER_ID_V1, GRAPH_STORAGE_CAPABILITY_ID_V1};
use makosh_graph_persistence::graph_storage_bundle_v1;
use makosh_graph_runtime::{graph_module_descriptor_v1, graph_settings_schema_bytes_v1};
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_memory_api::{MEMORY_MODULE_ID_V1, MEMORY_OWNER_ID_V1, MEMORY_STORAGE_CAPABILITY_ID_V1};
use makosh_memory_persistence::memory_storage_bundle_v1;
use makosh_memory_runtime::{memory_module_descriptor_v1, memory_settings_schema_bytes_v1};
use makosh_risk_api::{RISK_MODULE_ID_V1, RISK_OWNER_ID_V1, RISK_STORAGE_CAPABILITY_ID_V1};
use makosh_risk_persistence::risk_storage_bundle_v1;
use makosh_risk_runtime::{risk_module_descriptor_v1, risk_settings_schema_bytes_v1};
use makosh_runtime_protocol::v1::{
    ManagedEngineRuntimeConfigurationV1, ModuleDescriptorV1, SettingsSnapshotV1,
};
use makosh_search_api::{SEARCH_MODULE_ID_V1, SEARCH_OWNER_ID_V1, SEARCH_STORAGE_CAPABILITY_ID_V1};
use makosh_search_persistence::search_storage_bundle_v1;
use makosh_search_runtime::{search_module_descriptor_v1, search_settings_schema_bytes_v1};
use makosh_storage_protocol::v1::StorageBundleV1;
use makosh_timeline_api::{
    TIMELINE_MODULE_ID_V1, TIMELINE_OWNER_ID_V1, TIMELINE_STORAGE_CAPABILITY_ID_V1,
};
use makosh_timeline_persistence::timeline_storage_bundle_v1;
use makosh_timeline_runtime::{timeline_module_descriptor_v1, timeline_settings_schema_bytes_v1};

pub(super) const PROJECTION_HUMAN_OWNER_V1: &str = "owner-1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProjectionKindV1 {
    Search,
    Timeline,
    Graph,
    Memory,
    Consistency,
    Risk,
}

impl ProjectionKindV1 {
    fn owner_id(self) -> &'static str {
        match self {
            Self::Search => SEARCH_OWNER_ID_V1,
            Self::Timeline => TIMELINE_OWNER_ID_V1,
            Self::Graph => GRAPH_OWNER_ID_V1,
            Self::Memory => MEMORY_OWNER_ID_V1,
            Self::Consistency => CONSISTENCY_OWNER_ID_V1,
            Self::Risk => RISK_OWNER_ID_V1,
        }
    }

    fn module_id(self) -> &'static str {
        match self {
            Self::Search => SEARCH_MODULE_ID_V1,
            Self::Timeline => TIMELINE_MODULE_ID_V1,
            Self::Graph => GRAPH_MODULE_ID_V1,
            Self::Memory => MEMORY_MODULE_ID_V1,
            Self::Consistency => CONSISTENCY_MODULE_ID_V1,
            Self::Risk => RISK_MODULE_ID_V1,
        }
    }

    fn storage_capability_id(self) -> &'static str {
        match self {
            Self::Search => SEARCH_STORAGE_CAPABILITY_ID_V1,
            Self::Timeline => TIMELINE_STORAGE_CAPABILITY_ID_V1,
            Self::Graph => GRAPH_STORAGE_CAPABILITY_ID_V1,
            Self::Memory => MEMORY_STORAGE_CAPABILITY_ID_V1,
            Self::Consistency => CONSISTENCY_STORAGE_CAPABILITY_ID_V1,
            Self::Risk => RISK_STORAGE_CAPABILITY_ID_V1,
        }
    }

    fn artifact_id(self) -> &'static str {
        match self {
            Self::Search => "search.runtime.v1",
            Self::Timeline => "timeline.runtime.v1",
            Self::Graph => "graph.runtime.v1",
            Self::Memory => "memory.runtime.v1",
            Self::Consistency => "consistency.runtime.v1",
            Self::Risk => "risk.runtime.v1",
        }
    }

    fn binary_env(self) -> &'static str {
        match self {
            Self::Search => "MAKOSH_SEARCH_RUNTIME_BIN",
            Self::Timeline => "MAKOSH_TIMELINE_RUNTIME_BIN",
            Self::Graph => "MAKOSH_GRAPH_RUNTIME_BIN",
            Self::Memory => "MAKOSH_MEMORY_RUNTIME_BIN",
            Self::Consistency => "MAKOSH_CONSISTENCY_RUNTIME_BIN",
            Self::Risk => "MAKOSH_RISK_RUNTIME_BIN",
        }
    }

    fn descriptor(self) -> ModuleDescriptorV1 {
        match self {
            Self::Search => search_module_descriptor_v1("managed-search-live"),
            Self::Timeline => timeline_module_descriptor_v1("managed-timeline-live"),
            Self::Graph => graph_module_descriptor_v1("managed-graph-live"),
            Self::Memory => memory_module_descriptor_v1("managed-memory-live"),
            Self::Consistency => consistency_module_descriptor_v1("managed-consistency-live"),
            Self::Risk => risk_module_descriptor_v1("managed-risk-live"),
        }
    }

    fn settings(self) -> Vec<u8> {
        match self {
            Self::Search => search_settings_schema_bytes_v1(),
            Self::Timeline => timeline_settings_schema_bytes_v1(),
            Self::Graph => graph_settings_schema_bytes_v1(),
            Self::Memory => memory_settings_schema_bytes_v1(),
            Self::Consistency => consistency_settings_schema_bytes_v1(),
            Self::Risk => risk_settings_schema_bytes_v1(),
        }
    }

    fn storage_bundle(self) -> StorageBundleV1 {
        match self {
            Self::Search => search_storage_bundle_v1(),
            Self::Timeline => timeline_storage_bundle_v1(),
            Self::Graph => graph_storage_bundle_v1(),
            Self::Memory => memory_storage_bundle_v1(),
            Self::Consistency => consistency_storage_bundle_v1(),
            Self::Risk => risk_storage_bundle_v1(),
        }
    }
}

pub(super) struct AdmittedProjectionV1 {
    kind: ProjectionKindV1,
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedProjectionV1 {
    pub(super) kind: ProjectionKindV1,
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_projection_release_v1(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = vec![
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
    ];
    for kind in [
        ProjectionKindV1::Search,
        ProjectionKindV1::Timeline,
        ProjectionKindV1::Graph,
        ProjectionKindV1::Memory,
        ProjectionKindV1::Consistency,
        ProjectionKindV1::Risk,
    ] {
        artifacts.push(
            SignedRuntimeArtifact::new(
                kind.artifact_id(),
                binary(kind.binary_env()),
                kind.descriptor().encode_to_vec(),
            )
            .with_settings_schema(kind.settings()),
        );
    }
    InstalledSignedBundle::install(root, &artifacts).expect("install signed projection release")
}

pub(super) fn admit_projection_v1(
    store: &SqliteControlStore,
    kind: ProjectionKindV1,
) -> AdmittedProjectionV1 {
    let descriptor = kind.descriptor();
    assert_eq!(descriptor.module_id, kind.module_id());
    assert_eq!(descriptor.owner_id, kind.owner_id());
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register projection descriptor");
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
    .expect("approve projection capabilities");
    let settings = kind.settings();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            kind.artifact_id(),
            Sha256::digest(std::fs::read(binary(kind.binary_env())).expect("projection binary"))
                .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record projection release binding");
    let bundle = kind.storage_bundle().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                kind.owner_id(),
                1,
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("projection storage bundle"),
        )
        .expect("persist projection storage bundle");
    AdmittedProjectionV1 {
        kind,
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_projection_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedProjectionV1,
) -> AdmittedProjectionV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve projection launch");
    let bundle = store
        .platform_storage_bundle(admitted.kind.owner_id(), 1)
        .expect("read projection bundle")
        .expect("projection bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        admitted.kind.storage_capability_id(),
        StorageBindingIssueV1::new(1, 1, 1, *bundle.digest())
            .expect("projection storage binding issue"),
    )
    .expect("issue projection storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision projection storage binding");
    admitted
}

pub(super) fn start_projection_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedProjectionV1,
) -> StartedProjectionV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load projection reservation");
    start_reserved_projection_v1(supervisor, store, runtime_dir, reservation, admitted)
}

pub(super) fn restart_projection_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedProjectionV1,
) -> StartedProjectionV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding =
        projection_storage_binding_v1(store, &predecessor.registration_id, predecessor.kind);
    let issue = storage_successor::issue_after(&binding).expect("derive projection successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        predecessor.kind.storage_capability_id(),
        issue,
    )
    .expect("reserve projection successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision projection successor binding");
    let successor = start_reserved_projection_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedProjectionV1 {
            kind: predecessor.kind,
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_projection_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedProjectionV1,
) -> StartedProjectionV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = projection_storage_binding_v1(store, &admitted.registration_id, admitted.kind);
    let topology =
        crate::platform::storage::topology::current(store).expect("projection Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("projection Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    managed_launch::start_reserved_engine(
        supervisor,
        runtime_dir,
        reservation,
        ManagedEngineRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: admitted.kind.owner_id().to_owned(),
            logical_human_owner_id: PROJECTION_HUMAN_OWNER_V1.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id: runtime_instance_id.clone(),
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
            settings_revision: 1,
            runtime_artifacts: Vec::new(),
        },
        SettingsSnapshotV1 {
            target_id: admitted.registration_id.clone(),
            revision: 1,
            values: Vec::new(),
        }
        .encode_to_vec(),
        &admitted.capability_ids,
    )
    .expect("start managed projection engine");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "projection readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedProjectionV1 {
        kind: admitted.kind,
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn projection_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
    kind: ProjectionKindV1,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, kind.storage_capability_id())
        .expect("read projection storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active projection storage binding")
}
