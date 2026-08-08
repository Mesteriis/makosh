//! Full managed Communication Explanation orchestration through Gateway and replayable SSE.

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
use makosh_communication_explanation_api::{
    COMMUNICATION_EXPLANATION_CAPABILITY_ID_V1, COMMUNICATION_EXPLANATION_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_EXPLANATION_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_EXPLANATION_CONTRACT_MAJOR_V1, COMMUNICATION_EXPLANATION_CONTRACT_REVISION_V1,
    COMMUNICATION_EXPLANATION_MODULE_ID_V1, COMMUNICATION_EXPLANATION_OWNER_V1,
    COMMUNICATION_EXPLANATION_QUERY_CONNECT_PATH_V1,
    COMMUNICATION_EXPLANATION_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_EXPLANATION_REALTIME_EVENT_KIND_V1, COMMUNICATION_EXPLANATION_SCHEMA_SHA256,
    wire::{
        CommunicationExplanationCompletenessV1, CommunicationExplanationErrorCodeV1,
        CommunicationExplanationReasonKindV1, CommunicationExplanationSourceBasisV1,
        CommunicationExplanationStateV1, CommunicationExplanationStatusChangedV1,
        GetCommunicationExplanationRequestV1, GetCommunicationExplanationResponseV1,
        StartCommunicationExplanationRequestV1, StartCommunicationExplanationResponseV1,
    },
};
use makosh_communication_explanation_runtime::COMMUNICATION_EXPLANATION_STORAGE_CAPABILITY_ID_V1;
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

type CommunicationExplanationGateway = makosh_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, Communication Explanation, AI inference and Ollama AI binaries"]
fn managed_communication_explanation_reaches_ai_and_replays_through_gateway_sse() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let ollama_probe = UnavailableOllamaProbeV1::start();
    let ollama_port = ollama_probe.port();

    let root = unique_target_root("makosh-managed-communication-explanation");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communication_explanation_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            COMMUNICATION_EXPLANATION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Communication Explanation logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        COMMUNICATION_EXPLANATION_LOGICAL_OWNER_ID_V1,
    );
    let admitted_explanation = admit_communication_explanation_runtime_v1(&store);
    let admitted_ollama = admit_ollama_ai_runtime_v1(&store);
    let admitted_ai = admit_ai_inference_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_ai_module_request_router_v1(&supervisor, &store);
    configure_communication_explanation_realtime_v1(&supervisor, &store, realtime.clone());
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
    let admitted_explanation =
        prepare_communication_explanation_runtime_v1(&supervisor, &store, admitted_explanation);
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
    let explanation = start_communication_explanation_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_explanation,
    );
    assert_eq!(ollama.runtime_generation, 1);
    assert_eq!(ai.runtime_generation, 1);
    assert_eq!(explanation.runtime_generation, 1);

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
    assert_communication_explanation_runtime_fences(
        &store,
        &supervisor,
        &explanation,
        source_message_id,
    );
    let wrong_owner = route_communication_explanation_as(
        &store,
        &supervisor,
        &explanation.registration_id,
        "owner-2",
        700,
        start_request([0x80; 16], source_message_id, 2),
    );
    assert_eq!(wrong_owner.request_id, 700);
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());
    ollama_probe.assert_attempts(0);
    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router =
        communication_explanation_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );

    let request = start_request([0x81; 16], source_message_id, 2);
    let accepted = post_proto::<_, StartCommunicationExplanationResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_EXPLANATION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(accepted.error, unspecified_error());
    assert_eq!(accepted.run_id.len(), 16);
    assert!(matches!(
        state(accepted.state),
        CommunicationExplanationStateV1::CommunicationExplanationStatePreparingSource
            | CommunicationExplanationStateV1::CommunicationExplanationStateAwaitingInference
            | CommunicationExplanationStateV1::CommunicationExplanationStateRejected
    ));

    let first = wait_for_terminal_explanation(
        &router,
        &gateway_runtime,
        &cookie,
        &accepted.run_id,
        CommunicationExplanationErrorCodeV1::CommunicationExplanationErrorCodeInferenceRejected,
    );
    assert_eq!(
        state(first.state),
        CommunicationExplanationStateV1::CommunicationExplanationStateRejected
    );
    assert_eq!(
        error(first.error),
        CommunicationExplanationErrorCodeV1::CommunicationExplanationErrorCodeInferenceRejected
    );
    assert!(first.candidate.is_none());
    assert_eq!(first.source_message_id, source_message_id);
    assert_eq!(first.expected_source_revision, 2);
    assert!(first.state_revision >= 4);
    let attempted_connections = ollama_probe.attempts();
    assert!(
        attempted_connections > 0,
        "full Communication Explanation path must reach the Ollama HTTP boundary"
    );

    let first_event =
        read_terminal_explanation_sse_event(&router, &gateway_runtime, &cookie, &accepted.run_id);
    let first_payload =
        CommunicationExplanationStatusChangedV1::decode(first_event.payload.as_slice())
            .expect("Communication Explanation realtime payload");
    assert_eq!(first_payload.run_id, accepted.run_id);
    assert_eq!(
        state(first_payload.state),
        CommunicationExplanationStateV1::CommunicationExplanationStateRejected
    );
    assert_eq!(
        error(first_payload.error),
        CommunicationExplanationErrorCodeV1::CommunicationExplanationErrorCodeInferenceRejected
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

    let duplicate = post_proto::<_, StartCommunicationExplanationResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_EXPLANATION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(duplicate.run_id, accepted.run_id);
    assert_eq!(
        state(duplicate.state),
        CommunicationExplanationStateV1::CommunicationExplanationStateRejected
    );
    assert_eq!(
        error(duplicate.error),
        CommunicationExplanationErrorCodeV1::CommunicationExplanationErrorCodeInferenceRejected
    );
    ollama_probe.assert_attempts(attempted_connections);

    let mut conflicting_request = request;
    conflicting_request.expected_source_revision += 1;
    let conflicting = post_proto::<_, StartCommunicationExplanationResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_EXPLANATION_COMMAND_CONNECT_PATH_V1,
        conflicting_request,
    );
    assert_eq!(
        error(conflicting.error),
        CommunicationExplanationErrorCodeV1::CommunicationExplanationErrorCodeInvalidRequest
    );
    ollama_probe.assert_attempts(attempted_connections);

    assert!(
        realtime
            .revoke_owner(COMMUNICATION_EXPLANATION_LOGICAL_OWNER_ID_V1)
            .expect("clear Communication Explanation Gateway replay cache")
    );
    let previous_generation = explanation.runtime_generation;
    let explanation = restart_communication_explanation_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        explanation,
    );
    assert_eq!(explanation.runtime_generation, previous_generation + 1);
    let restarted_router =
        communication_explanation_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed_query = get_explanation(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed_query, first);
    let replayed_event = read_terminal_explanation_sse_event(
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
    let stale = post_proto::<_, StartCommunicationExplanationResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        COMMUNICATION_EXPLANATION_COMMAND_CONNECT_PATH_V1,
        stale_request,
    );
    assert_eq!(stale.error, unspecified_error());
    let stale_terminal = wait_for_terminal_explanation(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &stale.run_id,
        CommunicationExplanationErrorCodeV1::CommunicationExplanationErrorCodeSourceRejected,
    );
    assert_eq!(
        state(stale_terminal.state),
        CommunicationExplanationStateV1::CommunicationExplanationStateRejected
    );
    assert_eq!(
        error(stale_terminal.error),
        CommunicationExplanationErrorCodeV1::CommunicationExplanationErrorCodeSourceRejected
    );
    assert!(stale_terminal.candidate.is_none());
    ollama_probe.assert_attempts(attempted_connections);

    let (owner_runtime_dir, owner_control) =
        start_owner_control(&data, &store, &shutdown, &supervisor);
    let revoked = transition_registration(
        &owner_runtime_dir,
        &owner_signer,
        &explanation.registration_id,
        "revoked",
    );
    assert_eq!(revoked.state, "revoked");
    assert!(revoked.grant_epoch > explanation.grant_epoch);
    assert_eq!(
        store
            .module_registration(&explanation.registration_id)
            .expect("read revoked Communication Explanation registration")
            .expect("revoked Communication Explanation registration")
            .state(),
        ModuleRegistrationState::Revoked
    );
    assert_eq!(
        store
            .platform_storage_binding(
                &explanation.registration_id,
                COMMUNICATION_EXPLANATION_STORAGE_CAPABILITY_ID_V1,
            )
            .expect("read revoked Communication Explanation Storage binding")
            .expect("revoked Communication Explanation Storage binding")
            .state(),
        PlatformStorageBindingStateV1::Revoking
    );
    assert!(
        !supervisor
            .stop_if_active(&explanation.registration_id)
            .expect("observe stopped Communication Explanation workflow"),
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
            "Communication Explanation revoke must not stop {owner}"
        );
    }
    assert_eq!(
        post_proto_status(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            COMMUNICATION_EXPLANATION_QUERY_CONNECT_PATH_V1,
            GetCommunicationExplanationRequestV1 {
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
        .expect("join Communication Explanation owner control server")
        .expect("Communication Explanation owner control server");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Communication Explanation fixture");
    std::fs::remove_dir_all(data).expect("remove short Communication Explanation Kernel fixture");
}

#[test]
#[ignore = "requires disposable Docker plus a real loopback Ollama service with makosh-conformance:latest"]
fn managed_communication_explanation_completes_real_provider_through_gateway_sse() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let ollama_port = required("MAKOSH_OLLAMA_LIVE_PORT")
        .parse::<u16>()
        .expect("valid live Ollama port");
    let root = unique_target_root("makosh-managed-communication-explanation-live");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communication_explanation_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            COMMUNICATION_EXPLANATION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim live Communication Explanation logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        COMMUNICATION_EXPLANATION_LOGICAL_OWNER_ID_V1,
    );
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_explanation = admit_communication_explanation_runtime_v1(&store);
    let admitted_ollama = admit_ollama_ai_runtime_v1(&store);
    let admitted_ai = admit_ai_inference_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_ai_module_request_router_v1(&supervisor, &store);
    configure_communication_explanation_realtime_v1(&supervisor, &store, realtime.clone());
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
    let admitted_explanation =
        prepare_communication_explanation_runtime_v1(&supervisor, &store, admitted_explanation);
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
    let explanation = start_communication_explanation_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_explanation,
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
    let router =
        communication_explanation_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );

    let request = start_request([0x91; 16], source_message_id, 2);
    let accepted = post_proto::<_, StartCommunicationExplanationResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_EXPLANATION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(accepted.error, unspecified_error());
    let first_event =
        read_terminal_explanation_sse_event(&router, &gateway_runtime, &cookie, &accepted.run_id);
    let first_payload =
        CommunicationExplanationStatusChangedV1::decode(first_event.payload.as_slice())
            .expect("successful Communication Explanation realtime payload");
    assert_eq!(first_payload.run_id, accepted.run_id);
    assert_eq!(
        state(first_payload.state),
        CommunicationExplanationStateV1::CommunicationExplanationStateReady
    );
    let ready = wait_for_ready_explanation(&router, &gateway_runtime, &cookie, &accepted.run_id);
    assert_eq!(ready.source_message_id, source_message_id);
    assert_eq!(ready.expected_source_revision, 2);
    assert!(ready.state_revision >= 4);
    assert_eq!(ready.error, unspecified_error());
    let candidate = ready
        .candidate
        .as_ref()
        .expect("typed Communication Explanation candidate");
    assert!(candidate.reasons.len() <= 8);
    for reason in &candidate.reasons {
        assert!(!reason.explanation_utf8.is_empty());
        assert!(reason.explanation_utf8.len() <= 512);
        assert!(matches!(
            CommunicationExplanationReasonKindV1::try_from(reason.kind),
            Ok(kind)
                if kind
                    != CommunicationExplanationReasonKindV1::CommunicationExplanationReasonKindUnspecified
        ));
        assert!(matches!(
            CommunicationExplanationSourceBasisV1::try_from(reason.source_basis),
            Ok(source_basis)
                if source_basis
                    != CommunicationExplanationSourceBasisV1::CommunicationExplanationSourceBasisUnspecified
        ));
        assert!(reason.confidence_basis_points <= 10_000);
    }
    assert!(matches!(
        CommunicationExplanationCompletenessV1::try_from(candidate.completeness),
        Ok(CommunicationExplanationCompletenessV1::CommunicationExplanationCompletenessComplete)
            | Ok(
                CommunicationExplanationCompletenessV1::CommunicationExplanationCompletenessPartial
            )
    ));
    assert!(candidate.confidence_basis_points <= 10_000);

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
            .revoke_owner(COMMUNICATION_EXPLANATION_LOGICAL_OWNER_ID_V1)
            .expect("clear successful Communication Explanation Gateway replay cache")
    );
    let explanation = restart_communication_explanation_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        explanation,
    );
    let restarted_router =
        communication_explanation_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    assert_eq!(
        get_explanation(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            &accepted.run_id,
        ),
        ready
    );
    let replayed_event = read_terminal_explanation_sse_event(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed_event.cursor, first_event.cursor);
    assert_eq!(replayed_event.payload, first_event.payload);
    let duplicate = post_proto::<_, StartCommunicationExplanationResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        COMMUNICATION_EXPLANATION_COMMAND_CONNECT_PATH_V1,
        request,
    );
    assert_eq!(duplicate.run_id, accepted.run_id);
    assert_eq!(
        state(duplicate.state),
        CommunicationExplanationStateV1::CommunicationExplanationStateReady
    );
    assert_eq!(explanation.runtime_generation, 2);

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove live Communication Explanation fixture");
    std::fs::remove_dir_all(data)
        .expect("remove short live Communication Explanation Kernel fixture");
}

fn start_request(
    operation_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
) -> StartCommunicationExplanationRequestV1 {
    StartCommunicationExplanationRequestV1 {
        protocol_major: 1,
        operation_id: operation_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
    }
}

fn route_communication_explanation_as(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    logical_owner_id: &str,
    request_id: u64,
    request: StartCommunicationExplanationRequestV1,
) -> ModuleClientResponseV1 {
    let launch = store
        .effective_managed_launch_record(registration_id)
        .expect("read Communication Explanation launch")
        .expect("Communication Explanation launch is active");
    let request =
        encode_communication_explanation_module_request_as(logical_owner_id, request_id, request);
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        registration_id,
        launch.runtime_instance_id(),
        launch.runtime_generation(),
        launch.grant_epoch(),
        COMMUNICATION_EXPLANATION_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route Communication Explanation owner-fence request");
    ModuleClientResponseV1::decode(bytes.as_slice())
        .expect("decode Communication Explanation owner-fence response")
}

fn assert_communication_explanation_runtime_fences(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    explanation: &StartedCommunicationExplanationRuntimeV1,
    source_message_id: [u8; 16],
) {
    let request = encode_communication_explanation_module_request_as(
        COMMUNICATION_EXPLANATION_LOGICAL_OWNER_ID_V1,
        699,
        start_request([0x7f; 16], source_message_id, 2),
    );
    for (runtime_generation, grant_epoch, label) in [
        (
            explanation.runtime_generation + 1,
            explanation.grant_epoch,
            "stale Communication Explanation runtime generation",
        ),
        (
            explanation.runtime_generation,
            explanation.grant_epoch + 1,
            "stale Communication Explanation grant epoch",
        ),
    ] {
        let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
            &explanation.registration_id,
            &explanation.runtime_instance_id,
            runtime_generation,
            grant_epoch,
            COMMUNICATION_EXPLANATION_CAPABILITY_ID_V1,
            &request,
        );
        assert_eq!(
            crate::modules::capability::router::route_managed_client_request(
                store,
                &supervisor.relay_port(),
                &route,
            )
            .expect_err(label),
            "managed runtime fence is stale"
        );
    }
}

fn encode_communication_explanation_module_request_as(
    logical_owner_id: &str,
    request_id: u64,
    request: StartCommunicationExplanationRequestV1,
) -> Vec<u8> {
    ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: COMMUNICATION_EXPLANATION_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_EXPLANATION_OWNER_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: COMMUNICATION_EXPLANATION_OWNER_V1.to_owned(),
            name: COMMUNICATION_EXPLANATION_COMMAND_CONTRACT_NAME_V1.to_owned(),
            major: COMMUNICATION_EXPLANATION_CONTRACT_MAJOR_V1,
            revision: COMMUNICATION_EXPLANATION_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATION_EXPLANATION_SCHEMA_SHA256.to_vec(),
        }),
        request_id,
        request_payload: request.encode_to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec()
}

fn wait_for_terminal_explanation(
    router: &CommunicationExplanationGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
    expected_error: CommunicationExplanationErrorCodeV1,
) -> GetCommunicationExplanationResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(12);
    loop {
        let response = get_explanation(router, runtime, cookie, run_id);
        if state(response.state)
            == CommunicationExplanationStateV1::CommunicationExplanationStateRejected
        {
            assert_eq!(error(response.error), expected_error);
            return response;
        }
        assert!(
            Instant::now() < deadline,
            "Communication Explanation did not reach the expected terminal state: {response:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_ready_explanation(
    router: &CommunicationExplanationGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetCommunicationExplanationResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let response = get_explanation(router, runtime, cookie, run_id);
        match state(response.state) {
            CommunicationExplanationStateV1::CommunicationExplanationStateReady => return response,
            CommunicationExplanationStateV1::CommunicationExplanationStateRejected => {
                panic!("live Communication Explanation was rejected: {response:?}")
            }
            _ => {}
        }
        assert!(
            Instant::now() < deadline,
            "Communication Explanation did not reach Ready: {response:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn get_explanation(
    router: &CommunicationExplanationGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetCommunicationExplanationResponseV1 {
    post_proto(
        router,
        runtime,
        cookie,
        COMMUNICATION_EXPLANATION_QUERY_CONNECT_PATH_V1,
        GetCommunicationExplanationRequestV1 {
            protocol_major: 1,
            run_id: run_id.to_vec(),
        },
    )
}

fn communication_explanation_gateway(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> CommunicationExplanationGateway {
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("communication-explanation-gateway-cert.der"),
        root.join("communication-explanation-gateway-key.der"),
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
    .expect("compose Communication Explanation Gateway routes")
}

fn post_proto<M, R>(
    router: &CommunicationExplanationGateway,
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
                    .expect("Communication Explanation Gateway request"),
            ),
        );
        let status = response.status();
        let bytes = runtime
            .block_on(response.into_body().collect())
            .expect("Communication Explanation Gateway response")
            .to_bytes();
        if status == StatusCode::SERVICE_UNAVAILABLE && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
            continue;
        }
        assert_eq!(
            status,
            StatusCode::OK,
            "Communication Explanation Gateway response body: {}",
            String::from_utf8_lossy(&bytes)
        );
        return R::decode(bytes.as_ref())
            .expect("decode Communication Explanation Gateway response");
    }
}

fn post_proto_status<M>(
    router: &CommunicationExplanationGateway,
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
                    .expect("Communication Explanation Gateway status request"),
            ),
        )
        .status()
}

fn read_terminal_explanation_sse_event(
    router: &CommunicationExplanationGateway,
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
                .expect("Communication Explanation Gateway SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(30),
            find_terminal_explanation_event(response.into_body(), run_id),
        )
        .await
        .expect("Communication Explanation SSE timeout")
    })
}

async fn find_terminal_explanation_event<B>(mut body: B, run_id: &[u8]) -> ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Communication Explanation SSE frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Communication Explanation SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Communication Explanation frame");
            let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                .expect("Communication Explanation realtime frame");
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            if event.contract_name != COMMUNICATION_EXPLANATION_REALTIME_CONTRACT_NAME_V1
                || event.event_kind != COMMUNICATION_EXPLANATION_REALTIME_EVENT_KIND_V1
            {
                continue;
            }
            let payload = CommunicationExplanationStatusChangedV1::decode(event.payload.as_slice())
                .expect("Communication Explanation realtime payload");
            if payload.run_id == run_id
                && matches!(
                    state(payload.state),
                    CommunicationExplanationStateV1::CommunicationExplanationStateReady
                        | CommunicationExplanationStateV1::CommunicationExplanationStateRejected
                )
            {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before terminal Communication Explanation event");
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
            "persisted or source-rejected Communication Explanation retried Ollama HTTP"
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

fn state(value: i32) -> CommunicationExplanationStateV1 {
    CommunicationExplanationStateV1::try_from(value).expect("known Communication Explanation state")
}

fn error(value: i32) -> CommunicationExplanationErrorCodeV1 {
    CommunicationExplanationErrorCodeV1::try_from(value)
        .expect("known Communication Explanation error")
}

fn unspecified_error() -> i32 {
    CommunicationExplanationErrorCodeV1::CommunicationExplanationErrorCodeUnspecified as i32
}
