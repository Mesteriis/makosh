use makosh_kernel_control_store::{
    ModuleDescriptorRegistrationRequestsV1, ModuleEventDeliveryPolicyV1, ModuleEventEnvelopeKindV1,
    ModuleEventRouteDirectionV1, ModuleEventRouteRequestInputV1, ModuleEventRouteRequestV1,
    ModuleEventSubscriptionRequirementV1, ModuleRegistration, ModuleRegistrationState,
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

#[test]
fn approved_registration_reconciles_event_contract_successor_with_same_descriptor() {
    let path = fixture_path("event-contract-successor");
    let store = SqliteControlStore::create(&path, "instance-registration-upgrade", 1)
        .expect("create store");
    let original = ModuleRegistration::new(
        "scheduler_developer",
        "scheduler",
        "scheduler",
        [5; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    let original_route = scheduler_route([6; 32]);
    store
        .create_pending_registration_with_all_descriptor_requests(
            &original,
            &["events.scheduler.ack".to_owned()],
            ModuleDescriptorRegistrationRequestsV1 {
                events: std::slice::from_ref(&original_route),
                ..empty_descriptor_requests()
            },
        )
        .expect("create original registration");
    store
        .approve_module_registration(
            original.registration_id(),
            &["events.scheduler.ack".to_owned()],
        )
        .expect("approve original registration");

    let successor = ModuleRegistration::new(
        original.registration_id(),
        original.module_id(),
        original.owner_id(),
        [5; 32],
        ModuleRegistrationState::Approved,
        3,
    );
    let successor_route = scheduler_route([7; 32]);
    store
        .reconcile_approved_registration_event_routes(
            &successor,
            std::slice::from_ref(&successor_route),
        )
        .expect("reconcile event contract successor");

    let snapshot = store
        .module_grant_snapshot(original.registration_id())
        .expect("read successor snapshot")
        .expect("successor registration exists");
    assert_eq!(snapshot.registration().descriptor_sha256(), &[5; 32]);
    assert_eq!(snapshot.registration().grant_epoch(), 3);
    assert_eq!(
        store
            .module_event_route_requests(
                original.registration_id(),
                successor_route.capability_id(),
            )
            .expect("read successor route"),
        vec![successor_route.clone()],
    );

    let no_op = ModuleRegistration::new(
        original.registration_id(),
        original.module_id(),
        original.owner_id(),
        [5; 32],
        ModuleRegistrationState::Approved,
        4,
    );
    assert!(
        store
            .reconcile_approved_registration_event_routes(
                &no_op,
                std::slice::from_ref(&successor_route),
            )
            .is_err()
    );
    assert_eq!(
        store
            .module_grant_snapshot(original.registration_id())
            .expect("read snapshot after rejected no-op")
            .expect("registration remains")
            .registration()
            .grant_epoch(),
        3,
    );

    drop(store);
    std::fs::remove_file(path).expect("remove control store");
}

fn scheduler_route(schema_sha256: [u8; 32]) -> ModuleEventRouteRequestV1 {
    ModuleEventRouteRequestV1::new(ModuleEventRouteRequestInputV1 {
        registration_id: "scheduler_developer".to_owned(),
        capability_id: "events.scheduler.ack".to_owned(),
        envelope_kind: ModuleEventEnvelopeKindV1::Ack,
        contract_owner: "scheduler".to_owned(),
        contract_name: "job_receipt".to_owned(),
        contract_major: 1,
        contract_revision: 1,
        contract_schema_sha256: schema_sha256,
        direction: ModuleEventRouteDirectionV1::Consume,
        max_in_flight: 16,
        delivery_policy: Some(ModuleEventDeliveryPolicyV1::new(
            ModuleEventSubscriptionRequirementV1::Required,
            3,
            2_000,
        )),
    })
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
