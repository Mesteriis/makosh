use makosh_kernel_control_store::{
    ModuleDescriptorRegistrationRequestsV1, ModuleRegistration, ModuleRegistrationState,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

#[test]
fn approved_registration_descriptor_upgrade_preserves_identity_and_replaces_grants() {
    let path = fixture_path("success");
    let store = SqliteControlStore::create(&path, "instance-registration-upgrade", 1)
        .expect("create store");
    let original = ModuleRegistration::new(
        "registration-mail",
        "integration-mail",
        "owner-local",
        [1; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration(
            &original,
            &[
                "mail.legacy.compose".to_owned(),
                "mail.messages.read".to_owned(),
            ],
        )
        .expect("create original registration");
    store
        .approve_module_registration(
            original.registration_id(),
            &["mail.messages.read".to_owned()],
        )
        .expect("approve original registration");

    let upgraded = ModuleRegistration::new(
        original.registration_id(),
        original.module_id(),
        original.owner_id(),
        [2; 32],
        ModuleRegistrationState::Approved,
        3,
    );
    store
        .upgrade_approved_registration_with_all_descriptor_requests(
            &upgraded,
            &[
                "mail.composition.write".to_owned(),
                "mail.messages.read".to_owned(),
            ],
            empty_descriptor_requests(),
        )
        .expect("upgrade approved registration");

    let snapshot = store
        .module_grant_snapshot(original.registration_id())
        .expect("read upgraded snapshot")
        .expect("upgraded registration exists");
    let registration = snapshot.registration();
    assert_eq!(registration.registration_id(), original.registration_id());
    assert_eq!(registration.module_id(), original.module_id());
    assert_eq!(registration.owner_id(), original.owner_id());
    assert_eq!(registration.descriptor_sha256(), &[2; 32]);
    assert_eq!(registration.grant_epoch(), 3);
    assert_eq!(registration.state(), ModuleRegistrationState::Approved);
    assert_eq!(
        snapshot
            .effective_grants()
            .expect("upgraded grants")
            .capability_ids(),
        &[
            "mail.composition.write".to_owned(),
            "mail.messages.read".to_owned(),
        ],
    );

    drop(store);
    std::fs::remove_file(path).expect("remove control store");
}

#[test]
fn rejected_descriptor_upgrade_keeps_the_previous_snapshot() {
    let path = fixture_path("rollback");
    let store = SqliteControlStore::create(&path, "instance-registration-upgrade", 1)
        .expect("create store");
    let original = ModuleRegistration::new(
        "registration-telegram",
        "integration-telegram",
        "owner-local",
        [3; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration(&original, &["telegram.authorization".to_owned()])
        .expect("create original registration");
    store
        .approve_module_registration(
            original.registration_id(),
            &["telegram.authorization".to_owned()],
        )
        .expect("approve original registration");

    let skipped_epoch = ModuleRegistration::new(
        original.registration_id(),
        original.module_id(),
        original.owner_id(),
        [4; 32],
        ModuleRegistrationState::Approved,
        4,
    );
    assert!(
        store
            .upgrade_approved_registration_with_all_descriptor_requests(
                &skipped_epoch,
                &["telegram.authorization".to_owned()],
                empty_descriptor_requests(),
            )
            .is_err()
    );

    let snapshot = store
        .module_grant_snapshot(original.registration_id())
        .expect("read original snapshot")
        .expect("original registration exists");
    assert_eq!(snapshot.registration().descriptor_sha256(), &[3; 32]);
    assert_eq!(snapshot.registration().grant_epoch(), 2);
    assert_eq!(
        snapshot
            .effective_grants()
            .expect("original grants")
            .capability_ids(),
        &["telegram.authorization".to_owned()],
    );

    drop(store);
    std::fs::remove_file(path).expect("remove control store");
}

fn empty_descriptor_requests() -> ModuleDescriptorRegistrationRequestsV1<'static> {
    ModuleDescriptorRegistrationRequestsV1 {
        storage: &[],
        events: &[],
        blobs: &[],
        scheduler: &[],
        vault_purposes: &[],
        client_rpc_routes: &[],
        client_blob_routes: &[],
        client_realtime_routes: &[],
        query_rpc_routes: &[],
        request_rpc_routes: &[],
        contract_dependencies: &[],
    }
}

fn fixture_path(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "makosh-module-registration-upgrade-{label}-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
