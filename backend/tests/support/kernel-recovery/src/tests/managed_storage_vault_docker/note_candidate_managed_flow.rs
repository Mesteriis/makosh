//! Live signed admission for the event-only reviewed Note candidate chain.

use super::*;

use crate::identity::device::signer::DeviceSigner;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_communication_note_candidate_api::{
    COMMUNICATION_NOTE_CANDIDATE_CAPABILITY_ID_V1,
    COMMUNICATION_NOTE_CANDIDATE_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_NOTE_CANDIDATE_CONTRACT_MAJOR_V1,
    COMMUNICATION_NOTE_CANDIDATE_CONTRACT_REVISION_V1, COMMUNICATION_NOTE_CANDIDATE_MODULE_ID_V1,
    COMMUNICATION_NOTE_CANDIDATE_OWNER_V1, COMMUNICATION_NOTE_CANDIDATE_SCHEMA_SHA256,
    wire::{
        CommunicationNoteCandidateErrorCodeV1, CommunicationNoteCandidateStateV1,
        StartCommunicationNoteCandidateRequestV1,
    },
};
use makosh_knowledge_command_api::{
    KNOWLEDGE_ADD_SOURCE_CONNECT_PATH_V1, KNOWLEDGE_CREATE_CONNECT_PATH_V1,
    KNOWLEDGE_GET_CONNECT_PATH_V1, KNOWLEDGE_LIST_CONNECT_PATH_V1, KNOWLEDGE_MODULE_ID_V1,
    KNOWLEDGE_OWNER_ID_V1, KNOWLEDGE_REMOVE_SOURCE_CONNECT_PATH_V1,
    KNOWLEDGE_SEARCH_CONNECT_PATH_V1, KNOWLEDGE_SET_STATE_CONNECT_PATH_V1,
    KNOWLEDGE_UPDATE_CONNECT_PATH_V1,
    client_wire::{
        AddKnowledgeSourceRequestV1, CreateKnowledgeNoteRequestV1, GetKnowledgeNoteRequestV1,
        KnowledgeNoteMutationResultV1, KnowledgeNoteStateV1, KnowledgeNoteV1,
        ListKnowledgeNotesRequestV1, ListKnowledgeNotesResultV1, RemoveKnowledgeSourceRequestV1,
        SearchKnowledgeNotesRequestV1, SetKnowledgeNoteStateRequestV1, TimestampV1,
        UpdateKnowledgeNoteRequestV1,
    },
};
use makosh_review_note_candidate_api::{
    REVIEW_NOTE_CANDIDATE_MODULE_ID_V1, REVIEW_NOTE_CANDIDATE_OWNER_V1,
    wire::{
        ReviewNoteCandidateDecisionV1, ReviewNoteCandidateErrorCodeV1,
        ReviewNoteCandidatePromotionStatusV1, ReviewNoteCandidateStateV1,
    },
};
use makosh_reviewed_note_candidate_promotion_core::{
    REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1, REVIEWED_NOTE_CANDIDATE_PROMOTION_OWNER_V1,
};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};

const APPROVED_NOTE_SOURCE_BODY_V1: &[u8] =
    b"Contract approved. The agreement is ready for the knowledge base.";
const REJECTED_NOTE_SOURCE_BODY_V1: &[u8] =
    b"Invoice payment deadline is Friday; retain this financial context for review.";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, extraction, Review and Knowledge binaries"]
fn managed_note_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-reviewed-note-candidate");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_note_candidate_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim reviewed Note candidate logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
    );
    let admitted = admit_note_candidate_ensemble_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime = makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64)
        .expect("reviewed Note candidate realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_note_candidate_realtime_v1(&supervisor, &store, realtime.clone());
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure reviewed Note candidate Event credential handler");
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
    let admitted = prepare_note_candidate_ensemble_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    assert_eq!(
        start_communications_domain(&supervisor, &store, &root.join("runtime")),
        1
    );
    let mut started =
        start_note_candidate_ensemble_v1(&supervisor, &store, &root.join("runtime"), admitted);
    assert_eq!(started.len(), 4);
    assert_eq!(
        started
            .iter()
            .map(|runtime| (runtime.module_id.as_str(), runtime.owner_id.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                COMMUNICATION_NOTE_CANDIDATE_MODULE_ID_V1,
                COMMUNICATION_NOTE_CANDIDATE_OWNER_V1,
            ),
            (
                REVIEW_NOTE_CANDIDATE_MODULE_ID_V1,
                REVIEW_NOTE_CANDIDATE_OWNER_V1,
            ),
            (
                REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1,
                REVIEWED_NOTE_CANDIDATE_PROMOTION_OWNER_V1,
            ),
            (KNOWLEDGE_MODULE_ID_V1, KNOWLEDGE_OWNER_ID_V1),
        ]
    );
    assert!(started.iter().all(|runtime| {
        runtime.runtime_generation == 1
            && runtime.grant_epoch > 0
            && !runtime.registration_id.is_empty()
            && !runtime.runtime_instance_id.is_empty()
    }));
    let approved_source_message_id =
        assert_communications_transferred_body_projection_with_plaintext_and_fixture_id(
            &store,
            &supervisor,
            &data,
            release.kernel(),
            &root.join("runtime"),
            APPROVED_NOTE_SOURCE_BODY_V1,
            false,
            1,
        );
    let rejected_source_message_id =
        assert_communications_transferred_body_projection_with_plaintext_and_fixture_id(
            &store,
            &supervisor,
            &data,
            release.kernel(),
            &root.join("runtime"),
            REJECTED_NOTE_SOURCE_BODY_V1,
            false,
            2,
        );
    assert_ne!(approved_source_message_id, rejected_source_message_id);
    let approved_source_message_id_exact: [u8; 16] = approved_source_message_id
        .as_slice()
        .try_into()
        .expect("approved canonical source message id");
    assert_note_candidate_runtime_fences_v1(
        &store,
        &supervisor,
        &started[0],
        approved_source_message_id_exact,
    );
    let wrong_owner = route_note_candidate_start_as_v1(
        &store,
        &supervisor,
        &started[0],
        "owner-2",
        700,
        note_candidate_start_request_v1([0x20; 16], approved_source_message_id_exact, 2),
    );
    assert_eq!(wrong_owner.request_id, 700);
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Note candidate Gateway runtime");
    let router = note_candidate_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let approved_extraction_sse = gateway_runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("approved Note candidate extraction Gateway SSE request"),
        ),
    );
    assert_eq!(approved_extraction_sse.status(), StatusCode::OK);
    let approved_start = start_note_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x21,
        &approved_source_message_id,
        2,
    );
    assert_eq!(
        approved_start.error,
        CommunicationNoteCandidateErrorCodeV1::CommunicationNoteCandidateErrorCodeUnspecified
            as i32
    );
    assert_eq!(
        approved_start.state,
        CommunicationNoteCandidateStateV1::CommunicationNoteCandidateStatePreparingSource as i32
    );
    let approved_ready = wait_for_ready_note_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &approved_start.run_id,
    );
    assert_eq!(approved_ready.source_message_id, approved_source_message_id);
    assert_eq!(approved_ready.expected_source_revision, 2);
    assert_eq!(approved_ready.candidates.len(), 1);

    let rejected_extraction_sse = gateway_runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("rejected Note candidate extraction Gateway SSE request"),
        ),
    );
    assert_eq!(rejected_extraction_sse.status(), StatusCode::OK);
    let rejected_start = start_note_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x23,
        &rejected_source_message_id,
        2,
    );
    assert_eq!(
        rejected_start.error,
        CommunicationNoteCandidateErrorCodeV1::CommunicationNoteCandidateErrorCodeUnspecified
            as i32
    );
    let rejected_ready = wait_for_ready_note_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &rejected_start.run_id,
    );
    assert_eq!(rejected_ready.source_message_id, rejected_source_message_id);
    assert_eq!(rejected_ready.expected_source_revision, 2);
    assert_eq!(rejected_ready.candidates.len(), 1);

    let candidate_titles = [&approved_ready.candidates[0], &rejected_ready.candidates[0]]
        .into_iter()
        .map(|candidate| candidate.title.as_bytes().to_vec())
        .collect::<Vec<_>>();
    let approved_extraction_event = read_note_candidate_extraction_terminal_event_v1(
        approved_extraction_sse,
        &gateway_runtime,
        &approved_start.run_id,
    );
    let rejected_extraction_event = read_note_candidate_extraction_terminal_event_v1(
        rejected_extraction_sse,
        &gateway_runtime,
        &rejected_start.run_id,
    );
    for title in &candidate_titles {
        for event in [&approved_extraction_event, &rejected_extraction_event] {
            assert!(
                !event
                    .encode_to_vec()
                    .windows(title.len())
                    .any(|window| window == title),
                "client realtime must not expose candidate presentation bytes"
            );
        }
    }
    for (event, source) in [
        (&approved_extraction_event, APPROVED_NOTE_SOURCE_BODY_V1),
        (&rejected_extraction_event, REJECTED_NOTE_SOURCE_BODY_V1),
    ] {
        assert!(
            !event
                .encode_to_vec()
                .windows(source.len())
                .any(|window| window == source),
            "client realtime must not expose communication source content"
        );
    }
    let approved_extraction_cursor = approved_extraction_event.cursor.clone();
    let rejected_extraction_cursor = rejected_extraction_event.cursor.clone();

    let duplicate = start_note_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x21,
        &approved_source_message_id,
        2,
    );
    assert_eq!(duplicate.run_id, approved_start.run_id);
    assert_eq!(
        duplicate.state,
        CommunicationNoteCandidateStateV1::CommunicationNoteCandidateStateReady as i32
    );
    let conflict = start_note_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x21,
        &approved_source_message_id,
        3,
    );
    assert_eq!(
        conflict.error,
        CommunicationNoteCandidateErrorCodeV1::CommunicationNoteCandidateErrorCodeInvalidRequest
            as i32
    );
    let stale = start_note_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x22,
        &approved_source_message_id,
        1,
    );
    assert_eq!(
        stale.error,
        CommunicationNoteCandidateErrorCodeV1::CommunicationNoteCandidateErrorCodeUnspecified
            as i32
    );
    let stale = wait_for_rejected_note_candidate_extraction_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &stale.run_id,
    );
    assert_eq!(
        stale.error,
        CommunicationNoteCandidateErrorCodeV1::CommunicationNoteCandidateErrorCodeSourceRejected
            as i32
    );
    assert!(stale.candidates.is_empty());

    let reviews = wait_for_extracted_note_candidate_reviews_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &approved_ready.candidates[0],
        &rejected_ready.candidates[0],
    );
    assert_no_note_materialization_v1(&gateway_runtime, &reviews);
    let terminal_sse = gateway_runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Note candidate terminal Gateway SSE request"),
        ),
    );
    assert_eq!(terminal_sse.status(), StatusCode::OK);

    let approved = decide_note_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x51,
        &reviews.approved_review_id,
        1,
        ReviewNoteCandidateDecisionV1::ReviewNoteCandidateDecisionApprove,
    );
    assert_eq!(
        approved.error,
        ReviewNoteCandidateErrorCodeV1::ReviewNoteCandidateErrorCodeUnspecified as i32
    );
    assert!(!approved.replayed);
    let approved_state = approved.review.expect("approved Review response");
    assert_eq!(
        approved_state.state,
        ReviewNoteCandidateStateV1::ReviewNoteCandidateStateApproved as i32
    );
    assert_eq!(
        approved_state.promotion_status,
        ReviewNoteCandidatePromotionStatusV1::ReviewNoteCandidatePromotionStatusPending as i32
    );
    assert_eq!(approved_state.review_revision, 2);

    let rejected = decide_note_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x61,
        &reviews.rejected_review_id,
        1,
        ReviewNoteCandidateDecisionV1::ReviewNoteCandidateDecisionReject,
    );
    assert_eq!(
        rejected.error,
        ReviewNoteCandidateErrorCodeV1::ReviewNoteCandidateErrorCodeUnspecified as i32
    );
    assert!(!rejected.replayed);
    let rejected_state = rejected.review.expect("rejected Review response");
    assert_eq!(
        rejected_state.state,
        ReviewNoteCandidateStateV1::ReviewNoteCandidateStateRejected as i32
    );
    assert_eq!(
        rejected_state.promotion_status,
        ReviewNoteCandidatePromotionStatusV1::ReviewNoteCandidatePromotionStatusNotRequested as i32
    );
    assert_eq!(rejected_state.review_revision, 2);

    let stale = decide_note_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x62,
        &reviews.rejected_review_id,
        1,
        ReviewNoteCandidateDecisionV1::ReviewNoteCandidateDecisionApprove,
    );
    assert_eq!(
        stale.error,
        ReviewNoteCandidateErrorCodeV1::ReviewNoteCandidateErrorCodeRevisionConflict as i32
    );
    assert!(stale.review.is_none());

    let (approved_final, rejected_final) =
        wait_for_note_candidate_terminal_states_v1(&router, &gateway_runtime, &cookie, &reviews);
    assert_note_candidate_response_states_v1(&approved_final, &rejected_final);
    let first_page = list_note_candidates_v1(&router, &gateway_runtime, &cookie, Vec::new(), 1);
    assert_eq!(
        first_page.error,
        ReviewNoteCandidateErrorCodeV1::ReviewNoteCandidateErrorCodeUnspecified as i32
    );
    assert_eq!(first_page.reviews.len(), 1);
    assert_eq!(
        first_page.next_after_review_id,
        first_page.reviews[0].review_id
    );
    let second_page = list_note_candidates_v1(
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
    let approved_replay = decide_note_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x51,
        &reviews.approved_review_id,
        1,
        ReviewNoteCandidateDecisionV1::ReviewNoteCandidateDecisionApprove,
    );
    assert_eq!(
        approved_replay.error,
        ReviewNoteCandidateErrorCodeV1::ReviewNoteCandidateErrorCodeUnspecified as i32
    );
    assert!(approved_replay.replayed);
    let approved_replay_state = approved_replay.review.expect("replayed Review response");
    assert_eq!(
        approved_replay_state.promotion_status,
        ReviewNoteCandidatePromotionStatusV1::ReviewNoteCandidatePromotionStatusSucceeded as i32
    );
    assert_eq!(approved_replay_state.review_revision, 3);

    let operation_conflict = decide_note_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x51,
        &reviews.approved_review_id,
        1,
        ReviewNoteCandidateDecisionV1::ReviewNoteCandidateDecisionReject,
    );
    assert_eq!(
        operation_conflict.error,
        ReviewNoteCandidateErrorCodeV1::ReviewNoteCandidateErrorCodeOperationConflict as i32
    );
    assert!(operation_conflict.review.is_none());

    let terminal = read_note_candidate_terminal_events_v1(terminal_sse, &gateway_runtime, &reviews);
    assert_exact_note_materialization_v1(&gateway_runtime, &reviews);
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
            .revoke_owner(NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
            .expect("clear Note candidate Gateway replay cache")
    );
    let restarted_router =
        note_candidate_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let approved_replay_sse = gateway_runtime.block_on(
        restarted_router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &restarted_cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("approved Note candidate replay Gateway SSE request"),
        ),
    );
    assert_eq!(approved_replay_sse.status(), StatusCode::OK);
    let rejected_replay_sse = gateway_runtime.block_on(
        restarted_router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &restarted_cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("rejected Note candidate replay Gateway SSE request"),
        ),
    );
    assert_eq!(rejected_replay_sse.status(), StatusCode::OK);
    let terminal_replay_sse = gateway_runtime.block_on(
        restarted_router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &restarted_cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Note candidate terminal replay Gateway SSE request"),
        ),
    );
    assert_eq!(terminal_replay_sse.status(), StatusCode::OK);
    let extraction_position = started
        .iter()
        .position(|runtime| runtime.module_id == COMMUNICATION_NOTE_CANDIDATE_MODULE_ID_V1)
        .expect("started Note candidate extraction runtime");
    let extraction = started.remove(extraction_position);
    let extraction =
        restart_note_candidate_runtime_v1(&supervisor, &store, &root.join("runtime"), extraction);
    started.insert(extraction_position, extraction);
    let review_position = started
        .iter()
        .position(|runtime| runtime.module_id == REVIEW_NOTE_CANDIDATE_MODULE_ID_V1)
        .expect("started Review Note candidate runtime");
    let review = started.remove(review_position);
    let review =
        restart_note_candidate_runtime_v1(&supervisor, &store, &root.join("runtime"), review);
    started.insert(review_position, review);
    let replayed_approved_extraction = read_note_candidate_extraction_terminal_event_v1(
        approved_replay_sse,
        &gateway_runtime,
        &approved_start.run_id,
    );
    let replayed_rejected_extraction = read_note_candidate_extraction_terminal_event_v1(
        rejected_replay_sse,
        &gateway_runtime,
        &rejected_start.run_id,
    );
    let replayed =
        read_note_candidate_terminal_events_v1(terminal_replay_sse, &gateway_runtime, &reviews);
    assert_eq!(
        replayed_approved_extraction.cursor,
        approved_extraction_cursor
    );
    assert_eq!(
        replayed_rejected_extraction.cursor,
        rejected_extraction_cursor
    );
    assert_eq!(replayed.approved.cursor, approved_cursor);
    assert_eq!(replayed.rejected.cursor, rejected_cursor);
    let restarted_page = list_note_candidates_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        Vec::new(),
        2,
    );
    assert_eq!(restarted_page.reviews.len(), 2);
    assert!(restarted_page.next_after_review_id.is_empty());
    assert_knowledge_reject_stale_blob_receipt_v1(&gateway_runtime, &store);
    assert_knowledge_lifecycle_v1(
        &store,
        &supervisor,
        &root,
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &mut started,
    );

    // Effective NOLOGIN/NOSUPERUSER/NOBYPASSRLS proof for all Note Review tables.
    gateway_runtime.block_on(assert_review_owner_rls_v1(
        "makosh_review_note_rls_test",
        &[
            "review_note_candidate_submissions",
            "review_note_candidate_state",
            "review_note_candidate_operations",
            "review_note_candidate_promotion_inbox",
            "review_note_candidate_outbox",
            "review_note_candidate_realtime",
        ],
    ));

    supervisor.shutdown().expect("stop managed processes");
    assert_reviewed_note_candidate_persistence_negatives_v1(&gateway_runtime);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove reviewed Note candidate fixture");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS and Knowledge binaries"]
fn managed_knowledge_lifecycle_search_replays_and_restarts_with_owner_rls() {
    managed_note_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart();
}

#[allow(clippy::too_many_arguments)]
fn assert_knowledge_lifecycle_v1(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    router: &NoteCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    started: &mut Vec<StartedNoteCandidateRuntimeV1>,
) {
    // AddSource is exercised through the public Gateway route; the final
    // database matrix runs with an effective NOSUPERUSER/NOBYPASSRLS role.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Knowledge lifecycle wall clock")
        .as_millis() as i64
        - 1_000;
    let time = |offset: i64| TimestampV1 {
        unix_seconds: (now + offset) / 1_000,
        nanos: (((now + offset) % 1_000) * 1_000_000) as i32,
    };
    let create = |operation: u8, title: &str| -> KnowledgeNoteMutationResultV1 {
        post_proto(
            router,
            runtime,
            cookie,
            KNOWLEDGE_CREATE_CONNECT_PATH_V1,
            CreateKnowledgeNoteRequestV1 {
                operation_id: vec![operation; 16],
                note_id: Vec::new(),
                logical_owner_id: String::new(),
                title: title.to_owned(),
                body: format!("{title} private owner body"),
                created_at: Some(time(0)),
            },
        )
    };
    let first = create(0xb1, "Lifecycle primary");
    let first_note = first.note.expect("created primary Knowledge note");
    assert_eq!(first_note.note_revision, 1);
    let second = create(0xb2, "Lifecycle secondary");
    let second_note = second.note.expect("created secondary Knowledge note");

    let updated: KnowledgeNoteMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        KNOWLEDGE_UPDATE_CONNECT_PATH_V1,
        UpdateKnowledgeNoteRequestV1 {
            operation_id: vec![0xb3; 16],
            note_id: first_note.note_id.clone(),
            logical_owner_id: String::new(),
            expected_note_revision: 1,
            title: Some("Lifecycle primary updated".to_owned()),
            body: None,
            updated_at: Some(time(1)),
        },
    );
    assert_eq!(
        updated.note.as_ref().expect("updated note").note_revision,
        2
    );

    let attached: KnowledgeNoteMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        KNOWLEDGE_ADD_SOURCE_CONNECT_PATH_V1,
        AddKnowledgeSourceRequestV1 {
            operation_id: vec![0xb4; 16],
            note_id: first_note.note_id.clone(),
            logical_owner_id: String::new(),
            expected_note_revision: 2,
            source_id: Vec::new(),
            source_owner_id: "communications".to_owned(),
            source_record_id: vec![0xc1; 16],
            source_revision: 7,
            evidence_digest: vec![0xd1; 32],
            changed_at: Some(time(2)),
        },
    );
    assert_eq!(
        attached.note.as_ref().expect("attached note").note_revision,
        3
    );

    let source_id: Vec<u8> = runtime.block_on(async {
        let pool = note_candidate_admin_pool_v1().await;
        let id = sqlx::query_scalar(
            "SELECT source_id FROM makosh_data.knowledge_sources \
             WHERE logical_owner_id=$1 AND note_id=$2",
        )
        .bind(NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
        .bind(&first_note.note_id)
        .fetch_one(&pool)
        .await
        .expect("load attached public Knowledge source");
        pool.close().await;
        id
    });
    let removed: KnowledgeNoteMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        KNOWLEDGE_REMOVE_SOURCE_CONNECT_PATH_V1,
        RemoveKnowledgeSourceRequestV1 {
            operation_id: vec![0xb5; 16],
            note_id: first_note.note_id.clone(),
            logical_owner_id: String::new(),
            expected_note_revision: 3,
            source_id,
            changed_at: Some(time(3)),
        },
    );
    assert_eq!(
        removed
            .note
            .as_ref()
            .expect("removed source note")
            .note_revision,
        4
    );

    let archived: KnowledgeNoteMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        KNOWLEDGE_SET_STATE_CONNECT_PATH_V1,
        SetKnowledgeNoteStateRequestV1 {
            operation_id: vec![0xb6; 16],
            note_id: first_note.note_id.clone(),
            logical_owner_id: String::new(),
            expected_note_revision: 4,
            state: KnowledgeNoteStateV1::KnowledgeNoteStateArchived as i32,
            changed_at: Some(time(4)),
        },
    );
    assert_eq!(
        archived.note.as_ref().expect("archived note").note_revision,
        5
    );
    let restore_request = SetKnowledgeNoteStateRequestV1 {
        operation_id: vec![0xb7; 16],
        note_id: first_note.note_id.clone(),
        logical_owner_id: String::new(),
        expected_note_revision: 5,
        state: KnowledgeNoteStateV1::KnowledgeNoteStateActive as i32,
        changed_at: Some(time(5)),
    };
    let restored: KnowledgeNoteMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        KNOWLEDGE_SET_STATE_CONNECT_PATH_V1,
        restore_request.clone(),
    );
    let replayed: KnowledgeNoteMutationResultV1 = post_proto(
        router,
        runtime,
        cookie,
        KNOWLEDGE_SET_STATE_CONNECT_PATH_V1,
        restore_request,
    );
    assert_eq!(
        restored, replayed,
        "exact Knowledge operation replay response"
    );

    let mut cursor = Vec::new();
    let mut ids = Vec::new();
    loop {
        let page: ListKnowledgeNotesResultV1 = post_proto(
            router,
            runtime,
            cookie,
            KNOWLEDGE_LIST_CONNECT_PATH_V1,
            ListKnowledgeNotesRequestV1 {
                logical_owner_id: String::new(),
                after_note_id: cursor,
                limit: 1,
            },
        );
        assert_eq!(page.notes.len(), 1);
        ids.push(page.notes[0].note_id.clone());
        if page.next_after_note_id.is_empty() {
            break;
        }
        cursor = page.next_after_note_id;
    }
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        ids.len()
    );
    assert!(ids.contains(&first_note.note_id));
    assert!(ids.contains(&second_note.note_id));
    let searched: ListKnowledgeNotesResultV1 = post_proto(
        router,
        runtime,
        cookie,
        KNOWLEDGE_SEARCH_CONNECT_PATH_V1,
        SearchKnowledgeNotesRequestV1 {
            logical_owner_id: String::new(),
            query: "primary updated".to_owned(),
            after_note_id: Vec::new(),
            limit: 10,
        },
    );
    assert_eq!(searched.notes.len(), 1);
    assert_eq!(searched.notes[0].note_id, first_note.note_id);

    let position = started
        .iter()
        .position(|value| value.module_id == KNOWLEDGE_MODULE_ID_V1)
        .expect("started Knowledge runtime");
    let knowledge = started.remove(position);
    let knowledge =
        restart_note_candidate_runtime_v1(supervisor, store, &root.join("runtime"), knowledge);
    started.insert(position, knowledge);
    let restarted: KnowledgeNoteV1 = post_proto(
        router,
        runtime,
        cookie,
        KNOWLEDGE_GET_CONNECT_PATH_V1,
        GetKnowledgeNoteRequestV1 {
            logical_owner_id: String::new(),
            note_id: first_note.note_id,
        },
    );
    assert_eq!(restarted.note_revision, 6);
    assert_eq!(restarted.title, "Lifecycle primary updated");

    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    loop {
        let (operations, pending, public_envelopes): (i64, i64, Vec<Vec<u8>>) =
            runtime.block_on(async {
                let pool = note_candidate_admin_pool_v1().await;
                let operations = sqlx::query_scalar(
                    "SELECT count(*) FROM makosh_data.knowledge_client_operations WHERE logical_owner_id=$1",
                )
                .bind(NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
                .fetch_one(&pool)
                .await
                .expect("count Knowledge lifecycle operations");
                let pending = sqlx::query_scalar(
                    "SELECT count(*) FROM makosh_data.knowledge_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL",
                )
                .bind(NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
                .fetch_one(&pool)
                .await
                .expect("count pending Knowledge outbox");
                let envelopes = sqlx::query_scalar(
                    "SELECT envelope_bytes FROM makosh_data.knowledge_outbox WHERE logical_owner_id=$1",
                )
                .bind(NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
                .fetch_all(&pool)
                .await
                .expect("load Knowledge public envelopes");
                pool.close().await;
                (operations, pending, envelopes)
            });
        for envelope in &public_envelopes {
            for marker in [
                b"Lifecycle primary updated".as_slice(),
                b"private owner body".as_slice(),
                b"communications".as_slice(),
            ] {
                assert!(
                    !envelope
                        .windows(marker.len())
                        .any(|window| window == marker)
                );
            }
        }
        if operations == 7 && pending == 0 {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Knowledge lifecycle relay did not drain: operations={operations}, pending={pending}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }

    runtime.block_on(assert_review_owner_rls_v1(
        "makosh_knowledge_lifecycle_rls_test",
        &[
            "knowledge_reviewed_candidate_inbox",
            "knowledge_state",
            "knowledge_outbox",
            "knowledge_sources",
            "knowledge_client_operations",
        ],
    ));
}

fn note_candidate_start_request_v1(
    operation_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
) -> StartCommunicationNoteCandidateRequestV1 {
    StartCommunicationNoteCandidateRequestV1 {
        protocol_major: 1,
        operation_id: operation_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
    }
}

fn route_note_candidate_start_as_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    workflow: &StartedNoteCandidateRuntimeV1,
    logical_owner_id: &str,
    request_id: u64,
    request: StartCommunicationNoteCandidateRequestV1,
) -> ModuleClientResponseV1 {
    let request = encode_note_candidate_module_request_v1(logical_owner_id, request_id, request);
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &workflow.registration_id,
        &workflow.runtime_instance_id,
        workflow.runtime_generation,
        workflow.grant_epoch,
        COMMUNICATION_NOTE_CANDIDATE_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route Note candidate owner-fence request");
    ModuleClientResponseV1::decode(bytes.as_slice()).expect("decode Note candidate response")
}

fn assert_note_candidate_runtime_fences_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    workflow: &StartedNoteCandidateRuntimeV1,
    source_message_id: [u8; 16],
) {
    let request = encode_note_candidate_module_request_v1(
        NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
        699,
        note_candidate_start_request_v1([0x1f; 16], source_message_id, 2),
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
            COMMUNICATION_NOTE_CANDIDATE_CAPABILITY_ID_V1,
            &request,
        );
        assert_eq!(
            crate::modules::capability::router::route_managed_client_request(
                store,
                &supervisor.relay_port(),
                &route,
            )
            .expect_err("stale Note candidate runtime fence"),
            "managed runtime fence is stale"
        );
    }
}

fn encode_note_candidate_module_request_v1(
    logical_owner_id: &str,
    request_id: u64,
    request: StartCommunicationNoteCandidateRequestV1,
) -> Vec<u8> {
    ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: COMMUNICATION_NOTE_CANDIDATE_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_NOTE_CANDIDATE_OWNER_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: COMMUNICATION_NOTE_CANDIDATE_OWNER_V1.to_owned(),
            name: COMMUNICATION_NOTE_CANDIDATE_COMMAND_CONTRACT_NAME_V1.to_owned(),
            major: COMMUNICATION_NOTE_CANDIDATE_CONTRACT_MAJOR_V1,
            revision: COMMUNICATION_NOTE_CANDIDATE_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATION_NOTE_CANDIDATE_SCHEMA_SHA256.to_vec(),
        }),
        request_id,
        request_payload: request.encode_to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec()
}
