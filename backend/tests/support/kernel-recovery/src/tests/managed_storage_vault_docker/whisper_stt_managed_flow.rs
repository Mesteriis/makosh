//! Live managed Whisper transcription, Blob custody, restart and owner-fence conformance.

use super::*;

use std::os::unix::fs::PermissionsExt;

use makosh_runtime_protocol::v1::{
    BlobCustodySourceProofV1, ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeModuleRequestDeliveryV1, ManagedRuntimeModuleRequestResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_speech_to_text_api::{
    SPEECH_TO_TEXT_MODULE_ID_V1, SPEECH_TO_TEXT_OWNER_V1, seal_speech_to_text_request_v1,
    speech_to_text_contract_reference_v1, validate_speech_to_text_result_v1,
    wire::{
        SpeechAudioFormatV1, SpeechAudioSourceReceiptV1, SpeechLanguageV1,
        SpeechToTextRejectCodeV1, SpeechToTextRequestV1, SpeechToTextResultV1,
        SpeechToTextTerminalStatusV1,
    },
};
use makosh_speech_to_text_runtime::SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1;
use makosh_speech_transcript_artifact::{
    validate_speech_transcript_document_v1, wire::SpeechTranscriptDocumentV1,
};

const TASK9_PRIVATE_AUDIO_SENTINEL_V1: &[u8] = b"task9-private-audio-sentinel-v1";
const TASK9_RAW_PROVIDER_SENTINEL_V1: &[u8] = b"task9-raw-provider-sentinel-v1";

#[test]
#[ignore = "requires disposable Docker plus actual Speech-to-Text, Whisper, Vault and Storage binaries"]
fn managed_speech_to_text_whisper_bootstrap_fails_closed_and_stops_promptly() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-speech-whisper-bootstrap-negative");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_whisper_stt_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim Speech-to-Text and Whisper bootstrap owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_engine = admit_speech_to_text_runtime_v1(&store);
    let admitted_provider = admit_whisper_stt_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted_provider = prepare_whisper_stt_runtime_v1(&supervisor, &store, admitted_provider);
    let admitted_engine = prepare_speech_to_text_runtime_v1(&supervisor, &store, admitted_engine);
    let runtime = root.join("runtime");
    assert_speech_to_text_bootstrap_matrix_v1(
        &supervisor,
        &store,
        &runtime,
        &root,
        admitted_engine,
    );
    start_vault(&supervisor, &store, &data, release.kernel());
    let provider = assert_whisper_stt_bootstrap_matrix_v1(
        &supervisor,
        &store,
        &data,
        &runtime,
        &root,
        admitted_provider,
    );
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_whisper_stt_runtime_artifacts_denied_v1(
        &supervisor,
        &store,
        &data,
        &runtime,
        &root,
        provider,
    );
    cleanup_task9_bootstrap_v1(supervisor, shutdown, root, data);
}

fn assert_speech_to_text_bootstrap_matrix_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime: &Path,
    root: &Path,
    admitted: AdmittedSpeechToTextRuntimeV1,
) {
    let capture = start_task9_child_capture_v1(root, "speech-missing-settings");
    let started = launch_speech_to_text_runtime_without_ready_v1(
        supervisor,
        store,
        runtime,
        admitted,
        SpeechToTextBootstrapOverrideV1::MissingSettings,
        &capture,
    );
    assert_task9_pre_spawn_denied_v1(
        supervisor,
        &started.registration_id,
        "Speech-to-Text missing settings",
        &capture,
    );

    let capture = start_task9_child_capture_v1(root, "speech-missing-storage");
    let started = retry_speech_to_text_runtime_without_ready_v1(
        supervisor,
        store,
        runtime,
        started,
        SpeechToTextBootstrapOverrideV1::MissingStorage,
        &capture,
    );
    assert_task9_pre_spawn_denied_v1(
        supervisor,
        &started.registration_id,
        "Speech-to-Text missing Storage",
        &capture,
    );

    let capture = start_task9_child_capture_v1(root, "speech-drifted-settings");
    let started = retry_speech_to_text_runtime_without_ready_v1(
        supervisor,
        store,
        runtime,
        started,
        SpeechToTextBootstrapOverrideV1::DriftedSettingsRevision,
        &capture,
    );
    assert_task9_bounded_runtime_denied_v1(
        supervisor,
        &started.registration_id,
        "Speech-to-Text drifted settings",
        &capture,
    );

    let healthy = retry_speech_to_text_runtime_v1(supervisor, store, runtime, started);
    supervisor
        .stop(&healthy.registration_id)
        .expect("stop healthy Speech-to-Text predecessor");
    let capture = start_task9_child_capture_v1(root, "speech-stale-storage");
    let started = retry_speech_to_text_runtime_without_ready_v1(
        supervisor,
        store,
        runtime,
        healthy,
        SpeechToTextBootstrapOverrideV1::StaleStorageFence,
        &capture,
    );
    assert_task9_active_until_requested_stop_v1(
        supervisor,
        &started.registration_id,
        "Speech-to-Text stale Storage fence",
        &capture,
    );

    let healthy = retry_speech_to_text_runtime_v1(supervisor, store, runtime, started);
    supervisor
        .stop(&healthy.registration_id)
        .expect("stop healthy Speech-to-Text Vault predecessor");
    let capture = start_task9_child_capture_v1(root, "speech-vault-lease");
    let started = retry_speech_to_text_runtime_without_ready_v1(
        supervisor,
        store,
        runtime,
        healthy,
        SpeechToTextBootstrapOverrideV1::StopVaultAfterConfiguration,
        &capture,
    );
    assert_task9_active_until_requested_stop_v1(
        supervisor,
        &started.registration_id,
        "Speech-to-Text Vault lease",
        &capture,
    );
}

fn assert_whisper_stt_bootstrap_matrix_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    data: &Path,
    runtime: &Path,
    root: &Path,
    admitted: AdmittedWhisperSttRuntimeV1,
) -> StartedWhisperSttRuntimeV1 {
    let capture = start_task9_child_capture_v1(root, "whisper-missing-settings");
    let started = launch_whisper_stt_runtime_without_ready_v1(
        supervisor,
        store,
        data,
        runtime,
        admitted,
        WhisperSttBootstrapOverrideV1::MissingSettings,
        &capture,
    );
    assert_task9_pre_spawn_denied_v1(
        supervisor,
        &started.registration_id,
        "Whisper STT missing settings",
        &capture,
    );

    let capture = start_task9_child_capture_v1(root, "whisper-missing-storage");
    let started = retry_whisper_stt_runtime_without_ready_v1(
        supervisor,
        store,
        data,
        runtime,
        started,
        WhisperSttBootstrapOverrideV1::MissingStorage,
        &capture,
    );
    assert_task9_pre_spawn_denied_v1(
        supervisor,
        &started.registration_id,
        "Whisper STT missing Storage",
        &capture,
    );

    let capture = start_task9_child_capture_v1(root, "whisper-drifted-settings");
    let started = retry_whisper_stt_runtime_without_ready_v1(
        supervisor,
        store,
        data,
        runtime,
        started,
        WhisperSttBootstrapOverrideV1::DriftedSettingsTarget,
        &capture,
    );
    assert_task9_bounded_runtime_denied_v1(
        supervisor,
        &started.registration_id,
        "Whisper STT drifted settings",
        &capture,
    );

    let healthy = retry_whisper_stt_runtime_v1(supervisor, store, data, runtime, started);
    supervisor
        .stop(&healthy.registration_id)
        .expect("stop healthy Whisper STT predecessor");
    let capture = start_task9_child_capture_v1(root, "whisper-stale-storage");
    let started = retry_whisper_stt_runtime_without_ready_v1(
        supervisor,
        store,
        data,
        runtime,
        healthy,
        WhisperSttBootstrapOverrideV1::StaleStorageFence,
        &capture,
    );
    assert_task9_active_until_requested_stop_v1(
        supervisor,
        &started.registration_id,
        "Whisper STT stale Storage fence",
        &capture,
    );

    let healthy = retry_whisper_stt_runtime_v1(supervisor, store, data, runtime, started);
    supervisor
        .stop(&healthy.registration_id)
        .expect("stop healthy Whisper STT Vault predecessor");
    let capture = start_task9_child_capture_v1(root, "whisper-vault-lease");
    let started = retry_whisper_stt_runtime_without_ready_v1(
        supervisor,
        store,
        data,
        runtime,
        healthy,
        WhisperSttBootstrapOverrideV1::StopVaultAfterConfiguration,
        &capture,
    );
    assert_task9_active_until_requested_stop_v1(
        supervisor,
        &started.registration_id,
        "Whisper STT Vault lease",
        &capture,
    );
    started
}

fn assert_whisper_stt_runtime_artifacts_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    data: &Path,
    runtime: &Path,
    root: &Path,
    predecessor: StartedWhisperSttRuntimeV1,
) {
    std::fs::remove_file(installed_whisper_stt_model_path_v1(root))
        .expect("remove signed Whisper model fixture");
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(installed_whisper_stt_runner_path_v1(root))
        .expect("open signed Whisper runner fixture")
        .write_all(b"drift")
        .expect("drift signed Whisper runner fixture");
    let capture = start_task9_child_capture_v1(root, "whisper-missing-drifted-artifacts");
    let started = retry_whisper_stt_runtime_without_ready_v1(
        supervisor,
        store,
        data,
        runtime,
        predecessor,
        WhisperSttBootstrapOverrideV1::MissingOrDriftedRuntimeArtifact,
        &capture,
    );
    assert_task9_pre_spawn_denied_v1(
        supervisor,
        &started.registration_id,
        "Whisper STT missing model and drifted runner",
        &capture,
    );
}

fn start_task9_child_capture_v1(root: &Path, phase: &str) -> PathBuf {
    private_directory(root.join(format!("stdio-{phase}")))
}

fn task9_child_capture_paths_v1(directory: &Path) -> Vec<PathBuf> {
    let mut paths = std::fs::read_dir(directory)
        .expect("read Task9 child capture directory")
        .map(|entry| entry.expect("read Task9 child capture entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn clear_task9_child_capture_v1() {
    unsafe {
        std::env::remove_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
        );
    }
}

fn assert_task9_pre_spawn_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    phase: &str,
    capture: &Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_millis(750);
    while std::time::Instant::now() < deadline {
        assert!(
            !matches!(supervisor.relay_port().is_ready(registration_id), Ok(true)),
            "{phase} must not signal Ready"
        );
        if !supervisor
            .is_active(registration_id)
            .expect("Task9 bootstrap activity")
        {
            assert!(
                task9_child_capture_paths_v1(capture).is_empty(),
                "{phase} must be denied before child spawn"
            );
            clear_task9_child_capture_v1();
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("{phase} pre-spawn denial remained active");
}

fn assert_task9_bounded_runtime_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    phase: &str,
    capture: &Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_millis(750);
    while std::time::Instant::now() < deadline {
        assert!(
            !matches!(supervisor.relay_port().is_ready(registration_id), Ok(true)),
            "{phase} must not signal Ready"
        );
        if !supervisor
            .is_active(registration_id)
            .expect("Task9 bounded runtime activity")
        {
            let captures = task9_child_capture_paths_v1(capture);
            assert!(
                captures.len() >= 2 && captures.len().is_multiple_of(2),
                "{phase} must be a captured bounded denial"
            );
            clear_task9_child_capture_v1();
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("{phase} runtime denial did not settle");
}

fn assert_task9_active_until_requested_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    phase: &str,
    capture: &Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_millis(100);
    while std::time::Instant::now() < deadline {
        assert!(
            supervisor
                .is_active(registration_id)
                .expect("Task9 active bootstrap child"),
            "{phase} child exited before requested stop"
        );
        assert!(
            !matches!(supervisor.relay_port().is_ready(registration_id), Ok(true)),
            "{phase} must not signal Ready"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let stopped_at = std::time::Instant::now();
    assert!(
        supervisor
            .request_stop_if_active(registration_id)
            .expect("request Task9 bootstrap stop"),
        "{phase} must own the active child"
    );
    assert!(
        supervisor
            .stop_if_active(registration_id)
            .expect("join Task9 bootstrap child"),
        "{phase} child must join"
    );
    assert!(stopped_at.elapsed() < Duration::from_secs(2));
    assert!(
        !supervisor
            .is_active(registration_id)
            .expect("Task9 stopped bootstrap activity"),
        "{phase} must not install a replacement"
    );
    assert_eq!(
        task9_child_capture_paths_v1(capture).len(),
        2,
        "{phase} must spawn exactly one child attempt"
    );
    clear_task9_child_capture_v1();
}

fn cleanup_task9_bootstrap_v1(
    supervisor: ManagedRuntimeSupervisor,
    shutdown: Arc<AtomicBool>,
    root: PathBuf,
    data: PathBuf,
) {
    supervisor
        .shutdown()
        .expect("stop Task9 bootstrap processes");
    shutdown.store(true, Ordering::SeqCst);
    clear_task9_child_capture_v1();
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Task9 bootstrap root");
    std::fs::remove_dir_all(data).expect("remove Task9 bootstrap data");
}

#[test]
#[ignore = "requires disposable Docker plus actual Speech-to-Text, Whisper, Vault, Storage and Blob binaries"]
fn managed_speech_to_text_whisper_private_surfaces_reject_malformed_provider_output() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-whisper-stt-private-surfaces");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let malformed_runner = task9_malformed_whisper_runner_v1(&root);
    let release = installed_whisper_stt_release_from_paths_v1(
        &root,
        &binary("MAKOSH_WHISPER_STT_MODEL"),
        &malformed_runner,
    );
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    blob_binding::bind_installed_release(&store, release.kernel())
        .expect("bind signed Blob release");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            WHISPER_STT_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim private Speech-to-Text owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_engine = admit_speech_to_text_runtime_v1(&store);
    let admitted_provider = admit_whisper_stt_runtime_v1(&store);
    let source = WhisperSttBlobSourceFixtureV1::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    configure_speech_to_text_module_request_router_v1(&supervisor, &store);
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
    let admitted_provider = prepare_whisper_stt_runtime_v1(&supervisor, &store, admitted_provider);
    let admitted_engine = prepare_speech_to_text_runtime_v1(&supervisor, &store, admitted_engine);

    let whisper_capture = start_task9_child_capture_v1(&root, "whisper-private");
    let provider = launch_whisper_stt_runtime_without_ready_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_provider,
        WhisperSttBootstrapOverrideV1::None,
        &whisper_capture,
    );
    supervisor
        .wait_until_ready(&provider.registration_id)
        .expect("private Whisper STT readiness");
    clear_task9_child_capture_v1();

    let speech_capture = start_task9_child_capture_v1(&root, "speech-private");
    let engine = launch_speech_to_text_runtime_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_engine,
        SpeechToTextBootstrapOverrideV1::None,
        &speech_capture,
    );
    supervisor
        .wait_until_ready(&engine.registration_id)
        .expect("private Speech-to-Text readiness");
    clear_task9_child_capture_v1();

    let mut audio = std::fs::read(required("MAKOSH_WHISPER_STT_TEST_WAV"))
        .expect("read bounded Whisper STT WAV fixture");
    audio.extend_from_slice(TASK9_PRIVATE_AUDIO_SENTINEL_V1);
    let source_blob = source.write_audio(&store, &supervisor, &data, [0xa1; 16], &audio);
    let mut request = speech_request_v1(&source_blob);
    request.request_id = vec![0xa2; 16];
    request = seal_speech_to_text_request_v1(request).expect("reseal private request");
    let first = deliver_speech_to_text_request_v1(
        &supervisor,
        &engine.registration_id,
        SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1,
        &request,
    );
    assert!(first.error_code.is_empty());
    let result = SpeechToTextResultV1::decode(first.response_payload.as_slice())
        .expect("typed malformed-provider result");
    validate_speech_to_text_result_v1(&request, &result)
        .expect("valid bounded malformed-provider result");
    assert_eq!(
        result.terminal_status,
        SpeechToTextTerminalStatusV1::Rejected as i32
    );
    assert_eq!(
        result.reject_code,
        SpeechToTextRejectCodeV1::ProviderRejected as i32
    );

    std::fs::remove_file(installed_whisper_stt_runner_path_v1(&root))
        .expect("remove malformed runner before exact replay");
    let replayed = deliver_speech_to_text_request_v1(
        &supervisor,
        &engine.registration_id,
        SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1,
        &request,
    );
    assert_eq!(
        replayed, first,
        "exact replay must not execute native provider"
    );

    let speech_storage_credential = runtime_storage_credential_for_registration_v1(
        &supervisor,
        &store,
        &data,
        &engine.registration_id,
        makosh_speech_to_text_runtime::SPEECH_TO_TEXT_STORAGE_CAPABILITY_ID_V1,
    );
    let whisper_storage_credential = runtime_storage_credential_for_registration_v1(
        &supervisor,
        &store,
        &data,
        &provider.registration_id,
        makosh_whisper_stt_runtime::WHISPER_STT_STORAGE_CAPABILITY_ID_V1,
    );
    let credentials = [
        &speech_storage_credential[..],
        &whisper_storage_credential[..],
    ];
    assert_task9_private_surfaces_v1(
        &first.response_payload,
        &format!("{:?}", supervisor.last_failure(&engine.registration_id)),
        &format!("{:?}", supervisor.last_failure(&provider.registration_id)),
        &speech_capture,
        &whisper_capture,
        &credentials,
    );

    supervisor
        .shutdown()
        .expect("stop private managed processes");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove private Whisper STT fixture");
    std::fs::remove_dir_all(data).expect("remove private Whisper STT kernel data fixture");
}

fn task9_malformed_whisper_runner_v1(root: &Path) -> PathBuf {
    let runner = root.join("task9-malformed-whisper-runner.sh");
    std::fs::create_dir_all(root).expect("create malformed Whisper runner parent");
    std::fs::write(
        &runner,
        concat!(
            "#!/bin/sh\n",
            "output=''\n",
            "while [ \"$#\" -gt 0 ]; do\n",
            "  if [ \"$1\" = \"--output-file\" ]; then shift; output=$1; fi\n",
            "  shift\n",
            "done\n",
            "printf '%s' 'task9-raw-provider-sentinel-v1' >&2\n",
            "printf '%s' 'task9-raw-provider-sentinel-v1' > \"${output}.json\"\n",
        ),
    )
    .expect("write malformed Whisper runner");
    std::fs::set_permissions(&runner, std::fs::Permissions::from_mode(0o500))
        .expect("make malformed Whisper runner executable");
    runner
}

fn assert_task9_private_surfaces_v1(
    terminal: &[u8],
    speech_diagnostic: &str,
    whisper_diagnostic: &str,
    speech_capture: &Path,
    whisper_capture: &Path,
    credentials: &[&[u8]; 2],
) {
    for private in [
        TASK9_PRIVATE_AUDIO_SENTINEL_V1,
        TASK9_RAW_PROVIDER_SENTINEL_V1,
    ] {
        assert_task9_private_bytes_absent_v1(terminal, private, "typed Speech-to-Text terminal");
        assert_task9_private_bytes_absent_v1(
            speech_diagnostic.as_bytes(),
            private,
            "Speech-to-Text supervisor diagnostic",
        );
        assert_task9_private_bytes_absent_v1(
            whisper_diagnostic.as_bytes(),
            private,
            "Whisper STT supervisor diagnostic",
        );
    }
    for credential in credentials {
        assert_task9_private_bytes_absent_v1(terminal, credential, "typed Speech-to-Text terminal");
        assert_task9_private_bytes_absent_v1(
            speech_diagnostic.as_bytes(),
            credential,
            "Speech-to-Text supervisor diagnostic",
        );
        assert_task9_private_bytes_absent_v1(
            whisper_diagnostic.as_bytes(),
            credential,
            "Whisper STT supervisor diagnostic",
        );
    }
    assert_supervised_speech_to_text_child_output_is_private_v1(speech_capture, credentials);
    assert_supervised_whisper_stt_child_output_is_private_v1(whisper_capture, credentials);
}

fn assert_supervised_speech_to_text_child_output_is_private_v1(
    directory: &Path,
    credentials: &[&[u8]; 2],
) {
    assert_task9_supervised_child_output_is_private_v1(
        directory,
        credentials,
        "supervised Speech-to-Text child output",
    );
}

fn assert_supervised_whisper_stt_child_output_is_private_v1(
    directory: &Path,
    credentials: &[&[u8]; 2],
) {
    assert_task9_supervised_child_output_is_private_v1(
        directory,
        credentials,
        "supervised Whisper STT child output",
    );
}

fn assert_task9_supervised_child_output_is_private_v1(
    directory: &Path,
    credentials: &[&[u8]; 2],
    surface: &str,
) {
    let captures = task9_child_capture_paths_v1(directory);
    assert_eq!(
        captures.len(),
        2,
        "{surface} needs exact stdout/stderr sinks"
    );
    for capture in captures {
        let bytes = std::fs::read(capture).expect("read supervised Task9 child output");
        for private in [
            TASK9_PRIVATE_AUDIO_SENTINEL_V1,
            TASK9_RAW_PROVIDER_SENTINEL_V1,
        ] {
            assert_task9_private_bytes_absent_v1(&bytes, private, surface);
        }
        for credential in credentials {
            assert_task9_private_bytes_absent_v1(&bytes, credential, surface);
        }
    }
}

fn assert_task9_private_bytes_absent_v1(surface: &[u8], private: &[u8], name: &str) {
    assert!(!private.is_empty());
    assert!(
        !surface.windows(private.len()).any(|value| value == private),
        "{name} exposed private material"
    );
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob and pinned Whisper binaries"]
fn managed_speech_to_text_routes_whisper_private_blob_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-whisper-stt");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_whisper_stt_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    blob_binding::bind_installed_release(&store, release.kernel())
        .expect("bind signed Blob release");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            WHISPER_STT_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim Whisper STT logical owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_engine = admit_speech_to_text_runtime_v1(&store);
    let admitted_provider = admit_whisper_stt_runtime_v1(&store);
    let source = WhisperSttBlobSourceFixtureV1::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    configure_speech_to_text_module_request_router_v1(&supervisor, &store);
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
    let admitted_provider = prepare_whisper_stt_runtime_v1(&supervisor, &store, admitted_provider);
    let admitted_engine = prepare_speech_to_text_runtime_v1(&supervisor, &store, admitted_engine);
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
    assert_eq!(provider.runtime_generation, 1);
    assert!(provider.grant_epoch > 0);
    assert_eq!(engine.runtime_generation, 1);
    let audio = std::fs::read(required("MAKOSH_WHISPER_STT_TEST_WAV"))
        .expect("read bounded Whisper STT WAV fixture");
    let source_blob = source.write_audio(&store, &supervisor, &data, [0x91; 16], &audio);
    let source_proof =
        BlobCustodySourceProofV1::decode(source_blob.custody_transfer_source_proof.as_slice())
            .expect("decode source custody proof");
    assert_eq!(source_proof.reference_id, source_blob.reference_id);
    assert_eq!(source_proof.target_owner_id, SPEECH_TO_TEXT_OWNER_V1);
    assert_eq!(source_proof.target_module_id, SPEECH_TO_TEXT_MODULE_ID_V1);
    assert_eq!(
        source_proof.target_capability_id,
        SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1
    );
    let request = speech_request_v1(&source_blob);
    let first = deliver_speech_to_text_request_v1(
        &supervisor,
        &engine.registration_id,
        SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1,
        &request,
    );
    assert!(
        first.error_code.is_empty(),
        "managed Speech-to-Text delivery failed with {}",
        first.error_code
    );
    let first_result = SpeechToTextResultV1::decode(first.response_payload.as_slice())
        .expect("typed Whisper STT result");
    validate_speech_to_text_result_v1(&request, &first_result).expect("valid Whisper STT result");
    assert_eq!(
        first_result.terminal_status,
        SpeechToTextTerminalStatusV1::Ready as i32,
        "managed Speech-to-Text rejected the request with code {}",
        first_result.reject_code,
    );
    let transcript = first_result
        .transcript
        .as_ref()
        .expect("Whisper transcript receipt");
    let transcript_blob = WhisperSttFixtureBlobV1 {
        reference_id: transcript
            .reference_id
            .as_slice()
            .try_into()
            .expect("transcript reference id"),
        receipt_sha256: transcript
            .sha256
            .as_slice()
            .try_into()
            .expect("transcript digest"),
        custody_transfer_source_proof: transcript.custody_transfer_source_proof.clone(),
        declared_size: transcript.declared_bytes,
    };
    let document_bytes = source.read_transcript(&store, &supervisor, &data, &transcript_blob);
    assert_eq!(
        Sha256::digest(&document_bytes).as_slice(),
        transcript.sha256
    );
    let document = SpeechTranscriptDocumentV1::decode(document_bytes.as_slice())
        .expect("decode private transcript document");
    validate_speech_transcript_document_v1(
        &document,
        request.duration_millis,
        request.maximum_segments,
        request.maximum_transcript_bytes,
    )
    .expect("validate private transcript document");
    let text = document
        .segments
        .iter()
        .flat_map(|segment| segment.content_utf8.iter().copied())
        .collect::<Vec<_>>();
    let text = std::str::from_utf8(&text).expect("private transcript utf8");
    assert!(text.to_ascii_lowercase().contains("makosh"));

    let previous_provider_generation = provider.runtime_generation;
    let previous_engine_generation = engine.runtime_generation;
    let provider =
        restart_whisper_stt_runtime_v1(&supervisor, &store, &data, &root.join("runtime"), provider);
    let engine =
        restart_speech_to_text_runtime_v1(&supervisor, &store, &root.join("runtime"), engine);
    assert_eq!(
        provider.runtime_generation,
        previous_provider_generation + 1
    );
    assert_eq!(engine.runtime_generation, previous_engine_generation + 1);
    let replayed = deliver_speech_to_text_request_v1(
        &supervisor,
        &engine.registration_id,
        SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1,
        &request,
    );
    assert!(replayed.error_code.is_empty());
    let replayed_result = SpeechToTextResultV1::decode(replayed.response_payload.as_slice())
        .expect("typed replayed Whisper STT result");
    assert_eq!(replayed_result.request_id, first_result.request_id);
    assert_eq!(replayed_result.request_digest, first_result.request_digest);
    let replayed_transcript = replayed_result
        .transcript
        .as_ref()
        .expect("replayed Whisper transcript receipt");
    assert_eq!(replayed_transcript.reference_id, transcript.reference_id);
    assert_eq!(replayed_transcript.sha256, transcript.sha256);
    assert_eq!(
        replayed_transcript.declared_bytes,
        transcript.declared_bytes
    );
    let replayed_blob = WhisperSttFixtureBlobV1 {
        reference_id: replayed_transcript
            .reference_id
            .as_slice()
            .try_into()
            .expect("replayed transcript reference id"),
        receipt_sha256: replayed_transcript
            .sha256
            .as_slice()
            .try_into()
            .expect("replayed transcript digest"),
        custody_transfer_source_proof: replayed_transcript.custody_transfer_source_proof.clone(),
        declared_size: replayed_transcript.declared_bytes,
    };
    assert_eq!(
        source.read_transcript(&store, &supervisor, &data, &replayed_blob),
        document_bytes
    );

    let wrong_owner = deliver_speech_to_text_request_v1(
        &supervisor,
        &engine.registration_id,
        "owner-2",
        &request,
    );
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());

    let mut conflicting = request.clone();
    conflicting.maximum_segments += 1;
    let conflicting = seal_speech_to_text_request_v1(conflicting).expect("seal conflict");
    let rejected = deliver_speech_to_text_request_v1(
        &supervisor,
        &engine.registration_id,
        SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1,
        &conflicting,
    );
    assert_eq!(rejected.error_code, "REJECTED");
    assert!(rejected.response_payload.is_empty());

    assert_speech_to_text_whisper_owner_rls_v1("makosh_storage_authenticated");

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Whisper STT fixture");
    std::fs::remove_dir_all(data).expect("remove short Whisper STT kernel data fixture");
}

fn speech_request_v1(blob: &WhisperSttFixtureBlobV1) -> SpeechToTextRequestV1 {
    seal_speech_to_text_request_v1(SpeechToTextRequestV1 {
        protocol_major: 0,
        request_id: vec![0x92; 16],
        logical_owner_id: WHISPER_STT_LOGICAL_OWNER_ID_V1.to_owned(),
        source: Some(SpeechAudioSourceReceiptV1 {
            reference_id: blob.reference_id.to_vec(),
            declared_bytes: blob.declared_size,
            sha256: blob.receipt_sha256.to_vec(),
            custody_transfer_source_proof: blob.custody_transfer_source_proof.clone(),
        }),
        audio_format: SpeechAudioFormatV1::WavPcmS16leMono16000Hz as i32,
        duration_millis: 10_000,
        requested_language: SpeechLanguageV1::English as i32,
        consent_receipt_id: vec![0x93; 16],
        consent_policy_revision: 1,
        maximum_transcript_bytes: 64 * 1024,
        maximum_segments: 128,
        request_digest: Vec::new(),
    })
    .expect("seal Whisper STT request")
}

fn deliver_speech_to_text_request_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    logical_owner_id: &str,
    request: &SpeechToTextRequestV1,
) -> ManagedRuntimeModuleRequestResponseV1 {
    let delivery = ManagedRuntimeModuleRequestDeliveryV1 {
        request_id: request.request_id.clone(),
        logical_owner_id: logical_owner_id.to_owned(),
        contract: Some(speech_to_text_contract_reference_v1()),
        request_payload: request.encode_to_vec(),
        response_blob_target_owner_id: SOURCE_OWNER_ID_V1.to_owned(),
        response_blob_target_module_id: SOURCE_MODULE_ID_V1.to_owned(),
        response_blob_target_capability_id: SOURCE_BLOB_CAPABILITY_ID_V1.to_owned(),
    };
    let response = supervisor
        .relay(
            registration_id,
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::DeliverModuleRequest(delivery)),
            }
            .encode_to_vec(),
        )
        .expect("deliver managed Speech-to-Text request");
    let response = ManagedRuntimeControlResponseV1::decode(response.as_slice())
        .expect("decode managed Speech-to-Text response");
    assert!(response.error_code.is_empty());
    match response.result {
        Some(ControlResult::ModuleRequestDelivery(response)) => response,
        _ => panic!("managed Speech-to-Text response is missing"),
    }
}
