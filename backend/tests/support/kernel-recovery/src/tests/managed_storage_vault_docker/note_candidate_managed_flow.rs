//! Live signed admission for the event-only reviewed Note candidate chain.

use super::*;

use crate::identity::device::signer::DeviceSigner;
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
use makosh_knowledge_command_api::{KNOWLEDGE_MODULE_ID_V1, KNOWLEDGE_OWNER_ID_V1};
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
        &router,
        &gateway_runtime,
        &cookie,
        &approved_start.run_id,
    );
    let rejected_extraction_event = read_note_candidate_extraction_terminal_event_v1(
        &router,
        &gateway_runtime,
        &cookie,
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

    let terminal =
        read_note_candidate_terminal_events_v1(&router, &gateway_runtime, &cookie, &reviews);
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
    let restarted_router =
        note_candidate_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed_approved_extraction = read_note_candidate_extraction_terminal_event_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &approved_start.run_id,
    );
    let replayed_rejected_extraction = read_note_candidate_extraction_terminal_event_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &rejected_start.run_id,
    );
    let replayed = read_note_candidate_terminal_events_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &reviews,
    );
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
    assert_knowledge_reject_stale_blob_receipt_v1(&gateway_runtime, &store);

    supervisor.shutdown().expect("stop managed processes");
    assert_reviewed_note_candidate_persistence_negatives_v1(&gateway_runtime);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove reviewed Note candidate fixture");
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
