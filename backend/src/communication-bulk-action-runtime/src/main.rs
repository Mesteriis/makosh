use std::ffi::{OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use makosh_communication_bulk_action_persistence::schema::communication_bulk_action_storage_bundle_v1;
use makosh_communication_bulk_action_runtime::{
    admission::{
        communication_bulk_action_module_descriptor_v1,
        communication_bulk_action_settings_schema_bytes_v1,
    },
    managed_runtime::{
        BulkDeliveryManagedRuntimeErrorV1, BulkDeliveryManagedRuntimeV1,
        BulkDeliveryRuntimeAdmissionV1,
    },
};
use makosh_runtime_protocol::{
    managed_runtime_poll::ManagedRuntimePollBackoffV1,
    v1::ManagedWorkflowRuntimeConfigurationV1,
    validation::{
        descriptor::decode_settings_schema_v1,
        managed_workflow_runtime::validate_managed_workflow_runtime_configuration,
    },
};
use prost::Message;

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    let command = arguments.next();
    let mut arguments = arguments.peekable();
    match command.as_deref() {
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(&mut arguments)
        }
        Some(command) if command == OsStr::new("export-storage-bundle") => export_bytes(
            &mut arguments,
            communication_bulk_action_storage_bundle_v1().encode_to_vec(),
        ),
        Some(command) if command == OsStr::new("export-module-descriptor") => {
            let build_id = arguments
                .next()
                .and_then(|value| value.into_string().ok())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Bulk Delivery descriptor build id is required".to_owned())?;
            export_bytes(
                &mut arguments,
                communication_bulk_action_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some(command) if command == OsStr::new("export-settings-schema") => export_bytes(
            &mut arguments,
            communication_bulk_action_settings_schema_bytes_v1(),
        ),
        _ => Err("Bulk Delivery runtime command is unavailable".to_owned()),
    }
}

fn export_bytes<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    require_no_arguments(arguments)?;
    std::io::Write::write_all(&mut std::io::stdout(), &bytes)
        .map_err(|_| "Bulk Delivery runtime artifact is unavailable".to_owned())
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let settings_schema = read_contract(&paths.settings_schema)?;
    decode_settings_schema_v1(&settings_schema)
        .map_err(|_| "Bulk Delivery settings schema is invalid".to_owned())?;
    let configuration = ManagedWorkflowRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Bulk Delivery runtime configuration is invalid".to_owned())?;
    validate_managed_workflow_runtime_configuration(&configuration)
        .map_err(|_| "Bulk Delivery runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Bulk Delivery runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Bulk Delivery storage is unavailable".to_owned())?;
    let worker_id = format!("{}:worker", configuration.runtime_instance_id);
    let admission = BulkDeliveryRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Bulk Delivery executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(BulkDeliveryManagedRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            settings_schema,
            &admission,
            storage,
        ))
        .map_err(runtime_error)?;
    let mut poll_backoff =
        ManagedRuntimePollBackoffV1::new(Duration::from_millis(25), Duration::from_millis(100))
            .map_err(|_| "Bulk Delivery runtime polling bounds are invalid".to_owned())?;
    loop {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Bulk Delivery clock is invalid".to_owned())?
            .as_secs()
            .try_into()
            .map_err(|_| "Bulk Delivery clock is invalid".to_owned())?;
        let mut progressed = executor
            .block_on(runtime.pump_control_once(now))
            .map_err(runtime_error)?;
        progressed |= match executor.block_on(runtime.process_next_target(&worker_id, now)) {
            Ok(progressed) => progressed,
            Err(BulkDeliveryManagedRuntimeErrorV1::Unavailable)
            | Err(BulkDeliveryManagedRuntimeErrorV1::Persistence(_)) => false,
            Err(error) => return Err(runtime_error(error)),
        };
        progressed |= match executor.block_on(runtime.pump_client_realtime_once()) {
            Ok(progressed) => progressed,
            Err(BulkDeliveryManagedRuntimeErrorV1::Unavailable)
            | Err(BulkDeliveryManagedRuntimeErrorV1::Persistence(_)) => false,
            Err(error) => return Err(runtime_error(error)),
        };
        let delay = poll_backoff.observe(progressed);
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
    }
}

fn runtime_error(error: BulkDeliveryManagedRuntimeErrorV1) -> String {
    let reason = match error {
        BulkDeliveryManagedRuntimeErrorV1::Admission => "admission_rejected",
        BulkDeliveryManagedRuntimeErrorV1::Persistence(_) => "storage_unavailable",
        BulkDeliveryManagedRuntimeErrorV1::InvalidTransition => "realtime_transition_rejected",
        BulkDeliveryManagedRuntimeErrorV1::Unavailable => "control_channel_unavailable",
    };
    format!("Bulk Delivery runtime failed: {reason}")
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let runtime_configuration = required_path(arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_string(arguments, "--runtime-instance-id")?;
    require_no_arguments(arguments)?;
    if runtime_instance_id.trim().is_empty() {
        return Err("Bulk Delivery runtime arguments are invalid".to_owned());
    }
    Ok(InheritedPaths {
        descriptor,
        settings_schema,
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
        return Err("Bulk Delivery runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| "Bulk Delivery runtime arguments are invalid".to_owned())
}

fn require_no_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Bulk Delivery runtime command is unavailable".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Bulk Delivery control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Bulk Delivery contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Bulk Delivery contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Bulk Delivery contract is unavailable".to_owned())
}
