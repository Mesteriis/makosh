use makosh_runtime_protocol::{
    v1::ManagedEngineRuntimeConfigurationV1,
    validation::{
        descriptor::{
            decode_settings_schema_v1, decode_settings_snapshot_v1,
            validate_settings_snapshot_against_schema_v1,
        },
        managed_engine_runtime::validate_managed_engine_runtime_configuration,
    },
};
use makosh_search_persistence::{SearchPersistenceErrorV1, search_storage_bundle_v1};
use makosh_search_runtime::{
    SearchManagedRuntimeErrorV1, SearchManagedRuntimeV1, SearchRuntimeAdmissionV1,
    search_module_descriptor_v1, search_settings_schema_bytes_v1,
};
use prost::Message;
use std::{
    ffi::{OsStr, OsString},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::net::UnixStream,
    },
    path::{Path, PathBuf},
};

struct Paths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    settings_snapshot: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut args = std::env::args_os();
    let _ = args.next();
    let command = args.next();
    let mut args = args.peekable();
    match command.as_deref() {
        Some(value) if value == OsStr::new("serve-inherited") => serve(&mut args),
        Some(value) if value == OsStr::new("export-storage-bundle") => {
            export(&mut args, search_storage_bundle_v1().encode_to_vec())
        }
        Some(value) if value == OsStr::new("export-module-descriptor") => {
            let build = args
                .next()
                .and_then(|value| value.into_string().ok())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Search descriptor build id is required".to_owned())?;
            export(
                &mut args,
                search_module_descriptor_v1(&build).encode_to_vec(),
            )
        }
        Some(value) if value == OsStr::new("export-settings-schema") => {
            export(&mut args, search_settings_schema_bytes_v1())
        }
        _ => Err("Search runtime command is unavailable".to_owned()),
    }
}

fn serve<I: Iterator<Item = OsString>>(args: &mut std::iter::Peekable<I>) -> Result<(), String> {
    let paths = parse(args)?;
    let descriptor = read(&paths.descriptor)?;
    let schema_bytes = read(&paths.settings_schema)?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Search settings schema is invalid".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&read(&paths.settings_snapshot)?)
        .map_err(|_| "Search settings snapshot is invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| "Search settings snapshot is invalid".to_owned())?;
    let configuration =
        ManagedEngineRuntimeConfigurationV1::decode(read(&paths.runtime_configuration)?.as_slice())
            .map_err(|_| "Search runtime configuration is invalid".to_owned())?;
    validate_managed_engine_runtime_configuration(&configuration)
        .map_err(|_| "Search runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id
        || configuration.settings_revision != snapshot.revision
    {
        return Err("Search runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Search storage is unavailable".to_owned())?;
    let admission = SearchRuntimeAdmissionV1 {
        module_owner_id: configuration.logical_owner_id,
        logical_human_owner_id: configuration.logical_human_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor =
        tokio::runtime::Runtime::new().map_err(|_| "Search executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(SearchManagedRuntimeV1::open(
            inherited()?,
            descriptor,
            schema_bytes,
            &admission,
            storage,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
            now()?,
        ))
        .map_err(runtime_error)?;
    let mut failures = 0_u8;
    loop {
        match executor.block_on(runtime.service_once(now()?)) {
            Ok(_) => failures = 0,
            Err(SearchManagedRuntimeErrorV1::ControlClosed) => return Ok(()),
            Err(SearchManagedRuntimeErrorV1::EventUnavailable)
            | Err(SearchManagedRuntimeErrorV1::Persistence(
                SearchPersistenceErrorV1::StorageUnavailable,
            )) => {
                let delay = std::time::Duration::from_millis(
                    25_u64.saturating_mul(1_u64 << u32::from(failures.min(7))),
                )
                .min(std::time::Duration::from_secs(2));
                failures = failures.saturating_add(1);
                if !executor
                    .block_on(runtime.wait_retry_delay(delay))
                    .map_err(runtime_error)?
                {
                    return Ok(());
                }
            }
            Err(error) => return Err(runtime_error(error)),
        }
    }
}

fn runtime_error(error: SearchManagedRuntimeErrorV1) -> String {
    let code = match error {
        SearchManagedRuntimeErrorV1::Admission => "admission",
        SearchManagedRuntimeErrorV1::EventContract => "event_contract",
        SearchManagedRuntimeErrorV1::EventUnavailable => "event_unavailable",
        SearchManagedRuntimeErrorV1::Persistence(_) => "persistence",
        SearchManagedRuntimeErrorV1::ControlClosed => "control_closed",
        SearchManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Search runtime failed: {code}")
}

fn now() -> Result<i64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|value| i64::try_from(value.as_millis()).ok())
        .ok_or_else(|| "Search clock is invalid".to_owned())
}

fn export<I: Iterator<Item = OsString>>(args: &mut I, bytes: Vec<u8>) -> Result<(), String> {
    if args.next().is_some() {
        return Err("Search runtime command is unavailable".to_owned());
    }
    std::io::Write::write_all(&mut std::io::stdout(), &bytes)
        .map_err(|_| "Search artifact is unavailable".to_owned())
}

fn parse<I: Iterator<Item = OsString>>(args: &mut std::iter::Peekable<I>) -> Result<Paths, String> {
    let descriptor = required_path(args, "--descriptor-path")?;
    let settings_schema = required_path(args, "--settings-schema-path")?;
    let settings_snapshot = required_path(args, "--settings-snapshot-path")?;
    let runtime_configuration = required_path(args, "--runtime-configuration-path")?;
    let runtime_instance_id = required(args, "--runtime-instance-id")?;
    if args.next().is_some() {
        return Err("Search runtime arguments are invalid".to_owned());
    }
    Ok(Paths {
        descriptor,
        settings_schema,
        settings_snapshot,
        runtime_configuration,
        runtime_instance_id,
    })
}
fn required_path<I: Iterator<Item = OsString>>(
    args: &mut I,
    name: &str,
) -> Result<PathBuf, String> {
    required(args, name).map(PathBuf::from)
}
fn required<I: Iterator<Item = OsString>>(args: &mut I, name: &str) -> Result<String, String> {
    if args.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Search runtime arguments are invalid".to_owned());
    }
    args.next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Search runtime arguments are invalid".to_owned())
}
fn inherited() -> Result<UnixStream, String> {
    let fd = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if fd < 0 {
        return Err("Search control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(fd) })
}
fn read(path: &Path) -> Result<Vec<u8>, String> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| "Search contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 512 * 1024 {
        return Err("Search contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Search contract is unavailable".to_owned())
}
