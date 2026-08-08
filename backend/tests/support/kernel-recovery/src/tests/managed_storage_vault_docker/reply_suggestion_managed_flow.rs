//! Full managed Reply Suggestion orchestration through Gateway and replayable SSE.

use std::{
    io::ErrorKind,
    net::TcpListener,
    sync::atomic::AtomicUsize,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use super::*;

use crate::identity::device::signer::DeviceSigner;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_communication_reply_suggestion_api::{
    COMMUNICATION_REPLY_SUGGESTION_CAPABILITY_ID_V1,
    COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_REPLY_SUGGESTION_CONTRACT_MAJOR_V1,
    COMMUNICATION_REPLY_SUGGESTION_CONTRACT_REVISION_V1,
    COMMUNICATION_REPLY_SUGGESTION_MODULE_ID_V1, COMMUNICATION_REPLY_SUGGESTION_OWNER_V1,
    COMMUNICATION_REPLY_SUGGESTION_QUERY_CONNECT_PATH_V1,
    COMMUNICATION_REPLY_SUGGESTION_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_REPLY_SUGGESTION_REALTIME_EVENT_KIND_V1,
    COMMUNICATION_REPLY_SUGGESTION_SCHEMA_SHA256,
    wire::{
        GetReplySuggestionRequestV1, GetReplySuggestionResponseV1, ReplySuggestionCompletenessV1,
        ReplySuggestionErrorCodeV1, ReplySuggestionLanguageV1, ReplySuggestionStateV1,
        ReplySuggestionStatusChangedV1, ReplySuggestionToneV1, StartReplySuggestionRequestV1,
        StartReplySuggestionResponseV1,
    },
};
use makosh_communication_reply_suggestion_runtime::COMMUNICATION_REPLY_SUGGESTION_STORAGE_CAPABILITY_ID_V1;
use makosh_gateway_protocol::v1::{
    ClientRealtimeEventV1, ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};
use makosh_kernel_control_store::{ModuleRegistrationState, PlatformStorageBindingStateV1};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};

const SOURCE_BODY: &[u8] = b"fixture source body for custody transfer";
const SOURCE_SENDER: &[u8] = b"Alice Example <alice@example.test>";
const SOURCE_SUBJECT: &[u8] = b"Quarterly update";

type ReplySuggestionGateway = makosh_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, Reply Suggestion, AI inference and Ollama AI binaries"]
fn managed_reply_suggestion_reaches_ai_and_replays_through_gateway_sse() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let ollama_probe = UnavailableOllamaProbeV1::start();
    let ollama_port = ollama_probe.port();

    let root = unique_target_root("makosh-managed-reply-suggestion");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_reply_suggestion_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            REPLY_SUGGESTION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Reply Suggestion logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        REPLY_SUGGESTION_LOGICAL_OWNER_ID_V1,
    );
    let admitted_reply = admit_reply_suggestion_runtime_v1(&store);
    let admitted_ollama = admit_ollama_ai_runtime_v1(&store);
    let admitted_ai = admit_ai_inference_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_ai_module_request_router_v1(&supervisor, &store);
    configure_reply_suggestion_realtime_v1(&supervisor, &store, realtime.clone());
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
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
    let admitted_reply = prepare_reply_suggestion_runtime_v1(&supervisor, &store, admitted_reply);
    let admitted_ollama = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted_ollama);
    let admitted_ai = prepare_ai_inference_runtime_v1(&supervisor, &store, admitted_ai);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let ollama = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_ollama,
        ollama_port,
    );
    let ai = start_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted_ai);
    let reply = start_reply_suggestion_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_reply,
    );
    assert_eq!(ollama.runtime_generation, 1);
    assert_eq!(ai.runtime_generation, 1);
    assert_eq!(reply.runtime_generation, 1);

    let source_message_id = assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        false,
    );
    let source_message_id: [u8; 16] = source_message_id
        .try_into()
        .expect("canonical source message ID");
    let wrong_owner = route_reply_suggestion_as(
        &store,
        &supervisor,
        &reply.registration_id,
        "owner-2",
        700,
        start_request([0x80; 16], source_message_id, 2),
    );
    assert_eq!(wrong_owner.request_id, 700);
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());
    ollama_probe.assert_attempts(0);
    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = reply_suggestion_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );

    let request = start_request([0x81; 16], source_message_id, 2);
    let accepted = post_proto::<_, StartReplySuggestionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(accepted.error, unspecified_error());
    assert_eq!(accepted.run_id.len(), 16);
    assert!(matches!(
        state(accepted.state),
        ReplySuggestionStateV1::ReplySuggestionStatePreparingSource
            | ReplySuggestionStateV1::ReplySuggestionStateAwaitingInference
            | ReplySuggestionStateV1::ReplySuggestionStateRejected
    ));

    let first = wait_for_terminal_reply(
        &router,
        &gateway_runtime,
        &cookie,
        &accepted.run_id,
        ReplySuggestionErrorCodeV1::ReplySuggestionErrorCodeInferenceRejected,
    );
    assert_eq!(
        state(first.state),
        ReplySuggestionStateV1::ReplySuggestionStateRejected
    );
    assert_eq!(
        error(first.error),
        ReplySuggestionErrorCodeV1::ReplySuggestionErrorCodeInferenceRejected
    );
    assert!(first.candidate.is_none());
    assert_eq!(first.source_message_id, source_message_id);
    assert_eq!(first.expected_source_revision, 2);
    assert!(first.state_revision >= 4);
    let attempted_connections = ollama_probe.attempts();
    assert!(
        attempted_connections > 0,
        "full Reply Suggestion path must reach the Ollama HTTP boundary"
    );

    let first_event =
        read_terminal_reply_sse_event(&router, &gateway_runtime, &cookie, &accepted.run_id);
    let first_payload = ReplySuggestionStatusChangedV1::decode(first_event.payload.as_slice())
        .expect("Reply Suggestion realtime payload");
    assert_eq!(first_payload.run_id, accepted.run_id);
    assert_eq!(
        state(first_payload.state),
        ReplySuggestionStateV1::ReplySuggestionStateRejected
    );
    assert_eq!(
        error(first_payload.error),
        ReplySuggestionErrorCodeV1::ReplySuggestionErrorCodeInferenceRejected
    );
    assert_private_content_absent(&first_event.encode_to_vec());
    assert!(
        !first_event
            .encode_to_vec()
            .windows(source_message_id.len())
            .any(|window| window == source_message_id),
        "client realtime must not expose source message identity"
    );
    let first_cursor = first_event.cursor.clone();

    let duplicate = post_proto::<_, StartReplySuggestionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(duplicate.run_id, accepted.run_id);
    assert_eq!(
        state(duplicate.state),
        ReplySuggestionStateV1::ReplySuggestionStateRejected
    );
    assert_eq!(
        error(duplicate.error),
        ReplySuggestionErrorCodeV1::ReplySuggestionErrorCodeInferenceRejected
    );
    ollama_probe.assert_attempts(attempted_connections);

    let mut conflicting_request = request;
    conflicting_request.language = ReplySuggestionLanguageV1::ReplySuggestionLanguageRussian as i32;
    let conflicting = post_proto::<_, StartReplySuggestionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONNECT_PATH_V1,
        conflicting_request,
    );
    assert_eq!(
        error(conflicting.error),
        ReplySuggestionErrorCodeV1::ReplySuggestionErrorCodeInvalidRequest
    );
    ollama_probe.assert_attempts(attempted_connections);

    assert!(
        realtime
            .revoke_owner(REPLY_SUGGESTION_LOGICAL_OWNER_ID_V1)
            .expect("clear Reply Suggestion Gateway replay cache")
    );
    let previous_generation = reply.runtime_generation;
    let reply =
        restart_reply_suggestion_runtime_v1(&supervisor, &store, &root.join("runtime"), reply);
    assert_eq!(reply.runtime_generation, previous_generation + 1);
    let restarted_router =
        reply_suggestion_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed_query = get_reply(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed_query, first);
    let replayed_event = read_terminal_reply_sse_event(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed_event.cursor, first_cursor);
    assert_eq!(replayed_event.payload, first_event.payload);
    assert_private_content_absent(&replayed_event.encode_to_vec());
    ollama_probe.assert_attempts(attempted_connections);

    let stale_request = start_request([0x82; 16], source_message_id, 1);
    let stale = post_proto::<_, StartReplySuggestionResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONNECT_PATH_V1,
        stale_request,
    );
    assert_eq!(stale.error, unspecified_error());
    let stale_terminal = wait_for_terminal_reply(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &stale.run_id,
        ReplySuggestionErrorCodeV1::ReplySuggestionErrorCodeSourceRejected,
    );
    assert_eq!(
        state(stale_terminal.state),
        ReplySuggestionStateV1::ReplySuggestionStateRejected
    );
    assert_eq!(
        error(stale_terminal.error),
        ReplySuggestionErrorCodeV1::ReplySuggestionErrorCodeSourceRejected
    );
    assert!(stale_terminal.candidate.is_none());
    ollama_probe.assert_attempts(attempted_connections);

    let (owner_runtime_dir, owner_control) =
        start_owner_control(&data, &store, &shutdown, &supervisor);
    let revoked = transition_registration(
        &owner_runtime_dir,
        &owner_signer,
        &reply.registration_id,
        "revoked",
    );
    assert_eq!(revoked.state, "revoked");
    assert!(revoked.grant_epoch > reply.grant_epoch);
    assert_eq!(
        store
            .module_registration(&reply.registration_id)
            .expect("read revoked Reply Suggestion registration")
            .expect("revoked Reply Suggestion registration")
            .state(),
        ModuleRegistrationState::Revoked
    );
    assert_eq!(
        store
            .platform_storage_binding(
                &reply.registration_id,
                COMMUNICATION_REPLY_SUGGESTION_STORAGE_CAPABILITY_ID_V1,
            )
            .expect("read revoked Reply Suggestion Storage binding")
            .expect("revoked Reply Suggestion Storage binding")
            .state(),
        PlatformStorageBindingStateV1::Revoking
    );
    assert!(
        !supervisor
            .stop_if_active(&reply.registration_id)
            .expect("observe stopped Reply Suggestion workflow"),
        "owner revoke must already stop the exact workflow process"
    );
    for (registration_id, owner) in [
        (ollama.registration_id.as_str(), "Ollama integration"),
        (ai.registration_id.as_str(), "AI engine"),
        (COMMUNICATIONS_REGISTRATION, "Communications domain"),
    ] {
        assert!(
            supervisor
                .is_active(registration_id)
                .unwrap_or_else(|error| panic!("observe {owner} after workflow revoke: {error}")),
            "Reply Suggestion revoke must not stop {owner}"
        );
    }
    assert_eq!(
        post_proto_status(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            COMMUNICATION_REPLY_SUGGESTION_QUERY_CONNECT_PATH_V1,
            GetReplySuggestionRequestV1 {
                protocol_major: 1,
                run_id: accepted.run_id.clone(),
            },
        ),
        StatusCode::NOT_FOUND,
        "revoked workflow route must fail closed at Gateway"
    );

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    owner_control
        .join()
        .expect("join Reply Suggestion owner control server")
        .expect("Reply Suggestion owner control server");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Reply Suggestion fixture");
    std::fs::remove_dir_all(data).expect("remove short Reply Suggestion Kernel fixture");
}

#[test]
#[ignore = "requires disposable Docker plus a real loopback Ollama service with makosh-conformance:latest"]
fn managed_reply_suggestion_completes_real_provider_through_gateway_sse() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let ollama_port = required("MAKOSH_OLLAMA_LIVE_PORT")
        .parse::<u16>()
        .expect("valid live Ollama port");
    let root = unique_target_root("makosh-managed-reply-suggestion-live");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_reply_suggestion_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            REPLY_SUGGESTION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim live Reply Suggestion logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        REPLY_SUGGESTION_LOGICAL_OWNER_ID_V1,
    );
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_reply = admit_reply_suggestion_runtime_v1(&store);
    let admitted_ollama = admit_ollama_ai_runtime_v1(&store);
    let admitted_ai = admit_ai_inference_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_ai_module_request_router_v1(&supervisor, &store);
    configure_reply_suggestion_realtime_v1(&supervisor, &store, realtime.clone());
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
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
    let admitted_reply = prepare_reply_suggestion_runtime_v1(&supervisor, &store, admitted_reply);
    let admitted_ollama = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted_ollama);
    let admitted_ai = prepare_ai_inference_runtime_v1(&supervisor, &store, admitted_ai);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let ollama = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_ollama,
        ollama_port,
    );
    let _ai =
        start_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted_ai);
    let reply = start_reply_suggestion_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_reply,
    );
    let source_message_id = assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        false,
    );
    let source_message_id: [u8; 16] = source_message_id
        .try_into()
        .expect("canonical source message ID");
    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = reply_suggestion_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );

    let request = start_request([0x91; 16], source_message_id, 2);
    let accepted = post_proto::<_, StartReplySuggestionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(accepted.error, unspecified_error());
    let ready = wait_for_ready_reply(&router, &gateway_runtime, &cookie, &accepted.run_id);
    assert_eq!(ready.source_message_id, source_message_id);
    assert_eq!(ready.expected_source_revision, 2);
    assert!(ready.state_revision >= 4);
    assert_eq!(ready.error, unspecified_error());
    let candidate = ready
        .candidate
        .as_ref()
        .expect("typed Reply Suggestion candidate");
    assert!(!candidate.body_utf8.is_empty());
    assert_eq!(
        candidate.resolved_tone,
        ReplySuggestionToneV1::ReplySuggestionToneFormal as i32,
        "Professional is resolved through the bounded AI contract as Formal"
    );
    assert_eq!(
        candidate.resolved_language,
        ReplySuggestionLanguageV1::ReplySuggestionLanguageEnglish as i32
    );
    assert_eq!(
        candidate.completeness,
        ReplySuggestionCompletenessV1::ReplySuggestionCompletenessComplete as i32
    );
    assert!(candidate.confidence_basis_points <= 10_000);

    let first_event =
        read_terminal_reply_sse_event(&router, &gateway_runtime, &cookie, &accepted.run_id);
    let first_payload = ReplySuggestionStatusChangedV1::decode(first_event.payload.as_slice())
        .expect("successful Reply Suggestion realtime payload");
    assert_eq!(first_payload.run_id, accepted.run_id);
    assert_eq!(
        state(first_payload.state),
        ReplySuggestionStateV1::ReplySuggestionStateReady
    );
    assert_private_content_absent(&first_event.encode_to_vec());
    assert!(
        !first_event
            .encode_to_vec()
            .windows(source_message_id.len())
            .any(|window| window == source_message_id),
        "successful realtime must not expose source message identity"
    );

    supervisor
        .stop(&ollama.registration_id)
        .expect("stop Ollama dependency before successful workflow replay");
    assert!(
        realtime
            .revoke_owner(REPLY_SUGGESTION_LOGICAL_OWNER_ID_V1)
            .expect("clear successful Reply Suggestion Gateway replay cache")
    );
    let reply =
        restart_reply_suggestion_runtime_v1(&supervisor, &store, &root.join("runtime"), reply);
    let restarted_router =
        reply_suggestion_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    assert_eq!(
        get_reply(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            &accepted.run_id,
        ),
        ready
    );
    let replayed_event = read_terminal_reply_sse_event(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed_event.cursor, first_event.cursor);
    assert_eq!(replayed_event.payload, first_event.payload);
    let duplicate = post_proto::<_, StartReplySuggestionResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONNECT_PATH_V1,
        request,
    );
    assert_eq!(duplicate.run_id, accepted.run_id);
    assert_eq!(
        state(duplicate.state),
        ReplySuggestionStateV1::ReplySuggestionStateReady
    );
    assert_eq!(reply.runtime_generation, 2);

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove live Reply Suggestion fixture");
    std::fs::remove_dir_all(data).expect("remove short live Reply Suggestion Kernel fixture");
}

fn start_request(
    operation_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
) -> StartReplySuggestionRequestV1 {
    StartReplySuggestionRequestV1 {
        protocol_major: 1,
        operation_id: operation_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
        tone: ReplySuggestionToneV1::ReplySuggestionToneProfessional as i32,
        language: ReplySuggestionLanguageV1::ReplySuggestionLanguageEnglish as i32,
    }
}

fn route_reply_suggestion_as(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    logical_owner_id: &str,
    request_id: u64,
    request: StartReplySuggestionRequestV1,
) -> ModuleClientResponseV1 {
    let launch = store
        .effective_managed_launch_record(registration_id)
        .expect("read Reply Suggestion launch")
        .expect("Reply Suggestion launch is active");
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: COMMUNICATION_REPLY_SUGGESTION_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_REPLY_SUGGESTION_OWNER_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: COMMUNICATION_REPLY_SUGGESTION_OWNER_V1.to_owned(),
            name: COMMUNICATION_REPLY_SUGGESTION_COMMAND_CONTRACT_NAME_V1.to_owned(),
            major: COMMUNICATION_REPLY_SUGGESTION_CONTRACT_MAJOR_V1,
            revision: COMMUNICATION_REPLY_SUGGESTION_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATION_REPLY_SUGGESTION_SCHEMA_SHA256.to_vec(),
        }),
        request_id,
        request_payload: request.encode_to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        registration_id,
        launch.runtime_instance_id(),
        launch.runtime_generation(),
        launch.grant_epoch(),
        COMMUNICATION_REPLY_SUGGESTION_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route Reply Suggestion owner-fence request");
    ModuleClientResponseV1::decode(bytes.as_slice())
        .expect("decode Reply Suggestion owner-fence response")
}

fn wait_for_terminal_reply(
    router: &ReplySuggestionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
    expected_error: ReplySuggestionErrorCodeV1,
) -> GetReplySuggestionResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(12);
    loop {
        let response = get_reply(router, runtime, cookie, run_id);
        if state(response.state) == ReplySuggestionStateV1::ReplySuggestionStateRejected {
            assert_eq!(error(response.error), expected_error);
            return response;
        }
        assert!(
            Instant::now() < deadline,
            "Reply Suggestion did not reach the expected terminal state: {response:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_ready_reply(
    router: &ReplySuggestionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetReplySuggestionResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let response = get_reply(router, runtime, cookie, run_id);
        match state(response.state) {
            ReplySuggestionStateV1::ReplySuggestionStateReady => return response,
            ReplySuggestionStateV1::ReplySuggestionStateRejected => {
                panic!("live Reply Suggestion was rejected: {response:?}")
            }
            _ => {}
        }
        assert!(
            Instant::now() < deadline,
            "Reply Suggestion did not reach Ready: {response:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn get_reply(
    router: &ReplySuggestionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetReplySuggestionResponseV1 {
    post_proto(
        router,
        runtime,
        cookie,
        COMMUNICATION_REPLY_SUGGESTION_QUERY_CONNECT_PATH_V1,
        GetReplySuggestionRequestV1 {
            protocol_major: 1,
            run_id: run_id.to_vec(),
        },
    )
}

fn reply_suggestion_gateway(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> ReplySuggestionGateway {
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("reply-suggestion-gateway-cert.der"),
        root.join("reply-suggestion-gateway-key.der"),
    )
    .expect("Gateway configuration");
    crate::platform::gateway::gateway_service(
        Arc::clone(store),
        data,
        supervisor.clone(),
        realtime,
        &configuration,
        None,
    )
    .expect("compose Reply Suggestion Gateway routes")
}

fn post_proto<M, R>(
    router: &ReplySuggestionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    path: &str,
    message: M,
) -> R
where
    M: Message,
    R: Message + Default,
{
    let payload = message.encode_to_vec();
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let response = runtime.block_on(
            router.route(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/connect+proto")
                    .header("cookie", cookie)
                    .body(http_body_util::Full::new(Bytes::from(payload.clone())))
                    .expect("Reply Suggestion Gateway request"),
            ),
        );
        let status = response.status();
        let bytes = runtime
            .block_on(response.into_body().collect())
            .expect("Reply Suggestion Gateway response")
            .to_bytes();
        if status == StatusCode::SERVICE_UNAVAILABLE && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
            continue;
        }
        assert_eq!(
            status,
            StatusCode::OK,
            "Reply Suggestion Gateway response body: {}",
            String::from_utf8_lossy(&bytes)
        );
        return R::decode(bytes.as_ref()).expect("decode Reply Suggestion Gateway response");
    }
}

fn post_proto_status<M>(
    router: &ReplySuggestionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    path: &str,
    message: M,
) -> StatusCode
where
    M: Message,
{
    runtime
        .block_on(
            router.route(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/connect+proto")
                    .header("cookie", cookie)
                    .body(http_body_util::Full::new(Bytes::from(
                        message.encode_to_vec(),
                    )))
                    .expect("Reply Suggestion Gateway status request"),
            ),
        )
        .status()
}

fn read_terminal_reply_sse_event(
    router: &ReplySuggestionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> ClientRealtimeEventV1 {
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Reply Suggestion Gateway SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(8),
            find_terminal_reply_event(response.into_body(), run_id),
        )
        .await
        .expect("Reply Suggestion SSE timeout")
    })
}

async fn find_terminal_reply_event<B>(mut body: B, run_id: &[u8]) -> ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Reply Suggestion SSE frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Reply Suggestion SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Reply Suggestion frame");
            let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                .expect("Reply Suggestion realtime frame");
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            if event.contract_name != COMMUNICATION_REPLY_SUGGESTION_REALTIME_CONTRACT_NAME_V1
                || event.event_kind != COMMUNICATION_REPLY_SUGGESTION_REALTIME_EVENT_KIND_V1
            {
                continue;
            }
            let payload = ReplySuggestionStatusChangedV1::decode(event.payload.as_slice())
                .expect("Reply Suggestion realtime payload");
            if payload.run_id == run_id
                && matches!(
                    state(payload.state),
                    ReplySuggestionStateV1::ReplySuggestionStateReady
                        | ReplySuggestionStateV1::ReplySuggestionStateRejected
                )
            {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before terminal Reply Suggestion event");
}

struct UnavailableOllamaProbeV1 {
    port: u16,
    attempts: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl UnavailableOllamaProbeV1 {
    fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind unavailable Ollama probe");
        listener
            .set_nonblocking(true)
            .expect("make unavailable Ollama probe nonblocking");
        let port = listener
            .local_addr()
            .expect("read unavailable Ollama probe address")
            .port();
        let attempts = Arc::new(AtomicUsize::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_attempts = Arc::clone(&attempts);
        let worker_shutdown = Arc::clone(&shutdown);
        let worker = std::thread::spawn(move || {
            while !worker_shutdown.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((_connection, _address)) => {
                        worker_attempts.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(error) if error.kind() == ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept unavailable Ollama probe request: {error}"),
                }
            }
        });
        Self {
            port,
            attempts,
            shutdown,
            worker: Some(worker),
        }
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn attempts(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }

    fn assert_attempts(&self, expected: usize) {
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(
            self.attempts(),
            expected,
            "persisted or source-rejected Reply Suggestion retried Ollama HTTP"
        );
    }
}

impl Drop for UnavailableOllamaProbeV1 {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            worker.join().expect("join unavailable Ollama probe");
        }
    }
}

fn assert_private_content_absent(bytes: &[u8]) {
    for private in [SOURCE_BODY, SOURCE_SENDER, SOURCE_SUBJECT] {
        assert!(
            !bytes.windows(private.len()).any(|window| window == private),
            "owner-private source content crossed the client realtime boundary"
        );
    }
}

fn state(value: i32) -> ReplySuggestionStateV1 {
    ReplySuggestionStateV1::try_from(value).expect("known Reply Suggestion state")
}

fn error(value: i32) -> ReplySuggestionErrorCodeV1 {
    ReplySuggestionErrorCodeV1::try_from(value).expect("known Reply Suggestion error")
}

fn unspecified_error() -> i32 {
    ReplySuggestionErrorCodeV1::ReplySuggestionErrorCodeUnspecified as i32
}
