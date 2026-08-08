use makosh_kernel_control_store::{
    ModuleRegistration, ModuleRegistrationState, ModuleStorageRequestV1,
    PlatformStorageBindingInputV1, PlatformStorageBindingV1, PlatformStorageBundleV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use sha2::{Digest, Sha256};

use super::common::unique_target_root;

#[test]
fn control_store_retains_only_exact_descriptor_storage_requests() {
    let root = unique_target_root("makosh-storage-request");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let registration = registration();
    let requested_capabilities = vec!["events.publish".to_owned(), "storage.access".to_owned()];
    let request = storage_request("storage.access", "owner_notes");

    store
        .create_pending_registration_with_requests(
            &registration,
            &requested_capabilities,
            std::slice::from_ref(&request),
            &[],
            &[],
        )
        .expect("persist pending registration and Storage request together");
    assert_eq!(
        store
            .module_storage_request("registration_notes", "storage.access")
            .expect("read Storage request"),
        Some(request)
    );
    assert_eq!(
        store
            .module_storage_request("registration_notes", "events.publish")
            .expect("read absent request"),
        None
    );

    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_rejects_cross_owner_or_unrequested_storage_requests() {
    let root = unique_target_root("makosh-storage-request-invalid");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let capabilities = vec!["storage.access".to_owned()];
    let cross_owner = storage_request("storage.access", "owner_other");

    assert!(
        store
            .create_pending_registration_with_requests(
                &registration(),
                &capabilities,
                std::slice::from_ref(&cross_owner),
                &[],
                &[],
            )
            .is_err()
    );
    assert!(
        store
            .create_pending_registration_with_requests(
                &registration(),
                &capabilities,
                std::slice::from_ref(&storage_request("unrequested", "owner_notes")),
                &[],
                &[],
            )
            .is_err()
    );
    assert!(
        store
            .module_registration("registration_notes")
            .expect("registration remains absent after rejected request")
            .is_none()
    );

    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_keeps_only_the_next_durable_storage_binding_revision() {
    let root = unique_target_root("makosh-storage-binding");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let first = storage_binding(1, "runtime_notes_1");
    store
        .record_platform_storage_binding(&first)
        .expect("record first binding");
    assert_eq!(
        store
            .platform_storage_binding("registration_notes", "storage.access")
            .expect("read binding"),
        Some(first),
    );
    assert!(
        store
            .record_platform_storage_binding(&storage_binding(1, "runtime_notes_2"))
            .is_err()
    );
    let revoking = store
        .begin_platform_storage_binding_revocation("registration_notes", "storage.access", 1)
        .expect("reserve first binding for revoke");
    assert!(matches!(
        revoking.state(),
        makosh_kernel_control_store::PlatformStorageBindingStateV1::Revoking
    ));
    assert_eq!(
        store
            .begin_platform_storage_binding_revocation("registration_notes", "storage.access", 1,)
            .expect("retry exact reserved binding"),
        revoking,
    );
    store
        .record_platform_storage_binding(&storage_binding(2, "runtime_notes_2"))
        .expect("record next binding revision");
    assert_eq!(
        store
            .platform_storage_binding("registration_notes", "storage.access")
            .expect("read replacement")
            .expect("replacement binding")
            .runtime_instance_id(),
        "runtime_notes_2",
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_atomically_reserves_all_active_storage_bindings_for_a_registration() {
    let root = unique_target_root("makosh-registration-storage-revoke");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    for binding in [
        storage_binding_for(
            1,
            "registration_notes",
            "storage.primary",
            "owner_notes",
            "runtime_notes",
            "runtime_notes_primary",
        ),
        storage_binding_for(
            1,
            "registration_notes",
            "storage.archive",
            "owner_notes",
            "runtime_notes",
            "runtime_notes_archive",
        ),
        storage_binding_for(
            1,
            "registration_other",
            "storage.primary",
            "owner_other",
            "runtime_other",
            "runtime_other_primary",
        ),
    ] {
        store
            .record_platform_storage_binding(&binding)
            .expect("record active Storage binding");
    }

    let reserved = store
        .begin_registration_storage_bindings_revocation("registration_notes")
        .expect("reserve registration Storage bindings");
    assert_eq!(
        reserved
            .iter()
            .map(|binding| binding.capability_id())
            .collect::<Vec<_>>(),
        vec!["storage.archive", "storage.primary"],
    );
    assert!(reserved.iter().all(|binding| matches!(
        binding.state(),
        makosh_kernel_control_store::PlatformStorageBindingStateV1::Revoking
    )));
    assert_eq!(
        store
            .begin_registration_storage_bindings_revocation("registration_notes")
            .expect("retry registration Storage reservations"),
        reserved,
    );
    assert!(matches!(
        store
            .platform_storage_binding("registration_other", "storage.primary")
            .expect("read unrelated Storage binding")
            .expect("unrelated Storage binding")
            .state(),
        makosh_kernel_control_store::PlatformStorageBindingStateV1::Active
    ));

    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn registration_storage_fence_remains_durable_when_storage_runtime_is_unavailable() {
    let root = unique_target_root("makosh-registration-storage-fence");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    store
        .record_platform_storage_binding(&storage_binding(1, "runtime_notes"))
        .expect("record active Storage binding");
    let supervisor = crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor::new(
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
    );

    assert_eq!(
        crate::platform::storage::revocation::fence_registration_bindings(
            &supervisor,
            &store,
            "registration_notes",
        )
        .expect_err("inactive Storage runtime cannot prove physical fencing"),
        "managed Storage revocation is incomplete"
    );
    assert!(matches!(
        store
            .platform_storage_binding("registration_notes", "storage.access")
            .expect("read reserved Storage binding")
            .expect("reserved Storage binding")
            .state(),
        makosh_kernel_control_store::PlatformStorageBindingStateV1::Revoking
    ));

    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_retains_one_immutable_canonical_bundle_per_owner_revision() {
    let root = unique_target_root("makosh-storage-bundle");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let first = storage_bundle(1, [8; 32], vec![1, 2, 3]);
    store
        .record_platform_storage_bundle(&first)
        .expect("record canonical Storage bundle");
    store
        .record_platform_storage_bundle(&first)
        .expect("retry exact canonical Storage bundle");
    assert_eq!(
        store
            .platform_storage_bundle("owner_notes", 1)
            .expect("read Storage bundle"),
        Some(first),
    );
    assert!(
        store
            .record_platform_storage_bundle(&storage_bundle(1, [9; 32], vec![4]))
            .is_err()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
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

fn storage_request(capability_id: &str, owner_id: &str) -> ModuleStorageRequestV1 {
    ModuleStorageRequestV1::new("registration_notes", capability_id, owner_id, 4, 5_000)
}

fn storage_binding(revision: u64, runtime_instance_id: &str) -> PlatformStorageBindingV1 {
    storage_binding_for(
        revision,
        "registration_notes",
        "storage.access",
        "owner_notes",
        runtime_instance_id,
        "runtime_notes",
    )
}

fn storage_binding_for(
    revision: u64,
    registration_id: &str,
    capability_id: &str,
    owner_id: &str,
    runtime_instance_id: &str,
    runtime_principal: &str,
) -> PlatformStorageBindingV1 {
    PlatformStorageBindingV1::new(PlatformStorageBindingInputV1 {
        registration_id: registration_id.to_owned(),
        capability_id: capability_id.to_owned(),
        owner_id: owner_id.to_owned(),
        binding_revision: revision,
        topology_revision: 1,
        storage_generation: 1,
        runtime_instance_id: runtime_instance_id.to_owned(),
        runtime_generation: 7,
        grant_epoch: 3,
        role_epoch: revision,
        runtime_principal: runtime_principal.to_owned(),
        connection_budget: 4,
        statement_timeout_millis: 5_000,
        credential_lease_revision: revision,
        storage_bundle_revision: 1,
        storage_bundle_digest: [7; 32],
    })
    .expect("valid binding")
}

fn storage_bundle(revision: u64, _: [u8; 32], bytes: Vec<u8>) -> PlatformStorageBundleV1 {
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    PlatformStorageBundleV1::new("owner_notes", revision, digest, bytes)
        .expect("valid canonical Storage bundle")
}
