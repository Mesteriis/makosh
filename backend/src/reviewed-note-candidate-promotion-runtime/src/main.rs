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

use makosh_reviewed_note_candidate_promotion_persistence::schema::reviewed_note_candidate_promotion_storage_bundle_v1;
use makosh_reviewed_note_candidate_promotion_runtime::{
    ReviewedNoteCandidatePromotionManagedRuntimeErrorV1,
    ReviewedNoteCandidatePromotionManagedRuntimeV1,
    ReviewedNoteCandidatePromotionRuntimeAdmissionV1,
    reviewed_note_candidate_promotion_module_descriptor_v1,
    reviewed_note_candidate_promotion_settings_schema_bytes_v1,
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
    let Some(command) = arguments.next() else {
        return Err("Promotion workflow command is required".to_owned());
    };
    match command.to_str() {
        Some("export-storage-bundle") => export(
            &mut arguments,
            reviewed_note_candidate_promotion_storage_bundle_v1().encode_to_vec(),
        ),
        Some("export-module-descriptor") => {
            let build_id = required_string(&mut arguments, "build id")?;
            export(
                &mut arguments,
                reviewed_note_candidate_promotion_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some("export-settings-schema") => export(
            &mut arguments,
            reviewed_note_candidate_promotion_settings_schema_bytes_v1(),
        ),
        Some("serve-inherited") => serve_inherited(&mut arguments),
        _ => Err("Promotion workflow command is invalid".to_owned()),
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
        .map_err(|_| "Promotion workflow settings schema is invalid".to_owned())?;
    let configuration = ManagedWorkflowRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Promotion workflow runtime configuration is invalid".to_owned())?;
    validate_managed_workflow_runtime_configuration(&configuration)
        .map_err(|_| "Promotion workflow runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Promotion workflow runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Promotion workflow storage is unavailable".to_owned())?;
    let admission = ReviewedNoteCandidatePromotionRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Promotion workflow executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(ReviewedNoteCandidatePromotionManagedRuntimeV1::open(
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
            .map_err(|_| "Promotion workflow clock is invalid".to_owned())?
            .as_millis()
            .try_into()
            .map_err(|_| "Promotion workflow clock is invalid".to_owned())?;
        retry(runtime.pump_control_once())?;
        retry(executor.block_on(runtime.consume_approval_once(now)))?;
        retry(executor.block_on(runtime.consume_note_created_once(now)))?;
        retry(executor.block_on(runtime.consume_note_rejected_once(now)))?;
        retry(executor.block_on(runtime.relay_outbox_once(now)))?;
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn retry(
    result: Result<bool, ReviewedNoteCandidatePromotionManagedRuntimeErrorV1>,
) -> Result<(), String> {
    match result {
        Ok(_)
        | Err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::EventUnavailable)
        | Err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)
        | Err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Persistence(
            makosh_reviewed_note_candidate_promotion_persistence::ReviewedNoteCandidatePromotionPersistenceErrorV1::StorageUnavailable,
        )) => Ok(()),
        Err(error) => Err(runtime_error(error)),
    }
}

fn runtime_error(error: ReviewedNoteCandidatePromotionManagedRuntimeErrorV1) -> String {
    let code = match error {
        ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission => "admission",
        ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::EventContract => "event_contract",
        ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::EventUnavailable => {
            "event_unavailable"
        }
        ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Persistence(_) => "persistence",
        ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Promotion workflow runtime failed: {code}")
}

fn export<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Promotion workflow output path is required".to_owned())?;
    require_no_arguments(arguments)?;
    fs::write(output, bytes).map_err(|_| "Promotion workflow export failed".to_owned())
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
        return Err("Promotion workflow arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Promotion workflow {name} is required"))
}

fn require_no_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Promotion workflow arguments are invalid".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Promotion workflow control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Promotion workflow contract path is invalid".to_owned());
    }
    let bytes =
        fs::read(path).map_err(|_| "Promotion workflow contract is unavailable".to_owned())?;
    if bytes.is_empty() || bytes.len() > 2 * 1024 * 1024 {
        return Err("Promotion workflow contract is invalid".to_owned());
    }
    Ok(bytes)
}
