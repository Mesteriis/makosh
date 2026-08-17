use std::{
    ffi::{OsStr, OsString},
    fs,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::net::UnixStream,
    },
    path::{Path, PathBuf},
};

use makosh_review_person_match_candidate_persistence::{
    ReviewPersonMatchCandidatePersistenceErrorV1, review_person_match_candidate_storage_bundle_v1,
};
use makosh_review_person_match_candidate_runtime::{
    ReviewPersonMatchCandidateManagedRuntimeErrorV1, ReviewPersonMatchCandidateManagedRuntimeV1,
    ReviewPersonMatchCandidateRuntimeAdmissionV1,
    review_person_match_candidate_module_descriptor_v1,
    review_person_match_candidate_settings_schema_bytes_v1,
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
        return Err("Review Person Match Candidate command is required".to_owned());
    };
    match command.to_str() {
        Some("export-storage-bundle") => export(
            &mut arguments,
            review_person_match_candidate_storage_bundle_v1().encode_to_vec(),
        ),
        Some("export-module-descriptor") => {
            let build_id = required_string(&mut arguments, "build id")?;
            export(
                &mut arguments,
                review_person_match_candidate_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some("export-settings-schema") => export(
            &mut arguments,
            review_person_match_candidate_settings_schema_bytes_v1(),
        ),
        Some("serve-inherited") => serve_inherited(&mut arguments),
        _ => Err("Review Person Match Candidate command is invalid".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let settings = read_contract(&paths.settings_schema)?;
    decode_settings_schema_v1(&settings)
        .map_err(|_| "Review Person Match Candidate settings schema is invalid".to_owned())?;
    let configuration = ManagedDomainRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Review Person Match Candidate runtime configuration is invalid".to_owned())?;
    validate_managed_domain_runtime_configuration(&configuration)
        .map_err(|_| "Review Person Match Candidate runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Review Person Match Candidate runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Review Person Match Candidate storage is unavailable".to_owned())?;
    let admission = ReviewPersonMatchCandidateRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        logical_human_owner_id: configuration.logical_human_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new()
        .map_err(|_| "Review Person Match Candidate executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(ReviewPersonMatchCandidateManagedRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            settings,
            &admission,
            storage,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        ))
        .map_err(runtime_error)?;
    let mut failures = 0_u8;
    let mut poll_backoff = ManagedRuntimePollBackoffV1::new(
        std::time::Duration::from_millis(25),
        std::time::Duration::from_millis(100),
    )
    .map_err(|_| "Review Person Match Candidate polling bounds are invalid".to_owned())?;
    loop {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Review Person Match Candidate clock is invalid".to_owned())?
            .as_millis()
            .try_into()
            .map_err(|_| "Review Person Match Candidate clock is invalid".to_owned())?;
        match executor.block_on(runtime.service_once(now)) {
            Ok(progressed) => {
                failures = 0;
                let delay = poll_backoff.observe(progressed);
                if !delay.is_zero()
                    && !executor
                        .block_on(runtime.wait_retry_delay(delay))
                        .map_err(runtime_error)?
                {
                    return Ok(());
                }
            }
            Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::ControlClosed) => return Ok(()),
            Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::EventUnavailable)
            | Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Persistence(
                ReviewPersonMatchCandidatePersistenceErrorV1::StorageUnavailable,
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

fn runtime_error(error: ReviewPersonMatchCandidateManagedRuntimeErrorV1) -> String {
    let code = match error {
        ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission => "admission",
        ReviewPersonMatchCandidateManagedRuntimeErrorV1::EventContract => "event_contract",
        ReviewPersonMatchCandidateManagedRuntimeErrorV1::EventUnavailable => "event_unavailable",
        ReviewPersonMatchCandidateManagedRuntimeErrorV1::Persistence(_) => "persistence",
        ReviewPersonMatchCandidateManagedRuntimeErrorV1::ControlClosed => "control_closed",
        ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Review Person Match Candidate runtime failed: {code}")
}

fn export<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Review Person Match Candidate output path is required".to_owned())?;
    require_no_arguments(arguments)?;
    fs::write(output, bytes).map_err(|_| "Review Person Match Candidate export failed".to_owned())
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
        return Err("Review Person Match Candidate arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Review Person Match Candidate {name} is required"))
}

fn require_no_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        Err("Review Person Match Candidate arguments are invalid".to_owned())
    } else {
        Ok(())
    }
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Review Person Match Candidate control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Review Person Match Candidate contract path is invalid".to_owned());
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| "Review Person Match Candidate contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 2 * 1024 * 1024
    {
        return Err("Review Person Match Candidate contract is invalid".to_owned());
    }
    let bytes = fs::read(path)
        .map_err(|_| "Review Person Match Candidate contract is unavailable".to_owned())?;
    if bytes.is_empty() {
        return Err("Review Person Match Candidate contract is invalid".to_owned());
    }
    Ok(bytes)
}
