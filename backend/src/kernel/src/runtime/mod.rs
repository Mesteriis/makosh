//! Online Kernel runtime dispatch.

pub(crate) mod external;
pub(crate) mod lifecycle;
pub(crate) mod managed;

use std::path::PathBuf;

use makosh_kernel_control_store::{InitialOwnerIdentity, StoreHealth};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::cli::Command;
use crate::control_store::lifecycle::bootstrap_control_store;
use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::infrastructure::filesystem::{
    acquire_runtime_directory_lock, resolve_data_directory, resolve_runtime_directory,
};
use crate::infrastructure::paths::prepare_runtime_directories;
use crate::platform::control_plane::serve as serve_platform_control_plane;
use crate::recovery::serve_recovery_socket;
use crate::runtime::lifecycle::shutdown::install as install_shutdown_signal;

pub(crate) fn run(data_dir_override: Option<PathBuf>, command: Command) -> Result<(), String> {
    let data_dir = resolve_data_directory(data_dir_override)?;
    let data_dir = prepare_runtime_directories(&data_dir)?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store_path = data_dir.join("kernel-control-store.sqlite");
    match command {
        Command::Status => print_status(&data_dir, bootstrap_control_store(&data_dir, &store_path)),
        Command::Serve { browser_gateway } => {
            let store = bootstrap_control_store(&data_dir, &store_path);
            let browser_gateway = browser_gateway.into_configuration()?;
            serve(store, &data_dir, &runtime_dir, &store_path, browser_gateway)
        }
        _ => unreachable!("non-runtime command was dispatched to runtime"),
    }
}

fn print_status(
    data_dir: &std::path::Path,
    store: Result<SqliteControlStore, String>,
) -> Result<(), String> {
    let (state, control_store, owner_identity, owner_device_signer) = match store {
        Ok(store) if store.snapshot().health() == StoreHealth::Trustworthy => {
            let owner = store.initial_owner_identity();
            let owner_identity = match &owner {
                Ok(Some(_)) => "enrolled",
                Ok(None) => "missing",
                Err(_) => "unavailable",
            };
            let owner_device_signer = device_signer_status(data_dir, owner.ok().flatten().as_ref());
            (
                "module_control_plane",
                "trustworthy",
                owner_identity,
                owner_device_signer,
            )
        }
        Ok(_) | Err(_) => (
            "recovery_only",
            "unavailable",
            "unavailable",
            device_signer_status(data_dir, None),
        ),
    };
    println!("state={state}");
    println!("control_store={control_store}");
    println!("owner_identity={owner_identity}");
    println!("owner_device_signer={owner_device_signer}");
    Ok(())
}

fn device_signer_status(
    data_dir: &std::path::Path,
    owner: Option<&InitialOwnerIdentity>,
) -> &'static str {
    match std::fs::symlink_metadata(FileDeviceSigner::key_path(data_dir)) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "missing",
        Err(_) => "unavailable",
        Ok(_) => match FileDeviceSigner::open_for_instance(data_dir) {
            Ok(signer)
                if owner.is_some_and(|identity| {
                    signer.public_key_sec1() != *identity.public_key_sec1()
                }) =>
            {
                "mismatch"
            }
            Ok(_) => "ready",
            Err(_) => "unavailable",
        },
    }
}

fn serve(
    store: Result<makosh_kernel_control_store_sqlite::SqliteControlStore, String>,
    data_dir: &std::path::Path,
    runtime_dir: &std::path::Path,
    store_path: &std::path::Path,
    browser_gateway: Option<crate::platform::gateway::BrowserGatewayConfigurationV1>,
) -> Result<(), String> {
    match store {
        Ok(store) if store.snapshot().health() == StoreHealth::Trustworthy => {
            serve_platform_control_plane(store, data_dir, runtime_dir, store_path, browser_gateway)
        }
        Ok(_) | Err(_) if browser_gateway.is_none() => {
            serve_recovery_socket(runtime_dir, store_path, None, install_shutdown_signal()?)
        }
        Ok(_) | Err(_) => Err("browser Gateway requires a trustworthy control store".to_owned()),
    }
}
