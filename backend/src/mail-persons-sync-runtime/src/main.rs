use std::{
    ffi::{OsStr, OsString},
    fs,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::net::UnixStream,
    },
    path::{Path, PathBuf},
};

use makosh_mail_persons_sync_persistence::{
    MailPersonsSyncPersistenceErrorV1, mail_persons_sync_storage_bundle_v1,
};
use makosh_mail_persons_sync_runtime::{
    MailPersonsSyncManagedRuntimeErrorV1, MailPersonsSyncManagedRuntimeV1,
    MailPersonsSyncRuntimeAdmissionV1, mail_persons_sync_module_descriptor_v1,
    mail_persons_sync_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::{
    v1::ManagedWorkflowRuntimeConfigurationV1,
    validation::{
        descriptor::{
            decode_settings_schema_v1, decode_settings_snapshot_v1,
            validate_settings_snapshot_against_schema_v1,
        },
        managed_workflow_runtime::validate_managed_workflow_runtime_configuration,
    },
};
use prost::Message;

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    settings_snapshot: Option<PathBuf>,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os().skip(1).peekable();
    let Some(command) = arguments.next() else {
        return Err("Mail Persons Sync command is required".to_owned());
    };
    match command.to_str() {
        Some("export-storage-bundle") => export(
            &mut arguments,
            mail_persons_sync_storage_bundle_v1().encode_to_vec(),
        ),
        Some("export-module-descriptor") => {
            let build_id = required_string(&mut arguments, "build id")?;
            export(
                &mut arguments,
                mail_persons_sync_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some("export-settings-schema") => {
            export(&mut arguments, mail_persons_sync_settings_schema_bytes_v1())
        }
        Some("serve-inherited") => serve_inherited(&mut arguments),
        _ => Err("Mail Persons Sync command is invalid".to_owned()),
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
        .map_err(|_| "Mail Persons Sync settings schema is invalid".to_owned())?;
    let configuration = ManagedWorkflowRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Mail Persons Sync runtime configuration is invalid".to_owned())?;
    validate_managed_workflow_runtime_configuration(&configuration)
        .map_err(|_| "Mail Persons Sync runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Mail Persons Sync runtime configuration is stale".to_owned());
    }
    validate_selected_settings(&schema, paths.settings_snapshot.as_deref(), &configuration)?;
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Mail Persons Sync storage is unavailable".to_owned())?;
    let admission = MailPersonsSyncRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Mail Persons Sync executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(MailPersonsSyncManagedRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            settings_schema,
            &admission,
            storage,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        ))
        .map_err(runtime_error)?;
    let mut retry_backoff = RuntimeRetryBackoffV1::default();
    loop {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Mail Persons Sync clock is invalid".to_owned())?
            .as_millis()
            .try_into()
            .map_err(|_| "Mail Persons Sync clock is invalid".to_owned())?;
        match retry_action_v1(
            &mut retry_backoff,
            executor.block_on(runtime.service_once(now)),
        )? {
            RetryActionV1::Continue => {}
            RetryActionV1::Stop => return Ok(()),
            RetryActionV1::Wait(delay) => {
                if !executor
                    .block_on(runtime.wait_retry_delay(delay))
                    .map_err(runtime_error)?
                {
                    return Ok(());
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RetryActionV1 {
    Continue,
    Wait(std::time::Duration),
    Stop,
}

#[derive(Default)]
struct RuntimeRetryBackoffV1 {
    consecutive_failures: u8,
}

fn retry_action_v1(
    backoff: &mut RuntimeRetryBackoffV1,
    result: Result<bool, MailPersonsSyncManagedRuntimeErrorV1>,
) -> Result<RetryActionV1, String> {
    match result {
        Ok(_) => {
            backoff.consecutive_failures = 0;
            Ok(RetryActionV1::Continue)
        }
        Err(MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable)
        | Err(MailPersonsSyncManagedRuntimeErrorV1::Persistence(
            MailPersonsSyncPersistenceErrorV1::StorageUnavailable,
        )) => {
            let exponent = u32::from(backoff.consecutive_failures.min(7));
            let delay = std::time::Duration::from_millis(25_u64.saturating_mul(1_u64 << exponent))
                .min(std::time::Duration::from_secs(2));
            backoff.consecutive_failures = backoff.consecutive_failures.saturating_add(1);
            Ok(RetryActionV1::Wait(delay))
        }
        Err(MailPersonsSyncManagedRuntimeErrorV1::ControlClosed) => Ok(RetryActionV1::Stop),
        Err(error) => Err(runtime_error(error)),
    }
}

fn runtime_error(error: MailPersonsSyncManagedRuntimeErrorV1) -> String {
    let code = match error {
        MailPersonsSyncManagedRuntimeErrorV1::Admission => "admission",
        MailPersonsSyncManagedRuntimeErrorV1::EventContract => "event_contract",
        MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable => "event_unavailable",
        MailPersonsSyncManagedRuntimeErrorV1::Persistence(_) => "persistence",
        MailPersonsSyncManagedRuntimeErrorV1::ControlClosed => "control_closed",
        MailPersonsSyncManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Mail Persons Sync runtime failed: {code}")
}

fn export<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Mail Persons Sync output path is required".to_owned())?;
    require_no_arguments(arguments)?;
    fs::write(output, bytes).map_err(|_| "Mail Persons Sync export failed".to_owned())
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let settings_snapshot = if arguments.peek().map(OsString::as_os_str)
        == Some(OsStr::new("--settings-snapshot-path"))
    {
        Some(required_path(arguments, "--settings-snapshot-path")?)
    } else {
        None
    };
    let paths = InheritedPaths {
        descriptor,
        settings_schema,
        settings_snapshot,
        runtime_configuration: required_path(arguments, "--runtime-configuration-path")?,
        runtime_instance_id: required_string(arguments, "--runtime-instance-id")?,
    };
    require_no_arguments(arguments)?;
    Ok(paths)
}

fn validate_selected_settings(
    schema: &makosh_runtime_protocol::v1::SettingsSchemaV1,
    snapshot_path: Option<&Path>,
    configuration: &ManagedWorkflowRuntimeConfigurationV1,
) -> Result<(), String> {
    let Some(snapshot_path) = snapshot_path else {
        if schema.definitions.is_empty()
            && configuration.configuration_instance_id.is_empty()
            && configuration.settings_revision == 0
            && configuration.configuration_instances.is_empty()
        {
            return Ok(());
        }
        return Err("Mail Persons Sync selected settings are unavailable".to_owned());
    };
    let selected_snapshot_bytes = read_contract(snapshot_path)?;
    let selected_snapshot = decode_settings_snapshot_v1(&selected_snapshot_bytes)
        .map_err(|_| "Mail Persons Sync selected settings are invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(schema, &selected_snapshot)
        .map_err(|_| "Mail Persons Sync selected settings are invalid".to_owned())?;
    let selected = configuration
        .configuration_instances
        .iter()
        .find(|instance| {
            instance.configuration_instance_id == configuration.configuration_instance_id
        })
        .ok_or_else(|| "Mail Persons Sync settings catalog is invalid".to_owned())?;
    if selected.settings_snapshot_bytes != selected_snapshot_bytes
        || selected_snapshot.target_id != configuration.configuration_instance_id
        || selected_snapshot.revision != configuration.settings_revision
    {
        return Err("Mail Persons Sync settings catalog is stale".to_owned());
    }
    Ok(())
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
        return Err("Mail Persons Sync arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Mail Persons Sync {name} is required"))
}

fn require_no_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        Err("Mail Persons Sync arguments are invalid".to_owned())
    } else {
        Ok(())
    }
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Mail Persons Sync control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Mail Persons Sync contract path is invalid".to_owned());
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| "Mail Persons Sync contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 2 * 1024 * 1024
    {
        return Err("Mail Persons Sync contract is invalid".to_owned());
    }
    let bytes =
        fs::read(path).map_err(|_| "Mail Persons Sync contract is unavailable".to_owned())?;
    if bytes.is_empty() {
        return Err("Mail Persons Sync contract is invalid".to_owned());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherited_paths_allow_the_empty_settings_schema_without_a_snapshot() {
        let mut arguments = [
            "--descriptor-path",
            "/tmp/descriptor.pb",
            "--settings-schema-path",
            "/tmp/settings.pb",
            "--runtime-configuration-path",
            "/tmp/runtime.pb",
            "--runtime-instance-id",
            "runtime-1",
        ]
        .into_iter()
        .map(OsString::from)
        .peekable();

        let paths = parse_paths(&mut arguments).expect("paths without a settings snapshot");

        assert!(paths.settings_snapshot.is_none());
        assert_eq!(paths.runtime_instance_id, "runtime-1");
    }

    #[test]
    fn empty_settings_schema_requires_an_empty_workflow_selection() {
        let schema = makosh_runtime_protocol::v1::SettingsSchemaV1 {
            major: 1,
            revision: 1,
            definitions: Vec::new(),
        };
        let configuration = ManagedWorkflowRuntimeConfigurationV1::default();

        assert!(validate_selected_settings(&schema, None, &configuration).is_ok());

        let selected = ManagedWorkflowRuntimeConfigurationV1 {
            configuration_instance_id: "unexpected".to_owned(),
            ..Default::default()
        };
        assert!(validate_selected_settings(&schema, None, &selected).is_err());
    }

    #[test]
    fn transient_retry_is_exponential_bounded_and_resets_after_progress() {
        let mut backoff = RuntimeRetryBackoffV1::default();
        let transient = Err(MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable);
        assert_eq!(
            retry_action_v1(&mut backoff, transient),
            Ok(RetryActionV1::Wait(std::time::Duration::from_millis(25)))
        );
        assert_eq!(
            retry_action_v1(&mut backoff, transient),
            Ok(RetryActionV1::Wait(std::time::Duration::from_millis(50)))
        );
        for _ in 0..16 {
            let _ = retry_action_v1(&mut backoff, transient);
        }
        assert_eq!(
            retry_action_v1(&mut backoff, transient),
            Ok(RetryActionV1::Wait(std::time::Duration::from_secs(2)))
        );
        assert_eq!(
            retry_action_v1(&mut backoff, Ok(true)),
            Ok(RetryActionV1::Continue)
        );
        assert_eq!(
            retry_action_v1(&mut backoff, transient),
            Ok(RetryActionV1::Wait(std::time::Duration::from_millis(25)))
        );
        assert_eq!(
            retry_action_v1(
                &mut backoff,
                Err(MailPersonsSyncManagedRuntimeErrorV1::ControlClosed)
            ),
            Ok(RetryActionV1::Stop)
        );
    }

    #[test]
    fn nontransient_runtime_failure_is_not_retried() {
        let mut backoff = RuntimeRetryBackoffV1::default();
        assert!(
            retry_action_v1(
                &mut backoff,
                Err(MailPersonsSyncManagedRuntimeErrorV1::EventContract)
            )
            .is_err()
        );
    }
}
