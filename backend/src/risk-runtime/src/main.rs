use makosh_risk_persistence::{RiskPersistenceErrorV1, risk_storage_bundle_v1};
use makosh_risk_runtime::{
    RiskManagedRuntimeErrorV1, RiskManagedRuntimeV1, RiskRuntimeAdmissionV1,
    risk_module_descriptor_v1, risk_settings_schema_bytes_v1,
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
        Some(v) if v == OsStr::new("export-storage-bundle") => {
            export(&mut args, risk_storage_bundle_v1().encode_to_vec())
        }
        Some(v) if v == OsStr::new("export-module-descriptor") => {
            let build = args
                .next()
                .and_then(|v| v.into_string().ok())
                .filter(|v| !v.is_empty())
                .ok_or_else(|| "Risk descriptor build id is required".to_owned())?;
            export(&mut args, risk_module_descriptor_v1(&build).encode_to_vec())
        }
        Some(v) if v == OsStr::new("export-settings-schema") => {
            export(&mut args, risk_settings_schema_bytes_v1())
        }
        _ => Err("Risk runtime command is unavailable".into()),
    }
}
fn serve<I: Iterator<Item = OsString>>(args: &mut std::iter::Peekable<I>) -> Result<(), String> {
    let paths = parse(args)?;
    let descriptor = read(&paths.descriptor)?;
    let schema_bytes = read(&paths.settings_schema)?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Risk settings schema is invalid".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&read(&paths.settings_snapshot)?)
        .map_err(|_| "Risk settings snapshot is invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| "Risk settings snapshot is invalid".to_owned())?;
    let config =
        ManagedEngineRuntimeConfigurationV1::decode(read(&paths.runtime_configuration)?.as_slice())
            .map_err(|_| "Risk runtime configuration is invalid".to_owned())?;
    validate_managed_engine_runtime_configuration(&config)
        .map_err(|_| "Risk runtime configuration is invalid".to_owned())?;
    if config.runtime_instance_id != paths.runtime_instance_id
        || config.settings_revision != snapshot.revision
    {
        return Err("Risk runtime configuration is stale".into());
    }
    let storage = config
        .storage
        .clone()
        .ok_or_else(|| "Risk storage is unavailable".to_owned())?;
    let admission = RiskRuntimeAdmissionV1 {
        module_owner_id: config.logical_owner_id,
        logical_human_owner_id: config.logical_human_owner_id,
        registration_id: config.registration_id,
        runtime_instance_id: config.runtime_instance_id,
        runtime_generation: config.runtime_generation,
        grant_epoch: config.grant_epoch,
    };
    let executor =
        tokio::runtime::Runtime::new().map_err(|_| "Risk executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(RiskManagedRuntimeV1::open(
            inherited()?,
            descriptor,
            schema_bytes,
            &admission,
            storage,
            &config.event_hub_endpoint,
            config.event_credential_revision,
            now()?,
        ))
        .map_err(runtime_error)?;
    let mut failures = 0_u8;
    loop {
        match executor.block_on(runtime.service_once(now()?)) {
            Ok(_) => failures = 0,
            Err(RiskManagedRuntimeErrorV1::ControlClosed) => return Ok(()),
            Err(RiskManagedRuntimeErrorV1::EventUnavailable)
            | Err(RiskManagedRuntimeErrorV1::Persistence(
                RiskPersistenceErrorV1::StorageUnavailable,
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
fn runtime_error(error: RiskManagedRuntimeErrorV1) -> String {
    let code = match error {
        RiskManagedRuntimeErrorV1::Admission => "admission",
        RiskManagedRuntimeErrorV1::EventContract => "event_contract",
        RiskManagedRuntimeErrorV1::EventUnavailable => "event_unavailable",
        RiskManagedRuntimeErrorV1::Persistence(_) => "persistence",
        RiskManagedRuntimeErrorV1::ControlClosed => "control_closed",
        RiskManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Risk runtime failed: {code}")
}
fn now() -> Result<i64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|v| i64::try_from(v.as_millis()).ok())
        .ok_or_else(|| "Risk clock is invalid".into())
}
fn export<I: Iterator<Item = OsString>>(args: &mut I, bytes: Vec<u8>) -> Result<(), String> {
    if args.next().is_some() {
        return Err("Risk runtime command is unavailable".into());
    }
    std::io::Write::write_all(&mut std::io::stdout(), &bytes)
        .map_err(|_| "Risk artifact is unavailable".into())
}
fn parse<I: Iterator<Item = OsString>>(args: &mut std::iter::Peekable<I>) -> Result<Paths, String> {
    let descriptor = required_path(args, "--descriptor-path")?;
    let settings_schema = required_path(args, "--settings-schema-path")?;
    let settings_snapshot = required_path(args, "--settings-snapshot-path")?;
    let runtime_configuration = required_path(args, "--runtime-configuration-path")?;
    let runtime_instance_id = required(args, "--runtime-instance-id")?;
    if args.next().is_some() {
        return Err("Risk runtime arguments are invalid".into());
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
        return Err("Risk runtime arguments are invalid".into());
    }
    args.next()
        .and_then(|v| v.into_string().ok())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "Risk runtime arguments are invalid".into())
}
fn inherited() -> Result<UnixStream, String> {
    let fd = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if fd < 0 {
        return Err("Risk control channel is unavailable".into());
    }
    Ok(unsafe { UnixStream::from_raw_fd(fd) })
}
fn read(path: &Path) -> Result<Vec<u8>, String> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| "Risk contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 512 * 1024 {
        return Err("Risk contract is unavailable".into());
    }
    std::fs::read(path).map_err(|_| "Risk contract is unavailable".into())
}
