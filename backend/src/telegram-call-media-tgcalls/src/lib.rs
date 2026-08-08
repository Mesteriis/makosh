use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::Path;
use std::ptr::{self, NonNull};
use std::sync::Arc;

use libloading::Library;
use makosh_telegram_call_media_contract::{
    CALL_ENCRYPTION_KEY_BYTES, MAX_LIBRARY_VERSION_BYTES, MAX_LIBRARY_VERSIONS,
    MAX_SIGNALING_DATA_BYTES, TD_CALL_MAX_LAYER_V1, TelegramCallDiscardContextV1,
    TelegramCallMediaContractError, TelegramCallMediaEventV1, TelegramCallMediaFinalV1,
    TelegramCallMediaStateV1, TelegramCallProtocolV1, TelegramCallReadyPlanV1,
    TelegramCallSecretBytesV1, TelegramCallServerKindV1, TelegramCallSignalingMediaPort,
};
use zeroize::{Zeroize, Zeroizing};

pub const PACKAGE: &str = "makosh-telegram-call-media-tgcalls";
pub const TGCALLS_BRIDGE_ABI_VERSION_V1: u32 = 1;

const RESULT_OK: i32 = 0;
const RESULT_EVENT: i32 = 1;
const RESULT_INVALID_ARGUMENT: i32 = -1;
const RESULT_UNSUPPORTED_VERSION: i32 = -2;
const RESULT_INVALID_STATE: i32 = -3;
const RESULT_QUEUE_OVERFLOW: i32 = -4;
const RESULT_NATIVE_FAILURE: i32 = -5;
const RESULT_BUFFER_TOO_SMALL: i32 = -6;

const SERVER_TELEGRAM_REFLECTOR: u32 = 1;
const SERVER_WEBRTC: u32 = 2;
const STATE_CONNECTING: u32 = 1;
const STATE_ESTABLISHED: u32 = 2;
const STATE_RECONNECTING: u32 = 3;
const STATE_FAILED: u32 = 4;
const EVENT_STATE: u32 = 1;
const EVENT_SIGNALING: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeServerV1 {
    abi_version: u32,
    kind: u32,
    reflector_id: u8,
    is_tcp: u8,
    supports_stun: u8,
    supports_turn: u8,
    port: u16,
    host: *const c_char,
    username: *const c_char,
    password: *const c_char,
    peer_tag: [u8; 16],
}

#[repr(C)]
struct NativeSessionConfigV1 {
    abi_version: u32,
    library_version: *const c_char,
    initialization_timeout_seconds: f64,
    receive_timeout_seconds: f64,
    enable_p2p: u8,
    allow_tcp: u8,
    is_outgoing: u8,
    call_config: *const c_char,
    custom_parameters: *const c_char,
    encryption_key: *const u8,
    encryption_key_length: usize,
    servers: *const NativeServerV1,
    server_count: usize,
    input_device_id: *const c_char,
    output_device_id: *const c_char,
}

#[repr(C)]
struct NativeEventV1 {
    abi_version: u32,
    kind: u32,
    state: u32,
    payload_length: usize,
}

#[repr(C)]
struct NativeSnapshotV1 {
    abi_version: u32,
    state: u32,
    duration_seconds: u32,
    connection_id: i64,
    failed: u8,
}

type AbiVersionFn = unsafe extern "C" fn() -> u32;
type VersionCountFn = unsafe extern "C" fn() -> usize;
type VersionAtFn = unsafe extern "C" fn(usize, *mut c_char, usize) -> i32;
type MaxLayerFn = unsafe extern "C" fn() -> i32;
type SessionCreateFn = unsafe extern "C" fn(*const NativeSessionConfigV1, *mut *mut c_void) -> i32;
type SessionReceiveSignalingFn = unsafe extern "C" fn(*mut c_void, *const u8, usize) -> i32;
type SessionSetMutedFn = unsafe extern "C" fn(*mut c_void, u8) -> i32;
type SessionPollEventFn =
    unsafe extern "C" fn(*mut c_void, *mut NativeEventV1, *mut u8, usize) -> i32;
type SessionSnapshotFn = unsafe extern "C" fn(*mut c_void, *mut NativeSnapshotV1) -> i32;
type SessionStopFn = unsafe extern "C" fn(*mut c_void, *mut NativeSnapshotV1) -> i32;
type SessionDestroyFn = unsafe extern "C" fn(*mut c_void) -> i32;

#[derive(Clone, Copy)]
struct NativeSessionApi {
    receive_signaling: SessionReceiveSignalingFn,
    set_muted: SessionSetMutedFn,
    poll_event: SessionPollEventFn,
    snapshot: SessionSnapshotFn,
    stop: SessionStopFn,
    destroy: SessionDestroyFn,
}

struct NativeApi {
    abi_version: AbiVersionFn,
    version_count: VersionCountFn,
    version_at: VersionAtFn,
    max_layer: MaxLayerFn,
    session_create: SessionCreateFn,
    session_receive_signaling: SessionReceiveSignalingFn,
    session_set_muted: SessionSetMutedFn,
    session_poll_event: SessionPollEventFn,
    session_snapshot: SessionSnapshotFn,
    session_stop: SessionStopFn,
    session_destroy: SessionDestroyFn,
    _library: Library,
}

impl NativeApi {
    fn load_exact(path: &Path) -> Result<Self, TelegramCallMediaContractError> {
        if !path.is_absolute() {
            return Err(TelegramCallMediaContractError::Unavailable);
        }
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|_| TelegramCallMediaContractError::Unavailable)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() == 0 {
            return Err(TelegramCallMediaContractError::Unavailable);
        }
        let library = unsafe { Library::new(path) }
            .map_err(|_| TelegramCallMediaContractError::Unavailable)?;
        let api = unsafe {
            Self {
                abi_version: load_symbol(&library, b"makosh_tgcalls_abi_version_v1\0")?,
                version_count: load_symbol(&library, b"makosh_tgcalls_version_count_v1\0")?,
                version_at: load_symbol(&library, b"makosh_tgcalls_version_at_v1\0")?,
                max_layer: load_symbol(&library, b"makosh_tgcalls_max_layer_v1\0")?,
                session_create: load_symbol(&library, b"makosh_tgcalls_session_create_v1\0")?,
                session_receive_signaling: load_symbol(
                    &library,
                    b"makosh_tgcalls_session_receive_signaling_v1\0",
                )?,
                session_set_muted: load_symbol(&library, b"makosh_tgcalls_session_set_muted_v1\0")?,
                session_poll_event: load_symbol(
                    &library,
                    b"makosh_tgcalls_session_poll_event_v1\0",
                )?,
                session_snapshot: load_symbol(&library, b"makosh_tgcalls_session_snapshot_v1\0")?,
                session_stop: load_symbol(&library, b"makosh_tgcalls_session_stop_v1\0")?,
                session_destroy: load_symbol(&library, b"makosh_tgcalls_session_destroy_v1\0")?,
                _library: library,
            }
        };
        let abi_version = unsafe { (api.abi_version)() };
        if abi_version != TGCALLS_BRIDGE_ABI_VERSION_V1 {
            return Err(TelegramCallMediaContractError::Unavailable);
        }
        Ok(api)
    }

    fn protocol(&self) -> Result<TelegramCallProtocolV1, TelegramCallMediaContractError> {
        let count = unsafe { (self.version_count)() };
        if count == 0 || count > MAX_LIBRARY_VERSIONS {
            return Err(TelegramCallMediaContractError::InvalidProtocol);
        }
        let mut versions = Vec::with_capacity(count);
        for index in 0..count {
            let mut output = vec![0_u8; MAX_LIBRARY_VERSION_BYTES + 1];
            let result =
                unsafe { (self.version_at)(index, output.as_mut_ptr().cast(), output.len()) };
            map_native_result(result)?;
            let version = unsafe { CStr::from_ptr(output.as_ptr().cast()) }
                .to_str()
                .map_err(|_| TelegramCallMediaContractError::InvalidProtocol)?
                .to_owned();
            versions.push(version);
        }
        versions.sort();
        versions.dedup();
        if unsafe { (self.max_layer)() } != TD_CALL_MAX_LAYER_V1 {
            return Err(TelegramCallMediaContractError::InvalidProtocol);
        }
        TelegramCallProtocolV1::new(true, true, versions)
    }

    fn session_api(&self) -> NativeSessionApi {
        NativeSessionApi {
            receive_signaling: self.session_receive_signaling,
            set_muted: self.session_set_muted,
            poll_event: self.session_poll_event,
            snapshot: self.session_snapshot,
            stop: self.session_stop,
            destroy: self.session_destroy,
        }
    }
}

unsafe fn load_symbol<T: Copy>(
    library: &Library,
    name: &[u8],
) -> Result<T, TelegramCallMediaContractError> {
    let symbol = unsafe { library.get::<T>(name) }
        .map_err(|_| TelegramCallMediaContractError::Unavailable)?;
    Ok(*symbol)
}

struct OwnedNativeServer {
    host: CString,
    username: Option<CString>,
    password: Option<Zeroizing<Vec<u8>>>,
    native: NativeServerV1,
}

impl OwnedNativeServer {
    fn reflector(
        host: &str,
        port: u16,
        reflector_id: u8,
        is_tcp: bool,
        peer_tag: [u8; 16],
    ) -> Result<Self, TelegramCallMediaContractError> {
        let host = CString::new(host).map_err(|_| TelegramCallMediaContractError::InvalidPlan)?;
        let native = NativeServerV1 {
            abi_version: TGCALLS_BRIDGE_ABI_VERSION_V1,
            kind: SERVER_TELEGRAM_REFLECTOR,
            reflector_id,
            is_tcp: u8::from(is_tcp),
            supports_stun: 0,
            supports_turn: 1,
            port,
            host: host.as_ptr(),
            username: ptr::null(),
            password: ptr::null(),
            peer_tag,
        };
        Ok(Self {
            host,
            username: None,
            password: None,
            native,
        })
    }

    fn web_rtc(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        supports_stun: bool,
        supports_turn: bool,
    ) -> Result<Self, TelegramCallMediaContractError> {
        let host = CString::new(host).map_err(|_| TelegramCallMediaContractError::InvalidPlan)?;
        let username =
            CString::new(username).map_err(|_| TelegramCallMediaContractError::InvalidPlan)?;
        let password = Zeroizing::new(
            CString::new(password)
                .map_err(|_| TelegramCallMediaContractError::InvalidPlan)?
                .into_bytes_with_nul(),
        );
        let native = NativeServerV1 {
            abi_version: TGCALLS_BRIDGE_ABI_VERSION_V1,
            kind: SERVER_WEBRTC,
            reflector_id: 0,
            is_tcp: 0,
            supports_stun: u8::from(supports_stun),
            supports_turn: u8::from(supports_turn),
            port,
            host: host.as_ptr(),
            username: username.as_ptr(),
            password: password.as_ptr().cast(),
            peer_tag: [0; 16],
        };
        Ok(Self {
            host,
            username: Some(username),
            password: Some(password),
            native,
        })
    }

    fn keepalive(&self) -> usize {
        self.host.as_bytes().len()
            + self
                .username
                .as_ref()
                .map_or(0, |value| value.as_bytes().len())
            + self.password.as_ref().map_or(0, |value| value.len())
    }
}

impl Drop for OwnedNativeServer {
    fn drop(&mut self) {
        self.native.peer_tag.zeroize();
    }
}

struct NativeSession {
    api: NativeSessionApi,
    library_guard: Arc<NativeApi>,
    handle: Option<NonNull<c_void>>,
}

impl NativeSession {
    fn snapshot(&self) -> Result<NativeSnapshotV1, TelegramCallMediaContractError> {
        let handle = self
            .handle
            .ok_or(TelegramCallMediaContractError::InvalidState)?;
        let mut snapshot = empty_snapshot();
        let result = unsafe { (self.api.snapshot)(handle.as_ptr(), &mut snapshot) };
        map_native_result(result)?;
        validate_snapshot(&snapshot)?;
        Ok(snapshot)
    }

    fn stop(mut self) -> Result<NativeSnapshotV1, TelegramCallMediaContractError> {
        let handle = self
            .handle
            .ok_or(TelegramCallMediaContractError::InvalidState)?;
        let mut snapshot = empty_snapshot();
        let result = unsafe { (self.api.stop)(handle.as_ptr(), &mut snapshot) };
        if let Err(error) = map_native_result(result) {
            self.abandon_active_native_session();
            return Err(error);
        }
        let snapshot_result = validate_snapshot(&snapshot);
        let destroy_result = unsafe { (self.api.destroy)(handle.as_ptr()) };
        if let Err(error) = map_native_result(destroy_result) {
            self.abandon_active_native_session();
            return Err(error);
        }
        self.handle = None;
        snapshot_result?;
        Ok(snapshot)
    }

    fn abandon_active_native_session(&mut self) {
        self.handle = None;
        let library_guard = Arc::clone(&self.library_guard);
        std::mem::forget(library_guard);
    }
}

impl Drop for NativeSession {
    fn drop(&mut self) {
        let Some(handle) = self.handle.take() else {
            return;
        };
        let mut snapshot = empty_snapshot();
        let stopped = unsafe { (self.api.stop)(handle.as_ptr(), &mut snapshot) };
        if stopped == RESULT_OK {
            let destroyed = unsafe { (self.api.destroy)(handle.as_ptr()) };
            if destroyed == RESULT_OK {
                return;
            }
        }
        let library_guard = Arc::clone(&self.library_guard);
        std::mem::forget(library_guard);
    }
}

pub struct TgCallsMediaAdapter {
    // Sessions must be dropped before the dynamic library that owns their code.
    sessions: HashMap<String, NativeSession>,
    protocol: TelegramCallProtocolV1,
    native: Arc<NativeApi>,
    poisoned: bool,
}

impl TgCallsMediaAdapter {
    pub fn load_exact(path: &Path) -> Result<Self, TelegramCallMediaContractError> {
        let native = Arc::new(NativeApi::load_exact(path)?);
        let protocol = native.protocol()?;
        Ok(Self {
            sessions: HashMap::new(),
            protocol,
            native,
            poisoned: false,
        })
    }
}

impl TelegramCallSignalingMediaPort for TgCallsMediaAdapter {
    fn supported_protocol(&self) -> Result<TelegramCallProtocolV1, TelegramCallMediaContractError> {
        Ok(self.protocol.clone())
    }

    fn start_session(
        &mut self,
        plan: TelegramCallReadyPlanV1,
    ) -> Result<(), TelegramCallMediaContractError> {
        plan.validate()?;
        if self.poisoned
            || self.sessions.contains_key(&plan.call_session_id)
            || !self
                .protocol
                .library_versions
                .iter()
                .any(|version| version == &plan.library_version)
        {
            return Err(TelegramCallMediaContractError::InvalidState);
        }
        let library_version = CString::new(plan.library_version.as_str())
            .map_err(|_| TelegramCallMediaContractError::InvalidPlan)?;
        let call_config = CString::new(plan.call_config.expose())
            .map_err(|_| TelegramCallMediaContractError::InvalidPlan)?;
        let custom_parameters = CString::new(plan.custom_parameters.expose())
            .map_err(|_| TelegramCallMediaContractError::InvalidPlan)?;
        let default_device =
            CString::new("").map_err(|_| TelegramCallMediaContractError::InvalidPlan)?;
        let mut owned_servers = Vec::new();
        for server in &plan.servers {
            let hosts = [server.ipv4.as_str(), server.ipv6.as_str()];
            for host in hosts.into_iter().filter(|host| !host.is_empty()) {
                let mapped = match &server.kind {
                    TelegramCallServerKindV1::TelegramReflector {
                        reflector_id,
                        peer_tag,
                        is_tcp,
                    } => OwnedNativeServer::reflector(
                        host,
                        server.port,
                        *reflector_id,
                        *is_tcp,
                        *peer_tag,
                    )?,
                    TelegramCallServerKindV1::WebRtc {
                        username,
                        password,
                        supports_stun,
                        supports_turn,
                    } => OwnedNativeServer::web_rtc(
                        host,
                        server.port,
                        username.expose(),
                        password.expose(),
                        *supports_stun,
                        *supports_turn,
                    )?,
                };
                owned_servers.push(mapped);
            }
        }
        let native_servers: Vec<_> = owned_servers
            .iter()
            .map(|server| NativeServerV1 { ..server.native })
            .collect();
        let _keepalive_bytes: usize = owned_servers.iter().map(OwnedNativeServer::keepalive).sum();
        let config = NativeSessionConfigV1 {
            abi_version: TGCALLS_BRIDGE_ABI_VERSION_V1,
            library_version: library_version.as_ptr(),
            initialization_timeout_seconds: 30.0,
            receive_timeout_seconds: 10.0,
            enable_p2p: u8::from(plan.allow_p2p),
            allow_tcp: u8::from(plan.allow_tcp),
            is_outgoing: u8::from(plan.is_outgoing),
            call_config: call_config.as_ptr(),
            custom_parameters: custom_parameters.as_ptr(),
            encryption_key: plan.encryption_key.expose().as_ptr(),
            encryption_key_length: CALL_ENCRYPTION_KEY_BYTES,
            servers: native_servers.as_ptr(),
            server_count: native_servers.len(),
            input_device_id: default_device.as_ptr(),
            output_device_id: default_device.as_ptr(),
        };
        let mut handle = ptr::null_mut();
        let result = unsafe { (self.native.session_create)(&config, &mut handle) };
        map_native_result(result)?;
        let handle = NonNull::new(handle).ok_or(TelegramCallMediaContractError::NativeFailure)?;
        let session_id = plan.call_session_id.clone();
        self.sessions.insert(
            session_id,
            NativeSession {
                api: self.native.session_api(),
                library_guard: Arc::clone(&self.native),
                handle: Some(handle),
            },
        );
        Ok(())
    }

    fn receive_signaling_data(
        &mut self,
        call_session_id: &str,
        data: TelegramCallSecretBytesV1,
    ) -> Result<(), TelegramCallMediaContractError> {
        let session = self
            .sessions
            .get(call_session_id)
            .ok_or(TelegramCallMediaContractError::SessionNotFound)?;
        let bytes = data.expose();
        if bytes.is_empty() || bytes.len() > MAX_SIGNALING_DATA_BYTES {
            return Err(TelegramCallMediaContractError::InvalidPlan);
        }
        let handle = session
            .handle
            .ok_or(TelegramCallMediaContractError::InvalidState)?;
        let result = unsafe {
            (session.api.receive_signaling)(handle.as_ptr(), bytes.as_ptr(), bytes.len())
        };
        map_native_result(result)
    }

    fn poll_event(
        &mut self,
        call_session_id: &str,
    ) -> Result<Option<TelegramCallMediaEventV1>, TelegramCallMediaContractError> {
        let session = self
            .sessions
            .get(call_session_id)
            .ok_or(TelegramCallMediaContractError::SessionNotFound)?;
        let handle = session
            .handle
            .ok_or(TelegramCallMediaContractError::InvalidState)?;
        let mut event = NativeEventV1 {
            abi_version: 0,
            kind: 0,
            state: 0,
            payload_length: 0,
        };
        let mut payload = vec![0_u8; MAX_SIGNALING_DATA_BYTES];
        let result = unsafe {
            (session.api.poll_event)(
                handle.as_ptr(),
                &mut event,
                payload.as_mut_ptr(),
                payload.len(),
            )
        };
        if result == RESULT_OK {
            return Ok(None);
        }
        if result != RESULT_EVENT {
            return Err(map_native_error(result));
        }
        if event.abi_version != TGCALLS_BRIDGE_ABI_VERSION_V1
            || event.payload_length > payload.len()
        {
            return Err(TelegramCallMediaContractError::NativeFailure);
        }
        match event.kind {
            EVENT_STATE => Ok(Some(TelegramCallMediaEventV1::State(map_state(
                event.state,
            )?))),
            EVENT_SIGNALING => {
                payload.truncate(event.payload_length);
                Ok(Some(TelegramCallMediaEventV1::OutboundSignaling(
                    TelegramCallSecretBytesV1::new(payload, MAX_SIGNALING_DATA_BYTES)?,
                )))
            }
            _ => Err(TelegramCallMediaContractError::NativeFailure),
        }
    }

    fn stop_session(
        &mut self,
        call_session_id: &str,
    ) -> Result<TelegramCallMediaFinalV1, TelegramCallMediaContractError> {
        let session = self
            .sessions
            .remove(call_session_id)
            .ok_or(TelegramCallMediaContractError::SessionNotFound)?;
        let snapshot = match session.stop() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.poisoned = true;
                return Err(error);
            }
        };
        Ok(media_final(snapshot))
    }

    fn discard_context(
        &self,
        call_session_id: &str,
    ) -> Result<TelegramCallDiscardContextV1, TelegramCallMediaContractError> {
        let session = self
            .sessions
            .get(call_session_id)
            .ok_or(TelegramCallMediaContractError::SessionNotFound)?;
        Ok(discard_context(session.snapshot()?))
    }

    fn set_local_mute(
        &mut self,
        call_session_id: &str,
        muted: bool,
    ) -> Result<(), TelegramCallMediaContractError> {
        let session = self
            .sessions
            .get(call_session_id)
            .ok_or(TelegramCallMediaContractError::SessionNotFound)?;
        let handle = session
            .handle
            .ok_or(TelegramCallMediaContractError::InvalidState)?;
        let result = unsafe { (session.api.set_muted)(handle.as_ptr(), u8::from(muted)) };
        map_native_result(result)
    }
}

fn empty_snapshot() -> NativeSnapshotV1 {
    NativeSnapshotV1 {
        abi_version: 0,
        state: 0,
        duration_seconds: 0,
        connection_id: 0,
        failed: 0,
    }
}

fn validate_snapshot(snapshot: &NativeSnapshotV1) -> Result<(), TelegramCallMediaContractError> {
    if snapshot.abi_version != TGCALLS_BRIDGE_ABI_VERSION_V1 {
        return Err(TelegramCallMediaContractError::NativeFailure);
    }
    map_state(snapshot.state).map(|_| ())
}

fn map_state(state: u32) -> Result<TelegramCallMediaStateV1, TelegramCallMediaContractError> {
    match state {
        STATE_CONNECTING => Ok(TelegramCallMediaStateV1::Connecting),
        STATE_ESTABLISHED => Ok(TelegramCallMediaStateV1::Established),
        STATE_RECONNECTING => Ok(TelegramCallMediaStateV1::Reconnecting),
        STATE_FAILED => Ok(TelegramCallMediaStateV1::Failed),
        _ => Err(TelegramCallMediaContractError::NativeFailure),
    }
}

fn discard_context(snapshot: NativeSnapshotV1) -> TelegramCallDiscardContextV1 {
    TelegramCallDiscardContextV1 {
        duration_seconds: snapshot.duration_seconds,
        connection_id: snapshot.connection_id,
    }
}

fn media_final(snapshot: NativeSnapshotV1) -> TelegramCallMediaFinalV1 {
    let failed = snapshot.failed != 0;
    TelegramCallMediaFinalV1 {
        discard_context: discard_context(snapshot),
        failed,
    }
}

fn map_native_result(result: i32) -> Result<(), TelegramCallMediaContractError> {
    if result == RESULT_OK {
        Ok(())
    } else {
        Err(map_native_error(result))
    }
}

fn map_native_error(result: i32) -> TelegramCallMediaContractError {
    match result {
        RESULT_INVALID_ARGUMENT | RESULT_BUFFER_TOO_SMALL => {
            TelegramCallMediaContractError::InvalidPlan
        }
        RESULT_UNSUPPORTED_VERSION => TelegramCallMediaContractError::InvalidProtocol,
        RESULT_INVALID_STATE => TelegramCallMediaContractError::InvalidState,
        RESULT_QUEUE_OVERFLOW => TelegramCallMediaContractError::QueueOverflow,
        RESULT_NATIVE_FAILURE => TelegramCallMediaContractError::NativeFailure,
        _ => TelegramCallMediaContractError::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_errors_are_sanitized_contract_categories() {
        assert_eq!(
            map_native_error(RESULT_UNSUPPORTED_VERSION),
            TelegramCallMediaContractError::InvalidProtocol
        );
        assert_eq!(
            map_native_error(RESULT_QUEUE_OVERFLOW),
            TelegramCallMediaContractError::QueueOverflow
        );
        assert_eq!(
            map_native_error(-500),
            TelegramCallMediaContractError::Unavailable
        );
    }

    #[test]
    #[ignore = "requires exact pinned tgcalls bridge artifact"]
    fn exact_bridge_reports_tdlib_compatible_protocol() {
        let path = std::env::var_os("MAKOSH_TGCALLS_BRIDGE_PATH")
            .map(std::path::PathBuf::from)
            .expect("MAKOSH_TGCALLS_BRIDGE_PATH");
        let adapter = TgCallsMediaAdapter::load_exact(&path).expect("exact bridge");
        let protocol = adapter.supported_protocol().expect("protocol");

        assert_eq!(protocol.max_layer, TD_CALL_MAX_LAYER_V1);
        assert!(
            protocol
                .library_versions
                .iter()
                .any(|value| value == "13.0.0")
        );
        assert!(
            protocol
                .library_versions
                .iter()
                .any(|value| value == "14.0.0")
        );
    }
}
