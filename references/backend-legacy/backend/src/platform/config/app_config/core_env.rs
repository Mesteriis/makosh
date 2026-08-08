use std::path::PathBuf;

use crate::platform::secrets::models::ResolvedSecret;

use super::super::errors::ConfigError;
use super::super::parsing::parse_bool_env;
use super::AppConfig;

pub(super) fn apply_core_env(
    config: &mut AppConfig,
    key: &str,
    value: &str,
) -> Result<bool, ConfigError> {
    match key {
        "MAKOSH_HTTP_ADDR" => {
            let raw_addr = value.trim();
            config.http_addr = raw_addr
                .parse()
                .map_err(|source| ConfigError::InvalidHttpAddr {
                    value: raw_addr.to_owned(),
                    source,
                })?;
        }
        "DATABASE_URL" => {
            config.database_url = Some(non_empty(value, ConfigError::EmptyDatabaseUrl)?.to_owned());
        }
        "MAKOSH_LOCAL_API_SECRET" => {
            config.local_api_secret =
                Some(non_empty(value, ConfigError::EmptyLocalApiSecret)?.to_owned());
        }
        "MAKOSH_NATS_SERVER_URL" => {
            config.nats_server_url =
                Some(non_empty(value, ConfigError::EmptyNatsServerUrl)?.to_owned());
        }
        "MAKOSH_SECRET_VAULT_PATH" => {
            config.secret_vault_path = Some(PathBuf::from(non_empty(
                value,
                ConfigError::EmptySecretVaultPath,
            )?));
        }
        "MAKOSH_SECRET_VAULT_KEY" => {
            config.secret_vault_key = Some(ResolvedSecret::new(non_empty(
                value,
                ConfigError::EmptySecretVaultKey,
            )?)?);
        }
        "MAKOSH_VAULT_HOME" => {
            config.vault_home = PathBuf::from(non_empty(value, ConfigError::EmptyVaultHome)?);
        }
        "MAKOSH_DEV_MODE" => {
            config.dev_mode = parse_bool_env("MAKOSH_DEV_MODE", value.trim())?;
        }
        "MAKOSH_DEV_KEY_PATH" => {
            config.dev_key_path = PathBuf::from(non_empty(value, ConfigError::EmptyDevKeyPath)?);
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn non_empty(value: &str, error: ConfigError) -> Result<&str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(error)
    } else {
        Ok(trimmed)
    }
}
