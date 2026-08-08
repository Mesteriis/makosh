use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, ManagedLaunchRecord, ModuleRegistration, ModuleRegistrationState,
    ModuleStorageRequestV1, PlatformStorageBindingStateV1, PlatformStorageBundleV1,
    PlatformStorageEndpointV1, PlatformStorageTopology, StorageDeploymentProfileV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use sha2::{Digest, Sha256};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use crate::platform::{
    scheduler::lifecycle as scheduler_lifecycle, storage::issuance::issue_managed,
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

use super::common::unique_target_root;

#[test]
fn active_scheduler_binding_is_the_only_automatic_restart_intent() {
    let (root, store, registration, capability, binding) = active_scheduler_fixture();

    let selected = scheduler_lifecycle::active_scheduler_binding(&store)
        .expect("resolve automatic Scheduler restart intent")
        .expect("active binding is desired state");
    assert_eq!(selected, binding);
    let successor = crate::platform::storage::successor::issue_after(&selected)
        .expect("derive fresh successor fences");
    assert_eq!(successor.role_epoch(), 2);
    assert_eq!(successor.credential_lease_revision(), 2);
    assert_eq!(successor.storage_bundle_revision(), 1);
    let bundle_digest: [u8; 32] = Sha256::digest([9]).into();
    assert_eq!(successor.storage_bundle_digest(), &bundle_digest);
    let upgraded_digest: [u8; 32] = Sha256::digest([10]).into();
    let upgraded =
        crate::platform::storage::successor::issue_after_with_bundle(&selected, 2, upgraded_digest)
            .expect("derive successor with upgraded Storage bundle");
    assert_eq!(upgraded.role_epoch(), 2);
    assert_eq!(upgraded.credential_lease_revision(), 2);
    assert_eq!(upgraded.storage_bundle_revision(), 2);
    assert_eq!(upgraded.storage_bundle_digest(), &upgraded_digest);

    let revoking = store
        .begin_platform_storage_binding_revocation(
            registration.registration_id(),
            &capability,
            binding.binding_revision(),
        )
        .expect("intentionally revoke Scheduler binding");
    assert_eq!(revoking.state(), PlatformStorageBindingStateV1::Revoking);
    assert!(
        scheduler_lifecycle::active_scheduler_binding(&store)
            .expect("read revoked Scheduler state")
            .is_none(),
        "revoked binding must not resurrect Scheduler automatically"
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn lifecycle_blocks_after_first_failed_successor_launch() {
    let (root, store, registration, _, _) = active_scheduler_fixture();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let worker_store = Arc::clone(&store);
    let worker_shutdown = Arc::clone(&shutdown_requested);
    let worker_supervisor = supervisor.clone();
    let runtime_dir = root.join("runtime");
    let worker = std::thread::spawn(move || {
        scheduler_lifecycle::serve(
            worker_store,
            std::path::Path::new("/unavailable/makosh-kernel"),
            &runtime_dir,
            worker_shutdown,
            worker_supervisor,
            None,
        )
    });

    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let generation = store
            .effective_managed_launch_record(registration.registration_id())
            .expect("read Scheduler launch record")
            .expect("Scheduler launch record remains durable")
            .runtime_generation();
        if generation == 2 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "lifecycle worker did not make one successor attempt"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    std::thread::sleep(Duration::from_millis(600));
    assert_eq!(
        store
            .effective_managed_launch_record(registration.registration_id())
            .expect("read bounded Scheduler launch record")
            .expect("Scheduler launch record remains durable")
            .runtime_generation(),
        2,
        "automatic recovery must block after the first failed successor launch"
    );
    shutdown_requested.store(true, Ordering::Release);
    worker
        .join()
        .expect("join lifecycle worker")
        .expect("lifecycle worker exits on Kernel shutdown");
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn active_scheduler_fixture() -> (
    std::path::PathBuf,
    Arc<SqliteControlStore>,
    ModuleRegistration,
    String,
    makosh_kernel_control_store::PlatformStorageBindingV1,
) {
    let root = unique_target_root("makosh-scheduler-lifecycle");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
            .expect("create Control Store"),
    );
    let registration = ModuleRegistration::new(
        "scheduler_registration",
        "scheduler",
        "owner_scheduler",
        [1; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    let capability = "storage.scheduler".to_owned();
    admit_scheduler_registration(&store, &registration, &capability);
    record_storage_topology_and_bundle(&store);
    let bundle_digest: [u8; 32] = Sha256::digest([9]).into();
    let binding = issue_managed(
        &store,
        registration.registration_id(),
        "scheduler_runtime_1",
        1,
        &capability,
        crate::platform::storage::issuance::StorageBindingIssueV1::new(1, 1, 1, bundle_digest)
            .expect("initial Scheduler Storage issue"),
    )
    .expect("issue active Scheduler Storage binding");

    assert_eq!(binding.storage_bundle_digest(), &bundle_digest);
    (root, store, registration, capability, binding)
}

fn admit_scheduler_registration(
    store: &SqliteControlStore,
    registration: &ModuleRegistration,
    capability: &str,
) {
    let request = ModuleStorageRequestV1::new(
        registration.registration_id(),
        capability,
        registration.owner_id(),
        4,
        5_000,
    );
    store
        .create_pending_registration_with_requests(
            registration,
            std::slice::from_ref(&capability.to_owned()),
            std::slice::from_ref(&request),
            &[],
            &[],
        )
        .expect("record Scheduler registration");
    let grants = store
        .approve_module_registration(
            registration.registration_id(),
            std::slice::from_ref(&capability.to_owned()),
        )
        .expect("approve Scheduler Storage capability");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "distribution_scheduler",
            "artifact_scheduler",
            [2; 32],
            [1; 32],
            Some([4; 32]),
        ))
        .expect("record Scheduler release binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            registration.registration_id(),
            "scheduler_runtime_1",
            1,
            1,
            1,
            grants.grant_epoch(),
        ))
        .expect("record Scheduler launch reservation");
}

fn record_storage_topology_and_bundle(store: &SqliteControlStore) {
    store
        .record_platform_storage_topology(&PlatformStorageTopology::new(
            makosh_kernel_control_store::PlatformStorageTopologyInputV1 {
                revision: 1,
                storage_generation: 1,
                storage_instance_id: "storage_main".to_owned(),
                database_id: "makosh".to_owned(),
                deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded,
                postgres_endpoint: PlatformStorageEndpointV1::new("127.0.0.1", 5_432),
                pgbouncer_endpoint: PlatformStorageEndpointV1::new("127.0.0.1", 6_432),
                postgres_artifact_sha256: [5; 32],
                pgbouncer_artifact_sha256: [6; 32],
            },
        ))
        .expect("record Storage topology");
    let bytes = vec![9];
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                "owner_scheduler",
                1,
                Sha256::digest(&bytes).into(),
                bytes,
            )
            .expect("create Scheduler Storage bundle"),
        )
        .expect("record Scheduler Storage bundle");
}
