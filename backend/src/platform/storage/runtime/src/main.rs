//! Storage Control composition root.

mod admin;
mod cli;
mod control;
mod observability;
mod recovery;
pub(crate) use makosh_storage_vault as vault;

use std::path::Path;

use makosh_storage_protocol::{
    v1::StorageRuntimeConfigurationV1, validation::validate_storage_runtime_configuration,
};
use prost::Message;

const MAX_STORAGE_RUNTIME_CONTRACT_BYTES: u64 = 4 * 1024 * 1024;

fn main() {
    if let Err(error) = observability::initialize() {
        eprintln!("Storage observability initialization failed: {error}");
        std::process::exit(1);
    }
    tracing::info!(event = "storage.process.started");
    if let Err(error) = run() {
        tracing::error!(
            event = "storage.process.failed",
            error.class = "storage_runtime",
            error.message = %error,
        );
        std::process::exit(1);
    }
    tracing::info!(event = "storage.process.stopped");
}

fn run() -> Result<(), String> {
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
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Storage runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_STORAGE_RUNTIME_CONTRACT_BYTES
    {
        return Err("Storage runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Storage runtime contract is unavailable".to_owned())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{MAX_STORAGE_RUNTIME_CONTRACT_BYTES, read_contract_file};

    static NEXT_CONTRACT_ID: AtomicU64 = AtomicU64::new(1);

    fn contract_file(length: usize) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "makosh-storage-runtime-contract-{}-{}",
            std::process::id(),
            NEXT_CONTRACT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, vec![0_u8; length]).expect("contract fixture must be writable");
        path
    }

    #[test]
    fn storage_runtime_contract_limit_accepts_the_admitted_multi_owner_configuration() {
        let path = contract_file(600 * 1024);

        let bytes = read_contract_file(&path).expect("multi-owner contract must be accepted");
        std::fs::remove_file(path).expect("contract fixture must be removable");

        assert_eq!(bytes.len(), 600 * 1024);
    }

    #[test]
    fn storage_runtime_contract_limit_remains_bounded() {
        let path = contract_file((MAX_STORAGE_RUNTIME_CONTRACT_BYTES + 1) as usize);

        let error = read_contract_file(&path).expect_err("oversized contract must be rejected");
        std::fs::remove_file(path).expect("contract fixture must be removable");

        assert_eq!(error, "Storage runtime contract is unavailable");
    }
}
