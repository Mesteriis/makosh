//! Kernel-inherited process root for the Attachment Security engine.

use std::{
    ffi::{OsStr, OsString},
    os::fd::{AsRawFd, FromRawFd},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    time::Duration,
};

use makosh_attachment_security_persistence::attachment_security_storage_bundle_v1;
use makosh_attachment_security_runtime::{
    admission::attachment_security_module_descriptor_v1,
    runtime::{
        AttachmentSecurityRuntimeAdmissionV1, AttachmentSecurityRuntimeV1, current_runtime_time_v1,
    },
    settings::{
        attachment_security_settings_schema_bytes_v1, decode_attachment_security_settings_v1,
    },
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
        Some(command) if command == OsStr::new("export-storage-bundle") => {
            export_storage_bundle(&mut arguments.peekable())
        }
        Some(command) if command == OsStr::new("export-module-descriptor") => {
            export_module_descriptor(&mut arguments.peekable())
        }
        Some(command) if command == OsStr::new("export-settings-schema") => {
            export_settings_schema(&mut arguments.peekable())
        }
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(&mut arguments.peekable())
        }
        _ => Err("Attachment Security runtime command is unavailable".to_owned()),
    }
}

fn export_storage_bundle<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    reject_trailing_arguments(arguments)?;
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &attachment_security_storage_bundle_v1().encode_to_vec(),
    )
    .map_err(|_| "Attachment Security storage bundle is unavailable".to_owned())
}

fn export_module_descriptor<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let build_id = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Attachment Security descriptor build id is required".to_owned())?;
    reject_trailing_arguments(arguments)?;
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &attachment_security_module_descriptor_v1(&build_id).encode_to_vec(),
    )
    .map_err(|_| "Attachment Security module descriptor is unavailable".to_owned())
}

fn export_settings_schema<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    reject_trailing_arguments(arguments)?;
    std::io::Write::write_all(
        &mut std::io::stdout(),
        &attachment_security_settings_schema_bytes_v1(),
    )
    .map_err(|_| "Attachment Security settings schema is unavailable".to_owned())
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let schema_bytes = read_contract(&paths.settings_schema)?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Attachment Security settings schema is invalid".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&read_contract(&paths.settings_snapshot)?)
        .map_err(|_| "Attachment Security settings snapshot is invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| "Attachment Security settings snapshot is invalid".to_owned())?;
    let configuration = ManagedEngineRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Attachment Security runtime configuration is invalid".to_owned())?;
    validate_managed_engine_runtime_configuration(&configuration)
        .map_err(|_| "Attachment Security runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Attachment Security runtime configuration is stale".to_owned());
    }
    let settings = decode_attachment_security_settings_v1(
        &snapshot,
        &configuration.registration_id,
        configuration.settings_revision,
    )
    .map_err(|_| "Attachment Security settings snapshot is invalid".to_owned())?;
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Attachment Security runtime configuration is invalid".to_owned())?;
    let admission = AttachmentSecurityRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Attachment Security runtime executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(AttachmentSecurityRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            schema_bytes,
            &admission,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
            storage,
            settings,
        ))
        .map_err(|error| {
            developer_diagnostic("startup", error);
            "Attachment Security runtime admission was rejected".to_owned()
        })?;
    let mut interval =
        executor.block_on(async { tokio::time::interval(Duration::from_millis(250)) });
    loop {
        executor.block_on(interval.tick());
        let (seconds, nanos) = current_runtime_time_v1()
            .map_err(|_| "Attachment Security runtime clock is unavailable".to_owned())?;
        if let Err(error) = executor.block_on(runtime.consume_next(seconds)) {
            developer_diagnostic("consume", error);
        }
        match executor.block_on(runtime.process_next_scan_job(seconds, nanos)) {
            Ok(
                makosh_attachment_security_runtime::runtime::AttachmentSecurityScanTickV1::RetryScheduled(
                    error,
                ),
            ) => developer_scan_diagnostic("retry", error),
            Ok(
                makosh_attachment_security_runtime::runtime::AttachmentSecurityScanTickV1::Exhausted(
                    error,
                ),
            ) => developer_scan_diagnostic("exhausted", error),
            Ok(_) => {}
            Err(error) => developer_diagnostic("scan", error),
        }
        match executor.block_on(runtime.process_next_archive_delegation(seconds, nanos)) {
            Ok(
                makosh_attachment_security_runtime::runtime::AttachmentSecurityArchiveDelegationTickV1::RetryScheduled,
            ) => developer_diagnostic(
                "archive-delegation-retry",
                makosh_attachment_security_runtime::runtime::AttachmentSecurityRuntimeErrorV1::Unavailable,
            ),
            Ok(_) => {}
            Err(error) => developer_diagnostic("archive-delegation", error),
        }
        match executor.block_on(runtime.process_next_text_delegation(seconds, nanos)) {
            Ok(
                makosh_attachment_security_runtime::runtime::AttachmentSecurityTextDelegationTickV1::RetryScheduled,
            ) => developer_diagnostic(
                "text-delegation-retry",
                makosh_attachment_security_runtime::runtime::AttachmentSecurityRuntimeErrorV1::Unavailable,
            ),
            Ok(_) => {}
            Err(error) => developer_diagnostic("text-delegation", error),
        }
        match executor.block_on(runtime.process_next_preview_delegation(seconds, nanos)) {
            Ok(
                makosh_attachment_security_runtime::runtime::AttachmentSecurityPreviewDelegationTickV1::RetryScheduled,
            ) => developer_diagnostic(
                "preview-delegation-retry",
                makosh_attachment_security_runtime::runtime::AttachmentSecurityRuntimeErrorV1::Unavailable,
            ),
            Ok(_) => {}
            Err(error) => developer_diagnostic("preview-delegation", error),
        }
        if let Err(error) = executor.block_on(runtime.relay_verdict_outbox(seconds)) {
            developer_diagnostic("outbox", error);
        }
        if let Err(error) = executor.block_on(runtime.relay_archive_delegation_outbox(seconds)) {
            developer_diagnostic("archive-delegation-outbox", error);
        }
        if let Err(error) = executor.block_on(runtime.relay_text_delegation_outbox(seconds)) {
            developer_diagnostic("text-delegation-outbox", error);
        }
        if let Err(error) = executor.block_on(runtime.relay_preview_delegation_outbox(seconds)) {
            developer_diagnostic("preview-delegation-outbox", error);
        }
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
        return Err("Attachment Security runtime arguments are invalid".to_owned());
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
        return Err("Attachment Security runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| "Attachment Security runtime arguments are invalid".to_owned())
}

fn reject_trailing_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Attachment Security runtime command is unavailable".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Attachment Security inherited control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Attachment Security runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Attachment Security runtime contract is unavailable".to_owned());
    }
    std::fs::read(path)
        .map_err(|_| "Attachment Security runtime contract is unavailable".to_owned())
}

fn developer_diagnostic(
    stage: &str,
    error: makosh_attachment_security_runtime::runtime::AttachmentSecurityRuntimeErrorV1,
) {
    use std::sync::atomic::{AtomicBool, Ordering};

    static EMPTY_CONSUME_REPORTED: AtomicBool = AtomicBool::new(false);
    if stage == "consume"
        && error
            == makosh_attachment_security_runtime::runtime::AttachmentSecurityRuntimeErrorV1::Unavailable
        && EMPTY_CONSUME_REPORTED.swap(true, Ordering::Relaxed)
    {
        return;
    }
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_attachment_security_runtime_error stage={stage} error={error:?}");
    }
}

fn developer_scan_diagnostic(
    stage: &str,
    error: makosh_attachment_security_runtime::AttachmentSecurityScanAdapterErrorV1,
) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_attachment_security_scan_error stage={stage} error={error:?}");
    }
}
