use makosh_kernel_control_store::{
    ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1, ModuleEventRouteRequestV1,
    ModuleRegistration, ModuleRegistrationState,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::platform::events::catalog;

use super::common::unique_target_root;

#[test]
fn control_store_retains_exact_descriptor_event_routes_with_registration() {
    let root = unique_target_root("makosh-event-route-request");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let request = event_route("events.publish");

    store
        .create_pending_registration_with_requests(
            &registration(),
            &["events.publish".to_owned()],
            &[],
            std::slice::from_ref(&request),
            &[],
        )
        .expect("persist registration and Event route together");

    assert_eq!(
        store
            .module_event_route_requests("registration_notes", "events.publish")
            .expect("read Event route"),
        vec![request.clone()]
    );
    assert!(
        store
            .module_event_route_requests("registration_notes", "unrequested")
            .expect("read absent Event route")
            .is_empty()
    );
    assert!(
        catalog::resolve(&store)
            .expect("resolve catalog without approvals")
            .is_empty()
    );
    store
        .approve_module_registration("registration_notes", &["events.publish".to_owned()])
        .expect("approve Event capability");
    let catalog_entries = catalog::resolve(&store).expect("resolve approved Event catalog");
    assert_eq!(catalog_entries.len(), 1);
    assert_eq!(catalog_entries[0].registration_id(), "registration_notes");
    assert_eq!(catalog_entries[0].module_id(), "module_notes");
    assert_eq!(catalog_entries[0].grant_epoch(), 2);
    assert_eq!(catalog_entries[0].capability_id(), "events.publish");
    assert_eq!(catalog_entries[0].route(), &request);
    let contracts = catalog::resolve_contracts(&store).expect("resolve Event contracts");
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].owner(), "owner_notes");
    assert_eq!(contracts[0].name(), "changed");
    assert_eq!(contracts[0].major(), 1);
    assert_eq!(contracts[0].revision(), 1);
    assert_eq!(contracts[0].publishers().len(), 1);
    assert!(contracts[0].consumers().is_empty());
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_rejects_event_routes_without_a_unique_requested_capability() {
    let root = unique_target_root("makosh-event-route-request-invalid");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let route = event_route("events.publish");

    assert!(
        store
            .create_pending_registration_with_requests(
                &registration(),
                &["events.publish".to_owned()],
                &[],
                &[route.clone(), route],
                &[],
            )
            .is_err()
    );
    assert!(
        store
            .create_pending_registration_with_requests(
                &registration(),
                &["events.publish".to_owned()],
                &[],
                &[event_route_with_limit("events.publish", 4_097)],
                &[],
            )
            .is_err()
    );
    assert!(
        store
            .module_registration("registration_notes")
            .expect("registration remains absent")
            .is_none()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_rejects_consumer_without_explicit_delivery_policy() {
    let root = unique_target_root("makosh-event-route-consumer-policy");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let consumer = ModuleEventRouteRequestV1::new(
        makosh_kernel_control_store::ModuleEventRouteRequestInputV1 {
            registration_id: "registration_notes".to_owned(),
            capability_id: "events.consume".to_owned(),
            envelope_kind: ModuleEventEnvelopeKindV1::Event,
            contract_owner: "owner_notes".to_owned(),
            contract_name: "changed".to_owned(),
            contract_major: 1,
            contract_revision: 1,
            contract_schema_sha256: [7; 32],
            direction: ModuleEventRouteDirectionV1::Consume,
            max_in_flight: 32,
            delivery_policy: None,
        },
    );

    assert!(
        store
            .create_pending_registration_with_requests(
                &registration(),
                &["events.consume".to_owned()],
                &[],
                &[consumer],
                &[],
            )
            .is_err()
    );
    assert!(
        store
            .module_registration("registration_notes")
            .expect("registration remains absent")
            .is_none()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn event_catalog_rejects_incompatible_contract_revisions_before_broker_reconciliation() {
    let root = unique_target_root("makosh-event-route-conflict");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let first = registration();
    let second = ModuleRegistration::new(
        "registration_search",
        "module_search",
        "owner_search",
        [2; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration_with_requests(
            &first,
            &["events.publish".to_owned()],
            &[],
            &[event_route("events.publish")],
            &[],
        )
        .expect("persist first Event route");
    store
        .create_pending_registration_with_requests(
            &second,
            &["events.publish".to_owned()],
            &[],
            &[ModuleEventRouteRequestV1::new(
                makosh_kernel_control_store::ModuleEventRouteRequestInputV1 {
                    registration_id: "registration_search".to_owned(),
                    capability_id: "events.publish".to_owned(),
                    envelope_kind: ModuleEventEnvelopeKindV1::Event,
                    contract_owner: "owner_notes".to_owned(),
                    contract_name: "changed".to_owned(),
                    contract_major: 1,
                    contract_revision: 2,
                    contract_schema_sha256: [8; 32],
                    direction: ModuleEventRouteDirectionV1::Publish,
                    max_in_flight: 32,
                    delivery_policy: None,
                },
            )],
            &[],
        )
        .expect("persist second Event route");
    store
        .approve_module_registration("registration_notes", &["events.publish".to_owned()])
        .expect("approve first Event route");
    store
        .approve_module_registration("registration_search", &["events.publish".to_owned()])
        .expect("approve second Event route");

    assert_eq!(
        catalog::resolve_contracts(&store),
        Err("Event catalog is incompatible: IncompatibleRevisionOrSchema".to_owned())
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn event_catalog_accepts_only_the_exact_wire_compatible_scheduler_package_rename() {
    const LEGACY_SCHEMA: [u8; 32] = [
        0x3f, 0x9b, 0xb7, 0xb2, 0xde, 0xa5, 0xe0, 0xa7, 0x8d, 0x4c, 0x9f, 0xd6, 0x8c, 0xf5, 0x86,
        0x7d, 0x36, 0x6b, 0x2b, 0x70, 0xf2, 0x1f, 0x81, 0xaf, 0xe6, 0x9d, 0x31, 0xb8, 0x64, 0x76,
        0x74, 0x15,
    ];
    const CURRENT_SCHEMA: [u8; 32] = [
        0xc5, 0x60, 0x05, 0x21, 0x88, 0x8d, 0x9f, 0x76, 0x89, 0xb3, 0xc6, 0x5e, 0x61, 0xab, 0x72,
        0x6c, 0x9d, 0xdf, 0x16, 0x5f, 0xd6, 0xea, 0xb3, 0x3f, 0xe8, 0x8c, 0x5c, 0xa7, 0x20, 0x95,
        0x37, 0x10,
    ];
    let root = unique_target_root("makosh-scheduler-package-rename");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    for (registration_id, module_id, capability_id, schema_sha256) in [
        (
            "scheduler_legacy",
            "scheduler-legacy-consumer",
            "events.scheduler.legacy",
            LEGACY_SCHEMA,
        ),
        (
            "scheduler_current",
            "scheduler-current-publisher",
            "events.scheduler.current",
            CURRENT_SCHEMA,
        ),
    ] {
        let registration = ModuleRegistration::new(
            registration_id,
            module_id,
            "scheduler",
            schema_sha256,
            ModuleRegistrationState::Pending,
            1,
        );
        let route = ModuleEventRouteRequestV1::new(
            makosh_kernel_control_store::ModuleEventRouteRequestInputV1 {
                registration_id: registration_id.to_owned(),
                capability_id: capability_id.to_owned(),
                envelope_kind: ModuleEventEnvelopeKindV1::Ack,
                contract_owner: "scheduler".to_owned(),
                contract_name: "job_receipt".to_owned(),
                contract_major: 1,
                contract_revision: 1,
                contract_schema_sha256: schema_sha256,
                direction: ModuleEventRouteDirectionV1::Publish,
                max_in_flight: 16,
                delivery_policy: None,
            },
        );
        store
            .create_pending_registration_with_requests(
                &registration,
                &[capability_id.to_owned()],
                &[],
                &[route],
                &[],
            )
            .expect("persist Scheduler package route");
        store
            .approve_module_registration(registration_id, &[capability_id.to_owned()])
            .expect("approve Scheduler package route");
    }

    let contracts = catalog::resolve_contracts(&store).expect("resolve compatible rename");
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].schema_sha256(), &CURRENT_SCHEMA);
    assert_eq!(contracts[0].publishers().len(), 2);

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

fn event_route(capability_id: &str) -> ModuleEventRouteRequestV1 {
    event_route_with_limit(capability_id, 32)
}

fn event_route_with_limit(capability_id: &str, max_in_flight: u16) -> ModuleEventRouteRequestV1 {
    ModuleEventRouteRequestV1::new(
        makosh_kernel_control_store::ModuleEventRouteRequestInputV1 {
            registration_id: "registration_notes".to_owned(),
            capability_id: capability_id.to_owned(),
            envelope_kind: ModuleEventEnvelopeKindV1::Event,
            contract_owner: "owner_notes".to_owned(),
            contract_name: "changed".to_owned(),
            contract_major: 1,
            contract_revision: 1,
            contract_schema_sha256: [7; 32],
            direction: ModuleEventRouteDirectionV1::Publish,
            max_in_flight,
            delivery_policy: None,
        },
    )
}
