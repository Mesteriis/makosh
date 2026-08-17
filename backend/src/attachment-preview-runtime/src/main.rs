use std::{
    ffi::{OsStr, OsString},
    os::fd::{AsRawFd, FromRawFd},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    time::Duration,
};

use makosh_attachment_preview_api::ATTACHMENT_PREVIEW_OWNER_V1;
use makosh_attachment_preview_persistence::attachment_preview_storage_bundle_v1;
use makosh_attachment_preview_runtime::{
    AttachmentPreviewRendererRuntimeV1, attachment_preview_module_descriptor_v1,
    attachment_preview_settings_schema_bytes_v1,
    runtime::{
        AttachmentPreviewManagedRuntimeV1, AttachmentPreviewRuntimeAdmissionV1, JobTickV1,
        current_runtime_time_v1,
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

mod diagnostics;

use diagnostics::{
    AttachmentPreviewDiagnosticStageV1 as DiagnosticStage,
    emit_attachment_preview_diagnostic_v1 as diagnostic,
};

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    match arguments.next().as_deref() {
        Some(command) if command == OsStr::new("export-storage-bundle") => {
            no_trailing(&mut arguments)?;
            write_stdout(
                &attachment_preview_storage_bundle_v1().encode_to_vec(),
                "storage bundle",
            )
        }
        Some(command) if command == OsStr::new("export-module-descriptor") => {
            let build_id = arguments
                .next()
                .and_then(|value| value.into_string().ok())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Attachment Preview build id is required".to_owned())?;
            no_trailing(&mut arguments)?;
            write_stdout(
                &attachment_preview_module_descriptor_v1(&build_id).encode_to_vec(),
                "descriptor",
            )
        }
        Some(command) if command == OsStr::new("export-settings-schema") => {
            no_trailing(&mut arguments)?;
            write_stdout(
                &attachment_preview_settings_schema_bytes_v1(),
                "settings schema",
            )
        }
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(parse_paths(&mut arguments.peekable())?)
        }
        _ => Err("Attachment Preview runtime command is unavailable".to_owned()),
    }
}

fn serve_inherited(paths: InheritedPaths) -> Result<(), String> {
    let descriptor = read_contract(&paths.descriptor)?;
    let schema_bytes = read_contract(&paths.settings_schema)?;
    decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Attachment Preview settings schema is invalid".to_owned())?;
    let configuration = ManagedWorkflowRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Attachment Preview runtime configuration is invalid".to_owned())?;
    validate_managed_workflow_runtime_configuration(&configuration)
        .map_err(|_| "Attachment Preview runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Attachment Preview runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Attachment Preview storage configuration is missing".to_owned())?;
    let admission = AttachmentPreviewRuntimeAdmissionV1 {
        module_owner_id: ATTACHMENT_PREVIEW_OWNER_V1.to_owned(),
        logical_human_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Attachment Preview executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(AttachmentPreviewManagedRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            schema_bytes,
            &admission,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
            storage,
            AttachmentPreviewRendererRuntimeV1,
        ))
        .map_err(|error| {
            diagnostic(DiagnosticStage::Startup, error);
            "Attachment Preview runtime admission was rejected".to_owned()
        })?;
    let mut poll_backoff =
        ManagedRuntimePollBackoffV1::new(Duration::from_millis(250), Duration::from_millis(500))
            .map_err(|_| "Attachment Preview polling bounds are invalid".to_owned())?;
    loop {
        let (now_millis, nanos) = current_runtime_time_v1()
            .map_err(|_| "Attachment Preview clock is unavailable".to_owned())?;
        let mut progressed = false;
        for (stage, result) in [
            (
                DiagnosticStage::ClientDelivery,
                executor.block_on(runtime.pump_control_once(now_millis)),
            ),
            (
                DiagnosticStage::Consume,
                executor.block_on(runtime.consume_next(now_millis)),
            ),
        ] {
            match result {
                Ok(value) => progressed |= value,
                Err(error) => diagnostic(stage, error),
            }
        }
        match executor.block_on(runtime.materialize_pending_custody_requests(now_millis, nanos)) {
            Ok(count) => progressed |= count > 0,
            Err(error) => diagnostic(DiagnosticStage::CustodyMaterialize, error),
        }
        match executor.block_on(runtime.relay_custody_outbox(now_millis)) {
            Ok(count) => progressed |= count > 0,
            Err(error) => diagnostic(DiagnosticStage::CustodyOutbox, error),
        }
        match executor.block_on(runtime.process_next_job(now_millis)) {
            Ok(JobTickV1::Idle) => {}
            Ok(JobTickV1::Completed | JobTickV1::Rejected(_)) => progressed = true,
            Err(error) => diagnostic(DiagnosticStage::Render, error),
        }
        match executor.block_on(runtime.pump_client_realtime_once()) {
            Ok(value) => progressed |= value,
            Err(error) => diagnostic(DiagnosticStage::ClientRealtime, error),
        }
        let delay = poll_backoff.observe(progressed);
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
    }
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let runtime_configuration = required_path(arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_string(arguments, "--runtime-instance-id")?;
    if arguments.next().is_some() {
        return Err("Attachment Preview runtime arguments are invalid".to_owned());
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
        return Err("Attachment Preview runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Attachment Preview runtime arguments are invalid".to_owned())
}

fn no_trailing<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Attachment Preview runtime command is unavailable".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Attachment Preview control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Attachment Preview contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_BYTES
    {
        return Err("Attachment Preview contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Attachment Preview contract is unavailable".to_owned())
}

fn write_stdout(bytes: &[u8], artifact: &str) -> Result<(), String> {
    std::io::Write::write_all(&mut std::io::stdout(), bytes)
        .map_err(|_| format!("Attachment Preview {artifact} is unavailable"))
}
