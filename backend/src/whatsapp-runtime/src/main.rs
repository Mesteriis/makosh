//! WhatsApp integration process root for exact Kernel-inherited contracts.

use std::ffi::{OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use makosh_runtime_protocol::{
    v1::{ManagedIntegrationHostBridgeConfigurationV1, ManagedIntegrationRuntimeConfigurationV1},
    validation::{
        descriptor::{
            decode_settings_schema_v1, decode_settings_snapshot_v1,
            validate_settings_snapshot_against_schema_v1,
        },
        integration_host_bridge::validate_managed_integration_host_bridge_configuration,
        managed_integration_runtime::validate_managed_integration_runtime_configuration,
    },
};
use makosh_whatsapp_runtime::host_bridge_transport::{
    WhatsAppHostBridgeSession, WhatsAppHostBridgeSessionProgress,
};
use makosh_whatsapp_runtime::{WhatsAppRuntimeAdmission, managed, settings};
use prost::Message;

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    settings_snapshot: PathBuf,
    runtime_configuration: PathBuf,
    host_bridge_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    match arguments.next().as_deref() {
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(&mut arguments.peekable())
        }
        _ => Err("WhatsApp runtime command is unavailable".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let schema_bytes = read_contract(&paths.settings_schema)?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "WhatsApp runtime settings schema is invalid".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&read_contract(&paths.settings_snapshot)?)
        .map_err(|_| "WhatsApp runtime settings snapshot is invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| "WhatsApp runtime settings snapshot is invalid".to_owned())?;
    let runtime_settings = settings::decode(&snapshot)?;
    let configuration = ManagedIntegrationRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "WhatsApp runtime configuration is invalid".to_owned())?;
    validate_managed_integration_runtime_configuration(&configuration)
        .map_err(|_| "WhatsApp runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("WhatsApp runtime configuration is stale".to_owned());
    }
    let host_bridge_configuration = ManagedIntegrationHostBridgeConfigurationV1::decode(
        read_contract(&paths.host_bridge_configuration)?.as_slice(),
    )
    .map_err(|_| "WhatsApp host bridge configuration is invalid".to_owned())?;
    validate_managed_integration_host_bridge_configuration(&host_bridge_configuration)
        .map_err(|_| "WhatsApp host bridge configuration is invalid".to_owned())?;
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "WhatsApp runtime configuration is invalid".to_owned())?;
    let admission = WhatsAppRuntimeAdmission {
        logical_owner_id: configuration.logical_owner_id,
        logical_human_owner_id: configuration.logical_human_owner_id,
        module_registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "WhatsApp runtime executor is unavailable".to_owned())?;
    let mut admitted = executor
        .block_on(managed::open_admitted_runtime(
            inherited_control_channel()?,
            descriptor,
            schema_bytes,
            &runtime_settings,
            &admission,
            storage,
            host_bridge_configuration,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        ))
        .map_err(|error| format!("WhatsApp runtime admission was rejected: {error:?}"))?;
    let listener = admitted
        .bind_host_bridge_listener()
        .map_err(|_| "WhatsApp host bridge listener is unavailable".to_owned())?;
    listener
        .set_nonblocking(true)
        .map_err(|_| "WhatsApp host bridge listener is unavailable".to_owned())?;
    let mut host_bridge_session: Option<WhatsAppHostBridgeSession> = None;

    loop {
        let now = unix_seconds()?;
        executor
            .block_on(admitted.try_handle_client_delivery(now))
            .map_err(|_| "WhatsApp runtime client delivery failed".to_owned())?;
        match executor.block_on(admitted.consume_next_delivery_intent(now)) {
            Ok(_) | Err(
                makosh_whatsapp_runtime::delivery_intent_consumer::WhatsAppDeliveryIntentConsumeErrorV1::Unavailable,
            ) => {}
            Err(_) => {
                return Err("WhatsApp delivery-intent consume failed".to_owned());
            }
        }
        executor
            .block_on(admitted.process_next_delivery_intent(now))
            .map_err(|_| "WhatsApp delivery-intent worker failed".to_owned())?;
        if host_bridge_session.is_none() {
            host_bridge_session = admitted
                .try_accept_host_bridge_session(&listener)
                .map_err(|_| "WhatsApp host bridge delivery failed".to_owned())?;
        }
        if host_bridge_session.as_mut().is_some_and(|session| {
            session.try_handle_once(&mut admitted, executor.handle())
                == WhatsAppHostBridgeSessionProgress::Closed
        }) {
            host_bridge_session = None;
        }
        admitted
            .publish_pending_operational_realtime(unix_millis()?)
            .map_err(|_| "WhatsApp operational realtime publication failed".to_owned())?;
        match executor.block_on(admitted.relay_communications_outbox(now)) {
            Ok(_)
            | Err(makosh_whatsapp_runtime::WhatsAppCommunicationsOutboxRelayError::Unavailable) => {
            }
            Err(makosh_whatsapp_runtime::WhatsAppCommunicationsOutboxRelayError::Persistence(
                _,
            )) => {
                return Err("WhatsApp runtime outbox persistence failed".to_owned());
            }
        }
        match executor.block_on(admitted.relay_delivery_intent_outbox(now)) {
            Ok(_)
            | Err(
                makosh_whatsapp_runtime::delivery_intent_outbox::WhatsAppDeliveryIntentOutboxRelayErrorV1::Unavailable,
            ) => {}
            Err(
                makosh_whatsapp_runtime::delivery_intent_outbox::WhatsAppDeliveryIntentOutboxRelayErrorV1::Persistence(
                    _,
                ),
            ) => {
                return Err("WhatsApp delivery-intent outbox persistence failed".to_owned());
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn unix_millis() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system time is before the Unix epoch".to_owned())
        .and_then(|duration| {
            u64::try_from(duration.as_millis())
                .map_err(|_| "system time exceeds the realtime timestamp range".to_owned())
        })
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let settings_snapshot = required_path(arguments, "--settings-snapshot-path")?;
    let runtime_configuration = required_path(arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_string(arguments, "--runtime-instance-id")?;
    let host_bridge_configuration = required_path(arguments, "--host-bridge-configuration-path")?;
    if arguments.next().is_some() || runtime_instance_id.trim().is_empty() {
        return Err("WhatsApp runtime arguments are invalid".to_owned());
    }
    Ok(InheritedPaths {
        descriptor,
        settings_schema,
        settings_snapshot,
        runtime_configuration,
        host_bridge_configuration,
        runtime_instance_id,
    })
}

fn required_path<I>(arguments: &mut I, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    required_string(arguments, name).map(PathBuf::from)
}

fn required_string<I>(arguments: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err("WhatsApp runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| "WhatsApp runtime arguments are invalid".to_owned())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("WhatsApp runtime inherited control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn unix_seconds() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "WhatsApp runtime clock is unavailable".to_owned())
        .and_then(|elapsed| {
            i64::try_from(elapsed.as_secs())
                .map_err(|_| "WhatsApp runtime clock is unavailable".to_owned())
        })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "WhatsApp runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("WhatsApp runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "WhatsApp runtime contract is unavailable".to_owned())
}
