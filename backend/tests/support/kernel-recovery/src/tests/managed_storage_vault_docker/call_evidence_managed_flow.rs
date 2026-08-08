//! Live Telegram call evidence through NATS into Communications query and shared SSE.

use std::collections::BTreeSet;

use super::*;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_communications_call_evidence_api::{
    CALL_EVIDENCE_QUERY_CONNECT_PATH_V1, CALL_EVIDENCE_REALTIME_CONTRACT_NAME_V1,
    wire::{
        CallEvidenceChangedV1, CallEvidenceQueryRequestV1, CallEvidenceQueryResponseV1,
        CallEvidenceSummaryV1, ListCallEvidenceRequestV1,
        call_evidence_query_request_v1::Operation as CallEvidenceQueryOperation,
        call_evidence_query_response_v1::Result as CallEvidenceQueryResult,
    },
};
use makosh_gateway_protocol::v1::{
    ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};
use makosh_telegram_calls_api::{
    contract::TelegramCallsContractV1,
    wire::{
        CallsCommandRequestV1, InitiateAudioCallRequestV1, calls_command_request_v1,
        calls_command_response_v1,
    },
};
use makosh_telegram_calls_core::{
    TelegramCallDirection, TelegramCallDiscardReason, TelegramProviderCallState,
    TelegramProviderCallUpdate,
};
use makosh_telegram_calls_persistence::TelegramCallsPersistence;

const HUMAN_OWNER_ID: &str = "owner-1";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_call_evidence_survives_nats_outage_and_replays_through_gateway_sse() {
    let mut fixture = super::telegram_managed_flow::prepare_managed_telegram_fixture();
    let telegram = fixture.start_telegram();
    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = super::delivery_intent_realtime_flow::delivery_intent_gateway(
        &fixture.store,
        &fixture.supervisor,
        &fixture.root,
        &fixture.data,
        fixture.realtime.clone(),
    );
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let initial = wait_for_call_evidence_query(&router, &gateway_runtime, &cookie);
    let initial_ids = initial
        .iter()
        .map(|evidence| evidence.call_evidence_id.clone())
        .collect::<BTreeSet<_>>();
    initiate_managed_call(
        &fixture,
        &telegram,
        9_001,
        "managed-call-evidence-live",
        "909001",
    );
    let live = wait_for_call_evidence(&router, &gateway_runtime, &cookie, &initial_ids);
    let baseline_ids = live
        .iter()
        .map(|evidence| evidence.call_evidence_id.clone())
        .collect::<BTreeSet<_>>();

    let events = fixture
        .store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let nats_runtime = tokio::runtime::Runtime::new().expect("NATS observer runtime");
    let nats_observer = nats_runtime
        .block_on(async_nats::connect(events.nats_endpoint()))
        .expect("connect call evidence outage observer");
    super::nats_outage_fixture::set_authenticated_nats_container_running(false);

    let private_provider_user_id = "private-provider-user-909";
    enqueue_provider_call_evidence_during_outage(
        &nats_runtime,
        &telegram,
        private_provider_user_id,
    );
    wait_for_pending_call_evidence();

    super::nats_outage_fixture::set_authenticated_nats_container_running(true);
    super::nats_outage_fixture::wait_for_authenticated_nats_reconnect(
        &nats_runtime,
        &nats_observer,
        "call evidence outage observer",
    );
    let after_outage = wait_for_call_evidence(&router, &gateway_runtime, &cookie, &baseline_ids);
    let evidence = after_outage
        .iter()
        .find(|evidence| !baseline_ids.contains(&evidence.call_evidence_id))
        .expect("new call evidence after NATS recovery");
    let first_event = read_call_evidence_sse(
        &router,
        &gateway_runtime,
        &cookie,
        &evidence.call_evidence_id,
    );
    assert_call_evidence_event_is_client_safe(
        &first_event,
        &evidence.call_evidence_id,
        private_provider_user_id.as_bytes(),
    );

    assert!(
        fixture
            .realtime
            .revoke_owner(HUMAN_OWNER_ID)
            .expect("clear Gateway call evidence cache"),
        "managed call evidence must have admitted the human owner",
    );
    let restarted_generation = restart_communications_domain(
        &fixture.supervisor,
        &fixture.store,
        &fixture.root.join("runtime"),
    );
    assert_eq!(restarted_generation, 2);
    let restarted_router = super::delivery_intent_realtime_flow::delivery_intent_gateway(
        &fixture.store,
        &fixture.supervisor,
        &fixture.root,
        &fixture.data,
        fixture.realtime.clone(),
    );
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed = read_call_evidence_sse(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &evidence.call_evidence_id,
    );
    assert_eq!(
        replayed.cursor, first_event.cursor,
        "Communications restart must restore the stable call evidence cursor",
    );
    assert_call_evidence_event_is_client_safe(
        &replayed,
        &evidence.call_evidence_id,
        private_provider_user_id.as_bytes(),
    );
}

fn initiate_managed_call(
    fixture: &super::telegram_managed_flow::PreparedManagedTelegramFixture,
    telegram: &StartedTelegramRuntime,
    request_id: u64,
    operation_id: &str,
    provider_user_id: &str,
) {
    let response = super::telegram_managed_flow::decode_calls_command_response(
        &super::telegram_managed_flow::route_telegram_calls_until_ready(
            &fixture.store,
            &fixture.supervisor,
            telegram,
            TelegramCallsContractV1::Command,
            request_id,
            &CallsCommandRequestV1 {
                request: Some(calls_command_request_v1::Request::InitiateAudioCall(
                    InitiateAudioCallRequestV1 {
                        operation_id: operation_id.to_owned(),
                        account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                        provider_user_id: provider_user_id.to_owned(),
                    },
                )),
            }
            .encode_to_vec(),
        ),
    );
    assert!(
        matches!(
            response.response,
            Some(calls_command_response_v1::Response::Accepted(_))
        ),
        "managed Telegram call was not accepted: {response:?}",
    );
}

fn enqueue_provider_call_evidence_during_outage(
    runtime: &tokio::runtime::Runtime,
    telegram: &StartedTelegramRuntime,
    provider_user_id: &str,
) {
    runtime.block_on(async {
        let persistence = TelegramCallsPersistence::new(
            super::telegram_managed_setup::telegram_admin_pool().await,
        );
        persistence
            .ingest_provider_update_with_call_evidence(
                "managed-call-evidence-outage-session",
                &TelegramProviderCallUpdate {
                    account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                    runtime_generation: telegram.runtime_generation,
                    tdlib_call_id: 9_091,
                    provider_call_unique_id: Some(9_091),
                    provider_user_id: provider_user_id.to_owned(),
                    direction: TelegramCallDirection::Incoming,
                    state: TelegramProviderCallState::Discarded,
                    pending_created: false,
                    pending_received: false,
                    discard_reason: Some(TelegramCallDiscardReason::Missed),
                    failure_category: None,
                    observed_at_unix_seconds: 1_783_034_501,
                },
                HUMAN_OWNER_ID,
                &telegram.runtime_instance_id,
            )
            .await
            .expect("persist provider-owned call evidence during NATS outage");
    });
}

fn wait_for_pending_call_evidence() {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if telegram_pending_call_evidence_count() > 0 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Telegram did not retain call evidence during the NATS outage",
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_call_evidence(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    excluded_ids: &BTreeSet<Vec<u8>>,
) -> Vec<CallEvidenceSummaryV1> {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(evidence) = query_call_evidence(router, runtime, cookie)
            && evidence
                .iter()
                .any(|item| !excluded_ids.contains(&item.call_evidence_id))
        {
            return evidence;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "call evidence did not reach the managed Communications query",
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn wait_for_call_evidence_query(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
) -> Vec<CallEvidenceSummaryV1> {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(evidence) = query_call_evidence(router, runtime, cookie) {
            return evidence;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "managed Communications call evidence query did not become ready",
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn query_call_evidence(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
) -> Option<Vec<CallEvidenceSummaryV1>> {
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("POST")
                .uri(CALL_EVIDENCE_QUERY_CONNECT_PATH_V1)
                .header("content-type", "application/connect+proto")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(Bytes::from(
                    CallEvidenceQueryRequestV1 {
                        protocol_major: 1,
                        operation: Some(CallEvidenceQueryOperation::List(
                            ListCallEvidenceRequestV1 {
                                limit: 100,
                                cursor: Vec::new(),
                                provider: None,
                                direction: None,
                                media_kind: None,
                                state: None,
                            },
                        )),
                    }
                    .encode_to_vec(),
                )))
                .expect("call evidence Gateway query"),
        ),
    );
    if response.status() == StatusCode::SERVICE_UNAVAILABLE {
        return None;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let response = CallEvidenceQueryResponseV1::decode(
        runtime
            .block_on(response.into_body().collect())
            .expect("call evidence Gateway response")
            .to_bytes(),
    )
    .expect("decode call evidence response");
    assert!(response.error_code.is_empty(), "{}", response.error_code);
    let Some(CallEvidenceQueryResult::List(page)) = response.result else {
        panic!("call evidence list response is missing");
    };
    Some(page.evidence)
}

fn read_call_evidence_sse(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    expected_id: &[u8],
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
    runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(30),
            find_call_evidence_event(response.into_body(), expected_id),
        )
        .await
        .expect("call evidence SSE event timeout")
    })
}

async fn find_call_evidence_event<B>(
    mut body: B,
    expected_id: &[u8],
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
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            if event.contract_name != CALL_EVIDENCE_REALTIME_CONTRACT_NAME_V1 {
                continue;
            }
            let payload =
                CallEvidenceChangedV1::decode(event.payload.as_slice()).expect("event payload");
            if payload.call_evidence_id == expected_id {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before call evidence event");
}

fn assert_call_evidence_event_is_client_safe(
    event: &makosh_gateway_protocol::v1::ClientRealtimeEventV1,
    expected_id: &[u8],
    private_provider_id: &[u8],
) {
    let payload = CallEvidenceChangedV1::decode(event.payload.as_slice())
        .expect("decode call evidence changed payload");
    assert_eq!(payload.call_evidence_id, expected_id);
    assert!(payload.canonical_revision > 0);
    assert!(
        !event
            .payload
            .windows(private_provider_id.len())
            .any(|window| window == private_provider_id),
        "provider locator must not enter the shared SSE frame",
    );
}
