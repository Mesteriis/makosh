use super::*;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt;
use hyper::{Request, StatusCode, body::Bytes, service::Service};
use makosh_communication_cross_channel_forward_api::{
    COMMUNICATION_CROSS_CHANNEL_FORWARD_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_QUERY_CONNECT_PATH_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_REALTIME_CONTRACT_NAME_V1,
    wire::{
        CrossChannelForwardErrorCodeV1, CrossChannelForwardStateV1,
        CrossChannelForwardStatusChangedV1, GetCrossChannelForwardStatusRequestV1,
        GetCrossChannelForwardStatusResponseV1, StartCrossChannelForwardRequestV1,
        StartCrossChannelForwardResponseV1,
    },
};
use makosh_gateway_protocol::v1::{
    ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};
use prost::Message;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use zeroize::Zeroizing;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, delivery-intent and cross-channel-forward binaries"]
fn managed_cross_channel_forward_reaches_delivery_intent_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-cross-channel-forward");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_cross_channel_forward_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            CROSS_CHANNEL_FORWARD_LOGICAL_OWNER_ID,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim logical browser owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        CROSS_CHANNEL_FORWARD_LOGICAL_OWNER_ID,
    );
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let delivery_intent = admit_delivery_intent_runtime(&store);
    let cross_channel_forward = admit_cross_channel_forward_runtime(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
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
    let delivery_intent = prepare_delivery_intent_runtime(&supervisor, &store, delivery_intent);
    let cross_channel_forward =
        prepare_cross_channel_forward_runtime(&supervisor, &store, cross_channel_forward);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    assert_communications_ingress_delivery(&store, &supervisor);
    let target_conversation_id = managed_mail_target_conversation(&store, &supervisor);
    let source_message_id = assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        false,
    );
    let _delivery_intent =
        start_delivery_intent_runtime(&supervisor, &store, &root.join("runtime"), delivery_intent);
    let cross_channel_forward = start_cross_channel_forward_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        cross_channel_forward,
    );

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = super::delivery_intent_realtime_flow::delivery_intent_gateway(
        &store,
        &supervisor,
        &root,
        &data,
        realtime.clone(),
    );
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let forward_id = vec![0x71; 16];
    let response = gateway_runtime.block_on(
        router.route(
            Request::builder()
                .method("POST")
                .uri(COMMUNICATION_CROSS_CHANNEL_FORWARD_COMMAND_CONNECT_PATH_V1)
                .header("content-type", "application/connect+proto")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(Bytes::from(
                    StartCrossChannelForwardRequestV1 {
                        protocol_major: 1,
                        forward_operation_id: forward_id.clone(),
                        source_message_id: source_message_id.clone(),
                        target_conversation_id: target_conversation_id.clone(),
                        target_reply_to_message_id: None,
                    }
                    .encode_to_vec(),
                )))
                .expect("cross-channel-forward Gateway request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    let response = StartCrossChannelForwardResponseV1::decode(
        gateway_runtime
            .block_on(response.into_body().collect())
            .expect("cross-channel-forward Gateway response")
            .to_bytes(),
    )
    .expect("decode cross-channel-forward response");
    assert_eq!(response.forward_id, forward_id);
    assert_eq!(
        response.state,
        CrossChannelForwardStateV1::CrossChannelForwardStateAccepted as i32
    );
    assert_eq!(
        response.error,
        CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeUnspecified as i32
    );

    let terminal =
        read_terminal_cross_channel_forward_sse(&router, &gateway_runtime, &cookie, &forward_id);
    assert_eq!(
        terminal.state,
        CrossChannelForwardStateV1::CrossChannelForwardStateDeliveryAccepted as i32
    );
    assert_cross_channel_forward_cleanup_completed(
        &gateway_runtime,
        &cross_channel_forward_database_id(
            &supervisor,
            &store,
            &cross_channel_forward.registration_id,
        ),
        &forward_id,
    );
    let status = query_cross_channel_forward(&router, &gateway_runtime, &cookie, &forward_id);
    assert_eq!(
        status.state,
        CrossChannelForwardStateV1::CrossChannelForwardStateDeliveryAccepted as i32
    );
    assert_eq!(status.delivery_intent_id, Some(forward_id.clone()));
    assert_eq!(
        status.error,
        CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeUnspecified as i32
    );

    assert!(
        realtime
            .revoke_owner(CROSS_CHANNEL_FORWARD_LOGICAL_OWNER_ID)
            .expect("clear Gateway delivery cache"),
        "managed publications must admit the logical owner"
    );
    let previous_generation = cross_channel_forward.runtime_generation;
    let cross_channel_forward = restart_cross_channel_forward_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        cross_channel_forward,
    );
    assert_eq!(
        cross_channel_forward.runtime_generation,
        previous_generation + 1
    );
    let restarted_router = super::delivery_intent_realtime_flow::delivery_intent_gateway(
        &store,
        &supervisor,
        &root,
        &data,
        realtime,
    );
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed = read_terminal_cross_channel_forward_sse(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &forward_id,
    );
    assert_eq!(replayed, terminal);

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove kernel fixture");
}

fn assert_cross_channel_forward_cleanup_completed(
    runtime: &tokio::runtime::Runtime,
    database_id: &str,
    forward_id: &[u8],
) {
    runtime.block_on(async {
        let password = Zeroizing::new(
            std::fs::read_to_string(required(
                "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
            ))
            .expect("read disposable PostgreSQL credential")
            .trim()
            .to_owned(),
        );
        let options = PgConnectOptions::new()
            .host(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
            .port(
                required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
                    .parse()
                    .expect("valid PostgreSQL port"),
            )
            .username("makosh_postgres_admin")
            .password(password.as_str())
            .database(database_id)
            .ssl_mode(PgSslMode::Disable);
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect cross-channel-forward conformance database");
        let completed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.communication_cross_channel_forward_cleanup
             WHERE logical_owner_id = $1 AND forward_id = $2
               AND completed_at_unix_millis IS NOT NULL",
        )
        .bind(CROSS_CHANNEL_FORWARD_LOGICAL_OWNER_ID)
        .bind(forward_id)
        .fetch_one(&pool)
        .await
        .expect("read cross-channel-forward cleanup completion");
        pool.close().await;
        assert_eq!(completed, 1, "target-bound Blob custody must be released");
    });
}

fn query_cross_channel_forward(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    forward_id: &[u8],
) -> GetCrossChannelForwardStatusResponseV1 {
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("POST")
                .uri(COMMUNICATION_CROSS_CHANNEL_FORWARD_QUERY_CONNECT_PATH_V1)
                .header("content-type", "application/connect+proto")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(Bytes::from(
                    GetCrossChannelForwardStatusRequestV1 {
                        protocol_major: 1,
                        forward_id: forward_id.to_vec(),
                    }
                    .encode_to_vec(),
                )))
                .expect("cross-channel-forward status request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    GetCrossChannelForwardStatusResponseV1::decode(
        runtime
            .block_on(response.into_body().collect())
            .expect("cross-channel-forward status response")
            .to_bytes(),
    )
    .expect("decode cross-channel-forward status")
}

fn read_terminal_cross_channel_forward_sse(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    forward_id: &[u8],
) -> CrossChannelForwardStatusChangedV1 {
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
    let outcome = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(45), async {
            let mut body = response.into_body();
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
                    let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: "))
                    else {
                        continue;
                    };
                    let bytes = URL_SAFE_NO_PAD
                        .decode(encoded)
                        .expect("decode Gateway realtime frame");
                    let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                        .expect("decode realtime frame");
                    let Some(RealtimeFrame::Event(event)) = frame.frame else {
                        continue;
                    };
                    if event.contract_name
                        != COMMUNICATION_CROSS_CHANNEL_FORWARD_REALTIME_CONTRACT_NAME_V1
                    {
                        continue;
                    }
                    let payload =
                        CrossChannelForwardStatusChangedV1::decode(event.payload.as_slice())
                            .expect("decode cross-channel-forward event");
                    if payload.forward_id == forward_id
                        && payload.state
                            == CrossChannelForwardStateV1::CrossChannelForwardStateDeliveryAccepted
                                as i32
                    {
                        assert_eq!(
                            payload.error,
                            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeUnspecified
                                as i32
                        );
                        return payload;
                    }
                }
            }
            panic!("Gateway SSE closed before terminal cross-channel-forward event");
        })
        .await
    });
    outcome.unwrap_or_else(|error| {
        let status = query_cross_channel_forward(router, runtime, cookie, forward_id);
        panic!(
            "cross-channel-forward SSE timeout: {error:?}; state={}, revision={}, error={}",
            status.state, status.state_revision, status.error
        )
    })
}
