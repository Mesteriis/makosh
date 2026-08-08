use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, StatusCode};
use makosh_gateway_protocol::{
    v1::{
        ClientRealtimeEventV1, ClientRealtimeFrameV1, ClientSystemStatusChangedV1,
        client_realtime_frame_v1::Frame,
    },
    validation::validate_client_realtime_frame,
};
use makosh_gateway_runtime::{
    BrowserRealtimeRouter, BrowserRealtimeSubscriptionSource, ClientRealtimeSubscriptionV1,
    InMemoryBrowserRealtimeSource,
};
use makosh_gateway_session::{BrowserGatewaySessionService, BrowserSession};
use makosh_gateway_session_contract::{
    BrowserAssertionAuthority, BrowserAuthenticationAuthority, BrowserDeviceAuthority,
    BrowserDeviceCredentialV1, BrowserDevicePrincipalV1, ClientSystemComponentIdV1,
    ClientSystemComponentStateV1, ClientSystemComponentStatusProjectionV1, GatewayIdentityFenceV1,
};
use prost::Message;
use std::sync::Arc;
use tokio::sync::broadcast;

struct RealtimeReplaySource {
    sender: broadcast::Sender<ClientRealtimeFrameV1>,
}

impl BrowserRealtimeSubscriptionSource for RealtimeReplaySource {
    fn subscribe(
        &self,
        session: &BrowserSession,
        after_cursor: Option<&str>,
    ) -> Result<ClientRealtimeSubscriptionV1, String> {
        if session.owner_id() != "owner-1" || session.device_id() != "browser-1" {
            return Err("unexpected browser session".to_owned());
        }
        if !matches!(after_cursor, Some("cursor-1") | Some("cursor-2")) && after_cursor.is_some() {
            return Err("unexpected cursor".to_owned());
        }
        ClientRealtimeSubscriptionV1::new(vec![], self.sender.subscribe())
    }
}

#[derive(Clone)]
struct DummyAuthority;

impl BrowserDeviceAuthority for DummyAuthority {
    fn current_identity_fence(&self) -> Result<GatewayIdentityFenceV1, String> {
        GatewayIdentityFenceV1::new("instance", 1, 1)
    }

    fn active_browser_device(&self, _device_id: &str) -> Result<BrowserDevicePrincipalV1, String> {
        BrowserDevicePrincipalV1::new("owner-1", "browser-1")
    }

    fn active_browser_device_by_credential(
        &self,
        _credential_id: &[u8],
    ) -> Result<BrowserDevicePrincipalV1, String> {
        BrowserDevicePrincipalV1::new("owner-1", "browser-1")
    }
}

impl BrowserAssertionAuthority for DummyAuthority {
    fn accept_verified_browser_assertion(
        &self,
        _credential_id: &[u8],
        _sign_count: u32,
        _backup_eligible: bool,
        _backup_state: bool,
    ) -> Result<BrowserDevicePrincipalV1, String> {
        BrowserDevicePrincipalV1::new("owner-1", "browser-1")
    }
}

impl BrowserAuthenticationAuthority for DummyAuthority {
    fn active_browser_credential(
        &self,
        _credential_id: &[u8],
    ) -> Result<BrowserDeviceCredentialV1, String> {
        BrowserDeviceCredentialV1::new(vec![1], vec![2; 65], vec![4; 65], 1, false, false)
    }
}

#[test]
fn live_frames_sends_replay_gap_for_invalid_live_frame_then_disconnects() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let (sender, _receiver) = broadcast::channel(4);
    let source = RealtimeReplaySource {
        sender: sender.clone(),
    };
    let service = Arc::new(
        BrowserGatewaySessionService::new_lan_development(
            DummyAuthority,
            "http://192.168.1.10:9443",
            "owner-1",
            "browser-1",
        )
        .expect("LAN development service"),
    );
    let router = BrowserRealtimeRouter::new(service, source);

    let response = router.route(
        Request::builder()
            .method("GET")
            .uri("/api/realtime/v1/events")
            .header("last-event-id", "cursor-1")
            .body(Full::new(Bytes::new()))
            .expect("realtime request"),
    );
    assert_eq!(response.status(), StatusCode::OK);

    sender
        .send(invalid_frame("bad\ncursor"))
        .expect("invalid live frame");

    let body = runtime
        .block_on(response.into_body().collect())
        .expect("realtime response body")
        .to_bytes();
    let gap = parse_realtime_replay_gaps(&body)
        .into_iter()
        .next()
        .expect("gap frame exists");
    let frame = gap.frame.expect("frame exists");
    let Frame::ReplayGap(frame) = frame else {
        panic!("expected replay gap for invalid live frame");
    };
    assert_eq!(frame.requested_cursor, "cursor-1");
    assert_eq!(frame.reason_code, "invalid_live_frame");
}

#[test]
fn live_frames_sends_replay_gap_on_lag_and_disconnects() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let (sender, _receiver) = broadcast::channel(1);
    let source = RealtimeReplaySource {
        sender: sender.clone(),
    };
    let service = Arc::new(
        BrowserGatewaySessionService::new_lan_development(
            DummyAuthority,
            "http://192.168.1.10:9443",
            "owner-1",
            "browser-1",
        )
        .expect("LAN development service"),
    );
    let router = BrowserRealtimeRouter::new(service, source);

    let response = router.route(
        Request::builder()
            .method("GET")
            .uri("/api/realtime/v1/events")
            .header("last-event-id", "cursor-2")
            .body(Full::new(Bytes::new()))
            .expect("realtime request"),
    );
    assert_eq!(response.status(), StatusCode::OK);

    sender
        .send(valid_frame("cursor-live-1"))
        .expect("first live frame");
    sender
        .send(valid_frame("cursor-live-2"))
        .expect("second live frame causes lag");

    let body = runtime
        .block_on(response.into_body().collect())
        .expect("realtime response body")
        .to_bytes();
    let gap = parse_realtime_replay_gaps(&body)
        .into_iter()
        .next()
        .expect("gap frame exists");
    let frame = gap.frame.expect("frame exists");
    let Frame::ReplayGap(frame) = frame else {
        panic!("expected replay gap for lag");
    };
    assert_eq!(frame.requested_cursor, "cursor-2");
    assert_eq!(frame.reason_code, "live_buffer_overrun");
}

#[test]
fn system_status_transition_reaches_the_shared_gateway_sse_without_bootstrap_polling() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let source = InMemoryBrowserRealtimeSource::new(4).expect("realtime source");
    source.admit_owner("owner-1").expect("admitted owner");
    let service = Arc::new(
        BrowserGatewaySessionService::new_loopback_development(
            DummyAuthority,
            "http://127.0.0.1:5173",
            "owner-1",
            "browser-1",
        )
        .expect("loopback development service"),
    );
    let router = BrowserRealtimeRouter::new(service, source.clone());

    let response = router.route(
        Request::builder()
            .method("GET")
            .uri("/api/realtime/v1/events")
            .body(Full::new(Bytes::new()))
            .expect("realtime request"),
    );
    assert_eq!(response.status(), StatusCode::OK);
    source
        .reconcile_system_status("owner-1", &system_statuses(), 1_000)
        .expect("system status transition");
    source.revoke_owner("owner-1").expect("close owner stream");

    let body = runtime
        .block_on(response.into_body().collect())
        .expect("realtime response body")
        .to_bytes();
    let status_event = parse_realtime_frames(&body)
        .into_iter()
        .find_map(|frame| match frame.frame {
            Some(Frame::Event(event)) if event.contract_name == "makosh.gateway.system-status" => {
                Some(event)
            }
            _ => None,
        })
        .expect("typed system status event");
    assert_eq!(status_event.event_kind, "platform.system_status.changed");
    let payload =
        ClientSystemStatusChangedV1::decode(status_event.payload.as_slice()).expect("payload");
    assert_eq!(payload.revision, 1);
    assert_eq!(payload.statuses.len(), 15);
}

fn valid_frame(cursor: &str) -> ClientRealtimeFrameV1 {
    ClientRealtimeFrameV1 {
        frame: Some(Frame::Event(ClientRealtimeEventV1 {
            event_id: vec![1; 16],
            cursor: cursor.to_owned(),
            contract_name: "makosh.client.status".to_owned(),
            contract_version: 1,
            event_kind: "status_changed".to_owned(),
            occurred_at_unix_millis: 1,
            causation_id: String::new(),
            correlation_id: String::new(),
            trace_id: String::new(),
            payload: vec![2; 32],
        })),
    }
}

fn invalid_frame(cursor: &str) -> ClientRealtimeFrameV1 {
    ClientRealtimeFrameV1 {
        frame: Some(Frame::Event(ClientRealtimeEventV1 {
            event_id: vec![1; 16],
            cursor: cursor.to_owned(),
            contract_name: "makosh.client.status".to_owned(),
            contract_version: 1,
            event_kind: "status_changed".to_owned(),
            occurred_at_unix_millis: 1,
            causation_id: String::new(),
            correlation_id: String::new(),
            trace_id: String::new(),
            payload: vec![2; 32],
        })),
    }
}

fn parse_realtime_replay_gaps(body: &[u8]) -> Vec<ClientRealtimeFrameV1> {
    parse_realtime_frames(body)
        .into_iter()
        .filter(|frame| matches!(frame.frame.as_ref(), Some(Frame::ReplayGap(_))))
        .collect()
}

fn parse_realtime_frames(body: &[u8]) -> Vec<ClientRealtimeFrameV1> {
    let text = std::str::from_utf8(body).expect("SSE response body");
    text.split("\n\n")
        .filter_map(|frame| {
            let data = frame.lines().find_map(|line| line.strip_prefix("data: "))?;
            let frame =
                ClientRealtimeFrameV1::decode(URL_SAFE_NO_PAD.decode(data).ok()?.as_slice())
                    .ok()?;
            validate_client_realtime_frame(&frame)
                .is_ok()
                .then_some(frame)
        })
        .collect()
}

fn system_statuses() -> Vec<ClientSystemComponentStatusProjectionV1> {
    [
        ClientSystemComponentIdV1::Kernel,
        ClientSystemComponentIdV1::ControlStore,
        ClientSystemComponentIdV1::ModuleControlPlane,
        ClientSystemComponentIdV1::Gateway,
        ClientSystemComponentIdV1::Vault,
        ClientSystemComponentIdV1::StorageControl,
        ClientSystemComponentIdV1::Postgresql,
        ClientSystemComponentIdV1::Pgbouncer,
        ClientSystemComponentIdV1::Nats,
        ClientSystemComponentIdV1::EventHub,
        ClientSystemComponentIdV1::Scheduler,
        ClientSystemComponentIdV1::Clock,
        ClientSystemComponentIdV1::Blob,
        ClientSystemComponentIdV1::Telemetry,
        ClientSystemComponentIdV1::Sse,
    ]
    .into_iter()
    .map(|component_id| {
        ClientSystemComponentStatusProjectionV1::new(
            component_id,
            ClientSystemComponentStateV1::Healthy,
            None,
        )
    })
    .collect()
}
