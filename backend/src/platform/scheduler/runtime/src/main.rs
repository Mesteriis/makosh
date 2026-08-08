//! Independently supervised Scheduler receipt-runtime composition root.

mod cli;
mod control;
mod recovery;

use makosh_runtime_protocol::{
    v1::SchedulerRuntimeConfigurationV1,
    validation::scheduler::validate_scheduler_runtime_configuration,
};
use prost::Message;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    let command = arguments.next();
    let mut arguments = arguments.peekable();
    match command.as_deref() {
        Some(command) if command == "serve-inherited" => serve_inherited(&mut arguments),
        Some(command) if command == "export-storage-bundle" => {
            export_storage_bundle(&mut arguments)
        }
        Some(command) if command == "export-recovery-bundle" => {
            recovery::export_storage_bundle(&cli::parse_export_bundle_arguments(&mut arguments)?)
        }
        Some(command) if command == "prepare-event-replay" => {
            recovery::prepare_event_hub_replay(&cli::parse_recovery_arguments(&mut arguments)?)
        }
        _ => Err("Scheduler runtime command is unavailable".to_owned()),
    }
}

fn export_storage_bundle<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    if arguments.next().is_some() {
        return Err("Scheduler runtime command is unavailable".to_owned());
    }
    let bytes = makosh_scheduler_persistence::scheduler_storage_bundle_v1().encode_to_vec();
    std::io::Write::write_all(&mut std::io::stdout(), &bytes)
        .map_err(|_| "Scheduler storage bundle is unavailable".to_owned())
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

fn read_configuration(path: &std::path::Path) -> Result<SchedulerRuntimeConfigurationV1, String> {
    let bytes = read_contract_file(path)?;
    let configuration = SchedulerRuntimeConfigurationV1::decode(bytes.as_slice())
        .map_err(|_| "Scheduler runtime configuration is invalid".to_owned())?;
    validate_scheduler_runtime_configuration(&configuration)
        .map_err(|_| "Scheduler runtime configuration is invalid".to_owned())?;
    Ok(configuration)
}

fn read_contract_file(path: &std::path::Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Scheduler runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Scheduler runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Scheduler runtime contract is unavailable".to_owned())
}
