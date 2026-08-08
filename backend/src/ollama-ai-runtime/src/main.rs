use std::ffi::{OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use makosh_ollama_ai_api::{decode_ollama_ai_settings_v1, ollama_ai_settings_schema_bytes_v1};
use makosh_ollama_ai_persistence::schema::ollama_ai_storage_bundle_v1;
use makosh_ollama_ai_runtime::{
    OllamaAiManagedRuntimeErrorV1, OllamaAiManagedRuntimeV1, OllamaAiRuntimeAdmissionV1,
    ollama_ai_module_descriptor_v1,
};
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
    let command = arguments.next();
    let mut arguments = arguments.peekable();
    match command.as_deref() {
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(&mut arguments)
        }
        Some(command) if command == OsStr::new("export-storage-bundle") => export_bytes(
            &mut arguments,
            ollama_ai_storage_bundle_v1().encode_to_vec(),
        ),
        Some(command) if command == OsStr::new("export-module-descriptor") => {
            let build_id = arguments
                .next()
                .and_then(|value| value.into_string().ok())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Ollama AI descriptor build id is required".to_owned())?;
            export_bytes(
                &mut arguments,
                ollama_ai_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some(command) if command == OsStr::new("export-settings-schema") => {
            export_bytes(&mut arguments, ollama_ai_settings_schema_bytes_v1())
        }
        _ => Err("Ollama AI runtime command is unavailable".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let settings_schema = read_contract(&paths.settings_schema)?;
    let schema = decode_settings_schema_v1(&settings_schema)
        .map_err(|_| "Ollama AI settings schema is invalid".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&read_contract(&paths.settings_snapshot)?)
        .map_err(|_| "Ollama AI settings snapshot is invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| "Ollama AI settings snapshot is invalid".to_owned())?;
    let configuration = ManagedIntegrationRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Ollama AI runtime configuration is invalid".to_owned())?;
    validate_managed_integration_runtime_configuration(&configuration)
        .map_err(|_| "Ollama AI runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Ollama AI runtime configuration is stale".to_owned());
    }
    let settings =
        decode_ollama_ai_settings_v1(&snapshot, &configuration.configuration_instance_id)
            .map_err(|_| "Ollama AI effective settings are invalid".to_owned())?;
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Ollama AI storage is unavailable".to_owned())?;
    let admission = OllamaAiRuntimeAdmissionV1 {
        module_owner_id: configuration.logical_owner_id,
        logical_human_owner_id: configuration.logical_human_owner_id,
        configuration_instance_id: configuration.configuration_instance_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Ollama AI executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(OllamaAiManagedRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            settings_schema,
            &admission,
            settings,
            storage,
        ))
        .map_err(runtime_error)?;
    let mut interval =
        executor.block_on(async { tokio::time::interval(Duration::from_millis(50)) });
    loop {
        executor.block_on(interval.tick());
        executor
            .block_on(runtime.pump_control_once())
            .map_err(runtime_error)?;
    }
}

fn runtime_error(error: OllamaAiManagedRuntimeErrorV1) -> String {
    let reason = match error {
        OllamaAiManagedRuntimeErrorV1::Admission => "admission_rejected",
        OllamaAiManagedRuntimeErrorV1::Persistence(_) => "storage_unavailable",
        OllamaAiManagedRuntimeErrorV1::Unavailable => "runtime_unavailable",
    };
    format!("Ollama AI runtime failed: {reason}")
}

fn export_bytes<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    require_no_arguments(arguments)?;
    std::io::Write::write_all(&mut std::io::stdout(), &bytes)
        .map_err(|_| "Ollama AI runtime artifact is unavailable".to_owned())
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
    require_no_arguments(arguments)?;
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
        return Err("Ollama AI runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Ollama AI runtime arguments are invalid".to_owned())
}

fn require_no_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Ollama AI runtime command is unavailable".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Ollama AI control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Ollama AI contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Ollama AI contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Ollama AI contract is unavailable".to_owned())
}
