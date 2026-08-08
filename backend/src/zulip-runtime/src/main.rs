//! Zulip integration process root for the exact Kernel-inherited runtime contract.

use std::ffi::{OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use makosh_runtime_protocol::{
    v1::ManagedIntegrationRuntimeConfigurationV1,
    validation::{
        descriptor::{
            decode_settings_schema_v1, decode_settings_snapshot_v1,
            validate_settings_snapshot_against_schema_v1,
        },
        managed_integration_runtime::validate_managed_integration_runtime_configuration,
    },
};
use makosh_zulip_runtime::{ZulipRuntimeAdmissionV1, managed, settings};
use prost::Message;

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    settings_snapshot: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    match arguments.next().as_deref() {
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(&mut arguments.peekable())
        }
        _ => Err("Zulip runtime command is unavailable".to_owned()),
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
        .map_err(|_| "Zulip runtime settings schema is invalid".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&read_contract(&paths.settings_snapshot)?)
        .map_err(|_| "Zulip runtime settings snapshot is invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| "Zulip runtime settings snapshot is invalid".to_owned())?;
    let configuration = ManagedIntegrationRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Zulip runtime configuration is invalid".to_owned())?;
    validate_managed_integration_runtime_configuration(&configuration)
        .map_err(|_| "Zulip runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Zulip runtime configuration is stale".to_owned());
    }
    let provider_settings = settings::decode(&snapshot)?;
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Zulip runtime configuration is invalid".to_owned())?;
    let admission = ZulipRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        configuration_instance_id: configuration.configuration_instance_id,
        module_registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
        vault_runtime_generation: storage.vault_runtime_generation,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Zulip runtime executor is unavailable".to_owned())?;
    let mut admitted = executor
        .block_on(managed::open_admitted_runtime(
            inherited_control_channel()?,
            descriptor,
            schema_bytes,
            &admission,
            provider_settings.account,
            storage,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        ))
        .map_err(|error| {
            developer_diagnostic(&format!(
                "developer_zulip_runtime_admission_error={error:?}"
            ));
            "Zulip runtime admission was rejected".to_owned()
        })?;
    let mut queue = executor
        .block_on(admitted.acquire_event_queue())
        .map_err(|error| {
            developer_diagnostic(&format!("developer_zulip_runtime_queue_error={error:?}"));
            "Zulip runtime event queue is unavailable".to_owned()
        })?;
    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Zulip runtime clock is unavailable".to_owned())?;
        let seconds = i64::try_from(now.as_secs())
            .map_err(|_| "Zulip runtime clock is unavailable".to_owned())?;
        let nanos = i32::try_from(now.subsec_nanos())
            .map_err(|_| "Zulip runtime clock is unavailable".to_owned())?;
        executor
            .block_on(admitted.try_handle_client_delivery(seconds))
            .map_err(|error| {
                developer_diagnostic(&format!(
                    "developer_zulip_runtime_client_delivery_error={error:?}"
                ));
                "Zulip runtime client delivery failed".to_owned()
            })?;
        executor
            .block_on(admitted.run_tick(&mut queue, seconds, nanos))
            .map_err(|error| {
                developer_diagnostic(&format!("developer_zulip_runtime_tick_error={error:?}"));
                "Zulip runtime tick failed".to_owned()
            })?;
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn developer_diagnostic(message: &str) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("{message}");
    }
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
    if arguments.next().is_some() || runtime_instance_id.trim().is_empty() {
        return Err("Zulip runtime arguments are invalid".to_owned());
    }
    Ok(InheritedPaths {
        descriptor,
        settings_schema,
        settings_snapshot,
        runtime_configuration,
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
        return Err("Zulip runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| "Zulip runtime arguments are invalid".to_owned())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Zulip runtime inherited control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Zulip runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Zulip runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Zulip runtime contract is unavailable".to_owned())
}
