//! Managed Ollama conformance against unavailable and explicitly supplied live providers.

use std::io::ErrorKind;
use std::net::TcpListener;

use super::*;

use makosh_ai_contracts::{
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, ai_provider_reply_generation_contract_reference_v1,
    wire::{
        AiEgressPolicyV1, AiInferenceCompletenessV1, AiInferenceTerminalStatusV1,
        AiProviderReplyGenerationRequestV1, AiProviderReplyGenerationResultV1, AiReplyLanguageV1,
        AiReplySubjectPolicyV1, AiReplyToneV1,
    },
};
use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeModuleRequestDeliveryV1, ManagedRuntimeModuleRequestResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};

const OLLAMA_RAW_PROVIDER_SENTINEL_V1: &str = "task8-raw-provider-sentinel";
const AI_PRIVATE_INPUT_SENTINEL_V1: &str = "task8-private-input-sentinel";

#[test]
#[ignore = "requires disposable Docker plus actual Ollama AI, Vault and Storage binaries"]
fn managed_ollama_ai_bootstrap_fails_closed_and_stops_promptly() {
    let port_reservation = TcpListener::bind(("127.0.0.1", 0)).expect("reserve Ollama port");
    let ollama_port = port_reservation
        .local_addr()
        .expect("Ollama address")
        .port();
    drop(port_reservation);
    let root = unique_target_root("makosh-managed-ollama-ai-bootstrap-negative");
    let data = private_directory(root.join("kernel"));
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_ollama_ai_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            OLLAMA_AI_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim Ollama AI bootstrap owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted = admit_ollama_ai_runtime_v1(&store);
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
    let admitted = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted);
    let runtime_dir = root.join("runtime");

    let capture = start_ollama_managed_child_capture_v1(&root, "missing-settings");
    let started = launch_ollama_ai_runtime_without_ready_v1(
        &supervisor,
        &store,
        &data,
        &runtime_dir,
        admitted,
        ollama_port,
        OllamaAiBootstrapOverrideV1::MissingSettings,
        &capture,
    );
    assert_ollama_pre_spawn_bootstrap_denied_v1(
        &supervisor,
        &started,
        "missing settings",
        &capture,
    );
    let capture = start_ollama_managed_child_capture_v1(&root, "drifted-settings");
    let started = launch_ollama_ai_successor_without_ready_v1(
        &supervisor,
        &store,
        &data,
        &runtime_dir,
        started,
        ollama_port,
        OllamaAiBootstrapOverrideV1::DriftedSettingsTarget,
        &capture,
    );
    assert_ollama_runtime_bootstrap_denied_v1(&supervisor, &started, "drifted settings", &capture);
    let capture = start_ollama_managed_child_capture_v1(&root, "missing-storage");
    let started = launch_ollama_ai_successor_without_ready_v1(
        &supervisor,
        &store,
        &data,
        &runtime_dir,
        started,
        ollama_port,
        OllamaAiBootstrapOverrideV1::MissingStorage,
        &capture,
    );
    assert_ollama_pre_spawn_bootstrap_denied_v1(&supervisor, &started, "missing storage", &capture);
    let capture = start_ollama_managed_child_capture_v1(&root, "stale-storage-fence");
    let started = launch_ollama_ai_successor_without_ready_v1(
        &supervisor,
        &store,
        &data,
        &runtime_dir,
        started,
        ollama_port,
        OllamaAiBootstrapOverrideV1::StaleStorageFence,
        &capture,
    );
    assert_ollama_runtime_bootstrap_active_until_requested_stop_v1(
        &supervisor,
        &started,
        "stale storage fence",
        &capture,
    );
    let capture = start_ollama_managed_child_capture_v1(&root, "vault-lease");
    let started = launch_ollama_ai_successor_without_ready_v1(
        &supervisor,
        &store,
        &data,
        &runtime_dir,
        started,
        ollama_port,
        OllamaAiBootstrapOverrideV1::StopVaultAfterConfiguration,
        &capture,
    );
    assert_ollama_runtime_bootstrap_active_until_requested_stop_v1(
        &supervisor,
        &started,
        "Vault lease",
        &capture,
    );

    supervisor
        .shutdown()
        .expect("stop Ollama bootstrap dependencies");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Ollama bootstrap fixture");
}

fn assert_ollama_pre_spawn_bootstrap_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedOllamaAiRuntimeV1,
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
            .expect("Ollama bootstrap activity")
        {
            assert!(
                managed_ollama_child_capture_paths_v1(capture).is_empty(),
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

fn assert_ollama_runtime_bootstrap_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedOllamaAiRuntimeV1,
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
            .expect("Ollama drifted bootstrap activity")
        {
            let captures = managed_ollama_child_capture_paths_v1(capture);
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

fn assert_ollama_runtime_bootstrap_active_until_requested_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedOllamaAiRuntimeV1,
    phase: &str,
    capture: &std::path::Path,
) {
    let deadline = std::time::Instant::now() + Duration::from_millis(100);
    while std::time::Instant::now() < deadline {
        assert!(
            supervisor
                .is_active(&started.registration_id)
                .expect("Ollama runtime bootstrap activity"),
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
            .expect("request Ollama bootstrap stop"),
        "{phase} must still own the exact active child"
    );
    assert!(
        supervisor
            .stop_if_active(&started.registration_id)
            .expect("join Ollama bootstrap stop"),
        "{phase} requested child must join"
    );
    assert!(stopped_at.elapsed() < Duration::from_secs(2));
    assert!(
        !supervisor
            .is_active(&started.registration_id)
            .expect("Ollama stopped bootstrap activity"),
        "{phase} must not install a replacement"
    );
    assert_eq!(
        managed_ollama_child_capture_paths_v1(capture).len(),
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
            .expect("Ollama bootstrap diagnostic")
    );
    assert!(!diagnostic.contains(OLLAMA_RAW_PROVIDER_SENTINEL_V1));
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage and Ollama AI binaries"]
fn managed_ollama_ai_runtime_replays_provider_unavailable_without_second_http_attempt() {
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

    let root = unique_target_root("makosh-managed-ollama-ai-negative");
    let data = private_directory(root.join("kernel"));
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_ollama_ai_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            OLLAMA_AI_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim Ollama AI logical owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted = admit_ollama_ai_runtime_v1(&store);
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
    let admitted = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted);
    let child_stdio = start_ollama_managed_child_capture_v1(&root, "privacy");
    unsafe {
        std::env::set_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
            &child_stdio,
        );
    }
    let runtime = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted,
        ollama_port,
    );

    let mut request = provider_request_v1();
    request.input_utf8 = AI_PRIVATE_INPUT_SENTINEL_V1.as_bytes().to_vec();
    let first = deliver_provider_request_v1(&supervisor, &runtime.registration_id, &request);
    assert!(first.error_code.is_empty());
    let first_result = AiProviderReplyGenerationResultV1::decode(first.response_payload.as_slice())
        .expect("typed Ollama provider-unavailable result");
    assert_eq!(first_result.request_id, request.request_id);
    assert_eq!(
        first_result.terminal_status,
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable as i32
    );
    assert!(
        !first
            .response_payload
            .windows(OLLAMA_RAW_PROVIDER_SENTINEL_V1.len())
            .any(|bytes| { bytes == OLLAMA_RAW_PROVIDER_SENTINEL_V1.as_bytes() })
    );
    malformed_provider
        .join()
        .expect("join malformed Ollama provider");
    let supervisor_diagnostic = format!(
        "{:?}",
        supervisor
            .last_failure(&runtime.registration_id)
            .expect("read Ollama supervisor diagnostic")
    );
    for marker in [
        OLLAMA_RAW_PROVIDER_SENTINEL_V1,
        "task8-private-input-sentinel",
    ] {
        assert!(!supervisor_diagnostic.contains(marker));
    }
    let storage_credential = runtime_storage_credential_for_registration_v1(
        &supervisor,
        &store,
        &data,
        &runtime.registration_id,
        makosh_ollama_ai_api::OLLAMA_AI_STORAGE_CAPABILITY_ID_V1,
    );
    for marker in [
        AI_PRIVATE_INPUT_SENTINEL_V1,
        OLLAMA_RAW_PROVIDER_SENTINEL_V1,
    ] {
        assert!(
            !first
                .response_payload
                .windows(marker.len())
                .any(|bytes| bytes == marker.as_bytes()),
            "typed provider terminal exposed a private marker"
        );
    }
    assert!(
        !first
            .response_payload
            .windows(storage_credential.len())
            .any(|bytes| bytes == storage_credential.as_slice()),
        "typed provider terminal exposed the actual Storage credential"
    );
    assert!(
        !supervisor_diagnostic
            .as_bytes()
            .windows(storage_credential.len())
            .any(|bytes| bytes == storage_credential.as_slice())
    );
    unsafe {
        std::env::remove_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
        );
    }

    let no_second_attempt =
        TcpListener::bind(("127.0.0.1", ollama_port)).expect("guard Ollama replay port");
    no_second_attempt
        .set_nonblocking(true)
        .expect("make Ollama replay guard nonblocking");
    let previous_generation = runtime.runtime_generation;
    let runtime = restart_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        runtime,
        ollama_port,
    );
    assert_eq!(runtime.runtime_generation, previous_generation + 1);

    let replayed = deliver_provider_request_v1(&supervisor, &runtime.registration_id, &request);
    assert_eq!(replayed, first);
    assert_no_ollama_connection_v1(&no_second_attempt);

    let mut conflicting = request.clone();
    conflicting.input_utf8.extend_from_slice(b" changed");
    let rejected = deliver_provider_request_v1(&supervisor, &runtime.registration_id, &conflicting);
    assert_eq!(rejected.request_id, request.request_id);
    assert_eq!(rejected.error_code, "REJECTED");
    assert!(rejected.response_payload.is_empty());
    assert_no_ollama_connection_v1(&no_second_attempt);

    assert_ollama_ai_owner_rls_v1("makosh_storage_authenticated");

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
        std::env::remove_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
        );
    }
    assert_supervised_ollama_child_output_is_private_v1(&child_stdio, &storage_credential);
    std::fs::remove_dir_all(root).expect("remove Ollama AI fixture");
}

fn assert_supervised_ollama_child_output_is_private_v1(
    directory: &std::path::Path,
    storage_credential: &[u8],
) {
    let captures = managed_ollama_child_capture_paths_v1(directory);
    assert_eq!(
        captures.len(),
        2,
        "the supervised Ollama child needs exact stdout/stderr sinks"
    );
    for capture in captures {
        let bytes = std::fs::read(capture).expect("read supervised Ollama child output");
        for marker in [
            AI_PRIVATE_INPUT_SENTINEL_V1,
            OLLAMA_RAW_PROVIDER_SENTINEL_V1,
        ] {
            assert!(
                !bytes
                    .windows(marker.len())
                    .any(|value| value == marker.as_bytes()),
                "supervised child output exposed a private marker"
            );
        }
        assert!(
            !bytes
                .windows(storage_credential.len())
                .any(|value| value == storage_credential),
            "supervised child output exposed the actual Storage credential"
        );
    }
}

fn start_ollama_managed_child_capture_v1(
    root: &std::path::Path,
    phase: &str,
) -> std::path::PathBuf {
    private_directory(root.join(format!("stdio-{phase}")))
}

fn managed_ollama_child_capture_paths_v1(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut paths = std::fs::read_dir(directory)
        .expect("read managed Ollama child capture directory")
        .map(|entry| {
            entry
                .expect("read managed Ollama child capture entry")
                .path()
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

#[test]
#[ignore = "requires disposable Docker plus a real loopback Ollama service with makosh-conformance:latest"]
fn managed_ollama_ai_runtime_completes_real_provider_generation() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let ollama_port = required("MAKOSH_OLLAMA_LIVE_PORT")
        .parse::<u16>()
        .expect("valid live Ollama port");
    let root = unique_target_root("makosh-managed-ollama-ai-live");
    let data = private_directory(root.join("kernel"));
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_ollama_ai_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            OLLAMA_AI_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim live Ollama AI logical owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted = admit_ollama_ai_runtime_v1(&store);
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
    let admitted = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted);
    let runtime = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted,
        ollama_port,
    );

    let mut request = provider_request_v1();
    request.subject_policy = AiReplySubjectPolicyV1::AiReplySubjectPolicyGenerateIfMissing as i32;
    let response = deliver_provider_request_v1(&supervisor, &runtime.registration_id, &request);
    assert!(response.error_code.is_empty());
    let result = AiProviderReplyGenerationResultV1::decode(response.response_payload.as_slice())
        .expect("typed successful Ollama result");
    assert_eq!(result.request_id, request.request_id);
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
    assert_eq!(result.model_revision_sha256.len(), 32);
    assert!(result.input_tokens > 0);
    assert!(result.output_tokens > 0);
    assert_eq!(result.provider_settings_revision, 1);

    let wrong_owner = deliver_provider_request_for_owner_v1(
        &supervisor,
        &runtime.registration_id,
        "owner-2",
        &request,
    );
    assert_eq!(wrong_owner.request_id, request.request_id);
    assert_eq!(wrong_owner.error_code, "REJECTED");
    assert!(wrong_owner.response_payload.is_empty());

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove live Ollama AI fixture");
}

fn provider_request_v1() -> AiProviderReplyGenerationRequestV1 {
    AiProviderReplyGenerationRequestV1 {
        request_id: vec![0x61; 16],
        input_utf8: b"Private source for a bounded local reply".to_vec(),
        tone: AiReplyToneV1::AiReplyToneWarm as i32,
        language: AiReplyLanguageV1::AiReplyLanguageEnglish as i32,
        subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyOmit as i32,
        maximum_output_bytes: 1_024,
        maximum_output_tokens: 256,
        egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
        egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
    }
}

fn deliver_provider_request_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    request: &AiProviderReplyGenerationRequestV1,
) -> ManagedRuntimeModuleRequestResponseV1 {
    deliver_provider_request_for_owner_v1(
        supervisor,
        registration_id,
        OLLAMA_AI_LOGICAL_OWNER_ID_V1,
        request,
    )
}

fn deliver_provider_request_for_owner_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    logical_owner_id: &str,
    request: &AiProviderReplyGenerationRequestV1,
) -> ManagedRuntimeModuleRequestResponseV1 {
    let delivery = ManagedRuntimeModuleRequestDeliveryV1 {
        request_id: request.request_id.clone(),
        logical_owner_id: logical_owner_id.to_owned(),
        contract: Some(ai_provider_reply_generation_contract_reference_v1()),
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
        .expect("deliver managed Ollama provider request");
    let response = ManagedRuntimeControlResponseV1::decode(response.as_slice())
        .expect("decode managed Ollama response");
    assert!(response.error_code.is_empty());
    match response.result {
        Some(ControlResult::ModuleRequestDelivery(response)) => response,
        _ => panic!("managed Ollama response is missing"),
    }
}

fn assert_no_ollama_connection_v1(listener: &TcpListener) {
    match listener.accept() {
        Err(error) if error.kind() == ErrorKind::WouldBlock => {}
        Ok(_) => panic!("persisted Ollama terminal replay attempted provider HTTP"),
        Err(error) => panic!("inspect Ollama replay guard: {error}"),
    }
}
