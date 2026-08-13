use makosh_identity_resolution_persistence::{
    IdentityResolutionPersistenceErrorV1, identity_resolution_storage_bundle_v1,
};
use makosh_identity_resolution_runtime::{
    IdentityResolutionManagedRuntimeErrorV1, IdentityResolutionManagedRuntimeV1,
    IdentityResolutionRuntimeAdmissionV1, identity_resolution_module_descriptor_v1,
    identity_resolution_settings_schema_bytes_v1,
};
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
        Some(v) if v == OsStr::new("serve-inherited") => serve(&mut args),
        Some(v) if v == OsStr::new("export-storage-bundle") => export(
            &mut args,
            identity_resolution_storage_bundle_v1().encode_to_vec(),
        ),
        Some(v) if v == OsStr::new("export-module-descriptor") => {
            let build = args
                .next()
                .and_then(|v| v.into_string().ok())
                .filter(|v| !v.is_empty())
                .ok_or_else(|| "Identity Resolution descriptor build id is required".to_owned())?;
            export(
                &mut args,
                identity_resolution_module_descriptor_v1(&build).encode_to_vec(),
            )
        }
        Some(v) if v == OsStr::new("export-settings-schema") => {
            export(&mut args, identity_resolution_settings_schema_bytes_v1())
        }
        _ => Err("Identity Resolution runtime command is unavailable".to_owned()),
    }
}
fn serve<I: Iterator<Item = OsString>>(args: &mut std::iter::Peekable<I>) -> Result<(), String> {
    let p = parse(args)?;
    let descriptor = read(&p.descriptor)?;
    let schema_bytes = read(&p.settings_schema)?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Identity Resolution settings schema is invalid".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&read(&p.settings_snapshot)?)
        .map_err(|_| "Identity Resolution settings snapshot is invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| "Identity Resolution settings snapshot is invalid".to_owned())?;
    let config =
        ManagedEngineRuntimeConfigurationV1::decode(read(&p.runtime_configuration)?.as_slice())
            .map_err(|_| "Identity Resolution runtime configuration is invalid".to_owned())?;
    validate_managed_engine_runtime_configuration(&config)
        .map_err(|_| "Identity Resolution runtime configuration is invalid".to_owned())?;
    if config.runtime_instance_id != p.runtime_instance_id
        || config.settings_revision != snapshot.revision
    {
        return Err("Identity Resolution runtime configuration is stale".to_owned());
    }
    let storage = config
        .storage
        .clone()
        .ok_or_else(|| "Identity Resolution storage is unavailable".to_owned())?;
    let admission = IdentityResolutionRuntimeAdmissionV1 {
        module_owner_id: config.logical_owner_id,
        logical_human_owner_id: config.logical_human_owner_id,
        registration_id: config.registration_id,
        runtime_instance_id: config.runtime_instance_id,
        runtime_generation: config.runtime_generation,
        grant_epoch: config.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Identity Resolution executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(IdentityResolutionManagedRuntimeV1::open(
            inherited()?,
            descriptor,
            schema_bytes,
            &admission,
            storage,
            &config.event_hub_endpoint,
            config.event_credential_revision,
        ))
        .map_err(runtime_error)?;
    let mut failures = 0_u8;
    loop {
        let now = now()?;
        match executor.block_on(runtime.service_once(now)) {
            Ok(_) => failures = 0,
            Err(IdentityResolutionManagedRuntimeErrorV1::ControlClosed) => return Ok(()),
            Err(IdentityResolutionManagedRuntimeErrorV1::EventUnavailable)
            | Err(IdentityResolutionManagedRuntimeErrorV1::Persistence(
                IdentityResolutionPersistenceErrorV1::StorageUnavailable,
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
            Err(e) => return Err(runtime_error(e)),
        }
    }
}
fn runtime_error(e: IdentityResolutionManagedRuntimeErrorV1) -> String {
    let code = match e {
        IdentityResolutionManagedRuntimeErrorV1::Admission => "admission",
        IdentityResolutionManagedRuntimeErrorV1::EventContract => "event_contract",
        IdentityResolutionManagedRuntimeErrorV1::EventUnavailable => "event_unavailable",
        IdentityResolutionManagedRuntimeErrorV1::Persistence(_) => "persistence",
        IdentityResolutionManagedRuntimeErrorV1::ControlClosed => "control_closed",
        IdentityResolutionManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Identity Resolution runtime failed: {code}")
}
fn now() -> Result<i64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|v| i64::try_from(v.as_millis()).ok())
        .ok_or_else(|| "Identity Resolution clock is invalid".to_owned())
}
fn export<I: Iterator<Item = OsString>>(args: &mut I, bytes: Vec<u8>) -> Result<(), String> {
    if args.next().is_some() {
        return Err("Identity Resolution runtime command is unavailable".to_owned());
    }
    std::io::Write::write_all(&mut std::io::stdout(), &bytes)
        .map_err(|_| "Identity Resolution artifact is unavailable".to_owned())
}
fn parse<I: Iterator<Item = OsString>>(args: &mut std::iter::Peekable<I>) -> Result<Paths, String> {
    let descriptor = required_path(args, "--descriptor-path")?;
    let settings_schema = required_path(args, "--settings-schema-path")?;
    let settings_snapshot = required_path(args, "--settings-snapshot-path")?;
    let runtime_configuration = required_path(args, "--runtime-configuration-path")?;
    let runtime_instance_id = required(args, "--runtime-instance-id")?;
    if args.next().is_some() {
        return Err("Identity Resolution runtime arguments are invalid".to_owned());
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
        return Err("Identity Resolution runtime arguments are invalid".to_owned());
    }
    args.next()
        .and_then(|v| v.into_string().ok())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "Identity Resolution runtime arguments are invalid".to_owned())
}
fn inherited() -> Result<UnixStream, String> {
    let fd = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if fd < 0 {
        return Err("Identity Resolution control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(fd) })
}
fn read(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Identity Resolution contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 512 * 1024 {
        return Err("Identity Resolution contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Identity Resolution contract is unavailable".to_owned())
}
