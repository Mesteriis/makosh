//! Events authority managed-child composition root.

use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use makosh_events_authority_runtime_control::serve_inherited as serve_control_inherited;
use makosh_runtime_protocol::{
    v1::EventsAuthorityRuntimeConfigurationV1,
    validation::events_authority::validate_events_authority_runtime_configuration,
};
use prost::Message;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    let command = arguments.next();
    let mut arguments = arguments.peekable();
    match command.as_deref() {
        Some(command) if command == "serve-inherited" => serve_inherited(&mut arguments),
        _ => Err("Events authority command is unavailable".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    let descriptor_path = required_path(arguments, "--descriptor-path")?;
    let settings_schema_path = required_path(arguments, "--settings-schema-path")?;
    let configuration_path = required_path(arguments, "--configuration-path")?;
    if arguments.next().is_some() {
        return Err("Events authority arguments are invalid".to_owned());
    }
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Events authority inherited channel is unavailable".to_owned());
    }
    let channel = unsafe { UnixStream::from_raw_fd(duplicated) };
    serve_control_inherited(
        channel,
        read_contract_file(&descriptor_path)?,
        read_contract_file(&settings_schema_path)?,
        read_configuration(&configuration_path)?,
    )
}

fn required_path<I>(arguments: &mut std::iter::Peekable<I>, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new(name)) {
        return Err("Events authority arguments are invalid".to_owned());
    }
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "Events authority arguments are invalid".to_owned())
}

fn read_configuration(path: &Path) -> Result<EventsAuthorityRuntimeConfigurationV1, String> {
    let bytes = read_contract_file(path)?;
    let configuration = EventsAuthorityRuntimeConfigurationV1::decode(bytes.as_slice())
        .map_err(|_| "Events authority configuration is invalid".to_owned())?;
    validate_events_authority_runtime_configuration(&configuration)
        .map_err(|_| "Events authority configuration is invalid".to_owned())?;
    Ok(configuration)
}

fn read_contract_file(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Events authority contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Events authority contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Events authority contract is unavailable".to_owned())
}
