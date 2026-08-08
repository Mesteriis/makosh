//! Signed admission and owner-local lifecycle for the reviewed Task candidate chain.

use super::*;

use makosh_communication_task_candidate_api::{
    COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1, COMMUNICATION_TASK_CANDIDATE_OWNER_V1,
};
use makosh_communication_task_candidate_persistence::{
    COMMUNICATION_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
    communication_task_candidate_extraction_storage_bundle_v1,
};
use makosh_communication_task_candidate_runtime::{
    COMMUNICATION_TASK_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
    communication_task_candidate_module_descriptor_v1,
    communication_task_candidate_settings_schema_bytes_v1,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_review_task_candidate_api::{
    REVIEW_TASK_CANDIDATE_MODULE_ID_V1, REVIEW_TASK_CANDIDATE_OWNER_V1,
};
use makosh_review_task_candidate_persistence::{
    REVIEW_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V1, review_task_candidate_storage_bundle_v1,
};
use makosh_review_task_candidate_runtime::{
    REVIEW_TASK_CANDIDATE_STORAGE_CAPABILITY_ID_V1, review_task_candidate_module_descriptor_v1,
    review_task_candidate_settings_schema_bytes_v1,
};
use makosh_reviewed_task_candidate_promotion_core::{
    REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1, REVIEWED_TASK_CANDIDATE_PROMOTION_OWNER_V1,
};
use makosh_reviewed_task_candidate_promotion_persistence::schema::{
    REVIEWED_TASK_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V1,
    reviewed_task_candidate_promotion_storage_bundle_v1,
};
use makosh_reviewed_task_candidate_promotion_runtime::{
    REVIEWED_TASK_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
    reviewed_task_candidate_promotion_module_descriptor_v1,
    reviewed_task_candidate_promotion_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::v1::{
    ManagedWorkflowRuntimeConfigurationV1, ModuleDescriptorV1, ModuleKindV1,
};
use makosh_storage_protocol::v1::StorageBundleV1;
use makosh_tasks_command_api::{TASKS_MODULE_ID_V1, TASKS_OWNER_ID_V1};
use makosh_tasks_persistence::{TASKS_STORAGE_BUNDLE_REVISION_V1, tasks_storage_bundle_v1};
use makosh_tasks_runtime::{
    TASKS_STORAGE_CAPABILITY_ID_V1, tasks_module_descriptor_v1, tasks_settings_schema_bytes_v1,
};

const TASK_CANDIDATE_BUILD_ID_V1: &str = "managed-reviewed-task-candidate-live";
pub(super) const TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1: &str = "owner-1";

#[derive(Clone)]
pub(super) struct TaskCandidateManagedUnitV1 {
    label: &'static str,
    artifact_id: &'static str,
    binary_environment: &'static str,
    storage_capability_id: &'static str,
    storage_revision: u32,
    descriptor: ModuleDescriptorV1,
    settings: Vec<u8>,
    storage_bundle: StorageBundleV1,
}

pub(super) struct AdmittedTaskCandidateRuntimeV1 {
    unit: TaskCandidateManagedUnitV1,
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedTaskCandidateRuntimeV1 {
    pub(super) module_id: String,
    pub(super) owner_id: String,
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    unit: TaskCandidateManagedUnitV1,
    capability_ids: Vec<String>,
}

pub(super) fn configure_task_candidate_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(
            crate::platform::client_realtime::ClientRealtimePublishHandlerV1::new(
                Arc::clone(store),
                realtime,
            ),
        ))
        .expect("configure reviewed Task candidate client realtime");
}

pub(super) fn installed_task_candidate_ensemble_release_v1(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.extend(task_candidate_units_v1().into_iter().map(|unit| {
        SignedRuntimeArtifact::new(
            unit.artifact_id,
            binary(unit.binary_environment),
            unit.descriptor.encode_to_vec(),
        )
        .with_settings_schema(unit.settings)
    }));
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed reviewed Task candidate ensemble")
}

pub(super) fn admit_task_candidate_ensemble_v1(
    store: &SqliteControlStore,
) -> Vec<AdmittedTaskCandidateRuntimeV1> {
    task_candidate_units_v1()
        .into_iter()
        .map(|unit| admit_task_candidate_unit_v1(store, unit))
        .collect()
}

pub(super) fn prepare_task_candidate_ensemble_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: Vec<AdmittedTaskCandidateRuntimeV1>,
) -> Vec<AdmittedTaskCandidateRuntimeV1> {
    admitted
        .into_iter()
        .map(|unit| prepare_task_candidate_unit_v1(supervisor, store, unit))
        .collect()
}

pub(super) fn start_task_candidate_ensemble_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: Vec<AdmittedTaskCandidateRuntimeV1>,
) -> Vec<StartedTaskCandidateRuntimeV1> {
    admitted
        .into_iter()
        .map(|unit| start_task_candidate_unit_v1(supervisor, store, runtime_dir, unit))
        .collect()
}

fn task_candidate_units_v1() -> [TaskCandidateManagedUnitV1; 4] {
    [
        TaskCandidateManagedUnitV1 {
            label: "Communication Task Candidate",
            artifact_id: "communication_task_candidate_extraction.runtime.v1",
            binary_environment: "MAKOSH_COMMUNICATION_TASK_CANDIDATE_RUNTIME_BIN",
            storage_capability_id: COMMUNICATION_TASK_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
            storage_revision: COMMUNICATION_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
            descriptor: communication_task_candidate_module_descriptor_v1(
                TASK_CANDIDATE_BUILD_ID_V1,
            ),
            settings: communication_task_candidate_settings_schema_bytes_v1(),
            storage_bundle: communication_task_candidate_extraction_storage_bundle_v1(),
        },
        TaskCandidateManagedUnitV1 {
            label: "Review Task Candidate",
            artifact_id: "review.task-candidate.runtime.v1",
            binary_environment: "MAKOSH_REVIEW_TASK_CANDIDATE_RUNTIME_BIN",
            storage_capability_id: REVIEW_TASK_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
            storage_revision: REVIEW_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
            descriptor: review_task_candidate_module_descriptor_v1(TASK_CANDIDATE_BUILD_ID_V1),
            settings: review_task_candidate_settings_schema_bytes_v1(),
            storage_bundle: review_task_candidate_storage_bundle_v1(),
        },
        TaskCandidateManagedUnitV1 {
            label: "Reviewed Task Candidate Promotion",
            artifact_id: "reviewed_task_candidate_promotion.runtime.v1",
            binary_environment: "MAKOSH_REVIEWED_TASK_CANDIDATE_PROMOTION_RUNTIME_BIN",
            storage_capability_id: REVIEWED_TASK_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
            storage_revision: REVIEWED_TASK_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V1,
            descriptor: reviewed_task_candidate_promotion_module_descriptor_v1(
                TASK_CANDIDATE_BUILD_ID_V1,
            ),
            settings: reviewed_task_candidate_promotion_settings_schema_bytes_v1(),
            storage_bundle: reviewed_task_candidate_promotion_storage_bundle_v1(),
        },
        TaskCandidateManagedUnitV1 {
            label: "Tasks",
            artifact_id: "tasks.runtime.v1",
            binary_environment: "MAKOSH_TASKS_RUNTIME_BIN",
            storage_capability_id: TASKS_STORAGE_CAPABILITY_ID_V1,
            storage_revision: TASKS_STORAGE_BUNDLE_REVISION_V1,
            descriptor: tasks_module_descriptor_v1(TASK_CANDIDATE_BUILD_ID_V1),
            settings: tasks_settings_schema_bytes_v1(),
            storage_bundle: tasks_storage_bundle_v1(),
        },
    ]
}

fn admit_task_candidate_unit_v1(
    store: &SqliteControlStore,
    unit: TaskCandidateManagedUnitV1,
) -> AdmittedTaskCandidateRuntimeV1 {
    assert_exact_unit_boundary(&unit);
    let descriptor_bytes = unit.descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .unwrap_or_else(|error| panic!("register exact {} descriptor: {error}", unit.label));
    let capability_ids = unit
        .descriptor
        .capabilities
        .iter()
        .map(|capability| capability.capability_id.clone())
        .collect::<Vec<_>>();
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .unwrap_or_else(|error| panic!("approve exact {} capabilities: {error}", unit.label));
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            unit.artifact_id,
            Sha256::digest(
                std::fs::read(binary(unit.binary_environment))
                    .unwrap_or_else(|error| panic!("read {} runtime: {error}", unit.label)),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&unit.settings).into()),
        ))
        .unwrap_or_else(|error| panic!("record {} release binding: {error:?}", unit.label));
    let bundle = unit.storage_bundle.encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                &unit.descriptor.owner_id,
                u64::from(unit.storage_revision),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .unwrap_or_else(|error| panic!("compose {} Storage bundle: {error}", unit.label)),
        )
        .unwrap_or_else(|error| panic!("persist {} Storage bundle: {error:?}", unit.label));
    AdmittedTaskCandidateRuntimeV1 {
        unit,
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

fn prepare_task_candidate_unit_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedTaskCandidateRuntimeV1,
) -> AdmittedTaskCandidateRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .unwrap_or_else(|error| panic!("reserve {} launch: {error}", admitted.unit.label));
    let bundle = store
        .platform_storage_bundle(
            &admitted.unit.descriptor.owner_id,
            u64::from(admitted.unit.storage_revision),
        )
        .expect("read candidate Storage bundle")
        .expect("candidate Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        admitted.unit.storage_capability_id,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(admitted.unit.storage_revision),
            *bundle.digest(),
        )
        .expect("candidate Storage binding issue"),
    )
    .unwrap_or_else(|error| panic!("issue {} Storage binding: {error}", admitted.unit.label));
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .unwrap_or_else(|error| {
            panic!("provision {} Storage binding: {error}", admitted.unit.label)
        });
    admitted
}

fn start_task_candidate_unit_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedTaskCandidateRuntimeV1,
) -> StartedTaskCandidateRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .unwrap_or_else(|error| panic!("load {} launch reservation: {error}", admitted.unit.label));
    start_reserved_task_candidate_unit_v1(supervisor, store, runtime_dir, admitted, reservation)
}

pub(super) fn restart_task_candidate_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedTaskCandidateRuntimeV1,
) -> StartedTaskCandidateRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = active_task_candidate_storage_binding_v1(
        store,
        &AdmittedTaskCandidateRuntimeV1 {
            unit: predecessor.unit.clone(),
            registration_id: predecessor.registration_id.clone(),
            capability_ids: predecessor.capability_ids.clone(),
        },
    );
    let issue = storage_successor::issue_after(&binding)
        .unwrap_or_else(|error| panic!("derive {} successor: {error}", predecessor.unit.label));
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        predecessor.unit.storage_capability_id,
        issue,
    )
    .unwrap_or_else(|error| panic!("reserve {} successor: {error}", predecessor.unit.label));
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .unwrap_or_else(|error| panic!("provision {} successor: {error}", predecessor.unit.label));
    let successor = start_reserved_task_candidate_unit_v1(
        supervisor,
        store,
        runtime_dir,
        AdmittedTaskCandidateRuntimeV1 {
            unit: predecessor.unit,
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        reservation,
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_task_candidate_unit_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedTaskCandidateRuntimeV1,
    reservation: managed_launch::ManagedLaunchReservation,
) -> StartedTaskCandidateRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = active_task_candidate_storage_binding_v1(store, &admitted);
    let topology = crate::platform::storage::topology::current(store)
        .expect("reviewed Task candidate Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("live Vault status for reviewed Task candidate chain");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("reviewed Task candidate Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    match ModuleKindV1::try_from(admitted.unit.descriptor.module_kind)
        .expect("reviewed Task candidate module kind")
    {
        ModuleKindV1::Workflow => managed_launch::start_reserved_workflow(
            supervisor,
            runtime_dir,
            reservation,
            ManagedWorkflowRuntimeConfigurationV1 {
                major: 1,
                logical_owner_id: TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
                registration_id: admitted.registration_id.clone(),
                runtime_instance_id: runtime_instance_id.clone(),
                runtime_generation,
                grant_epoch,
                storage: Some(storage),
                event_hub_endpoint: events.nats_endpoint().to_owned(),
                event_credential_revision: events.credential_revision(),
                runtime_artifacts: Vec::new(),
                configuration_instance_id: String::new(),
                settings_revision: 0,
                configuration_instances: Vec::new(),
            },
            &[],
        ),
        ModuleKindV1::Domain => managed_launch::start_reserved_domain(
            supervisor,
            runtime_dir,
            reservation,
            ManagedDomainRuntimeConfigurationV1 {
                major: 1,
                logical_owner_id: admitted.unit.descriptor.owner_id.clone(),
                registration_id: admitted.registration_id.clone(),
                runtime_instance_id: runtime_instance_id.clone(),
                runtime_generation,
                grant_epoch,
                storage: Some(storage),
                event_hub_endpoint: events.nats_endpoint().to_owned(),
                event_credential_revision: events.credential_revision(),
                logical_human_owner_id: TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
            },
        ),
        kind => panic!("unsupported reviewed Task candidate module kind: {kind:?}"),
    }
    .unwrap_or_else(|error| panic!("start managed {}: {error}", admitted.unit.label));
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "{} readiness: {error}; last_failure={:?}",
                admitted.unit.label,
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedTaskCandidateRuntimeV1 {
        module_id: admitted.unit.descriptor.module_id.clone(),
        owner_id: admitted.unit.descriptor.owner_id.clone(),
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        unit: admitted.unit,
        capability_ids: admitted.capability_ids,
    }
}

fn active_task_candidate_storage_binding_v1(
    store: &SqliteControlStore,
    admitted: &AdmittedTaskCandidateRuntimeV1,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            &admitted.registration_id,
            admitted.unit.storage_capability_id,
        )
        .expect("read reviewed Task candidate Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active reviewed Task candidate Storage binding")
}

fn assert_exact_unit_boundary(unit: &TaskCandidateManagedUnitV1) {
    match unit.descriptor.module_id.as_str() {
        COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1 => {
            assert_eq!(
                unit.descriptor.owner_id,
                COMMUNICATION_TASK_CANDIDATE_OWNER_V1
            );
            assert_eq!(unit.descriptor.module_kind, ModuleKindV1::Workflow as i32);
        }
        REVIEW_TASK_CANDIDATE_MODULE_ID_V1 => {
            assert_eq!(unit.descriptor.owner_id, REVIEW_TASK_CANDIDATE_OWNER_V1);
            assert_eq!(unit.descriptor.module_kind, ModuleKindV1::Domain as i32);
        }
        REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1 => {
            assert_eq!(
                unit.descriptor.owner_id,
                REVIEWED_TASK_CANDIDATE_PROMOTION_OWNER_V1
            );
            assert_eq!(unit.descriptor.module_kind, ModuleKindV1::Workflow as i32);
        }
        TASKS_MODULE_ID_V1 => {
            assert_eq!(unit.descriptor.owner_id, TASKS_OWNER_ID_V1);
            assert_eq!(unit.descriptor.module_kind, ModuleKindV1::Domain as i32);
        }
        module_id => panic!("unexpected reviewed Task candidate module: {module_id}"),
    }
}
