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

use makosh_mail_contacts_sync_persistence::{
    MailContactsSyncPersistenceErrorV1, mail_contacts_sync_storage_bundle_v1,
};
use makosh_mail_contacts_sync_runtime::{
    MailContactsSyncManagedRuntimeErrorV1, MailContactsSyncManagedRuntimeV1,
    MailContactsSyncRuntimeAdmissionV1, decode_mail_contacts_sync_settings_v1,
    mail_contacts_sync_module_descriptor_v1, mail_contacts_sync_settings_schema_bytes_v1,
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
    settings_snapshot: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os().skip(1).peekable();
    let Some(command) = arguments.next() else {
        return Err("Mail Contacts Sync command is required".to_owned());
    };
    match command.to_str() {
        Some("export-storage-bundle") => export_bytes(
            &mut arguments,
            mail_contacts_sync_storage_bundle_v1().encode_to_vec(),
        ),
        Some("export-module-descriptor") => {
            let build_id = required_string(&mut arguments, "build id")?;
            export_bytes(
                &mut arguments,
                mail_contacts_sync_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some("export-settings-schema") => export_bytes(
            &mut arguments,
            mail_contacts_sync_settings_schema_bytes_v1(),
        ),
        Some("serve-inherited") => serve_inherited(&mut arguments),
        _ => Err("Mail Contacts Sync command is invalid".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let schema_bytes = read_contract(&paths.settings_schema)?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Mail Contacts Sync settings schema is invalid".to_owned())?;
    let selected_snapshot_bytes = read_contract(&paths.settings_snapshot)?;
    let selected_snapshot = decode_settings_snapshot_v1(&selected_snapshot_bytes)
        .map_err(|_| "Mail Contacts Sync selected settings are invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &selected_snapshot)
        .map_err(|_| "Mail Contacts Sync selected settings are invalid".to_owned())?;
    let configuration = ManagedWorkflowRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Mail Contacts Sync runtime configuration is invalid".to_owned())?;
    validate_managed_workflow_runtime_configuration(&configuration)
        .map_err(|_| "Mail Contacts Sync runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Mail Contacts Sync runtime configuration is stale".to_owned());
    }
    let selected = configuration
        .configuration_instances
        .iter()
        .find(|instance| {
            instance.configuration_instance_id == configuration.configuration_instance_id
        })
        .ok_or_else(|| "Mail Contacts Sync settings catalog is invalid".to_owned())?;
    if selected.settings_snapshot_bytes != selected_snapshot_bytes
        || selected_snapshot.target_id != configuration.configuration_instance_id
        || selected_snapshot.revision != configuration.settings_revision
    {
        return Err("Mail Contacts Sync settings catalog is stale".to_owned());
    }
    let configurations = configuration
        .configuration_instances
        .iter()
        .map(|instance| {
            let snapshot = decode_settings_snapshot_v1(&instance.settings_snapshot_bytes)
                .map_err(|_| "Mail Contacts Sync settings catalog is invalid".to_owned())?;
            validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
                .map_err(|_| "Mail Contacts Sync settings catalog is invalid".to_owned())?;
            let settings = decode_mail_contacts_sync_settings_v1(&snapshot)
                .map_err(|_| "Mail Contacts Sync settings catalog is invalid".to_owned())?;
            Ok((instance.configuration_instance_id.clone(), settings))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Mail Contacts Sync storage is unavailable".to_owned())?;
    let admission = MailContactsSyncRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Mail Contacts Sync executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(MailContactsSyncManagedRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            schema_bytes,
            &admission,
            configurations,
            storage,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        ))
        .map_err(runtime_error)?;
    loop {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Mail Contacts Sync clock is invalid".to_owned())?
            .as_millis()
            .try_into()
            .map_err(|_| "Mail Contacts Sync clock is invalid".to_owned())?;
        retry_runtime("control", executor.block_on(runtime.pump_control_once(now)))?;
        retry_runtime("outbox", executor.block_on(runtime.relay_outbox_once(now)))?;
        retry_runtime(
            "scheduler_due",
            executor.block_on(runtime.consume_scheduler_due_once(now)),
        )?;
        retry_runtime(
            "mail_entry",
            executor.block_on(runtime.consume_mail_entry_once(now)),
        )?;
        retry_runtime(
            "mail_page_completed",
            executor.block_on(runtime.consume_mail_page_completed_once(now)),
        )?;
        retry_runtime(
            "mail_page_rejected",
            executor.block_on(runtime.consume_mail_page_rejected_once(now)),
        )?;
        retry_runtime(
            "contact_changed",
            executor.block_on(runtime.consume_contact_changed_once(now)),
        )?;
        retry_runtime(
            "contact_source_prepared",
            executor.block_on(runtime.consume_contact_source_prepared_once(now)),
        )?;
        retry_runtime(
            "contact_source_rejected",
            executor.block_on(runtime.consume_contact_source_rejected_once(now)),
        )?;
        retry_runtime(
            "mail_entry_upserted",
            executor.block_on(runtime.consume_mail_entry_upserted_once(now)),
        )?;
        retry_runtime(
            "mail_entry_upsert_rejected",
            executor.block_on(runtime.consume_mail_entry_upsert_rejected_once(now)),
        )?;
        retry_runtime(
            "provider_link_bound",
            executor.block_on(runtime.consume_provider_link_bound_once(now)),
        )?;
        retry_runtime(
            "provider_link_rejected",
            executor.block_on(runtime.consume_provider_link_rejected_once(now)),
        )?;
        retry_runtime(
            "contact_upserted",
            executor.block_on(runtime.consume_contact_upserted_once(now)),
        )?;
        retry_runtime(
            "contact_rejected",
            executor.block_on(runtime.consume_contact_rejected_once(now)),
        )?;
        retry_runtime(
            "scheduler_terminal",
            executor.block_on(runtime.queue_scheduler_terminal_once(now)),
        )?;
        retry_runtime("outbox", executor.block_on(runtime.relay_outbox_once(now)))?;
        retry_runtime(
            "client_realtime",
            executor.block_on(runtime.pump_client_realtime_once()),
        )?;
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn retry_runtime(
    stage: &'static str,
    result: Result<bool, MailContactsSyncManagedRuntimeErrorV1>,
) -> Result<(), String> {
    match result {
        Ok(_)
        | Err(MailContactsSyncManagedRuntimeErrorV1::EventUnavailable)
        | Err(MailContactsSyncManagedRuntimeErrorV1::Unavailable)
        | Err(MailContactsSyncManagedRuntimeErrorV1::Persistence(
            MailContactsSyncPersistenceErrorV1::StorageUnavailable,
        )) => Ok(()),
        Err(error) => Err(format!(
            "Mail Contacts Sync runtime {stage} failed: {}",
            runtime_error_code(error)
        )),
    }
}

fn runtime_error(error: MailContactsSyncManagedRuntimeErrorV1) -> String {
    format!(
        "Mail Contacts Sync runtime failed: {}",
        runtime_error_code(error)
    )
}

fn runtime_error_code(error: MailContactsSyncManagedRuntimeErrorV1) -> &'static str {
    match error {
        MailContactsSyncManagedRuntimeErrorV1::Admission => "admission",
        MailContactsSyncManagedRuntimeErrorV1::EventContract => "event_contract",
        MailContactsSyncManagedRuntimeErrorV1::EventUnavailable => "event_unavailable",
        MailContactsSyncManagedRuntimeErrorV1::Persistence(
            MailContactsSyncPersistenceErrorV1::InvalidInput,
        ) => "persistence_invalid_input",
        MailContactsSyncManagedRuntimeErrorV1::Persistence(
            MailContactsSyncPersistenceErrorV1::InvalidRow,
        ) => "persistence_invalid_row",
        MailContactsSyncManagedRuntimeErrorV1::Persistence(
            MailContactsSyncPersistenceErrorV1::StorageUnavailable,
        ) => "persistence_storage_unavailable",
        MailContactsSyncManagedRuntimeErrorV1::Persistence(
            MailContactsSyncPersistenceErrorV1::RequestConflict,
        ) => "persistence_request_conflict",
        MailContactsSyncManagedRuntimeErrorV1::Persistence(
            MailContactsSyncPersistenceErrorV1::InboxConflict,
        ) => "persistence_inbox_conflict",
        MailContactsSyncManagedRuntimeErrorV1::Persistence(
            MailContactsSyncPersistenceErrorV1::RevisionConflict,
        ) => "persistence_revision_conflict",
        MailContactsSyncManagedRuntimeErrorV1::Persistence(
            MailContactsSyncPersistenceErrorV1::InvalidTransition,
        ) => "persistence_invalid_transition",
        MailContactsSyncManagedRuntimeErrorV1::Persistence(
            MailContactsSyncPersistenceErrorV1::NotFound,
        ) => "persistence_not_found",
        MailContactsSyncManagedRuntimeErrorV1::Unavailable => "unavailable",
    }
}

fn export_bytes<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Mail Contacts Sync output path is required".to_owned())?;
    require_no_arguments(arguments)?;
    fs::write(output, bytes).map_err(|_| "Mail Contacts Sync export failed".to_owned())
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let paths = InheritedPaths {
        descriptor: required_path(arguments, "--descriptor-path")?,
        settings_schema: required_path(arguments, "--settings-schema-path")?,
        settings_snapshot: required_path(arguments, "--settings-snapshot-path")?,
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
        return Err("Mail Contacts Sync arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Mail Contacts Sync {name} is required"))
}

fn require_no_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Mail Contacts Sync arguments are invalid".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Mail Contacts Sync control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Mail Contacts Sync contract path is invalid".to_owned());
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| "Mail Contacts Sync contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > 2 * 1024 * 1024
    {
        return Err("Mail Contacts Sync contract is invalid".to_owned());
    }
    fs::read(path).map_err(|_| "Mail Contacts Sync contract is unavailable".to_owned())
}
