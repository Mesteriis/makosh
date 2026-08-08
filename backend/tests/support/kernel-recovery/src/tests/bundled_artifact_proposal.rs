use makosh_kernel_control_store::{
    BundledManagedArtifactProposalInputV1, ModuleDescriptorRegistrationRequestsV1,
    ModuleRegistration, ModuleRegistrationState, OperationIdV1,
};
use makosh_kernel_control_store_sqlite::{SqliteControlStore, StoreError};

use super::common::unique_target_root;

#[test]
fn bundled_artifact_proposal_is_atomic_idempotent_and_pending() {
    let root = unique_target_root("makosh-bundled-artifact-proposal");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let proposal = BundledManagedArtifactProposalInputV1::new(
        OperationIdV1::new([7; 16]),
        [8; 32],
        "makosh-development",
        1,
        "communications.runtime.v1",
    );
    let original = registration("registration-original");
    let created = store
        .propose_bundled_managed_artifact(
            &proposal,
            &original,
            &["communications.query".to_owned()],
            empty_requests(),
        )
        .expect("create bundled artifact proposal");
    assert!(!created.replayed());
    assert_eq!(created.registration(), &original);
    assert_eq!(
        store
            .module_grant_snapshot(original.registration_id())
            .expect("read grant snapshot")
            .expect("registration exists")
            .effective_grants(),
        None
    );

    let replay = store
        .propose_bundled_managed_artifact(
            &proposal,
            &registration("registration-must-not-be-created"),
            &["communications.query".to_owned()],
            empty_requests(),
        )
        .expect("replay bundled artifact proposal");
    assert!(replay.replayed());
    assert_eq!(replay.registration(), &original);
    assert!(
        store
            .module_registration("registration-must-not-be-created")
            .expect("read unused registration")
            .is_none()
    );

    let conflicting = BundledManagedArtifactProposalInputV1::new(
        OperationIdV1::new([7; 16]),
        [9; 32],
        "makosh-development",
        1,
        "communications.runtime.v1",
    );
    assert!(matches!(
        store.propose_bundled_managed_artifact(
            &conflicting,
            &registration("registration-conflicting"),
            &["communications.query".to_owned()],
            empty_requests(),
        ),
        Err(StoreError::OperationRequestDigestConflict)
    ));
    assert!(
        store
            .module_registration("registration-conflicting")
            .expect("read conflicting registration")
            .is_none()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn registration(registration_id: &str) -> ModuleRegistration {
    ModuleRegistration::new(
        registration_id,
        "communications",
        "communications",
        [4; 32],
        ModuleRegistrationState::Pending,
        1,
    )
}

fn empty_requests() -> ModuleDescriptorRegistrationRequestsV1<'static> {
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
