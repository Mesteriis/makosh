use makosh_kernel_control_store::{
    ModuleBlobOperationV1, ModuleBlobQuotaRequestV1, ModuleClientBlobContractVersionV1,
    ModuleClientBlobRouteV1, ModuleClientBlobTransportV1, ModuleDescriptorRegistrationRequestsV1,
    ModuleRegistration, ModuleRegistrationState,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use super::common::unique_target_root;

#[test]
fn control_store_exposes_only_approved_client_blob_routes() {
    let root = unique_target_root("makosh-client-blob-route");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let registration = registration();
    let route = client_blob_route(256 * 1024);
    let quota = blob_quota(vec![ModuleBlobOperationV1::ReadRange]);

    store
        .create_pending_registration_with_all_descriptor_requests(
            &registration,
            &["notes.content.v1".to_owned()],
            ModuleDescriptorRegistrationRequestsV1 {
                storage: &[],
                events: &[],
                blobs: std::slice::from_ref(&quota),
                scheduler: &[],
                vault_purposes: &[],
                client_rpc_routes: &[],
                client_blob_routes: std::slice::from_ref(&route),
                client_realtime_routes: &[],
                query_rpc_routes: &[],
                request_rpc_routes: &[],
                contract_dependencies: &[],
            },
        )
        .expect("create pending registration");
    assert!(
        store
            .approved_module_client_blob_routes()
            .expect("read pending routes")
            .is_empty()
    );

    store
        .approve_module_registration(
            registration.registration_id(),
            &["notes.content.v1".to_owned()],
        )
        .expect("approve client Blob route capability");
    assert_eq!(
        store
            .approved_module_client_blob_routes()
            .expect("read approved routes"),
        vec![route]
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_rejects_client_blob_route_without_matching_read_quota_atomically() {
    for (name, quota, route) in [
        (
            "write-only",
            blob_quota(vec![ModuleBlobOperationV1::Write]),
            client_blob_route(256 * 1024),
        ),
        (
            "undersized",
            blob_quota(vec![ModuleBlobOperationV1::ReadRange]),
            client_blob_route(256 * 1024 + 1),
        ),
    ] {
        let root = unique_target_root(&format!("makosh-client-blob-route-{name}"));
        std::fs::create_dir_all(&root).expect("create fixture directory");
        let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
            .expect("create Control Store");

        assert!(
            store
                .create_pending_registration_with_all_descriptor_requests(
                    &registration(),
                    &["notes.content.v1".to_owned()],
                    ModuleDescriptorRegistrationRequestsV1 {
                        storage: &[],
                        events: &[],
                        blobs: std::slice::from_ref(&quota),
                        scheduler: &[],
                        vault_purposes: &[],
                        client_rpc_routes: &[],
                        client_blob_routes: std::slice::from_ref(&route),
                        client_realtime_routes: &[],
                        query_rpc_routes: &[],
                        request_rpc_routes: &[],
                        contract_dependencies: &[],
                    },
                )
                .is_err()
        );
        assert!(
            store
                .module_registration("registration_notes")
                .expect("read registration")
                .is_none()
        );
        std::fs::remove_dir_all(root).expect("remove fixture directory");
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

fn blob_quota(allowed_operations: Vec<ModuleBlobOperationV1>) -> ModuleBlobQuotaRequestV1 {
    ModuleBlobQuotaRequestV1::new(
        "registration_notes",
        "notes.content.v1",
        "owner_notes",
        256 * 1024,
        "notes.content.v1",
        allowed_operations,
    )
}

fn client_blob_route(max_response_bytes: u64) -> ModuleClientBlobRouteV1 {
    ModuleClientBlobRouteV1::new(
        "registration_notes",
        "notes.content.v1",
        "owner_notes",
        "notes.content-read",
        ModuleClientBlobContractVersionV1 {
            major: 1,
            revision: 1,
        },
        [7; 32],
        ModuleClientBlobTransportV1 {
            path: "/api/blobs/notes/v1/content".to_owned(),
            max_response_bytes,
        },
    )
}
