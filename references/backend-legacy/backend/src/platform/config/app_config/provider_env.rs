use std::fs;
use std::path::PathBuf;

use crate::platform::secrets::models::ResolvedSecret;

use super::super::errors::ConfigError;
use super::super::google::GoogleOAuthClientConfig;
use super::super::parsing::parse_bool_env;
use super::AppConfig;

pub(super) fn apply_bundled_google_oauth_client(config: &mut AppConfig) -> Result<(), ConfigError> {
    if let Some(raw_json) = option_env!("MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON") {
        config.google_oauth_client = Some(GoogleOAuthClientConfig::from_client_secret_json(
            non_empty(raw_json, ConfigError::EmptyGoogleOAuthClientConfigJson)?,
        )?);
    }
    if let Some(client_id) = option_env!("MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_ID") {
        config.google_oauth_client_id =
            Some(non_empty(client_id, ConfigError::EmptyGoogleOAuthClientId)?.to_owned());
    }
    if let Some(client_secret) = option_env!("MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_SECRET") {
        config.google_oauth_client_secret = Some(ResolvedSecret::new(non_empty(
            client_secret,
            ConfigError::EmptyGoogleOAuthClientSecret,
        )?)?);
    }

    Ok(())
}

pub(super) fn apply_provider_env(
    config: &mut AppConfig,
    key: &str,
    value: &str,
) -> Result<bool, ConfigError> {
    match key {
        "MAKOSH_TDJSON_PATH" => {
            config.tdjson_path = Some(PathBuf::from(non_empty(
                value,
                ConfigError::EmptyTdjsonPath,
            )?));
        }
        "MAKOSH_TELEGRAM_API_ID" => {
            config.telegram_api_id = Some(parse_telegram_api_id(value)?);
        }
        "MAKOSH_TELEGRAM_API_HASH" => {
            config.telegram_api_hash = Some(ResolvedSecret::new(non_empty(
                value,
                ConfigError::EmptyTelegramApiHash,
            )?)?);
        }
        "MAKOSH_GOOGLE_OAUTH_CLIENT_ID" => {
            config.google_oauth_client_id =
                Some(non_empty(value, ConfigError::EmptyGoogleOAuthClientId)?.to_owned());
        }
        "MAKOSH_GOOGLE_OAUTH_CLIENT_SECRET" => {
            config.google_oauth_client_secret = Some(ResolvedSecret::new(non_empty(
                value,
                ConfigError::EmptyGoogleOAuthClientSecret,
            )?)?);
        }
        "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON" => {
            config.google_oauth_client = Some(GoogleOAuthClientConfig::from_client_secret_json(
                non_empty(value, ConfigError::EmptyGoogleOAuthClientConfigJson)?,
            )?);
        }
        "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH" => {
            config.google_oauth_client = Some(google_oauth_client_from_path(value)?);
        }
        "MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED" => {
            config.zoom_token_maintenance_scheduler_enabled = parse_bool_env(
                "MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED",
                value.trim(),
            )?;
        }
        "MAKOSH_ZOOM_RECORDING_SYNC_SCHEDULER_ENABLED" => {
            config.zoom_recording_sync_scheduler_enabled =
                parse_bool_env("MAKOSH_ZOOM_RECORDING_SYNC_SCHEDULER_ENABLED", value.trim())?;
        }
        "MAKOSH_ZOOM_RETENTION_CLEANUP_SCHEDULER_ENABLED" => {
            config.zoom_retention_cleanup_scheduler_enabled = parse_bool_env(
                "MAKOSH_ZOOM_RETENTION_CLEANUP_SCHEDULER_ENABLED",
                value.trim(),
            )?;
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn google_oauth_client_from_path(value: &str) -> Result<GoogleOAuthClientConfig, ConfigError> {
    let path = PathBuf::from(non_empty(
        value,
        ConfigError::EmptyGoogleOAuthClientConfigPath,
    )?);
    let raw_json = fs::read_to_string(&path)
        .map_err(|source| ConfigError::GoogleOAuthClientConfigRead { path, source })?;
    GoogleOAuthClientConfig::from_client_secret_json(&raw_json)
}

fn parse_telegram_api_id(value: &str) -> Result<i64, ConfigError> {
    let raw_id = value.trim();
    let api_id = raw_id
        .parse::<i64>()
        .map_err(|source| ConfigError::InvalidTelegramApiId {
            value: raw_id.to_owned(),
            reason: "must be a positive integer",
            source: Some(source),
        })?;
    if api_id <= 0 {
        return Err(ConfigError::InvalidTelegramApiId {
            value: raw_id.to_owned(),
            reason: "must be greater than zero",
            source: None,
        });
    }

    Ok(api_id)
}

fn non_empty(value: &str, error: ConfigError) -> Result<&str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(error)
    } else {
        Ok(trimmed)
    }
}
