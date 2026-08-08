//! Live Recording -> event -> STT -> transcript ClientBlob conformance.

use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

use hyper::StatusCode;
use makosh_call_transcription_api::{
    START_CONNECT_PATH_V1, TICKET_CONNECT_PATH_V1, run_id_v1,
    wire::{
        CallTranscriptionErrorCodeV1, CallTranscriptionLanguageV1, CallTranscriptionStateV1,
        IssueCallTranscriptReadRequestV1, IssueCallTranscriptReadResponseV1,
        StartCallTranscriptionRequestV1, StartCallTranscriptionResponseV1,
    },
};
use makosh_desktop_call_recording_api::wire::{
    DesktopCaptureCompletedV1, DesktopCaptureStartedV1, DesktopRecordingHostObservationV1,
    DesktopRecordingStateV1, GetDesktopCallRecordingRequestV1, GetDesktopCallRecordingResponseV1,
    StartDesktopCallRecordingRequestV1, StartDesktopCallRecordingResponseV1,
    StopDesktopCallRecordingRequestV1, StopDesktopCallRecordingResponseV1,
    desktop_recording_host_command_v1::Command, desktop_recording_host_observation_v1::Observation,
};
use makosh_speech_transcript_artifact::{
    validate_speech_transcript_document_v1, wire::SpeechTranscriptDocumentV1,
};

use crate::identity::device::signer::DeviceSigner;

#[test]
#[ignore = "requires disposable Docker plus real managed Recording, STT, Whisper, Vault, Storage, Blob and NATS binaries"]
fn managed_call_transcription_reaches_recording_stt_gateway_blob_and_restarts() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-call-transcription");
    let data = private_directory(short_communications_kernel_data_directory());
    let runtime_dir = private_directory(data.join("r"));
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_call_transcription_ensemble_release_v1(&root);
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
            CALL_TRANSCRIPTION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Call Transcription logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        CALL_TRANSCRIPTION_LOGICAL_OWNER_ID_V1,
    );
    super::super::browser_gateway_session::admit_secondary_browser_test_device(
        &store,
        CALL_TRANSCRIPTION_LOGICAL_OWNER_ID_V1,
    );

    let admitted_recording = admit_desktop_recording_runtime_v1(&store);
    let admitted_workflow = admit_call_transcription_runtime_v1(&store);
    let admitted_engine = admit_speech_to_text_runtime_v1(&store);
    let admitted_provider = admit_whisper_stt_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_speech_to_text_module_request_router_v1(&supervisor, &store);
    configure_call_transcription_realtime_v1(&supervisor, &store, realtime.clone());
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Call Transcription Event credentials");
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
    let admitted_recording =
        prepare_desktop_recording_runtime_v1(&supervisor, &store, admitted_recording);
    let admitted_workflow =
        prepare_call_transcription_runtime_v1(&supervisor, &store, admitted_workflow);
    let admitted_engine = prepare_speech_to_text_runtime_v1(&supervisor, &store, admitted_engine);
    let admitted_provider = prepare_whisper_stt_runtime_v1(&supervisor, &store, admitted_provider);
    configure_communications_jetstream(&store);

    let provider = start_whisper_stt_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_provider,
    );
    let engine = start_speech_to_text_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_engine,
    );
    let workflow =
        start_call_transcription_runtime_v1(&supervisor, &store, &runtime_dir, admitted_workflow);
    let recording = start_desktop_recording_runtime_v1(
        &supervisor,
        &store,
        &data,
        &runtime_dir,
        admitted_recording,
    );
    for registration_id in [
        &provider.registration_id,
        &engine.registration_id,
        &workflow.registration_id,
        &recording.registration_id,
    ] {
        assert!(
            supervisor
                .is_active(registration_id)
                .expect("observe managed Call Transcription ensemble"),
            "managed unit {registration_id} must remain independently active",
        );
    }
    let nats_runtime = tokio::runtime::Runtime::new().expect("Call Transcription NATS observer");
    let nats_client = nats_runtime.block_on(async {
        let endpoint = store
            .platform_event_hub_topology()
            .expect("read Call Transcription Event Hub topology")
            .expect("Call Transcription Event Hub topology")
            .nats_endpoint()
            .to_owned();
        async_nats::connect(endpoint)
            .await
            .expect("connect Call Transcription NATS observer")
    });

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Call Transcription Gateway");
    let router = call_transcription_gateway_v1(&store, &supervisor, &root, &data, realtime);
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    assert_eq!(
        post_call_transcription_proto_status_v1(
            &router,
            &gateway_runtime,
            Some("makosh_session=wrong-actor-session"),
            START_CONNECT_PATH_V1,
            StartCallTranscriptionRequestV1 {
                protocol_major: 1,
                operation_id: vec![0x61; 16],
                call_evidence_id: vec![0x62; 16],
                expected_call_evidence_revision: 7,
                recording_evidence_id: vec![0x63; 16],
                expected_recording_revision: 4,
                consent_receipt_id: vec![0x64; 16],
                consent_policy_revision: 3,
                requested_language: CallTranscriptionLanguageV1::CallTranscriptionLanguageEnglish
                    as i32,
            },
        ),
        StatusCode::UNAUTHORIZED,
    );
    let transcription_sse = open_call_transcription_sse_v1(&router, &gateway_runtime, &cookie);

    let recording_operation_id = vec![0x65; 16];
    let recording_start = post_call_transcription_proto_v1::<_, StartDesktopCallRecordingResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        makosh_desktop_call_recording_runtime::admission::START_PATH_V1,
        StartDesktopCallRecordingRequestV1 {
            operation_id: recording_operation_id.clone(),
            call_evidence_id: vec![0x62; 16],
            expected_call_revision: 7,
            maximum_duration_millis: 60_000,
            consent_policy_revision: 3,
        },
    );
    assert_eq!(
        DesktopRecordingStateV1::try_from(recording_start.state).expect("recording state"),
        DesktopRecordingStateV1::DesktopRecordingStateAwaitingConsentV1,
    );
    let begin_claim_id = [0x66; 16];
    let begin_commands = claim_desktop_recording_commands_v1(&recording, begin_claim_id);
    let begin = match begin_commands
        .first()
        .and_then(|command| command.command.as_ref())
        .expect("recording begin command")
    {
        Command::BeginCapture(begin) => begin.clone(),
        Command::StopCapture(_) => panic!("expected recording begin command"),
    };
    let wav = std::fs::read(required("MAKOSH_WHISPER_STT_TEST_WAV"))
        .expect("read real bounded Call Transcription WAV");
    let wav_duration_millis = canonical_pcm_wav_duration_millis_v1(&wav);
    let started_at = wall_millis_v1() - wav_duration_millis;
    submit_desktop_recording_observation_v1(
        &recording,
        DesktopRecordingHostObservationV1 {
            observation: Some(Observation::CaptureStarted(DesktopCaptureStartedV1 {
                command_id: begin_commands[0].command_id.clone(),
                host_claim_id: begin_claim_id.to_vec(),
                challenge_id: begin.challenge_id.clone(),
                recording_evidence_id: recording_start.recording_evidence_id.clone(),
                started_at_unix_ms: started_at,
                os_permission_revision: 1,
            })),
        },
    );
    post_call_transcription_proto_v1::<_, StopDesktopCallRecordingResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        makosh_desktop_call_recording_runtime::admission::STOP_PATH_V1,
        StopDesktopCallRecordingRequestV1 {
            recording_evidence_id: recording_start.recording_evidence_id.clone(),
        },
    );
    let stop_claim_id = [0x67; 16];
    let stop_commands = claim_desktop_recording_commands_v1(&recording, stop_claim_id);
    assert!(matches!(
        stop_commands
            .first()
            .and_then(|command| command.command.as_ref()),
        Some(Command::StopCapture(_))
    ));
    let audio_sha256: [u8; 32] = Sha256::digest(&wav).into();
    let ended_at = started_at + wav_duration_millis;
    super::nats_outage_fixture::set_authenticated_nats_container_running(false);
    let recording_ready = submit_desktop_recording_observation_v1(
        &recording,
        DesktopRecordingHostObservationV1 {
            observation: Some(Observation::CaptureCompleted(DesktopCaptureCompletedV1 {
                command_id: stop_commands[0].command_id.clone(),
                host_claim_id: stop_claim_id.to_vec(),
                challenge_id: begin.challenge_id,
                recording_evidence_id: recording_start.recording_evidence_id.clone(),
                started_at_unix_ms: started_at,
                ended_at_unix_ms: ended_at,
                canonical_wav_bytes: wav,
                audio_sha256: audio_sha256.to_vec(),
            })),
        },
    );
    assert_eq!(recording_ready.recording_revision, 4);
    let recording_status = post_call_transcription_proto_v1::<_, GetDesktopCallRecordingResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        makosh_desktop_call_recording_runtime::admission::GET_PATH_V1,
        GetDesktopCallRecordingRequestV1 {
            recording_evidence_id: recording_start.recording_evidence_id.clone(),
        },
    );
    let authority = recording_status
        .transcription_authority
        .expect("Ready recording must return typed transcription authority");
    assert_eq!(authority.operation_id, recording_operation_id);
    assert_eq!(
        authority.recording_evidence_id,
        recording_start.recording_evidence_id
    );
    assert_eq!(authority.recording_revision, 4);
    assert!(!authority.consent_receipt_id.is_empty());
    let operation_id: [u8; 16] = authority
        .operation_id
        .as_slice()
        .try_into()
        .expect("recording authority operation id");
    let run_id = run_id_v1(operation_id);

    let started = post_call_transcription_proto_v1::<_, StartCallTranscriptionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        START_CONNECT_PATH_V1,
        StartCallTranscriptionRequestV1 {
            protocol_major: 1,
            operation_id: authority.operation_id.clone(),
            call_evidence_id: authority.call_evidence_id.clone(),
            expected_call_evidence_revision: authority.call_evidence_revision,
            recording_evidence_id: authority.recording_evidence_id.clone(),
            expected_recording_revision: authority.recording_revision,
            consent_receipt_id: authority.consent_receipt_id.clone(),
            consent_policy_revision: authority.consent_policy_revision,
            requested_language: CallTranscriptionLanguageV1::CallTranscriptionLanguageEnglish
                as i32,
        },
    );
    assert_eq!(started.run_id, run_id);
    assert_eq!(
        CallTranscriptionErrorCodeV1::try_from(started.error).expect("start error"),
        CallTranscriptionErrorCodeV1::CallTranscriptionErrorCodeUnspecified,
    );
    assert!(matches!(
        CallTranscriptionStateV1::try_from(started.state).expect("start state"),
        CallTranscriptionStateV1::CallTranscriptionStateAccepted
            | CallTranscriptionStateV1::CallTranscriptionStateAwaitingRecording
    ));

    super::nats_outage_fixture::set_authenticated_nats_container_running(true);
    super::nats_outage_fixture::wait_for_authenticated_nats_reconnect(
        &nats_runtime,
        &nats_client,
        "Call Transcription outage observer",
    );
    assert!(supervisor.is_active(&provider.registration_id).unwrap());

    let terminal_event =
        read_terminal_call_transcription_sse_v1(&gateway_runtime, transcription_sse, &run_id);
    assert!(!terminal_event.cursor.is_empty());
    let ready = get_call_transcription_v1(&router, &gateway_runtime, &cookie, &run_id);
    assert_eq!(
        CallTranscriptionStateV1::try_from(ready.state).expect("terminal state"),
        CallTranscriptionStateV1::CallTranscriptionStateReady,
    );
    let artifact = ready
        .artifact
        .as_ref()
        .expect("terminal transcript artifact");
    assert_eq!(artifact.transcript_sha256.len(), 32);
    assert!(artifact.transcript_size_bytes > 0);
    assert!(artifact.segment_count > 0);

    let ticket = issue_transcript_ticket_v1(&router, &gateway_runtime, &cookie, &run_id);
    let secondary_cookie =
        super::super::browser_gateway_session::authenticate_secondary_gateway_router(
            &router,
            &gateway_runtime,
        );
    let (wrong_actor, wrong_actor_body) = read_call_transcript_blob_v1(
        &router,
        &gateway_runtime,
        Some(&secondary_cookie),
        ticket.opaque_read_ticket.clone(),
    );
    assert_eq!(wrong_actor, StatusCode::NOT_FOUND);
    assert!(wrong_actor_body.is_empty());
    let (read_status, transcript_bytes) = read_call_transcript_blob_v1(
        &router,
        &gateway_runtime,
        Some(&cookie),
        ticket.opaque_read_ticket.clone(),
    );
    assert_eq!(read_status, StatusCode::OK);
    assert_eq!(
        transcript_bytes.len() as u64,
        artifact.transcript_size_bytes
    );
    assert_eq!(
        Sha256::digest(&transcript_bytes).as_slice(),
        artifact.transcript_sha256
    );
    let document = SpeechTranscriptDocumentV1::decode(transcript_bytes.as_slice())
        .expect("decode exact SpeechTranscriptDocumentV1 ClientBlob");
    validate_speech_transcript_document_v1(
        &document,
        artifact.duration_millis,
        makosh_call_transcription_api::MAX_SEGMENTS_V1,
        makosh_call_transcription_api::MAX_TRANSCRIPT_BYTES_V1
            .try_into()
            .expect("bounded transcript bytes"),
    )
    .expect("validate exact transcript document");
    let text = document
        .segments
        .iter()
        .flat_map(|segment| segment.content_utf8.iter().copied())
        .collect::<Vec<_>>();
    assert!(
        std::str::from_utf8(&text)
            .expect("transcript UTF-8")
            .to_ascii_lowercase()
            .contains("makosh")
    );
    assert_eq!(
        read_call_transcript_blob_v1(
            &router,
            &gateway_runtime,
            Some(&cookie),
            ticket.opaque_read_ticket,
        )
        .0,
        StatusCode::NOT_FOUND,
    );

    let outage_ticket = issue_transcript_ticket_v1(&router, &gateway_runtime, &cookie, &run_id);
    supervisor
        .stop(blob_binding::BLOB_PROCESS_ID)
        .expect("stop Blob for transcript read outage");
    assert_eq!(
        read_call_transcript_blob_v1(
            &router,
            &gateway_runtime,
            Some(&cookie),
            outage_ticket.opaque_read_ticket,
        )
        .0,
        StatusCode::SERVICE_UNAVAILABLE,
    );
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("restart Blob after transcript outage"),
        2,
    );

    let stale_ticket = issue_transcript_ticket_v1(&router, &gateway_runtime, &cookie, &run_id);
    let previous_generation = workflow.runtime_generation;
    let workflow =
        restart_call_transcription_runtime_v1(&supervisor, &store, &runtime_dir, workflow);
    assert_eq!(workflow.runtime_generation, previous_generation + 1);
    assert_eq!(
        read_call_transcript_blob_v1(
            &router,
            &gateway_runtime,
            Some(&cookie),
            stale_ticket.opaque_read_ticket,
        )
        .0,
        StatusCode::NOT_FOUND,
    );
    assert_eq!(
        get_call_transcription_v1(&router, &gateway_runtime, &cookie, &run_id),
        ready,
        "owner-local transcript metadata must survive workflow restart",
    );

    drop(nats_client);
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Call Transcription release fixture");
    std::fs::remove_dir_all(data).expect("remove Call Transcription Kernel fixture");
}

fn issue_transcript_ticket_v1(
    router: &CallTranscriptionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> IssueCallTranscriptReadResponseV1 {
    let ticket: IssueCallTranscriptReadResponseV1 = post_call_transcription_proto_v1(
        router,
        runtime,
        cookie,
        TICKET_CONNECT_PATH_V1,
        IssueCallTranscriptReadRequestV1 {
            protocol_major: 1,
            run_id: run_id.to_vec(),
        },
    );
    assert_eq!(
        CallTranscriptionErrorCodeV1::try_from(ticket.error).expect("ticket error"),
        CallTranscriptionErrorCodeV1::CallTranscriptionErrorCodeUnspecified,
    );
    assert_eq!(ticket.opaque_read_ticket.len(), 32);
    ticket
}

fn wall_millis_v1() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock")
        .as_millis()
        .try_into()
        .expect("wall millis")
}

fn canonical_pcm_wav_duration_millis_v1(wav: &[u8]) -> i64 {
    assert!(wav.len() >= 44, "canonical WAV header");
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert_eq!(&wav[36..40], b"data");
    let byte_rate = u32::from_le_bytes(wav[28..32].try_into().expect("WAV byte rate"));
    let data_bytes = u32::from_le_bytes(wav[40..44].try_into().expect("WAV data bytes"));
    assert!(byte_rate > 0 && data_bytes > 0, "bounded WAV duration");
    let numerator = i64::from(data_bytes).saturating_mul(1_000);
    ((numerator + i64::from(byte_rate) - 1) / i64::from(byte_rate)).max(1)
}
