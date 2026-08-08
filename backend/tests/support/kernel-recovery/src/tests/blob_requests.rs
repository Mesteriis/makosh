use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, InitialOwnerIdentity, ManagedLaunchRecord, ModuleBlobOperationV1,
    ModuleBlobQuotaRequestV1, ModuleRegistration, ModuleRegistrationState,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    BlobDataOperationV1, BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1,
    CapabilityDescriptorV1, CapabilityRequestV1, ManagedRuntimeBlobSessionRequestV1,
    ModuleDescriptorV1, ModuleKindV1, capability_request_v1::Request,
};
use prost::Message;

use crate::modules::registration::registry;
use crate::platform::blob::catalog;
use crate::platform::blob::session::BlobSessionHandlerV1;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeExpectation,
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

use super::common::unique_target_root;

const CUSTODY_SCOPE_ID: &str = "notes.content.v1";

#[test]
fn blob_quotas_become_visible_only_after_exact_capability_approval() {
    let root = unique_target_root("makosh-blob-quota-request");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let request = blob_request("blob.content", 16 * 1024 * 1024);

    store
        .create_pending_registration_with_requests(
            &registration(),
            &["blob.content".to_owned(), "events.publish".to_owned()],
            &[],
            &[],
            std::slice::from_ref(&request),
        )
        .expect("persist pending registration and Blob request together");
    assert_eq!(
        store
            .module_blob_quota_request("registration_notes", "blob.content")
            .expect("read retained Blob request"),
        Some(request.clone())
    );
    assert!(
        catalog::resolve(&store)
            .expect("pending registration has no Blob catalog")
            .is_empty()
    );

    store
        .approve_module_registration("registration_notes", &["blob.content".to_owned()])
        .expect("approve Blob capability");
    let entries = catalog::resolve(&store).expect("resolve approved Blob catalog");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].registration_id(), "registration_notes");
    assert_eq!(entries[0].module_id(), "module_notes");
    assert_eq!(entries[0].grant_epoch(), 2);
    assert_eq!(entries[0].capability_id(), "blob.content");
    assert_eq!(entries[0].request(), &request);
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_rejects_invalid_or_unrequested_blob_quotas_atomically() {
    let root = unique_target_root("makosh-blob-quota-request-invalid");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");

    for request in [
        blob_request("unrequested", 1),
        blob_request("blob.content", 0),
        blob_request("blob.content", (1 << 40) + 1),
        ModuleBlobQuotaRequestV1::new(
            "registration_notes",
            "blob.content",
            "owner_notes",
            1,
            "",
            vec![ModuleBlobOperationV1::Write],
        ),
        ModuleBlobQuotaRequestV1::new(
            "registration_notes",
            "blob.content",
            "owner_notes",
            1,
            CUSTODY_SCOPE_ID,
            Vec::new(),
        ),
        ModuleBlobQuotaRequestV1::new(
            "registration_notes",
            "blob.content",
            "owner_notes",
            1,
            CUSTODY_SCOPE_ID,
            vec![ModuleBlobOperationV1::Write, ModuleBlobOperationV1::Write],
        ),
    ] {
        assert!(
            store
                .create_pending_registration_with_requests(
                    &registration(),
                    &["blob.content".to_owned()],
                    &[],
                    &[],
                    &[request],
                )
                .is_err()
        );
    }
    assert!(
        store
            .module_registration("registration_notes")
            .expect("registration remains absent after rejected request")
            .is_none()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn kernel_denies_a_blob_operation_not_declared_by_the_capability() {
    let root = unique_target_root("makosh-blob-operation-scope");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
            .expect("create Control Store"),
    );
    let request = ModuleBlobQuotaRequestV1::new(
        "registration_notes",
        "blob.content",
        "owner_notes",
        1024,
        CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::ReadRange],
    );
    store
        .create_pending_registration_with_requests(
            &registration(),
            &["blob.content".to_owned()],
            &[],
            &[],
            std::slice::from_ref(&request),
        )
        .expect("record read-only Blob capability");
    let grant_epoch = store
        .approve_module_registration("registration_notes", &["blob.content".to_owned()])
        .expect("approve read-only Blob capability")
        .grant_epoch();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            "registration_notes",
            1,
            "distribution",
            "module_notes",
            [2; 32],
            [1; 32],
            None,
        ))
        .expect("record managed binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            "registration_notes",
            "runtime-notes",
            1,
            1,
            1,
            grant_epoch,
        ))
        .expect("record current managed runtime");
    let supervisor = ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false)));
    let handler =
        BlobSessionHandlerV1::new(Arc::clone(&store), supervisor.relay_port(), root.clone());
    let result = handler.issue_blob_session(
        &ManagedRuntimeExpectation::new(
            "registration_notes",
            "runtime-notes",
            "module_notes",
            1,
            grant_epoch,
            [1; 32],
            None,
        ),
        ManagedRuntimeBlobSessionRequestV1 {
            request_id: vec![1; 16],
            capability_id: "blob.content".to_owned(),
            operation: BlobDataOperationV1::BlobDataOperationWriteV1 as u32,
            channel_binding_sha256: vec![2; 32],
            reference_id: vec![3; 16],
            declared_size: 1,
            backup_class: 1,
            ttl_seconds: 30,
            ..Default::default()
        },
    );
    assert_eq!(
        result.expect_err("read-only capability must not receive a write grant"),
        "managed runtime Blob session request is denied",
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn module_registration_retains_a_descriptor_declared_blob_quota() {
    let root = unique_target_root("makosh-blob-descriptor-registration");
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

    let registration = registry::register(&store, &descriptor(64 * 1024 * 1024).encode_to_vec())
        .expect("register Blob descriptor");
    assert_eq!(
        store
            .module_blob_quota_request(registration.registration_id(), "blob.content")
            .expect("read descriptor-declared Blob quota"),
        Some(ModuleBlobQuotaRequestV1::new(
            registration.registration_id(),
            "blob.content",
            "owner_notes",
            64 * 1024 * 1024,
            CUSTODY_SCOPE_ID,
            vec![
                ModuleBlobOperationV1::Write,
                ModuleBlobOperationV1::ReadRange,
            ],
        ))
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn module_registration_rejects_conflicting_quotas_for_one_custody_scope() {
    let root = unique_target_root("makosh-blob-conflicting-scope-quota");
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
    let mut descriptor = descriptor(64 * 1024 * 1024);
    descriptor.capabilities.push(CapabilityDescriptorV1 {
        capability_id: "blob.read".to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: 32 * 1024 * 1024,
                custody_scope_id: CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::ReadRange as i32],
            })),
        }],
        ..Default::default()
    });

    assert!(
        registry::register(&store, &descriptor.encode_to_vec()).is_err(),
        "one custody scope must not acquire conflicting quota buckets",
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

fn blob_request(capability_id: &str, max_bytes: u64) -> ModuleBlobQuotaRequestV1 {
    ModuleBlobQuotaRequestV1::new(
        "registration_notes",
        capability_id,
        "owner_notes",
        max_bytes,
        CUSTODY_SCOPE_ID,
        vec![
            ModuleBlobOperationV1::Write,
            ModuleBlobOperationV1::ReadRange,
        ],
    )
}

fn descriptor(max_bytes: u64) -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "module_notes".to_owned(),
        owner_id: "owner_notes".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "build".to_owned(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: "blob.content".to_owned(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Required as i32,
            requests: vec![CapabilityRequestV1 {
                request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                    max_bytes,
                    custody_scope_id: CUSTODY_SCOPE_ID.to_owned(),
                    allowed_operations: vec![
                        BlobQuotaOperationV1::Write as i32,
                        BlobQuotaOperationV1::ReadRange as i32,
                    ],
                })),
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}
