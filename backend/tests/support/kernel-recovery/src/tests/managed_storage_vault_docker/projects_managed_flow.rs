//! Actual Projects lifecycle, outcomes, references, replay, restart, privacy and owner-RLS contour.

use super::*;

use std::time::{Duration, Instant};

use makosh_projects_api::{
    PROJECTS_CLIENT_CAPABILITY_ID_V1, PROJECTS_MODULE_ID_V1, PROJECTS_OWNER_ID_V1,
    client_wire::{
        AddProjectOutcomeRequestV1, AddProjectReferenceRequestV1, CreateProjectRequestV1,
        GetProjectRequestV1, ListProjectOutcomesRequestV1, ListProjectOutcomesResultV1,
        ListProjectReferencesRequestV1, ListProjectReferencesResultV1, ListProjectsRequestV1,
        ListProjectsResultV1, ProjectMutationResultV1, ProjectOutcomeStateV1,
        ProjectReferenceKindV1, ProjectStateV1, ProjectV1, RemoveProjectReferenceRequestV1,
        SetProjectOutcomeStateRequestV1, SetProjectStateRequestV1, TimestampV1,
    },
    projects_client_add_outcome_contract_reference_v1,
    projects_client_add_reference_contract_reference_v1,
    projects_client_create_contract_reference_v1, projects_client_get_contract_reference_v1,
    projects_client_list_contract_reference_v1,
    projects_client_list_outcomes_contract_reference_v1,
    projects_client_list_references_contract_reference_v1,
    projects_client_remove_reference_contract_reference_v1,
    projects_client_set_outcome_state_contract_reference_v1,
    projects_client_set_state_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};

use crate::identity::device::signer::DeviceSigner;

const PRIVATE_PROJECT_TEXT_V1: &str = "projects-private-project-text";
const PRIVATE_OUTCOME_TEXT_V1: &str = "projects-private-outcome-text";
const PRIVATE_REFERENCE_LABEL_V1: &str = "projects-private-reference-label";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Projects binaries"]
fn managed_projects_lifecycle_replays_and_restarts_with_owner_rls() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-projects");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_projects_release_v1(&root);
    unsafe { std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel()) };
    let store = Arc::new(configured_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Projects owner signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            PROJECTS_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Projects owner");
    let admitted = admit_projects_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Projects Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_projects_runtime_v1(&supervisor, &store, admitted);
    record_communications_event_hub_topology_v1(&store);
    configure_communications_jetstream(&store);
    let projects = start_projects_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Projects clock")
        .as_secs() as i64;
    let at = |offset: i64| TimestampV1 {
        unix_seconds: now - 20 + offset,
        nanos: 0,
    };
    let future_at = |offset: i64| TimestampV1 {
        unix_seconds: now + offset,
        nanos: 0,
    };
    let create = CreateProjectRequestV1 {
        operation_id: vec![0x11; 16],
        logical_owner_id: String::new(),
        name: PRIVATE_PROJECT_TEXT_V1.to_owned(),
        description: "bounded owner-local metadata".to_owned(),
        start_at: Some(at(0)),
        target_at: Some(future_at(86_400)),
        created_at: Some(at(1)),
    };
    let first: ProjectMutationResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        1,
        projects_client_create_contract_reference_v1(),
        create.encode_to_vec(),
    );
    let replay: ProjectMutationResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        2,
        projects_client_create_contract_reference_v1(),
        create.encode_to_vec(),
    );
    assert_eq!(first, replay);
    let created = first.project.expect("created project");
    assert_eq!(created.project_revision, 1);
    assert_eq!(created.state, ProjectStateV1::ProjectStatePlanning as i32);

    let mut changed = create.clone();
    changed.description.push_str(" changed");
    assert_eq!(
        route_projects_response_v1(
            &store,
            &supervisor,
            &projects,
            3,
            projects_client_create_contract_reference_v1(),
            changed.encode_to_vec(),
        )
        .error_code,
        "CONFLICT"
    );

    let active: ProjectMutationResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        4,
        projects_client_set_state_contract_reference_v1(),
        SetProjectStateRequestV1 {
            operation_id: vec![0x12; 16],
            project_id: created.project_id.clone(),
            logical_owner_id: String::new(),
            expected_project_revision: 1,
            state: ProjectStateV1::ProjectStateActive as i32,
            changed_at: Some(at(2)),
        }
        .encode_to_vec(),
    );
    assert_eq!(active.project.expect("active").project_revision, 2);

    let outcome_added: ProjectMutationResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        5,
        projects_client_add_outcome_contract_reference_v1(),
        AddProjectOutcomeRequestV1 {
            operation_id: vec![0x13; 16],
            project_id: created.project_id.clone(),
            logical_owner_id: String::new(),
            expected_project_revision: 2,
            title: PRIVATE_OUTCOME_TEXT_V1.to_owned(),
            description: "expected result".to_owned(),
            target_at: Some(future_at(43_200)),
            changed_at: Some(at(3)),
        }
        .encode_to_vec(),
    );
    assert_eq!(outcome_added.project.expect("outcome").project_revision, 3);

    let reference_added: ProjectMutationResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        6,
        projects_client_add_reference_contract_reference_v1(),
        AddProjectReferenceRequestV1 {
            operation_id: vec![0x14; 16],
            project_id: created.project_id.clone(),
            logical_owner_id: String::new(),
            expected_project_revision: 3,
            kind: ProjectReferenceKindV1::ProjectReferenceKindDocument as i32,
            public_id: vec![0x31; 16],
            label: PRIVATE_REFERENCE_LABEL_V1.to_owned(),
            changed_at: Some(at(4)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        reference_added.project.expect("reference").project_revision,
        4
    );

    let outcomes: ListProjectOutcomesResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        7,
        projects_client_list_outcomes_contract_reference_v1(),
        ListProjectOutcomesRequestV1 {
            logical_owner_id: String::new(),
            project_id: created.project_id.clone(),
            after_outcome_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(outcomes.outcomes.len(), 1);
    let outcome_id = outcomes.outcomes[0].outcome_id.clone();
    let references: ListProjectReferencesResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        8,
        projects_client_list_references_contract_reference_v1(),
        ListProjectReferencesRequestV1 {
            logical_owner_id: String::new(),
            project_id: created.project_id.clone(),
            after_reference_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(references.references.len(), 1);

    let achieved: ProjectMutationResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        9,
        projects_client_set_outcome_state_contract_reference_v1(),
        SetProjectOutcomeStateRequestV1 {
            operation_id: vec![0x15; 16],
            project_id: created.project_id.clone(),
            logical_owner_id: String::new(),
            expected_project_revision: 4,
            outcome_id,
            expected_outcome_revision: 1,
            state: ProjectOutcomeStateV1::ProjectOutcomeStateAchieved as i32,
            changed_at: Some(at(5)),
        }
        .encode_to_vec(),
    );
    assert_eq!(achieved.project.expect("achieved").project_revision, 5);
    for (request_id, operation, revision, state, offset) in [
        (10, 0x16, 5, ProjectStateV1::ProjectStateCompleted, 6),
        (11, 0x17, 6, ProjectStateV1::ProjectStateArchived, 7),
        (12, 0x18, 7, ProjectStateV1::ProjectStateActive, 8),
    ] {
        let result: ProjectMutationResultV1 = route_projects_v1(
            &store,
            &supervisor,
            &projects,
            request_id,
            projects_client_set_state_contract_reference_v1(),
            SetProjectStateRequestV1 {
                operation_id: vec![operation; 16],
                project_id: created.project_id.clone(),
                logical_owner_id: String::new(),
                expected_project_revision: revision,
                state: state as i32,
                changed_at: Some(at(offset)),
            }
            .encode_to_vec(),
        );
        assert_eq!(
            result.project.expect("state").project_revision,
            revision + 1
        );
    }
    let removed: ProjectMutationResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        13,
        projects_client_remove_reference_contract_reference_v1(),
        RemoveProjectReferenceRequestV1 {
            operation_id: vec![0x19; 16],
            project_id: created.project_id.clone(),
            logical_owner_id: String::new(),
            expected_project_revision: 8,
            reference_id: references.references[0].reference_id.clone(),
            changed_at: Some(at(9)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        removed.project.expect("removed reference").project_revision,
        9
    );

    let second: ProjectMutationResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        14,
        projects_client_create_contract_reference_v1(),
        CreateProjectRequestV1 {
            operation_id: vec![0x21; 16],
            logical_owner_id: String::new(),
            name: "Second".to_owned(),
            description: String::new(),
            start_at: None,
            target_at: None,
            created_at: Some(at(10)),
        }
        .encode_to_vec(),
    );
    let second_id = second.project.expect("second").project_id;
    let page1: ListProjectsResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        15,
        projects_client_list_contract_reference_v1(),
        ListProjectsRequestV1 {
            logical_owner_id: String::new(),
            after_project_id: Vec::new(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let page2: ListProjectsResultV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        16,
        projects_client_list_contract_reference_v1(),
        ListProjectsRequestV1 {
            logical_owner_id: String::new(),
            after_project_id: page1.next_after_project_id.clone(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    let mut ids = vec![
        page1.projects[0].project_id.clone(),
        page2.projects[0].project_id.clone(),
    ];
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&created.project_id));
    assert!(ids.contains(&second_id));

    wait_for_projects_relay_v1();
    let before_restart = durable_projects_snapshot_v1();
    assert_eq!(before_restart, (2, 1, 1, 10, 10, 0));
    assert_public_projects_outbox_is_private_free_v1();
    let projects =
        restart_projects_runtime_v1(&supervisor, &store, &root.join("runtime"), projects);
    let restarted: ProjectV1 = route_projects_v1(
        &store,
        &supervisor,
        &projects,
        17,
        projects_client_get_contract_reference_v1(),
        GetProjectRequestV1 {
            logical_owner_id: String::new(),
            project_id: created.project_id,
        }
        .encode_to_vec(),
    );
    assert_eq!(restarted.project_revision, 9);
    assert_eq!(durable_projects_snapshot_v1(), before_restart);
    assert!(
        supervisor
            .is_active(&projects.registration_id)
            .expect("active")
    );
    assert_eq!(supervisor.last_failure(&projects.registration_id), Ok(None));

    tokio::runtime::Runtime::new()
        .expect("Projects RLS runtime")
        .block_on(assert_review_owner_rls_v1(
            "makosh_projects_rls_test",
            &[
                "projects_records",
                "projects_outcomes",
                "projects_references",
                "projects_client_operations",
                "projects_outbox",
            ],
        ));
    supervisor.shutdown().expect("stop Projects contour");
    unsafe { std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE") };
    std::fs::remove_dir_all(root).expect("remove Projects fixture");
}

fn route_projects_v1<T: Message + Default>(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    projects: &StartedProjectsRuntimeV1,
    request_id: u64,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    payload: Vec<u8>,
) -> T {
    let response =
        route_projects_response_v1(store, supervisor, projects, request_id, contract, payload);
    assert!(
        response.error_code.is_empty(),
        "Projects request {request_id} failed: {}",
        response.error_code
    );
    T::decode(response.response_payload.as_slice()).expect("decode Projects response")
}

fn route_projects_response_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    projects: &StartedProjectsRuntimeV1,
    request_id: u64,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    payload: Vec<u8>,
) -> ModuleClientResponseV1 {
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: PROJECTS_MODULE_ID_V1.to_owned(),
        owner_id: PROJECTS_OWNER_ID_V1.to_owned(),
        contract: Some(contract),
        request_id,
        request_payload: payload,
        logical_owner_id: PROJECTS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &projects.registration_id,
        &projects.runtime_instance_id,
        projects.runtime_generation,
        projects.grant_epoch,
        PROJECTS_CLIENT_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route authenticated Projects request");
    ModuleClientResponseV1::decode(bytes.as_slice()).expect("Projects response")
}

fn wait_for_projects_relay_v1() {
    let deadline = Instant::now() + Duration::from_secs(15);
    while durable_projects_snapshot_v1().5 != 0 {
        assert!(Instant::now() < deadline, "Projects relay did not drain");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn durable_projects_snapshot_v1() -> (i64, i64, i64, i64, i64, i64) {
    tokio::runtime::Runtime::new().expect("Projects SQL runtime").block_on(async {
        let pool = authenticated_storage_admin_pool_v1().await;
        sqlx::query_as(
            "SELECT \
             (SELECT count(*) FROM makosh_data.projects_records WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.projects_outcomes WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.projects_references WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.projects_client_operations WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.projects_outbox WHERE logical_owner_id='owner-1'), \
             (SELECT count(*) FROM makosh_data.projects_outbox WHERE logical_owner_id='owner-1' AND published_at_unix_millis IS NULL)",
        ).fetch_one(&pool).await.expect("Projects durable snapshot")
    })
}

fn assert_public_projects_outbox_is_private_free_v1() {
    tokio::runtime::Runtime::new().expect("Projects privacy runtime").block_on(async {
        let pool = authenticated_storage_admin_pool_v1().await;
        let rows: Vec<Vec<u8>> = sqlx::query_scalar(
            "SELECT envelope_bytes FROM makosh_data.projects_outbox WHERE logical_owner_id='owner-1' ORDER BY outbox_sequence",
        ).fetch_all(&pool).await.expect("Projects outbox bytes");
        assert!(!rows.is_empty());
        for row in rows {
            for private in [PRIVATE_PROJECT_TEXT_V1, PRIVATE_OUTCOME_TEXT_V1, PRIVATE_REFERENCE_LABEL_V1] {
                assert!(!row.windows(private.len()).any(|window| window == private.as_bytes()), "private Projects content escaped public outbox");
            }
        }
    });
}
