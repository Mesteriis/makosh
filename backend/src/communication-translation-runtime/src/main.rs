use std::{
    ffi::{OsStr, OsString},
    fs,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::net::UnixStream,
    },
    path::{Path, PathBuf},
    time::Duration,
};

use makosh_communication_translation_persistence::{
    CommunicationTranslationPersistenceErrorV1, communication_translation_storage_bundle_v1,
};
use makosh_communication_translation_runtime::{
    CommunicationTranslationManagedRuntimeErrorV1, CommunicationTranslationManagedRuntimeV1,
    CommunicationTranslationRuntimeAdmissionV1, communication_translation_module_descriptor_v1,
    communication_translation_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::{
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
    let mut arguments = std::env::args_os().skip(1).peekable();
    let Some(command) = arguments.next() else {
        return Err("Communication Translation command is required".to_owned());
    };
    match command.to_str() {
        Some("export-storage-bundle") => export_bytes(
            &mut arguments,
            communication_translation_storage_bundle_v1().encode_to_vec(),
        ),
        Some("export-module-descriptor") => {
            let build_id = required_string(&mut arguments, "build id")?;
            export_bytes(
                &mut arguments,
                communication_translation_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some("export-settings-schema") => export_bytes(
            &mut arguments,
            communication_translation_settings_schema_bytes_v1(),
        ),
        Some("serve-inherited") => serve_inherited(&mut arguments),
        _ => Err("Communication Translation command is invalid".to_owned()),
    }
}

fn export_bytes<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Communication Translation output path is required".to_owned())?;
    require_no_arguments(arguments)?;
    fs::write(output, bytes).map_err(|_| "Communication Translation export failed".to_owned())
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let settings_schema = read_contract(&paths.settings_schema)?;
    decode_settings_schema_v1(&settings_schema)
        .map_err(|_| "Communication Translation settings schema is invalid".to_owned())?;
    let configuration = ManagedWorkflowRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Communication Translation runtime configuration is invalid".to_owned())?;
    validate_managed_workflow_runtime_configuration(&configuration)
        .map_err(|_| "Communication Translation runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Communication Translation runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Communication Translation storage is unavailable".to_owned())?;
    let admission = CommunicationTranslationRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Communication Translation executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(CommunicationTranslationManagedRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            settings_schema,
            &admission,
            storage,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        ))
        .map_err(runtime_error)?;
    loop {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Communication Translation clock is invalid".to_owned())?
            .as_millis()
            .try_into()
            .map_err(|_| "Communication Translation clock is invalid".to_owned())?;
        retry_runtime(executor.block_on(runtime.pump_control_once(now)))?;
        retry_runtime(executor.block_on(runtime.process_inference_once(now)))?;
        retry_runtime(executor.block_on(runtime.relay_source_prepare_outbox_once(now)))?;
        retry_runtime(executor.block_on(runtime.consume_source_prepared_once(now)))?;
        retry_runtime(executor.block_on(runtime.consume_source_rejected_once(now)))?;
        retry_runtime(executor.block_on(runtime.process_inference_once(now)))?;
        retry_runtime(executor.block_on(runtime.pump_client_realtime_once()))?;
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn retry_runtime(
    result: Result<bool, CommunicationTranslationManagedRuntimeErrorV1>,
) -> Result<(), String> {
    match result {
        Ok(_)
        | Err(CommunicationTranslationManagedRuntimeErrorV1::EventUnavailable)
        | Err(CommunicationTranslationManagedRuntimeErrorV1::Unavailable)
        | Err(CommunicationTranslationManagedRuntimeErrorV1::Persistence(
            CommunicationTranslationPersistenceErrorV1::StorageUnavailable,
        )) => Ok(()),
        Err(error) => Err(runtime_error(error)),
    }
}

fn runtime_error(error: CommunicationTranslationManagedRuntimeErrorV1) -> String {
    let code = match error {
        CommunicationTranslationManagedRuntimeErrorV1::Admission => "admission",
        CommunicationTranslationManagedRuntimeErrorV1::EventContract => "event_contract",
        CommunicationTranslationManagedRuntimeErrorV1::EventUnavailable => "event_unavailable",
        CommunicationTranslationManagedRuntimeErrorV1::InvalidTransition => "invalid_transition",
        CommunicationTranslationManagedRuntimeErrorV1::Persistence(_) => "persistence",
        CommunicationTranslationManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Communication Translation runtime failed: {code}")
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let paths = InheritedPaths {
        descriptor: required_path(arguments, "--descriptor-path")?,
        settings_schema: required_path(arguments, "--settings-schema-path")?,
        runtime_configuration: required_path(arguments, "--runtime-configuration-path")?,
        runtime_instance_id: required_string(arguments, "--runtime-instance-id")?,
    };
    require_no_arguments(arguments)?;
    Ok(paths)
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
        return Err("Communication Translation arguments are invalid".to_owned());
    }
    let value = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| format!("Communication Translation {name} is required"))?;
    if value.is_empty() {
        return Err(format!("Communication Translation {name} is invalid"));
    }
    Ok(value)
}

fn require_no_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Communication Translation arguments are invalid".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Communication Translation control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Communication Translation contract path is invalid".to_owned());
    }
    let bytes = fs::read(path)
        .map_err(|_| "Communication Translation contract is unavailable".to_owned())?;
    if bytes.is_empty() || bytes.len() > 2 * 1024 * 1024 {
        return Err("Communication Translation contract is invalid".to_owned());
    }
    Ok(bytes)
}
