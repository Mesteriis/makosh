//! Authenticated Gateway, private result Blob and replayable SSE fixture for Attachment Translation.

use std::time::{Duration, Instant};

use super::*;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_attachment_translation_api::{
    ATTACHMENT_TRANSLATION_QUERY_CONNECT_PATH_V1, ATTACHMENT_TRANSLATION_READ_BLOB_PATH_V1,
    ATTACHMENT_TRANSLATION_REALTIME_CONTRACT_NAME_V1,
    ATTACHMENT_TRANSLATION_REALTIME_EVENT_KIND_V1,
    read_wire::ReadAttachmentTranslationRequestV1,
    wire::{
        AttachmentTranslationStateV1, AttachmentTranslationStatusChangedV1,
        GetAttachmentTranslationRequestV1, GetAttachmentTranslationResponseV1,
    },
};
use makosh_gateway_protocol::v1::{
    ClientRealtimeEventV1, ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};

pub(super) type AttachmentTranslationGateway = makosh_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

pub(super) fn attachment_translation_gateway_v1(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> AttachmentTranslationGateway {
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("attachment-translation-gateway-cert.der"),
        root.join("attachment-translation-gateway-key.der"),
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
    .expect("compose Attachment Translation Gateway routes")
}

pub(super) fn post_attachment_translation_proto_v1<M, R>(
    router: &AttachmentTranslationGateway,
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
                    .expect("Attachment Translation Gateway request"),
            ),
        );
        let status = response.status();
        let bytes = runtime
            .block_on(response.into_body().collect())
            .expect("Attachment Translation Gateway response")
            .to_bytes();
        if status == StatusCode::SERVICE_UNAVAILABLE && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
            continue;
        }
        assert_eq!(
            status,
            StatusCode::OK,
            "Attachment Translation Gateway response body: {}",
            String::from_utf8_lossy(&bytes)
        );
        return R::decode(bytes.as_ref()).expect("decode Attachment Translation response");
    }
}

pub(super) fn post_attachment_translation_proto_status_v1<M>(
    router: &AttachmentTranslationGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: Option<&str>,
    path: &str,
    message: M,
) -> StatusCode
where
    M: Message,
{
    let mut request = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/connect+proto");
    if let Some(cookie) = cookie {
        request = request.header("cookie", cookie);
    }
    runtime
        .block_on(
            router.route(
                request
                    .body(http_body_util::Full::new(Bytes::from(
                        message.encode_to_vec(),
                    )))
                    .expect("Attachment Translation Gateway status request"),
            ),
        )
        .status()
}

pub(super) fn read_attachment_translation_blob_v1(
    router: &AttachmentTranslationGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: Option<&str>,
    opaque_read_ticket: Vec<u8>,
) -> (StatusCode, Vec<u8>) {
    let mut request = Request::builder()
        .method("POST")
        .uri(ATTACHMENT_TRANSLATION_READ_BLOB_PATH_V1)
        .header("content-type", "application/protobuf");
    if let Some(cookie) = cookie {
        request = request.header("cookie", cookie);
    }
    let response = runtime.block_on(
        router.route(
            request
                .body(http_body_util::Full::new(Bytes::from(
                    ReadAttachmentTranslationRequestV1 {
                        protocol_major: 1,
                        opaque_read_ticket,
                    }
                    .encode_to_vec(),
                )))
                .expect("Attachment Translation client Blob request"),
        ),
    );
    let status = response.status();
    let body = runtime
        .block_on(response.into_body().collect())
        .expect("Attachment Translation client Blob response")
        .to_bytes()
        .to_vec();
    (status, body)
}

pub(super) fn get_attachment_translation_v1(
    router: &AttachmentTranslationGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetAttachmentTranslationResponseV1 {
    post_attachment_translation_proto_v1(
        router,
        runtime,
        cookie,
        ATTACHMENT_TRANSLATION_QUERY_CONNECT_PATH_V1,
        GetAttachmentTranslationRequestV1 {
            protocol_major: 1,
            run_id: run_id.to_vec(),
        },
    )
}

pub(super) fn read_terminal_attachment_translation_sse_event_v1(
    router: &AttachmentTranslationGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> ClientRealtimeEventV1 {
    let response = open_attachment_translation_sse_v1(router, runtime, cookie);
    read_terminal_attachment_translation_sse_response_v1(runtime, response, run_id)
}

pub(super) fn open_attachment_translation_sse_v1(
    router: &AttachmentTranslationGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
) -> makosh_gateway_runtime::GatewayHttpResponse {
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Attachment Translation Gateway SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    response
}

pub(super) fn read_terminal_attachment_translation_sse_response_v1(
    runtime: &tokio::runtime::Runtime,
    response: makosh_gateway_runtime::GatewayHttpResponse,
    run_id: &[u8],
) -> ClientRealtimeEventV1 {
    runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(40),
            find_terminal_attachment_translation_event_v1(response.into_body(), run_id),
        )
        .await
        .expect("Attachment Translation SSE timeout")
    })
}

async fn find_terminal_attachment_translation_event_v1<B>(
    mut body: B,
    run_id: &[u8],
) -> ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Attachment Translation SSE frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Attachment Translation SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Attachment Translation frame");
            let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                .expect("Attachment Translation realtime frame");
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            if event.contract_name != ATTACHMENT_TRANSLATION_REALTIME_CONTRACT_NAME_V1
                || event.event_kind != ATTACHMENT_TRANSLATION_REALTIME_EVENT_KIND_V1
            {
                continue;
            }
            let payload = AttachmentTranslationStatusChangedV1::decode(event.payload.as_slice())
                .expect("Attachment Translation realtime payload");
            let state = AttachmentTranslationStateV1::try_from(payload.state)
                .expect("known Attachment Translation realtime state");
            if payload.run_id == run_id
                && matches!(
                    state,
                    AttachmentTranslationStateV1::AttachmentTranslationStateReady
                        | AttachmentTranslationStateV1::AttachmentTranslationStateRejected
                )
            {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before terminal Attachment Translation event");
}
