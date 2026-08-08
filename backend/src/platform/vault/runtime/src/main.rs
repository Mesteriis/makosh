//! Standalone process entry point for the first Vault runtime foundation.

mod bootstrap;
pub mod control;
mod offline;
pub mod service;
pub mod transport;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use clap::{Parser, Subcommand};
use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_store_sqlcipher::VaultStore;

#[derive(Parser)]
struct CommandLine {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Initialize {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        instance_id: String,
        #[arg(long)]
        platform_credential_dir: Option<PathBuf>,
    },
    ImportPlatformCredentials {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        platform_credential_dir: PathBuf,
    },
    Status {
        #[arg(long)]
        data_dir: PathBuf,
    },
    ExportBackup {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        destination: PathBuf,
    },
    VerifyBackup {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        source: PathBuf,
    },
    RestoreBackup {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        recovery_key_file: PathBuf,
    },
    Serve {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        runtime_dir: PathBuf,
        #[arg(long)]
        instance_id: String,
        #[arg(long)]
        runtime_generation: u64,
    },
    ServeInherited {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        instance_id: String,
        #[arg(long)]
        runtime_generation: u64,
        #[arg(long)]
        descriptor_path: PathBuf,
        #[arg(long)]
        settings_schema_path: Option<PathBuf>,
        #[arg(long)]
        kernel_authorization_key_sec1_hex: String,
    },
}

fn main() -> Result<(), String> {
    match CommandLine::parse().command {
        Command::Initialize {
            data_dir,
            instance_id,
            platform_credential_dir,
        } => initialize(&data_dir, &instance_id, platform_credential_dir.as_deref()),
        Command::ImportPlatformCredentials {
            data_dir,
            platform_credential_dir,
        } => import_platform_credentials(&data_dir, &platform_credential_dir),
        Command::Status { data_dir } => status(&data_dir),
        Command::ExportBackup {
            data_dir,
            destination,
        } => offline::backup::export(&data_dir, &destination),
        Command::VerifyBackup { data_dir, source } => offline::backup::verify(&data_dir, &source),
        Command::RestoreBackup {
            data_dir,
            source,
            recovery_key_file,
        } => offline::backup::restore(&data_dir, &source, &recovery_key_file),
        Command::Serve {
            data_dir,
            runtime_dir,
            instance_id,
            runtime_generation,
        } => serve(&data_dir, &runtime_dir, &instance_id, runtime_generation),
        Command::ServeInherited {
            data_dir,
            instance_id,
            runtime_generation,
            descriptor_path,
            settings_schema_path,
            kernel_authorization_key_sec1_hex,
        } => serve_inherited(
            &data_dir,
            &instance_id,
            runtime_generation,
            &descriptor_path,
            settings_schema_path.as_deref(),
            &kernel_authorization_key_sec1_hex,
        ),
    }
}

fn serve_inherited(
    data_dir: &Path,
    instance_id: &str,
    runtime_generation: u64,
    descriptor_path: &Path,
    settings_schema_path: Option<&Path>,
    kernel_authorization_key_sec1_hex: &str,
) -> Result<(), String> {
    ensure_private_directory(data_dir)?;
    let key = FileWrappingKeyProvider::new(&data_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .map_err(|_| "Vault file key is unavailable".to_owned())?;
    let store = VaultStore::open(
        &data_dir.join("vault.db"),
        &data_dir.join("vault.anchor"),
        &key,
    )
    .map_err(|_| "Vault recovery is required".to_owned())?;
    if store.instance_id() != instance_id {
        return Err("Vault instance is unavailable".to_owned());
    }
    let mut service = service::runtime::VaultService::new(store, runtime_generation)
        .map_err(|_| "Vault runtime generation is invalid".to_owned())?;
    let keys = transport::keys::VaultTransportKeyPair::generate();
    let authorization_key = parse_sec1_key(kernel_authorization_key_sec1_hex)?;
    control::runtime::serve(
        &mut service,
        &keys,
        read_contract_file(descriptor_path)?,
        settings_schema_path.map_or_else(|| Ok(Vec::new()), read_contract_file)?,
        authorization_key,
    )
}

fn parse_sec1_key(value: &str) -> Result<[u8; 65], String> {
    if value.len() != 130 {
        return Err("Vault authorization key is invalid".to_owned());
    }
    let mut key = [0_u8; 65];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| "Vault authorization key is invalid".to_owned())?;
    }
    Ok(key)
}

fn read_contract_file(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Vault runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Vault runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Vault runtime contract is unavailable".to_owned())
}

fn initialize(
    data_dir: &Path,
    instance_id: &str,
    platform_credential_dir: Option<&Path>,
) -> Result<(), String> {
    ensure_private_directory(data_dir)?;
    let key = FileWrappingKeyProvider::new(&data_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .map_err(|_| "Vault file key is unavailable".to_owned())?;
    let store = VaultStore::initialize(
        &data_dir.join("vault.db"),
        &data_dir.join("vault.anchor"),
        instance_id,
        &key,
    )
    .map_err(|_| "Vault initialization failed".to_owned())?;
    bootstrap::import_platform_credentials(&store, platform_credential_dir)?;
    println!("vault_state=sealed");
    Ok(())
}

fn status(data_dir: &Path) -> Result<(), String> {
    let key_path = data_dir.join("platform-wrapping-key.bin");
    let database_path = data_dir.join("vault.db");
    if !key_path.exists() || !database_path.exists() {
        println!("vault_state=uninitialized");
        return Ok(());
    }
    let key = FileWrappingKeyProvider::new(&key_path)
        .load_or_create()
        .map_err(|_| "Vault file key is unavailable".to_owned())?;
    VaultStore::open(&database_path, &data_dir.join("vault.anchor"), &key)
        .map_err(|_| "Vault recovery is required".to_owned())?;
    println!("vault_state=sealed");
    Ok(())
}

fn import_platform_credentials(
    data_dir: &Path,
    platform_credential_dir: &Path,
) -> Result<(), String> {
    ensure_private_directory(data_dir)?;
    let key = FileWrappingKeyProvider::new(&data_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .map_err(|_| "Vault file key is unavailable".to_owned())?;
    let store = VaultStore::open(
        &data_dir.join("vault.db"),
        &data_dir.join("vault.anchor"),
        &key,
    )
    .map_err(|_| "Vault recovery is required".to_owned())?;
    bootstrap::import_platform_credentials(&store, Some(platform_credential_dir))?;
    println!("vault_platform_credentials=imported");
    Ok(())
}

fn serve(
    data_dir: &Path,
    runtime_dir: &Path,
    instance_id: &str,
    runtime_generation: u64,
) -> Result<(), String> {
    ensure_private_directory(data_dir)?;
    ensure_private_directory(runtime_dir)?;
    let key = FileWrappingKeyProvider::new(&data_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .map_err(|_| "Vault file key is unavailable".to_owned())?;
    let store = VaultStore::open(
        &data_dir.join("vault.db"),
        &data_dir.join("vault.anchor"),
        &key,
    )
    .map_err(|_| "Vault recovery is required".to_owned())?;
    if store.instance_id() != instance_id {
        return Err("Vault instance is unavailable".to_owned());
    }
    let service = service::runtime::VaultService::new(store, runtime_generation)
        .map_err(|_| "Vault runtime generation is invalid".to_owned())?;
    let transport_keys = transport::keys::VaultTransportKeyPair::generate();
    let shutdown_requested = AtomicBool::new(false);
    control::socket::serve(
        runtime_dir,
        service.runtime_generation(),
        &transport_keys,
        &shutdown_requested,
    )
}

fn ensure_private_directory(path: &Path) -> Result<(), String> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|_| "Vault data directory is unavailable".to_owned())?;
    }
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Vault data directory is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Vault data directory is invalid".to_owned());
    }
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .map_err(|_| "Vault data directory permissions cannot be set".to_owned())
}
