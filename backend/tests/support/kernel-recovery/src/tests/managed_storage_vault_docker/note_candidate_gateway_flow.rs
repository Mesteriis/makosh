//! Gateway, SSE and disposable-storage probes for the reviewed candidate chain.

use super::*;

use std::time::Instant;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_communication_note_candidate_api::{
    COMMUNICATION_NOTE_CANDIDATE_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_NOTE_CANDIDATE_QUERY_CONNECT_PATH_V1,
    COMMUNICATION_NOTE_CANDIDATE_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_NOTE_CANDIDATE_REALTIME_EVENT_KIND_V1,
    wire::{
        CommunicationNoteCandidateStateV1, CommunicationNoteCandidateStatusChangedV1,
        CommunicationNoteCandidateV1, GetCommunicationNoteCandidateRequestV1,
        GetCommunicationNoteCandidateResponseV1, StartCommunicationNoteCandidateRequestV1,
        StartCommunicationNoteCandidateResponseV1,
    },
};
use makosh_gateway_protocol::v1::{
    ClientRealtimeEventV1, ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};
use makosh_review_note_candidate_api::{
    REVIEW_NOTE_CANDIDATE_COMMAND_CONNECT_PATH_V1, REVIEW_NOTE_CANDIDATE_QUERY_CONNECT_PATH_V1,
    REVIEW_NOTE_CANDIDATE_REALTIME_CONTRACT_NAME_V1, REVIEW_NOTE_CANDIDATE_REALTIME_EVENT_KIND_V1,
    wire::{
        DecideReviewNoteCandidateRequestV1, DecideReviewNoteCandidateResponseV1,
        GetReviewNoteCandidateRequestV1, GetReviewNoteCandidateResponseV1,
        ReviewNoteCandidateDecisionV1, ReviewNoteCandidateErrorCodeV1,
        ReviewNoteCandidatePromotionStatusV1, ReviewNoteCandidateStateV1,
        ReviewNoteCandidateStatusChangedV1,
    },
};
use makosh_review_note_candidate_core::derive_review_note_candidate_id_v1;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use zeroize::Zeroizing;

pub(super) type NoteCandidateGateway = makosh_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

pub(super) struct NoteCandidateReviewsV1 {
    pub approved_review_id: Vec<u8>,
    pub approved_candidate_id: Vec<u8>,
    pub rejected_review_id: Vec<u8>,
    pub rejected_candidate_id: Vec<u8>,
}

pub(super) struct NoteCandidateTerminalEventsV1 {
    pub approved: ClientRealtimeEventV1,
    pub rejected: ClientRealtimeEventV1,
}

type ObservedNoteCandidateFramesV1 = Arc<std::sync::Mutex<Vec<(String, Vec<u8>, i32, i32, u64)>>>;

pub(super) fn note_candidate_gateway_v1(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> NoteCandidateGateway {
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("note-candidate-gateway-cert.der"),
        root.join("note-candidate-gateway-key.der"),
    )
    .expect("Note candidate Gateway configuration");
    crate::platform::gateway::gateway_service(
        Arc::clone(store),
        data,
        supervisor.clone(),
        realtime,
        &configuration,
        None,
    )
    .expect("compose Note candidate Gateway routes")
}

pub(super) fn start_note_candidate_extraction_v1(
    router: &NoteCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    operation_id: u8,
    source_message_id: &[u8],
    expected_source_revision: u64,
) -> StartCommunicationNoteCandidateResponseV1 {
    post_proto(
        router,
        runtime,
        cookie,
        COMMUNICATION_NOTE_CANDIDATE_COMMAND_CONNECT_PATH_V1,
        StartCommunicationNoteCandidateRequestV1 {
            protocol_major: 1,
            operation_id: vec![operation_id; 16],
            source_message_id: source_message_id.to_vec(),
            expected_source_revision,
        },
    )
}

pub(super) fn wait_for_ready_note_candidate_extraction_v1(
    router: &NoteCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetCommunicationNoteCandidateResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let response: GetCommunicationNoteCandidateResponseV1 = post_proto(
            router,
            runtime,
            cookie,
            COMMUNICATION_NOTE_CANDIDATE_QUERY_CONNECT_PATH_V1,
            GetCommunicationNoteCandidateRequestV1 {
                protocol_major: 1,
                run_id: run_id.to_vec(),
            },
        );
        if response.state
            == CommunicationNoteCandidateStateV1::CommunicationNoteCandidateStateReady as i32
        {
            return response;
        }
        assert!(
            response.state
                != CommunicationNoteCandidateStateV1::CommunicationNoteCandidateStateRejected
                    as i32,
            "Note candidate extraction rejected: {response:?}"
        );
        assert!(
            Instant::now() < deadline,
            "Note candidate extraction timeout: {response:?}; storage={}",
            note_candidate_storage_diagnostics_v1(runtime)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn wait_for_rejected_note_candidate_extraction_v1(
    router: &NoteCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetCommunicationNoteCandidateResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let response: GetCommunicationNoteCandidateResponseV1 = post_proto(
            router,
            runtime,
            cookie,
            COMMUNICATION_NOTE_CANDIDATE_QUERY_CONNECT_PATH_V1,
            GetCommunicationNoteCandidateRequestV1 {
                protocol_major: 1,
                run_id: run_id.to_vec(),
            },
        );
        if response.state
            == CommunicationNoteCandidateStateV1::CommunicationNoteCandidateStateRejected as i32
        {
            return response;
        }
        assert!(
            Instant::now() < deadline,
            "Note candidate rejection timeout: {response:?}; storage={}",
            note_candidate_storage_diagnostics_v1(runtime)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn wait_for_extracted_note_candidate_reviews_v1(
    router: &NoteCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    approved_candidate: &CommunicationNoteCandidateV1,
    rejected_candidate: &CommunicationNoteCandidateV1,
) -> NoteCandidateReviewsV1 {
    let ids = [approved_candidate, rejected_candidate]
        .iter()
        .map(|candidate| {
            let candidate_id: [u8; 16] = candidate
                .candidate_id
                .as_slice()
                .try_into()
                .expect("candidate id");
            let digest: [u8; 32] = candidate
                .candidate_digest
                .as_slice()
                .try_into()
                .expect("candidate digest");
            let review_id = derive_review_note_candidate_id_v1(
                NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
                &candidate_id,
                &digest,
            )
            .expect("derived Review id");
            (review_id.to_vec(), candidate_id.to_vec())
        })
        .collect::<Vec<_>>();
    let reviews = NoteCandidateReviewsV1 {
        approved_review_id: ids[0].0.clone(),
        approved_candidate_id: ids[0].1.clone(),
        rejected_review_id: ids[1].0.clone(),
        rejected_candidate_id: ids[1].1.clone(),
    };
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let approved =
            query_note_candidate_v1(router, runtime, cookie, &reviews.approved_review_id);
        let rejected =
            query_note_candidate_v1(router, runtime, cookie, &reviews.rejected_review_id);
        if [approved.review.as_ref(), rejected.review.as_ref()]
            .into_iter()
            .all(|review| {
                review.is_some_and(|review| {
                    review.state
                        == ReviewNoteCandidateStateV1::ReviewNoteCandidateStatePending as i32
                        && review.review_revision == 1
                })
            })
        {
            return reviews;
        }
        assert!(
            Instant::now() < deadline,
            "Review submission timeout: approved={approved:?}; rejected={rejected:?}; storage={}",
            note_candidate_storage_diagnostics_v1(runtime)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn decide_note_candidate_v1(
    router: &NoteCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    operation_id: u8,
    review_id: &[u8],
    expected_review_revision: u64,
    decision: ReviewNoteCandidateDecisionV1,
) -> DecideReviewNoteCandidateResponseV1 {
    post_proto(
        router,
        runtime,
        cookie,
        REVIEW_NOTE_CANDIDATE_COMMAND_CONNECT_PATH_V1,
        DecideReviewNoteCandidateRequestV1 {
            protocol_major: 1,
            operation_id: vec![operation_id; 16],
            review_id: review_id.to_vec(),
            expected_review_revision,
            decision: decision as i32,
        },
    )
}

pub(super) fn query_note_candidate_v1(
    router: &NoteCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    review_id: &[u8],
) -> GetReviewNoteCandidateResponseV1 {
    post_proto(
        router,
        runtime,
        cookie,
        REVIEW_NOTE_CANDIDATE_QUERY_CONNECT_PATH_V1,
        GetReviewNoteCandidateRequestV1 {
            protocol_major: 1,
            review_id: review_id.to_vec(),
        },
    )
}

pub(super) fn wait_for_note_candidate_terminal_states_v1(
    router: &NoteCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    reviews: &NoteCandidateReviewsV1,
) -> (
    GetReviewNoteCandidateResponseV1,
    GetReviewNoteCandidateResponseV1,
) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let approved =
            query_note_candidate_v1(router, runtime, cookie, &reviews.approved_review_id);
        let rejected =
            query_note_candidate_v1(router, runtime, cookie, &reviews.rejected_review_id);
        let approved_terminal = approved.review.as_ref().is_some_and(|review| {
            review.promotion_status
                == ReviewNoteCandidatePromotionStatusV1::ReviewNoteCandidatePromotionStatusSucceeded
                    as i32
                && review.review_revision == 3
        });
        let rejected_terminal = rejected.review.as_ref().is_some_and(|review| {
            review.state == ReviewNoteCandidateStateV1::ReviewNoteCandidateStateRejected as i32
                && review.review_revision == 2
        });
        if approved_terminal && rejected_terminal {
            return (approved, rejected);
        }
        assert!(
            Instant::now() < deadline,
            "Note candidate terminal state timeout: approved={approved:?}; rejected={rejected:?}; storage={}",
            note_candidate_storage_diagnostics_v1(runtime)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn note_candidate_storage_diagnostics_v1(runtime: &tokio::runtime::Runtime) -> String {
    runtime.block_on(async {
        let pool = note_candidate_admin_pool_v1().await;
        let review_outbox: (i64, i64) = sqlx::query_as(
            "SELECT count(*), count(*) FILTER (WHERE published_at_unix_millis IS NOT NULL)
             FROM makosh_data.review_note_candidate_outbox",
        )
        .fetch_one(&pool)
        .await
        .expect("Review outbox diagnostics");
        let workflow: (i64, i64, i64) = sqlx::query_as(
            "SELECT
               (SELECT count(*) FROM makosh_data.reviewed_note_candidate_promotion_requests),
               (SELECT count(*) FROM makosh_data.reviewed_note_candidate_promotion_result_inbox),
               (SELECT count(*) FROM makosh_data.reviewed_note_candidate_promotion_outbox)",
        )
        .fetch_one(&pool)
        .await
        .expect("promotion workflow diagnostics");
        let knowledge: (i64, i64, i64) = sqlx::query_as(
            "SELECT
               (SELECT count(*) FROM makosh_data.knowledge_reviewed_candidate_inbox),
               (SELECT count(*) FROM makosh_data.knowledge_state),
               (SELECT count(*) FROM makosh_data.knowledge_outbox)",
        )
        .fetch_one(&pool)
        .await
        .expect("Knowledge diagnostics");
        let review_results: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM makosh_data.review_note_candidate_promotion_inbox",
        )
        .fetch_one(&pool)
        .await
        .expect("Review result inbox diagnostics");
        pool.close().await;
        format!(
            "review_outbox={review_outbox:?}, workflow={workflow:?}, knowledge={knowledge:?}, review_results={review_results}"
        )
    })
}

pub(super) fn read_note_candidate_terminal_events_v1(
    router: &NoteCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    reviews: &NoteCandidateReviewsV1,
) -> NoteCandidateTerminalEventsV1 {
    let observed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Note candidate Gateway SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    runtime.block_on(async {
        match tokio::time::timeout(
            Duration::from_secs(15),
            find_terminal_events(response.into_body(), reviews, Arc::clone(&observed)),
        )
        .await
        {
            Ok(events) => events,
            Err(_) => panic!(
                "Note candidate SSE timeout; observed={:?}",
                observed.lock().expect("observed Note candidate frames"),
            ),
        }
    })
}

pub(super) fn read_note_candidate_extraction_terminal_event_v1(
    router: &NoteCandidateGateway,
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
                .expect("Note candidate extraction Gateway SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(15),
            find_extraction_terminal_event(response.into_body(), run_id),
        )
        .await
        .unwrap_or_else(|_| panic!("Note candidate extraction SSE timeout for {run_id:?}"))
    })
}

pub(super) fn assert_exact_note_materialization_v1(
    runtime: &tokio::runtime::Runtime,
    reviews: &NoteCandidateReviewsV1,
) {
    runtime.block_on(async {
        let pool = note_candidate_admin_pool_v1().await;
        let approved: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM makosh_data.knowledge_state
             WHERE logical_owner_id=$1 AND approved_candidate_id=$2",
        )
        .bind(NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
        .bind(&reviews.approved_candidate_id)
        .fetch_one(&pool)
        .await
        .expect("count approved candidate Knowledge");
        let rejected: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM makosh_data.knowledge_state
             WHERE logical_owner_id=$1 AND approved_candidate_id=$2",
        )
        .bind(NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
        .bind(&reviews.rejected_candidate_id)
        .fetch_one(&pool)
        .await
        .expect("count rejected candidate Knowledge");
        assert_eq!(
            approved, 1,
            "approve must materialize exactly one Knowledge note"
        );
        assert_eq!(
            rejected, 0,
            "reject must never materialize a Knowledge note"
        );
        pool.close().await;
    });
}

pub(super) fn assert_no_note_materialization_v1(
    runtime: &tokio::runtime::Runtime,
    reviews: &NoteCandidateReviewsV1,
) {
    runtime.block_on(async {
        let pool = note_candidate_admin_pool_v1().await;
        for candidate_id in [
            &reviews.approved_candidate_id,
            &reviews.rejected_candidate_id,
        ] {
            let count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM makosh_data.knowledge_state
                 WHERE logical_owner_id=$1 AND approved_candidate_id=$2",
            )
            .bind(NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
            .bind(candidate_id)
            .fetch_one(&pool)
            .await
            .expect("count pre-decision Knowledge");
            assert_eq!(
                count, 0,
                "extraction must not create Knowledge note before approve"
            );
        }
        pool.close().await;
    });
}

fn post_proto<M, R>(
    router: &NoteCandidateGateway,
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
                    .expect("Note candidate Gateway request"),
            ),
        );
        let status = response.status();
        let bytes = runtime
            .block_on(response.into_body().collect())
            .expect("Note candidate Gateway response")
            .to_bytes();
        if status == StatusCode::SERVICE_UNAVAILABLE && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
            continue;
        }
        assert_eq!(
            status,
            StatusCode::OK,
            "Note candidate Gateway response: {}",
            String::from_utf8_lossy(&bytes)
        );
        return R::decode(bytes.as_ref()).expect("decode Note candidate Gateway response");
    }
}

async fn find_terminal_events<B>(
    mut body: B,
    reviews: &NoteCandidateReviewsV1,
    observed: ObservedNoteCandidateFramesV1,
) -> NoteCandidateTerminalEventsV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    let mut approved: Option<ClientRealtimeEventV1> = None;
    let mut rejected: Option<ClientRealtimeEventV1> = None;
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Note candidate SSE frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Note candidate SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Note candidate frame");
            let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                .expect("Note candidate realtime frame");
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            observed.lock().expect("record realtime contract").push((
                event.contract_name.clone(),
                Vec::new(),
                0,
                0,
                0,
            ));
            if event.contract_name != REVIEW_NOTE_CANDIDATE_REALTIME_CONTRACT_NAME_V1
                || event.event_kind != REVIEW_NOTE_CANDIDATE_REALTIME_EVENT_KIND_V1
            {
                continue;
            }
            let payload = ReviewNoteCandidateStatusChangedV1::decode(event.payload.as_slice())
                .expect("Note candidate status payload");
            observed.lock().expect("record Note candidate frame").push((
                event.contract_name.clone(),
                payload.review_id.clone(),
                payload.state,
                payload.promotion_status,
                payload.review_revision,
            ));
            if payload.review_id == reviews.approved_review_id
                && payload.state == ReviewNoteCandidateStateV1::ReviewNoteCandidateStateApproved as i32
                && payload.promotion_status
                    == ReviewNoteCandidatePromotionStatusV1::ReviewNoteCandidatePromotionStatusSucceeded as i32
                && payload.review_revision == 3
            {
                approved = Some(event);
            } else if payload.review_id == reviews.rejected_review_id
                && payload.state == ReviewNoteCandidateStateV1::ReviewNoteCandidateStateRejected as i32
                && payload.promotion_status
                    == ReviewNoteCandidatePromotionStatusV1::ReviewNoteCandidatePromotionStatusNotRequested as i32
                && payload.review_revision == 2
            {
                rejected = Some(event);
            }
            if let (Some(approved), Some(rejected)) = (&approved, &rejected) {
                return NoteCandidateTerminalEventsV1 {
                    approved: approved.clone(),
                    rejected: rejected.clone(),
                };
            }
        }
    }
    panic!("Gateway SSE closed before both Note candidate terminal events");
}

async fn find_extraction_terminal_event<B>(mut body: B, run_id: &[u8]) -> ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Note candidate extraction SSE frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Note candidate extraction SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Note candidate extraction frame");
            let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                .expect("Note candidate extraction realtime frame");
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            if event.contract_name != COMMUNICATION_NOTE_CANDIDATE_REALTIME_CONTRACT_NAME_V1
                || event.event_kind != COMMUNICATION_NOTE_CANDIDATE_REALTIME_EVENT_KIND_V1
            {
                continue;
            }
            let payload =
                CommunicationNoteCandidateStatusChangedV1::decode(event.payload.as_slice())
                    .expect("Note candidate extraction status payload");
            if payload.run_id == run_id
                && payload.state
                    == CommunicationNoteCandidateStateV1::CommunicationNoteCandidateStateReady
                        as i32
            {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before Note candidate extraction terminal event");
}

pub(super) async fn note_candidate_admin_pool_v1() -> sqlx::PgPool {
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
        .database("makosh_storage_authenticated")
        .ssl_mode(PgSslMode::Disable);
    PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect Note candidate conformance database")
}

pub(super) fn assert_note_candidate_response_states_v1(
    approved: &GetReviewNoteCandidateResponseV1,
    rejected: &GetReviewNoteCandidateResponseV1,
) {
    assert_eq!(
        approved.error,
        ReviewNoteCandidateErrorCodeV1::ReviewNoteCandidateErrorCodeUnspecified as i32
    );
    let approved = approved.review.as_ref().expect("approved Review state");
    assert_eq!(
        approved.promotion_status,
        ReviewNoteCandidatePromotionStatusV1::ReviewNoteCandidatePromotionStatusSucceeded as i32
    );
    assert_eq!(approved.review_revision, 3);
    assert_eq!(
        rejected.error,
        ReviewNoteCandidateErrorCodeV1::ReviewNoteCandidateErrorCodeUnspecified as i32
    );
    let rejected = rejected.review.as_ref().expect("rejected Review state");
    assert_eq!(
        rejected.state,
        ReviewNoteCandidateStateV1::ReviewNoteCandidateStateRejected as i32
    );
    assert_eq!(rejected.review_revision, 2);
}
