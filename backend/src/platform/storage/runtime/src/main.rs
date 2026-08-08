//! Storage Control composition root.

mod admin;
mod cli;
mod control;
mod recovery;
pub(crate) use makosh_storage_vault as vault;

use std::path::Path;

use makosh_storage_protocol::{
    v1::StorageRuntimeConfigurationV1, validation::validate_storage_runtime_configuration,
};
use prost::Message;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    let command = arguments.next();
    let mut arguments = arguments.peekable();
    match command.as_deref() {
        Some(command) if command == "serve-inherited" => serve_inherited(&mut arguments),
        Some(command) => recovery::execute(cli::parse_offline_recovery_command(
            command,
            &mut arguments,
        )?),
        _ => Err("Storage runtime command is unavailable".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    let paths = cli::parse_serve_inherited_arguments(arguments)?;
    control::serve_inherited(
        read_contract_file(&paths.descriptor_path)?,
        paths
            .settings_schema_path
            .map_or_else(|| Ok(Vec::new()), |path| read_contract_file(&path))?,
        read_configuration(&paths.configuration_path)?,
    )
}

fn read_configuration(path: &Path) -> Result<StorageRuntimeConfigurationV1, String> {
    let bytes = read_contract_file(path)?;
    let configuration = StorageRuntimeConfigurationV1::decode(bytes.as_slice())
        .map_err(|_| "Storage runtime configuration is invalid".to_owned())?;
    validate_storage_runtime_configuration(&configuration)
        .map_err(|_| "Storage runtime configuration is invalid".to_owned())?;
    Ok(configuration)
}

fn read_contract_file(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Storage runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Storage runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Storage runtime contract is unavailable".to_owned())
}
