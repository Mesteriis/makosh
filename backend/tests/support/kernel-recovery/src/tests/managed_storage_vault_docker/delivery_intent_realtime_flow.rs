//! Live delivery-intent publication through the owner-neutral Kernel and Gateway SSE.

use super::*;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_DELIVERY_INTENT_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_REALTIME_EVENT_KIND_V1,
    wire::{
        DeliveryIntentErrorCodeV1, DeliveryIntentStatusChangedV1, DeliveryIntentStatusV1,
        SubmitDeliveryIntentRequestV1, SubmitDeliveryIntentResponseV1,
    },
};
use makosh_gateway_protocol::v1::{
    ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};

pub(super) type DeliveryIntentGateway = makosh_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications and delivery-intent binaries"]
fn managed_delivery_intent_reaches_gateway_sse_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-delivery-intent-realtime");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communications_delivery_intent_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            DELIVERY_INTENT_LOGICAL_OWNER_ID,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim logical browser owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        DELIVERY_INTENT_LOGICAL_OWNER_ID,
    );
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted = admit_delivery_intent_runtime(&store);
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
    let admitted = prepare_delivery_intent_runtime(&supervisor, &store, admitted);
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
    let delivery_intent =
        start_delivery_intent_runtime(&supervisor, &store, &root.join("runtime"), admitted);

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = delivery_intent_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let intent_id = vec![0x51; 16];
    let private_body = b"managed delivery body must never enter realtime";
    let response = gateway_runtime.block_on(
        router.route(
            Request::builder()
                .method("POST")
                .uri(COMMUNICATION_DELIVERY_INTENT_COMMAND_CONNECT_PATH_V1)
                .header("content-type", "application/connect+proto")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(Bytes::from(
                    SubmitDeliveryIntentRequestV1 {
                        protocol_major: 1,
                        operation_id: intent_id.clone(),
                        conversation_id: conversation_id.clone(),
                        reply_to_message_id: None,
                        body_utf8: private_body.to_vec(),
                    }
                    .encode_to_vec(),
                )))
                .expect("delivery-intent Gateway request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    let response_bytes = gateway_runtime
        .block_on(response.into_body().collect())
        .expect("delivery-intent Gateway response")
        .to_bytes();
    let response = SubmitDeliveryIntentResponseV1::decode(response_bytes.as_ref())
        .expect("decode delivery-intent response");
    assert_eq!(response.intent_id, intent_id);
    assert_eq!(
        response.error,
        DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnspecified as i32
    );
    assert_eq!(
        response.status,
        DeliveryIntentStatusV1::DeliveryIntentStatusAccepted as i32
    );

    let first = read_delivery_intent_sse_event(&router, &gateway_runtime, &cookie);
    assert_client_safe_event(&first, &intent_id, private_body);
    let cursor = first.cursor.clone();

    assert!(
        realtime
            .revoke_owner(DELIVERY_INTENT_LOGICAL_OWNER_ID)
            .expect("clear Gateway delivery cache"),
        "the first managed publication must admit the logical owner"
    );
    let previous_generation = delivery_intent.runtime_generation;
    let delivery_intent = restart_delivery_intent_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        delivery_intent,
    );
    assert_eq!(delivery_intent.runtime_generation, previous_generation + 1);
    let restarted_router =
        delivery_intent_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed =
        read_delivery_intent_sse_event(&restarted_router, &gateway_runtime, &restarted_cookie);
    assert_client_safe_event(&replayed, &intent_id, private_body);
    assert_eq!(
        replayed.cursor, cursor,
        "runtime restart must reconstruct the exact stable owner cursor"
    );
    super::delivery_intent_module_request_flow::assert_live_delivery_intent_module_request(
        &store,
        &supervisor,
        conversation_id,
    );

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove kernel fixture");
}

pub(super) fn canonical_conversation_for_message(
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

pub(super) fn delivery_intent_gateway(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> DeliveryIntentGateway {
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("delivery-intent-gateway-cert.der"),
        root.join("delivery-intent-gateway-key.der"),
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

fn read_delivery_intent_sse_event(
    router: &DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
) -> makosh_gateway_protocol::v1::ClientRealtimeEventV1 {
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
            find_delivery_intent_event(response.into_body()),
        )
        .await
        .expect("delivery-intent SSE event timeout")
    })
}

async fn find_delivery_intent_event<B>(
    mut body: B,
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
                && event.contract_name == COMMUNICATION_DELIVERY_INTENT_REALTIME_CONTRACT_NAME_V1
            {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before delivery-intent event");
}

fn assert_client_safe_event(
    event: &makosh_gateway_protocol::v1::ClientRealtimeEventV1,
    intent_id: &[u8],
    private_body: &[u8],
) {
    assert_eq!(
        event.contract_name,
        COMMUNICATION_DELIVERY_INTENT_REALTIME_CONTRACT_NAME_V1
    );
    assert_eq!(
        event.event_kind,
        COMMUNICATION_DELIVERY_INTENT_REALTIME_EVENT_KIND_V1
    );
    assert!(
        !event
            .encode_to_vec()
            .windows(private_body.len())
            .any(|window| window == private_body),
        "client realtime frame must not contain private delivery content"
    );
    let payload = DeliveryIntentStatusChangedV1::decode(event.payload.as_slice())
        .expect("decode client-safe delivery-intent event");
    assert_eq!(payload.intent_id, intent_id);
    assert_eq!(
        payload.status,
        DeliveryIntentStatusV1::DeliveryIntentStatusAccepted as i32
    );
    assert_eq!(
        payload.rejection,
        DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnspecified as i32
    );
    assert!(payload.state_revision > 0);
    assert!(payload.occurred_at_unix_millis > 0);
}
