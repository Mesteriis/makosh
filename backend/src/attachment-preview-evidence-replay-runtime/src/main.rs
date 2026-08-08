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

use makosh_attachment_preview_evidence_replay_persistence::attachment_preview_evidence_replay_storage_bundle_v1;
use makosh_attachment_preview_evidence_replay_runtime::{
    attachment_preview_evidence_replay_module_descriptor_v1,
    attachment_preview_evidence_replay_settings_schema_bytes_v1,
    managed_runtime::{
        AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1,
        AttachmentPreviewEvidenceReplayManagedRuntimeV1,
        AttachmentPreviewEvidenceReplayRuntimeAdmissionV1,
    },
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
    let Some(command) = arguments.next().and_then(|value| value.into_string().ok()) else {
        return Err("Replay workflow command is required".to_owned());
    };
    match command.as_str() {
        "export-storage-bundle" => export(
            &mut arguments,
            attachment_preview_evidence_replay_storage_bundle_v1().encode_to_vec(),
        ),
        "export-module-descriptor" => {
            let build_id = required_string(&mut arguments, "build id")?;
            export(
                &mut arguments,
                attachment_preview_evidence_replay_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        "export-settings-schema" => export(
            &mut arguments,
            attachment_preview_evidence_replay_settings_schema_bytes_v1(),
        ),
        "serve-inherited" => serve_inherited(&mut arguments),
        _ => Err("Replay workflow command is invalid".to_owned()),
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
        .map_err(|_| "Replay workflow settings schema is invalid".to_owned())?;
    let configuration = ManagedWorkflowRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Replay workflow runtime configuration is invalid".to_owned())?;
    validate_managed_workflow_runtime_configuration(&configuration)
        .map_err(|_| "Replay workflow runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Replay workflow runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Replay workflow storage is unavailable".to_owned())?;
    let admission = AttachmentPreviewEvidenceReplayRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Replay workflow executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(AttachmentPreviewEvidenceReplayManagedRuntimeV1::open(
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
        let now = clock()?;
        retry(executor.block_on(runtime.pump_control_once(now.0, now.1)))?;
        retry(executor.block_on(runtime.consume_communications_result_once(now.0)))?;
        retry(executor.block_on(runtime.consume_mail_result_once(now.0)))?;
        retry(executor.block_on(runtime.relay_commands_once(now.0)))?;
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn retry(
    result: Result<bool, AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1>,
) -> Result<(), String> {
    match result {
        Ok(_)
        | Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::EventUnavailable)
        | Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable)
        | Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Persistence(
            makosh_attachment_preview_evidence_replay_persistence::ReplayPersistenceErrorV1::StorageUnavailable,
        )) => Ok(()),
        Err(error) => Err(runtime_error(error)),
    }
}

fn runtime_error(error: AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1) -> String {
    let code = match error {
        AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission => "admission",
        AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::EventContract => "event_contract",
        AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::EventUnavailable => {
            "event_unavailable"
        }
        AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Persistence(_) => "persistence",
        AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Replay workflow runtime failed: {code}")
}

fn export<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Replay workflow output path is required".to_owned())?;
    if arguments.next().is_some() {
        return Err("Replay workflow arguments are invalid".to_owned());
    }
    fs::write(output, bytes).map_err(|_| "Replay workflow export failed".to_owned())
}

fn required_string<I>(arguments: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Replay workflow {name} is required"))
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let paths = InheritedPaths {
        descriptor: required_path(arguments, "--descriptor-path")?,
        settings_schema: required_path(arguments, "--settings-schema-path")?,
        runtime_configuration: required_path(arguments, "--runtime-configuration-path")?,
        runtime_instance_id: required_option(arguments, "--runtime-instance-id")?,
    };
    if arguments.next().is_some() {
        return Err("Replay workflow arguments are invalid".to_owned());
    }
    Ok(paths)
}

fn required_path<I>(arguments: &mut I, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    required_option(arguments, name).map(PathBuf::from)
}

fn required_option<I>(arguments: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Replay workflow arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Replay workflow arguments are invalid".to_owned())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Replay workflow control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Replay workflow contract path is invalid".to_owned());
    }
    let bytes = fs::read(path).map_err(|_| "Replay workflow contract is unavailable".to_owned())?;
    if bytes.is_empty() || bytes.len() > 2 * 1024 * 1024 {
        return Err("Replay workflow contract is invalid".to_owned());
    }
    Ok(bytes)
}

fn clock() -> Result<(i64, i32), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "Replay workflow clock is invalid".to_owned())?;
    Ok((
        now.as_secs()
            .try_into()
            .map_err(|_| "Replay workflow clock is invalid".to_owned())?,
        now.subsec_nanos()
            .try_into()
            .map_err(|_| "Replay workflow clock is invalid".to_owned())?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_exact_export_arguments() {
        let mut missing = Vec::<OsString>::new().into_iter();
        assert!(required_string(&mut missing, "build id").is_err());
        let mut exact = vec![OsString::from("build-1")].into_iter();
        assert_eq!(
            required_string(&mut exact, "build id"),
            Ok("build-1".to_owned())
        );
    }
}
