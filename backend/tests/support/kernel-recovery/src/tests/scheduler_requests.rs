use makosh_kernel_control_store::{
    InitialOwnerIdentity, ModuleRegistration, ModuleRegistrationState, ModuleSchedulerJobRequestV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ContractReferenceV1,
    ModuleDescriptorV1, ModuleKindV1, SchedulerJobRequestV1, UpsertSchedulerScheduleRequestV1,
    capability_request_v1::Request,
};
use prost::Message;

use crate::modules::registration::registry;
use crate::platform::scheduler::{admission as scheduler_admission, catalog};

use super::common::unique_target_root;

#[test]
fn control_store_retains_owner_bound_scheduler_job_contracts_atomically() {
    let root = unique_target_root("makosh-scheduler-job-request");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let request = scheduler_request("scheduler.jobs");

    store
        .create_pending_registration_with_descriptor_requests(
            &registration(),
            &["scheduler.jobs".to_owned()],
            makosh_kernel_control_store::ModuleDescriptorRegistrationRequestsV1 {
                storage: &[],
                events: &[],
                blobs: &[],
                scheduler: std::slice::from_ref(&request),
                vault_purposes: &[],
                client_rpc_routes: &[],
                client_blob_routes: &[],
                client_realtime_routes: &[],
                query_rpc_routes: &[],
                request_rpc_routes: &[],
                contract_dependencies: &[],
            },
        )
        .expect("persist pending registration and Scheduler request together");

    assert_eq!(
        store
            .module_scheduler_job_requests("registration_notes", "scheduler.jobs")
            .expect("read retained Scheduler request"),
        vec![request.clone()]
    );
    assert!(
        catalog::resolve(&store)
            .expect("resolve pending Scheduler catalog")
            .is_empty()
    );
    store
        .approve_module_registration("registration_notes", &["scheduler.jobs".to_owned()])
        .expect("approve Scheduler capability");
    let entries = catalog::resolve(&store).expect("resolve approved Scheduler catalog");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].registration_id(), "registration_notes");
    assert_eq!(entries[0].module_id(), "module_notes");
    assert_eq!(entries[0].grant_epoch(), 2);
    assert_eq!(entries[0].capability_id(), "scheduler.jobs");
    assert_eq!(entries[0].request(), &request);
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn control_store_rejects_foreign_or_duplicate_scheduler_job_contracts_atomically() {
    let root = unique_target_root("makosh-scheduler-job-request-invalid");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let request = scheduler_request("scheduler.jobs");
    let foreign = ModuleSchedulerJobRequestV1::new(
        "registration_notes",
        "scheduler.jobs",
        "owner_other",
        "refresh",
        1,
        1,
        [7; 32],
    );

    for requests in [vec![foreign], vec![request.clone(), request]] {
        assert!(
            store
                .create_pending_registration_with_descriptor_requests(
                    &registration(),
                    &["scheduler.jobs".to_owned()],
                    makosh_kernel_control_store::ModuleDescriptorRegistrationRequestsV1 {
                        storage: &[],
                        events: &[],
                        blobs: &[],
                        scheduler: &requests,
                        vault_purposes: &[],
                        client_rpc_routes: &[],
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
            .expect("registration remains absent after rejected request")
            .is_none()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn module_registration_retains_descriptor_declared_scheduler_job_contract() {
    let root = unique_target_root("makosh-scheduler-job-descriptor-registration");
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
        .expect("register Scheduler descriptor");
    assert_eq!(
        store
            .module_scheduler_job_requests(registration.registration_id(), "scheduler.jobs")
            .expect("read descriptor-declared Scheduler job request"),
        vec![ModuleSchedulerJobRequestV1::new(
            registration.registration_id(),
            "scheduler.jobs",
            "owner_notes",
            "refresh",
            1,
            1,
            [7; 32],
        )]
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn scheduler_schedule_admission_requires_the_current_exact_approved_job_contract() {
    let root = unique_target_root("makosh-scheduler-schedule-admission");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let request = scheduler_request("scheduler.jobs");
    store
        .create_pending_registration_with_descriptor_requests(
            &registration(),
            &["scheduler.jobs".to_owned()],
            makosh_kernel_control_store::ModuleDescriptorRegistrationRequestsV1 {
                storage: &[],
                events: &[],
                blobs: &[],
                scheduler: std::slice::from_ref(&request),
                vault_purposes: &[],
                client_rpc_routes: &[],
                client_blob_routes: &[],
                client_realtime_routes: &[],
                query_rpc_routes: &[],
                request_rpc_routes: &[],
                contract_dependencies: &[],
            },
        )
        .expect("persist Scheduler job request");
    store
        .approve_module_registration("registration_notes", &["scheduler.jobs".to_owned()])
        .expect("approve Scheduler job request");

    scheduler_admission::require_current_job_contract(&store, &schedule_upsert())
        .expect("admit exact approved Scheduler job contract");

    let mut stale = schedule_upsert();
    stale.contract_revision = 2;
    assert!(scheduler_admission::require_current_job_contract(&store, &stale).is_err());
    let mut foreign = schedule_upsert();
    foreign.job_owner = "other".to_owned();
    foreign.contract_name = "other.refresh".to_owned();
    assert!(scheduler_admission::require_current_job_contract(&store, &foreign).is_err());
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn schedule_upsert() -> UpsertSchedulerScheduleRequestV1 {
    UpsertSchedulerScheduleRequestV1 {
        schedule_id: vec![1; 16],
        schedule_revision: 1,
        job_owner: "owner_notes".to_owned(),
        job_name: "refresh".to_owned(),
        job_major: 1,
        contract_name: "owner_notes.refresh".to_owned(),
        contract_revision: 1,
        contract_schema_sha256: vec![7; 32],
        scope_id: "scope:opaque".to_owned(),
        concurrency_key: "scope:opaque".to_owned(),
        enabled: true,
        policy_canonical_bytes: vec![1],
        next_due_at_unix_millis: 1_000,
        updated_at_unix_millis: 1_000,
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

fn scheduler_request(capability_id: &str) -> ModuleSchedulerJobRequestV1 {
    ModuleSchedulerJobRequestV1::new(
        "registration_notes",
        capability_id,
        "owner_notes",
        "refresh",
        1,
        1,
        [7; 32],
    )
}

fn descriptor() -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "module_notes".to_owned(),
        owner_id: "owner_notes".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "build".to_owned(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: "scheduler.jobs".to_owned(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Required as i32,
            requests: vec![CapabilityRequestV1 {
                request: Some(Request::SchedulerJob(SchedulerJobRequestV1 {
                    job_kind: Some(ContractReferenceV1 {
                        owner: "owner_notes".to_owned(),
                        name: "refresh".to_owned(),
                        major: 1,
                        revision: 1,
                        schema_sha256: vec![7; 32],
                    }),
                })),
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}
