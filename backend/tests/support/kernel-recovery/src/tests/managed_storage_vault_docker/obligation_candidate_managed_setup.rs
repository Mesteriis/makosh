//! Signed admission and owner-local lifecycle for the reviewed Obligation candidate chain.

use super::*;

use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_obligations_api::{OBLIGATIONS_MODULE_ID_V1, OBLIGATIONS_OWNER_ID_V1};
use makosh_obligations_persistence::{
    OBLIGATIONS_STORAGE_BUNDLE_REVISION_V3, obligations_storage_bundle_v1,
};
use makosh_obligations_runtime::{
    OBLIGATIONS_STORAGE_CAPABILITY_ID_V1, obligations_module_descriptor_v1,
    obligations_settings_schema_bytes_v1,
};
use makosh_review_obligation_candidate_api::{
    REVIEW_OBLIGATION_CANDIDATE_MODULE_ID_V1, REVIEW_OBLIGATION_CANDIDATE_OWNER_V1,
};
use makosh_review_obligation_candidate_persistence::{
    REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V3,
    review_obligation_candidate_storage_bundle_v1,
};
use makosh_review_obligation_candidate_runtime::{
    REVIEW_OBLIGATION_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
    review_obligation_candidate_module_descriptor_v1,
    review_obligation_candidate_settings_schema_bytes_v1,
};
use makosh_reviewed_obligation_candidate_promotion_core::{
    REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_MODULE_ID_V1,
    REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_OWNER_V1,
};
use makosh_reviewed_obligation_candidate_promotion_persistence::schema::{
    REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V2,
    reviewed_obligation_candidate_promotion_storage_bundle_v1,
};
use makosh_reviewed_obligation_candidate_promotion_runtime::{
    REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
    reviewed_obligation_candidate_promotion_module_descriptor_v1,
    reviewed_obligation_candidate_promotion_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::v1::{
    ManagedWorkflowRuntimeConfigurationV1, ModuleDescriptorV1, ModuleKindV1,
};
use makosh_storage_protocol::v1::StorageBundleV1;

const OBLIGATION_CANDIDATE_BUILD_ID_V1: &str = "managed-reviewed-obligation-candidate-live";
pub(super) const OBLIGATION_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1: &str = "owner-1";

#[derive(Clone)]
pub(super) struct ObligationCandidateManagedUnitV1 {
    label: &'static str,
    artifact_id: &'static str,
    binary_environment: &'static str,
    storage_capability_id: &'static str,
    storage_revision: u32,
    descriptor: ModuleDescriptorV1,
    settings: Vec<u8>,
    storage_bundle: StorageBundleV1,
}

pub(super) struct AdmittedObligationCandidateRuntimeV1 {
    unit: ObligationCandidateManagedUnitV1,
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedObligationCandidateRuntimeV1 {
    pub(super) module_id: String,
    pub(super) owner_id: String,
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    unit: ObligationCandidateManagedUnitV1,
    capability_ids: Vec<String>,
}

pub(super) fn configure_obligation_candidate_realtime_v1(
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
        .expect("configure reviewed Obligation candidate client realtime");
}

pub(super) fn installed_obligation_candidate_ensemble_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.extend(obligation_candidate_units_v1().into_iter().map(|unit| {
        SignedRuntimeArtifact::new(
            unit.artifact_id,
            binary(unit.binary_environment),
            unit.descriptor.encode_to_vec(),
        )
        .with_settings_schema(unit.settings)
    }));
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed reviewed Obligation candidate ensemble")
}

pub(super) fn admit_obligation_candidate_ensemble_v1(
    store: &SqliteControlStore,
) -> Vec<AdmittedObligationCandidateRuntimeV1> {
    obligation_candidate_units_v1()
        .into_iter()
        .map(|unit| admit_obligation_candidate_unit_v1(store, unit))
        .collect()
}

pub(super) fn prepare_obligation_candidate_ensemble_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: Vec<AdmittedObligationCandidateRuntimeV1>,
) -> Vec<AdmittedObligationCandidateRuntimeV1> {
    admitted
        .into_iter()
        .map(|unit| prepare_obligation_candidate_unit_v1(supervisor, store, unit))
        .collect()
}

pub(super) fn start_obligation_candidate_ensemble_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: Vec<AdmittedObligationCandidateRuntimeV1>,
    child_stdio: &Path,
) -> Vec<StartedObligationCandidateRuntimeV1> {
    admitted
        .into_iter()
        .map(|unit| {
            start_obligation_candidate_unit_v1(supervisor, store, runtime_dir, unit, child_stdio)
        })
        .collect()
}

fn obligation_candidate_units_v1() -> [ObligationCandidateManagedUnitV1; 3] {
    [
        ObligationCandidateManagedUnitV1 {
            label: "Review Obligation Candidate",
            artifact_id: "review.obligation-candidate.runtime.v1",
            binary_environment: "MAKOSH_REVIEW_OBLIGATION_CANDIDATE_RUNTIME_BIN",
            storage_capability_id: REVIEW_OBLIGATION_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
            storage_revision: REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V3,
            descriptor: review_obligation_candidate_module_descriptor_v1(
                OBLIGATION_CANDIDATE_BUILD_ID_V1,
            ),
            settings: review_obligation_candidate_settings_schema_bytes_v1(),
            storage_bundle: review_obligation_candidate_storage_bundle_v1(),
        },
        ObligationCandidateManagedUnitV1 {
            label: "Reviewed Obligation Candidate Promotion",
            artifact_id: "reviewed_obligation_candidate_promotion.runtime.v1",
            binary_environment: "MAKOSH_REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_RUNTIME_BIN",
            storage_capability_id: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
            storage_revision: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V2,
            descriptor: reviewed_obligation_candidate_promotion_module_descriptor_v1(
                OBLIGATION_CANDIDATE_BUILD_ID_V1,
            ),
            settings: reviewed_obligation_candidate_promotion_settings_schema_bytes_v1(),
            storage_bundle: reviewed_obligation_candidate_promotion_storage_bundle_v1(),
        },
        ObligationCandidateManagedUnitV1 {
            label: "Obligations",
            artifact_id: "obligations.runtime.v1",
            binary_environment: "MAKOSH_OBLIGATIONS_RUNTIME_BIN",
            storage_capability_id: OBLIGATIONS_STORAGE_CAPABILITY_ID_V1,
            storage_revision: OBLIGATIONS_STORAGE_BUNDLE_REVISION_V3,
            descriptor: obligations_module_descriptor_v1(OBLIGATION_CANDIDATE_BUILD_ID_V1),
            settings: obligations_settings_schema_bytes_v1(),
            storage_bundle: obligations_storage_bundle_v1(),
        },
    ]
}

fn admit_obligation_candidate_unit_v1(
    store: &SqliteControlStore,
    unit: ObligationCandidateManagedUnitV1,
) -> AdmittedObligationCandidateRuntimeV1 {
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
    let effective_capability_ids = store
        .module_grant_snapshot(registration.registration_id())
        .expect("read exact managed Obligation candidate grants")
        .and_then(|snapshot| {
            snapshot
                .effective_grants()
                .map(|grants| grants.capability_ids().to_vec())
        })
        .expect("effective managed Obligation candidate grants");
    assert_eq!(
        effective_capability_ids, capability_ids,
        "{} exact capability grant drift",
        unit.label
    );
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
    AdmittedObligationCandidateRuntimeV1 {
        unit,
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

fn prepare_obligation_candidate_unit_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedObligationCandidateRuntimeV1,
) -> AdmittedObligationCandidateRuntimeV1 {
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

fn start_obligation_candidate_unit_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedObligationCandidateRuntimeV1,
    child_stdio: &Path,
) -> StartedObligationCandidateRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .unwrap_or_else(|error| panic!("load {} launch reservation: {error}", admitted.unit.label));
    start_reserved_obligation_candidate_unit_v1(
        supervisor,
        store,
        runtime_dir,
        admitted,
        reservation,
        child_stdio,
    )
}

pub(super) fn restart_obligation_candidate_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedObligationCandidateRuntimeV1,
    child_stdio: &Path,
) -> StartedObligationCandidateRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = active_obligation_candidate_storage_binding_v1(
        store,
        &AdmittedObligationCandidateRuntimeV1 {
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
    let successor = start_reserved_obligation_candidate_unit_v1(
        supervisor,
        store,
        runtime_dir,
        AdmittedObligationCandidateRuntimeV1 {
            unit: predecessor.unit,
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        reservation,
        child_stdio,
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_obligation_candidate_unit_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedObligationCandidateRuntimeV1,
    reservation: managed_launch::ManagedLaunchReservation,
    child_stdio: &Path,
) -> StartedObligationCandidateRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = active_obligation_candidate_storage_binding_v1(store, &admitted);
    let topology = crate::platform::storage::topology::current(store)
        .expect("reviewed Obligation candidate Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("live Vault status for reviewed Obligation candidate chain");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("reviewed Obligation candidate Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let module_kind = ModuleKindV1::try_from(admitted.unit.descriptor.module_kind)
        .expect("reviewed Obligation candidate module kind");
    unsafe {
        std::env::set_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
            child_stdio,
        );
    }
    let start_result = match module_kind {
        ModuleKindV1::Workflow => managed_launch::start_reserved_workflow(
            supervisor,
            runtime_dir,
            reservation,
            ManagedWorkflowRuntimeConfigurationV1 {
                major: 1,
                logical_owner_id: OBLIGATION_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
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
            &admitted.capability_ids,
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
                logical_human_owner_id: OBLIGATION_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
            },
        ),
        kind => panic!("unsupported reviewed Obligation candidate module kind: {kind:?}"),
    };
    unsafe {
        std::env::remove_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
        );
    }
    start_result.unwrap_or_else(|error| panic!("start managed {}: {error}", admitted.unit.label));
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "{} readiness: {error}; last_failure={:?}; child_category={}",
                admitted.unit.label,
                supervisor.last_failure(&admitted.registration_id),
                managed_child_failure_category_v1(child_stdio),
            )
        });
    StartedObligationCandidateRuntimeV1 {
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

fn managed_child_failure_category_v1(directory: &Path) -> &'static str {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return "capture-unavailable";
    };
    for entry in entries.flatten() {
        let Ok(bytes) = std::fs::read(entry.path()) else {
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        for category in [
            "admission",
            "event_contract",
            "event_unavailable",
            "invalid_transition",
            "persistence",
            "unavailable",
        ] {
            if text.contains(&format!("runtime failed: {category}")) {
                return category;
            }
        }
        if text.contains("runtime configuration is invalid") {
            return "configuration-invalid";
        }
        if text.contains("storage is unavailable") {
            return "storage-unavailable";
        }
    }
    "unclassified"
}

fn active_obligation_candidate_storage_binding_v1(
    store: &SqliteControlStore,
    admitted: &AdmittedObligationCandidateRuntimeV1,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            &admitted.registration_id,
            admitted.unit.storage_capability_id,
        )
        .expect("read reviewed Obligation candidate Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active reviewed Obligation candidate Storage binding")
}

fn assert_exact_unit_boundary(unit: &ObligationCandidateManagedUnitV1) {
    match unit.descriptor.module_id.as_str() {
        REVIEW_OBLIGATION_CANDIDATE_MODULE_ID_V1 => {
            assert_eq!(
                unit.descriptor.owner_id,
                REVIEW_OBLIGATION_CANDIDATE_OWNER_V1
            );
            assert_eq!(unit.descriptor.module_kind, ModuleKindV1::Domain as i32);
        }
        REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_MODULE_ID_V1 => {
            assert_eq!(
                unit.descriptor.owner_id,
                REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_OWNER_V1
            );
            assert_eq!(unit.descriptor.module_kind, ModuleKindV1::Workflow as i32);
        }
        OBLIGATIONS_MODULE_ID_V1 => {
            assert_eq!(unit.descriptor.owner_id, OBLIGATIONS_OWNER_ID_V1);
            assert_eq!(unit.descriptor.module_kind, ModuleKindV1::Domain as i32);
        }
        module_id => panic!("unexpected reviewed Obligation candidate module: {module_id}"),
    }
}
