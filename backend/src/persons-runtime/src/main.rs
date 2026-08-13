use std::{
    ffi::{OsStr, OsString},
    fs,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::net::UnixStream,
    },
    path::{Path, PathBuf},
};

use makosh_persons_persistence::{PersonsPersistenceErrorV1, persons_storage_bundle_v1};
use makosh_persons_runtime::{
    PersonsManagedRuntimeErrorV1, PersonsManagedRuntimeV1, PersonsRuntimeAdmissionV1,
    persons_module_descriptor_v1, persons_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::{
    v1::ManagedDomainRuntimeConfigurationV1,
    validation::{
        descriptor::decode_settings_schema_v1,
        managed_domain_runtime::validate_managed_domain_runtime_configuration,
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
        return Err("Persons command is required".to_owned());
    };
    match command.to_str() {
        Some("export-storage-bundle") => {
            export(&mut arguments, persons_storage_bundle_v1().encode_to_vec())
        }
        Some("export-module-descriptor") => {
            let build_id = required_string(&mut arguments, "build id")?;
            export(
                &mut arguments,
                persons_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some("export-settings-schema") => {
            export(&mut arguments, persons_settings_schema_bytes_v1())
        }
        Some("serve-inherited") => serve_inherited(&mut arguments),
        _ => Err("Persons command is invalid".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let settings_schema = read_contract(&paths.settings_schema)?;
    decode_settings_schema_v1(&settings_schema)
        .map_err(|_| "Persons settings schema is invalid".to_owned())?;
    let configuration = ManagedDomainRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Persons runtime configuration is invalid".to_owned())?;
    validate_managed_domain_runtime_configuration(&configuration)
        .map_err(|_| "Persons runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Persons runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Persons storage is unavailable".to_owned())?;
    let admission = PersonsRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        logical_human_owner_id: configuration.logical_human_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor =
        tokio::runtime::Runtime::new().map_err(|_| "Persons executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(PersonsManagedRuntimeV1::open(
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
            .map_err(|_| "Persons clock is invalid".to_owned())?
            .as_millis()
            .try_into()
            .map_err(|_| "Persons clock is invalid".to_owned())?;
        if !retry_runtime(executor.block_on(runtime.service_once(now)))? {
            return Ok(());
        }
    }
}

fn retry_runtime(result: Result<bool, PersonsManagedRuntimeErrorV1>) -> Result<bool, String> {
    match result {
        Ok(_)
        | Err(PersonsManagedRuntimeErrorV1::EventUnavailable)
        | Err(PersonsManagedRuntimeErrorV1::Persistence(
            PersonsPersistenceErrorV1::StorageUnavailable,
        )) => Ok(true),
        Err(PersonsManagedRuntimeErrorV1::ControlClosed) => Ok(false),
        Err(error) => Err(runtime_error(error)),
    }
}

fn runtime_error(error: PersonsManagedRuntimeErrorV1) -> String {
    let code = match error {
        PersonsManagedRuntimeErrorV1::Admission => "admission",
        PersonsManagedRuntimeErrorV1::EventContract => "event_contract",
        PersonsManagedRuntimeErrorV1::EventUnavailable => "event_unavailable",
        PersonsManagedRuntimeErrorV1::Persistence(_) => "persistence",
        PersonsManagedRuntimeErrorV1::ControlClosed => "control_closed",
        PersonsManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Persons runtime failed: {code}")
}

fn export<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Persons output path is required".to_owned())?;
    require_no_arguments(arguments)?;
    fs::write(output, bytes).map_err(|_| "Persons export failed".to_owned())
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
    if name.starts_with("--") && arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Persons arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Persons {name} is required"))
}

fn require_no_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Persons arguments are invalid".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Persons control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Persons contract path is invalid".to_owned());
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "Persons contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 2 * 1024 * 1024
    {
        return Err("Persons contract is invalid".to_owned());
    }
    let bytes = fs::read(path).map_err(|_| "Persons contract is unavailable".to_owned())?;
    if bytes.is_empty() {
        return Err("Persons contract is invalid".to_owned());
    }
    Ok(bytes)
}
