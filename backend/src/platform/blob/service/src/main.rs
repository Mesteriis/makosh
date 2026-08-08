//! Blob service composition root for the inherited managed-child contract.

mod cli;
mod control;

use makosh_runtime_protocol::{
    v1::BlobRuntimeConfigurationV1, validation::blob::validate_blob_runtime_configuration,
};
use prost::Message;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    let command = arguments.next();
    let mut arguments = arguments.peekable();
    match command.as_deref() {
        Some(command) if command == "serve-inherited" => serve_inherited(&mut arguments),
        Some(command) => offline_recovery(command, &mut arguments),
        _ => Err("Blob service command is unavailable".to_owned()),
    }
}

fn offline_recovery<I>(
    command: &std::ffi::OsStr,
    arguments: &mut std::iter::Peekable<I>,
) -> Result<(), String>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    match cli::parse_offline_recovery_command(command, arguments)? {
        cli::OfflineRecoveryCommand::Export {
            data_dir,
            destination,
        } => makosh_blob_runtime::recovery::export_backup_offline(&data_dir, &destination)
            .map(|_| ()),
        cli::OfflineRecoveryCommand::Verify { source } => {
            makosh_blob_runtime::recovery::verify_backup_offline(&source).map(|_| ())
        }
        cli::OfflineRecoveryCommand::Restore { source, data_dir } => {
            makosh_blob_runtime::recovery::restore_backup_offline(&source, &data_dir).map(|_| ())
        }
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

fn read_configuration(path: &std::path::Path) -> Result<BlobRuntimeConfigurationV1, String> {
    let bytes = read_contract_file(path)?;
    let configuration = BlobRuntimeConfigurationV1::decode(bytes.as_slice())
        .map_err(|_| "Blob runtime configuration is invalid".to_owned())?;
    validate_blob_runtime_configuration(&configuration)
        .map_err(|_| "Blob runtime configuration is invalid".to_owned())?;
    Ok(configuration)
}

fn read_contract_file(path: &std::path::Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Blob runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Blob runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Blob runtime contract is unavailable".to_owned())
}
