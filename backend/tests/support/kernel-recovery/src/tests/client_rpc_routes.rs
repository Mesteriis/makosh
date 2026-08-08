use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, InitialOwnerIdentity, ManagedLaunchRecord, ModuleClientRpcRouteV1,
    ModuleRegistration, ModuleRegistrationState,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, ClientRpcRouteV1, ContractReferenceV1,
    ModuleClientRequestV1, ModuleDescriptorV1, ModuleKindV1, ProvidedSurfaceKindV1,
    ProvidedSurfaceV1,
};
use prost::Message;

use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, ManagedRuntimeRelay, route_managed_client_request,
};
use crate::modules::registration::registry;

use super::common::unique_target_root;

#[test]
fn control_store_exposes_only_approved_owner_client_rpc_routes() {
    let root = unique_target_root("makosh-client-rpc-route");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(
            "owner_notes",
            "device_notes",
            [4; 65],
        ))
        .expect("claim initial owner");

    let registration = registry::register(&store, &descriptor().encode_to_vec())
        .expect("register descriptor-declared client route");
    assert!(
        store
            .approved_module_client_rpc_routes()
            .expect("read pending routes")
            .is_empty()
    );

    store
        .approve_module_registration(registration.registration_id(), &["notes.query".to_owned()])
        .expect("approve client route capability");
    assert_eq!(
        store
            .approved_module_client_rpc_routes()
            .expect("read approved routes"),
        vec![ModuleClientRpcRouteV1::new(
            registration.registration_id(),
            "notes.query",
            "owner_notes",
            "notes.query",
            makosh_kernel_control_store::ModuleClientRpcContractVersionV1 {
                major: 1,
                revision: 1,
            },
            [7; 32],
            "/makosh.notes.v1.NotesQueryService/Query",
        )],
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_rejects_foreign_or_duplicate_client_rpc_routes_atomically() {
    let root = unique_target_root("makosh-client-rpc-route-invalid");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let valid = client_route("owner_notes", "/makosh.notes.v1.NotesQueryService/Query");
    let foreign = client_route("owner_other", "/makosh.notes.v1.NotesQueryService/Query");

    for routes in [vec![foreign], vec![valid.clone(), valid]] {
        assert!(
            store
                .create_pending_registration_with_all_descriptor_requests(
                    &registration(),
                    &["notes.query".to_owned()],
                    makosh_kernel_control_store::ModuleDescriptorRegistrationRequestsV1 {
                        storage: &[],
                        events: &[],
                        blobs: &[],
                        scheduler: &[],
                        vault_purposes: &[],
                        client_rpc_routes: &routes,
                        client_blob_routes: &[],
                        client_realtime_routes: &[],
                        query_rpc_routes: &[],
                        request_rpc_routes: &[],
                        contract_dependencies: &[],
                    },
                )
                .is_err()
        );
    }
    assert!(
        store
            .module_registration("registration_notes")
            .expect("read registration")
            .is_none()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn managed_client_route_rejects_a_stale_runtime_generation_before_relay() {
    let (root, store, registration, grant_epoch, request) =
        managed_route_fixture("makosh-client-rpc-stale-generation");
    let route = ManagedCapabilityRouteRequest::new(
        &registration,
        "runtime-current",
        1,
        grant_epoch,
        "notes.query",
        &request,
    );

    assert_eq!(
        route_managed_client_request(&store, &UnreachableRelay, &route)
            .expect_err("stale runtime generation"),
        "managed runtime fence is stale"
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn managed_client_route_rejects_a_replaced_runtime_binding_before_relay() {
    let (root, store, registration, grant_epoch, request) =
        managed_route_fixture("makosh-client-rpc-replaced-binding");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            &registration,
            2,
            "distribution-notes-v2",
            "runtime-notes-v2",
            [9; 32],
            *store
                .module_registration(&registration)
                .expect("read registration")
                .expect("managed registration")
                .descriptor_sha256(),
            None,
        ))
        .expect("replace managed launch binding");
    let route = ManagedCapabilityRouteRequest::new(
        &registration,
        "runtime-current",
        2,
        grant_epoch,
        "notes.query",
        &request,
    );

    assert_eq!(
        route_managed_client_request(&store, &UnreachableRelay, &route)
            .expect_err("replaced runtime binding"),
        "managed runtime fence is stale"
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn managed_route_fixture(
    prefix: &str,
) -> (std::path::PathBuf, SqliteControlStore, String, u64, Vec<u8>) {
    let root = unique_target_root(prefix);
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(
            "owner_notes",
            "device_notes",
            [4; 65],
        ))
        .expect("claim initial owner");
    let descriptor_bytes = descriptor().encode_to_vec();
    let registration =
        registry::register(&store, &descriptor_bytes).expect("register managed client route");
    let grants = store
        .approve_module_registration(registration.registration_id(), &["notes.query".to_owned()])
        .expect("approve managed client route");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "distribution-notes",
            "runtime-notes",
            [8; 32],
            *registration.descriptor_sha256(),
            None,
        ))
        .expect("record managed launch binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            registration.registration_id(),
            "runtime-current",
            1,
            1,
            2,
            grants.grant_epoch(),
        ))
        .expect("record current managed generation");
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: "module_notes".to_owned(),
        owner_id: "owner_notes".to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: "owner_notes".to_owned(),
            name: "notes.query".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![7; 32],
        }),
        request_id: 1,
        request_payload: vec![1],
        logical_owner_id: "owner-local".to_owned(),
        authenticated_device_id: "device-local".to_owned(),
        authenticated_client_session_id: "session-local".to_owned(),
    }
    .encode_to_vec();
    (
        root,
        store,
        registration.registration_id().to_owned(),
        grants.grant_epoch(),
        request,
    )
}

struct UnreachableRelay;

impl ManagedRuntimeRelay for UnreachableRelay {
    fn relay(&self, _: &str, _: Vec<u8>) -> Result<Vec<u8>, String> {
        Err("managed runtime relay was reached".to_owned())
    }
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

fn client_route(owner: &str, path: &str) -> ModuleClientRpcRouteV1 {
    ModuleClientRpcRouteV1::new(
        "registration_notes",
        "notes.query",
        owner,
        "notes.query",
        makosh_kernel_control_store::ModuleClientRpcContractVersionV1 {
            major: 1,
            revision: 1,
        },
        [7; 32],
        path,
    )
}

fn descriptor() -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "module_notes".to_owned(),
        owner_id: "owner_notes".to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: "build".to_owned(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: "notes.query".to_owned(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Required as i32,
            provides: vec![ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRpc as i32,
                contract: Some(ContractReferenceV1 {
                    owner: "owner_notes".to_owned(),
                    name: "notes.query".to_owned(),
                    major: 1,
                    revision: 1,
                    schema_sha256: vec![7; 32],
                }),
                client_rpc_route: Some(ClientRpcRouteV1 {
                    path: "/makosh.notes.v1.NotesQueryService/Query".to_owned(),
                }),
                client_blob_route: None,
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}
