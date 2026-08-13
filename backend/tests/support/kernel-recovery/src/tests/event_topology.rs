use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, ManagedLaunchRecord, ModuleEventDeliveryPolicyV1,
    ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1, ModuleEventRouteRequestV1,
    ModuleEventSubscriptionRequirementV1, ModuleRegistration, ModuleRegistrationState,
    PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::ManagedRuntimeEventCredentialRequestV1;
use std::sync::Arc;

use super::common::unique_target_root;
use crate::platform::events::{catalog, credential, reconciliation, topology};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeEventCredentialHandler, ManagedRuntimeExpectation,
};

#[path = "event_topology/authority_fixture.rs"]
mod authority_fixture;
use authority_fixture::CapturingAuthorityRelay;

#[test]
fn approved_catalog_builds_deterministic_exact_event_topology() {
    let (root, _, first, second, _) = event_topology_fixture();

    assert_eq!(first, second);
    assert_eq!(first.streams().len(), 1);
    assert_eq!(first.streams()[0].kind().subject_token(), "event");
    assert_eq!(first.streams()[0].max_bytes(), 1_048_576);
    assert_eq!(first.streams()[0].max_age_millis(), 3_600_000);
    assert_eq!(first.streams()[0].replicas(), 1);
    assert_eq!(first.publishers().len(), 1);
    assert_eq!(
        first.publishers()[0].subject().as_str(),
        "makosh.event.v1.owner_notes.changed.v1"
    );
    assert_eq!(first.consumers().len(), 1);
    assert_eq!(first.consumers()[0].max_in_flight(), 32);
    assert_eq!(first.consumers()[0].delivery_policy().max_deliver(), 3);
    assert_eq!(
        first.consumers()[0].delivery_policy().ack_wait_millis(),
        2_000
    );
    assert_eq!(
        first.consumers()[0].subject().as_str(),
        "makosh.event.v1.owner_notes.changed.v1"
    );
    assert!(first.consumers()[0].durable_name().starts_with("event-"));
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn event_hub_recovery_snapshot_is_exact_and_bound_to_restored_control_state() {
    let (root, store, _, _, _) = event_topology_fixture();
    let snapshot = reconciliation::recovery_snapshot(&store).expect("capture topology");

    reconciliation::validate_recovery_snapshot(&store, &snapshot).expect("validate exact topology");
    let mut corrupt = snapshot.clone();
    let last = corrupt.last_mut().expect("snapshot bytes");
    *last ^= 1;
    assert!(reconciliation::validate_recovery_snapshot(&store, &corrupt).is_err());

    let replacement = PlatformEventHubTopologyV1::new(
        2,
        "nats://127.0.0.1:4222",
        "makosh_event_hub",
        2,
        event_hub_topology().stream_budgets().to_vec(),
    );
    store
        .record_platform_event_hub_topology(&replacement)
        .expect("replace topology");
    assert!(reconciliation::validate_recovery_snapshot(&store, &snapshot).is_err());
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn kernel_derives_event_credential_subjects_only_from_current_approved_topology() {
    let (root, _, topology, _, registration) = event_topology_fixture();
    let request = credential::permit::derive_credential_request(
        credential::permit::EventCredentialRequestInputV1 {
            registration: &registration,
            runtime_instance_id: "runtime_1",
            runtime_generation: 3,
            credential_revision: 2,
            request_id: [8; 16],
            recipient_public_key_x25519: [9; 32],
            ttl_seconds: 60,
            topology: &topology,
        },
    )
    .expect("credential request");

    assert_eq!(request.logical_owner_id, "owner_notes");
    assert_eq!(request.registration_id, "registration_notes");
    assert_eq!(request.grant_epoch, 2);
    assert_eq!(
        request.publish_subjects,
        ["makosh.event.v1.owner_notes.changed.v1"]
    );
    assert!(request.subscribe_subjects.is_empty());
    let stale = approved_registration("registration_notes", "module_notes", "owner_notes", 3);
    assert_eq!(
        credential::permit::derive_credential_request(
            credential::permit::EventCredentialRequestInputV1 {
                registration: &stale,
                runtime_instance_id: "runtime_1",
                runtime_generation: 3,
                credential_revision: 2,
                request_id: [8; 16],
                recipient_public_key_x25519: [9; 32],
                ttl_seconds: 60,
                topology: &topology,
            },
        ),
        Err(credential::permit::EventCredentialPermitErrorV1::NoApprovedRoute)
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn kernel_derives_exact_durable_consumer_grants_from_current_approved_topology() {
    let (root, _, topology, _, _) = event_topology_fixture();
    let registration =
        approved_registration("registration_search", "module_search", "owner_search", 2);
    let request = credential::permit::derive_credential_request(
        credential::permit::EventCredentialRequestInputV1 {
            registration: &registration,
            runtime_instance_id: "runtime_1",
            runtime_generation: 3,
            credential_revision: 2,
            request_id: [8; 16],
            recipient_public_key_x25519: [9; 32],
            ttl_seconds: 60,
            topology: &topology,
        },
    )
    .expect("credential request");

    assert!(request.publish_subjects.is_empty());
    assert!(request.subscribe_subjects.is_empty());
    assert_eq!(request.consumer_grants.len(), 1);
    assert_eq!(
        request.consumer_grants[0].filter_subject,
        topology.consumers()[0].subject().as_str()
    );
    assert_eq!(
        request.consumer_grants[0].durable_name,
        topology.consumers()[0].durable_name()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn kernel_derives_scheduler_receipt_bindings_only_from_approved_topology() {
    let (root, store) = scheduler_receipt_fixture();
    store
        .record_platform_event_hub_topology(&event_hub_topology())
        .expect("record multi-stream Event Hub topology");
    let contracts = catalog::resolve_contracts(&*store).expect("resolve catalog");
    let topology = topology::plan(&contracts, &event_hub_topology()).expect("build topology");
    let recovery_snapshot =
        reconciliation::recovery_snapshot(&store).expect("capture multi-stream topology");
    reconciliation::validate_recovery_snapshot(&store, &recovery_snapshot)
        .expect("multi-stream topology must satisfy the Events authority wire contract");

    let bindings = topology::scheduler_receipt_bindings(&topology, "scheduler_runtime", 2)
        .expect("Scheduler receipt bindings");

    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].stream_name, "MAKOSH_ACK_V1");
    assert_eq!(
        bindings[0].filter_subject,
        "makosh.ack.v1.owner_notes.job_receipt.v1"
    );
    assert_eq!(bindings[1].stream_name, "MAKOSH_RESULT_V1");
    assert_eq!(
        bindings[1].filter_subject,
        "makosh.result.v1.owner_notes.job_receipt.v1"
    );
    assert_eq!(
        topology::scheduler_dispatch_bindings(&topology, "scheduler_runtime", 2)
            .expect("Scheduler dispatch bindings"),
        [
            makosh_runtime_protocol::v1::SchedulerRuntimeDispatchPublisherBindingV1 {
                subject: "makosh.command.v1.owner_notes.sync_job.v1".to_owned(),
            }
        ]
    );
    assert_eq!(
        topology::scheduler_receipt_bindings(&topology, "scheduler_runtime", 3),
        Err(topology::SchedulerReceiptTopologyErrorV1::Unavailable)
    );
    assert_eq!(
        topology::scheduler_dispatch_bindings(&topology, "scheduler_runtime", 3),
        Err(topology::SchedulerDispatchTopologyErrorV1::Unavailable)
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn kernel_relays_only_topology_derived_credential_request_to_the_authority_child() {
    let (root, _, topology, _, registration) = event_topology_fixture();
    let request = credential::permit::derive_credential_request(
        credential::permit::EventCredentialRequestInputV1 {
            registration: &registration,
            runtime_instance_id: "runtime_1",
            runtime_generation: 3,
            credential_revision: 2,
            request_id: [8; 16],
            recipient_public_key_x25519: [9; 32],
            ttl_seconds: 60,
            topology: &topology,
        },
    )
    .expect("credential request");
    let relay = credential::authority::EventAuthorityCredentialRelayV1::new(
        "events_authority".to_owned(),
        CapturingAuthorityRelay,
    )
    .expect("authority relay");
    let delivery = relay.issue(request).expect("opaque delivery");

    assert_eq!(delivery.encapped_key, vec![1; 32]);
    assert_eq!(delivery.ciphertext, vec![2; 32]);
    assert_eq!(delivery.tag, vec![3; 16]);
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn managed_runtime_event_request_is_fenced_and_returns_only_authority_ciphertext() {
    let (root, store, _, _, _) = event_topology_fixture();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            "registration_notes",
            1,
            "distribution-notes",
            "runtime-notes",
            [2; 32],
            [1; 32],
            None,
        ))
        .expect("record current managed binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            "registration_notes",
            "runtime_1",
            1,
            1,
            3,
            2,
        ))
        .expect("record current managed launch");
    let handler = credential::handler::EventCredentialHandlerV1::new(
        Arc::clone(&store),
        "events_authority".to_owned(),
        CapturingAuthorityRelay,
    )
    .expect("credential handler");
    let expectation = ManagedRuntimeExpectation::new(
        "registration_notes",
        "runtime_1",
        "module_notes",
        3,
        2,
        [1; 32],
        None,
    );
    let request = ManagedRuntimeEventCredentialRequestV1 {
        request_id: vec![8; 16],
        credential_revision: 1,
        ttl_seconds: 60,
        recipient_public_key_x25519: vec![9; 32],
    };
    let delivery = handler
        .issue_event_credential(&expectation, request.clone())
        .expect("opaque delivery");

    assert_eq!(delivery.encapped_key, vec![1; 32]);
    assert_eq!(delivery.ciphertext, vec![2; 32]);
    assert_eq!(delivery.tag, vec![3; 16]);
    store
        .transition_module_registration("registration_notes", ModuleRegistrationState::Revoked)
        .expect("revoke registration");
    assert_eq!(
        handler
            .issue_event_credential(&expectation, request)
            .expect_err("revoked registration"),
        "managed runtime Events credential fence is stale"
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn event_topology_fixture() -> (
    std::path::PathBuf,
    Arc<SqliteControlStore>,
    topology::EventTopologyPlanV1,
    topology::EventTopologyPlanV1,
    ModuleRegistration,
) {
    let root = unique_target_root("makosh-event-topology");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
            .expect("create Control Store"),
    );
    register(
        &store,
        registration("registration_notes", "module_notes", "owner_notes", [1; 32]),
        ModuleEventRouteDirectionV1::Publish,
        "events.publish",
    );
    register(
        &store,
        registration(
            "registration_search",
            "module_search",
            "owner_search",
            [2; 32],
        ),
        ModuleEventRouteDirectionV1::Consume,
        "events.consume",
    );

    let contracts = catalog::resolve_contracts(&*store).expect("resolve catalog");
    let configuration = event_hub_topology();
    store
        .record_platform_event_hub_topology(&configuration)
        .expect("record Event Hub topology");
    let first = topology::plan(&contracts, &configuration).expect("build topology");
    let second = topology::plan(&contracts, &configuration).expect("rebuild topology");
    (
        root,
        store,
        first,
        second,
        approved_registration("registration_notes", "module_notes", "owner_notes", 2),
    )
}

fn scheduler_receipt_fixture() -> (std::path::PathBuf, Arc<SqliteControlStore>) {
    let root = unique_target_root("makosh-scheduler-receipt-topology");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
            .expect("create Control Store"),
    );
    let registration = registration(
        "scheduler_runtime",
        "scheduler_runtime",
        "scheduler",
        [8; 32],
    );
    let requests = [
        scheduler_dispatch_route(registration.registration_id(), "scheduler.dispatches"),
        scheduler_receipt_route(
            registration.registration_id(),
            "scheduler.receipts.acceptance",
            ModuleEventEnvelopeKindV1::Ack,
        ),
        scheduler_receipt_route(
            registration.registration_id(),
            "scheduler.receipts.terminal",
            ModuleEventEnvelopeKindV1::Result,
        ),
    ];
    let capabilities = requests
        .iter()
        .map(|request| request.capability_id().to_owned())
        .collect::<Vec<_>>();
    store
        .create_pending_registration_with_requests(
            &registration,
            &capabilities,
            &[],
            &requests,
            &[],
        )
        .expect("persist Scheduler routes");
    store
        .approve_module_registration(registration.registration_id(), &capabilities)
        .expect("approve Scheduler routes");
    (root, store)
}

fn scheduler_dispatch_route(
    registration_id: &str,
    capability_id: &str,
) -> ModuleEventRouteRequestV1 {
    ModuleEventRouteRequestV1::new(
        makosh_kernel_control_store::ModuleEventRouteRequestInputV1 {
            registration_id: registration_id.to_owned(),
            capability_id: capability_id.to_owned(),
            envelope_kind: ModuleEventEnvelopeKindV1::Command,
            contract_owner: "owner_notes".to_owned(),
            contract_name: "sync_job".to_owned(),
            contract_major: 1,
            contract_revision: 1,
            contract_schema_sha256: [8; 32],
            direction: ModuleEventRouteDirectionV1::Publish,
            max_in_flight: 32,
            delivery_policy: None,
        },
    )
}

fn scheduler_receipt_route(
    registration_id: &str,
    capability_id: &str,
    envelope_kind: ModuleEventEnvelopeKindV1,
) -> ModuleEventRouteRequestV1 {
    ModuleEventRouteRequestV1::new(
        makosh_kernel_control_store::ModuleEventRouteRequestInputV1 {
            registration_id: registration_id.to_owned(),
            capability_id: capability_id.to_owned(),
            envelope_kind,
            contract_owner: "owner_notes".to_owned(),
            contract_name: "job_receipt".to_owned(),
            contract_major: 1,
            contract_revision: 1,
            contract_schema_sha256: [8; 32],
            direction: ModuleEventRouteDirectionV1::Consume,
            max_in_flight: 32,
            delivery_policy: delivery_policy(ModuleEventRouteDirectionV1::Consume),
        },
    )
}

fn event_hub_topology() -> PlatformEventHubTopologyV1 {
    let kinds = [
        ModuleEventEnvelopeKindV1::Command,
        ModuleEventEnvelopeKindV1::Event,
        ModuleEventEnvelopeKindV1::Observation,
        ModuleEventEnvelopeKindV1::Result,
        ModuleEventEnvelopeKindV1::Ack,
    ];
    PlatformEventHubTopologyV1::new(
        1,
        "nats://127.0.0.1:4222",
        "event_hub",
        1,
        kinds
            .into_iter()
            .map(|kind| PlatformEventStreamBudgetV1::new(kind, 1_048_576, 3_600_000, 1))
            .collect(),
    )
}

fn register(
    store: &SqliteControlStore,
    registration: ModuleRegistration,
    direction: ModuleEventRouteDirectionV1,
    capability_id: &str,
) {
    let route = ModuleEventRouteRequestV1::new(
        makosh_kernel_control_store::ModuleEventRouteRequestInputV1 {
            registration_id: registration.registration_id().to_owned(),
            capability_id: capability_id.to_owned(),
            envelope_kind: ModuleEventEnvelopeKindV1::Event,
            contract_owner: "owner_notes".to_owned(),
            contract_name: "changed".to_owned(),
            contract_major: 1,
            contract_revision: 1,
            contract_schema_sha256: [7; 32],
            direction,
            max_in_flight: 32,
            delivery_policy: delivery_policy(direction),
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
        .expect("persist event route");
    store
        .approve_module_registration(registration.registration_id(), &[capability_id.to_owned()])
        .expect("approve event route");
}

fn delivery_policy(direction: ModuleEventRouteDirectionV1) -> Option<ModuleEventDeliveryPolicyV1> {
    match direction {
        ModuleEventRouteDirectionV1::Publish => None,
        ModuleEventRouteDirectionV1::Consume => Some(ModuleEventDeliveryPolicyV1::new(
            ModuleEventSubscriptionRequirementV1::Required,
            3,
            2_000,
        )),
    }
}

fn registration(
    registration_id: &str,
    module_id: &str,
    owner_id: &str,
    descriptor_digest: [u8; 32],
) -> ModuleRegistration {
    ModuleRegistration::new(
        registration_id,
        module_id,
        owner_id,
        descriptor_digest,
        ModuleRegistrationState::Pending,
        1,
    )
}

fn approved_registration(
    registration_id: &str,
    module_id: &str,
    owner_id: &str,
    grant_epoch: u64,
) -> ModuleRegistration {
    ModuleRegistration::new(
        registration_id,
        module_id,
        owner_id,
        [1; 32],
        ModuleRegistrationState::Approved,
        grant_epoch,
    )
}
