use std::sync::Arc;

use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, InitialOwnerIdentity, ManagedLaunchRecord,
    ModuleClientRealtimeContractVersionV1, ModuleClientRealtimeRouteV1,
    ModuleDescriptorRegistrationRequestsV1, ModuleRegistration, ModuleRegistrationState,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ManagedRuntimeClientRealtimePublishRequestV1,
};

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeClientRealtimeHandler, ManagedRuntimeExpectation,
};

use super::common::unique_target_root;

const REGISTRATION: &str = "delivery-intent";
const MODULE: &str = "makosh-communication-delivery-intent-runtime";
const OWNER: &str = "communication_delivery_intent";
const LOGICAL_OWNER: &str = "owner_local";
const CAPABILITY: &str = "communication.delivery_intent.v1";

#[test]
fn managed_realtime_publication_is_exact_owner_fenced_and_idempotent() {
    let fixture = RealtimeRouteFixture::new("makosh-client-realtime-route");
    let source = InMemoryBrowserRealtimeSource::new(8).expect("source");
    let handler = ClientRealtimePublishHandlerV1::new(Arc::clone(&fixture.store), source);
    let request = publication(LOGICAL_OWNER, b"status-v1");

    let accepted = handler
        .publish_client_realtime(&fixture.expectation, request.clone())
        .expect("publish exact owner event");
    assert_eq!(accepted.accepted_cursor, "delivery-intent/1");
    assert_eq!(
        handler
            .publish_client_realtime(&fixture.expectation, request.clone())
            .expect("exact duplicate")
            .accepted_cursor,
        "delivery-intent/1"
    );

    let mut conflict = request.clone();
    conflict.payload = b"status-v2".to_vec();
    assert!(
        handler
            .publish_client_realtime(&fixture.expectation, conflict)
            .expect_err("cursor conflict")
            .contains("cursor conflicts")
    );

    let mut foreign_owner = request.clone();
    foreign_owner.logical_owner_id = "owner_other".to_owned();
    assert_eq!(
        handler
            .publish_client_realtime(&fixture.expectation, foreign_owner)
            .expect_err("foreign logical owner"),
        "managed ClientRealtime logical owner is prohibited"
    );

    let stale = ManagedRuntimeExpectation::new(
        REGISTRATION,
        "delivery-runtime",
        MODULE,
        2,
        fixture.expectation.grant_epoch(),
        [7; 32],
        None,
    );
    assert_eq!(
        handler
            .publish_client_realtime(&stale, request.clone())
            .expect_err("stale generation"),
        "managed ClientRealtime publisher fence is stale"
    );

    fixture
        .store
        .transition_module_registration(REGISTRATION, ModuleRegistrationState::Revoked)
        .expect("revoke publisher");
    assert!(
        handler
            .publish_client_realtime(&fixture.expectation, request)
            .is_err(),
        "revoked publisher must fail closed"
    );
    std::fs::remove_dir_all(fixture.root).expect("remove fixture");
}

struct RealtimeRouteFixture {
    root: std::path::PathBuf,
    store: Arc<SqliteControlStore>,
    expectation: ManagedRuntimeExpectation,
}

impl RealtimeRouteFixture {
    fn new(prefix: &str) -> Self {
        let root = unique_target_root(prefix);
        std::fs::create_dir_all(&root).expect("create fixture");
        let store = Arc::new(
            SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
                .expect("create Control Store"),
        );
        store
            .claim_initial_owner(&InitialOwnerIdentity::new(
                LOGICAL_OWNER,
                "device_local",
                [4; 65],
            ))
            .expect("claim owner");
        let registration = ModuleRegistration::new(
            REGISTRATION,
            MODULE,
            OWNER,
            [7; 32],
            ModuleRegistrationState::Pending,
            1,
        );
        let route = ModuleClientRealtimeRouteV1::new(
            REGISTRATION,
            CAPABILITY,
            OWNER,
            "communication.delivery_intent.status_changed",
            ModuleClientRealtimeContractVersionV1 {
                major: 1,
                revision: 1,
            },
            [9; 32],
        );
        store
            .create_pending_registration_with_all_descriptor_requests(
                &registration,
                &[CAPABILITY.to_owned()],
                ModuleDescriptorRegistrationRequestsV1 {
                    storage: &[],
                    events: &[],
                    blobs: &[],
                    scheduler: &[],
                    vault_purposes: &[],
                    client_rpc_routes: &[],
                    client_blob_routes: &[],
                    client_realtime_routes: std::slice::from_ref(&route),
                    query_rpc_routes: &[],
                    request_rpc_routes: &[],
                    contract_dependencies: &[],
                },
            )
            .expect("register realtime route");
        let grants = store
            .approve_module_registration(REGISTRATION, &[CAPABILITY.to_owned()])
            .expect("approve route");
        store
            .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
                REGISTRATION,
                1,
                "distribution-1",
                "runtime-artifact-1",
                [8; 32],
                [7; 32],
                None,
            ))
            .expect("record binding");
        store
            .record_managed_launch(&ManagedLaunchRecord::new(
                REGISTRATION,
                "delivery-runtime",
                1,
                1,
                1,
                grants.grant_epoch(),
            ))
            .expect("record launch");
        let expectation = ManagedRuntimeExpectation::new(
            REGISTRATION,
            "delivery-runtime",
            MODULE,
            1,
            grants.grant_epoch(),
            [7; 32],
            None,
        );
        Self {
            root,
            store,
            expectation,
        }
    }
}

fn publication(
    logical_owner_id: &str,
    payload: &[u8],
) -> ManagedRuntimeClientRealtimePublishRequestV1 {
    ManagedRuntimeClientRealtimePublishRequestV1 {
        contract: Some(ContractReferenceV1 {
            owner: OWNER.to_owned(),
            name: "communication.delivery_intent.status_changed".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![9; 32],
        }),
        logical_owner_id: logical_owner_id.to_owned(),
        event_id: vec![1; 16],
        cursor: "delivery-intent/1".to_owned(),
        event_kind: "delivery_intent_status_changed".to_owned(),
        occurred_at_unix_millis: 1_000,
        causation_id: String::new(),
        correlation_id: String::new(),
        trace_id: String::new(),
        payload: payload.to_vec(),
    }
}
