//! Owner-controlled WhatsApp Web host executor.
//!
//! The WebView owns provider session state in the platform WebView profile. This
//! host boundary neither reads browser storage nor extracts provider content.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::{
    fs::{MetadataExt, PermissionsExt},
    net::UnixStream,
};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use makosh_runtime_protocol::{
    v1::{
        ContractReferenceV1, ManagedIntegrationHostBridgeConfigurationV1, ModuleClientRequestV1,
        ModuleClientResponseV1,
    },
    validation::integration_host_bridge::validate_managed_integration_host_bridge_configuration,
};
use makosh_whatsapp_api::client_contract::{
    WHATSAPP_DESCRIPTOR_SET_V1, WHATSAPP_MODULE_ID, WHATSAPP_OWNER_ID,
};
use makosh_whatsapp_api::host_bridge::{
    HOST_BRIDGE_CONTRACT_MAJOR, HOST_BRIDGE_CONTRACT_NAME, HOST_BRIDGE_CONTRACT_REVISION,
    HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION, WhatsAppHostBridgeEnvelopeV1,
    WhatsAppHostBridgeHandshakeV1, WhatsAppHostObservationV1,
    decode_host_bridge_handshake_accepted, decode_host_bridge_observation_accepted,
    encode_host_bridge_handshake, encode_host_bridge_payload,
};
use prost::Message;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const PROVIDER_SHAPE: &str = "whatsapp_web_companion";
const RUNTIME_KIND: &str = "webview_companion";
const WINDOW_LABEL_PREFIX: &str = "whatsapp-companion";
const WHATSAPP_WEB_URL: &str = "https://web.whatsapp.com/";
const MAX_HOST_DESCRIPTOR_BYTES: u64 = 8 * 1024;
const MAX_HOST_FRAME_BYTES: usize = 512 * 1024;
const HOST_BRIDGE_TIMEOUT: Duration = Duration::from_secs(5);
const WHATSAPP_MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const HOST_ROUTE_ATTACHED_STATE: &str = "host_route_attached";
const WEBVIEW_LOADED_STATE: &str = "webview_loaded";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct WhatsAppWebCompanionRequest {
    pub(crate) account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct WhatsAppRuntimeBridgeRequest {
    pub(crate) account_id: String,
    pub(crate) registration_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppRuntimeBridgeAttachment {
    pub(crate) account_id: String,
    pub(crate) registration_id: String,
    pub(crate) runtime_generation: u64,
    pub(crate) route_state: &'static str,
}

#[derive(Clone)]
struct ActiveWhatsAppHostRoute {
    account_id: String,
    route: ManagedIntegrationHostBridgeConfigurationV1,
}

#[derive(Default)]
pub(crate) struct WhatsAppHostRoutes {
    routes: Mutex<HashMap<String, ActiveWhatsAppHostRoute>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionManifest {
    pub(crate) account_id: String,
    pub(crate) provider_shape: &'static str,
    pub(crate) runtime_kind: &'static str,
    pub(crate) driver_id: &'static str,
    pub(crate) window_label: String,
    pub(crate) target_url: &'static str,
    pub(crate) opened_window: bool,
    pub(crate) reused_existing_window: bool,
    pub(crate) owner_visible: bool,
    pub(crate) hidden_headless_mode: &'static str,
    pub(crate) tauri_ipc_available_to_companion_window: bool,
    pub(crate) event_flow: &'static str,
    pub(crate) event_extractor: WhatsAppWebCompanionExtractorContract,
    pub(crate) bridge_routes: WhatsAppWebCompanionBridgeRoutes,
    pub(crate) command_channel: WhatsAppWebCompanionCommandChannel,
    pub(crate) secret_policy: WhatsAppWebCompanionSecretPolicy,
    pub(crate) remaining_blockers: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionExtractorContract {
    pub(crate) state: &'static str,
    pub(crate) initialization_script: &'static str,
    pub(crate) script_scope: &'static str,
    pub(crate) origin_guard: &'static str,
    pub(crate) navigation_guard: &'static str,
    pub(crate) relay_channel: &'static str,
    pub(crate) runtime_bridge_dispatch: &'static str,
    pub(crate) allowed_observations: Vec<&'static str>,
    pub(crate) forbidden_reads: Vec<&'static str>,
    pub(crate) next_gate: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionBridgeRoutes {
    pub(crate) authorized_session_path: &'static str,
    pub(crate) runtime_event_path: &'static str,
    pub(crate) sync_lifecycle_path: &'static str,
    pub(crate) message_paths: Vec<&'static str>,
    pub(crate) conversation_paths: Vec<&'static str>,
    pub(crate) media_paths: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionCommandChannel {
    pub(crate) kind: &'static str,
    pub(crate) claim_path: &'static str,
    pub(crate) failure_path: &'static str,
    pub(crate) result_path: &'static str,
    pub(crate) completion_rule: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WhatsAppWebCompanionSecretPolicy {
    pub(crate) session_material: &'static str,
    pub(crate) cookies: &'static str,
    pub(crate) browser_profile_secrets: &'static str,
    pub(crate) qr_pair_code_artifacts: &'static str,
    pub(crate) message_bodies: &'static str,
    pub(crate) media_bytes: &'static str,
    pub(crate) postgres_storage: &'static str,
}

#[tauri::command]
pub(crate) async fn whatsapp_web_companion_manifest(
    app: AppHandle,
    request: WhatsAppWebCompanionRequest,
) -> Result<WhatsAppWebCompanionManifest, String> {
    manifest_for_account(&app, &request.account_id, false, false)
}

#[tauri::command]
pub(crate) async fn start_hidden_whatsapp_webview(
    app: AppHandle,
    request: WhatsAppWebCompanionRequest,
) -> Result<WhatsAppWebCompanionManifest, String> {
    ensure_companion_window(&app, &request.account_id, false)
}

#[tauri::command]
pub(crate) async fn open_whatsapp_web_companion(
    app: AppHandle,
    request: WhatsAppWebCompanionRequest,
) -> Result<WhatsAppWebCompanionManifest, String> {
    ensure_companion_window(&app, &request.account_id, true)
}

#[tauri::command]
pub(crate) async fn hide_whatsapp_web_companion(
    app: AppHandle,
    request: WhatsAppWebCompanionRequest,
) -> Result<WhatsAppWebCompanionManifest, String> {
    let account_id = required_account_id(&request.account_id)?;
    let window_label = companion_window_label(account_id)?;
    let window = app
        .get_webview_window(&window_label)
        .ok_or_else(|| "WhatsApp companion window is not running".to_owned())?;
    window
        .hide()
        .map_err(|error| format!("failed to hide WhatsApp companion window: {error}"))?;
    manifest_for_account(&app, account_id, false, true)
}

#[tauri::command]
pub(crate) async fn connect_whatsapp_runtime_bridge(
    app: AppHandle,
    routes: State<'_, WhatsAppHostRoutes>,
    request: WhatsAppRuntimeBridgeRequest,
) -> Result<WhatsAppRuntimeBridgeAttachment, String> {
    let account_id = required_account_id(&request.account_id)?;
    let registration_id = required_registration_id(&request.registration_id)?;
    let route = load_active_host_route(&app, registration_id)?;
    let window_label = companion_window_label(account_id)?;
    connect_host_route(&route)?;
    routes
        .routes
        .lock()
        .map_err(|_| "WhatsApp host route state is unavailable".to_owned())?
        .insert(
            window_label,
            ActiveWhatsAppHostRoute {
                account_id: account_id.to_owned(),
                route: route.clone(),
            },
        );
    ensure_companion_window(&app, account_id, false)?;
    relay_runtime_state(
        &ActiveWhatsAppHostRoute {
            account_id: account_id.to_owned(),
            route: route.clone(),
        },
        HOST_ROUTE_ATTACHED_STATE,
    )?;
    Ok(WhatsAppRuntimeBridgeAttachment {
        account_id: account_id.to_owned(),
        registration_id: registration_id.to_owned(),
        runtime_generation: route.runtime_generation,
        route_state: "connected_exact_kernel_route",
    })
}

#[tauri::command]
pub(crate) async fn whatsapp_web_companion_relay_runtime_state(
    webview_window: WebviewWindow,
    routes: State<'_, WhatsAppHostRoutes>,
) -> Result<(), String> {
    let route = routes
        .routes
        .lock()
        .map_err(|_| "WhatsApp host route state is unavailable".to_owned())?
        .get(webview_window.label())
        .cloned()
        .ok_or_else(|| "WhatsApp host route is unavailable".to_owned())?;
    relay_runtime_state(&route, WEBVIEW_LOADED_STATE)
}

fn ensure_companion_window(
    app: &AppHandle,
    account_id: &str,
    owner_visible: bool,
) -> Result<WhatsAppWebCompanionManifest, String> {
    let account_id = required_account_id(account_id)?;
    let window_label = companion_window_label(account_id)?;
    if let Some(window) = app.get_webview_window(&window_label) {
        if owner_visible {
            window
                .show()
                .map_err(|error| format!("failed to show WhatsApp companion window: {error}"))?;
            window
                .set_focus()
                .map_err(|error| format!("failed to focus WhatsApp companion window: {error}"))?;
        } else {
            window
                .hide()
                .map_err(|error| format!("failed to hide WhatsApp companion window: {error}"))?;
        }
        return manifest_for_account(app, account_id, false, true);
    }

    let url = WHATSAPP_WEB_URL
        .parse()
        .map_err(|error| format!("invalid WhatsApp Web URL: {error}"))?;
    WebviewWindowBuilder::new(app, window_label, WebviewUrl::External(url))
        .title("WhatsApp Web companion")
        .visible(owner_visible)
        .resizable(true)
        .initialization_script(whatsapp_runtime_state_initialization_script())
        .on_navigation(|url| url.scheme() == "https" && url.host_str() == Some("web.whatsapp.com"))
        .build()
        .map_err(|error| format!("failed to start WhatsApp WebView companion: {error}"))?;

    manifest_for_account(app, account_id, true, false)
}

fn manifest_for_account(
    app: &AppHandle,
    account_id: &str,
    opened_window: bool,
    reused_existing_window: bool,
) -> Result<WhatsAppWebCompanionManifest, String> {
    let account_id = required_account_id(account_id)?;
    let window_label = companion_window_label(account_id)?;
    let owner_visible = app
        .get_webview_window(&window_label)
        .map(|window| window.is_visible().unwrap_or(false))
        .unwrap_or(false);
    Ok(WhatsAppWebCompanionManifest {
        account_id: account_id.to_owned(),
        provider_shape: PROVIDER_SHAPE,
        runtime_kind: RUNTIME_KIND,
        driver_id: "tauri_host_only_whatsapp_webview",
        window_label,
        target_url: WHATSAPP_WEB_URL,
        opened_window,
        reused_existing_window,
        owner_visible,
        hidden_headless_mode: "hidden_tauri_webview_after_explicit_owner_pairing",
        tauri_ipc_available_to_companion_window: true,
        event_flow: "explicit_owner_pairing -> active_exact_host_route -> sanitized_runtime_lifecycle_observation",
        event_extractor: WhatsAppWebCompanionExtractorContract {
            state: "runtime_lifecycle_relay_only_no_provider_data_extractor",
            initialization_script: "installed_on_hidden_companion_webview",
            script_scope: "main_frame_only",
            origin_guard: "https://web.whatsapp.com",
            navigation_guard: "https://web.whatsapp.com_only",
            relay_channel: "whatsapp_web_companion_relay_runtime_state_without_payload",
            runtime_bridge_dispatch: "exact_active_route_native_typed_client_request",
            allowed_observations: vec!["host_route_attached", "webview_loaded"],
            forbidden_reads: vec![
                "cookies",
                "web_storage",
                "indexed_db",
                "browser_profile_secrets",
                "session_material",
                "message_bodies",
                "media_bytes",
            ],
            next_gate: "provider_dom_metadata_extractor_with_live_smoke",
        },
        bridge_routes: WhatsAppWebCompanionBridgeRoutes {
            authorized_session_path: "not_available",
            runtime_event_path: "exact_active_route_runtime_lifecycle_only",
            sync_lifecycle_path: "not_available",
            message_paths: Vec::new(),
            conversation_paths: Vec::new(),
            media_paths: Vec::new(),
        },
        command_channel: WhatsAppWebCompanionCommandChannel {
            kind: "not_available",
            claim_path: "not_available",
            failure_path: "not_available",
            result_path: "not_available",
            completion_rule: "provider_commands_require_admitted_whatsapp_runtime",
        },
        secret_policy: WhatsAppWebCompanionSecretPolicy {
            session_material: "owned_by_os_managed_webview_profile",
            cookies: "not_read_or_returned_by_tauri",
            browser_profile_secrets: "not_read_or_returned_by_tauri",
            qr_pair_code_artifacts: "visible_only_in_owner_controlled_webview",
            message_bodies: "not_read_by_host_executor",
            media_bytes: "not_read_by_host_executor",
            postgres_storage: "not_used_by_host_executor",
        },
        remaining_blockers: vec![
            "provider_dom_metadata_extractor_not_implemented",
            "provider_command_executor_not_implemented",
            "manual_live_pairing_smoke_required",
        ],
    })
}

fn companion_window_label(account_id: &str) -> Result<String, String> {
    let account_id = required_account_id(account_id)?;
    let mut sanitized = String::with_capacity(account_id.len());
    let mut previous_dash = false;
    for character in account_id.chars() {
        let next = if character.is_ascii_alphanumeric() {
            previous_dash = false;
            character.to_ascii_lowercase()
        } else if character == '-' || character == '_' {
            if previous_dash {
                continue;
            }
            previous_dash = character == '-';
            character
        } else if !previous_dash {
            previous_dash = true;
            '-'
        } else {
            continue;
        };
        sanitized.push(next);
        if sanitized.len() >= 64 {
            break;
        }
    }
    let sanitized = sanitized.trim_matches('-').trim_matches('_');
    if sanitized.is_empty() {
        return Err("account_id does not contain a valid window label segment".to_owned());
    }
    Ok(format!("{WINDOW_LABEL_PREFIX}-{sanitized}"))
}

fn required_account_id(account_id: &str) -> Result<&str, String> {
    let account_id = account_id.trim();
    if account_id.is_empty() {
        return Err("account_id is required for WhatsApp companion runtime".to_owned());
    }
    Ok(account_id)
}

fn required_registration_id(registration_id: &str) -> Result<&str, String> {
    let registration_id = registration_id.trim();
    (!registration_id.is_empty()
        && registration_id.len() <= 128
        && registration_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')))
    .then_some(registration_id)
    .ok_or_else(|| "registration_id is invalid for WhatsApp companion runtime".to_owned())
}

fn load_active_host_route(
    app: &AppHandle,
    registration_id: &str,
) -> Result<ManagedIntegrationHostBridgeConfigurationV1, String> {
    let data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "Kernel data directory is unavailable".to_owned())?;
    let runtime_dir = kernel_runtime_directory(&data_dir)?;
    let path = runtime_dir
        .join("host-bridges")
        .join(host_descriptor_file_name(registration_id));
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|_| "WhatsApp host route is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_file()
        || metadata.uid() != current_uid()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() == 0
        || metadata.len() > MAX_HOST_DESCRIPTOR_BYTES
    {
        return Err("WhatsApp host route is unavailable".to_owned());
    }
    let route = ManagedIntegrationHostBridgeConfigurationV1::decode(
        std::fs::read(&path)
            .map_err(|_| "WhatsApp host route is unavailable".to_owned())?
            .as_slice(),
    )
    .map_err(|_| "WhatsApp host route is unavailable".to_owned())?;
    validate_managed_integration_host_bridge_configuration(&route)
        .map_err(|_| "WhatsApp host route is unavailable".to_owned())?;
    if route.owner_id != "whatsapp" || route.registration_id != registration_id {
        return Err("WhatsApp host route is unavailable".to_owned());
    }
    Ok(route)
}

fn connect_host_route(
    route: &ManagedIntegrationHostBridgeConfigurationV1,
) -> Result<UnixStream, String> {
    let route_binding_sha256: [u8; 32] = route
        .route_binding_sha256
        .as_slice()
        .try_into()
        .map_err(|_| "WhatsApp host route is unavailable".to_owned())?;
    let handshake = encode_host_bridge_handshake(&WhatsAppHostBridgeHandshakeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        route_binding_sha256,
    })
    .map_err(|_| "WhatsApp host route is unavailable".to_owned())?;
    let mut stream = UnixStream::connect(&route.socket_path)
        .map_err(|_| "WhatsApp host runtime is unavailable".to_owned())?;
    stream
        .set_read_timeout(Some(HOST_BRIDGE_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(HOST_BRIDGE_TIMEOUT)))
        .map_err(|_| "WhatsApp host runtime is unavailable".to_owned())?;
    write_frame(&mut stream, &handshake)?;
    decode_host_bridge_handshake_accepted(&read_frame(&mut stream)?)
        .map_err(|_| "WhatsApp host runtime rejected its route".to_owned())?;
    Ok(stream)
}

fn relay_runtime_state(route: &ActiveWhatsAppHostRoute, state: &'static str) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "WhatsApp host runtime is unavailable".to_owned())?;
    let observed_at_unix_seconds = i64::try_from(now.as_secs())
        .map_err(|_| "WhatsApp host runtime is unavailable".to_owned())?;
    let provider_event_id = format!(
        "host-runtime-{}-{}-{}",
        route.route.runtime_generation,
        observed_at_unix_seconds,
        now.subsec_nanos(),
    );
    let payload = encode_host_bridge_payload(&WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: route.account_id.clone(),
        provider_event_id: provider_event_id.clone(),
        observed_at_unix_seconds,
        observation: WhatsAppHostObservationV1::RuntimeState {
            state: state.to_owned(),
        },
    })
    .map_err(|_| "WhatsApp host runtime is unavailable".to_owned())?;
    let request = ModuleClientRequestV1 {
        protocol_major: WHATSAPP_MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: WHATSAPP_MODULE_ID.to_owned(),
        owner_id: WHATSAPP_OWNER_ID.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: WHATSAPP_OWNER_ID.to_owned(),
            name: HOST_BRIDGE_CONTRACT_NAME.to_owned(),
            major: HOST_BRIDGE_CONTRACT_MAJOR,
            revision: HOST_BRIDGE_CONTRACT_REVISION,
            schema_sha256: Sha256::digest(WHATSAPP_DESCRIPTOR_SET_V1).to_vec(),
        }),
        request_id: 1,
        request_payload: payload,
        logical_owner_id: String::new(),
        authenticated_device_id: String::new(),
        authenticated_client_session_id: String::new(),
    };
    let mut stream = connect_host_route(&route.route)?;
    write_frame(&mut stream, &request.encode_to_vec())?;
    let response = ModuleClientResponseV1::decode(read_frame(&mut stream)?.as_slice())
        .map_err(|_| "WhatsApp host runtime rejected its observation".to_owned())?;
    if response.protocol_major != WHATSAPP_MODULE_CLIENT_PROTOCOL_MAJOR
        || response.request_id != 1
        || !response.error_code.is_empty()
    {
        return Err("WhatsApp host runtime rejected its observation".to_owned());
    }
    let accepted_provider_event_id =
        decode_host_bridge_observation_accepted(response.response_payload.as_slice())
            .map_err(|_| "WhatsApp host runtime rejected its observation".to_owned())?;
    (accepted_provider_event_id == provider_event_id)
        .then_some(())
        .ok_or_else(|| "WhatsApp host runtime rejected its observation".to_owned())
}

fn whatsapp_runtime_state_initialization_script() -> &'static str {
    r#"
(() => {
  if (window.location.hostname !== 'web.whatsapp.com') return;
  const invoke = window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke;
  if (typeof invoke !== 'function') return;
  window.addEventListener('load', () => {
    void invoke('whatsapp_web_companion_relay_runtime_state');
  }, { once: true });
})();
"#
}

fn kernel_runtime_directory(data_dir: &Path) -> Result<PathBuf, String> {
    let data_dir = data_dir
        .canonicalize()
        .map_err(|_| "Kernel data directory is unavailable".to_owned())?;
    let project = directories::ProjectDirs::from("dev", "makosh", "makosh")
        .ok_or_else(|| "Kernel runtime directory is unavailable".to_owned())?;
    let digest = Sha256::digest(data_dir.as_os_str().as_encoded_bytes());
    let instance_key = digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(project.cache_dir().join("runtime").join(instance_key))
}

fn host_descriptor_file_name(registration_id: &str) -> String {
    let digest = Sha256::digest(registration_id.as_bytes());
    format!(
        "route-{}.bin",
        digest[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
    )
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > MAX_HOST_FRAME_BYTES {
        return Err("WhatsApp host runtime is unavailable".to_owned());
    }
    let mut length = u32::try_from(bytes.len())
        .map_err(|_| "WhatsApp host runtime is unavailable".to_owned())?;
    let mut prefix = [0_u8; 5];
    let mut index = 0;
    while length >= 0x80 {
        prefix[index] = (length as u8 & 0x7f) | 0x80;
        length >>= 7;
        index += 1;
    }
    prefix[index] = length as u8;
    stream
        .write_all(&prefix[..=index])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|_| "WhatsApp host runtime is unavailable".to_owned())
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let mut length = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|_| "WhatsApp host runtime is unavailable".to_owned())?;
        length |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            let length = usize::try_from(length)
                .map_err(|_| "WhatsApp host runtime is unavailable".to_owned())?;
            if length == 0 || length > MAX_HOST_FRAME_BYTES {
                return Err("WhatsApp host runtime is unavailable".to_owned());
            }
            let mut bytes = vec![0_u8; length];
            stream
                .read_exact(&mut bytes)
                .map_err(|_| "WhatsApp host runtime is unavailable".to_owned())?;
            return Ok(bytes);
        }
    }
    Err("WhatsApp host runtime is unavailable".to_owned())
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn companion_window_label_is_account_scoped_and_stable() {
        assert_eq!(
            companion_window_label(" Wa Live/Primary ").expect("label"),
            "whatsapp-companion-wa-live-primary"
        );
        assert_eq!(
            companion_window_label("wa_live_primary").expect("label"),
            "whatsapp-companion-wa_live_primary"
        );
    }

    #[test]
    fn lifecycle_script_has_an_exact_origin_guard_and_no_payload() {
        let script = whatsapp_runtime_state_initialization_script();

        assert!(script.contains("window.location.hostname !== 'web.whatsapp.com'"));
        assert!(script.contains("invoke('whatsapp_web_companion_relay_runtime_state')"));
        assert!(!script.contains("JSON.stringify"));
    }
}
