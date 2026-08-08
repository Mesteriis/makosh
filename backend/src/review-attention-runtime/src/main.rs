use std::ffi::{OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use makosh_review_attention_persistence::schema::review_attention_storage_bundle_v1;
use makosh_review_attention_runtime::{
    ReviewAttentionManagedRuntimeErrorV1, ReviewAttentionManagedRuntimeV1,
    ReviewAttentionRuntimeAdmissionV1, review_attention_module_descriptor_v1,
    review_attention_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::{
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
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    let command = arguments.next();
    let mut arguments = arguments.peekable();
    match command.as_deref() {
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(&mut arguments)
        }
        Some(command) if command == OsStr::new("export-storage-bundle") => export_bytes(
            &mut arguments,
            review_attention_storage_bundle_v1().encode_to_vec(),
        ),
        Some(command) if command == OsStr::new("export-module-descriptor") => {
            let build_id = arguments
                .next()
                .and_then(|value| value.into_string().ok())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Review descriptor build id is required".to_owned())?;
            export_bytes(
                &mut arguments,
                review_attention_module_descriptor_v1(&build_id).encode_to_vec(),
            )
        }
        Some(command) if command == OsStr::new("export-settings-schema") => {
            export_bytes(&mut arguments, review_attention_settings_schema_bytes_v1())
        }
        _ => Err("Review runtime command is unavailable".to_owned()),
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
        .map_err(|_| "Review settings schema is invalid".to_owned())?;
    let configuration = ManagedDomainRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Review runtime configuration is invalid".to_owned())?;
    validate_managed_domain_runtime_configuration(&configuration)
        .map_err(|_| "Review runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Review runtime configuration is stale".to_owned());
    }
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Review storage is unavailable".to_owned())?;
    let admission = ReviewAttentionRuntimeAdmissionV1 {
        logical_owner_id: configuration.logical_owner_id,
        logical_human_owner_id: configuration.logical_human_owner_id,
        registration_id: configuration.registration_id,
        runtime_instance_id: configuration.runtime_instance_id,
        runtime_generation: configuration.runtime_generation,
        grant_epoch: configuration.grant_epoch,
    };
    let executor =
        tokio::runtime::Runtime::new().map_err(|_| "Review executor is unavailable".to_owned())?;
    let mut runtime = executor
        .block_on(ReviewAttentionManagedRuntimeV1::open(
            inherited_control_channel()?,
            descriptor,
            settings_schema,
            &admission,
            storage,
        ))
        .map_err(runtime_error)?;
    loop {
        executor
            .block_on(runtime.pump_control_once())
            .map_err(runtime_error)?;
    }
}

fn runtime_error(error: ReviewAttentionManagedRuntimeErrorV1) -> String {
    let reason = match error {
        ReviewAttentionManagedRuntimeErrorV1::Admission => "admission_rejected",
        ReviewAttentionManagedRuntimeErrorV1::Persistence(_) => "storage_unavailable",
        ReviewAttentionManagedRuntimeErrorV1::InvalidTransition => "realtime_transition_rejected",
        ReviewAttentionManagedRuntimeErrorV1::Unavailable => "control_channel_unavailable",
    };
    format!("Review runtime failed: {reason}")
}

fn export_bytes<I>(arguments: &mut I, bytes: Vec<u8>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    require_no_arguments(arguments)?;
    std::io::Write::write_all(&mut std::io::stdout(), &bytes)
        .map_err(|_| "Review runtime artifact is unavailable".to_owned())
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let runtime_configuration = required_path(arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_string(arguments, "--runtime-instance-id")?;
    require_no_arguments(arguments)?;
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
        return Err("Review runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Review runtime arguments are invalid".to_owned())
}

fn require_no_arguments<I>(arguments: &mut I) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().is_some() {
        return Err("Review runtime command is unavailable".to_owned());
    }
    Ok(())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Review control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| "Review contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Review contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Review contract is unavailable".to_owned())
}
