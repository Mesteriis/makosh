//! Live Review command/query publication through the owner-neutral Gateway SSE.

use super::*;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_gateway_protocol::v1::{
    ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};
use makosh_review_attention_api::{
    REVIEW_ATTENTION_COMMAND_CONNECT_PATH_V1, REVIEW_ATTENTION_QUERY_CONNECT_PATH_V1,
    REVIEW_ATTENTION_REALTIME_CONTRACT_NAME_V1, REVIEW_ATTENTION_REALTIME_EVENT_KIND_V1,
    wire::{
        GetReviewAttentionV1, ListReviewAttentionV1, MarkPendingV1, ReviewAttentionChangedV1,
        ReviewAttentionCommandRequestV1, ReviewAttentionCommandResponseV1,
        ReviewAttentionQueryRequestV1, ReviewAttentionQueryResponseV1, SetPinnedV1,
        review_attention_command_request_v1::Operation as CommandOperation,
        review_attention_query_request_v1::Operation as QueryOperation,
        review_attention_query_response_v1::Result as QueryResult,
    },
};

type ReviewAttentionGateway = makosh_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage and Review binaries"]
fn managed_review_attention_reaches_gateway_sse_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-review-attention");
    let data = private_directory(root.join("kernel"));
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_review_attention_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            REVIEW_ATTENTION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim Review logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        REVIEW_ATTENTION_LOGICAL_OWNER_ID_V1,
    );
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted = admit_review_attention_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(32).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_review_attention_realtime_v1(&supervisor, &store, realtime.clone());
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_review_attention_runtime_v1(&supervisor, &store, admitted);
    let review =
        start_review_attention_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted);
    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = review_attention_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let first_sse = gateway_runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", &cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Review Gateway first SSE request"),
        ),
    );
    assert_eq!(first_sse.status(), StatusCode::OK);

    let operation_id = vec![0x41; 16];
    let source_evidence_id = vec![0x52; 16];
    let command = ReviewAttentionCommandRequestV1 {
        protocol_major: 1,
        operation_id,
        source_evidence_id: source_evidence_id.clone(),
        expected_revision: 0,
        operation: Some(CommandOperation::MarkPending(MarkPendingV1 {})),
    };
    let response: ReviewAttentionCommandResponseV1 = post_proto(
        &router,
        &gateway_runtime,
        &cookie,
        REVIEW_ATTENTION_COMMAND_CONNECT_PATH_V1,
        command,
    );
    assert!(response.error_code.is_empty());
    assert!(!response.replayed);
    let attention = response.attention.expect("created attention");
    assert_eq!(attention.revision, 1);
    assert_eq!(attention.source_evidence_id, source_evidence_id);
    let durable_counts: (i64, i64) = gateway_runtime.block_on(async {
        let admin = authenticated_storage_admin_pool_v1().await;
        sqlx::query_as(
            "SELECT
               (SELECT COUNT(*) FROM makosh_data.review_attention_state
                WHERE logical_owner_id='owner-1'),
               (SELECT COUNT(*) FROM makosh_data.review_attention_realtime
                WHERE logical_owner_id='owner-1')",
        )
        .fetch_one(&admin)
        .await
        .expect("read durable Review Attention stages")
    });
    assert_eq!(durable_counts, (1, 1));

    let immediate_query: ReviewAttentionQueryResponseV1 = post_proto(
        &router,
        &gateway_runtime,
        &cookie,
        REVIEW_ATTENTION_QUERY_CONNECT_PATH_V1,
        ReviewAttentionQueryRequestV1 {
            protocol_major: 1,
            operation: Some(QueryOperation::Get(GetReviewAttentionV1 {
                attention_id: attention.attention_id.clone(),
            })),
        },
    );
    assert!(matches!(
        immediate_query.result,
        Some(QueryResult::Attention(ref value)) if value.revision == 1
    ));

    let stale_response: ReviewAttentionCommandResponseV1 = post_proto(
        &router,
        &gateway_runtime,
        &cookie,
        REVIEW_ATTENTION_COMMAND_CONNECT_PATH_V1,
        ReviewAttentionCommandRequestV1 {
            protocol_major: 1,
            operation_id: vec![0x42; 16],
            source_evidence_id: source_evidence_id.clone(),
            expected_revision: 0,
            operation: Some(CommandOperation::SetPinned(SetPinnedV1 { pinned: true })),
        },
    );
    assert_eq!(stale_response.error_code, "stale_revision");
    assert!(stale_response.attention.is_none());

    let first = read_review_attention_sse_event(
        first_sse,
        &gateway_runtime,
        "first",
        &supervisor,
        &review.registration_id,
    );
    let first_payload = ReviewAttentionChangedV1::decode(first.payload.as_slice())
        .expect("Review realtime payload");
    assert_eq!(first_payload.attention_id, attention.attention_id);
    assert_eq!(first_payload.revision, 1);
    assert!(
        !first
            .encode_to_vec()
            .windows(source_evidence_id.len())
            .any(|window| window == source_evidence_id),
        "client realtime must not expose source evidence identity"
    );
    let first_cursor = first.cursor.clone();

    let query = ReviewAttentionQueryRequestV1 {
        protocol_major: 1,
        operation: Some(QueryOperation::Get(GetReviewAttentionV1 {
            attention_id: attention.attention_id.clone(),
        })),
    };
    let query_response: ReviewAttentionQueryResponseV1 = post_proto(
        &router,
        &gateway_runtime,
        &cookie,
        REVIEW_ATTENTION_QUERY_CONNECT_PATH_V1,
        query,
    );
    assert!(query_response.error_code.is_empty());
    assert!(matches!(
        query_response.result,
        Some(QueryResult::Attention(ref value)) if value.revision == 1
    ));
    let list_response: ReviewAttentionQueryResponseV1 = post_proto(
        &router,
        &gateway_runtime,
        &cookie,
        REVIEW_ATTENTION_QUERY_CONNECT_PATH_V1,
        ReviewAttentionQueryRequestV1 {
            protocol_major: 1,
            operation: Some(QueryOperation::List(ListReviewAttentionV1 {
                limit: 1,
                cursor: Vec::new(),
                disposition: None,
                pinned: None,
                importance: None,
                snoozed: None,
            })),
        },
    );
    assert!(matches!(
        list_response.result,
        Some(QueryResult::Page(ref page))
            if page.attention.len() == 1 && page.next_cursor.is_empty()
    ));

    assert!(
        realtime
            .revoke_owner(REVIEW_ATTENTION_LOGICAL_OWNER_ID_V1)
            .expect("clear Review Gateway replay cache")
    );
    let restarted_router =
        review_attention_gateway(&store, &supervisor, &root, &data, realtime.clone());
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
                .expect("Review Gateway replay SSE request"),
        ),
    );
    assert_eq!(replay_sse.status(), StatusCode::OK);
    let previous_generation = review.runtime_generation;
    let review =
        restart_review_attention_runtime_v1(&supervisor, &store, &root.join("runtime"), review);
    assert_eq!(review.runtime_generation, previous_generation + 1);
    let replayed = read_review_attention_sse_event(
        replay_sse,
        &gateway_runtime,
        "restart",
        &supervisor,
        &review.registration_id,
    );
    assert_eq!(replayed.cursor, first_cursor);
    let replayed_payload = ReviewAttentionChangedV1::decode(replayed.payload.as_slice())
        .expect("replayed Review payload");
    assert_eq!(replayed_payload.attention_id, attention.attention_id);
    assert_eq!(replayed_payload.revision, 1);

    // Effective NOLOGIN/NOSUPERUSER/NOBYPASSRLS proof for all Review Attention tables.
    gateway_runtime.block_on(assert_review_owner_rls_v1(
        "makosh_review_attention_rls_test",
        &[
            "review_attention_state",
            "review_attention_operations",
            "review_attention_realtime",
        ],
    ));

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Review fixture");
}

fn review_attention_gateway(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> ReviewAttentionGateway {
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("review-attention-gateway-cert.der"),
        root.join("review-attention-gateway-key.der"),
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
    .expect("compose Review Gateway routes")
}

fn post_proto<M, R>(
    router: &ReviewAttentionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    path: &str,
    message: M,
) -> R
where
    M: Message,
    R: Message + Default,
{
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/connect+proto")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(Bytes::from(
                    message.encode_to_vec(),
                )))
                .expect("Review Gateway request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = runtime
        .block_on(response.into_body().collect())
        .expect("Review Gateway response")
        .to_bytes();
    R::decode(bytes.as_ref()).expect("decode Review Gateway response")
}

fn read_review_attention_sse_event<B>(
    response: hyper::Response<B>,
    runtime: &tokio::runtime::Runtime,
    stage: &str,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
) -> makosh_gateway_protocol::v1::ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let result = runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(8),
            find_review_attention_event(response.into_body()),
        )
        .await
    });
    result.unwrap_or_else(|error| {
        panic!(
            "Review SSE timeout stage={stage} error={error:?} active={:?} last_failure={:?}",
            supervisor.is_active(registration_id),
            supervisor.last_failure(registration_id)
        )
    })
}

async fn find_review_attention_event<B>(
    mut body: B,
) -> makosh_gateway_protocol::v1::ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Review SSE frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Review SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Review frame");
            let frame =
                ClientRealtimeFrameV1::decode(bytes.as_slice()).expect("Review realtime frame");
            if let Some(RealtimeFrame::Event(event)) = frame.frame
                && event.contract_name == REVIEW_ATTENTION_REALTIME_CONTRACT_NAME_V1
                && event.event_kind == REVIEW_ATTENTION_REALTIME_EVENT_KIND_V1
            {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before Review attention event");
}
