//! Live managed Telegram process through Kernel leases into managed Communications.

use super::*;
use std::{
    io::Read as _,
    os::unix::fs::{MetadataExt as _, PermissionsExt as _},
    process::{Command, Stdio},
    time::Instant,
};

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use base64::Engine as _;
use makosh_events_protocol::validation::envelope::decode_envelope_v1;
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use makosh_telegram_api::{
    TelegramClientRequest, TelegramClientResponse, TelegramHistorySyncMode, TelegramOperationState,
    TelegramProviderCommand, TelegramProviderQuery, TelegramProviderQueryResponse,
    TelegramRuntimeReconfigurationRequest, TelegramRuntimeReconfigurationState,
    TelegramRuntimeState, TelegramSendMessage, client_contract::TelegramClientContractV1,
};
use makosh_telegram_automation_api::{
    contract::{
        TELEGRAM_AUTOMATION_CONTRACT_MAJOR, TELEGRAM_AUTOMATION_CONTRACT_REVISION,
        TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1, TELEGRAM_AUTOMATION_MODULE_ID,
        TELEGRAM_AUTOMATION_OWNER_ID, TelegramAutomationContractV1,
    },
    wire::{
        AutomationCommandRequestV1, AutomationCommandResponseV1, AutomationFailureCodeV1,
        AutomationPolicyV1, AutomationPreviewReceiptV1, AutomationQueryRequestV1,
        AutomationQueryResponseV1, AutomationTemplateV1, AutomationVariableV1,
        GetAutomationPreviewReceiptQueryV1, ListAutomationPoliciesQueryV1,
        ListAutomationTemplatesQueryV1, PreviewAutomationPolicyCommandV1,
        UpsertAutomationPolicyCommandV1, UpsertAutomationTemplateCommandV1,
        automation_command_request_v1, automation_command_response_v1, automation_query_request_v1,
        automation_query_response_v1,
    },
};
use makosh_telegram_calls_api::{
    contract::{
        TELEGRAM_CALLS_CONTRACT_MAJOR, TELEGRAM_CALLS_CONTRACT_REVISION,
        TELEGRAM_CALLS_DESCRIPTOR_SET_V1, TELEGRAM_CALLS_MODULE_ID, TELEGRAM_CALLS_OWNER_ID,
        TelegramCallsContractV1,
    },
    wire::{
        CallDiscardReasonV1, CallOperationKindV1, CallOperationStateV1, CallStateV1,
        CallsCommandRequestV1, CallsCommandResponseV1, CallsQueryRequestV1, CallsQueryResponseV1,
        CallsReplayRequestV1, CallsReplayResponseV1, EndCallRequestV1, GetCallOperationRequestV1,
        InitiateAudioCallRequestV1, ListCallsRequestV1, call_frame_v1, calls_command_request_v1,
        calls_command_response_v1, calls_failure_v1::Code as CallsFailureCodeV1,
        calls_query_request_v1, calls_query_response_v1,
    },
};
use makosh_telegram_runtime::admission::{
    TELEGRAM_STORAGE_CAPABILITY_ID, TELEGRAM_TDJSON_ARTIFACT_ID, TELEGRAM_TGCALLS_ARTIFACT_ID,
};
use makosh_telegram_runtime::client_port::{
    TelegramClientPortError, decode_module_response, encode_module_request,
};
use prost::Message as _;
use sha2::Digest as _;
use zeroize::Zeroizing;

const AUTOMATION_TEMPLATE_ID: &str = "managed-template-1";
const AUTOMATION_POLICY_ID: &str = "managed-policy-1";
const AUTOMATION_PREVIEW_ID: &str = "managed-preview-1";
const AUTOMATION_CHAT_ID: &str = "telegram-chat-1";
const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const TASK10_PRIVATE_BODY_SENTINEL_V1: &[u8] = b"task10-private-body-sentinel";
const TASK10_RAW_PROVIDER_SENTINEL_V1: &[u8] = b"task10-raw-provider-sentinel";
const TASK10_PRIVATE_API_HASH_SENTINEL_V1: &[u8] = b"managed-telegram-api-hash";
const TASK10_PRIVATE_CALL_CONFIG_SENTINEL_V1: &[u8] = b"managed-private-config";
const TASK10_PRIVATE_CALL_PARAMETERS_SENTINEL_V1: &[u8] = b"managed-private-parameters";
const TASK10_PRIVATE_PROVIDER_ERROR_SENTINEL_V1: &[u8] = b"private fixture failure";
const TASK10_REAL_TDJSON_ENV_V1: &str = "MAKOSH_TELEGRAM_REAL_TDJSON";
const TASK10_REAL_TGCALLS_ROOT_ENV_V1: &str = "MAKOSH_TELEGRAM_REAL_TGCALLS_ROOT";
const TASK10_REAL_API_ID_FILE_ENV_V1: &str = "MAKOSH_TELEGRAM_REAL_API_ID_FILE";
const TASK10_REAL_API_HASH_FILE_ENV_V1: &str = "MAKOSH_TELEGRAM_REAL_API_HASH_FILE";
const TASK10_REAL_TDJSON_SHA256_V1: &str =
    "5cae8a2457076befc948c9203e8158af880a4d4ac6bd29a2f68475d4660fedb8";
const TASK10_TGCALLS_BRIDGE_V1: &str = "libmakosh_tgcalls_bridge.dylib";
const TASK10_TGCALLS_AUDIO_PROBE_V1: &str = "makosh_tgcalls_audio_device_conformance";

#[derive(Debug)]
enum TelegramClientRouteError {
    Kernel(String),
    Client(TelegramClientPortError),
}

impl TelegramClientRouteError {
    fn is_retryable(&self) -> bool {
        match self {
            Self::Client(TelegramClientPortError::Protocol(code)) => {
                matches!(code.as_str(), "RUNTIME_BUSY" | "RUNTIME_UNAVAILABLE")
            }
            Self::Kernel(error) => matches!(
                error.as_str(),
                "managed runtime V2 relay response is invalid"
                    | "managed runtime relay timed out"
                    | "managed runtime relay is unavailable"
            ),
            Self::Client(_) => false,
        }
    }
}

pub(super) struct PreparedManagedTelegramFixture {
    pub(super) root: PathBuf,
    pub(super) data: PathBuf,
    pub(super) store: Arc<SqliteControlStore>,
    pub(super) supervisor: ManagedRuntimeSupervisor,
    pub(super) realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
    admitted_telegram: Option<AdmittedTelegramRuntime>,
}

impl PreparedManagedTelegramFixture {
    pub(super) fn start_telegram(&mut self) -> StartedTelegramRuntime {
        start_telegram_runtime(
            &self.supervisor,
            &self.store,
            &self.data,
            &self.root.join("runtime"),
            self.admitted_telegram
                .take()
                .expect("prepared Telegram admission"),
        )
    }

    pub(super) fn start_telegram_with_capture_v1(
        &mut self,
        capture_directory: &Path,
    ) -> StartedTelegramRuntime {
        start_telegram_runtime_with_capture_v1(
            &self.supervisor,
            &self.store,
            &self.data,
            &self.root.join("runtime"),
            self.admitted_telegram
                .take()
                .expect("prepared Telegram admission"),
            capture_directory,
        )
    }

    pub(super) fn restart_telegram(
        &self,
        predecessor: StartedTelegramRuntime,
    ) -> StartedTelegramRuntime {
        restart_telegram_runtime(
            &self.supervisor,
            &self.store,
            &self.data,
            &self.root.join("runtime"),
            predecessor,
        )
    }
}

impl Drop for PreparedManagedTelegramFixture {
    fn drop(&mut self) {
        let _ = self.supervisor.shutdown();
        unsafe {
            std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
        }
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::remove_dir_all(&self.data);
    }
}

pub(super) fn prepare_managed_telegram_fixture() -> PreparedManagedTelegramFixture {
    prepare_managed_telegram_fixture_without_capability(None)
}

fn prepare_managed_telegram_fixture_without_capability(
    excluded_capability_id: Option<&str>,
) -> PreparedManagedTelegramFixture {
    let tdjson = telegram_tdjson_fixture();
    let tgcalls = telegram_tgcalls_fixture();
    prepare_managed_telegram_fixture_with_inputs_v1(
        excluded_capability_id,
        &tdjson,
        &tgcalls,
        42,
        b"managed-telegram-api-hash",
        &[31_u8; 32],
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_managed_telegram_fixture_with_inputs_v1(
    excluded_capability_id: Option<&str>,
    tdjson: &Path,
    tgcalls: &Path,
    api_id: i64,
    api_hash: &[u8],
    session_encryption_key: &[u8],
) -> PreparedManagedTelegramFixture {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-telegram-runtime");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    seed_telegram_vault_with_secrets_v1(&vault_dir, api_hash, session_encryption_key);
    let release = installed_communications_telegram_release_with_native_v1(&root, tdjson, tgcalls);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            [4; 65],
        ))
        .expect("claim initial owner");
    super::super::browser_gateway_session::admit_browser_test_device(&store, "owner-1");
    let admitted_telegram =
        admit_telegram_runtime_with_api_id_v1(&store, excluded_capability_id, api_id);
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(&store),
            realtime.clone(),
        )))
        .expect("configure managed client realtime handler");
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
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
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    let admitted_telegram = prepare_telegram_runtime(&supervisor, &store, admitted_telegram);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    PreparedManagedTelegramFixture {
        root,
        data,
        store,
        supervisor,
        realtime,
        admitted_telegram: Some(admitted_telegram),
    }
}

fn prepare_managed_telegram_real_provider_fixture_v1(
    tdjson: &Path,
    tgcalls: &Path,
    api_id: i64,
    api_hash: &[u8],
    session_encryption_key: &[u8],
) -> PreparedManagedTelegramFixture {
    prepare_managed_telegram_fixture_with_inputs_v1(
        None,
        tdjson,
        tgcalls,
        api_id,
        api_hash,
        session_encryption_key,
    )
}

#[test]
#[ignore = "requires an approved real TDLib user credential contour and release-eligible native artifacts"]
fn managed_telegram_real_tdlib_reaches_qr_authorization() {
    let tdjson = required_regular_artifact_v1(TASK10_REAL_TDJSON_ENV_V1, false);
    assert_eq!(
        tdjson.file_name().and_then(|name| name.to_str()),
        Some("libtdjson.1.8.0.dylib"),
        "Task 10 TDLib gate requires the exact admitted Homebrew TDLib artifact",
    );
    assert_eq!(
        sha256_hex_v1(&std::fs::read(&tdjson).expect("read exact Task 10 TDLib artifact")),
        TASK10_REAL_TDJSON_SHA256_V1,
        "Task 10 TDLib artifact digest is not exact",
    );
    let (tgcalls, _) = task10_release_tgcalls_artifacts_v1();
    let api_id_bytes = read_private_input_file_v1(TASK10_REAL_API_ID_FILE_ENV_V1, 32);
    let api_id = std::str::from_utf8(api_id_bytes.as_slice())
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
        .filter(|value| *value > 0)
        .expect("Task 10 Telegram API ID file must contain one positive integer");
    let api_hash_file = read_private_input_file_v1(TASK10_REAL_API_HASH_FILE_ENV_V1, 128);
    let api_hash = Zeroizing::new(trim_single_line_secret_v1(api_hash_file.as_slice()));
    assert!(
        api_hash.len() == 32
            && api_hash
                .iter()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)),
        "Task 10 Telegram API hash must be one canonical lowercase hexadecimal value",
    );
    let mut session_encryption_key = Zeroizing::new([0_u8; 32]);
    getrandom::fill(session_encryption_key.as_mut())
        .expect("generate fresh Telegram session encryption key");

    let mut fixture = prepare_managed_telegram_real_provider_fixture_v1(
        &tdjson,
        &tgcalls,
        api_id,
        api_hash.as_slice(),
        session_encryption_key.as_slice(),
    );
    let child_stdio = telegram_child_capture_v1(&fixture.root, "real-tdlib-qr");
    let telegram = fixture.start_telegram_with_capture_v1(&child_stdio);
    let storage_credential = runtime_storage_credential_for_registration_v1(
        &fixture.supervisor,
        &fixture.store,
        &fixture.data,
        &telegram.registration_id,
        TELEGRAM_STORAGE_CAPABILITY_ID,
    );
    let relay = fixture.supervisor.relay_port();
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut request_id = 80_000_u64;
    let qr_link = loop {
        request_id = request_id.saturating_add(1);
        match route_telegram_client(
            &fixture.store,
            &relay,
            &telegram,
            TelegramClientContractV1::Authorization,
            request_id,
            &TelegramClientRequest::AuthorizationStatus,
        ) {
            Ok(TelegramClientResponse::AuthorizationStatus(status)) => {
                assert_dynamic_private_bytes_absent_v1(
                    format!("{status:?}").as_bytes(),
                    &[
                        api_hash.as_slice(),
                        session_encryption_key.as_slice(),
                        storage_credential.as_slice(),
                    ],
                    "typed real Telegram authorization response",
                );
                if status.state == "waiting_qr_scan"
                    && let Some(link) = status.qr_link
                {
                    assert!(
                        link.starts_with("tg://login?token=")
                            && link.len() > "tg://login?token=".len(),
                        "TDLib returned a non-canonical Telegram QR authorization link",
                    );
                    break Zeroizing::new(link);
                }
            }
            Err(error) if error.is_retryable() => {}
            Err(_) => panic!("real Telegram authorization status route failed closed"),
            Ok(_) => panic!("real Telegram authorization route returned the wrong response type"),
        }
        assert!(
            Instant::now() < deadline,
            "real TDLib did not reach the bounded QR authorization state",
        );
        std::thread::sleep(Duration::from_millis(100));
    };

    let diagnostic = format!(
        "{:?}",
        fixture
            .supervisor
            .last_failure(&telegram.registration_id)
            .expect("read real Telegram supervisor diagnostic"),
    );
    assert_dynamic_private_bytes_absent_v1(
        diagnostic.as_bytes(),
        &[
            api_hash.as_slice(),
            session_encryption_key.as_slice(),
            qr_link.as_bytes(),
            storage_credential.as_slice(),
        ],
        "real Telegram supervisor diagnostic",
    );
    assert!(
        fixture
            .supervisor
            .request_stop_if_active(&telegram.registration_id)
            .expect("request real Telegram runtime stop"),
        "real Telegram runtime must remain active until the QR gate stops it",
    );
    let stop_started = Instant::now();
    assert!(
        fixture
            .supervisor
            .stop_if_active(&telegram.registration_id)
            .expect("join real Telegram runtime stop"),
        "real Telegram runtime must join after the stop request",
    );
    assert!(
        stop_started.elapsed() < Duration::from_secs(2),
        "real Telegram runtime did not stop within the control deadline",
    );
    clear_telegram_child_capture_v1();
    assert_dynamic_private_durable_surfaces_v1(&[
        api_hash.as_slice(),
        session_encryption_key.as_slice(),
        qr_link.as_bytes(),
        storage_credential.as_slice(),
    ]);
    assert_dynamic_supervised_child_output_is_private_v1(
        &child_stdio,
        &[
            api_hash.as_slice(),
            session_encryption_key.as_slice(),
            qr_link.as_bytes(),
            storage_credential.as_slice(),
        ],
    );
}

#[test]
#[ignore = "requires the pinned release-eligible tgcalls build and explicit local audio-device consent"]
fn managed_telegram_real_tgcalls_audio_device_conformance() {
    let (_, audio_probe) = task10_release_tgcalls_artifacts_v1();
    let mut child = Command::new(audio_probe)
        .arg("--allow-microphone-and-speaker-access")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start pinned tgcalls audio-device conformance probe");
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .expect("poll tgcalls audio-device conformance probe")
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("tgcalls audio-device conformance probe exceeded its bounded deadline");
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    let mut output = Vec::new();
    child
        .stdout
        .take()
        .expect("captured tgcalls audio probe stdout")
        .read_to_end(&mut output)
        .expect("read tgcalls audio probe stdout");
    child
        .stderr
        .take()
        .expect("captured tgcalls audio probe stderr")
        .read_to_end(&mut output)
        .expect("read tgcalls audio probe stderr");
    assert!(
        status.success(),
        "real tgcalls audio-device conformance failed"
    );
    assert!(
        output
            .windows(b"audio-device-conformance: ok".len())
            .any(|window| window == b"audio-device-conformance: ok"),
        "real tgcalls audio-device conformance did not return its exact success marker",
    );
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_runtime_bootstrap_fails_closed_and_stops_promptly() {
    let excluded_query = TelegramClientContractV1::Query.capability_id();
    let mut fixture = prepare_managed_telegram_fixture_without_capability(Some(excluded_query));
    let runtime_dir = fixture.root.join("runtime");
    let admitted = fixture
        .admitted_telegram
        .take()
        .expect("prepared Telegram admission");

    let capture = telegram_child_capture_v1(&fixture.root, "missing-settings");
    let started = launch_telegram_runtime_without_ready_v1(
        &fixture.supervisor,
        &fixture.store,
        &fixture.data,
        &runtime_dir,
        admitted,
        TelegramBootstrapOverrideV1::MissingSettings,
        &capture,
    );
    assert_telegram_pre_spawn_denied_v1(
        &fixture.supervisor,
        &started,
        "missing settings",
        &capture,
    );

    let capture = telegram_child_capture_v1(&fixture.root, "invalid-settings");
    let started = launch_telegram_successor_without_ready_v1(
        &fixture.supervisor,
        &fixture.store,
        &fixture.data,
        &runtime_dir,
        started,
        TelegramBootstrapOverrideV1::InvalidSettingsValue,
        &capture,
    );
    assert_telegram_bounded_runtime_denied_v1(
        &fixture.supervisor,
        &started,
        "invalid settings",
        &capture,
    );

    let capture = telegram_child_capture_v1(&fixture.root, "missing-storage");
    let started = launch_telegram_successor_without_ready_v1(
        &fixture.supervisor,
        &fixture.store,
        &fixture.data,
        &runtime_dir,
        started,
        TelegramBootstrapOverrideV1::MissingStorage,
        &capture,
    );
    assert_telegram_pre_spawn_denied_v1(&fixture.supervisor, &started, "missing Storage", &capture);

    let capture = telegram_child_capture_v1(&fixture.root, "stale-storage");
    let started = launch_telegram_successor_without_ready_v1(
        &fixture.supervisor,
        &fixture.store,
        &fixture.data,
        &runtime_dir,
        started,
        TelegramBootstrapOverrideV1::StaleStorageFence,
        &capture,
    );
    assert_telegram_active_until_requested_stop_v1(
        &fixture.supervisor,
        &started,
        "stale Storage fence",
        &capture,
    );

    let capture = telegram_child_capture_v1(&fixture.root, "stale-vault");
    let started = launch_telegram_successor_without_ready_v1(
        &fixture.supervisor,
        &fixture.store,
        &fixture.data,
        &runtime_dir,
        started,
        TelegramBootstrapOverrideV1::StaleVaultFence,
        &capture,
    );
    assert_telegram_active_until_requested_stop_v1(
        &fixture.supervisor,
        &started,
        "stale Vault fence",
        &capture,
    );

    let capture = telegram_child_capture_v1(&fixture.root, "stale-event");
    let started = launch_telegram_successor_without_ready_v1(
        &fixture.supervisor,
        &fixture.store,
        &fixture.data,
        &runtime_dir,
        started,
        TelegramBootstrapOverrideV1::StaleEventFence,
        &capture,
    );
    assert_telegram_active_until_requested_stop_v1(
        &fixture.supervisor,
        &started,
        "stale Event fence",
        &capture,
    );

    let healthy = fixture.restart_telegram(started);
    let request = encode_module_request(
        68,
        &TelegramClientRequest::Query(TelegramProviderQuery::CachedChats {
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            limit: 1,
        }),
    )
    .expect("encode ungranted Telegram query");
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &healthy.registration_id,
        &healthy.runtime_instance_id,
        healthy.runtime_generation,
        healthy.grant_epoch,
        excluded_query,
        &request,
    );
    assert_eq!(
        crate::modules::capability::router::route_managed_client_request(
            &*fixture.store,
            &fixture.supervisor.relay_port(),
            &route,
        )
        .expect_err("ungranted Telegram query route"),
        "capability is not granted to this registration"
    );
    fixture
        .supervisor
        .stop(&healthy.registration_id)
        .expect("stop healthy Telegram predecessor");

    let native_root = fixture
        .root
        .join("Макошь.app/Contents/Resources/makosh-kernel-release/distribution/lib");
    std::fs::remove_file(native_root.join(TELEGRAM_TDJSON_ARTIFACT_ID))
        .expect("remove signed TDJSON artifact");
    std::fs::remove_file(native_root.join(TELEGRAM_TGCALLS_ARTIFACT_ID))
        .expect("remove signed tgcalls artifact");
    let capture = telegram_child_capture_v1(&fixture.root, "missing-native-artifacts");
    let started = launch_telegram_successor_without_ready_v1(
        &fixture.supervisor,
        &fixture.store,
        &fixture.data,
        &runtime_dir,
        healthy,
        TelegramBootstrapOverrideV1::MissingNativeArtifacts,
        &capture,
    );
    assert_telegram_pre_spawn_denied_v1(
        &fixture.supervisor,
        &started,
        "missing native artifacts",
        &capture,
    );
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_private_surfaces_reject_malformed_provider_output() {
    let mut fixture = prepare_managed_telegram_fixture();
    let child_stdio = telegram_child_capture_v1(&fixture.root, "private-surfaces");
    unsafe {
        std::env::set_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
            &child_stdio,
        );
    }
    let telegram = fixture.start_telegram();
    let storage_credential = runtime_storage_credential_for_registration_v1(
        &fixture.supervisor,
        &fixture.store,
        &fixture.data,
        &telegram.registration_id,
        makosh_telegram_runtime::admission::TELEGRAM_STORAGE_CAPABILITY_ID,
    );
    assert_telegram_lifecycle_query(&fixture.store, &fixture.supervisor, &telegram);
    assert_telegram_account_started(&fixture.store, &fixture.supervisor, &telegram);

    let typed_response_deadline = Instant::now() + Duration::from_secs(10);
    let typed_response = loop {
        match route_telegram_client(
            &fixture.store,
            &fixture.supervisor.relay_port(),
            &telegram,
            TelegramClientContractV1::Lifecycle,
            96,
            &TelegramClientRequest::ListAccounts,
        ) {
            Ok(response) => break response,
            Err(error) if error.is_retryable() => {
                assert!(
                    Instant::now() < typed_response_deadline,
                    "typed Telegram lifecycle response remained unavailable: {error:?}"
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("read typed Telegram lifecycle response: {error:?}"),
        }
    };
    assert_telegram_private_bytes_absent_v1(
        format!("{typed_response:?}").as_bytes(),
        storage_credential.as_slice(),
        "typed Telegram lifecycle response",
    );

    assert_telegram_command_accepted(
        &fixture.store,
        &fixture.supervisor,
        &telegram,
        "task10-private-malformed-provider",
        "managed malformed provider trigger",
    );
    assert_telegram_operation_state_v1("task10-private-malformed-provider", "dead_letter");
    let failed_operation = telegram_operation_response_v1(
        &fixture.store,
        &fixture.supervisor,
        &telegram,
        "task10-private-malformed-provider",
    );
    assert_eq!(failed_operation.state, TelegramOperationState::DeadLetter);
    assert_telegram_private_bytes_absent_v1(
        format!("{failed_operation:?}").as_bytes(),
        storage_credential.as_slice(),
        "typed Telegram operation terminal",
    );
    let diagnostic = format!(
        "{:?}",
        fixture
            .supervisor
            .last_failure(&telegram.registration_id)
            .expect("read malformed Telegram runtime diagnostic")
    );
    assert_telegram_private_bytes_absent_v1(
        diagnostic.as_bytes(),
        storage_credential.as_slice(),
        "Telegram supervisor diagnostic",
    );
    if fixture
        .supervisor
        .request_stop_if_active(&telegram.registration_id)
        .expect("request malformed Telegram runtime stop")
    {
        assert!(
            fixture
                .supervisor
                .stop_if_active(&telegram.registration_id)
                .expect("join malformed Telegram runtime stop")
        );
    }

    assert_telegram_durable_surfaces_are_private_v1(storage_credential.as_slice());
    clear_telegram_child_capture_v1();
    assert_supervised_telegram_child_output_is_private_v1(
        &child_stdio,
        storage_credential.as_slice(),
    );
}

fn telegram_operation_response_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    operation_id: &str,
) -> makosh_telegram_api::TelegramOperation {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match route_telegram_client(
            store,
            &supervisor.relay_port(),
            telegram,
            TelegramClientContractV1::Query,
            97,
            &TelegramClientRequest::Query(TelegramProviderQuery::Operations {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                limit: 16,
            }),
        ) {
            Ok(TelegramClientResponse::Query(TelegramProviderQueryResponse::Operations(
                operations,
            ))) => {
                if let Some(operation) = operations
                    .into_iter()
                    .find(|operation| operation.operation_id == operation_id)
                {
                    return operation;
                }
            }
            Ok(_) => panic!("Telegram operation query returned the wrong response type"),
            Err(error) if error.is_retryable() => {}
            Err(error) => panic!("Telegram operation query failed: {error:?}"),
        }
        assert!(
            Instant::now() < deadline,
            "Telegram typed operation terminal remained unavailable"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_telegram_operation_state_v1(operation_id: &str, expected_state: &str) {
    let runtime = tokio::runtime::Runtime::new().expect("Telegram operation state runtime");
    let pool = runtime.block_on(telegram_admin_pool());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let state = runtime
            .block_on(
                sqlx::query_scalar::<_, String>(
                    "SELECT state FROM makosh_data.telegram_runtime_operations WHERE operation_id = $1",
                )
                .bind(operation_id)
                .fetch_optional(&pool),
            )
            .expect("read Telegram operation state");
        if state.as_deref() == Some(expected_state) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Telegram privacy operation did not reach {expected_state}; last state={state:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_telegram_private_bytes_absent_v1(bytes: &[u8], storage_credential: &[u8], surface: &str) {
    for marker in telegram_private_markers_v1() {
        assert!(
            !bytes.windows(marker.len()).any(|value| value == marker),
            "{surface} exposed a private Telegram marker"
        );
    }
    assert!(
        !bytes
            .windows(storage_credential.len())
            .any(|value| value == storage_credential),
        "{surface} exposed the exact Telegram Storage credential"
    );
}

fn telegram_private_markers_v1() -> [&'static [u8]; 6] {
    [
        TASK10_PRIVATE_BODY_SENTINEL_V1,
        TASK10_RAW_PROVIDER_SENTINEL_V1,
        TASK10_PRIVATE_API_HASH_SENTINEL_V1,
        TASK10_PRIVATE_CALL_CONFIG_SENTINEL_V1,
        TASK10_PRIVATE_CALL_PARAMETERS_SENTINEL_V1,
        TASK10_PRIVATE_PROVIDER_ERROR_SENTINEL_V1,
    ]
}

fn assert_telegram_durable_surfaces_are_private_v1(storage_credential: &[u8]) {
    tokio::runtime::Runtime::new()
        .expect("Telegram privacy database runtime")
        .block_on(async {
            let pool = telegram_admin_pool().await;
            let credential_hex = storage_credential
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            for table in makosh_telegram_persistence::TELEGRAM_OWNER_RLS_TABLES_V1 {
                let rows: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                    "SELECT COALESCE(string_agg(row_to_json(source)::text, E'\\n'), '') \
                     FROM makosh_data.{table} AS source"
                )))
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|error| {
                    panic!("serialize private Telegram table {table}: {error}")
                });
                for marker in telegram_private_markers_v1() {
                    let marker = std::str::from_utf8(marker).expect("ASCII Telegram marker");
                    let marker_hex = marker
                        .as_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>();
                    assert!(
                        !rows.contains(marker) && !rows.contains(&marker_hex),
                        "durable Telegram table {table} exposed a private marker"
                    );
                }
                assert!(
                    !rows.contains(&credential_hex),
                    "durable Telegram table {table} exposed the exact Storage credential"
                );
            }
        });
}

fn assert_supervised_telegram_child_output_is_private_v1(
    directory: &Path,
    storage_credential: &[u8],
) {
    let captures = telegram_child_capture_paths_v1(directory);
    assert!(
        captures.len() >= 2 && captures.len().is_multiple_of(2),
        "Telegram privacy contour must capture complete supervised child attempts"
    );
    for capture in captures {
        let bytes = std::fs::read(capture).expect("read supervised Telegram child output");
        assert_telegram_private_bytes_absent_v1(
            &bytes,
            storage_credential,
            "supervised Telegram child output",
        );
    }
}

fn telegram_child_capture_v1(root: &Path, phase: &str) -> PathBuf {
    private_directory(root.join(format!("telegram-stdio-{phase}")))
}

fn telegram_child_capture_paths_v1(directory: &Path) -> Vec<PathBuf> {
    let mut paths = std::fs::read_dir(directory)
        .expect("read Telegram child capture directory")
        .map(|entry| entry.expect("read Telegram child capture entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn clear_telegram_child_capture_v1() {
    unsafe {
        std::env::remove_var(
            crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
        );
    }
}

fn required_regular_artifact_v1(name: &str, executable: bool) -> PathBuf {
    let path = PathBuf::from(required(name));
    assert!(path.is_absolute(), "{name} must be an absolute path");
    let metadata = std::fs::symlink_metadata(&path)
        .unwrap_or_else(|_| panic!("{name} must name an existing regular file"));
    assert!(
        metadata.file_type().is_file(),
        "{name} must name a regular non-symlink file",
    );
    if executable {
        assert!(
            metadata.permissions().mode() & 0o111 != 0,
            "{name} must name an executable file",
        );
    }
    path
}

fn read_private_input_file_v1(name: &str, maximum_bytes: u64) -> Zeroizing<Vec<u8>> {
    let path = PathBuf::from(required(name));
    assert!(path.is_absolute(), "{name} must be an absolute path");
    let metadata = std::fs::symlink_metadata(&path)
        .unwrap_or_else(|_| panic!("{name} must name an existing private file"));
    assert!(
        metadata.file_type().is_file()
            && metadata.uid() == unsafe { libc::geteuid() }
            && metadata.permissions().mode() & 0o077 == 0
            && (1..=maximum_bytes).contains(&metadata.len()),
        "{name} must be a bounded owner-private regular non-symlink file",
    );
    Zeroizing::new(
        std::fs::read(path).unwrap_or_else(|_| panic!("{name} private file could not be read")),
    )
}

fn trim_single_line_secret_v1(value: &[u8]) -> Vec<u8> {
    let value = value.strip_suffix(b"\n").unwrap_or(value);
    let value = value.strip_suffix(b"\r").unwrap_or(value);
    assert!(
        !value.is_empty() && !value.iter().any(u8::is_ascii_whitespace),
        "Task 10 secret file must contain exactly one non-empty value",
    );
    value.to_vec()
}

fn task10_release_tgcalls_artifacts_v1() -> (PathBuf, PathBuf) {
    let root = PathBuf::from(required(TASK10_REAL_TGCALLS_ROOT_ENV_V1));
    assert!(
        root.is_absolute(),
        "Task 10 tgcalls root must be an absolute path",
    );
    let root_metadata = std::fs::symlink_metadata(&root).expect("Task 10 tgcalls root must exist");
    assert!(
        root_metadata.file_type().is_dir(),
        "Task 10 tgcalls root must be a real directory",
    );
    let bridge = root.join(TASK10_TGCALLS_BRIDGE_V1);
    let audio_probe = root.join(TASK10_TGCALLS_AUDIO_PROBE_V1);
    for (path, executable) in [(&bridge, false), (&audio_probe, true)] {
        let metadata =
            std::fs::symlink_metadata(path).expect("Task 10 tgcalls release artifact must exist");
        assert!(
            metadata.file_type().is_file(),
            "Task 10 tgcalls release artifact must be a regular non-symlink file",
        );
        if executable {
            assert!(
                metadata.permissions().mode() & 0o111 != 0,
                "Task 10 tgcalls audio probe must be executable",
            );
        }
    }
    let provenance_bytes =
        std::fs::read(root.join("provenance.json")).expect("read Task 10 tgcalls provenance");
    assert!(
        provenance_bytes.len() <= 16 * 1024,
        "Task 10 tgcalls provenance exceeds its bounded size",
    );
    let provenance: serde_json::Value =
        serde_json::from_slice(&provenance_bytes).expect("decode Task 10 tgcalls provenance");
    for (field, expected) in [
        ("artifact", TASK10_TGCALLS_BRIDGE_V1),
        (
            "audio_device_conformance_artifact",
            TASK10_TGCALLS_AUDIO_PROBE_V1,
        ),
        ("build_profile", "release"),
        ("platform", "darwin-arm64"),
        ("xcode_version", "26.2"),
        ("xcode_version_pin", "26.2"),
        ("bazel_version", "8.4.2"),
        (
            "telegram_ios_commit",
            "6ad963e5b62d354da79040f388ae2b9132fb17b8",
        ),
        ("tgcalls_commit", "e3069322a3d1e16ecb11a5e302242e59ddd7f09e"),
        ("webrtc_commit", "3817e906cb6c22ec9cc62023b073e1a668d9cb33"),
        ("libvpx_commit", "e7bfd8b6c230a6824e7fd1efa2378a7322986128"),
        ("dav1d_commit", "330e20672e85f9de1678dccd6957845898ef57a1"),
    ] {
        assert_eq!(
            provenance.get(field).and_then(serde_json::Value::as_str),
            Some(expected),
            "Task 10 tgcalls provenance field {field} is not exact",
        );
    }
    assert_eq!(
        provenance
            .get("release_eligible")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "Task 10 tgcalls artifact is not release eligible",
    );
    assert_eq!(
        provenance
            .get("bridge_abi")
            .and_then(serde_json::Value::as_u64),
        Some(1),
        "Task 10 tgcalls bridge ABI is not exact",
    );
    let bridge_sha = sha256_hex_v1(
        &std::fs::read(&bridge).expect("read Task 10 tgcalls bridge for provenance binding"),
    );
    assert_eq!(
        provenance
            .get("artifact_sha256")
            .and_then(serde_json::Value::as_str),
        Some(bridge_sha.as_str()),
        "Task 10 tgcalls provenance does not bind the exact bridge bytes",
    );
    (bridge, audio_probe)
}

fn assert_dynamic_private_bytes_absent_v1(bytes: &[u8], private_values: &[&[u8]], surface: &str) {
    for value in private_values {
        assert!(
            !value.is_empty(),
            "Task 10 private probe value must be non-empty"
        );
        assert!(
            !bytes.windows(value.len()).any(|window| window == *value),
            "{surface} exposed a Task 10 private value",
        );
        let value_hex = value
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let value_base64 = base64::engine::general_purpose::STANDARD.encode(value);
        let value_base64_unpadded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(value);
        for encoded in [&value_hex, &value_base64, &value_base64_unpadded] {
            assert!(
                !bytes
                    .windows(encoded.len())
                    .any(|window| window == encoded.as_bytes()),
                "{surface} exposed an encoded Task 10 private value",
            );
        }
    }
}

fn sha256_hex_v1(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn assert_dynamic_private_durable_surfaces_v1(private_values: &[&[u8]]) {
    tokio::runtime::Runtime::new()
        .expect("real Telegram privacy database runtime")
        .block_on(async {
            let pool = telegram_admin_pool().await;
            for table in makosh_telegram_persistence::TELEGRAM_OWNER_RLS_TABLES_V1 {
                let rows: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                    "SELECT COALESCE(string_agg(row_to_json(source)::text, E'\\n'), '') \
                     FROM makosh_data.{table} AS source"
                )))
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|error| panic!("serialize real Telegram table {table}: {error}"));
                for value in private_values {
                    assert_dynamic_private_bytes_absent_v1(
                        rows.as_bytes(),
                        &[*value],
                        "real Telegram durable row",
                    );
                }
            }
        });
}

fn assert_dynamic_supervised_child_output_is_private_v1(
    directory: &Path,
    private_values: &[&[u8]],
) {
    let captures = telegram_child_capture_paths_v1(directory);
    assert_eq!(
        captures.len(),
        2,
        "real Telegram QR gate must capture exactly one supervised child attempt",
    );
    for capture in captures {
        let bytes = std::fs::read(capture).expect("read real Telegram supervised child output");
        assert_dynamic_private_bytes_absent_v1(
            &bytes,
            private_values,
            "real Telegram supervised child output",
        );
    }
}

fn assert_telegram_pre_spawn_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedTelegramRuntime,
    phase: &str,
    capture: &Path,
) {
    assert!(
        !matches!(
            supervisor.relay_port().is_ready(&started.registration_id),
            Ok(true)
        ),
        "{phase} must not signal Ready"
    );
    assert!(
        !supervisor
            .is_active(&started.registration_id)
            .expect("Telegram pre-spawn activity"),
        "{phase} must be denied before child spawn"
    );
    assert!(
        telegram_child_capture_paths_v1(capture).is_empty(),
        "{phase} must not create supervised child output"
    );
    clear_telegram_child_capture_v1();
}

fn assert_telegram_bounded_runtime_denied_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedTelegramRuntime,
    phase: &str,
    capture: &Path,
) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while supervisor
        .is_active(&started.registration_id)
        .expect("Telegram bounded denial activity")
    {
        assert!(Instant::now() < deadline, "{phase} did not terminate");
        assert!(
            !matches!(
                supervisor.relay_port().is_ready(&started.registration_id),
                Ok(true)
            ),
            "{phase} must not signal Ready"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let captures = telegram_child_capture_paths_v1(capture);
    assert!(
        captures.len() >= 2 && captures.len().is_multiple_of(2),
        "{phase} must have bounded complete supervised child attempts"
    );
    clear_telegram_child_capture_v1();
}

fn assert_telegram_active_until_requested_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedTelegramRuntime,
    phase: &str,
    capture: &Path,
) {
    let deadline = Instant::now() + Duration::from_millis(100);
    while Instant::now() < deadline {
        assert!(
            supervisor
                .is_active(&started.registration_id)
                .expect("Telegram bootstrap activity"),
            "{phase} child exited before requested stop"
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
    let stopped_at = Instant::now();
    assert!(
        supervisor
            .request_stop_if_active(&started.registration_id)
            .expect("request Telegram bootstrap stop"),
        "{phase} must own the active child"
    );
    assert!(
        supervisor
            .stop_if_active(&started.registration_id)
            .expect("join Telegram bootstrap stop"),
        "{phase} requested child must join"
    );
    assert!(stopped_at.elapsed() < Duration::from_secs(2));
    assert!(
        !supervisor
            .is_active(&started.registration_id)
            .expect("Telegram stopped activity"),
        "{phase} must not install a replacement"
    );
    assert_eq!(
        telegram_child_capture_paths_v1(capture).len(),
        2,
        "{phase} must spawn exactly one supervised child"
    );
    clear_telegram_child_capture_v1();
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_runtime_uses_kernel_leases_and_event_only_communications_handoff() {
    let mut fixture = prepare_managed_telegram_fixture();
    let store = Arc::clone(&fixture.store);
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let event_runtime = tokio::runtime::Runtime::new().expect("Event observer runtime");
    let _event_runtime_context = event_runtime.enter();
    let (mut observations, mut canonical_events) = event_runtime.block_on(async {
        let client = async_nats::connect(events.nats_endpoint())
            .await
            .expect("connect event observer");
        let observations = client
            .subscribe("makosh.observation.v1.communications.communication_observed.v1")
            .await
            .expect("subscribe Telegram observations");
        let canonical_events = client
            .subscribe("makosh.event.v1.communications.communication_evidence_recorded.v1")
            .await
            .expect("subscribe canonical Communications events");
        (observations, canonical_events)
    });

    let telegram = fixture.start_telegram();
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    assert_telegram_account_started(&store, &fixture.supervisor, &telegram);

    let (observation, canonical) = event_runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .expect("managed Telegram observation timeout")
            .expect("managed Telegram observation");
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .expect("canonical Communications event timeout")
            .expect("canonical Communications event");
        (observation, canonical)
    });
    let observation_bytes = observation.payload.to_vec();
    let observation =
        decode_envelope_v1(&observation_bytes).expect("Telegram observation envelope");
    assert_eq!(
        observation
            .source
            .expect("Telegram observation source")
            .module_id,
        makosh_telegram_runtime::PACKAGE
    );
    let canonical =
        decode_envelope_v1(canonical.payload.as_ref()).expect("Communications event envelope");
    assert_eq!(
        canonical.causation_message_id, observation.message_id,
        "Communications must derive canonical evidence only from the typed Telegram observation"
    );
    event_runtime.block_on(async {
        let client = async_nats::connect(events.nats_endpoint())
            .await
            .expect("connect duplicate observation publisher");
        client
            .publish(
                "makosh.observation.v1.communications.communication_observed.v1",
                observation_bytes.into(),
            )
            .await
            .expect("republish exact Telegram observation");
        client.flush().await.expect("flush duplicate observation");
        let duplicate_observation =
            tokio::time::timeout(Duration::from_secs(1), observations.next())
                .await
                .expect("duplicate Telegram observation timeout")
                .expect("duplicate Telegram observation");
        let duplicate_observation = decode_envelope_v1(duplicate_observation.payload.as_ref())
            .expect("duplicate Telegram observation envelope");
        assert_eq!(
            duplicate_observation.message_id, observation.message_id,
            "the observer must drain the exact duplicate before the outage replay"
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(1), canonical_events.next())
                .await
                .is_err(),
            "duplicate Telegram observation must not create a second Communications event"
        );
    });
    let initial_evidence_id = assert_communications_query_delivery(&store, &fixture.supervisor);

    set_authenticated_nats_container_running(false);
    const OUTAGE_OPERATION_ID: &str = "managed-telegram-outage-send-1";
    assert_telegram_command_accepted(
        &store,
        &fixture.supervisor,
        &telegram,
        OUTAGE_OPERATION_ID,
        "managed Telegram outage replay trigger",
    );
    assert_telegram_operation_completed(
        &store,
        &fixture.supervisor,
        &telegram,
        OUTAGE_OPERATION_ID,
    );
    std::thread::sleep(Duration::from_millis(2_500));
    assert_telegram_operation_completed(
        &store,
        &fixture.supervisor,
        &telegram,
        OUTAGE_OPERATION_ID,
    );
    set_authenticated_nats_container_running(true);

    let (replayed_observation, replayed_canonical) = event_runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .expect("replayed Telegram observation timeout")
            .expect("replayed Telegram observation");
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .expect("replayed Communications event timeout")
            .expect("replayed Communications event");
        (observation, canonical)
    });
    let replayed_observation = decode_envelope_v1(replayed_observation.payload.as_ref())
        .expect("replayed Telegram observation envelope");
    let replayed_canonical = decode_envelope_v1(replayed_canonical.payload.as_ref())
        .expect("replayed Communications event envelope");
    assert_eq!(
        replayed_canonical.causation_message_id, replayed_observation.message_id,
        "Communications replay must retain typed Telegram causation"
    );
    assert_ne!(
        replayed_canonical.message_id, canonical.message_id,
        "the outage replay must deliver the second provider observation"
    );
    let replayed_evidence_id = assert_communications_query_delivery(&store, &fixture.supervisor);
    assert_ne!(
        replayed_evidence_id, initial_evidence_id,
        "Communications durable query must expose the replayed evidence"
    );
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_realtime_route_requires_exact_grant() {
    let realtime_capability = TelegramClientContractV1::Realtime.capability_id();
    let mut fixture =
        prepare_managed_telegram_fixture_without_capability(Some(realtime_capability));
    let store = Arc::clone(&fixture.store);
    let telegram = fixture.start_telegram();
    let request = encode_module_request(
        69,
        &TelegramClientRequest::Replay {
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            after_sequence: 0,
            limit: 10,
        },
    )
    .expect("encode ungranted Telegram realtime request");
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &telegram.registration_id,
        &telegram.runtime_instance_id,
        telegram.runtime_generation,
        telegram.grant_epoch,
        realtime_capability,
        &request,
    );
    assert_eq!(
        crate::modules::capability::router::route_managed_client_request(
            &*store,
            &fixture.supervisor.relay_port(),
            &route,
        )
        .expect_err("ungranted Telegram realtime route"),
        "capability is not granted to this registration"
    );
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_reconfiguration_route_requires_exact_grant() {
    let capability = TelegramClientContractV1::Reconfiguration.capability_id();
    let mut fixture = prepare_managed_telegram_fixture_without_capability(Some(capability));
    let store = Arc::clone(&fixture.store);
    let telegram = fixture.start_telegram();
    let request = encode_module_request(
        70,
        &TelegramClientRequest::Reconfiguration(TelegramRuntimeReconfigurationRequest::Begin {
            reconfiguration_id: "ungranted-reconfiguration".to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            expected_runtime_epoch: 1,
        }),
    )
    .expect("encode ungranted Telegram reconfiguration request");
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &telegram.registration_id,
        &telegram.runtime_instance_id,
        telegram.runtime_generation,
        telegram.grant_epoch,
        capability,
        &request,
    );
    assert_eq!(
        crate::modules::capability::router::route_managed_client_request(
            &*store,
            &fixture.supervisor.relay_port(),
            &route,
        )
        .expect_err("ungranted Telegram reconfiguration route"),
        "capability is not granted to this registration"
    );
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_runtime_reconfiguration_replaces_provider_session_once() {
    let mut fixture = prepare_managed_telegram_fixture();
    let store = Arc::clone(&fixture.store);
    let telegram = fixture.start_telegram();
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    assert_telegram_account_started(&store, &fixture.supervisor, &telegram);

    assert_telegram_command_accepted(
        &store,
        &fixture.supervisor,
        &telegram,
        "managed-telegram-reconfiguration-seed",
        "managed Telegram operational fixture trigger",
    );
    assert_telegram_operation_completed(
        &store,
        &fixture.supervisor,
        &telegram,
        "managed-telegram-reconfiguration-seed",
    );
    wait_for_telegram_folder_ids(&store, &fixture.supervisor, &telegram, &[7, 9]);

    const BEFORE_ID: &str = "managed-telegram-reconfiguration-before";
    assert_telegram_provider_command_accepted(
        &store,
        &fixture.supervisor,
        &telegram,
        88,
        BEFORE_ID,
        TelegramProviderCommand::ReassignChatFolders {
            operation_id: BEFORE_ID.to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            provider_chat_id: "9002".to_owned(),
            target_provider_folder_ids: vec![9, 11],
        },
    );
    let before =
        assert_telegram_operation_completed(&store, &fixture.supervisor, &telegram, BEFORE_ID);
    assert!(
        before.retry_count >= 1,
        "first provider client must expose the fixture's one ambiguous add"
    );

    const RECONFIGURATION_ID: &str = "managed-telegram-reconfiguration-1";
    let accepted = route_telegram_client(
        &store,
        &fixture.supervisor.relay_port(),
        &telegram,
        TelegramClientContractV1::Reconfiguration,
        89,
        &TelegramClientRequest::Reconfiguration(TelegramRuntimeReconfigurationRequest::Begin {
            reconfiguration_id: RECONFIGURATION_ID.to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            expected_runtime_epoch: 1,
        }),
    )
    .expect("accept Telegram runtime reconfiguration");
    let TelegramClientResponse::Reconfiguration(accepted) = accepted else {
        panic!("Telegram reconfiguration returned the wrong response type");
    };
    assert_eq!(
        accepted.state,
        TelegramRuntimeReconfigurationState::Accepted
    );
    assert_eq!(accepted.expected_runtime_epoch, 1);
    assert_eq!(accepted.target_runtime_epoch, 2);

    let completed = wait_for_telegram_runtime_reconfiguration(
        &store,
        &fixture.supervisor,
        &telegram,
        RECONFIGURATION_ID,
    );
    assert_eq!(
        completed.state,
        TelegramRuntimeReconfigurationState::Completed
    );
    assert_eq!(completed.target_runtime_epoch, 2);

    let exact_retry = route_telegram_reconfiguration_until_ready(
        &store,
        &fixture.supervisor,
        &telegram,
        90,
        &TelegramClientRequest::Reconfiguration(TelegramRuntimeReconfigurationRequest::Begin {
            reconfiguration_id: RECONFIGURATION_ID.to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            expected_runtime_epoch: 1,
        }),
    )
    .expect("read exact reconfiguration retry");
    assert_eq!(
        exact_retry,
        TelegramClientResponse::Reconfiguration(completed.clone())
    );

    let stale = route_telegram_reconfiguration_until_ready(
        &store,
        &fixture.supervisor,
        &telegram,
        91,
        &TelegramClientRequest::Reconfiguration(TelegramRuntimeReconfigurationRequest::Begin {
            reconfiguration_id: "managed-telegram-reconfiguration-stale".to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            expected_runtime_epoch: 1,
        }),
    )
    .expect_err("stale runtime epoch must fail before another provider replacement");
    assert!(matches!(
        stale,
        TelegramClientRouteError::Client(TelegramClientPortError::Protocol(code))
            if code == "RUNTIME_EPOCH_CONFLICT"
    ));

    const AFTER_ID: &str = "managed-telegram-reconfiguration-after";
    assert_telegram_provider_command_accepted(
        &store,
        &fixture.supervisor,
        &telegram,
        92,
        AFTER_ID,
        TelegramProviderCommand::ReassignChatFolders {
            operation_id: AFTER_ID.to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            provider_chat_id: "9002".to_owned(),
            target_provider_folder_ids: vec![9, 11],
        },
    );
    let after =
        assert_telegram_operation_completed(&store, &fixture.supervisor, &telegram, AFTER_ID);
    assert!(
        after.retry_count >= 1,
        "fresh TDLib client must expose a fresh one-shot ambiguous add"
    );
    wait_for_telegram_folder_ids(&store, &fixture.supervisor, &telegram, &[9, 11]);
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_runtime_reconfiguration_recovers_same_epoch_after_process_crash() {
    let mut fixture = prepare_managed_telegram_fixture();
    let store = Arc::clone(&fixture.store);
    let mut telegram = fixture.start_telegram();
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    assert_telegram_account_started(&store, &fixture.supervisor, &telegram);

    const RECONFIGURATION_ID: &str = "managed-telegram-reconfiguration-crash";
    let accepted = route_telegram_client(
        &store,
        &fixture.supervisor.relay_port(),
        &telegram,
        TelegramClientContractV1::Reconfiguration,
        94,
        &TelegramClientRequest::Reconfiguration(TelegramRuntimeReconfigurationRequest::Begin {
            reconfiguration_id: RECONFIGURATION_ID.to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            expected_runtime_epoch: 1,
        }),
    )
    .expect("accept Telegram runtime reconfiguration before process replacement");
    let TelegramClientResponse::Reconfiguration(accepted) = accepted else {
        panic!("Telegram reconfiguration returned the wrong response type");
    };
    assert_eq!(
        accepted.state,
        TelegramRuntimeReconfigurationState::Accepted
    );
    assert_eq!(accepted.target_runtime_epoch, 2);

    let predecessor_generation = telegram.runtime_generation;
    telegram = fixture.restart_telegram(telegram);
    assert_eq!(telegram.runtime_generation, predecessor_generation + 1);

    let completed = wait_for_telegram_runtime_reconfiguration(
        &store,
        &fixture.supervisor,
        &telegram,
        RECONFIGURATION_ID,
    );
    assert_eq!(
        completed.state,
        TelegramRuntimeReconfigurationState::Completed
    );
    assert_eq!(completed.expected_runtime_epoch, 1);
    assert_eq!(completed.target_runtime_epoch, 2);
    assert_telegram_account_runtime_epoch(&store, &fixture.supervisor, &telegram, 2);

    let exact_retry = route_telegram_reconfiguration_until_ready(
        &store,
        &fixture.supervisor,
        &telegram,
        95,
        &TelegramClientRequest::Reconfiguration(TelegramRuntimeReconfigurationRequest::Begin {
            reconfiguration_id: RECONFIGURATION_ID.to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            expected_runtime_epoch: 1,
        }),
    )
    .expect("read crash-recovered Telegram reconfiguration");
    assert_eq!(
        exact_retry,
        TelegramClientResponse::Reconfiguration(completed.clone())
    );

    let recovered_generation = telegram.runtime_generation;
    telegram = fixture.restart_telegram(telegram);
    assert_eq!(telegram.runtime_generation, recovered_generation + 1);
    assert_telegram_account_runtime_epoch(&store, &fixture.supervisor, &telegram, 2);
    let persisted = wait_for_telegram_runtime_reconfiguration(
        &store,
        &fixture.supervisor,
        &telegram,
        RECONFIGURATION_ID,
    );
    assert_eq!(persisted, completed);
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_core_operational_projection_is_restart_safe() {
    let mut fixture = prepare_managed_telegram_fixture();
    let store = Arc::clone(&fixture.store);
    let mut telegram = fixture.start_telegram();
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    assert_telegram_account_started(&store, &fixture.supervisor, &telegram);

    const OPERATIONAL_OPERATION_ID: &str = "managed-telegram-core-operational-1";
    assert_telegram_command_accepted(
        &store,
        &fixture.supervisor,
        &telegram,
        OPERATIONAL_OPERATION_ID,
        "managed Telegram operational fixture trigger",
    );
    assert_telegram_operation_completed(
        &store,
        &fixture.supervisor,
        &telegram,
        OPERATIONAL_OPERATION_ID,
    );
    let replay_cursor = assert_telegram_core_operational(
        &store,
        &fixture.supervisor,
        &telegram,
        OPERATIONAL_OPERATION_ID,
    );
    let predecessor_generation = telegram.runtime_generation;
    telegram = fixture.restart_telegram(telegram);
    assert_eq!(telegram.runtime_generation, predecessor_generation + 1);
    assert_telegram_core_operational_after_restart(
        &store,
        &fixture.supervisor,
        &telegram,
        replay_cursor,
    );
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_folder_reassignment_converges_after_partial_provider_failure() {
    let mut fixture = prepare_managed_telegram_fixture();
    let store = Arc::clone(&fixture.store);
    let mut telegram = fixture.start_telegram();
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    assert_telegram_account_started(&store, &fixture.supervisor, &telegram);

    assert_telegram_command_accepted(
        &store,
        &fixture.supervisor,
        &telegram,
        "managed-telegram-folder-seed",
        "managed Telegram operational fixture trigger",
    );
    assert_telegram_operation_completed(
        &store,
        &fixture.supervisor,
        &telegram,
        "managed-telegram-folder-seed",
    );
    wait_for_telegram_folder_ids(&store, &fixture.supervisor, &telegram, &[7, 9]);

    const OPERATION_ID: &str = "managed-telegram-folder-reassign-retry";
    assert_telegram_provider_command_accepted(
        &store,
        &fixture.supervisor,
        &telegram,
        87,
        OPERATION_ID,
        TelegramProviderCommand::ReassignChatFolders {
            operation_id: OPERATION_ID.to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            provider_chat_id: "9002".to_owned(),
            target_provider_folder_ids: vec![11, 9],
        },
    );
    let completed =
        assert_telegram_operation_completed(&store, &fixture.supervisor, &telegram, OPERATION_ID);
    assert!(
        completed.retry_count >= 1,
        "ambiguous provider failure must reuse the durable operation through retry"
    );
    wait_for_telegram_folder_ids(&store, &fixture.supervisor, &telegram, &[9, 11]);

    let predecessor_generation = telegram.runtime_generation;
    telegram = fixture.restart_telegram(telegram);
    assert_eq!(telegram.runtime_generation, predecessor_generation + 1);
    wait_for_telegram_folder_ids(&store, &fixture.supervisor, &telegram, &[9, 11]);
    let restored =
        assert_telegram_operation_completed(&store, &fixture.supervisor, &telegram, OPERATION_ID);
    assert_eq!(restored.retry_count, completed.retry_count);
}

fn assert_telegram_core_operational(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    operation_id: &str,
) -> u64 {
    let deleted_message_id = format!("telegram:{TELEGRAM_ACCOUNT_ID}:9002:7101");
    wait_for_telegram_tombstone(store, supervisor, telegram, &deleted_message_id);

    let search = telegram_query(
        store,
        supervisor,
        telegram,
        71,
        TelegramProviderQuery::SearchMessages {
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            provider_chat_id: Some("9002".to_owned()),
            query: "edited operational fixture".to_owned(),
            limit: 10,
        },
    );
    let message = match search {
        TelegramProviderQueryResponse::CachedMessages(messages) if messages.len() == 1 => {
            messages.into_iter().next().expect("searched message")
        }
        response => panic!("unexpected Telegram search response: {response:?}"),
    };
    assert_eq!(message.provider_message_id, "7100");
    assert_eq!(message.text.as_deref(), Some("edited operational fixture"));

    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            72,
            TelegramProviderQuery::MessageVersions {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                message_id: message.message_id.clone(),
            },
        ),
        TelegramProviderQueryResponse::MessageVersions(versions)
            if versions.len() >= 2
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            73,
            TelegramProviderQuery::AttachmentForMessage {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
                provider_message_id: "7100".to_owned(),
            },
        ),
        TelegramProviderQueryResponse::Attachment(Some(attachment))
            if attachment.provider_file_id == "42"
                && attachment.filename.as_deref() == Some("report.pdf")
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            74,
            TelegramProviderQuery::File {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_file_id: "42".to_owned(),
            },
        ),
        TelegramProviderQueryResponse::File(Some(file))
            if file.is_downloaded && file.provider_unique_id.as_deref() == Some("managed-file-42")
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            75,
            TelegramProviderQuery::ReactionSummary {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
                provider_message_id: "7100".to_owned(),
            },
        ),
        TelegramProviderQueryResponse::ReactionSummary(summary)
            if summary.len() == 1 && summary[0].emoji == "ok" && summary[0].count == 1
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            76,
            TelegramProviderQuery::PinnedMessages {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
                limit: 10,
            },
        ),
        TelegramProviderQueryResponse::CachedMessages(messages)
            if messages.iter().any(|value| value.provider_message_id == "7100")
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            77,
            TelegramProviderQuery::ChatPositions {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
            },
        ),
        TelegramProviderQueryResponse::ChatPositions(positions)
            if positions.len() == 2
                && positions.iter().any(|position|
                    position.provider_folder_id == Some(7) && position.is_pinned
                )
                && positions.iter().any(|position|
                    position.provider_folder_id == Some(9) && !position.is_pinned
                )
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            78,
            TelegramProviderQuery::ChatOperationalState {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
            },
        ),
        TelegramProviderQueryResponse::ChatOperationalState(Some(state))
            if state.is_pinned && state.is_muted && state.mute_for_seconds == 3600
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            79,
            TelegramProviderQuery::MessageTombstones {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                message_id: deleted_message_id,
            },
        ),
        TelegramProviderQueryResponse::MessageTombstones(tombstones)
            if tombstones.len() == 1 && tombstones[0].is_provider_delete
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            80,
            TelegramProviderQuery::Operations {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                limit: 100,
            },
        ),
        TelegramProviderQueryResponse::Operations(operations)
            if operations.iter().any(
                |operation| operation.operation_id == operation_id
                    && operation.state == TelegramOperationState::Completed
            )
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            81,
            TelegramProviderQuery::Commands {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: Some("9001".to_owned()),
                provider_message_id: None,
                command_kinds: vec!["send_text".to_owned()],
                limit: 100,
            },
        ),
        TelegramProviderQueryResponse::Commands(commands)
            if commands.iter().any(|record| record.operation.operation_id == operation_id)
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            82,
            TelegramProviderQuery::LoadHistory {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
                from_message_id: None,
                mode: TelegramHistorySyncMode::Latest,
                limit: 10,
            },
        ),
        TelegramProviderQueryResponse::HistoryPage(page)
            if page.items.len() == 1
                && page.items[0].text.as_deref() == Some("managed Telegram history fixture")
    ));

    let replay = telegram_replay(store, supervisor, telegram, 83, 0);
    assert!(!replay.is_empty());
    assert!(
        replay
            .windows(2)
            .all(|pair| pair[0].sequence < pair[1].sequence),
        "Telegram operational replay must be strictly ascending",
    );
    replay.last().expect("Telegram replay cursor").sequence
}

fn wait_for_telegram_tombstone(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    message_id: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match telegram_query_result(
            store,
            supervisor,
            telegram,
            70,
            TelegramProviderQuery::MessageTombstones {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                message_id: message_id.to_owned(),
            },
        ) {
            Ok(TelegramProviderQueryResponse::MessageTombstones(tombstones))
                if !tombstones.is_empty() =>
            {
                return;
            }
            Ok(_) => {}
            Err(error) if error.is_retryable() => {}
            Err(error) => panic!("Telegram operational projection failed: {error:?}"),
        }
        assert!(
            Instant::now() < deadline,
            "Telegram operational projection did not reach its terminal tombstone",
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_telegram_core_operational_after_restart(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    replay_cursor: u64,
) {
    let message = match telegram_query(
        store,
        supervisor,
        telegram,
        84,
        TelegramProviderQuery::SearchMessages {
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            provider_chat_id: Some("9002".to_owned()),
            query: "edited operational fixture".to_owned(),
            limit: 10,
        },
    ) {
        TelegramProviderQueryResponse::CachedMessages(messages) if messages.len() == 1 => {
            messages.into_iter().next().expect("restored message")
        }
        response => panic!("unexpected restored Telegram search response: {response:?}"),
    };
    assert_eq!(message.provider_message_id, "7100");
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            85,
            TelegramProviderQuery::MessageVersions {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                message_id: message.message_id,
            },
        ),
        TelegramProviderQueryResponse::MessageVersions(versions)
            if versions.len() >= 2
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            86,
            TelegramProviderQuery::AttachmentForMessage {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
                provider_message_id: "7100".to_owned(),
            },
        ),
        TelegramProviderQueryResponse::Attachment(Some(attachment))
            if attachment.provider_file_id == "42"
                && attachment.filename.as_deref() == Some("report.pdf")
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            87,
            TelegramProviderQuery::File {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_file_id: "42".to_owned(),
            },
        ),
        TelegramProviderQueryResponse::File(Some(file)) if file.is_downloaded
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            88,
            TelegramProviderQuery::ReactionSummary {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
                provider_message_id: "7100".to_owned(),
            },
        ),
        TelegramProviderQueryResponse::ReactionSummary(summary)
            if summary.len() == 1 && summary[0].emoji == "ok" && summary[0].count == 1
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            89,
            TelegramProviderQuery::PinnedMessages {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
                limit: 10,
            },
        ),
        TelegramProviderQueryResponse::CachedMessages(messages)
            if messages.iter().any(|value| value.provider_message_id == "7100")
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            90,
            TelegramProviderQuery::ChatPositions {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
            },
        ),
        TelegramProviderQueryResponse::ChatPositions(positions)
            if positions.len() == 2
                && positions.iter().any(|position|
                    position.provider_folder_id == Some(7) && position.is_pinned
                )
                && positions.iter().any(|position|
                    position.provider_folder_id == Some(9) && !position.is_pinned
                )
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            91,
            TelegramProviderQuery::ChatOperationalState {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
            },
        ),
        TelegramProviderQueryResponse::ChatOperationalState(Some(state))
            if state.is_pinned && state.is_muted && state.mute_for_seconds == 3600
    ));
    assert!(matches!(
        telegram_query(
            store,
            supervisor,
            telegram,
            92,
            TelegramProviderQuery::MessageTombstones {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                message_id: format!("telegram:{TELEGRAM_ACCOUNT_ID}:9002:7101"),
            },
        ),
        TelegramProviderQueryResponse::MessageTombstones(tombstones)
            if tombstones.len() == 1 && tombstones[0].is_provider_delete
    ));
    assert!(
        telegram_replay(store, supervisor, telegram, 93, 0)
            .iter()
            .any(|frame| frame.sequence == replay_cursor),
        "successor Telegram runtime must replay the predecessor cursor",
    );
    let reset = telegram_replay_page(
        store,
        supervisor,
        telegram,
        94,
        replay_cursor.saturating_add(10_000),
    );
    assert!(reset.reset_required);
    assert!(reset.frames.is_empty());
    assert_eq!(reset.next_after_sequence, reset.latest_sequence);
    assert!(matches!(
        route_telegram_client(
            store,
            &supervisor.relay_port(),
            telegram,
            TelegramClientContractV1::Realtime,
            95,
            &TelegramClientRequest::Replay {
                account_id: "another-account".to_owned(),
                after_sequence: 0,
                limit: 10,
            },
        ),
        Err(TelegramClientRouteError::Client(
            TelegramClientPortError::Protocol(error)
        )) if error == "INVALID_ARGUMENT"
    ));
    assert!(
        telegram_replay(store, supervisor, telegram, 96, 0)
            .iter()
            .any(|frame| frame.sequence == replay_cursor),
        "invalid account scope must not terminate the Telegram runtime",
    );
}

fn telegram_query(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    request_id: u64,
    query: TelegramProviderQuery,
) -> TelegramProviderQueryResponse {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match telegram_query_result(store, supervisor, telegram, request_id, query.clone()) {
            Ok(response) => return response,
            Err(error) if error.is_retryable() => {
                assert!(
                    Instant::now() < deadline,
                    "Telegram query remained unavailable after restart: {error:?}; child failure: {:?}",
                    supervisor.last_failure(&telegram.registration_id),
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("Telegram query failed: {error:?}"),
        }
    }
}

fn telegram_query_result(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    request_id: u64,
    query: TelegramProviderQuery,
) -> Result<TelegramProviderQueryResponse, TelegramClientRouteError> {
    match route_telegram_client(
        store,
        &supervisor.relay_port(),
        telegram,
        TelegramClientContractV1::Query,
        request_id,
        &TelegramClientRequest::Query(query),
    )? {
        TelegramClientResponse::Query(response) => Ok(response),
        response => Err(TelegramClientRouteError::Kernel(format!(
            "unexpected Telegram query response: {response:?}"
        ))),
    }
}

fn telegram_replay(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    request_id: u64,
    after_sequence: u64,
) -> Vec<makosh_telegram_api::TelegramRealtimeFrame> {
    let page = telegram_replay_page(store, supervisor, telegram, request_id, after_sequence);
    if page.reset_required {
        panic!(
            "Telegram replay unexpectedly required reset: earliest={}, latest={}",
            page.earliest_available_sequence, page.latest_sequence
        );
    }
    page.frames
}

fn telegram_replay_page(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    request_id: u64,
    after_sequence: u64,
) -> makosh_telegram_api::TelegramRealtimeReplayPage {
    match route_telegram_client(
        store,
        &supervisor.relay_port(),
        telegram,
        TelegramClientContractV1::Realtime,
        request_id,
        &TelegramClientRequest::Replay {
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            after_sequence,
            limit: 5_000,
        },
    )
    .unwrap_or_else(|error| panic!("Telegram replay failed: {error:?}"))
    {
        TelegramClientResponse::Realtime(page) => page,
        response => panic!("unexpected Telegram replay response: {response:?}"),
    }
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_automation_route_is_durable_and_provider_side_effect_free() {
    let mut fixture = prepare_managed_telegram_fixture();
    let store = Arc::clone(&fixture.store);
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let event_runtime = tokio::runtime::Runtime::new().expect("Event observer runtime");
    let _event_runtime_context = event_runtime.enter();
    let (mut observations, mut canonical_events) = event_runtime.block_on(async {
        let client = async_nats::connect(events.nats_endpoint())
            .await
            .expect("connect automation event observer");
        let observations = client
            .subscribe("makosh.observation.v1.communications.communication_observed.v1")
            .await
            .expect("subscribe Telegram observations");
        let canonical_events = client
            .subscribe("makosh.event.v1.communications.communication_evidence_recorded.v1")
            .await
            .expect("subscribe canonical Communications events");
        (observations, canonical_events)
    });

    let telegram = fixture.start_telegram();
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    assert_telegram_account_started(&store, &fixture.supervisor, &telegram);
    let baseline_observation = event_runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(10), observations.next()).await
    });
    let baseline_observation = baseline_observation.unwrap_or_else(|error| {
        panic!(
            "baseline Telegram observation timeout: {error:?}; active={:?}; failure={:?}",
            fixture.supervisor.is_active(&telegram.registration_id),
            fixture.supervisor.last_failure(&telegram.registration_id),
        );
    });
    let baseline_observation = baseline_observation.expect("baseline Telegram observation");
    let baseline_canonical = event_runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .expect("baseline Communications event timeout")
            .expect("baseline Communications event")
    });
    let baseline_observation = decode_envelope_v1(baseline_observation.payload.as_ref())
        .expect("baseline Telegram observation envelope");
    let baseline_canonical = decode_envelope_v1(baseline_canonical.payload.as_ref())
        .expect("baseline Communications event envelope");
    assert_eq!(
        baseline_canonical.causation_message_id, baseline_observation.message_id,
        "baseline Communications event must derive from the startup Telegram observation"
    );

    let automation = assert_telegram_automation_management(&store, &fixture.supervisor, &telegram);
    event_runtime.block_on(async {
        assert!(
            tokio::time::timeout(Duration::from_millis(500), observations.next())
                .await
                .is_err(),
            "Telegram automation preview must not emit a provider observation"
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(500), canonical_events.next())
                .await
                .is_err(),
            "Telegram automation preview must not create Communications evidence"
        );
    });

    let stale_runtime = telegram.clone();
    let telegram = fixture.restart_telegram(telegram);
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    let replay = route_telegram_automation_until_ready(
        &store,
        &fixture.supervisor,
        &telegram,
        TelegramAutomationContractV1::Command,
        81,
        &automation.template_request,
    );
    assert_eq!(
        replay, automation.template_response,
        "Telegram automation retry must replay exact response bytes after process restart"
    );
    assert_automation_query_projection(
        &store,
        &fixture.supervisor,
        &telegram,
        &automation.template,
        &automation.policy,
        &automation.preview,
    );
    let stale_query = AutomationQueryRequestV1 {
        request: Some(automation_query_request_v1::Request::ListTemplates(
            ListAutomationTemplatesQueryV1 {
                limit: 10,
                after_template_id: String::new(),
            },
        )),
    }
    .encode_to_vec();
    assert!(matches!(
        route_telegram_automation_client(
            &store,
            &fixture.supervisor.relay_port(),
            &stale_runtime,
            TelegramAutomationContractV1::Query,
            90,
            &stale_query,
        ),
        Err(TelegramClientRouteError::Kernel(error))
            if error == "managed runtime fence is stale"
    ));
    // Admission-grade NOBYPASSRLS proof covers the complete 46-table store.
    assert_telegram_owner_rls_v1("makosh_storage_authenticated");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_call_history_route_is_durable_and_replayable() {
    let mut fixture = prepare_managed_telegram_fixture();
    let store = Arc::clone(&fixture.store);
    seed_telegram_legacy_call_frame();
    let telegram = fixture.start_telegram();
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);

    let list_request = CallsQueryRequestV1 {
        request: Some(calls_query_request_v1::Request::ListCalls(
            ListCallsRequestV1 {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                after_call_session_id: String::new(),
                limit: 10,
            },
        )),
    }
    .encode_to_vec();
    let list_response = wait_for_telegram_call_history(
        &store,
        &fixture.supervisor,
        &telegram,
        101,
        &list_request,
        2,
    );
    let calls_query_response_v1::Response::CallList(list) = list_response
        .response
        .as_ref()
        .expect("Telegram Calls list response")
    else {
        panic!("Telegram Calls list response is unexpected");
    };
    assert_eq!(list.calls.len(), 2);
    let call = list
        .calls
        .iter()
        .find(|call| call.provider_call_unique_id == Some(5001))
        .expect("managed provider Telegram call");
    assert_eq!(call.provider_call_unique_id, Some(5001));
    assert_eq!(call.provider_user_id, "42");
    assert_eq!(call.state, CallStateV1::Ended as i32);
    assert_eq!(
        call.discard_reason,
        Some(CallDiscardReasonV1::Missed as i32)
    );
    assert_eq!(call.revision, 3);
    assert_eq!(
        telegram_call_media_state(),
        "active",
        "managed TDLib ready/signaling must drive the signed tgcalls artifact into a durable media state"
    );
    assert_eq!(
        telegram_calls_backfill_state(),
        ("succeeded".to_owned(), 1, 1),
        "managed runtime must finish the owner-local V3-to-V4 backfill before readiness"
    );

    let replay = route_telegram_calls_until_ready(
        &store,
        &fixture.supervisor,
        &telegram,
        TelegramCallsContractV1::Realtime,
        102,
        &CallsReplayRequestV1 {
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            after_sequence: 0,
            limit: 10,
        }
        .encode_to_vec(),
    );
    let replay = decode_calls_replay_response(&replay);
    assert_eq!(replay.frames.len(), 4);
    let Some(call_frame_v1::Event::Call(legacy_call)) = replay.frames[0].event.as_ref() else {
        panic!("legacy call frame is unexpected");
    };
    let Some(call_frame_v1::Event::Call(pending_call)) = replay.frames[1].event.as_ref() else {
        panic!("pending call frame is unexpected");
    };
    let Some(call_frame_v1::Event::Call(ready_call)) = replay.frames[2].event.as_ref() else {
        panic!("ready call frame is unexpected");
    };
    let Some(call_frame_v1::Event::Call(ended_call)) = replay.frames[3].event.as_ref() else {
        panic!("ended call frame is unexpected");
    };
    assert_eq!(legacy_call.provider_call_unique_id, Some(4001));
    assert_eq!(legacy_call.revision, 1);
    assert_eq!(pending_call.revision, 1);
    assert_eq!(ready_call.revision, 2);
    assert_eq!(ended_call.revision, 3);
    assert!(replay.frames[0].sequence < replay.frames[1].sequence);
    assert!(replay.frames[1].sequence < replay.frames[2].sequence);
    assert!(replay.frames[2].sequence < replay.frames[3].sequence);
    assert!(!replay.reset_required);

    let initiate_request = CallsCommandRequestV1 {
        request: Some(calls_command_request_v1::Request::InitiateAudioCall(
            InitiateAudioCallRequestV1 {
                operation_id: "managed-call-initiate".to_owned(),
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_user_id: "43".to_owned(),
            },
        )),
    }
    .encode_to_vec();
    let initiate = decode_calls_command_response(&route_telegram_calls_until_ready(
        &store,
        &fixture.supervisor,
        &telegram,
        TelegramCallsContractV1::Command,
        103,
        &initiate_request,
    ));
    let calls_command_response_v1::Response::Accepted(initiate) =
        initiate.response.expect("Telegram Calls initiate response")
    else {
        panic!("Telegram Calls initiate was not durably accepted");
    };
    assert_eq!(initiate.kind, CallOperationKindV1::InitiateAudio as i32);
    assert_eq!(initiate.state, CallOperationStateV1::Accepted as i32);
    let outgoing_call_session_id = initiate.call_session_id.clone();
    let completed_initiate = wait_for_telegram_call_operation(
        &store,
        &fixture.supervisor,
        &telegram,
        104,
        "managed-call-initiate",
        CallOperationStateV1::Completed,
    );
    assert_eq!(completed_initiate.call_session_id, outgoing_call_session_id);

    let conflict = decode_calls_command_response(&route_telegram_calls_until_ready(
        &store,
        &fixture.supervisor,
        &telegram,
        TelegramCallsContractV1::Command,
        105,
        &CallsCommandRequestV1 {
            request: Some(calls_command_request_v1::Request::InitiateAudioCall(
                InitiateAudioCallRequestV1 {
                    operation_id: "managed-call-initiate".to_owned(),
                    account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                    provider_user_id: "44".to_owned(),
                },
            )),
        }
        .encode_to_vec(),
    ));
    let calls_command_response_v1::Response::Failure(conflict) =
        conflict.response.expect("Telegram Calls conflict response")
    else {
        panic!("Telegram Calls idempotency conflict was not rejected");
    };
    assert_eq!(conflict.code, CallsFailureCodeV1::Conflict as i32);
    assert_eq!(conflict.field, "call_state");

    let end_request = CallsCommandRequestV1 {
        request: Some(calls_command_request_v1::Request::EndCall(
            EndCallRequestV1 {
                operation_id: "managed-call-end".to_owned(),
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                call_session_id: outgoing_call_session_id.clone(),
            },
        )),
    }
    .encode_to_vec();
    let end = decode_calls_command_response(&route_telegram_calls_until_ready(
        &store,
        &fixture.supervisor,
        &telegram,
        TelegramCallsContractV1::Command,
        106,
        &end_request,
    ));
    let calls_command_response_v1::Response::Accepted(end) =
        end.response.expect("Telegram Calls end response")
    else {
        panic!("Telegram Calls end was not durably accepted");
    };
    assert_eq!(end.kind, CallOperationKindV1::End as i32);
    assert_eq!(end.state, CallOperationStateV1::Accepted as i32);
    wait_for_telegram_call_operation(
        &store,
        &fixture.supervisor,
        &telegram,
        107,
        "managed-call-end",
        CallOperationStateV1::Completed,
    );

    let stale_runtime = telegram.clone();
    let telegram = fixture.restart_telegram(telegram);
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    let replayed_list = wait_for_telegram_call_history(
        &store,
        &fixture.supervisor,
        &telegram,
        108,
        &list_request,
        2,
    );
    let calls_query_response_v1::Response::CallList(replayed_list) = replayed_list
        .response
        .expect("restarted Telegram Calls list response")
    else {
        panic!("restarted Telegram Calls list response is unexpected");
    };
    let outgoing_call = replayed_list
        .calls
        .iter()
        .find(|call| call.call_session_id == outgoing_call_session_id)
        .expect("restarted outgoing Telegram call");
    assert_eq!(outgoing_call.provider_call_unique_id, Some(6001));
    assert_eq!(outgoing_call.state, CallStateV1::Ended as i32);
    assert!(matches!(
        route_telegram_calls_client(
            &store,
            &fixture.supervisor.relay_port(),
            &stale_runtime,
            TelegramCallsContractV1::Command,
            109,
            &end_request,
        ),
        Err(TelegramClientRouteError::Kernel(error))
            if error == "managed runtime fence is stale"
    ));
}

struct TelegramAutomationConformanceState {
    template: AutomationTemplateV1,
    policy: AutomationPolicyV1,
    preview: AutomationPreviewReceiptV1,
    template_request: Vec<u8>,
    template_response: Vec<u8>,
}

fn assert_telegram_automation_management(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) -> TelegramAutomationConformanceState {
    let operations_before = telegram_operation_count(store, supervisor, telegram);
    let (template, template_request, template_response) =
        assert_automation_template(store, supervisor, telegram);
    let policy = assert_automation_policy(store, supervisor, telegram);
    let preview = assert_automation_preview(store, supervisor, telegram);
    assert_automation_query_projection(store, supervisor, telegram, &template, &policy, &preview);
    assert_stale_automation_template_revision_is_rejected(store, supervisor, telegram);
    assert_eq!(
        telegram_operation_count(store, supervisor, telegram),
        operations_before,
        "Telegram automation management and preview must not create provider operations"
    );
    TelegramAutomationConformanceState {
        template,
        policy,
        preview,
        template_request,
        template_response,
    }
}

fn assert_automation_template(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) -> (AutomationTemplateV1, Vec<u8>, Vec<u8>) {
    let request = AutomationCommandRequestV1 {
        command: Some(automation_command_request_v1::Command::UpsertTemplate(
            UpsertAutomationTemplateCommandV1 {
                mutation_id: "managed-template-mutation-1".to_owned(),
                expected_revision: 0,
                template_id: AUTOMATION_TEMPLATE_ID.to_owned(),
                name: "Managed greeting".to_owned(),
                body_template: "Hello {{name}}".to_owned(),
                required_variables: vec!["name".to_owned()],
            },
        )),
    }
    .encode_to_vec();
    let first = route_telegram_automation_until_ready(
        store,
        supervisor,
        telegram,
        TelegramAutomationContractV1::Command,
        81,
        &request,
    );
    let replay = route_telegram_automation_until_ready(
        store,
        supervisor,
        telegram,
        TelegramAutomationContractV1::Command,
        81,
        &request,
    );
    assert_eq!(
        replay, first,
        "an exact Telegram automation mutation retry must replay exact response bytes"
    );
    let response = decode_automation_command_response(81, &first);
    let Some(automation_command_response_v1::Response::Template(template)) = response.response
    else {
        panic!("Telegram automation template upsert returned the wrong response type");
    };
    assert_eq!(template.template_id, AUTOMATION_TEMPLATE_ID);
    assert_eq!(template.revision, 1);
    assert_eq!(template.required_variables, ["name"]);

    let conflicting_request = AutomationCommandRequestV1 {
        command: Some(automation_command_request_v1::Command::UpsertTemplate(
            UpsertAutomationTemplateCommandV1 {
                mutation_id: "managed-template-mutation-1".to_owned(),
                expected_revision: 0,
                template_id: AUTOMATION_TEMPLATE_ID.to_owned(),
                name: "Conflicting greeting".to_owned(),
                body_template: "Different {{name}}".to_owned(),
                required_variables: vec!["name".to_owned()],
            },
        )),
    }
    .encode_to_vec();
    let conflict = decode_automation_command_response(
        82,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Command,
            82,
            &conflicting_request,
        ),
    );
    assert_automation_failure(
        conflict.response,
        AutomationFailureCodeV1::AutomationFailureCodeIdempotencyConflict,
        "idempotency_key",
    );
    (template, request, first)
}

fn assert_automation_policy(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) -> AutomationPolicyV1 {
    let request = AutomationCommandRequestV1 {
        command: Some(automation_command_request_v1::Command::UpsertPolicy(
            UpsertAutomationPolicyCommandV1 {
                mutation_id: "managed-policy-mutation-1".to_owned(),
                expected_revision: 0,
                policy_id: AUTOMATION_POLICY_ID.to_owned(),
                template_id: AUTOMATION_TEMPLATE_ID.to_owned(),
                name: "Managed preview policy".to_owned(),
                enabled: true,
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_ids: vec![AUTOMATION_CHAT_ID.to_owned()],
                expires_at_unix_seconds: None,
            },
        )),
    }
    .encode_to_vec();
    let response = decode_automation_command_response(
        83,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Command,
            83,
            &request,
        ),
    );
    let Some(automation_command_response_v1::Response::Policy(policy)) = response.response else {
        panic!("Telegram automation policy upsert returned the wrong response type");
    };
    assert_eq!(policy.policy_id, AUTOMATION_POLICY_ID);
    assert_eq!(policy.revision, 1);
    assert_eq!(policy.provider_chat_ids, [AUTOMATION_CHAT_ID]);
    policy
}

fn assert_automation_preview(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) -> AutomationPreviewReceiptV1 {
    let request = AutomationCommandRequestV1 {
        command: Some(automation_command_request_v1::Command::PreviewPolicy(
            PreviewAutomationPolicyCommandV1 {
                preview_id: AUTOMATION_PREVIEW_ID.to_owned(),
                policy_id: AUTOMATION_POLICY_ID.to_owned(),
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: AUTOMATION_CHAT_ID.to_owned(),
                variables: vec![AutomationVariableV1 {
                    name: "name".to_owned(),
                    value: "Ada".to_owned(),
                }],
            },
        )),
    }
    .encode_to_vec();
    let first = route_telegram_automation_until_ready(
        store,
        supervisor,
        telegram,
        TelegramAutomationContractV1::Command,
        84,
        &request,
    );
    let replay = route_telegram_automation_until_ready(
        store,
        supervisor,
        telegram,
        TelegramAutomationContractV1::Command,
        84,
        &request,
    );
    assert_eq!(
        replay, first,
        "an exact Telegram automation preview retry must replay exact response bytes"
    );
    let response = decode_automation_command_response(84, &first);
    let Some(automation_command_response_v1::Response::Preview(preview)) = response.response else {
        panic!("Telegram automation preview returned the wrong response type");
    };
    assert_eq!(preview.preview_id, AUTOMATION_PREVIEW_ID);
    assert_eq!(preview.rendered_text, "Hello Ada");
    assert_eq!(
        preview.rendered_sha256,
        sha2::Sha256::digest(b"Hello Ada").as_slice()
    );
    preview
}

fn assert_automation_query_projection(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    template: &AutomationTemplateV1,
    policy: &AutomationPolicyV1,
    preview: &AutomationPreviewReceiptV1,
) {
    let templates = decode_automation_query_response(
        85,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Query,
            85,
            &AutomationQueryRequestV1 {
                request: Some(automation_query_request_v1::Request::ListTemplates(
                    ListAutomationTemplatesQueryV1 {
                        limit: 10,
                        after_template_id: String::new(),
                    },
                )),
            }
            .encode_to_vec(),
        ),
    );
    let Some(automation_query_response_v1::Response::Templates(templates)) = templates.response
    else {
        panic!("Telegram automation template query returned the wrong response type");
    };
    assert_eq!(templates.items.as_slice(), std::slice::from_ref(template));

    let policies = decode_automation_query_response(
        86,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Query,
            86,
            &AutomationQueryRequestV1 {
                request: Some(automation_query_request_v1::Request::ListPolicies(
                    ListAutomationPoliciesQueryV1 {
                        limit: 10,
                        after_policy_id: String::new(),
                    },
                )),
            }
            .encode_to_vec(),
        ),
    );
    let Some(automation_query_response_v1::Response::Policies(policies)) = policies.response else {
        panic!("Telegram automation policy query returned the wrong response type");
    };
    assert_eq!(policies.items.as_slice(), std::slice::from_ref(policy));

    let receipt = decode_automation_query_response(
        87,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Query,
            87,
            &AutomationQueryRequestV1 {
                request: Some(automation_query_request_v1::Request::GetPreviewReceipt(
                    GetAutomationPreviewReceiptQueryV1 {
                        preview_id: AUTOMATION_PREVIEW_ID.to_owned(),
                    },
                )),
            }
            .encode_to_vec(),
        ),
    );
    let Some(automation_query_response_v1::Response::PreviewReceipt(receipt)) = receipt.response
    else {
        panic!("Telegram automation preview receipt query returned the wrong response type");
    };
    assert_eq!(receipt, *preview);
}

fn assert_stale_automation_template_revision_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) {
    let response = decode_automation_command_response(
        88,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Command,
            88,
            &AutomationCommandRequestV1 {
                command: Some(automation_command_request_v1::Command::UpsertTemplate(
                    UpsertAutomationTemplateCommandV1 {
                        mutation_id: "managed-template-stale-1".to_owned(),
                        expected_revision: 0,
                        template_id: AUTOMATION_TEMPLATE_ID.to_owned(),
                        name: "Stale update".to_owned(),
                        body_template: "Stale {{name}}".to_owned(),
                        required_variables: vec!["name".to_owned()],
                    },
                )),
            }
            .encode_to_vec(),
        ),
    );
    assert_automation_failure(
        response.response,
        AutomationFailureCodeV1::AutomationFailureCodeRevisionConflict,
        "expected_revision",
    );
}

fn assert_automation_failure(
    response: Option<automation_command_response_v1::Response>,
    expected_code: AutomationFailureCodeV1,
    expected_field: &str,
) {
    let Some(automation_command_response_v1::Response::Failure(failure)) = response else {
        panic!("Telegram automation command did not return a typed failure");
    };
    assert_eq!(failure.code, expected_code as i32);
    assert_eq!(failure.field, expected_field);
}

fn telegram_operation_count(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) -> usize {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Query,
            89,
            &TelegramClientRequest::Query(TelegramProviderQuery::Operations {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                limit: 100,
            }),
        ) {
            Ok(TelegramClientResponse::Query(TelegramProviderQueryResponse::Operations(
                operations,
            ))) => return operations.len(),
            Ok(_) => panic!("Telegram operation query returned the wrong response type"),
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram operation query remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("Telegram operation query failed: {error:?}"),
        }
    }
}

fn wait_for_telegram_call_history(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    request_id: u64,
    request_payload: &[u8],
    expected_minimum: usize,
) -> CallsQueryResponseV1 {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let bytes = route_telegram_calls_until_ready(
            store,
            supervisor,
            telegram,
            TelegramCallsContractV1::Query,
            request_id,
            request_payload,
        );
        let response = decode_calls_query_response(&bytes);
        if matches!(
            response.response,
            Some(calls_query_response_v1::Response::CallList(ref list))
                if list.calls.len() >= expected_minimum
        ) {
            return response;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Telegram call history was not projected"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_telegram_call_operation(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    request_id: u64,
    operation_id: &str,
    expected_state: CallOperationStateV1,
) -> makosh_telegram_calls_api::wire::CallOperationV1 {
    let request = CallsQueryRequestV1 {
        request: Some(calls_query_request_v1::Request::GetCallOperation(
            GetCallOperationRequestV1 {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                operation_id: operation_id.to_owned(),
            },
        )),
    }
    .encode_to_vec();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let response = decode_calls_query_response(&route_telegram_calls_until_ready(
            store,
            supervisor,
            telegram,
            TelegramCallsContractV1::Query,
            request_id,
            &request,
        ));
        if let Some(calls_query_response_v1::Response::Operation(operation)) = response.response
            && operation.state == expected_state as i32
        {
            return operation;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Telegram call operation did not reach the expected state"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn route_telegram_calls_until_ready(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    contract: TelegramCallsContractV1,
    request_id: u64,
    request_payload: &[u8],
) -> Vec<u8> {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match route_telegram_calls_client(
            store,
            &relay,
            telegram,
            contract,
            request_id,
            request_payload,
        ) {
            Ok(response) => return response,
            Err(error) if error.is_retryable() => {
                if std::time::Instant::now() >= deadline {
                    let runtime_failure = supervisor
                        .last_failure(&telegram.registration_id)
                        .expect("read Telegram runtime failure");
                    panic!(
                        "Telegram Calls route remained busy: {error:?}; runtime failure: {runtime_failure:?}"
                    );
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => {
                panic!("Telegram Calls route {request_id} failed: {error:?}")
            }
        }
    }
}

fn route_telegram_calls_client(
    store: &SqliteControlStore,
    relay: &crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort,
    telegram: &StartedTelegramRuntime,
    contract: TelegramCallsContractV1,
    request_id: u64,
    request_payload: &[u8],
) -> Result<Vec<u8>, TelegramClientRouteError> {
    let request = ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: TELEGRAM_CALLS_MODULE_ID.to_owned(),
        owner_id: TELEGRAM_CALLS_OWNER_ID.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: TELEGRAM_CALLS_OWNER_ID.to_owned(),
            name: contract.contract_name().to_owned(),
            major: TELEGRAM_CALLS_CONTRACT_MAJOR,
            revision: TELEGRAM_CALLS_CONTRACT_REVISION,
            schema_sha256: sha2::Sha256::digest(TELEGRAM_CALLS_DESCRIPTOR_SET_V1).to_vec(),
        }),
        request_id,
        request_payload: request_payload.to_vec(),
        logical_owner_id: String::new(),
        authenticated_device_id: String::new(),
        authenticated_client_session_id: String::new(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &telegram.registration_id,
        &telegram.runtime_instance_id,
        telegram.runtime_generation,
        telegram.grant_epoch,
        contract.capability_id(),
        &request,
    );
    let bytes =
        crate::modules::capability::router::route_managed_client_request(store, relay, &route)
            .map_err(TelegramClientRouteError::Kernel)?;
    let response = ModuleClientResponseV1::decode(bytes.as_slice()).map_err(|error| {
        TelegramClientRouteError::Client(TelegramClientPortError::Codec(error.to_string()))
    })?;
    if !response.error_code.is_empty() {
        return Err(TelegramClientRouteError::Client(
            TelegramClientPortError::Protocol(response.error_code),
        ));
    }
    Ok(bytes)
}

fn decode_calls_query_response(bytes: &[u8]) -> CallsQueryResponseV1 {
    let response = ModuleClientResponseV1::decode(bytes).expect("Telegram Calls module response");
    CallsQueryResponseV1::decode(response.response_payload.as_slice())
        .expect("Telegram Calls query response")
}

fn decode_calls_replay_response(bytes: &[u8]) -> CallsReplayResponseV1 {
    let response = ModuleClientResponseV1::decode(bytes).expect("Telegram Calls module response");
    CallsReplayResponseV1::decode(response.response_payload.as_slice())
        .expect("Telegram Calls replay response")
}

pub(super) fn decode_calls_command_response(bytes: &[u8]) -> CallsCommandResponseV1 {
    let response = ModuleClientResponseV1::decode(bytes).expect("Telegram Calls module response");
    CallsCommandResponseV1::decode(response.response_payload.as_slice())
        .expect("Telegram Calls command response")
}

fn route_telegram_automation_until_ready(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    contract: TelegramAutomationContractV1,
    request_id: u64,
    request_payload: &[u8],
) -> Vec<u8> {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match route_telegram_automation_client(
            store,
            &relay,
            telegram,
            contract,
            request_id,
            request_payload,
        ) {
            Ok(response) => return response,
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram automation route remained busy: {error:?}; active={:?}; failure={:?}",
                    supervisor.is_active(&telegram.registration_id),
                    supervisor.last_failure(&telegram.registration_id),
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("Telegram automation route failed: {error:?}"),
        }
    }
}

fn route_telegram_automation_client(
    store: &SqliteControlStore,
    relay: &crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort,
    telegram: &StartedTelegramRuntime,
    contract: TelegramAutomationContractV1,
    request_id: u64,
    request_payload: &[u8],
) -> Result<Vec<u8>, TelegramClientRouteError> {
    let request = ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: TELEGRAM_AUTOMATION_MODULE_ID.to_owned(),
        owner_id: TELEGRAM_AUTOMATION_OWNER_ID.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: TELEGRAM_AUTOMATION_OWNER_ID.to_owned(),
            name: contract.contract_name().to_owned(),
            major: TELEGRAM_AUTOMATION_CONTRACT_MAJOR,
            revision: TELEGRAM_AUTOMATION_CONTRACT_REVISION,
            schema_sha256: sha2::Sha256::digest(TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1).to_vec(),
        }),
        request_id,
        request_payload: request_payload.to_vec(),
        logical_owner_id: String::new(),
        authenticated_device_id: String::new(),
        authenticated_client_session_id: String::new(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &telegram.registration_id,
        &telegram.runtime_instance_id,
        telegram.runtime_generation,
        telegram.grant_epoch,
        contract.capability_id(),
        &request,
    );
    let bytes =
        crate::modules::capability::router::route_managed_client_request(store, relay, &route)
            .map_err(TelegramClientRouteError::Kernel)?;
    let response = ModuleClientResponseV1::decode(bytes.as_slice()).map_err(|error| {
        TelegramClientRouteError::Client(TelegramClientPortError::Codec(error.to_string()))
    })?;
    if !response.error_code.is_empty() {
        return Err(TelegramClientRouteError::Client(
            TelegramClientPortError::Protocol(response.error_code),
        ));
    }
    Ok(bytes)
}

fn decode_automation_command_response(
    request_id: u64,
    bytes: &[u8],
) -> AutomationCommandResponseV1 {
    let payload = decode_automation_response_payload(request_id, bytes);
    AutomationCommandResponseV1::decode(payload.as_slice())
        .expect("decode Telegram automation command response")
}

fn decode_automation_query_response(request_id: u64, bytes: &[u8]) -> AutomationQueryResponseV1 {
    let payload = decode_automation_response_payload(request_id, bytes);
    AutomationQueryResponseV1::decode(payload.as_slice())
        .expect("decode Telegram automation query response")
}

fn decode_automation_response_payload(request_id: u64, bytes: &[u8]) -> Vec<u8> {
    let response =
        ModuleClientResponseV1::decode(bytes).expect("decode Telegram automation module response");
    assert_eq!(response.protocol_major, MODULE_CLIENT_PROTOCOL_MAJOR);
    assert_eq!(response.request_id, request_id);
    assert!(response.error_code.is_empty());
    assert!(!response.response_payload.is_empty());
    response.response_payload
}

fn route_telegram_client(
    store: &SqliteControlStore,
    relay: &crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort,
    telegram: &StartedTelegramRuntime,
    contract: TelegramClientContractV1,
    request_id: u64,
    request: &TelegramClientRequest,
) -> Result<TelegramClientResponse, TelegramClientRouteError> {
    let request =
        encode_module_request(request_id, request).map_err(TelegramClientRouteError::Client)?;
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &telegram.registration_id,
        &telegram.runtime_instance_id,
        telegram.runtime_generation,
        telegram.grant_epoch,
        contract.capability_id(),
        &request,
    );
    let bytes =
        crate::modules::capability::router::route_managed_client_request(store, relay, &route)
            .map_err(TelegramClientRouteError::Kernel)?;
    let (response_request_id, response) =
        decode_module_response(contract, &bytes).map_err(TelegramClientRouteError::Client)?;
    if response_request_id != request_id {
        return Err(TelegramClientRouteError::Kernel(format!(
            "Telegram response request ID mismatch: expected {request_id}, got {response_request_id}"
        )));
    }
    Ok(response)
}

fn assert_telegram_lifecycle_query(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while relay.is_ready(&telegram.registration_id) != Ok(true) {
        assert!(
            std::time::Instant::now() < deadline,
            "managed Telegram runtime did not become ready: {:?}",
            supervisor.last_failure(&telegram.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    loop {
        let last_error = match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Lifecycle,
            71,
            &TelegramClientRequest::ListAccounts,
        ) {
            Ok(TelegramClientResponse::Accounts(accounts)) => {
                assert!(
                    accounts
                        .iter()
                        .any(|account| account.account_id == TELEGRAM_ACCOUNT_ID)
                );
                return;
            }
            Ok(_) => "Telegram returned the wrong lifecycle response type".to_owned(),
            Err(error) => format!("{error:?}"),
        };
        assert!(
            std::time::Instant::now() < deadline,
            "managed Telegram lifecycle query is unavailable: {last_error}; child failure: {:?}",
            supervisor.last_failure(&telegram.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_telegram_account_started(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let request = TelegramClientRequest::GetAccount {
        account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
    };
    loop {
        match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Lifecycle,
            72,
            &request,
        ) {
            Ok(TelegramClientResponse::Account(account))
                if account.runtime_state == TelegramRuntimeState::Running =>
            {
                return;
            }
            Ok(TelegramClientResponse::Account(_)) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram managed account did not reach Running: {:?}",
                    supervisor.last_failure(&telegram.registration_id),
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok(_) => panic!("Telegram lifecycle query returned the wrong response type"),
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram managed account activation remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("Telegram managed account activation failed: {error:?}"),
        }
    }
}

fn assert_telegram_account_runtime_epoch(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    expected_runtime_epoch: u64,
) {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let request = TelegramClientRequest::GetAccount {
        account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
    };
    loop {
        match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Lifecycle,
            96,
            &request,
        ) {
            Ok(TelegramClientResponse::Account(account))
                if account.runtime_state == TelegramRuntimeState::Running
                    && account.runtime_epoch == expected_runtime_epoch =>
            {
                return;
            }
            Ok(TelegramClientResponse::Account(_)) => {}
            Ok(_) => panic!("Telegram lifecycle query returned the wrong response type"),
            Err(error) if error.is_retryable() => {}
            Err(error) => panic!("Telegram managed account query failed: {error:?}"),
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Telegram account did not converge to runtime epoch {expected_runtime_epoch}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_telegram_runtime_reconfiguration(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    reconfiguration_id: &str,
) -> makosh_telegram_api::TelegramRuntimeReconfiguration {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Reconfiguration,
            93,
            &TelegramClientRequest::Reconfiguration(
                TelegramRuntimeReconfigurationRequest::Status {
                    reconfiguration_id: reconfiguration_id.to_owned(),
                },
            ),
        ) {
            Ok(TelegramClientResponse::Reconfiguration(reconfiguration))
                if reconfiguration.state == TelegramRuntimeReconfigurationState::Completed =>
            {
                return reconfiguration;
            }
            Ok(TelegramClientResponse::Reconfiguration(reconfiguration)) => {
                assert!(
                    !reconfiguration.state.is_terminal(),
                    "Telegram runtime reconfiguration failed: {:?}",
                    reconfiguration.sanitized_reason_code
                );
            }
            Ok(_) => panic!("Telegram reconfiguration status returned the wrong response type"),
            Err(error) if error.is_retryable() => {}
            Err(error) => panic!("Telegram reconfiguration status failed: {error:?}"),
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Telegram runtime reconfiguration did not complete"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn route_telegram_reconfiguration_until_ready(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    request_id: u64,
    request: &TelegramClientRequest,
) -> Result<TelegramClientResponse, TelegramClientRouteError> {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Reconfiguration,
            request_id,
            request,
        ) {
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram reconfiguration route remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            result => return result,
        }
    }
}

fn assert_telegram_command_accepted(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    operation_id: &str,
    text: &str,
) {
    let command = TelegramProviderCommand::SendText(TelegramSendMessage {
        operation_id: operation_id.to_owned(),
        account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
        provider_chat_id: "9001".to_owned(),
        text: text.to_owned(),
    });
    assert_telegram_provider_command_accepted(
        store,
        supervisor,
        telegram,
        73,
        operation_id,
        command,
    );
}

fn assert_telegram_provider_command_accepted(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    request_id: u64,
    operation_id: &str,
    command: TelegramProviderCommand,
) {
    let relay = supervisor.relay_port();
    let command = TelegramClientRequest::Command(command);
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let response = loop {
        match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Command,
            request_id,
            &command,
        ) {
            Ok(response) => break response,
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram command route remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("Telegram command route failed: {error:?}"),
        }
    };
    let TelegramClientResponse::Operation(operation) = response else {
        panic!("Telegram command returned the wrong response type");
    };
    assert_eq!(operation.operation_id, operation_id);
    assert_eq!(
        operation.state,
        TelegramOperationState::Accepted,
        "accepted receipt is distinct from provider completion"
    );
}

fn assert_telegram_operation_completed(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    operation_id: &str,
) -> makosh_telegram_api::TelegramOperation {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let response = match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Query,
            74,
            &TelegramClientRequest::Query(TelegramProviderQuery::Operations {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                limit: 16,
            }),
        ) {
            Ok(response) => response,
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram operation query remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
                continue;
            }
            Err(error) => panic!("Telegram operation query failed: {error:?}"),
        };
        let TelegramClientResponse::Query(TelegramProviderQueryResponse::Operations(operations)) =
            response
        else {
            panic!("Telegram operation query returned the wrong response type");
        };
        if let Some(operation) = operations
            .iter()
            .find(|operation| operation.operation_id == operation_id)
        {
            match operation.state {
                TelegramOperationState::Completed => return operation.clone(),
                TelegramOperationState::Failed | TelegramOperationState::DeadLetter => {
                    panic!(
                        "Telegram provider command reached a failure terminal state: operation={} retries={}",
                        operation.operation_id, operation.retry_count
                    )
                }
                _ => {}
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Telegram provider command did not reach a terminal result"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_telegram_folder_ids(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    expected: &[i64],
) {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let response = match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Query,
            88,
            &TelegramClientRequest::Query(TelegramProviderQuery::ChatPositions {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: "9002".to_owned(),
            }),
        ) {
            Ok(response) => response,
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram folder query remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
                continue;
            }
            Err(error) => panic!("Telegram folder query failed: {error:?}"),
        };
        let TelegramClientResponse::Query(TelegramProviderQueryResponse::ChatPositions(positions)) =
            response
        else {
            panic!("Telegram folder query returned the wrong response type");
        };
        let mut folder_ids = positions
            .iter()
            .filter(|position| position.list_kind == "folder" && position.order > 0)
            .filter_map(|position| position.provider_folder_id)
            .collect::<Vec<_>>();
        folder_ids.sort_unstable();
        folder_ids.dedup();
        if folder_ids == expected {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Telegram folder projection did not converge"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}
