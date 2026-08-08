//! Managed desktop recording through host consent, private Blob, event, Gateway SSE and restart.

use std::time::{Duration, Instant};

use super::*;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_call_transcription_ingress::{
    CONTRACT_MAJOR_V1 as TARGET_CONTRACT_MAJOR_V1, OWNER_ID_V1 as TARGET_OWNER_ID_V1,
    RECORDING_READY_CONTRACT_NAME_V1, wire::RecordingReadyV1,
};
use makosh_desktop_call_recording_api::{
    GET_CONTRACT_NAME_V1, REALTIME_CONTRACT_NAME_V1, START_CONTRACT_NAME_V1, STOP_CONTRACT_NAME_V1,
    contract_reference_v1,
    wire::{
        DesktopCallRecordingStatusChangedV1, DesktopCaptureCompletedV1, DesktopCaptureStartedV1,
        DesktopRecordingHostObservationV1, DesktopRecordingStateV1,
        GetDesktopCallRecordingRequestV1, GetDesktopCallRecordingResponseV1,
        StartDesktopCallRecordingRequestV1, StartDesktopCallRecordingResponseV1,
        StopDesktopCallRecordingRequestV1, StopDesktopCallRecordingResponseV1,
        desktop_recording_host_command_v1::Command,
        desktop_recording_host_observation_v1::Observation,
    },
};
use makosh_events_jetstream::{DurableSubjectV1, StreamKindV1};
use makosh_events_protocol::v1::DurableEnvelopeV1;
use makosh_gateway_protocol::v1::{
    ClientRealtimeEventV1, ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};

use crate::identity::device::signer::DeviceSigner;

type DesktopRecordingGateway = makosh_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS and desktop recording binaries"]
fn managed_desktop_recording_reaches_blob_event_gateway_sse_and_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-desktop-recording");
    let data = private_directory(short_communications_kernel_data_directory());
    let recording_runtime_dir = private_directory(data.join("r"));
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_desktop_recording_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    blob_binding::bind_installed_release(&store, release.kernel())
        .expect("bind signed Blob release");
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            DESKTOP_RECORDING_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim desktop recording logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        DESKTOP_RECORDING_LOGICAL_OWNER_ID_V1,
    );

    let admitted = admit_desktop_recording_runtime_v1(&store);
    let target = DesktopRecordingBlobTargetFixtureV1::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(32).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_desktop_recording_realtime_v1(&supervisor, &store, realtime.clone());
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure desktop recording Event credentials");
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
    let admitted = prepare_desktop_recording_runtime_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    let recording = start_desktop_recording_runtime_v1(
        &supervisor,
        &store,
        &data,
        &recording_runtime_dir,
        admitted,
    );
    assert_eq!(recording.runtime_generation, 1);
    assert!(recording.grant_epoch > 0);

    let nats_runtime = tokio::runtime::Runtime::new().expect("recording NATS observer runtime");
    let _nats_runtime_context = nats_runtime.enter();
    let (nats_client, ready_subject) = nats_runtime.block_on(async {
        let endpoint = store
            .platform_event_hub_topology()
            .expect("read Event Hub topology")
            .expect("Event Hub topology")
            .nats_endpoint()
            .to_owned();
        let client = async_nats::connect(endpoint)
            .await
            .expect("connect recording event observer");
        let subject = DurableSubjectV1::new(
            StreamKindV1::Event,
            TARGET_OWNER_ID_V1,
            RECORDING_READY_CONTRACT_NAME_V1,
            TARGET_CONTRACT_MAJOR_V1,
        )
        .expect("recording-ready subject")
        .as_str();
        (client, subject)
    });

    let gateway_runtime = tokio::runtime::Runtime::new().expect("recording Gateway runtime");
    let router = desktop_recording_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    assert_eq!(
        post_proto_status(
            &router,
            &gateway_runtime,
            "makosh_session=wrong-actor-session",
            makosh_desktop_call_recording_runtime::admission::START_PATH_V1,
            &StartDesktopCallRecordingRequestV1 {
                operation_id: vec![0x40; 16],
                call_evidence_id: vec![0x42; 16],
                expected_call_revision: 7,
                maximum_duration_millis: 10_000,
                consent_policy_revision: 3,
            },
        ),
        StatusCode::UNAUTHORIZED,
    );
    assert_eq!(
        open_recording_sse_status(
            &router,
            &gateway_runtime,
            "makosh_session=wrong-actor-session",
            None,
        ),
        StatusCode::UNAUTHORIZED,
    );
    let sse_response = open_recording_sse(&router, &gateway_runtime, &cookie, None);
    let start_request = StartDesktopCallRecordingRequestV1 {
        operation_id: vec![0x41; 16],
        call_evidence_id: vec![0x42; 16],
        expected_call_revision: 7,
        maximum_duration_millis: 10_000,
        consent_policy_revision: 3,
    };
    let started = post_proto::<_, StartDesktopCallRecordingResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        makosh_desktop_call_recording_runtime::admission::START_PATH_V1,
        start_request.clone(),
    );
    assert_eq!(
        DesktopRecordingStateV1::try_from(started.state).unwrap(),
        DesktopRecordingStateV1::DesktopRecordingStateAwaitingConsentV1
    );
    assert_eq!(started.recording_revision, 1);

    let begin_claim_id = [0x43; 16];
    let commands = claim_desktop_recording_commands_v1(&recording, begin_claim_id);
    assert_eq!(commands.len(), 1);
    let begin = match commands[0].command.as_ref().expect("begin command") {
        Command::BeginCapture(begin) => begin.clone(),
        Command::StopCapture(_) => panic!("expected begin capture command"),
    };
    assert_eq!(begin.recording_evidence_id, started.recording_evidence_id);
    assert_eq!(begin.call_evidence_id, start_request.call_evidence_id);
    assert_eq!(begin.call_evidence_revision, 7);
    assert_eq!(begin.consent_purpose, "call_transcription");
    assert_eq!(begin.canonical_audio_format, "wav_pcm_s16le_mono_16000");
    let started_at = wall_millis();
    let capture_started = DesktopRecordingHostObservationV1 {
        observation: Some(Observation::CaptureStarted(DesktopCaptureStartedV1 {
            command_id: commands[0].command_id.clone(),
            host_claim_id: begin_claim_id.to_vec(),
            challenge_id: begin.challenge_id.clone(),
            recording_evidence_id: begin.recording_evidence_id.clone(),
            started_at_unix_ms: started_at,
            os_permission_revision: 1,
        })),
    };
    let capturing = submit_desktop_recording_observation_v1(&recording, capture_started.clone());
    assert_eq!(capturing.recording_revision, 2);
    assert_eq!(
        submit_desktop_recording_observation_v1(&recording, capture_started).recording_revision,
        2,
        "duplicate native start observation must replay idempotently",
    );

    let stop = post_proto::<_, StopDesktopCallRecordingResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        makosh_desktop_call_recording_runtime::admission::STOP_PATH_V1,
        StopDesktopCallRecordingRequestV1 {
            recording_evidence_id: started.recording_evidence_id.clone(),
        },
    );
    assert_eq!(
        DesktopRecordingStateV1::try_from(stop.state).unwrap(),
        DesktopRecordingStateV1::DesktopRecordingStateCapturingV1
    );
    let stop_claim_id = [0x44; 16];
    let stop_commands = claim_desktop_recording_commands_v1(&recording, stop_claim_id);
    assert_eq!(stop_commands.len(), 1);
    assert!(matches!(
        stop_commands[0].command,
        Some(Command::StopCapture(_))
    ));
    let wav = canonical_recording_wav_v1(160);
    let audio_sha256: [u8; 32] = Sha256::digest(&wav).into();
    std::thread::sleep(Duration::from_millis(15));
    let ended_at = wall_millis().max(started_at + 1);
    super::nats_outage_fixture::set_authenticated_nats_container_running(false);
    let completed = DesktopRecordingHostObservationV1 {
        observation: Some(Observation::CaptureCompleted(DesktopCaptureCompletedV1 {
            command_id: stop_commands[0].command_id.clone(),
            host_claim_id: stop_claim_id.to_vec(),
            challenge_id: begin.challenge_id,
            recording_evidence_id: started.recording_evidence_id.clone(),
            started_at_unix_ms: started_at,
            ended_at_unix_ms: ended_at,
            canonical_wav_bytes: wav.clone(),
            audio_sha256: audio_sha256.to_vec(),
        })),
    };
    let ready = submit_desktop_recording_observation_v1(&recording, completed.clone());
    assert_eq!(ready.recording_revision, 4);
    assert_eq!(
        submit_desktop_recording_observation_v1(&recording, completed.clone()).recording_revision,
        4,
        "duplicate native completion must not duplicate Blob or event",
    );
    let mut malformed = completed;
    malformed
        .observation
        .as_mut()
        .and_then(|observation| match observation {
            Observation::CaptureCompleted(value) => Some(&mut value.canonical_wav_bytes),
            Observation::CaptureStarted(_) | Observation::CaptureRejected(_) => None,
        })
        .expect("completed WAV")
        .pop();
    assert_desktop_recording_observation_rejected_v1(&recording, malformed);
    assert!(
        supervisor
            .is_active(&recording.registration_id)
            .expect("observe recording runtime after malformed WAV"),
        "malformed native audio must fail only its connection",
    );

    let realtime_event = gateway_runtime.block_on(read_recording_sse_event(
        sse_response.into_body(),
        &started.recording_evidence_id,
    ));
    let realtime_payload =
        DesktopCallRecordingStatusChangedV1::decode(realtime_event.payload.as_slice())
            .expect("decode recording realtime payload");
    assert_eq!(realtime_payload.recording_revision, 4);
    assert_eq!(
        DesktopRecordingStateV1::try_from(realtime_payload.state).unwrap(),
        DesktopRecordingStateV1::DesktopRecordingStateReadyV1
    );
    let status = get_recording(
        &router,
        &gateway_runtime,
        &cookie,
        &started.recording_evidence_id,
    );
    assert_eq!(status.recording_revision, 4);
    assert_eq!(status.duration_millis, 10);
    assert!(status.public_error_code.is_empty());
    let replay_gap = open_recording_sse(
        &router,
        &gateway_runtime,
        &cookie,
        Some("missing-desktop-recording-cursor"),
    );
    gateway_runtime.block_on(assert_recording_replay_gap(
        replay_gap.into_body(),
        "missing-desktop-recording-cursor",
    ));

    super::nats_outage_fixture::set_authenticated_nats_container_running(true);
    super::nats_outage_fixture::wait_for_authenticated_nats_reconnect(
        &nats_runtime,
        &nats_client,
        "desktop recording outage observer",
    );

    let exact_event = nats_runtime.block_on(wait_for_recording_ready_event_v1(
        &nats_client,
        &ready_subject,
        &started.recording_evidence_id,
    ));
    let payload = RecordingReadyV1::decode(exact_event.payload.as_slice())
        .expect("decode recording-ready payload");
    assert_eq!(payload.recording_evidence_id, started.recording_evidence_id);
    assert_eq!(payload.audio_sha256, audio_sha256);
    assert_eq!(payload.declared_bytes, u64::try_from(wav.len()).unwrap());
    let transferred = target.accept_and_read(
        &store,
        &supervisor,
        &data,
        &DesktopRecordingReadyBlobV1 {
            reference_id: payload
                .target_blob_reference_id
                .as_slice()
                .try_into()
                .unwrap(),
            receipt_sha256: payload.audio_sha256.as_slice().try_into().unwrap(),
            custody_transfer_source_proof: &payload.custody_transfer_source_proof,
            declared_size: payload.declared_bytes,
            evidence_id: exact_event.message_id.as_slice().try_into().unwrap(),
            evidence_envelope_sha256: Sha256::digest(exact_event.encode_to_vec()).into(),
        },
    );
    assert_eq!(transferred, wav);

    assert_blob_outage_rejects_recording_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &recording,
        &supervisor,
    );
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("restart signed Blob runtime after recording outage"),
        2,
    );

    drop(nats_client);
    let stale_recording = recording.clone();
    let recording = restart_desktop_recording_runtime_v1(
        &supervisor,
        &store,
        &data,
        &recording_runtime_dir,
        recording,
    );
    assert_eq!(recording.runtime_generation, 2);
    assert_stale_desktop_recording_host_route_rejected_v1(&stale_recording, &recording);
    assert!(
        supervisor
            .is_active(&recording.registration_id)
            .expect("observe recording successor after stale host binding"),
        "stale host binding must not stop the successor runtime",
    );
    let replayed = get_recording(
        &router,
        &gateway_runtime,
        &cookie,
        &started.recording_evidence_id,
    );
    assert_eq!(replayed, status);

    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove desktop recording fixture");
    std::fs::remove_dir_all(data).expect("remove desktop recording Kernel fixture");
}

async fn wait_for_recording_ready_event_v1(
    client: &async_nats::Client,
    subject: &str,
    recording_id: &[u8],
) -> DurableEnvelopeV1 {
    let context = async_nats::jetstream::new(client.clone());
    let stream = context
        .get_stream("MAKOSH_EVENT_V1")
        .await
        .expect("read recording event stream");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(raw) = stream.get_last_raw_message_by_subject(subject).await {
            let envelope = DurableEnvelopeV1::decode(raw.payload.as_ref())
                .expect("decode recording-ready envelope");
            let payload = RecordingReadyV1::decode(envelope.payload.as_slice())
                .expect("decode recording-ready event candidate");
            if payload.recording_evidence_id == recording_id {
                return envelope;
            }
        }
        assert!(
            Instant::now() < deadline,
            "recording-ready event was not retained after NATS recovery",
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn assert_blob_outage_rejects_recording_v1(
    router: &DesktopRecordingGateway,
    gateway_runtime: &tokio::runtime::Runtime,
    cookie: &str,
    recording: &StartedDesktopRecordingRuntimeV1,
    supervisor: &ManagedRuntimeSupervisor,
) {
    let sse = open_recording_sse(router, gateway_runtime, cookie, None);
    let start = post_proto::<_, StartDesktopCallRecordingResponseV1>(
        router,
        gateway_runtime,
        cookie,
        makosh_desktop_call_recording_runtime::admission::START_PATH_V1,
        StartDesktopCallRecordingRequestV1 {
            operation_id: vec![0x51; 16],
            call_evidence_id: vec![0x52; 16],
            expected_call_revision: 8,
            maximum_duration_millis: 10_000,
            consent_policy_revision: 3,
        },
    );
    let begin_claim_id = [0x53; 16];
    let begin_commands = claim_desktop_recording_commands_v1(recording, begin_claim_id);
    assert_eq!(begin_commands.len(), 1);
    let begin = match begin_commands[0]
        .command
        .as_ref()
        .expect("outage begin command")
    {
        Command::BeginCapture(begin) => begin.clone(),
        Command::StopCapture(_) => panic!("expected outage begin command"),
    };
    let started_at = wall_millis();
    submit_desktop_recording_observation_v1(
        recording,
        DesktopRecordingHostObservationV1 {
            observation: Some(Observation::CaptureStarted(DesktopCaptureStartedV1 {
                command_id: begin_commands[0].command_id.clone(),
                host_claim_id: begin_claim_id.to_vec(),
                challenge_id: begin.challenge_id.clone(),
                recording_evidence_id: start.recording_evidence_id.clone(),
                started_at_unix_ms: started_at,
                os_permission_revision: 1,
            })),
        },
    );
    post_proto::<_, StopDesktopCallRecordingResponseV1>(
        router,
        gateway_runtime,
        cookie,
        makosh_desktop_call_recording_runtime::admission::STOP_PATH_V1,
        StopDesktopCallRecordingRequestV1 {
            recording_evidence_id: start.recording_evidence_id.clone(),
        },
    );
    let stop_claim_id = [0x54; 16];
    let stop_commands = claim_desktop_recording_commands_v1(recording, stop_claim_id);
    assert_eq!(stop_commands.len(), 1);
    let wav = canonical_recording_wav_v1(160);
    let audio_sha256: [u8; 32] = Sha256::digest(&wav).into();
    std::thread::sleep(Duration::from_millis(15));
    let ended_at = wall_millis().max(started_at + 1);
    supervisor
        .stop(blob_binding::BLOB_PROCESS_ID)
        .expect("stop Blob for desktop recording outage");
    let rejected = submit_desktop_recording_observation_v1(
        recording,
        DesktopRecordingHostObservationV1 {
            observation: Some(Observation::CaptureCompleted(DesktopCaptureCompletedV1 {
                command_id: stop_commands[0].command_id.clone(),
                host_claim_id: stop_claim_id.to_vec(),
                challenge_id: begin.challenge_id,
                recording_evidence_id: start.recording_evidence_id.clone(),
                started_at_unix_ms: started_at,
                ended_at_unix_ms: ended_at,
                canonical_wav_bytes: wav,
                audio_sha256: audio_sha256.to_vec(),
            })),
        },
    );
    assert_eq!(rejected.recording_revision, 4);
    let event = gateway_runtime.block_on(read_recording_sse_event(
        sse.into_body(),
        &start.recording_evidence_id,
    ));
    let payload = DesktopCallRecordingStatusChangedV1::decode(event.payload.as_slice())
        .expect("decode Blob outage recording status");
    assert_eq!(
        DesktopRecordingStateV1::try_from(payload.state).unwrap(),
        DesktopRecordingStateV1::DesktopRecordingStateRejectedV1,
    );
    assert_eq!(payload.public_error_code, "blob_unavailable");
    let status = get_recording(
        router,
        gateway_runtime,
        cookie,
        &start.recording_evidence_id,
    );
    assert_eq!(status.public_error_code, "blob_unavailable");
    assert!(
        supervisor
            .is_active(&recording.registration_id)
            .expect("observe recording runtime during Blob outage"),
        "Blob outage must fail the operation, not stop its owner runtime",
    );
}

fn desktop_recording_gateway(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> DesktopRecordingGateway {
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("desktop-recording-gateway-cert.der"),
        root.join("desktop-recording-gateway-key.der"),
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
    .expect("compose desktop recording Gateway routes")
}

fn post_proto<M, R>(
    router: &DesktopRecordingGateway,
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
                .expect("desktop recording Gateway request"),
        ),
    );
    let status = response.status();
    let bytes = runtime
        .block_on(response.into_body().collect())
        .expect("desktop recording Gateway response")
        .to_bytes();
    assert_eq!(
        status,
        StatusCode::OK,
        "desktop recording Gateway body: {}",
        String::from_utf8_lossy(&bytes)
    );
    R::decode(bytes.as_ref()).expect("decode desktop recording Gateway response")
}

fn post_proto_status<M: Message>(
    router: &DesktopRecordingGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    path: &str,
    message: &M,
) -> StatusCode {
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
                    .expect("desktop recording Gateway status request"),
            ),
        )
        .status()
}

fn get_recording(
    router: &DesktopRecordingGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    recording_id: &[u8],
) -> GetDesktopCallRecordingResponseV1 {
    post_proto::<_, GetDesktopCallRecordingResponseV1>(
        router,
        runtime,
        cookie,
        makosh_desktop_call_recording_runtime::admission::GET_PATH_V1,
        GetDesktopCallRecordingRequestV1 {
            recording_evidence_id: recording_id.to_vec(),
        },
    )
}

fn open_recording_sse(
    router: &DesktopRecordingGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    last_event_id: Option<&str>,
) -> makosh_gateway_runtime::GatewayHttpResponse {
    let mut request = Request::builder()
        .method("GET")
        .uri("/api/realtime/v1/events")
        .header("cookie", cookie);
    if let Some(cursor) = last_event_id {
        request = request.header("last-event-id", cursor);
    }
    let response = runtime.block_on(
        router.route(
            request
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("desktop recording SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    response
}

fn open_recording_sse_status(
    router: &DesktopRecordingGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    last_event_id: Option<&str>,
) -> StatusCode {
    let mut request = Request::builder()
        .method("GET")
        .uri("/api/realtime/v1/events")
        .header("cookie", cookie);
    if let Some(cursor) = last_event_id {
        request = request.header("last-event-id", cursor);
    }
    runtime
        .block_on(
            router.route(
                request
                    .body(http_body_util::Full::new(Bytes::new()))
                    .expect("desktop recording SSE status request"),
            ),
        )
        .status()
}

async fn assert_recording_replay_gap<B>(mut body: B, requested_cursor: &str)
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    tokio::time::timeout(Duration::from_secs(3), async {
        while let Some(frame) = body.frame().await {
            let frame = frame.expect("desktop recording replay-gap SSE frame");
            let Ok(data) = frame.into_data() else {
                continue;
            };
            pending.extend_from_slice(&data);
            while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
                let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
                let text = std::str::from_utf8(&block).expect("recording replay-gap UTF-8");
                let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: "))
                else {
                    continue;
                };
                let bytes = URL_SAFE_NO_PAD
                    .decode(encoded)
                    .expect("decode recording replay-gap frame");
                let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                    .expect("decode recording replay-gap payload");
                let Some(RealtimeFrame::ReplayGap(gap)) = frame.frame else {
                    continue;
                };
                assert_eq!(gap.requested_cursor, requested_cursor);
                assert_eq!(gap.reason_code, "cursor_not_available");
                return;
            }
        }
        panic!("Gateway SSE closed before desktop recording replay gap");
    })
    .await
    .expect("desktop recording replay-gap timeout");
}

async fn read_recording_sse_event<B>(mut body: B, recording_id: &[u8]) -> ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(frame) = body.frame().await {
            let frame = frame.expect("desktop recording SSE frame");
            let Ok(data) = frame.into_data() else {
                continue;
            };
            pending.extend_from_slice(&data);
            while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
                let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
                let text = std::str::from_utf8(&block).expect("desktop recording SSE UTF-8");
                let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: "))
                else {
                    continue;
                };
                let bytes = URL_SAFE_NO_PAD
                    .decode(encoded)
                    .expect("decode recording SSE frame");
                let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                    .expect("decode recording realtime frame");
                let Some(RealtimeFrame::Event(event)) = frame.frame else {
                    continue;
                };
                if event.contract_name != REALTIME_CONTRACT_NAME_V1
                    || event.event_kind != REALTIME_CONTRACT_NAME_V1
                {
                    continue;
                }
                let payload = DesktopCallRecordingStatusChangedV1::decode(event.payload.as_slice())
                    .expect("decode recording status change");
                if payload.recording_evidence_id == recording_id
                    && matches!(
                        DesktopRecordingStateV1::try_from(payload.state).unwrap(),
                        DesktopRecordingStateV1::DesktopRecordingStateReadyV1
                            | DesktopRecordingStateV1::DesktopRecordingStateRejectedV1
                    )
                {
                    return event;
                }
            }
        }
        panic!("Gateway SSE closed before terminal desktop recording event");
    })
    .await
    .expect("desktop recording SSE timeout")
}

fn wall_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock")
        .as_millis()
        .try_into()
        .expect("wall millis")
}

#[test]
fn desktop_recording_contract_references_remain_distinct() {
    assert_ne!(
        contract_reference_v1(START_CONTRACT_NAME_V1),
        contract_reference_v1(STOP_CONTRACT_NAME_V1)
    );
    assert_ne!(
        contract_reference_v1(STOP_CONTRACT_NAME_V1),
        contract_reference_v1(GET_CONTRACT_NAME_V1)
    );
}
