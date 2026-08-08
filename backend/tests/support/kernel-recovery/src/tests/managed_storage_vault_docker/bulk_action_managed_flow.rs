//! Live bulk-action publication through the owner-neutral Kernel and Gateway SSE.

use super::*;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_communication_bulk_action_api::{
    COMMUNICATION_BULK_ACTION_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_BULK_ACTION_QUERY_CONNECT_PATH_V1,
    COMMUNICATION_BULK_ACTION_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_BULK_ACTION_REALTIME_EVENT_KIND_V1,
    wire::{
        BulkDeliveryBatchStateV1, BulkDeliveryErrorCodeV1, BulkDeliveryStatusChangedV1,
        BulkDeliveryTargetStateV1, BulkDeliveryTargetV1, GetBulkDeliveryStatusRequestV1,
        GetBulkDeliveryStatusResponseV1, StartBulkDeliveryRequestV1, StartBulkDeliveryResponseV1,
    },
};
use makosh_gateway_protocol::v1::{
    ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};

type BulkActionGateway = makosh_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications and bulk-action binaries"]
fn managed_bulk_action_reaches_gateway_sse_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-bulk-action-realtime");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communications_bulk_action_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            BULK_ACTION_LOGICAL_OWNER_ID,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim logical browser owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        BULK_ACTION_LOGICAL_OWNER_ID,
    );
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_delivery_intent = admit_delivery_intent_runtime(&store);
    let admitted_bulk_action = admit_bulk_action_runtime(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(32).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_delivery_intent_runtime_routes(&supervisor, &store, realtime.clone());
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    blob_launch::start_from_kernel(
        &supervisor,
        &store,
        release.kernel(),
        &data,
        &root.join("runtime"),
    )
    .expect("start signed Blob runtime");
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
    let admitted_delivery_intent =
        prepare_delivery_intent_runtime(&supervisor, &store, admitted_delivery_intent);
    let admitted_bulk_action =
        prepare_bulk_action_runtime(&supervisor, &store, admitted_bulk_action);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let message_id = assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        false,
    );
    let conversation_id = canonical_conversation_for_message(&store, &supervisor, &message_id);
    let _delivery_intent = start_delivery_intent_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_delivery_intent,
    );
    let bulk_action = start_bulk_action_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_bulk_action,
    );

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = bulk_action_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let batch_id = vec![0x51; 16];
    let target_operation_id = vec![0x52; 16];
    let private_body = b"managed delivery body must never enter realtime";
    let response = gateway_runtime.block_on(
        router.route(
            Request::builder()
                .method("POST")
                .uri(COMMUNICATION_BULK_ACTION_COMMAND_CONNECT_PATH_V1)
                .header("content-type", "application/connect+proto")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(Bytes::from(
                    StartBulkDeliveryRequestV1 {
                        protocol_major: 1,
                        batch_operation_id: batch_id.clone(),
                        targets: vec![BulkDeliveryTargetV1 {
                            target_operation_id: target_operation_id.clone(),
                            conversation_id: conversation_id.clone(),
                            reply_to_message_id: None,
                            body_utf8: private_body.to_vec(),
                        }],
                    }
                    .encode_to_vec(),
                )))
                .expect("bulk-action Gateway request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    let response_bytes = gateway_runtime
        .block_on(response.into_body().collect())
        .expect("bulk-action Gateway response")
        .to_bytes();
    let response = StartBulkDeliveryResponseV1::decode(response_bytes.as_ref())
        .expect("decode bulk-action response");
    assert_eq!(response.batch_id, batch_id);
    assert_eq!(response.target_count, 1);
    assert_eq!(
        response.error,
        BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeUnspecified as i32
    );
    assert_eq!(
        response.state,
        BulkDeliveryBatchStateV1::BulkDeliveryBatchStateAccepted as i32
    );

    let first = read_bulk_action_sse_event(
        &router,
        &gateway_runtime,
        &cookie,
        None,
        BulkDeliveryBatchStateV1::BulkDeliveryBatchStateAccepted,
    );
    assert_client_safe_event(
        &first,
        &batch_id,
        BulkDeliveryBatchStateV1::BulkDeliveryBatchStateAccepted,
        private_body,
    );
    let cursor = first.cursor.clone();
    let terminal_event = read_bulk_action_sse_event(
        &router,
        &gateway_runtime,
        &cookie,
        Some(&cursor),
        BulkDeliveryBatchStateV1::BulkDeliveryBatchStateCompleted,
    );
    assert_client_safe_event(
        &terminal_event,
        &batch_id,
        BulkDeliveryBatchStateV1::BulkDeliveryBatchStateCompleted,
        private_body,
    );
    let terminal = read_bulk_status(&router, &gateway_runtime, &cookie, &batch_id);
    assert_eq!(
        terminal.state,
        BulkDeliveryBatchStateV1::BulkDeliveryBatchStateCompleted as i32
    );
    assert_terminal_target(&terminal, &target_operation_id);

    assert!(
        realtime
            .revoke_owner(BULK_ACTION_LOGICAL_OWNER_ID)
            .expect("clear Gateway delivery cache"),
        "the first managed publication must admit the logical owner"
    );
    let previous_generation = bulk_action.runtime_generation;
    let bulk_action =
        restart_bulk_action_runtime(&supervisor, &store, &root.join("runtime"), bulk_action);
    assert_eq!(bulk_action.runtime_generation, previous_generation + 1);
    let restarted_router = bulk_action_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed = read_bulk_action_sse_event(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        None,
        BulkDeliveryBatchStateV1::BulkDeliveryBatchStateAccepted,
    );
    assert_client_safe_event(
        &replayed,
        &batch_id,
        BulkDeliveryBatchStateV1::BulkDeliveryBatchStateAccepted,
        private_body,
    );
    assert_eq!(
        replayed.cursor, cursor,
        "runtime restart must reconstruct the exact stable owner cursor"
    );
    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove kernel fixture");
}

fn canonical_conversation_for_message(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    message_id: &[u8],
) -> Vec<u8> {
    use makosh_communications_api::query_wire::{
        CommunicationsQueryRequestV1, GetMessageRequestV1,
        communications_query_request_v1::Operation, communications_query_response_v1::Result,
    };

    let response = route_communications_query(
        store,
        supervisor,
        70,
        &CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(Operation::GetMessage(GetMessageRequestV1 {
                message_id: message_id.to_vec(),
            })),
        }
        .encode_to_vec(),
    );
    let Some(Result::GetMessage(result)) = response.result else {
        panic!("managed Communications message query result");
    };
    let conversation_id = result
        .message
        .expect("managed Communications message")
        .conversation_id;
    let response = route_communications_query(
        store,
        supervisor,
        71,
        &CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(Operation::GetConversation(
                makosh_communications_api::query_wire::GetConversationRequestV1 {
                    conversation_id: conversation_id.clone(),
                },
            )),
        }
        .encode_to_vec(),
    );
    assert!(
        matches!(response.result, Some(Result::GetConversation(_))),
        "managed Communications conversation query must resolve the delivery route source"
    );
    conversation_id
}

fn bulk_action_gateway(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> BulkActionGateway {
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("bulk-action-gateway-cert.der"),
        root.join("bulk-action-gateway-key.der"),
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
    .expect("compose owner Gateway routes")
}

fn read_bulk_action_sse_event(
    router: &BulkActionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    last_event_id: Option<&str>,
    expected_state: BulkDeliveryBatchStateV1,
) -> makosh_gateway_protocol::v1::ClientRealtimeEventV1 {
    let mut request = Request::builder()
        .method("GET")
        .uri("/api/realtime/v1/events")
        .header("cookie", cookie);
    if let Some(last_event_id) = last_event_id {
        request = request.header("last-event-id", last_event_id);
    }
    let response = runtime.block_on(
        router.route(
            request
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Gateway SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );
    runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(8),
            find_bulk_action_event(response.into_body(), expected_state),
        )
        .await
        .expect("bulk-action SSE event timeout")
    })
}

async fn find_bulk_action_event<B>(
    mut body: B,
    expected_state: BulkDeliveryBatchStateV1,
) -> makosh_gateway_protocol::v1::ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Gateway SSE body frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Gateway SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Gateway realtime frame");
            let frame =
                ClientRealtimeFrameV1::decode(bytes.as_slice()).expect("decode realtime frame");
            if let Some(RealtimeFrame::Event(event)) = frame.frame
                && event.contract_name == COMMUNICATION_BULK_ACTION_REALTIME_CONTRACT_NAME_V1
                && BulkDeliveryStatusChangedV1::decode(event.payload.as_slice())
                    .is_ok_and(|payload| payload.state == expected_state as i32)
            {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before bulk-action event");
}

fn assert_client_safe_event(
    event: &makosh_gateway_protocol::v1::ClientRealtimeEventV1,
    batch_id: &[u8],
    expected_state: BulkDeliveryBatchStateV1,
    private_body: &[u8],
) {
    assert_eq!(
        event.contract_name,
        COMMUNICATION_BULK_ACTION_REALTIME_CONTRACT_NAME_V1
    );
    assert_eq!(
        event.event_kind,
        COMMUNICATION_BULK_ACTION_REALTIME_EVENT_KIND_V1
    );
    assert!(
        !event
            .encode_to_vec()
            .windows(private_body.len())
            .any(|window| window == private_body),
        "client realtime frame must not contain private delivery content"
    );
    let payload = BulkDeliveryStatusChangedV1::decode(event.payload.as_slice())
        .expect("decode client-safe bulk-action event");
    assert_eq!(payload.batch_id, batch_id);
    assert_eq!(payload.state, expected_state as i32);
    assert!(payload.state_revision > 0);
    assert!(payload.occurred_at_unix_millis > 0);
}

fn assert_terminal_target(response: &GetBulkDeliveryStatusResponseV1, target_operation_id: &[u8]) {
    assert_eq!(response.targets.len(), 1);
    assert_eq!(response.targets[0].target_operation_id, target_operation_id);
    assert_eq!(
        response.targets[0].state,
        BulkDeliveryTargetStateV1::BulkDeliveryTargetStateAccepted as i32
    );
    assert_eq!(
        response.targets[0].error,
        BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeUnspecified as i32
    );
    assert!(
        response.targets[0]
            .delivery_intent_id
            .as_ref()
            .is_some_and(|intent_id| intent_id.len() == 16)
    );
}

fn read_bulk_status(
    router: &BulkActionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    batch_id: &[u8],
) -> GetBulkDeliveryStatusResponseV1 {
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("POST")
                .uri(COMMUNICATION_BULK_ACTION_QUERY_CONNECT_PATH_V1)
                .header("content-type", "application/connect+proto")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(Bytes::from(
                    GetBulkDeliveryStatusRequestV1 {
                        protocol_major: 1,
                        batch_id: batch_id.to_vec(),
                        limit: 100,
                        cursor: Vec::new(),
                    }
                    .encode_to_vec(),
                )))
                .expect("bulk-action status Gateway request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = runtime
        .block_on(response.into_body().collect())
        .expect("bulk-action status Gateway response")
        .to_bytes();
    let response = GetBulkDeliveryStatusResponseV1::decode(bytes.as_ref())
        .expect("decode bulk-action status response");
    assert_eq!(
        response.error,
        BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeUnspecified as i32
    );
    response
}
