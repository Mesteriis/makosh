use std::io;
use std::net::AddrParseError;
use std::num::ParseIntError;
use std::path::PathBuf;

use thiserror::Error;

use crate::platform::secrets::errors::SecretResolutionError;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid MAKOSH_HTTP_ADDR `{value}`: {source}")]
    InvalidHttpAddr {
        value: String,
        #[source]
        source: AddrParseError,
    },

    #[error("invalid MAKOSH_AI_PROVIDER `{value}`: expected ollama or omniroute")]
    InvalidAiProvider { value: String },

    #[error("DATABASE_URL is set but empty")]
    EmptyDatabaseUrl,

    #[error("MAKOSH_LOCAL_API_SECRET is set but empty")]
    EmptyLocalApiSecret,

    #[error("MAKOSH_NATS_SERVER_URL is set but empty")]
    EmptyNatsServerUrl,

    #[error("MAKOSH_SECRET_VAULT_PATH is set but empty")]
    EmptySecretVaultPath,

    #[error("MAKOSH_SECRET_VAULT_KEY is set but empty")]
    EmptySecretVaultKey,

    #[error("MAKOSH_VAULT_HOME is set but empty")]
    EmptyVaultHome,

    #[error("MAKOSH_DEV_KEY_PATH is set but empty")]
    EmptyDevKeyPath,

    #[error("invalid {name} `{value}`: expected true or false")]
    InvalidBoolEnv { name: &'static str, value: String },

    #[error("MAKOSH_TDJSON_PATH is set but empty")]
    EmptyTdjsonPath,

    #[error("invalid MAKOSH_TELEGRAM_API_ID `{value}`: {reason}")]
    InvalidTelegramApiId {
        value: String,
        reason: &'static str,
        #[source]
        source: Option<ParseIntError>,
    },

    #[error("MAKOSH_TELEGRAM_API_HASH is set but empty")]
    EmptyTelegramApiHash,

    #[error("MAKOSH_GOOGLE_OAUTH_CLIENT_ID is set but empty")]
    EmptyGoogleOAuthClientId,

    #[error("MAKOSH_GOOGLE_OAUTH_CLIENT_SECRET is set but empty")]
    EmptyGoogleOAuthClientSecret,

    #[error("MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON is set but empty")]
    EmptyGoogleOAuthClientConfigJson,

    #[error("MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH is set but empty")]
    EmptyGoogleOAuthClientConfigPath,

    #[error("failed to read MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH `{}`: {source}", path.display())]
    GoogleOAuthClientConfigRead {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("invalid Google OAuth client credentials JSON: {0}")]
    GoogleOAuthClientConfigJson(serde_json::Error),

    #[error("invalid Google OAuth client config field {field}: {message}")]
    InvalidGoogleOAuthClientConfig {
        field: &'static str,
        message: &'static str,
    },

    #[error("MAKOSH_OLLAMA_BASE_URL is set but empty")]
    EmptyOllamaBaseUrl,

    #[error("MAKOSH_OLLAMA_CHAT_MODEL is set but empty")]
    EmptyOllamaChatModel,

    #[error("MAKOSH_OLLAMA_EMBED_MODEL is set but empty")]
    EmptyOllamaEmbedModel,

    #[error("invalid MAKOSH_OLLAMA_TIMEOUT_SECONDS `{value}`: {reason}")]
    InvalidOllamaTimeout {
        value: String,
        reason: &'static str,
        #[source]
        source: Option<ParseIntError>,
    },

    #[error("MAKOSH_OMNIROUTE_BASE_URL is set but empty")]
    EmptyOmniRouteBaseUrl,

    #[error("MAKOSH_OMNIROUTE_CHAT_MODEL is set but empty")]
    EmptyOmniRouteChatModel,

    #[error("MAKOSH_OMNIROUTE_EMBED_MODEL is set but empty")]
    EmptyOmniRouteEmbedModel,

    #[error("MAKOSH_OMNIROUTE_API_KEY is set but empty")]
    EmptyOmniRouteApiKey,

    #[error("invalid MAKOSH_OMNIROUTE_TIMEOUT_SECONDS `{value}`: {reason}")]
    InvalidOmniRouteTimeout {
        value: String,
        reason: &'static str,
        #[source]
        source: Option<ParseIntError>,
    },

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),
}
