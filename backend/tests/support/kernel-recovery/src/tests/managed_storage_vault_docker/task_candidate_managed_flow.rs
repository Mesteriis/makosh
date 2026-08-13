//! Live signed admission for the event-only reviewed Task candidate chain.

use super::*;

use std::time::Instant;

use crate::identity::device::signer::DeviceSigner;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_communication_task_candidate_api::{
    COMMUNICATION_TASK_CANDIDATE_CAPABILITY_ID_V1,
    COMMUNICATION_TASK_CANDIDATE_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_TASK_CANDIDATE_CONTRACT_MAJOR_V1,
    COMMUNICATION_TASK_CANDIDATE_CONTRACT_REVISION_V1, COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1,
    COMMUNICATION_TASK_CANDIDATE_OWNER_V1, COMMUNICATION_TASK_CANDIDATE_SCHEMA_SHA256,
    wire::{
        CommunicationTaskCandidateErrorCodeV1, CommunicationTaskCandidateStateV1,
        StartCommunicationTaskCandidateRequestV1,
    },
};
use makosh_review_task_candidate_api::{
    REVIEW_TASK_CANDIDATE_MODULE_ID_V1, REVIEW_TASK_CANDIDATE_OWNER_V1,
    wire::{
        ReviewTaskCandidateDecisionV1, ReviewTaskCandidateErrorCodeV1,
        ReviewTaskCandidatePromotionStatusV1, ReviewTaskCandidateStateV1,
    },
};
use makosh_reviewed_task_candidate_promotion_core::{
    REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1, REVIEWED_TASK_CANDIDATE_PROMOTION_OWNER_V1,
};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use makosh_tasks_command_api::{
    TASKS_ADD_CHECKLIST_ITEM_CONNECT_PATH_V1, TASKS_ADD_DEPENDENCY_CONNECT_PATH_V1,
    TASKS_CLIENT_CAPABILITY_ID_V1, TASKS_CREATE_CONNECT_PATH_V1, TASKS_GET_CONNECT_PATH_V1,
    TASKS_LIST_CONNECT_PATH_V1, TASKS_MODULE_ID_V1, TASKS_OWNER_ID_V1,
    TASKS_SET_PRIORITY_CONNECT_PATH_V1, TASKS_SET_STATE_CONNECT_PATH_V1,
    TASKS_UPDATE_CHECKLIST_ITEM_CONNECT_PATH_V1, TASKS_UPDATE_CONNECT_PATH_V1,
    client_wire::{
        AddChecklistItemRequestV1, AddTaskDependencyRequestV1, CreateTaskRequestV1,
        GetTaskRequestV1, ListTasksRequestV1, ListTasksResultV1, SetTaskPriorityRequestV1,
        SetTaskStateRequestV1, TaskMutationResultV1, TaskPriorityV1, TaskStateV1, TaskSummaryV1,
        TimestampV1, UpdateChecklistItemRequestV1, UpdateTaskRequestV1,
    },
    tasks_client_add_dependency_contract_reference_v1,
    tasks_client_set_priority_contract_reference_v1,
};

const TASK_CANDIDATE_SOURCE_BODY_V1: &[u8] =
    b"Action: prepare the release brief by Friday\nCould you check the backup before Monday?";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, extraction, Review and Tasks binaries"]
fn managed_task_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-reviewed-task-candidate");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_task_candidate_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim reviewed Task candidate logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
    );
    let admitted = admit_task_candidate_ensemble_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime = makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64)
        .expect("reviewed Task candidate realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_task_candidate_realtime_v1(&supervisor, &store, realtime.clone());
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure reviewed Task candidate Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("start signed Blob runtime"),
        1
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    let admitted = prepare_task_candidate_ensemble_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    assert_eq!(
        start_communications_domain(&supervisor, &store, &root.join("runtime")),
        1
    );
    let mut started =
        start_task_candidate_ensemble_v1(&supervisor, &store, &root.join("runtime"), admitted);
    assert_eq!(started.len(), 4);
    assert_eq!(
        started
            .iter()
            .map(|runtime| (runtime.module_id.as_str(), runtime.owner_id.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1,
                COMMUNICATION_TASK_CANDIDATE_OWNER_V1,
            ),
            (
                REVIEW_TASK_CANDIDATE_MODULE_ID_V1,
                REVIEW_TASK_CANDIDATE_OWNER_V1,
            ),
            (
                REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1,
                REVIEWED_TASK_CANDIDATE_PROMOTION_OWNER_V1,
            ),
            (TASKS_MODULE_ID_V1, TASKS_OWNER_ID_V1),
        ]
    );
    assert!(started.iter().all(|runtime| {
        runtime.runtime_generation == 1
            && runtime.grant_epoch > 0
            && !runtime.registration_id.is_empty()
            && !runtime.runtime_instance_id.is_empty()
    }));
    let source_message_id = assert_communications_transferred_body_projection_with_plaintext(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        TASK_CANDIDATE_SOURCE_BODY_V1,
        false,
    );
    let source_message_id_exact: [u8; 16] = source_message_id
        .as_slice()
        .try_into()
        .expect("canonical source message id");
    assert_task_candidate_runtime_fences_v1(
        &store,
        &supervisor,
        &started[0],
        source_message_id_exact,
    );
    let wrong_owner = route_task_candidate_start_as_v1(
        &store,
        &supervisor,
        &started[0],
        "owner-2",
        700,
        task_candidate_start_request_v1([0x20; 16], source_message_id_exact, 2),
    );
    assert_eq!(wrong_owner.request_id, 700);
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Task candidate Gateway runtime");
    let router = task_candidate_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let extraction_sse = gateway_runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Task candidate extraction Gateway SSE request"),
        ),
    );
    assert_eq!(extraction_sse.status(), StatusCode::OK);
    let start = start_task_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x21,
        &source_message_id,
        2,
    );
    assert_eq!(
        start.error,
        CommunicationTaskCandidateErrorCodeV1::CommunicationTaskCandidateErrorCodeUnspecified
            as i32
    );
    assert_eq!(
        start.state,
        CommunicationTaskCandidateStateV1::CommunicationTaskCandidateStatePreparingSource as i32
    );
    let ready = wait_for_ready_task_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &start.run_id,
    );
    assert_eq!(ready.source_message_id, source_message_id);
    assert_eq!(ready.expected_source_revision, 2);
    assert!(ready.candidates.len() >= 2);
    let candidate_titles = ready
        .candidates
        .iter()
        .map(|candidate| candidate.title.as_bytes().to_vec())
        .collect::<Vec<_>>();
    let extraction_event = read_task_candidate_extraction_terminal_event_v1(
        extraction_sse,
        &gateway_runtime,
        &start.run_id,
    );
    for title in &candidate_titles {
        assert!(
            !extraction_event
                .encode_to_vec()
                .windows(title.len())
                .any(|window| window == title),
            "client realtime must not expose candidate presentation bytes"
        );
    }
    assert!(
        !extraction_event
            .encode_to_vec()
            .windows(TASK_CANDIDATE_SOURCE_BODY_V1.len())
            .any(|window| window == TASK_CANDIDATE_SOURCE_BODY_V1),
        "client realtime must not expose communication source content"
    );
    let extraction_cursor = extraction_event.cursor.clone();

    let duplicate = start_task_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x21,
        &source_message_id,
        2,
    );
    assert_eq!(duplicate.run_id, start.run_id);
    assert_eq!(
        duplicate.state,
        CommunicationTaskCandidateStateV1::CommunicationTaskCandidateStateReady as i32
    );
    let conflict = start_task_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x21,
        &source_message_id,
        3,
    );
    assert_eq!(
        conflict.error,
        CommunicationTaskCandidateErrorCodeV1::CommunicationTaskCandidateErrorCodeInvalidRequest
            as i32
    );
    let stale = start_task_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x22,
        &source_message_id,
        1,
    );
    assert_eq!(
        stale.error,
        CommunicationTaskCandidateErrorCodeV1::CommunicationTaskCandidateErrorCodeUnspecified
            as i32
    );
    let stale = wait_for_rejected_task_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &stale.run_id,
    );
    assert_eq!(
        stale.error,
        CommunicationTaskCandidateErrorCodeV1::CommunicationTaskCandidateErrorCodeSourceRejected
            as i32
    );
    assert!(stale.candidates.is_empty());

    let reviews = wait_for_extracted_task_candidate_reviews_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &ready.candidates,
    );
    assert_no_task_materialization_v1(&gateway_runtime, &reviews);
    let terminal_sse = gateway_runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Task candidate terminal Gateway SSE request"),
        ),
    );
    assert_eq!(terminal_sse.status(), StatusCode::OK);

    let approved = decide_task_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x51,
        &reviews.approved_review_id,
        1,
        ReviewTaskCandidateDecisionV1::ReviewTaskCandidateDecisionApprove,
    );
    assert_eq!(
        approved.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeUnspecified as i32
    );
    assert!(!approved.replayed);
    let approved_state = approved.review.expect("approved Review response");
    assert_eq!(
        approved_state.state,
        ReviewTaskCandidateStateV1::ReviewTaskCandidateStateApproved as i32
    );
    assert_eq!(
        approved_state.promotion_status,
        ReviewTaskCandidatePromotionStatusV1::ReviewTaskCandidatePromotionStatusPending as i32
    );
    assert_eq!(approved_state.review_revision, 2);

    let rejected = decide_task_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x61,
        &reviews.rejected_review_id,
        1,
        ReviewTaskCandidateDecisionV1::ReviewTaskCandidateDecisionReject,
    );
    assert_eq!(
        rejected.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeUnspecified as i32
    );
    assert!(!rejected.replayed);
    let rejected_state = rejected.review.expect("rejected Review response");
    assert_eq!(
        rejected_state.state,
        ReviewTaskCandidateStateV1::ReviewTaskCandidateStateRejected as i32
    );
    assert_eq!(
        rejected_state.promotion_status,
        ReviewTaskCandidatePromotionStatusV1::ReviewTaskCandidatePromotionStatusNotRequested as i32
    );
    assert_eq!(rejected_state.review_revision, 2);

    let stale = decide_task_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x62,
        &reviews.rejected_review_id,
        1,
        ReviewTaskCandidateDecisionV1::ReviewTaskCandidateDecisionApprove,
    );
    assert_eq!(
        stale.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeRevisionConflict as i32
    );
    assert!(stale.review.is_none());

    let (approved_final, rejected_final) =
        wait_for_task_candidate_terminal_states_v1(&router, &gateway_runtime, &cookie, &reviews);
    assert_task_candidate_response_states_v1(&approved_final, &rejected_final);
    let first_page = list_task_candidates_v1(&router, &gateway_runtime, &cookie, Vec::new(), 1);
    assert_eq!(
        first_page.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeUnspecified as i32
    );
    assert_eq!(first_page.reviews.len(), 1);
    assert_eq!(
        first_page.next_after_review_id,
        first_page.reviews[0].review_id
    );
    let second_page = list_task_candidates_v1(
        &router,
        &gateway_runtime,
        &cookie,
        first_page.next_after_review_id.clone(),
        1,
    );
    assert_eq!(second_page.reviews.len(), 1);
    assert!(second_page.next_after_review_id.is_empty());
    assert_ne!(
        first_page.reviews[0].review_id,
        second_page.reviews[0].review_id
    );
    let approved_replay = decide_task_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x51,
        &reviews.approved_review_id,
        1,
        ReviewTaskCandidateDecisionV1::ReviewTaskCandidateDecisionApprove,
    );
    assert_eq!(
        approved_replay.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeUnspecified as i32
    );
    assert!(approved_replay.replayed);
    let approved_replay_state = approved_replay.review.expect("replayed Review response");
    assert_eq!(
        approved_replay_state.promotion_status,
        ReviewTaskCandidatePromotionStatusV1::ReviewTaskCandidatePromotionStatusSucceeded as i32
    );
    assert_eq!(approved_replay_state.review_revision, 3);

    let operation_conflict = decide_task_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x51,
        &reviews.approved_review_id,
        1,
        ReviewTaskCandidateDecisionV1::ReviewTaskCandidateDecisionReject,
    );
    assert_eq!(
        operation_conflict.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeOperationConflict as i32
    );
    assert!(operation_conflict.review.is_none());

    let terminal = read_task_candidate_terminal_events_v1(terminal_sse, &gateway_runtime, &reviews);
    assert_exact_task_materialization_v1(&gateway_runtime, &reviews);
    for title in &candidate_titles {
        for event in [&terminal.approved, &terminal.rejected] {
            assert!(
                !event
                    .encode_to_vec()
                    .windows(title.len())
                    .any(|window| window == title),
                "client realtime must not expose candidate presentation bytes"
            );
        }
    }
    let approved_cursor = terminal.approved.cursor.clone();
    let rejected_cursor = terminal.rejected.cursor.clone();

    assert!(
        realtime
            .revoke_owner(TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
            .expect("clear Task candidate Gateway replay cache")
    );
    let restarted_router =
        task_candidate_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replay_sse = gateway_runtime.block_on(
        restarted_router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &restarted_cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Task candidate replay Gateway SSE request"),
        ),
    );
    assert_eq!(replay_sse.status(), StatusCode::OK);
    let terminal_replay_sse = gateway_runtime.block_on(
        restarted_router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &restarted_cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Task candidate terminal replay Gateway SSE request"),
        ),
    );
    assert_eq!(terminal_replay_sse.status(), StatusCode::OK);
    let extraction_position = started
        .iter()
        .position(|runtime| runtime.module_id == COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1)
        .expect("started Task candidate extraction runtime");
    let extraction = started.remove(extraction_position);
    let extraction =
        restart_task_candidate_runtime_v1(&supervisor, &store, &root.join("runtime"), extraction);
    started.insert(extraction_position, extraction);
    let review_position = started
        .iter()
        .position(|runtime| runtime.module_id == REVIEW_TASK_CANDIDATE_MODULE_ID_V1)
        .expect("started Review Task candidate runtime");
    let review = started.remove(review_position);
    let review =
        restart_task_candidate_runtime_v1(&supervisor, &store, &root.join("runtime"), review);
    started.insert(review_position, review);
    let replayed_extraction = read_task_candidate_extraction_terminal_event_v1(
        replay_sse,
        &gateway_runtime,
        &start.run_id,
    );
    let replayed =
        read_task_candidate_terminal_events_v1(terminal_replay_sse, &gateway_runtime, &reviews);
    assert_eq!(replayed_extraction.cursor, extraction_cursor);
    assert_eq!(replayed.approved.cursor, approved_cursor);
    assert_eq!(replayed.rejected.cursor, rejected_cursor);
    let restarted_page = list_task_candidates_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        Vec::new(),
        2,
    );
    assert_eq!(restarted_page.reviews.len(), 2);
    assert!(restarted_page.next_after_review_id.is_empty());
    assert_tasks_reject_stale_blob_receipt_v1(&gateway_runtime, &store);
    assert_tasks_lifecycle_v1(
        &store,
        &supervisor,
        &root,
        &router,
        &gateway_runtime,
        &cookie,
        &mut started,
    );

    // Effective NOLOGIN/NOSUPERUSER/NOBYPASSRLS proof for all Task Review tables.
    gateway_runtime.block_on(assert_review_owner_rls_v1(
        "makosh_review_task_rls_test",
        &[
            "review_task_candidate_submissions",
            "review_task_candidate_state",
            "review_task_candidate_operations",
            "review_task_candidate_promotion_inbox",
            "review_task_candidate_outbox",
            "review_task_candidate_realtime",
        ],
    ));

    supervisor.shutdown().expect("stop managed processes");
    assert_reviewed_task_candidate_persistence_negatives_v1(&gateway_runtime);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove reviewed Task candidate fixture");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS and Tasks binaries"]
fn managed_tasks_lifecycle_replays_and_restarts_with_owner_rls() {
    managed_task_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart();
}

#[allow(clippy::too_many_arguments)]
fn assert_tasks_lifecycle_v1(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    router: &TaskCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    started: &mut Vec<StartedTaskCandidateRuntimeV1>,
) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Tasks lifecycle wall clock")
        .as_millis() as i64
        - 1_000;
    let time = |offset: i64| TimestampV1 {
        unix_seconds: (now + offset) / 1_000,
        nanos: (((now + offset) % 1_000) * 1_000_000) as i32,
    };
    let create = |operation: u8, title: &str| -> TaskMutationResultV1 {
        post_proto(
            router,
            runtime,
            cookie,
            TASKS_CREATE_CONNECT_PATH_V1,
            CreateTaskRequestV1 {
                operation_id: vec![operation; 16],
                task_id: Vec::new(),
                logical_owner_id: String::new(),
                title: title.to_owned(),
                description: Some(format!("{title} private owner detail")),
                due_at: Some(time(86_400_000)),
                priority: TaskPriorityV1::TaskPriorityNormal as i32,
                created_at: Some(time(0)),
            },
        )
    };
    let first = create(0xa1, "Lifecycle primary");
    let first_task = first.task.expect("created primary Task");
    assert_eq!(first_task.task_revision, 1);
    let second = create(0xa2, "Lifecycle dependency");
    let second_task = second.task.expect("created dependency Task");

    let updated: TaskMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        TASKS_UPDATE_CONNECT_PATH_V1,
        UpdateTaskRequestV1 {
            operation_id: vec![0xa3; 16],
            task_id: first_task.task_id.clone(),
            logical_owner_id: String::new(),
            expected_task_revision: 1,
            title: Some("Lifecycle primary updated".to_owned()),
            description: None,
            clear_description: false,
            due_at: None,
            clear_due_at: false,
            updated_at: Some(time(1)),
        },
    );
    assert_eq!(
        updated.task.as_ref().expect("updated Task").task_revision,
        2
    );
    let state: TaskMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        TASKS_SET_STATE_CONNECT_PATH_V1,
        SetTaskStateRequestV1 {
            operation_id: vec![0xa4; 16],
            task_id: first_task.task_id.clone(),
            logical_owner_id: String::new(),
            expected_task_revision: 2,
            state: TaskStateV1::TaskStateInProgress as i32,
            changed_at: Some(time(2)),
        },
    );
    assert_eq!(state.task.as_ref().expect("state Task").task_revision, 3);
    let priority: TaskMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        TASKS_SET_PRIORITY_CONNECT_PATH_V1,
        SetTaskPriorityRequestV1 {
            operation_id: vec![0xa5; 16],
            task_id: first_task.task_id.clone(),
            logical_owner_id: String::new(),
            expected_task_revision: 3,
            priority: TaskPriorityV1::TaskPriorityHigh as i32,
            changed_at: Some(time(3)),
        },
    );
    assert_eq!(
        priority.task.as_ref().expect("priority Task").task_revision,
        4
    );
    let dependency: TaskMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        TASKS_ADD_DEPENDENCY_CONNECT_PATH_V1,
        AddTaskDependencyRequestV1 {
            operation_id: vec![0xa6; 16],
            task_id: first_task.task_id.clone(),
            logical_owner_id: String::new(),
            expected_task_revision: 4,
            dependency_id: vec![0xd1; 16],
            depends_on_task_id: second_task.task_id.clone(),
            changed_at: Some(time(4)),
        },
    );
    assert_eq!(
        dependency
            .task
            .as_ref()
            .expect("dependency Task")
            .dependencies
            .len(),
        1
    );
    let checklist: TaskMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        TASKS_ADD_CHECKLIST_ITEM_CONNECT_PATH_V1,
        AddChecklistItemRequestV1 {
            operation_id: vec![0xa7; 16],
            task_id: first_task.task_id.clone(),
            logical_owner_id: String::new(),
            expected_task_revision: 5,
            checklist_item_id: vec![0xc1; 16],
            label: "Verify exact restart".to_owned(),
            position: 10,
            changed_at: Some(time(5)),
        },
    );
    assert_eq!(
        checklist
            .task
            .as_ref()
            .expect("checklist Task")
            .task_revision,
        6
    );
    let checklist_request = UpdateChecklistItemRequestV1 {
        operation_id: vec![0xa8; 16],
        task_id: first_task.task_id.clone(),
        logical_owner_id: String::new(),
        expected_task_revision: 6,
        checklist_item_id: vec![0xc1; 16],
        label: None,
        completed: Some(true),
        position: None,
        changed_at: Some(time(6)),
    };
    let checked: TaskMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        TASKS_UPDATE_CHECKLIST_ITEM_CONNECT_PATH_V1,
        checklist_request.clone(),
    );
    let replay: TaskMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        TASKS_UPDATE_CHECKLIST_ITEM_CONNECT_PATH_V1,
        checklist_request,
    );
    assert_eq!(checked, replay, "exact operation replay response");
    assert!(checked.task.as_ref().expect("checked Task").checklist[0].completed);

    let mut cursor = Vec::new();
    let mut ids = Vec::new();
    loop {
        let page: ListTasksResultV1 = post_proto(
            router,
            runtime,
            cookie,
            TASKS_LIST_CONNECT_PATH_V1,
            ListTasksRequestV1 {
                logical_owner_id: String::new(),
                after_task_id: cursor,
                limit: 1,
            },
        );
        assert_eq!(page.tasks.len(), 1);
        ids.push(page.tasks[0].task_id.clone());
        if page.next_after_task_id.is_empty() {
            break;
        }
        cursor = page.next_after_task_id;
    }
    let unique = ids.iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        unique.len(),
        ids.len(),
        "Tasks pagination has no duplicates"
    );
    assert!(ids.contains(&first_task.task_id));
    assert!(ids.contains(&second_task.task_id));

    let tasks_position = started
        .iter()
        .position(|value| value.module_id == TASKS_MODULE_ID_V1)
        .expect("started Tasks runtime");
    let tasks = started.remove(tasks_position);
    let tasks = restart_task_candidate_runtime_v1(supervisor, store, &root.join("runtime"), tasks);
    started.insert(tasks_position, tasks);
    let restarted: TaskSummaryV1 = post_proto(
        router,
        runtime,
        cookie,
        TASKS_GET_CONNECT_PATH_V1,
        GetTaskRequestV1 {
            logical_owner_id: String::new(),
            task_id: first_task.task_id.clone(),
        },
    );
    assert_eq!(restarted.task_revision, 7);
    assert_eq!(restarted.dependencies.len(), 1);
    assert_eq!(restarted.checklist.len(), 1);

    let tasks_runtime = started
        .iter()
        .find(|value| value.module_id == TASKS_MODULE_ID_V1)
        .expect("restarted Tasks runtime");
    let cycle_request = AddTaskDependencyRequestV1 {
        operation_id: vec![0xa9; 16],
        task_id: second_task.task_id,
        logical_owner_id: String::new(),
        expected_task_revision: 1,
        dependency_id: vec![0xd2; 16],
        depends_on_task_id: restarted.task_id.clone(),
        changed_at: Some(time(7)),
    };
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: TASKS_MODULE_ID_V1.to_owned(),
        owner_id: TASKS_OWNER_ID_V1.to_owned(),
        contract: Some(tasks_client_add_dependency_contract_reference_v1()),
        request_id: 9_001,
        request_payload: cycle_request.encode_to_vec(),
        logical_owner_id: TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &tasks_runtime.registration_id,
        &tasks_runtime.runtime_instance_id,
        tasks_runtime.runtime_generation,
        tasks_runtime.grant_epoch,
        TASKS_CLIENT_CAPABILITY_ID_V1,
        &request,
    );
    let response = ModuleClientResponseV1::decode(
        crate::modules::capability::router::route_managed_client_request(
            store.as_ref(),
            &supervisor.relay_port(),
            &route,
        )
        .expect("route cycle negative")
        .as_slice(),
    )
    .expect("decode cycle negative");
    assert_eq!(response.error_code, "FAILED_PRECONDITION");

    let stale = SetTaskPriorityRequestV1 {
        operation_id: vec![0xaa; 16],
        task_id: restarted.task_id,
        logical_owner_id: String::new(),
        expected_task_revision: 1,
        priority: TaskPriorityV1::TaskPriorityUrgent as i32,
        changed_at: Some(time(8)),
    };
    let stale_request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: TASKS_MODULE_ID_V1.to_owned(),
        owner_id: TASKS_OWNER_ID_V1.to_owned(),
        contract: Some(tasks_client_set_priority_contract_reference_v1()),
        request_id: 9_002,
        request_payload: stale.encode_to_vec(),
        logical_owner_id: TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let stale_route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &tasks_runtime.registration_id,
        &tasks_runtime.runtime_instance_id,
        tasks_runtime.runtime_generation,
        tasks_runtime.grant_epoch,
        TASKS_CLIENT_CAPABILITY_ID_V1,
        &stale_request,
    );
    let stale_response = ModuleClientResponseV1::decode(
        crate::modules::capability::router::route_managed_client_request(
            store.as_ref(),
            &supervisor.relay_port(),
            &stale_route,
        )
        .expect("route stale revision negative")
        .as_slice(),
    )
    .expect("decode stale revision negative");
    assert_eq!(stale_response.error_code, "FAILED_PRECONDITION");

    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let (operations, pending, public_envelopes): (i64, i64, Vec<Vec<u8>>) =
            runtime.block_on(async {
                let pool = task_candidate_admin_pool_v1().await;
                let operations = sqlx::query_scalar(
                    "SELECT count(*) FROM makosh_data.tasks_client_operations \
                 WHERE logical_owner_id=$1",
                )
                .bind(TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
                .fetch_one(&pool)
                .await
                .expect("count Tasks lifecycle operations");
                let pending = sqlx::query_scalar(
                    "SELECT count(*) FROM makosh_data.tasks_outbox \
                 WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL",
                )
                .bind(TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
                .fetch_one(&pool)
                .await
                .expect("count pending Tasks outbox");
                let public_envelopes = sqlx::query_scalar(
                    "SELECT envelope_bytes FROM makosh_data.tasks_outbox WHERE logical_owner_id=$1",
                )
                .bind(TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
                .fetch_all(&pool)
                .await
                .expect("load Tasks public outbox envelopes");
                pool.close().await;
                (operations, pending, public_envelopes)
            });
        for envelope in &public_envelopes {
            for private_marker in [
                b"Lifecycle primary".as_slice(),
                b"private owner detail".as_slice(),
                b"Verify exact restart".as_slice(),
            ] {
                assert!(
                    !envelope
                        .windows(private_marker.len())
                        .any(|window| window == private_marker),
                    "Tasks public event retained owner-private content"
                );
            }
        }
        if operations == 8 && pending == 0 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Tasks lifecycle relay did not drain: operations={operations}, pending={pending}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }

    runtime.block_on(assert_review_owner_rls_v1(
        "makosh_tasks_rls_test",
        &[
            "tasks_reviewed_candidate_inbox",
            "tasks_state",
            "tasks_outbox",
            "tasks_dependencies",
            "tasks_checklist",
            "tasks_client_operations",
        ],
    ));
}

fn task_candidate_start_request_v1(
    operation_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
) -> StartCommunicationTaskCandidateRequestV1 {
    StartCommunicationTaskCandidateRequestV1 {
        protocol_major: 1,
        operation_id: operation_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
    }
}

fn route_task_candidate_start_as_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    workflow: &StartedTaskCandidateRuntimeV1,
    logical_owner_id: &str,
    request_id: u64,
    request: StartCommunicationTaskCandidateRequestV1,
) -> ModuleClientResponseV1 {
    let request = encode_task_candidate_module_request_v1(logical_owner_id, request_id, request);
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &workflow.registration_id,
        &workflow.runtime_instance_id,
        workflow.runtime_generation,
        workflow.grant_epoch,
        COMMUNICATION_TASK_CANDIDATE_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route Task candidate owner-fence request");
    ModuleClientResponseV1::decode(bytes.as_slice()).expect("decode Task candidate response")
}

fn assert_task_candidate_runtime_fences_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    workflow: &StartedTaskCandidateRuntimeV1,
    source_message_id: [u8; 16],
) {
    let request = encode_task_candidate_module_request_v1(
        TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
        699,
        task_candidate_start_request_v1([0x1f; 16], source_message_id, 2),
    );
    for (runtime_generation, grant_epoch) in [
        (workflow.runtime_generation + 1, workflow.grant_epoch),
        (workflow.runtime_generation, workflow.grant_epoch + 1),
    ] {
        let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
            &workflow.registration_id,
            &workflow.runtime_instance_id,
            runtime_generation,
            grant_epoch,
            COMMUNICATION_TASK_CANDIDATE_CAPABILITY_ID_V1,
            &request,
        );
        assert_eq!(
            crate::modules::capability::router::route_managed_client_request(
                store,
                &supervisor.relay_port(),
                &route,
            )
            .expect_err("stale Task candidate runtime fence"),
            "managed runtime fence is stale"
        );
    }
}

fn encode_task_candidate_module_request_v1(
    logical_owner_id: &str,
    request_id: u64,
    request: StartCommunicationTaskCandidateRequestV1,
) -> Vec<u8> {
    ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_TASK_CANDIDATE_OWNER_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: COMMUNICATION_TASK_CANDIDATE_OWNER_V1.to_owned(),
            name: COMMUNICATION_TASK_CANDIDATE_COMMAND_CONTRACT_NAME_V1.to_owned(),
            major: COMMUNICATION_TASK_CANDIDATE_CONTRACT_MAJOR_V1,
            revision: COMMUNICATION_TASK_CANDIDATE_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATION_TASK_CANDIDATE_SCHEMA_SHA256.to_vec(),
        }),
        request_id,
        request_payload: request.encode_to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec()
}
