use std::ffi::{OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use makosh_communication_delivery_intent_persistence::{
    DeliveryIntentPersistenceErrorV1, schema::communication_delivery_intent_storage_bundle_v1,
};
use makosh_communication_delivery_intent_runtime::{
    admission::{
        communication_delivery_intent_module_descriptor_v1,
        communication_delivery_intent_settings_schema_bytes_v1,
    },
    runtime::{
        DeliveryIntentManagedRuntimeV1, DeliveryIntentRuntimeAdmissionV1,
        DeliveryIntentRuntimeErrorV1,
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
        Some(command) if command == OsStr::new("export-storage-bundle") => {
            export_storage_bundle(&mut arguments)
        }
        Some(command) if command == OsStr::new("export-module-descriptor") => {
            export_module_descriptor(&mut arguments)
        }
        Some(command) if command == OsStr::new("export-settings-schema") => {
            export_settings_schema(&mut arguments)
        }
        _ => Err("Communication Delivery Intent runtime command is unavailable".to_owned()),
    }
}

fn export_storage_bundle<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    require_no_arguments(arguments)?;
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &communication_delivery_intent_storage_bundle_v1().encode_to_vec(),
    )
    .map_err(|_| "Communication Delivery Intent storage bundle is unavailable".to_owned())
}

fn export_module_descriptor<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let build_id = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "Communication Delivery Intent descriptor build id is required".to_owned()
        })?;
    require_no_arguments(arguments)?;
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &communication_delivery_intent_module_descriptor_v1(&build_id).encode_to_vec(),
    )
    .map_err(|_| "Communication Delivery Intent descriptor is unavailable".to_owned())
}

fn export_settings_schema<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    require_no_arguments(arguments)?;
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &communication_delivery_intent_settings_schema_bytes_v1(),
    )
    .map_err(|_| "Communication Delivery Intent settings schema is unavailable".to_owned())
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let settings_schema = read_contract(&paths.settings_schema)?;
    decode_settings_schema_v1(&settings_schema)
        .map_err(|_| "Communication Delivery Intent settings schema is invalid".to_owned())?;
    let configuration = ManagedWorkflowRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Communication Delivery Intent runtime configuration is invalid".to_owned())?;
    validate_managed_workflow_runtime_configuration(&configuration)
        .map_err(|_| "Communication Delivery Intent runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Communication Delivery Intent runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Communication Delivery Intent storage is unavailable".to_owned())?;
    let admission = DeliveryIntentRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Communication Delivery Intent executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(DeliveryIntentManagedRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            settings_schema,
            &admission,
            storage,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        ))
        .map_err(runtime_error)?;
    let mut poll_backoff =
        ManagedRuntimePollBackoffV1::new(Duration::from_millis(25), Duration::from_millis(100))
            .map_err(|_| "Communication Delivery Intent polling bounds are invalid".to_owned())?;
    loop {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Communication Delivery Intent clock is invalid".to_owned())?
            .as_secs()
            .try_into()
            .map_err(|_| "Communication Delivery Intent clock is invalid".to_owned())?;
        let mut progressed =
            executor
                .block_on(runtime.pump_control_once(now))
                .map_err(|error| match error {
                    DeliveryIntentRuntimeErrorV1::Unavailable => {
                        "Communication Delivery Intent runtime failed: control_channel_unavailable"
                            .to_owned()
                    }
                    error => runtime_error(error),
                })?;
        progressed |= retry_delivery_step(executor.block_on(runtime.pump_client_realtime_once()))?;
        progressed |=
            retry_delivery_step(executor.block_on(runtime.process_next_provider_command_v1(now)))?;
        progressed |=
            retry_terminal_step(executor.block_on(runtime.consume_next_terminal_result_v1(now)))?;
        progressed |= retry_event_ingress_step(
            executor.block_on(runtime.consume_next_event_ingress_v1(now)),
        )?;
        progressed |=
            retry_delivery_step(executor.block_on(runtime.relay_ingress_result_once_v1(now)))?;
        progressed |=
            retry_delivery_step(executor.block_on(runtime.process_ingress_cleanup_once_v1(now)))?;
        let delay = poll_backoff.observe(progressed);
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
    }
}

fn retry_delivery_step(result: Result<bool, DeliveryIntentRuntimeErrorV1>) -> Result<bool, String> {
    match result {
        Ok(progressed) => Ok(progressed),
        Err(DeliveryIntentRuntimeErrorV1::Unavailable)
        | Err(DeliveryIntentRuntimeErrorV1::Persistence(
            DeliveryIntentPersistenceErrorV1::StorageUnavailable,
        )) => Ok(false),
        Err(error) => Err(runtime_error(error)),
    }
}

fn retry_terminal_step(result: Result<bool, DeliveryIntentRuntimeErrorV1>) -> Result<bool, String> {
    match result {
        Ok(progressed) => Ok(progressed),
        Err(error) if retryable_terminal_result_error(error) => Ok(false),
        Err(error) => Err(runtime_error(error)),
    }
}

fn retry_event_ingress_step(
    result: Result<bool, DeliveryIntentRuntimeErrorV1>,
) -> Result<bool, String> {
    match result {
        Ok(progressed) => Ok(progressed),
        Err(error) if retryable_event_ingress_error(error) => Ok(false),
        Err(error) => Err(runtime_error(error)),
    }
}

fn retryable_terminal_result_error(error: DeliveryIntentRuntimeErrorV1) -> bool {
    matches!(
        error,
        DeliveryIntentRuntimeErrorV1::Unavailable
            | DeliveryIntentRuntimeErrorV1::Persistence(
                DeliveryIntentPersistenceErrorV1::StorageUnavailable
                    | DeliveryIntentPersistenceErrorV1::Conflict
                    | DeliveryIntentPersistenceErrorV1::ClaimLost
            )
    )
}

fn retryable_event_ingress_error(error: DeliveryIntentRuntimeErrorV1) -> bool {
    matches!(
        error,
        DeliveryIntentRuntimeErrorV1::Unavailable
            | DeliveryIntentRuntimeErrorV1::RouteUnavailable
            | DeliveryIntentRuntimeErrorV1::Persistence(
                DeliveryIntentPersistenceErrorV1::StorageUnavailable
            )
    )
}

fn runtime_error(error: DeliveryIntentRuntimeErrorV1) -> String {
    let reason = match error {
        DeliveryIntentRuntimeErrorV1::Admission => "admission_rejected",
        DeliveryIntentRuntimeErrorV1::EventContract => "event_contract_rejected",
        DeliveryIntentRuntimeErrorV1::InvalidRequest => "client_request_rejected",
        DeliveryIntentRuntimeErrorV1::RouteUnavailable => "communications_route_unavailable",
        DeliveryIntentRuntimeErrorV1::Coordinator(
            makosh_communication_delivery_intent_runtime::coordinator::DeliveryIntentCoordinatorErrorV1::InvalidInput,
        ) => "coordinator_input_rejected",
        DeliveryIntentRuntimeErrorV1::Coordinator(
            makosh_communication_delivery_intent_runtime::coordinator::DeliveryIntentCoordinatorErrorV1::BlobUnavailable,
        ) => "blob_unavailable",
        DeliveryIntentRuntimeErrorV1::Persistence(
            DeliveryIntentPersistenceErrorV1::InvalidInput,
        ) => "persistence_input_rejected",
        DeliveryIntentRuntimeErrorV1::Persistence(
            DeliveryIntentPersistenceErrorV1::InvalidRow,
        ) => "persistence_row_rejected",
        DeliveryIntentRuntimeErrorV1::Persistence(
            DeliveryIntentPersistenceErrorV1::StorageUnavailable,
        ) => "storage_unavailable",
        DeliveryIntentRuntimeErrorV1::Persistence(DeliveryIntentPersistenceErrorV1::Conflict) => {
            "persistence_conflict"
        }
        DeliveryIntentRuntimeErrorV1::Persistence(DeliveryIntentPersistenceErrorV1::ClaimLost) => {
            "claim_lost"
        }
        DeliveryIntentRuntimeErrorV1::Unavailable => "event_hub_unavailable",
    };
    format!("Communication Delivery Intent runtime failed: {reason}")
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
        return Err("Communication Delivery Intent runtime arguments are invalid".to_owned());
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
        return Err("Communication Delivery Intent runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| "Communication Delivery Intent runtime arguments are invalid".to_owned())
}

fn require_no_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Communication Delivery Intent runtime command is unavailable".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Communication Delivery Intent control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Communication Delivery Intent contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Communication Delivery Intent contract is unavailable".to_owned());
    }
    std::fs::read(path)
        .map_err(|_| "Communication Delivery Intent contract is unavailable".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orphan_terminal_result_is_retriable_without_acknowledgement() {
        assert!(retryable_terminal_result_error(
            DeliveryIntentRuntimeErrorV1::Persistence(DeliveryIntentPersistenceErrorV1::Conflict,),
        ));
        assert!(!retryable_terminal_result_error(
            DeliveryIntentRuntimeErrorV1::Persistence(DeliveryIntentPersistenceErrorV1::InvalidRow,),
        ));
        assert!(!retryable_terminal_result_error(
            DeliveryIntentRuntimeErrorV1::EventContract,
        ));
    }

    #[test]
    fn idle_and_retryable_steps_do_not_claim_progress() {
        assert_eq!(retry_delivery_step(Ok(false)), Ok(false));
        assert_eq!(
            retry_delivery_step(Err(DeliveryIntentRuntimeErrorV1::Unavailable)),
            Ok(false)
        );
        assert!(retry_delivery_step(Err(DeliveryIntentRuntimeErrorV1::EventContract)).is_err());
    }
}
