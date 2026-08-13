//! Live managed AI engine routing, owner fencing, terminal replay and request-conflict conformance.

use std::net::TcpListener;

use super::*;

use makosh_ai_contracts::{
    AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, communication_reply_inference_contract_reference_v1,
    encode_reply_source_content_v1, seal_reply_inference_request_v1,
    wire::{
        AiContextReceiptV1, AiEgressPolicyV1, AiInferenceCompletenessV1,
        AiInferenceTerminalStatusV1, AiPrivateSourceReceiptV1, AiReplyLanguageV1,
        AiReplySourceContentV1, AiReplySubjectPolicyV1, AiReplyToneV1, AiUseCaseV1,
        CommunicationReplySuggestionInferenceRequestV1,
        CommunicationReplySuggestionInferenceResultV1,
    },
};
use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeModuleRequestDeliveryV1, ManagedRuntimeModuleRequestResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};

const AI_PRIVATE_INPUT_SENTINEL_V1: &str = "task8-private-input-sentinel";
const OLLAMA_RAW_PROVIDER_SENTINEL_V1: &str = "task8-raw-provider-sentinel";

#[test]
#[ignore = "requires disposable Docker plus actual AI inference, Vault and Storage binaries"]
fn managed_ai_inference_bootstrap_fails_closed_and_stops_promptly() {
    let root = unique_target_root("makosh-managed-ai-inference-bootstrap-negative");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_ai_inference_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            AI_INFERENCE_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim AI inference bootstrap owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted = admit_ai_inference_runtime_v1(&store);
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
    let admitted = prepare_ai_inference_runtime_v1(&supervisor, &store, admitted);
    let runtime_dir = root.join("runtime");

    let capture = start_ai_managed_child_capture_v1(&root, "missing-settings");
    let started = launch_ai_inference_runtime_without_ready_v1(
        &supervisor,
        &store,
        &runtime_dir,
        admitted,
        AiInferenceBootstrapOverrideV1::MissingSettings,
        &capture,
    );
    assert_ai_pre_spawn_bootstrap_denied_v1(&supervisor, &started, "missing settings", &capture);
    let capture = start_ai_managed_child_capture_v1(&root, "drifted-settings");
    let started = launch_ai_inference_successor_without_ready_v1(
        &supervisor,
        &store,
        &runtime_dir,
        started,
        AiInferenceBootstrapOverrideV1::DriftedSettingsRevision,
        &capture,
    );
    assert_ai_runtime_bootstrap_denied_v1(&supervisor, &started, "drifted settings", &capture);
    let capture = start_ai_managed_child_capture_v1(&root, "missing-storage");
    let started = launch_ai_inference_successor_without_ready_v1(
        &supervisor,
        &store,
        &runtime_dir,
        started,
        AiInferenceBootstrapOverrideV1::MissingStorage,
        &capture,
    );
    assert_ai_pre_spawn_bootstrap_denied_v1(&supervisor, &started, "missing storage", &capture);
    let capture = start_ai_managed_child_capture_v1(&root, "stale-storage-fence");
    let started = launch_ai_inference_successor_without_ready_v1(
        &supervisor,
        &store,
        &runtime_dir,
        started,
        AiInferenceBootstrapOverrideV1::StaleStorageFence,
        &capture,
    );
    assert_ai_runtime_bootstrap_active_until_requested_stop_v1(
        &supervisor,
        &started,
        "stale storage fence",
        &capture,
    );
    let capture = start_ai_managed_child_capture_v1(&root, "vault-lease");
    let started = launch_ai_inference_successor_without_ready_v1(
        &supervisor,
        &store,
        &runtime_dir,
        started,
        AiInferenceBootstrapOverrideV1::StopVaultAfterConfiguration,
        &capture,
    );
    assert_ai_runtime_bootstrap_active_until_requested_stop_v1(
        &supervisor,
        &started,
        "Vault lease",
        &capture,
    );

    supervisor
        .shutdown()
        .expect("stop AI bootstrap dependencies");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove AI bootstrap root");
    std::fs::remove_dir_all(data).expect("remove AI bootstrap data");
}

fn assert_ai_pre_spawn_bootstrap_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedAiInferenceRuntimeV1,
    phase: &str,
    capture: &std::path::Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_millis(750);
    while std::time::Instant::now() < deadline {
        assert!(
            !matches!(
                supervisor.relay_port().is_ready(&started.registration_id),
                Ok(true)
            ),
            "{phase} must not signal Ready"
        );
        if !supervisor
            .is_active(&started.registration_id)
            .expect("AI bootstrap activity")
        {
            assert!(
                managed_child_capture_paths_v1(capture).is_empty(),
                "{phase} must be denied before supervised child spawn"
            );
            unsafe {
                std::env::remove_var(
                    crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
                );
            }
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("{phase} pre-spawn denial remained active");
}

fn assert_ai_runtime_bootstrap_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedAiInferenceRuntimeV1,
    phase: &str,
    capture: &std::path::Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_millis(750);
    while std::time::Instant::now() < deadline {
        assert!(
            !matches!(
                supervisor.relay_port().is_ready(&started.registration_id),
                Ok(true)
            ),
            "{phase} must not signal Ready"
        );
        if !supervisor
            .is_active(&started.registration_id)
            .expect("AI drifted bootstrap activity")
        {
            let captures = managed_child_capture_paths_v1(capture);
            assert!(
                captures.len() >= 2 && captures.len().is_multiple_of(2),
                "{phase} must be a captured bounded runtime denial"
            );
            unsafe {
                std::env::remove_var(
                    crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
                );
            }
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("{phase} runtime denial did not settle within its bound");
}

fn assert_ai_runtime_bootstrap_active_until_requested_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedAiInferenceRuntimeV1,
    phase: &str,
    capture: &std::path::Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_millis(100);
    while std::time::Instant::now() < deadline {
        assert!(
            supervisor
                .is_active(&started.registration_id)
                .expect("AI runtime bootstrap activity"),
            "{phase} child exited before the requested stop"
        );
        assert!(
            !matches!(
                supervisor.relay_port().is_ready(&started.registration_id),
                Ok(true)
            ),
            "{phase} must not signal Ready"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let stopped_at = std::time::Instant::now();
    assert!(
        supervisor
            .request_stop_if_active(&started.registration_id)
            .expect("request AI bootstrap stop"),
        "{phase} must still own the exact active child"
    );
    assert!(
        supervisor
            .stop_if_active(&started.registration_id)
            .expect("join AI bootstrap stop"),
        "{phase} requested child must join"
    );
    assert!(stopped_at.elapsed() < Duration::from_secs(2));
    assert!(
        !supervisor
            .is_active(&started.registration_id)
            .expect("AI stopped bootstrap activity"),
        "{phase} must not install a replacement"
    );
    assert_eq!(
        managed_child_capture_paths_v1(capture).len(),
        2,
        "{phase} must spawn exactly one supervised child attempt"
    );
    unsafe {
        std::env::remove_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
        );
    }
    let diagnostic = format!(
        "{:?}",
        supervisor
            .last_failure(&started.registration_id)
            .expect("AI bootstrap diagnostic")
    );
    assert!(!diagnostic.contains(AI_PRIVATE_INPUT_SENTINEL_V1));
    assert!(!diagnostic.contains(AI_PRIVATE_INPUT_SENTINEL_V1));
    assert!(!diagnostic.contains(OLLAMA_RAW_PROVIDER_SENTINEL_V1));
}

fn start_ai_managed_child_capture_v1(root: &std::path::Path, phase: &str) -> std::path::PathBuf {
    private_directory(root.join(format!("stdio-{phase}")))
}

fn managed_child_capture_paths_v1(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut paths = std::fs::read_dir(directory)
        .expect("read managed child capture directory")
        .map(|entry| entry.expect("read managed child capture entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, AI inference and Ollama AI binaries"]
fn managed_ai_inference_routes_to_ollama_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let malformed_provider =
        TcpListener::bind(("127.0.0.1", 0)).expect("bind malformed Ollama provider");
    let ollama_port = malformed_provider
        .local_addr()
        .expect("read malformed Ollama address")
        .port();
    let malformed_provider = std::thread::spawn(move || {
        let (mut stream, _) = malformed_provider.accept().expect("accept Ollama request");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set Ollama read timeout");
        let mut request = [0_u8; 16 * 1024];
        let _ = std::io::Read::read(&mut stream, &mut request);
        let body = OLLAMA_RAW_PROVIDER_SENTINEL_V1.as_bytes();
        let headers = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        std::io::Write::write_all(&mut stream, headers.as_bytes())
            .expect("write malformed Ollama headers");
        std::io::Write::write_all(&mut stream, body).expect("write malformed Ollama body");
    });

    let root = unique_target_root("makosh-managed-ai-inference-negative");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_ai_inference_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    crate::platform::blob::binding::bind_installed_release(&store, release.kernel())
        .expect("bind signed Blob release");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            AI_INFERENCE_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim AI inference logical owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_ollama = admit_ollama_ai_runtime_v1(&store);
    let admitted_ai = admit_ai_inference_runtime_v1(&store);
    let source = AiInferenceBlobSourceFixtureV1::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    configure_ai_module_request_router_v1(&supervisor, &store);
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
    let admitted_ollama = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted_ollama);
    let admitted_ai = prepare_ai_inference_runtime_v1(&supervisor, &store, admitted_ai);
    let ollama_child_stdio = start_ai_managed_child_capture_v1(&root, "privacy-ollama");
    unsafe {
        std::env::set_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
            &ollama_child_stdio,
        );
    }
    let ollama = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_ollama,
        ollama_port,
    );
    let ai_child_stdio = start_ai_managed_child_capture_v1(&root, "privacy-ai");
    unsafe {
        std::env::set_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
            &ai_child_stdio,
        );
    }
    let ai = start_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted_ai);
    unsafe {
        std::env::remove_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
        );
    }
    assert_eq!(ai.runtime_generation, 1);
    assert!(
        supervisor
            .is_active(&ai.registration_id)
            .expect("read AI inference process state")
    );
    let source_content = encode_reply_source_content_v1(&AiReplySourceContentV1 {
        sender_utf8: b"Alice Example <alice@example.test>".to_vec(),
        subject_utf8: b"Quarterly update".to_vec(),
        body_utf8: AI_PRIVATE_INPUT_SENTINEL_V1.as_bytes().to_vec(),
    })
    .expect("typed AI source content");
    let blob = source.write(&store, &supervisor, &data, [0x51; 16], &source_content);
    let request = inference_request_v1(&blob);
    let first = deliver_inference_request_v1(&supervisor, &ai.registration_id, &request);
    assert!(first.error_code.is_empty());
    let first_result =
        CommunicationReplySuggestionInferenceResultV1::decode(first.response_payload.as_slice())
            .expect("typed AI provider-unavailable result");
    assert_eq!(first_result.run_id, request.run_id);
    assert_eq!(
        first_result.terminal_status,
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable as i32
    );
    assert!(first_result.subject_utf8.is_empty());
    assert!(first_result.body_utf8.is_empty());
    assert!(
        !first
            .response_payload
            .windows(AI_PRIVATE_INPUT_SENTINEL_V1.len())
            .any(|bytes| bytes == AI_PRIVATE_INPUT_SENTINEL_V1.as_bytes())
    );
    malformed_provider
        .join()
        .expect("join malformed Ollama provider");
    let ai_storage_credential = runtime_storage_credential_for_registration_v1(
        &supervisor,
        &store,
        &data,
        &ai.registration_id,
        makosh_ai_inference_runtime::AI_INFERENCE_STORAGE_CAPABILITY_ID_V1,
    );
    let ollama_storage_credential = runtime_storage_credential_for_registration_v1(
        &supervisor,
        &store,
        &data,
        &ollama.registration_id,
        makosh_ollama_ai_api::OLLAMA_AI_STORAGE_CAPABILITY_ID_V1,
    );
    assert_ai_privacy_surfaces_v1(
        &first.response_payload,
        &format!("{:?}", supervisor.last_failure(&ai.registration_id)),
        &format!("{:?}", supervisor.last_failure(&ollama.registration_id)),
        [&ai_storage_credential[..], &ollama_storage_credential[..]],
        [&ai_child_stdio, &ollama_child_stdio],
    );

    supervisor
        .stop(&ollama.registration_id)
        .expect("stop Ollama dependency before AI replay");
    let previous_generation = ai.runtime_generation;
    let ai = restart_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), ai);
    assert_eq!(ai.runtime_generation, previous_generation + 1);

    let replayed = deliver_inference_request_v1(&supervisor, &ai.registration_id, &request);
    assert_eq!(replayed, first);

    let mut conflicting = request.clone();
    conflicting.maximum_output_tokens += 1;
    let conflicting =
        seal_reply_inference_request_v1(conflicting).expect("seal conflicting request");
    let rejected = deliver_inference_request_v1(&supervisor, &ai.registration_id, &conflicting);
    assert_eq!(rejected.request_id, request.run_id);
    assert_eq!(rejected.error_code, "REJECTED");
    assert!(rejected.response_payload.is_empty());

    assert_ai_inference_owner_rls_v1("makosh_storage_authenticated");

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove AI inference fixture");
    std::fs::remove_dir_all(data).expect("remove short AI inference kernel data fixture");
}

fn assert_ai_privacy_surfaces_v1(
    terminal: &[u8],
    ai_supervisor_diagnostic: &str,
    ollama_supervisor_diagnostic: &str,
    storage_credentials: [&[u8]; 2],
    child_capture_directories: [&std::path::Path; 2],
) {
    for marker in [
        AI_PRIVATE_INPUT_SENTINEL_V1.as_bytes(),
        OLLAMA_RAW_PROVIDER_SENTINEL_V1.as_bytes(),
    ] {
        assert_private_bytes_absent_v1(terminal, marker, "typed AI terminal");
        assert_private_bytes_absent_v1(
            ai_supervisor_diagnostic.as_bytes(),
            marker,
            "AI supervisor diagnostic",
        );
        assert_private_bytes_absent_v1(
            ollama_supervisor_diagnostic.as_bytes(),
            marker,
            "Ollama supervisor diagnostic",
        );
    }
    for credential in storage_credentials {
        assert_private_bytes_absent_v1(terminal, credential, "typed AI terminal");
        assert_private_bytes_absent_v1(
            ai_supervisor_diagnostic.as_bytes(),
            credential,
            "AI supervisor diagnostic",
        );
        assert_private_bytes_absent_v1(
            ollama_supervisor_diagnostic.as_bytes(),
            credential,
            "Ollama supervisor diagnostic",
        );
    }
    for directory in child_capture_directories {
        assert_supervised_ai_child_output_is_private_v1(directory, &storage_credentials);
    }
}

fn assert_supervised_ai_child_output_is_private_v1(
    directory: &std::path::Path,
    storage_credentials: &[&[u8]; 2],
) {
    let captures = managed_child_capture_paths_v1(directory);
    assert_eq!(
        captures.len(),
        2,
        "each supervised AI target child needs exact stdout/stderr sinks"
    );
    for capture in captures {
        let bytes = std::fs::read(capture).expect("read supervised AI target child output");
        for marker in [
            AI_PRIVATE_INPUT_SENTINEL_V1.as_bytes(),
            OLLAMA_RAW_PROVIDER_SENTINEL_V1.as_bytes(),
        ] {
            assert_private_bytes_absent_v1(&bytes, marker, "supervised AI target child output");
        }
        for credential in storage_credentials {
            assert_private_bytes_absent_v1(&bytes, credential, "supervised AI target child output");
        }
    }
}

fn assert_private_bytes_absent_v1(surface: &[u8], private: &[u8], surface_name: &str) {
    assert!(!private.is_empty());
    assert!(
        !surface.windows(private.len()).any(|value| value == private),
        "{surface_name} exposed private material"
    );
}

#[test]
#[ignore = "requires disposable Docker plus a real loopback Ollama service with makosh-conformance:latest"]
fn managed_ai_inference_completes_real_provider_generation() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let ollama_port = required("MAKOSH_OLLAMA_LIVE_PORT")
        .parse::<u16>()
        .expect("valid live Ollama port");
    let root = unique_target_root("makosh-managed-ai-inference-live");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_ai_inference_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    crate::platform::blob::binding::bind_installed_release(&store, release.kernel())
        .expect("bind signed Blob release");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            AI_INFERENCE_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim live AI inference logical owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted_ollama = admit_ollama_ai_runtime_v1(&store);
    let admitted_ai = admit_ai_inference_runtime_v1(&store);
    let source = AiInferenceBlobSourceFixtureV1::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    configure_ai_module_request_router_v1(&supervisor, &store);
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
    let admitted_ollama = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted_ollama);
    let admitted_ai = prepare_ai_inference_runtime_v1(&supervisor, &store, admitted_ai);
    let ollama = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_ollama,
        ollama_port,
    );
    let ai = start_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted_ai);

    let source_content = encode_reply_source_content_v1(&AiReplySourceContentV1 {
        sender_utf8: b"Alice Example <alice@example.test>".to_vec(),
        subject_utf8: b"Quarterly update".to_vec(),
        body_utf8: b"Private source body for a bounded local reply".to_vec(),
    })
    .expect("typed live AI source content");
    let blob = source.write(&store, &supervisor, &data, [0x71; 16], &source_content);
    let request = inference_request_v1(&blob);

    let wrong_owner = deliver_inference_request_for_owner_v1(
        &supervisor,
        &ai.registration_id,
        "owner-2",
        &request,
    );
    assert_eq!(wrong_owner.request_id, request.run_id);
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());

    let first = deliver_inference_request_v1(&supervisor, &ai.registration_id, &request);
    assert!(first.error_code.is_empty());
    let result =
        CommunicationReplySuggestionInferenceResultV1::decode(first.response_payload.as_slice())
            .expect("typed successful AI inference result");
    assert_eq!(result.run_id, request.run_id);
    assert_eq!(
        result.terminal_status,
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32
    );
    assert_eq!(
        result.completeness,
        AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32
    );
    assert_eq!(
        result.resolved_language,
        AiReplyLanguageV1::AiReplyLanguageEnglish as i32
    );
    assert!(!result.body_utf8.is_empty());
    let receipt = result
        .inference_receipt
        .as_ref()
        .expect("successful AI inference receipt");
    assert_eq!(receipt.model_revision_sha256.len(), 32);
    assert_eq!(receipt.prompt_policy_sha256.len(), 32);
    assert_eq!(receipt.provider_settings_revision, 1);

    supervisor
        .stop(&ollama.registration_id)
        .expect("stop Ollama dependency before successful replay");
    let previous_generation = ai.runtime_generation;
    let ai = restart_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), ai);
    assert_eq!(ai.runtime_generation, previous_generation + 1);
    let replayed = deliver_inference_request_v1(&supervisor, &ai.registration_id, &request);
    assert_eq!(replayed, first);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove live AI inference fixture");
    std::fs::remove_dir_all(data).expect("remove short live AI inference kernel data fixture");
}

fn inference_request_v1(
    blob: &AiInferenceFixtureBlobV1,
) -> CommunicationReplySuggestionInferenceRequestV1 {
    seal_reply_inference_request_v1(CommunicationReplySuggestionInferenceRequestV1 {
        run_id: vec![0x61; 16],
        context: Some(AiContextReceiptV1 {
            context_id: vec![0x62; 16],
            use_case: AiUseCaseV1::AiUseCaseCommunicationReplySuggestion as i32,
            source_evidence_id: vec![0x63; 16],
            source_evidence_revision: 1,
            contract_major: AI_CONTRACT_MAJOR_V1,
            contract_revision: AI_CONTRACT_REVISION_V1,
            contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
            request_digest: Vec::new(),
        }),
        source: Some(AiPrivateSourceReceiptV1 {
            reference_id: blob.reference_id.to_vec(),
            declared_bytes: blob.declared_size,
            sha256: blob.receipt_sha256.to_vec(),
            custody_transfer_source_proof: blob.custody_transfer_source_proof.clone(),
        }),
        tone: AiReplyToneV1::AiReplyToneNeutral as i32,
        language: AiReplyLanguageV1::AiReplyLanguageEnglish as i32,
        subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyPreserve as i32,
        maximum_output_bytes: 4_096,
        maximum_output_tokens: 512,
        egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
        egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        logical_owner_id: AI_INFERENCE_LOGICAL_OWNER_ID_V1.to_owned(),
    })
    .expect("seal AI inference request")
}

fn deliver_inference_request_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    request: &CommunicationReplySuggestionInferenceRequestV1,
) -> ManagedRuntimeModuleRequestResponseV1 {
    deliver_inference_request_for_owner_v1(
        supervisor,
        registration_id,
        AI_INFERENCE_LOGICAL_OWNER_ID_V1,
        request,
    )
}

fn deliver_inference_request_for_owner_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    logical_owner_id: &str,
    request: &CommunicationReplySuggestionInferenceRequestV1,
) -> ManagedRuntimeModuleRequestResponseV1 {
    let delivery = ManagedRuntimeModuleRequestDeliveryV1 {
        request_id: request.run_id.clone(),
        logical_owner_id: logical_owner_id.to_owned(),
        contract: Some(communication_reply_inference_contract_reference_v1()),
        request_payload: request.encode_to_vec(),
        response_blob_target_owner_id: String::new(),
        response_blob_target_module_id: String::new(),
        response_blob_target_capability_id: String::new(),
    };
    let response = supervisor
        .relay(
            registration_id,
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::DeliverModuleRequest(delivery)),
            }
            .encode_to_vec(),
        )
        .expect("deliver managed AI inference request");
    let response = ManagedRuntimeControlResponseV1::decode(response.as_slice())
        .expect("decode managed AI inference response");
    assert!(response.error_code.is_empty());
    match response.result {
        Some(ControlResult::ModuleRequestDelivery(response)) => response,
        _ => panic!("managed AI inference response is missing"),
    }
}
