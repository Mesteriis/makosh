use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, ExternalRuntimeAttestation, ManagedLaunchRecord,
    ModuleRegistration, ModuleRegistrationState, ModuleStorageRequestV1, PlatformStorageBundleV1,
    PlatformStorageEndpointV1, PlatformStorageTopology, StorageDeploymentProfileV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use sha2::{Digest, Sha256};
use std::sync::{Arc, atomic::AtomicBool};

use crate::platform::macos::managed_launch;
use crate::platform::scheduler::launch as scheduler_launch;
use crate::platform::storage::authorization::{authorize_binding, authorize_managed_binding};
use crate::platform::storage::issuance::{StorageBindingIssueV1, issue_external, issue_managed};
use crate::platform::storage::successor as storage_successor;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

use super::common::unique_target_root;

#[test]
fn storage_authorization_requires_request_approval_and_current_attestation() {
    let (root, store, grants) = approved_store(true);

    let authorization = authorize_binding(
        &store,
        "registration_notes",
        "runtime_notes",
        7,
        "storage.access",
    )
    .expect("authorize exact current Storage route");

    assert_eq!(authorization.registration_id(), "registration_notes");
    assert_eq!(authorization.capability_id(), "storage.access");
    assert_eq!(authorization.owner_id(), "owner_notes");
    assert_eq!(authorization.runtime_id(), "runtime_notes");
    assert_eq!(authorization.runtime_generation(), 7);
    assert_eq!(authorization.grant_epoch(), grants.grant_epoch());
    assert_eq!(authorization.connection_budget(), 4);
    assert_eq!(authorization.statement_timeout_millis(), 5_000);

    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn storage_authorization_rejects_missing_request_or_stale_runtime() {
    let (root, store, _) = approved_store(false);
    let missing = authorize_binding(
        &store,
        "registration_notes",
        "runtime_notes",
        7,
        "storage.access",
    );
    assert!(matches!(
        &missing,
        Err(error) if error == "capability did not declare a Storage request"
    ));
    std::fs::remove_dir_all(root).expect("remove missing-request fixture directory");

    let (root, store, _) = approved_store(true);
    let stale = authorize_binding(
        &store,
        "registration_notes",
        "runtime_notes",
        8,
        "storage.access",
    );
    assert!(matches!(
        &stale,
        Err(error) if error == "external runtime attestation is stale"
    ));
    std::fs::remove_dir_all(root).expect("remove stale-runtime fixture directory");
}

#[test]
fn managed_storage_authorization_requires_exact_current_launch_identity() {
    let (root, store, grants) = approved_store(true);
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            "registration_notes",
            1,
            "distribution_notes",
            "artifact_notes",
            [2; 32],
            [1; 32],
            None,
        ))
        .expect("record managed release binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            "registration_notes",
            "managed_runtime_notes",
            1,
            1,
            7,
            grants.grant_epoch(),
        ))
        .expect("record managed launch");

    let authorization = authorize_managed_binding(
        &store,
        "registration_notes",
        "managed_runtime_notes",
        7,
        "storage.access",
    )
    .expect("authorize exact managed launch");
    assert_eq!(authorization.runtime_id(), "managed_runtime_notes");
    assert_eq!(authorization.runtime_generation(), 7);
    assert!(
        authorize_managed_binding(
            &store,
            "registration_notes",
            "reused_runtime",
            7,
            "storage.access",
        )
        .is_err()
    );

    std::fs::remove_dir_all(root).expect("remove managed fixture directory");
}

#[test]
fn managed_launch_reservation_is_durable_before_storage_binding_issuance() {
    let (root, store, grants) = approved_store(true);
    prepare_managed_binding_store(&store, grants.grant_epoch());
    let supervisor = ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false)));

    let reservation = managed_launch::reserve(&supervisor, &store, "registration_notes")
        .expect("reserve the next managed launch identity");
    assert_eq!(reservation.runtime_generation(), 8);
    let persisted = store
        .effective_managed_launch_record("registration_notes")
        .expect("read durable launch reservation")
        .expect("launch reservation exists");
    assert_eq!(
        persisted.runtime_instance_id(),
        reservation.runtime_instance_id()
    );
    assert_eq!(
        persisted.runtime_generation(),
        reservation.runtime_generation()
    );
    let authorization = authorize_managed_binding(
        &store,
        "registration_notes",
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        "storage.access",
    )
    .expect("issue authorization against the reserved runtime identity");
    assert_eq!(authorization.runtime_generation(), 8);

    let reloaded = managed_launch::load(&supervisor, &store, "registration_notes")
        .expect("reconstruct the durable reservation after owner-control IPC");
    assert_eq!(
        reloaded.runtime_instance_id(),
        reservation.runtime_instance_id()
    );
    assert_eq!(
        reloaded.runtime_generation(),
        reservation.runtime_generation()
    );
    assert_eq!(reloaded.grant_epoch(), reservation.grant_epoch());

    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            "registration_notes",
            2,
            "distribution_notes",
            "artifact_notes",
            [2; 32],
            [1; 32],
            None,
        ))
        .expect("advance the managed release binding");
    assert!(
        managed_launch::load(&supervisor, &store, "registration_notes").is_err(),
        "a reservation cannot cross a managed release binding revision"
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn scheduler_restart_reserves_successor_and_issues_its_exact_storage_binding() {
    let (root, store, grants) = approved_store(true);
    prepare_managed_binding_store(&store, grants.grant_epoch());
    let supervisor = ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false)));
    let (reservation, binding) = storage_successor::reserve(
        &supervisor,
        &store,
        "registration_notes",
        "storage.access",
        issue(1, 1),
    )
    .expect("reserve Scheduler successor and issue its Storage binding");
    assert_exact_scheduler_binding(&reservation, &binding, 8);
    assert_successor_scheduler_binding(&supervisor, &store);
    assert_advanced_topology_rejects_binding(&reservation, &binding);

    std::fs::remove_dir_all(root).expect("remove Scheduler launch fixture directory");
}

fn assert_exact_scheduler_binding(
    reservation: &managed_launch::ManagedLaunchReservation,
    binding: &makosh_kernel_control_store::PlatformStorageBindingV1,
    generation: u64,
) {
    assert_eq!(reservation.runtime_generation(), generation);
    assert_eq!(
        binding.runtime_instance_id(),
        reservation.runtime_instance_id()
    );
    assert_eq!(
        binding.runtime_generation(),
        reservation.runtime_generation()
    );
    scheduler_launch::validate_storage_binding(reservation, binding, &topology())
        .expect("accept exact Scheduler binding");
}

fn assert_successor_scheduler_binding(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
) {
    let (successor, binding) = storage_successor::reserve(
        supervisor,
        store,
        "registration_notes",
        "storage.access",
        issue(2, 2),
    )
    .expect("fence the predecessor and issue its Scheduler successor binding");
    assert_exact_scheduler_binding(&successor, &binding, 9);
    assert_eq!(binding.binding_revision(), 2);
    assert_eq!(binding.role_epoch(), 2);
    let newer = managed_launch::reserve(supervisor, store, "registration_notes")
        .expect("reserve a successor managed identity");
    assert!(scheduler_launch::validate_storage_binding(&newer, &binding, &topology()).is_err());
    let (_, _, policy) = newer
        .into_single_attempt_launch_parts()
        .expect("derive a Scheduler single-attempt launch policy");
    assert_eq!(policy.max_attempts(), 1);
}

fn assert_advanced_topology_rejects_binding(
    reservation: &managed_launch::ManagedLaunchReservation,
    binding: &makosh_kernel_control_store::PlatformStorageBindingV1,
) {
    let topology = PlatformStorageTopology::new(
        makosh_kernel_control_store::PlatformStorageTopologyInputV1 {
            revision: 2,
            storage_generation: 2,
            storage_instance_id: "storage_successor".to_owned(),
            database_id: "makosh".to_owned(),
            deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded,
            postgres_endpoint: PlatformStorageEndpointV1::new("127.0.0.1", 5_433),
            pgbouncer_endpoint: PlatformStorageEndpointV1::new("127.0.0.1", 6_433),
            postgres_artifact_sha256: [3; 32],
            pgbouncer_artifact_sha256: [4; 32],
        },
    );
    assert!(scheduler_launch::validate_storage_binding(reservation, binding, &topology).is_err());
}

#[test]
fn managed_storage_binding_issues_only_successive_durable_fences() {
    let (root, store, grants) = approved_store(true);
    prepare_managed_binding_store(&store, grants.grant_epoch());

    let first = issue_managed(
        &store,
        "registration_notes",
        "managed_runtime_notes",
        7,
        "storage.access",
        issue(1, 1),
    )
    .expect("issue initial binding");
    assert_eq!(first.binding_revision(), 1);
    assert_eq!(first.runtime_principal(), "storage_9019e7125a029dd5_1");
    assert_eq!(
        issue_managed(
            &store,
            "registration_notes",
            "managed_runtime_notes",
            7,
            "storage.access",
            issue(1, 1),
        )
        .expect("retry exact initial binding"),
        first,
    );
    let revoking = store
        .begin_platform_storage_binding_revocation(
            "registration_notes",
            "storage.access",
            first.binding_revision(),
        )
        .expect("reserve first binding for revoke");
    assert!(matches!(
        revoking.state(),
        makosh_kernel_control_store::PlatformStorageBindingStateV1::Revoking
    ));

    let second = issue_managed(
        &store,
        "registration_notes",
        "managed_runtime_notes",
        7,
        "storage.access",
        issue(2, 2),
    )
    .expect("rotate durable binding");
    assert_eq!(second.binding_revision(), 2);
    assert_eq!(second.role_epoch(), 2);
    assert_eq!(second.runtime_principal(), "storage_9019e7125a029dd5_2");
    assert_eq!(
        store
            .platform_storage_binding("registration_notes", "storage.access")
            .expect("read binding"),
        Some(second),
    );
    assert!(
        issue_managed(
            &store,
            "registration_notes",
            "managed_runtime_notes",
            7,
            "storage.access",
            issue(2, 3),
        )
        .is_err()
    );

    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn external_storage_binding_requires_the_exact_current_attestation_and_rotates_fences() {
    let (root, store, _) = approved_store(true);
    prepare_storage_binding_store(&store);

    let first = issue_external(
        &store,
        "registration_notes",
        "runtime_notes",
        7,
        "storage.access",
        issue(1, 1),
    )
    .expect("issue attested external binding");
    assert_eq!(first.binding_revision(), 1);
    assert_eq!(first.runtime_instance_id(), "runtime_notes");

    store
        .begin_platform_storage_binding_revocation(
            "registration_notes",
            "storage.access",
            first.binding_revision(),
        )
        .expect("reserve external binding for revoke");
    let second = issue_external(
        &store,
        "registration_notes",
        "runtime_notes",
        7,
        "storage.access",
        issue(2, 2),
    )
    .expect("rotate attested external binding");
    assert_eq!(second.binding_revision(), 2);
    assert_eq!(second.role_epoch(), 2);
    assert!(
        issue_external(
            &store,
            "registration_notes",
            "runtime_notes",
            8,
            "storage.access",
            issue(3, 3),
        )
        .is_err()
    );

    std::fs::remove_dir_all(root).expect("remove external fixture directory");
}

fn prepare_managed_binding_store(store: &SqliteControlStore, grant_epoch: u64) {
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            "registration_notes",
            1,
            "distribution_notes",
            "artifact_notes",
            [2; 32],
            [1; 32],
            None,
        ))
        .expect("record managed release binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            "registration_notes",
            "managed_runtime_notes",
            1,
            1,
            7,
            grant_epoch,
        ))
        .expect("record managed launch");
    prepare_storage_binding_store(store);
}

fn prepare_storage_binding_store(store: &SqliteControlStore) {
    store
        .record_platform_storage_topology(&topology())
        .expect("record Storage topology");
    let bytes = vec![1];
    let bundle =
        PlatformStorageBundleV1::new("owner_notes", 1, Sha256::digest(&bytes).into(), bytes)
            .expect("valid Storage bundle");
    store
        .record_platform_storage_bundle(&bundle)
        .expect("record exact Storage bundle");
}

fn approved_store(
    with_storage_request: bool,
) -> (
    std::path::PathBuf,
    SqliteControlStore,
    makosh_kernel_control_store::GrantSet,
) {
    let root = unique_target_root("makosh-storage-authorization");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let registration = registration();
    let capabilities = vec!["storage.access".to_owned()];
    if with_storage_request {
        let request = ModuleStorageRequestV1::new(
            registration.registration_id(),
            "storage.access",
            registration.owner_id(),
            4,
            5_000,
        );
        store
            .create_pending_registration_with_requests(
                &registration,
                &capabilities,
                std::slice::from_ref(&request),
                &[],
                &[],
            )
            .expect("persist registration request");
    } else {
        store
            .create_pending_registration(&registration, &capabilities)
            .expect("persist registration");
    }
    let grants = store
        .approve_module_registration(registration.registration_id(), &capabilities)
        .expect("approve requested capability");
    store
        .attest_external_runtime(&ExternalRuntimeAttestation::new(
            registration.registration_id(),
            "runtime_notes",
            7,
            grants.grant_epoch(),
            [9; 32],
        ))
        .expect("attest current runtime");
    (root, store, grants)
}

fn registration() -> ModuleRegistration {
    ModuleRegistration::new(
        "registration_notes",
        "module_notes",
        "owner_notes",
        [1; 32],
        ModuleRegistrationState::Pending,
        1,
    )
}

fn issue(role_epoch: u64, credential_lease_revision: u64) -> StorageBindingIssueV1 {
    let digest: [u8; 32] = Sha256::digest([1]).into();
    StorageBindingIssueV1::new(role_epoch, credential_lease_revision, 1, digest)
        .expect("valid binding issue")
}

fn topology() -> PlatformStorageTopology {
    PlatformStorageTopology::new(
        makosh_kernel_control_store::PlatformStorageTopologyInputV1 {
            revision: 1,
            storage_generation: 1,
            storage_instance_id: "storage_main".to_owned(),
            database_id: "makosh".to_owned(),
            deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded,
            postgres_endpoint: PlatformStorageEndpointV1::new("127.0.0.1", 5_432),
            pgbouncer_endpoint: PlatformStorageEndpointV1::new("127.0.0.1", 6_432),
            postgres_artifact_sha256: [1; 32],
            pgbouncer_artifact_sha256: [2; 32],
        },
    )
}
