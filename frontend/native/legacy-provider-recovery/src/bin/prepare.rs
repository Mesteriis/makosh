use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;

use makosh_legacy_provider_recovery::LegacyProviderRecoveryErrorV1;
use makosh_legacy_provider_recovery::preparation::{
    LegacyProviderRecoveryPreparationInputV1, prepare_bundle,
};

fn main() {
    let result = run();
    match result {
        Ok(receipt) => match serde_json::to_string(&receipt) {
            Ok(receipt) => println!("{receipt}"),
            Err(_) => {
                eprintln!("invalid_configuration");
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("{}", error.code());
            std::process::exit(1);
        }
    }
}

fn run() -> Result<
    makosh_legacy_provider_recovery::preparation::LegacyProviderRecoveryPreparationReceiptV1,
    LegacyProviderRecoveryErrorV1,
> {
    let input = parse_args(std::env::args().skip(1))?;
    prepare_bundle(&input)
}

fn parse_args(
    args: impl Iterator<Item = String>,
) -> Result<LegacyProviderRecoveryPreparationInputV1, LegacyProviderRecoveryErrorV1> {
    let mut database_host = None;
    let mut database_port = None;
    let mut database_environment_file = None;
    let mut provider_environment_file = None;
    let mut legacy_vault_root = None;
    let mut legacy_vault_master_key_file = None;
    let mut output_root = None;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        let value = args
            .next()
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidArguments)?;
        match argument.as_str() {
            "--database-host" => {
                database_host = Some(
                    IpAddr::from_str(&value)
                        .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidArguments)?,
                );
            }
            "--database-port" => {
                database_port = Some(
                    value
                        .parse::<u16>()
                        .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidArguments)?,
                );
            }
            "--database-env-file" => database_environment_file = Some(PathBuf::from(value)),
            "--provider-env-file" => provider_environment_file = Some(PathBuf::from(value)),
            "--legacy-vault-root" => legacy_vault_root = Some(PathBuf::from(value)),
            "--legacy-vault-master-key-file" => {
                legacy_vault_master_key_file = Some(PathBuf::from(value));
            }
            "--output-root" => output_root = Some(PathBuf::from(value)),
            _ => return Err(LegacyProviderRecoveryErrorV1::InvalidArguments),
        }
    }
    Ok(LegacyProviderRecoveryPreparationInputV1 {
        database_host: database_host.ok_or(LegacyProviderRecoveryErrorV1::InvalidArguments)?,
        database_port: database_port.ok_or(LegacyProviderRecoveryErrorV1::InvalidArguments)?,
        database_environment_file: database_environment_file
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidArguments)?,
        provider_environment_file: provider_environment_file
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidArguments)?,
        legacy_vault_root: legacy_vault_root
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidArguments)?,
        legacy_vault_master_key_file: legacy_vault_master_key_file
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidArguments)?,
        output_root: output_root.ok_or(LegacyProviderRecoveryErrorV1::InvalidArguments)?,
    })
}
