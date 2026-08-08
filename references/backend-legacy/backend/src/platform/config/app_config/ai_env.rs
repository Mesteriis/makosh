use crate::platform::secrets::models::ResolvedSecret;

use super::super::ai::AiRuntimeProvider;
use super::super::errors::ConfigError;
use super::AppConfig;

pub(super) fn apply_ai_env(
    config: &mut AppConfig,
    key: &str,
    value: &str,
) -> Result<bool, ConfigError> {
    match key {
        "MAKOSH_AI_PROVIDER" => {
            config.ai_provider = AiRuntimeProvider::try_from(value)?;
        }
        "MAKOSH_OLLAMA_BASE_URL" => {
            config.ollama_base_url = non_empty(value, ConfigError::EmptyOllamaBaseUrl)?
                .trim_end_matches('/')
                .to_owned();
        }
        "MAKOSH_OLLAMA_CHAT_MODEL" => {
            config.ollama_chat_model =
                non_empty(value, ConfigError::EmptyOllamaChatModel)?.to_owned();
        }
        "MAKOSH_OLLAMA_EMBED_MODEL" => {
            config.ollama_embed_model =
                non_empty(value, ConfigError::EmptyOllamaEmbedModel)?.to_owned();
        }
        "MAKOSH_OLLAMA_TIMEOUT_SECONDS" => {
            config.ollama_timeout_seconds = parse_positive_timeout(value, TimeoutTarget::Ollama)?;
        }
        "MAKOSH_OMNIROUTE_BASE_URL" => {
            config.omniroute_base_url = non_empty(value, ConfigError::EmptyOmniRouteBaseUrl)?
                .trim_end_matches('/')
                .to_owned();
        }
        "MAKOSH_OMNIROUTE_CHAT_MODEL" => {
            config.omniroute_chat_model =
                non_empty(value, ConfigError::EmptyOmniRouteChatModel)?.to_owned();
        }
        "MAKOSH_OMNIROUTE_EMBED_MODEL" => {
            config.omniroute_embed_model =
                non_empty(value, ConfigError::EmptyOmniRouteEmbedModel)?.to_owned();
        }
        "MAKOSH_OMNIROUTE_TIMEOUT_SECONDS" => {
            config.omniroute_timeout_seconds =
                parse_positive_timeout(value, TimeoutTarget::OmniRoute)?;
        }
        "MAKOSH_OMNIROUTE_API_KEY" => {
            config.omniroute_api_key = Some(ResolvedSecret::new(non_empty(
                value,
                ConfigError::EmptyOmniRouteApiKey,
            )?)?);
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

#[derive(Clone, Copy)]
enum TimeoutTarget {
    Ollama,
    OmniRoute,
}

fn parse_positive_timeout(value: &str, target: TimeoutTarget) -> Result<u64, ConfigError> {
    let raw_timeout = value.trim();
    let timeout = raw_timeout.parse::<u64>().map_err(|source| {
        timeout_error(
            target,
            raw_timeout,
            "must be a positive integer",
            Some(source),
        )
    })?;
    if timeout == 0 {
        return Err(timeout_error(
            target,
            raw_timeout,
            "must be greater than zero",
            None,
        ));
    }
    Ok(timeout)
}

fn timeout_error(
    target: TimeoutTarget,
    value: &str,
    reason: &'static str,
    source: Option<std::num::ParseIntError>,
) -> ConfigError {
    match target {
        TimeoutTarget::Ollama => ConfigError::InvalidOllamaTimeout {
            value: value.to_owned(),
            reason,
            source,
        },
        TimeoutTarget::OmniRoute => ConfigError::InvalidOmniRouteTimeout {
            value: value.to_owned(),
            reason,
            source,
        },
    }
}
