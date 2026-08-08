//! Private Unix IPC transport for proof-backed external runtime capability sessions.

use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use makosh_gateway_protocol::v1::{
    AcknowledgeRuntimeSettingsLifecycleRequestV1, AcknowledgeRuntimeSettingsLifecycleResponseV1,
    AuthorizeExternalRuntimeCapabilityRequestV1, AuthorizeExternalRuntimeCapabilityResponseV1,
    BeginExternalRuntimeSessionRequestV1, BeginExternalRuntimeSessionResponseV1,
    CompleteExternalRuntimeSessionRequestV1, CompleteExternalRuntimeSessionResponseV1,
    ExternalRuntimeSessionRequestV1, ExternalRuntimeSessionResponseV1,
    GetExternalRuntimeStorageBindingRequestV1, GetExternalRuntimeStorageBindingResponseV1,
    RouteVaultCiphertextRequestV1, RouteVaultCiphertextResponseV1,
    SubmitRuntimeSettingsSchemaRequestV1, SubmitRuntimeSettingsSchemaResponseV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    ManagedVaultRuntimeControlRequestV1, ManagedVaultRuntimeControlResponseV1,
    managed_vault_runtime_control_request_v1::Operation as VaultOperation,
    managed_vault_runtime_control_response_v1::Result as VaultResult,
};
use prost::Message;

use crate::infrastructure::filesystem::remove_stale_owner_unix_socket;
use crate::modules::settings::application::{
    self as settings_apply_lifecycle, parse_acknowledgement,
};
use crate::modules::settings::schema as settings_schema;
use crate::platform::vault::binding::VAULT_PROCESS_ID;
use crate::platform::vault::launch as vault_launch;
use crate::runtime::external::sessions::ExternalRuntimeSessions;
use crate::runtime::external::storage;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const MAX_FRAME_BYTES: usize = 64 * 1024;
const IPC_TIMEOUT: Duration = Duration::from_secs(5);
const SHUTDOWN_POLL: Duration = Duration::from_millis(25);

pub fn serve(
    store: Arc<SqliteControlStore>,
    data_dir: &Path,
    runtime_dir: &Path,
    shutdown_requested: Arc<AtomicBool>,
    managed_runtime_supervisor: ManagedRuntimeSupervisor,
) -> Result<(), String> {
    let socket_path = runtime_dir.join("runtime.sock");
    remove_stale_owner_unix_socket(&socket_path, "external runtime session socket")?;
    let listener = UnixListener::bind(&socket_path).map_err(|error| error.to_string())?;
    let cleanup = SocketCleanup(socket_path.clone());
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;
    println!("external_runtime_session_socket={}", socket_path.display());
    let mut sessions = ExternalRuntimeSessions::default();
    let result = loop {
        if shutdown_requested.load(Ordering::Acquire) {
            break Ok(());
        }
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(SHUTDOWN_POLL);
                continue;
            }
            Err(error) => break Err(error.to_string()),
        };
        let _ = handle_connection(
            &store,
            data_dir,
            &managed_runtime_supervisor,
            &mut sessions,
            &mut stream,
        );
    };
    drop(cleanup);
    result
}

struct SocketCleanup(PathBuf);

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn handle_connection(
    store: &SqliteControlStore,
    data_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut ExternalRuntimeSessions,
    stream: &mut UnixStream,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(IPC_TIMEOUT)))
        .map_err(|error| error.to_string())?;
    let response = match read_frame(stream).and_then(|bytes| {
        ExternalRuntimeSessionRequestV1::decode(bytes.as_slice())
            .map_err(|_| "invalid external runtime session request".to_owned())
    }) {
        Ok(request) => handle(store, data_dir, supervisor, sessions, request),
        Err(_) => error_response("invalid_request"),
    };
    write_frame(stream, &response.encode_to_vec())
}

fn handle(
    store: &SqliteControlStore,
    data_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut ExternalRuntimeSessions,
    request: ExternalRuntimeSessionRequestV1,
) -> ExternalRuntimeSessionResponseV1 {
    use makosh_gateway_protocol::v1::external_runtime_session_request_v1::Operation;
    let result = match request.operation {
        Some(Operation::Begin(request)) => begin_session(store, sessions, request),
        Some(Operation::Complete(request)) => complete_session(store, sessions, request),
        Some(Operation::AuthorizeCapability(request)) => {
            authorize_capability(store, sessions, request)
        }
        Some(Operation::SubmitSettingsSchema(request)) => {
            submit_settings_schema(store, sessions, request)
        }
        Some(Operation::AcknowledgeSettingsLifecycle(request)) => {
            acknowledge_settings(store, sessions, request)
        }
        Some(Operation::RouteVaultCiphertext(request)) => {
            route_vault_ciphertext(store, data_dir, supervisor, sessions, request)
        }
        Some(Operation::GetStorageBinding(request)) => {
            get_storage_binding(store, supervisor, sessions, request)
        }
        None => Err("invalid_request".to_owned()),
    };
    match result {
        Ok(result) => ExternalRuntimeSessionResponseV1 {
            result: Some(result),
            error_code: String::new(),
        },
        Err(error) => error_response(error_code(&error)),
    }
}

type SessionResult = makosh_gateway_protocol::v1::external_runtime_session_response_v1::Result;

fn begin_session(
    store: &SqliteControlStore,
    sessions: &mut ExternalRuntimeSessions,
    request: BeginExternalRuntimeSessionRequestV1,
) -> Result<SessionResult, String> {
    let distribution_sha256 = request
        .distribution_artifact_sha256
        .as_slice()
        .try_into()
        .map_err(|_| "external runtime distribution digest is invalid".to_owned())?;
    sessions
        .begin(
            store,
            &request.registration_id,
            &request.runtime_id,
            request.runtime_generation,
            distribution_sha256,
        )
        .map(|challenge| {
            SessionResult::Begin(BeginExternalRuntimeSessionResponseV1 {
                challenge_id: challenge.challenge_id().to_owned(),
                challenge_bytes: challenge.bytes().to_vec(),
                kernel_instance_id: challenge.kernel_instance_id().to_owned(),
                grant_epoch: challenge.grant_epoch(),
                expires_at_unix_millis: challenge.expires_at_unix_millis(),
            })
        })
}

fn complete_session(
    store: &SqliteControlStore,
    sessions: &mut ExternalRuntimeSessions,
    request: CompleteExternalRuntimeSessionRequestV1,
) -> Result<SessionResult, String> {
    sessions
        .complete(store, &request.challenge_id, &request.signature_raw)
        .map(|session| {
            SessionResult::Complete(CompleteExternalRuntimeSessionResponseV1 {
                session_id: session.session_id().to_owned(),
                grant_epoch: session.grant_epoch(),
                expires_at_unix_millis: session.expires_at_unix_millis(),
            })
        })
}

fn authorize_capability(
    store: &SqliteControlStore,
    sessions: &mut ExternalRuntimeSessions,
    request: AuthorizeExternalRuntimeCapabilityRequestV1,
) -> Result<SessionResult, String> {
    sessions
        .authorize(store, &request.session_id, &request.capability_id)
        .map(|grant_epoch| {
            SessionResult::AuthorizeCapability(AuthorizeExternalRuntimeCapabilityResponseV1 {
                grant_epoch,
            })
        })
}

fn submit_settings_schema(
    store: &SqliteControlStore,
    sessions: &mut ExternalRuntimeSessions,
    request: SubmitRuntimeSettingsSchemaRequestV1,
) -> Result<SessionResult, String> {
    (|| {
        let registration_id = sessions.authorize_registration_action(store, &request.session_id)?;
        settings_schema::admit(
            store,
            &registration_id,
            &request.descriptor_bytes,
            &request.schema_bytes,
        )
    })()
    .map(|binding| {
        SessionResult::SubmitSettingsSchema(SubmitRuntimeSettingsSchemaResponseV1 {
            registration_id: binding.registration_id().to_owned(),
            schema_major: binding.schema_major(),
            schema_revision: binding.schema_revision(),
        })
    })
}

fn acknowledge_settings(
    store: &SqliteControlStore,
    sessions: &mut ExternalRuntimeSessions,
    request: AcknowledgeRuntimeSettingsLifecycleRequestV1,
) -> Result<SessionResult, String> {
    (|| {
        let registration_id = sessions.authorize_registration_action(store, &request.session_id)?;
        let reason_code = (!request.reason_code.is_empty()).then_some(request.reason_code.as_str());
        let acknowledgement = parse_acknowledgement(&request.acknowledgement, reason_code)?;
        settings_apply_lifecycle::acknowledge(
            store,
            &registration_id,
            request.revision,
            acknowledgement,
        )?;
        Ok((registration_id, request.revision))
    })()
    .map(|(registration_id, revision)| {
        SessionResult::AcknowledgeSettingsLifecycle(AcknowledgeRuntimeSettingsLifecycleResponseV1 {
            registration_id,
            revision,
        })
    })
}

fn route_vault_ciphertext(
    store: &SqliteControlStore,
    data_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut ExternalRuntimeSessions,
    request: RouteVaultCiphertextRequestV1,
) -> Result<SessionResult, String> {
    let route = request
        .route
        .ok_or_else(|| "Vault ciphertext route is invalid".to_owned())?;
    let launch = vault_launch::current_launch(store)?;
    if !supervisor.is_active(VAULT_PROCESS_ID)? {
        return Err("Vault runtime is unavailable".to_owned());
    }
    let validated = sessions.authorize_vault_route(
        store,
        &request.session_id,
        launch.runtime_generation(),
        route,
    )?;
    let mut route = validated.into_route();
    crate::platform::vault::ciphertext_route::sign_for_kernel(
        data_dir,
        store.snapshot().instance_id(),
        &mut route,
    )?;
    let response = supervisor.relay(
        VAULT_PROCESS_ID,
        ManagedVaultRuntimeControlRequestV1 {
            operation: Some(VaultOperation::CiphertextRoute(route.clone())),
        }
        .encode_to_vec(),
    )?;
    let response = ManagedVaultRuntimeControlResponseV1::decode(response.as_slice())
        .map_err(|_| "Vault ciphertext response is invalid".to_owned())?;
    if !response.error_code.is_empty() {
        return Err("Vault ciphertext response is unavailable".to_owned());
    }
    let response = match response.result {
        Some(VaultResult::CiphertextResponse(response)) => response,
        _ => return Err("Vault ciphertext response is unavailable".to_owned()),
    };
    let response = crate::platform::vault::ciphertext_route::validate_response(&route, response)?;
    Ok(SessionResult::RouteVaultCiphertext(
        RouteVaultCiphertextResponseV1 {
            response: Some(response),
        },
    ))
}

fn get_storage_binding(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut ExternalRuntimeSessions,
    request: GetExternalRuntimeStorageBindingRequestV1,
) -> Result<SessionResult, String> {
    storage::current_binding(
        store,
        supervisor,
        sessions,
        &request.session_id,
        &request.capability_id,
    )
    .map(|binding| {
        SessionResult::GetStorageBinding(GetExternalRuntimeStorageBindingResponseV1 {
            storage_binding_v1: binding.storage_binding_v1().to_vec(),
            pgbouncer_host: binding.pgbouncer_host().to_owned(),
            pgbouncer_port: binding.pgbouncer_port(),
            vault_instance_id: binding.vault_instance_id().to_owned(),
            vault_runtime_generation: binding.vault_runtime_generation(),
            vault_hpke_public_key_x25519: binding.vault_hpke_public_key_x25519().to_vec(),
        })
    })
}

fn error_response(error_code: &str) -> ExternalRuntimeSessionResponseV1 {
    ExternalRuntimeSessionResponseV1 {
        result: None,
        error_code: error_code.to_owned(),
    }
}

fn error_code(error: &str) -> &'static str {
    match error {
        "invalid_request"
        | "external runtime distribution digest is invalid"
        | "Vault ciphertext route is invalid" => "invalid_request",
        "runtime session rate limited" => "runtime_session_rate_limited",
        "runtime challenge is unavailable" | "runtime session is unavailable" => {
            "runtime_session_unavailable"
        }
        "external runtime signature is invalid" | "external runtime proof verification failed" => {
            "runtime_proof_invalid"
        }
        "external runtime challenge is stale"
        | "runtime session is stale or unauthorized"
        | "Vault ciphertext route is stale or unauthorized"
        | "Storage credential route is unauthorized"
        | "Storage credential route is stale or unauthorized" => "runtime_session_stale",
        "Vault runtime is unavailable" => "runtime_session_unavailable",
        _ => "runtime_session_denied",
    }
}

fn read_frame(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(stream)?)
        .map_err(|_| "external runtime session frame is too large".to_owned())?;
    if length > MAX_FRAME_BYTES {
        return Err("external runtime session frame is too large".to_owned());
    }
    let mut bytes = vec![0_u8; length];
    stream
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn read_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("external runtime session frame length is invalid".to_owned())
}

fn write_frame(stream: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let mut length = u32::try_from(bytes.len())
        .map_err(|_| "external runtime session response is too large".to_owned())?;
    while length >= 0x80 {
        stream
            .write_all(&[(length as u8 & 0x7f) | 0x80])
            .map_err(|error| error.to_string())?;
        length >>= 7;
    }
    stream
        .write_all(&[length as u8])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}
