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

use makosh_projects_persistence::{ProjectsPersistenceErrorV1, projects_storage_bundle_v1};
use makosh_projects_runtime::{
    ProjectsManagedRuntimeErrorV1, ProjectsManagedRuntimeV1, ProjectsRuntimeAdmissionV1,
    projects_module_descriptor_v1, projects_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::{
    managed_runtime_poll::ManagedRuntimePollBackoffV1,
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
        return Err("Projects command is required".to_owned());
    };
    match command.to_str() {
        Some("export-storage-bundle") => {
            export(&mut arguments, projects_storage_bundle_v1().encode_to_vec())
        }
        Some("export-module-descriptor") => {
            let build_id = required_string(&mut arguments, "build id")?;
            export(
                &mut arguments,
                projects_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some("export-settings-schema") => {
            export(&mut arguments, projects_settings_schema_bytes_v1())
        }
        Some("serve-inherited") => serve_inherited(&mut arguments),
        _ => Err("Projects command is invalid".to_owned()),
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
        .map_err(|_| "Projects settings schema is invalid".to_owned())?;
    let configuration = ManagedDomainRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Projects runtime configuration is invalid".to_owned())?;
    validate_managed_domain_runtime_configuration(&configuration)
        .map_err(|_| "Projects runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Projects runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Projects storage is unavailable".to_owned())?;
    let admission = ProjectsRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        logical_human_owner_id: configuration.logical_human_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Projects executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(ProjectsManagedRuntimeV1::open(
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
            .map_err(|_| "Projects runtime polling bounds are invalid".to_owned())?;
    loop {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Projects clock is invalid".to_owned())?
            .as_millis()
            .try_into()
            .map_err(|_| "Projects clock is invalid".to_owned())?;
        let mut progressed = retry_runtime(executor.block_on(runtime.pump_control_once(now)))?;
        for _ in 0..4 {
            let relayed = retry_runtime(executor.block_on(runtime.relay_outbox_once(now)))?;
            progressed |= relayed;
            if !relayed {
                break;
            }
        }
        let delay = poll_backoff.observe(progressed);
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
    }
}

fn retry_runtime(result: Result<bool, ProjectsManagedRuntimeErrorV1>) -> Result<bool, String> {
    match result {
        Ok(progressed) => Ok(progressed),
        Err(ProjectsManagedRuntimeErrorV1::EventUnavailable)
        | Err(ProjectsManagedRuntimeErrorV1::Persistence(
            ProjectsPersistenceErrorV1::StorageUnavailable,
        )) => Ok(false),
        Err(error) => Err(runtime_error(error)),
    }
}

fn runtime_error(error: ProjectsManagedRuntimeErrorV1) -> String {
    let code = match error {
        ProjectsManagedRuntimeErrorV1::Admission => "admission",
        ProjectsManagedRuntimeErrorV1::EventUnavailable => "event_unavailable",
        ProjectsManagedRuntimeErrorV1::Persistence(_) => "persistence",
        ProjectsManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Projects runtime failed: {code}")
}

fn export<I: Iterator<Item = OsString>>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String> {
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Projects output path is required".to_owned())?;
    require_no_arguments(arguments)?;
    fs::write(output, bytes).map_err(|_| "Projects export failed".to_owned())
}

fn parse_paths<I: Iterator<Item = OsString>>(
    arguments: &mut std::iter::Peekable<I>,
) -> Result<InheritedPaths, String> {
    let paths = InheritedPaths {
        descriptor: required_path(arguments, "--descriptor-path")?,
        settings_schema: required_path(arguments, "--settings-schema-path")?,
        runtime_configuration: required_path(arguments, "--runtime-configuration-path")?,
        runtime_instance_id: required_string(arguments, "--runtime-instance-id")?,
    };
    require_no_arguments(arguments)?;
    Ok(paths)
}

fn required_path<I: Iterator<Item = OsString>>(
    arguments: &mut I,
    name: &str,
) -> Result<PathBuf, String> {
    required_string(arguments, name).map(PathBuf::from)
}

fn required_string<I: Iterator<Item = OsString>>(
    arguments: &mut I,
    name: &str,
) -> Result<String, String> {
    if name.starts_with("--") && arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Projects arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Projects {name} is required"))
}

fn require_no_arguments<I: Iterator<Item = OsString>>(arguments: &mut I) -> Result<(), String> {
    if arguments.next().is_some() {
        return Err("Projects arguments are invalid".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Projects control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Projects contract path is invalid".to_owned());
    }
    let bytes = fs::read(path).map_err(|_| "Projects contract is unavailable".to_owned())?;
    if bytes.is_empty() || bytes.len() > 2 * 1024 * 1024 {
        return Err("Projects contract is invalid".to_owned());
    }
    Ok(bytes)
}
