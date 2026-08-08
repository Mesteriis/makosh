//! Full managed Recipient Suggestion path through Communications events, Gateway and SSE.

use std::time::{Duration, Instant};

use super::*;

use crate::identity::device::signer::DeviceSigner;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_communication_recipient_suggestion_api::{
    COMMUNICATION_RECIPIENT_SUGGESTION_CAPABILITY_ID_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_MAJOR_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_REVISION_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_MODULE_ID_V1, COMMUNICATION_RECIPIENT_SUGGESTION_OWNER_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_QUERY_CONNECT_PATH_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_REALTIME_EVENT_KIND_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_SCHEMA_SHA256,
    wire::{
        CommunicationRecipientRoleV1, CommunicationRecipientSuggestionErrorCodeV1,
        CommunicationRecipientSuggestionStateV1, CommunicationRecipientSuggestionStatusChangedV1,
        GetCommunicationRecipientSuggestionRequestV1,
        GetCommunicationRecipientSuggestionResponseV1,
        StartCommunicationRecipientSuggestionRequestV1,
        StartCommunicationRecipientSuggestionResponseV1,
    },
};
use makosh_communication_recipient_suggestion_runtime::COMMUNICATION_RECIPIENT_SUGGESTION_STORAGE_CAPABILITY_ID_V1;
use makosh_gateway_protocol::v1::{
    ClientRealtimeEventV1, ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};
use makosh_kernel_control_store::{ModuleRegistrationState, PlatformStorageBindingStateV1};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};

const COMBINED_SOURCE_BODY: &[u8] =
    b"Invoice payment and contract review for the project status update";

type RecipientSuggestionGateway = makosh_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications and Recipient Suggestion binaries"]
fn managed_recipient_suggestion_reaches_gateway_sse_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-recipient-suggestion");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communication_recipient_suggestion_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            COMMUNICATION_RECIPIENT_SUGGESTION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Recipient Suggestion logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        COMMUNICATION_RECIPIENT_SUGGESTION_LOGICAL_OWNER_ID_V1,
    );
    let admitted = admit_communication_recipient_suggestion_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_communication_recipient_suggestion_realtime_v1(&supervisor, &store, realtime.clone());
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
    let admitted =
        prepare_communication_recipient_suggestion_runtime_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let workflow = start_communication_recipient_suggestion_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted,
    );
    assert_eq!(workflow.runtime_generation, 1);

    let source_message_id = assert_communications_transferred_body_projection_with_plaintext(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        COMBINED_SOURCE_BODY,
        false,
    );
    let source_message_id: [u8; 16] = source_message_id
        .try_into()
        .expect("canonical source message ID");
    assert_runtime_fences(&store, &supervisor, &workflow, source_message_id);
    let wrong_owner = route_start_as(
        &store,
        &supervisor,
        &workflow.registration_id,
        "owner-2",
        700,
        start_request([0x80; 16], source_message_id, 2),
    );
    assert_eq!(wrong_owner.request_id, 700);
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = recipient_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let request = start_request([0x81; 16], source_message_id, 2);
    let accepted = post_proto::<_, StartCommunicationRecipientSuggestionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(error(accepted.error), unspecified_error());
    let ready = wait_for_ready(&router, &gateway_runtime, &cookie, &accepted.run_id);
    assert_eq!(state(ready.state), ready_state());
    assert_eq!(ready.source_message_id, source_message_id);
    assert_eq!(ready.expected_source_revision, 2);
    assert_eq!(ready.candidates.len(), 3);
    assert_eq!(
        ready
            .candidates
            .iter()
            .map(|candidate| CommunicationRecipientRoleV1::try_from(candidate.role).expect("role"))
            .collect::<Vec<_>>(),
        [
            CommunicationRecipientRoleV1::CommunicationRecipientRoleAccountingOrBookkeeping,
            CommunicationRecipientRoleV1::CommunicationRecipientRoleLegalCounsel,
            CommunicationRecipientRoleV1::CommunicationRecipientRoleProjectStakeholder,
        ]
    );
    let first_event = read_terminal_sse_event(&router, &gateway_runtime, &cookie, &accepted.run_id);
    assert_private_content_absent(&first_event.encode_to_vec(), source_message_id);
    let first_cursor = first_event.cursor.clone();

    let duplicate = post_proto::<_, StartCommunicationRecipientSuggestionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(duplicate.run_id, accepted.run_id);
    assert_eq!(state(duplicate.state), ready_state());
    let mut conflicting = request;
    conflicting.expected_source_revision += 1;
    let conflicting = post_proto::<_, StartCommunicationRecipientSuggestionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONNECT_PATH_V1,
        conflicting,
    );
    assert_eq!(
        error(conflicting.error),
        CommunicationRecipientSuggestionErrorCodeV1::CommunicationRecipientSuggestionErrorCodeInvalidRequest
    );

    assert!(
        realtime
            .revoke_owner(COMMUNICATION_RECIPIENT_SUGGESTION_LOGICAL_OWNER_ID_V1)
            .expect("clear Recipient Suggestion replay cache")
    );
    let previous_generation = workflow.runtime_generation;
    let workflow = restart_communication_recipient_suggestion_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        workflow,
    );
    assert_eq!(workflow.runtime_generation, previous_generation + 1);
    let restarted_router = recipient_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed = get_status(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed, ready);
    let replayed_event = read_terminal_sse_event(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed_event.cursor, first_cursor);
    assert_eq!(replayed_event.payload, first_event.payload);

    let stale = post_proto::<_, StartCommunicationRecipientSuggestionResponseV1>(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONNECT_PATH_V1,
        start_request([0x82; 16], source_message_id, 1),
    );
    let stale = wait_for_rejected(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &stale.run_id,
    );
    assert_eq!(
        error(stale.error),
        CommunicationRecipientSuggestionErrorCodeV1::CommunicationRecipientSuggestionErrorCodeSourceRejected
    );

    let (owner_runtime_dir, owner_control) =
        start_owner_control(&data, &store, &shutdown, &supervisor);
    let revoked = transition_registration(
        &owner_runtime_dir,
        &owner_signer,
        &workflow.registration_id,
        "revoked",
    );
    assert_eq!(revoked.state, "revoked");
    assert!(revoked.grant_epoch > workflow.grant_epoch);
    assert_eq!(
        store
            .module_registration(&workflow.registration_id)
            .expect("read revoked registration")
            .expect("revoked registration")
            .state(),
        ModuleRegistrationState::Revoked
    );
    assert_eq!(
        store
            .platform_storage_binding(
                &workflow.registration_id,
                COMMUNICATION_RECIPIENT_SUGGESTION_STORAGE_CAPABILITY_ID_V1,
            )
            .expect("read revoked Storage binding")
            .expect("revoked Storage binding")
            .state(),
        PlatformStorageBindingStateV1::Revoking
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("Communications active")
    );
    assert_eq!(
        post_proto_status(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            COMMUNICATION_RECIPIENT_SUGGESTION_QUERY_CONNECT_PATH_V1,
            GetCommunicationRecipientSuggestionRequestV1 {
                protocol_major: 1,
                run_id: accepted.run_id,
            },
        ),
        StatusCode::NOT_FOUND
    );

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    owner_control
        .join()
        .expect("join owner control server")
        .expect("owner control server");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Recipient Suggestion fixture");
    std::fs::remove_dir_all(data).expect("remove short Kernel fixture");
}

fn start_request(
    operation_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
) -> StartCommunicationRecipientSuggestionRequestV1 {
    StartCommunicationRecipientSuggestionRequestV1 {
        protocol_major: 1,
        operation_id: operation_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
    }
}

fn route_start_as(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    logical_owner_id: &str,
    request_id: u64,
    request: StartCommunicationRecipientSuggestionRequestV1,
) -> ModuleClientResponseV1 {
    let launch = store
        .effective_managed_launch_record(registration_id)
        .expect("read Recipient Suggestion launch")
        .expect("Recipient Suggestion launch is active");
    let request = encode_module_request(logical_owner_id, request_id, request);
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        registration_id,
        launch.runtime_instance_id(),
        launch.runtime_generation(),
        launch.grant_epoch(),
        COMMUNICATION_RECIPIENT_SUGGESTION_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route Recipient Suggestion owner-fence request");
    ModuleClientResponseV1::decode(bytes.as_slice()).expect("decode owner-fence response")
}

fn assert_runtime_fences(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    workflow: &StartedCommunicationRecipientSuggestionRuntimeV1,
    source_message_id: [u8; 16],
) {
    let request = encode_module_request(
        COMMUNICATION_RECIPIENT_SUGGESTION_LOGICAL_OWNER_ID_V1,
        699,
        start_request([0x7f; 16], source_message_id, 2),
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
            COMMUNICATION_RECIPIENT_SUGGESTION_CAPABILITY_ID_V1,
            &request,
        );
        assert_eq!(
            crate::modules::capability::router::route_managed_client_request(
                store,
                &supervisor.relay_port(),
                &route,
            )
            .expect_err("stale workflow fence"),
            "managed runtime fence is stale"
        );
    }
}

fn encode_module_request(
    logical_owner_id: &str,
    request_id: u64,
    request: StartCommunicationRecipientSuggestionRequestV1,
) -> Vec<u8> {
    ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: COMMUNICATION_RECIPIENT_SUGGESTION_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_RECIPIENT_SUGGESTION_OWNER_V1.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: COMMUNICATION_RECIPIENT_SUGGESTION_OWNER_V1.to_owned(),
            name: COMMUNICATION_RECIPIENT_SUGGESTION_COMMAND_CONTRACT_NAME_V1.to_owned(),
            major: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_MAJOR_V1,
            revision: COMMUNICATION_RECIPIENT_SUGGESTION_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATION_RECIPIENT_SUGGESTION_SCHEMA_SHA256.to_vec(),
        }),
        request_id,
        request_payload: request.encode_to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec()
}

fn wait_for_ready(
    router: &RecipientSuggestionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetCommunicationRecipientSuggestionResponseV1 {
    wait_for_terminal(router, runtime, cookie, run_id, ready_state())
}

fn wait_for_rejected(
    router: &RecipientSuggestionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetCommunicationRecipientSuggestionResponseV1 {
    wait_for_terminal(
        router,
        runtime,
        cookie,
        run_id,
        run_state(
            CommunicationRecipientSuggestionStateV1::CommunicationRecipientSuggestionStateRejected,
        ),
    )
}

fn wait_for_terminal(
    router: &RecipientSuggestionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
    expected: CommunicationRecipientSuggestionStateV1,
) -> GetCommunicationRecipientSuggestionResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let response = get_status(router, runtime, cookie, run_id);
        let current = state(response.state);
        if current == expected {
            return response;
        }
        assert_ne!(current, run_state(CommunicationRecipientSuggestionStateV1::CommunicationRecipientSuggestionStateRejected), "unexpected rejection: {response:?}");
        assert!(
            Instant::now() < deadline,
            "workflow did not reach terminal state: {response:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn get_status(
    router: &RecipientSuggestionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetCommunicationRecipientSuggestionResponseV1 {
    post_proto(
        router,
        runtime,
        cookie,
        COMMUNICATION_RECIPIENT_SUGGESTION_QUERY_CONNECT_PATH_V1,
        GetCommunicationRecipientSuggestionRequestV1 {
            protocol_major: 1,
            run_id: run_id.to_vec(),
        },
    )
}

fn recipient_gateway(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> RecipientSuggestionGateway {
    super::delivery_intent_realtime_flow::delivery_intent_gateway(
        store, supervisor, root, data, realtime,
    )
}

fn post_proto<M, R>(
    router: &RecipientSuggestionGateway,
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
                    .expect("Gateway request"),
            ),
        );
        let status = response.status();
        let bytes = runtime
            .block_on(response.into_body().collect())
            .expect("Gateway response")
            .to_bytes();
        if status == StatusCode::SERVICE_UNAVAILABLE && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
            continue;
        }
        assert_eq!(
            status,
            StatusCode::OK,
            "Gateway response: {}",
            String::from_utf8_lossy(&bytes)
        );
        return R::decode(bytes.as_ref()).expect("decode Gateway response");
    }
}

fn post_proto_status<M>(
    router: &RecipientSuggestionGateway,
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
                    .expect("Gateway status request"),
            ),
        )
        .status()
}

fn read_terminal_sse_event(
    router: &RecipientSuggestionGateway,
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
                .expect("Gateway SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(8),
            find_terminal_event(response.into_body(), run_id),
        )
        .await
        .expect("Recipient Suggestion SSE timeout")
    })
}

async fn find_terminal_event<B>(mut body: B, run_id: &[u8]) -> ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("SSE frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD.decode(encoded).expect("decode SSE frame");
            let frame = ClientRealtimeFrameV1::decode(bytes.as_slice()).expect("realtime frame");
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            if event.contract_name != COMMUNICATION_RECIPIENT_SUGGESTION_REALTIME_CONTRACT_NAME_V1
                || event.event_kind != COMMUNICATION_RECIPIENT_SUGGESTION_REALTIME_EVENT_KIND_V1
            {
                continue;
            }
            let payload =
                CommunicationRecipientSuggestionStatusChangedV1::decode(event.payload.as_slice())
                    .expect("status payload");
            if payload.run_id == run_id
                && matches!(state(payload.state), CommunicationRecipientSuggestionStateV1::CommunicationRecipientSuggestionStateReady | CommunicationRecipientSuggestionStateV1::CommunicationRecipientSuggestionStateRejected)
            {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before terminal event");
}

fn assert_private_content_absent(bytes: &[u8], source_message_id: [u8; 16]) {
    assert!(
        !bytes
            .windows(COMBINED_SOURCE_BODY.len())
            .any(|window| window == COMBINED_SOURCE_BODY)
    );
    assert!(
        !bytes
            .windows(source_message_id.len())
            .any(|window| window == source_message_id)
    );
}

fn state(value: i32) -> CommunicationRecipientSuggestionStateV1 {
    CommunicationRecipientSuggestionStateV1::try_from(value).expect("state")
}

const fn run_state(
    value: CommunicationRecipientSuggestionStateV1,
) -> CommunicationRecipientSuggestionStateV1 {
    value
}

const fn ready_state() -> CommunicationRecipientSuggestionStateV1 {
    CommunicationRecipientSuggestionStateV1::CommunicationRecipientSuggestionStateReady
}

fn error(value: i32) -> CommunicationRecipientSuggestionErrorCodeV1 {
    CommunicationRecipientSuggestionErrorCodeV1::try_from(value).expect("error")
}

const fn unspecified_error() -> CommunicationRecipientSuggestionErrorCodeV1 {
    CommunicationRecipientSuggestionErrorCodeV1::CommunicationRecipientSuggestionErrorCodeUnspecified
}
