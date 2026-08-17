use makosh_reviewed_person_match_candidate_promotion_persistence::{
    ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    reviewed_person_match_candidate_promotion_storage_bundle_v1,
};
use makosh_reviewed_person_match_candidate_promotion_runtime::{
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1,
    ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1,
    ReviewedPersonMatchCandidatePromotionManagedRuntimeV1,
    ReviewedPersonMatchCandidatePromotionRuntimeAdmissionV1,
    reviewed_person_match_candidate_promotion_module_descriptor_v1,
    reviewed_person_match_candidate_promotion_settings_schema_bytes_v1,
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
use std::{
    ffi::{OsStr, OsString},
    fs,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::net::UnixStream,
    },
    path::{Path, PathBuf},
};

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}
fn main() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1).peekable();
    let Some(command) = args.next() else {
        return Err("Reviewed Person Match Candidate Promotion command is required".into());
    };
    match command.to_str() {
        Some("export-storage-bundle") => export(
            &mut args,
            reviewed_person_match_candidate_promotion_storage_bundle_v1().encode_to_vec(),
        ),
        Some("export-module-descriptor") => {
            let build = required_string(&mut args, "build id")?;
            export(
                &mut args,
                reviewed_person_match_candidate_promotion_module_descriptor_v1(&build)
                    .encode_to_vec(),
            )
        }
        Some("export-settings-schema") => export(
            &mut args,
            reviewed_person_match_candidate_promotion_settings_schema_bytes_v1(),
        ),
        Some("serve-inherited") => serve_inherited(&mut args),
        _ => Err("Reviewed Person Match Candidate Promotion command is invalid".into()),
    }
}
fn serve_inherited<I>(args: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(args)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let settings = read_contract(&paths.settings_schema)?;
    decode_settings_schema_v1(&settings).map_err(|_| {
        "Reviewed Person Match Candidate Promotion settings schema is invalid".to_owned()
    })?;
    let configuration = ManagedWorkflowRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| {
        "Reviewed Person Match Candidate Promotion runtime configuration is invalid".to_owned()
    })?;
    validate_managed_workflow_runtime_configuration(&configuration).map_err(|_| {
        "Reviewed Person Match Candidate Promotion runtime configuration is invalid".to_owned()
    })?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err(
            "Reviewed Person Match Candidate Promotion runtime configuration is stale".into(),
        );
    }
    let storage = configuration.storage.clone().ok_or_else(|| {
        "Reviewed Person Match Candidate Promotion storage is unavailable".to_owned()
    })?;
    let admission = ReviewedPersonMatchCandidatePromotionRuntimeAdmissionV1 {
        logical_owner_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1.to_owned(),
        logical_human_owner_id: configuration.logical_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor = tokio::runtime::Runtime::new().map_err(|_| {
        "Reviewed Person Match Candidate Promotion executor is unavailable".to_owned()
    })?;
    let mut runtime = executor
        .block_on(ReviewedPersonMatchCandidatePromotionManagedRuntimeV1::open(
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
    .map_err(|_| {
        "Reviewed Person Match Candidate Promotion polling bounds are invalid".to_owned()
    })?;
    loop {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Reviewed Person Match Candidate Promotion clock is invalid".to_owned())?
            .as_millis()
            .try_into()
            .map_err(|_| "Reviewed Person Match Candidate Promotion clock is invalid".to_owned())?;
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
            Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::ControlClosed) => {
                return Ok(());
            }
            Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::EventUnavailable)
            | Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Persistence(
                ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::StorageUnavailable,
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
fn runtime_error(error: ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1) -> String {
    let code = match error {
        ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission => "admission",
        ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::EventContract => {
            "event_contract"
        }
        ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::EventUnavailable => {
            "event_unavailable"
        }
        ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Persistence(_) => "persistence",
        ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::ControlClosed => {
            "control_closed"
        }
        ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Unavailable => "unavailable",
    };
    format!("Reviewed Person Match Candidate Promotion runtime failed: {code}")
}
fn export<I: Iterator<Item = OsString>>(args: &mut I, bytes: Vec<u8>) -> Result<(), String> {
    let output = args.next().map(PathBuf::from).ok_or_else(|| {
        "Reviewed Person Match Candidate Promotion output path is required".to_owned()
    })?;
    require_no_arguments(args)?;
    fs::write(output, bytes)
        .map_err(|_| "Reviewed Person Match Candidate Promotion export failed".to_owned())
}
fn parse_paths<I: Iterator<Item = OsString>>(
    args: &mut std::iter::Peekable<I>,
) -> Result<InheritedPaths, String> {
    let paths = InheritedPaths {
        descriptor: required_path(args, "--descriptor-path")?,
        settings_schema: required_path(args, "--settings-schema-path")?,
        runtime_configuration: required_path(args, "--runtime-configuration-path")?,
        runtime_instance_id: required_string(args, "--runtime-instance-id")?,
    };
    require_no_arguments(args)?;
    Ok(paths)
}
fn required_path<I: Iterator<Item = OsString>>(
    args: &mut I,
    name: &str,
) -> Result<PathBuf, String> {
    required_string(args, name).map(PathBuf::from)
}
fn required_string<I: Iterator<Item = OsString>>(
    args: &mut I,
    name: &str,
) -> Result<String, String> {
    if name.starts_with("--") && args.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Reviewed Person Match Candidate Promotion arguments are invalid".into());
    }
    args.next()
        .and_then(|v| v.into_string().ok())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("Reviewed Person Match Candidate Promotion {name} is required"))
}
fn require_no_arguments<I: Iterator<Item = OsString>>(args: &mut I) -> Result<(), String> {
    if args.next().is_some() {
        Err("Reviewed Person Match Candidate Promotion arguments are invalid".into())
    } else {
        Ok(())
    }
}
fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err(
            "Reviewed Person Match Candidate Promotion control channel is unavailable".into(),
        );
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}
fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Reviewed Person Match Candidate Promotion contract path is invalid".into());
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        "Reviewed Person Match Candidate Promotion contract is unavailable".to_owned()
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 2 * 1024 * 1024
    {
        return Err("Reviewed Person Match Candidate Promotion contract is invalid".into());
    }
    let bytes = fs::read(path).map_err(|_| {
        "Reviewed Person Match Candidate Promotion contract is unavailable".to_owned()
    })?;
    if bytes.is_empty() {
        return Err("Reviewed Person Match Candidate Promotion contract is invalid".into());
    }
    Ok(bytes)
}
