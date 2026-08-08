# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `070-source-backend-part-050`
- Group / Группа: `backend`
- Role / Роль: `source`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/backend.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/src/platform/config/app_config/env.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/app_config/env.rs`
- Size bytes / Размер в байтах: `1078`
- Included characters / Включено символов: `1078`
- Truncated / Обрезано: `no`

```rust
use std::env;

use super::super::errors::ConfigError;
use super::AppConfig;
use super::ai_env::apply_ai_env;
use super::core_env::apply_core_env;
use super::provider_env::{apply_bundled_google_oauth_client, apply_provider_env};

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_pairs(env::vars())
    }

    pub fn from_pairs<I, K, V>(pairs: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut config = Self::default();
        apply_bundled_google_oauth_client(&mut config)?;

        for (key, value) in pairs {
            apply_config_pair(&mut config, key.as_ref(), value.as_ref())?;
        }

        Ok(config)
    }
}

pub(super) fn apply_config_pair(
    config: &mut AppConfig,
    key: &str,
    value: &str,
) -> Result<(), ConfigError> {
    if apply_core_env(config, key, value)?
        || apply_provider_env(config, key, value)?
        || apply_ai_env(config, key, value)?
    {
        return Ok(());
    }

    Ok(())
}
```

### `backend/src/platform/config/app_config/provider_env.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/app_config/provider_env.rs`
- Size bytes / Размер в байтах: `4071`
- Included characters / Включено символов: `4071`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::PathBuf;

use crate::platform::secrets::ResolvedSecret;

use super::super::errors::ConfigError;
use super::super::google::GoogleOAuthClientConfig;
use super::super::parsing::parse_bool_env;
use super::AppConfig;

pub(super) fn apply_bundled_google_oauth_client(config: &mut AppConfig) -> Result<(), ConfigError> {
    if let Some(raw_json) = option_env!("MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON") {
        config.google_oauth_client =
            Some(GoogleOAuthClientConfig::from_client_secret_json(raw_json)?);
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
```

### `backend/src/platform/config/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/constants.rs`
- Size bytes / Размер в байтах: `729`
- Included characters / Включено символов: `729`
- Truncated / Обрезано: `no`

```rust
pub(crate) const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:8080";
pub(crate) const DEFAULT_SERVICE_NAME: &str = "makosh-backend";
pub(crate) const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
pub(crate) const DEFAULT_OLLAMA_CHAT_MODEL: &str = "qwen3:4b";
pub(crate) const DEFAULT_OLLAMA_EMBED_MODEL: &str = "qwen3-embedding:4b";
pub(crate) const DEFAULT_OLLAMA_TIMEOUT_SECONDS: u64 = 120;
pub(crate) const DEFAULT_OMNIROUTE_BASE_URL: &str = "https://ai.sh-inc.ru/v1";
pub(crate) const DEFAULT_OMNIROUTE_CHAT_MODEL: &str = "codex/gpt-5.5";
pub(crate) const DEFAULT_OMNIROUTE_EMBED_MODEL: &str =
    "openai-compatible-chat-ollama-pve/qwen3-embedding:4b";
pub(crate) const DEFAULT_OMNIROUTE_TIMEOUT_SECONDS: u64 = 120;
```

### `backend/src/platform/config/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/errors.rs`
- Size bytes / Размер в байтах: `3616`
- Included characters / Включено символов: `3616`
- Truncated / Обрезано: `no`

```rust
use std::io;
use std::net::AddrParseError;
use std::num::ParseIntError;
use std::path::PathBuf;

use thiserror::Error;

use crate::platform::secrets::SecretResolutionError;

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
```

### `backend/src/platform/config/google.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/google.rs`
- Size bytes / Размер в байтах: `3239`
- Included characters / Включено символов: `3239`
- Truncated / Обрезано: `no`

```rust
use serde::Deserialize;

use crate::platform::secrets::ResolvedSecret;

use super::errors::ConfigError;
use super::parsing::required_trimmed;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoogleOAuthClientType {
    Installed,
    Web,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoogleOAuthClientConfig {
    client_type: GoogleOAuthClientType,
    client_id: String,
    client_secret: Option<ResolvedSecret>,
    authorization_endpoint: String,
    token_endpoint: String,
    redirect_uris: Vec<String>,
}

impl GoogleOAuthClientConfig {
    pub(crate) fn from_client_secret_json(raw_json: &str) -> Result<Self, ConfigError> {
        let file: GoogleOAuthClientSecretsFile =
            serde_json::from_str(raw_json).map_err(ConfigError::GoogleOAuthClientConfigJson)?;
        if let Some(installed) = file.installed {
            return Self::from_payload(GoogleOAuthClientType::Installed, installed);
        }
        if let Some(web) = file.web {
            return Self::from_payload(GoogleOAuthClientType::Web, web);
        }

        Err(ConfigError::InvalidGoogleOAuthClientConfig {
            field: "client_type",
            message: "must contain installed or web client credentials",
        })
    }

    fn from_payload(
        client_type: GoogleOAuthClientType,
        payload: GoogleOAuthClientSecretsPayload,
    ) -> Result<Self, ConfigError> {
        let client_id = required_trimmed("client_id", payload.client_id)?;
        let authorization_endpoint = required_trimmed("auth_uri", payload.auth_uri)?;
        let token_endpoint = required_trimmed("token_uri", payload.token_uri)?;
        let client_secret = payload
            .client_secret
            .map(|secret| required_trimmed("client_secret", Some(secret)))
            .transpose()?
            .map(ResolvedSecret::new)
            .transpose()?;
        let redirect_uris = payload
            .redirect_uris
            .unwrap_or_default()
            .into_iter()
            .map(|uri| required_trimmed("redirect_uris", Some(uri)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            client_type,
            client_id,
            client_secret,
            authorization_endpoint,
            token_endpoint,
            redirect_uris,
        })
    }

    pub fn client_type(&self) -> GoogleOAuthClientType {
        self.client_type
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn client_secret(&self) -> Option<&ResolvedSecret> {
        self.client_secret.as_ref()
    }

    pub fn authorization_endpoint(&self) -> &str {
        &self.authorization_endpoint
    }

    pub fn token_endpoint(&self) -> &str {
        &self.token_endpoint
    }

    pub fn redirect_uris(&self) -> &[String] {
        &self.redirect_uris
    }
}

#[derive(Debug, Deserialize)]
struct GoogleOAuthClientSecretsFile {
    installed: Option<GoogleOAuthClientSecretsPayload>,
    web: Option<GoogleOAuthClientSecretsPayload>,
}

#[derive(Debug, Deserialize)]
struct GoogleOAuthClientSecretsPayload {
    client_id: Option<String>,
    client_secret: Option<String>,
    auth_uri: Option<String>,
    token_uri: Option<String>,
    redirect_uris: Option<Vec<String>>,
}
```

### `backend/src/platform/config/parsing.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/config/parsing.rs`
- Size bytes / Размер в байтах: `895`
- Included characters / Включено символов: `895`
- Truncated / Обрезано: `no`

```rust
use super::errors::ConfigError;

pub(crate) fn required_trimmed(
    field: &'static str,
    value: Option<String>,
) -> Result<String, ConfigError> {
    let Some(value) = value else {
        return Err(ConfigError::InvalidGoogleOAuthClientConfig {
            field,
            message: "must be present",
        });
    };
    let value = value.trim();
    if value.is_empty() {
        return Err(ConfigError::InvalidGoogleOAuthClientConfig {
            field,
            message: "must not be empty",
        });
    }
    Ok(value.to_owned())
}

pub(crate) fn parse_bool_env(name: &'static str, value: &str) -> Result<bool, ConfigError> {
    match value {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        other => Err(ConfigError::InvalidBoolEnv {
            name,
            value: other.to_owned(),
        }),
    }
}
```

### `backend/src/platform/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events.rs`
- Size bytes / Размер в байтах: `1473`
- Included characters / Включено символов: `1473`
- Truncated / Обрезано: `no`

```rust
mod builder;
pub mod bus;
mod consumers;
mod cursors;
mod dispatcher;
mod errors;
mod migrations;
mod models;
mod nats;
mod query;
mod rows;
mod runtime;
mod store;
mod trace;
mod trace_context;
mod validation;

pub use self::builder::NewEventEnvelopeBuilder;
pub use self::bus::{EventBus, InMemoryEventBus};
pub use self::consumers::{
    EventConsumerConfig, EventConsumerRunReport, EventConsumerRunner, EventConsumerStore,
    EventDeadLetter, EventDeadLetterReviewState,
};
pub use self::cursors::ProjectionCursorStore;
pub use self::dispatcher::{EventDispatchReport, EventDispatcherError, EventOutboxDispatcher};
pub use self::errors::EventStoreError as EventLogPortError;
pub use self::errors::{EventEnvelopeError, EventStoreError};
pub use self::migrations::{MigrationSummary, expected_migration_summary, run_migrations};
pub use self::models::{
    DispatchableEventOutboxItem, EventEnvelope, EventOutboxItem, NewEventEnvelope,
    StoredEventEnvelope,
};
pub use self::nats::{NatsJetStreamEventBus, NatsJetStreamEventBusError};
pub use self::query::EventLogQuery;
pub use self::runtime::{
    ensure_runtime_processing_state, runtime_allows_processing, runtime_state_allows_processing,
    source_runtime_state_from_policies,
};
pub use self::store::EventStore;
pub use self::store::EventStore as EventLogPort;
pub use self::trace::{
    EventConsumerAnnotation, EventDeadLetterAnnotation, EventTrace, EventTraceEdge,
};
pub use self::trace_context::TraceContext;
```

### `backend/src/platform/events/builder.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/builder.rs`
- Size bytes / Размер в байтах: `2819`
- Included characters / Включено символов: `2819`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::Value;

use super::errors::EventEnvelopeError;
use super::models::NewEventEnvelope;
use super::validation::{validate_non_empty, validate_object};

pub struct NewEventEnvelopeBuilder {
    pub(super) event_id: String,
    pub(super) event_type: String,
    pub(super) schema_version: i32,
    pub(super) occurred_at: DateTime<Utc>,
    pub(super) source: Value,
    pub(super) actor: Option<Value>,
    pub(super) subject: Value,
    pub(super) payload: Value,
    pub(super) provenance: Value,
    pub(super) causation_id: Option<String>,
    pub(super) correlation_id: Option<String>,
}

impl NewEventEnvelopeBuilder {
    pub fn schema_version(mut self, schema_version: i32) -> Self {
        self.schema_version = schema_version;
        self
    }

    pub fn actor(mut self, actor: Value) -> Self {
        self.actor = Some(actor);
        self
    }

    pub fn payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }

    pub fn provenance(mut self, provenance: Value) -> Self {
        self.provenance = provenance;
        self
    }

    pub fn correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn causation_id(mut self, causation_id: impl Into<String>) -> Self {
        self.causation_id = Some(causation_id.into());
        self
    }

    pub fn build(self) -> Result<NewEventEnvelope, EventEnvelopeError> {
        validate_non_empty("event_id", &self.event_id)?;
        validate_non_empty("event_type", &self.event_type)?;

        if self.schema_version <= 0 {
            return Err(EventEnvelopeError::InvalidSchemaVersion);
        }

        validate_object("source", &self.source)?;
        validate_object("subject", &self.subject)?;
        validate_object("payload", &self.payload)?;
        validate_object("provenance", &self.provenance)?;

        if let Some(actor) = &self.actor {
            validate_object("actor", actor)?;
        }

        let event_id = self.event_id.trim().to_owned();
        let correlation_id = self
            .correlation_id
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| event_id.clone());

        Ok(NewEventEnvelope {
            event_id,
            event_type: self.event_type.trim().to_owned(),
            schema_version: self.schema_version,
            occurred_at: self.occurred_at,
            source: self.source,
            actor: self.actor,
            subject: self.subject,
            payload: self.payload,
            provenance: self.provenance,
            causation_id: self.causation_id,
            correlation_id: Some(correlation_id),
        })
    }
}
```

### `backend/src/platform/events/bus.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/bus.rs`
- Size bytes / Размер в байтах: `8632`
- Included characters / Включено символов: `8632`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::broadcast;

use super::models::{EventEnvelope, NewEventEnvelope};

/// Max events in the broadcast ring buffer before oldest are dropped.
const BUS_CAPACITY: usize = 4096;

pub type EventBus = InMemoryEventBus;

/// Shared event bus for realtime dispatch.
#[derive(Clone)]
pub struct InMemoryEventBus {
    tx: broadcast::Sender<Arc<NewEventEnvelope>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BUS_CAPACITY);
        Self { tx }
    }

    pub fn broadcast(&self, event: NewEventEnvelope) -> usize {
        self.tx.send(Arc::new(event)).unwrap_or(0)
    }

    pub fn broadcast_stored(&self, event: &EventEnvelope) -> usize {
        self.broadcast(NewEventEnvelope {
            event_id: event.event_id.clone(),
            event_type: event.event_type.clone(),
            schema_version: event.schema_version,
            occurred_at: event.occurred_at,
            source: event.source.clone(),
            actor: event.actor.clone(),
            subject: event.subject.clone(),
            payload: event.payload.clone(),
            provenance: event.provenance.clone(),
            causation_id: event.causation_id.clone(),
            correlation_id: event.correlation_id.clone(),
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<NewEventEnvelope>> {
        self.tx.subscribe()
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Telegram-specific event type constants (ADR-0091)
// ---------------------------------------------------------------------------

pub mod telegram_event_types {
    pub const SYNC_STARTED: &str = "telegram.sync.started";
    pub const SYNC_PROGRESS: &str = "telegram.sync.progress";
    pub const SYNC_COMPLETED: &str = "telegram.sync.completed";
    pub const SYNC_FAILED: &str = "telegram.sync.failed";

    pub const MESSAGE_CREATED: &str = "telegram.message.created";
    pub const MESSAGE_UPDATED: &str = "telegram.message.updated";
    pub const MESSAGE_DELETED: &str = "telegram.message.deleted";
    pub const MESSAGE_TOMBSTONED: &str = "telegram.message.tombstoned";
    pub const MESSAGE_VISIBILITY_RESTORED: &str = "telegram.message.visibility_restored";

    pub const REACTION_CHANGED: &str = "telegram.reaction.changed";

    pub const CHAT_UPDATED: &str = "telegram.chat.updated";
    pub const CHAT_PINNED: &str = "telegram.chat.pinned";
    pub const CHAT_ARCHIVED: &str = "telegram.chat.archived";
    pub const CHAT_MUTED: &str = "telegram.chat.muted";
    pub const FOLDERS_UPDATED: &str = "telegram.folders.updated";

    pub const TYPING_CHANGED: &str = "telegram.typing.changed";

    pub const TOPIC_UPDATED: &str = "telegram.topic.updated";

    pub const PARTICIPANT_UPDATED: &str = "telegram.participant.updated";

    pub const MEDIA_DOWNLOAD_STARTED: &str = "telegram.media.download.started";
    pub const MEDIA_DOWNLOAD_PROGRESS: &str = "telegram.media.download.progress";
    pub const MEDIA_DOWNLOAD_FAILED: &str = "telegram.media.download.failed";
    pub const MEDIA_DOWNLOADED: &str = "telegram.media.downloaded";
    pub const MEDIA_UPLOAD_STARTED: &str = "telegram.media.upload.started";
    pub const MEDIA_UPLOAD_PROGRESS: &str = "telegram.media.upload.progress";
    pub const MEDIA_UPLOAD_FAILED: &str = "telegram.media.upload.failed";
    pub const MEDIA_UPLOAD_COMPLETED: &str = "telegram.media.upload.completed";

    pub const COMMAND_STATUS_CHANGED: &str = "telegram.command.status_changed";
    pub const COMMAND_RECONCILED: &str = "telegram.command.reconciled";
}

pub mod whatsapp_event_types {
    pub const SYNC_STARTED: &str = "whatsapp.sync.started";
    pub const SYNC_PROGRESS: &str = "whatsapp.sync.progress";
    pub const SYNC_COMPLETED: &str = "whatsapp.sync.completed";
    pub const SYNC_FAILED: &str = "whatsapp.sync.failed";
    pub const RUNTIME_STATUS_CHANGED: &str = "whatsapp.runtime.status_changed";
    pub const RUNTIME_EVENT: &str = "whatsapp.runtime.event";
    pub const SESSION_LINK_STATE_CHANGED: &str = "whatsapp.session.link_state_changed";
    pub const DIALOG_UPDATED: &str = "whatsapp.dialog.updated";
    pub const MESSAGE_CREATED: &str = "whatsapp.message.created";
    pub const MESSAGE_UPDATED: &str = "whatsapp.message.updated";
    pub const MESSAGE_DELETED: &str = "whatsapp.message.deleted";
    pub const REACTION_CHANGED: &str = "whatsapp.reaction.changed";
    pub const RECEIPT_CHANGED: &str = "whatsapp.receipt.changed";
    pub const PARTICIPANT_CHANGED: &str = "whatsapp.participant.changed";
    pub const PRESENCE_CHANGED: &str = "whatsapp.presence.changed";
    pub const CALL_UPDATED: &str = "whatsapp.call.updated";
    pub const STATUS_UPDATED: &str = "whatsapp.status.updated";
    pub const STATUS_DELETED: &str = "whatsapp.status.deleted";
    pub const COMMAND_STATUS_CHANGED: &str = "whatsapp.command.status_changed";
    pub const COMMAND_RECONCILED: &str = "whatsapp.command.reconciled";
    pub const MEDIA_DOWNLOAD_REQUESTED: &str = "whatsapp.media.download.requested";
    pub const MEDIA_DOWNLOAD_STARTED: &str = "whatsapp.media.download.started";
    pub const MEDIA_DOWNLOAD_PROGRESS: &str = "whatsapp.media.download.progress";
    pub const MEDIA_DOWNLOAD_COMPLETED: &str = "whatsapp.media.download.completed";
    pub const MEDIA_DOWNLOAD_FAILED: &str = "whatsapp.media.download.failed";
    pub const MEDIA_UPLOAD_REQUESTED: &str = "whatsapp.media.upload.requested";
    pub const MEDIA_UPLOAD_STARTED: &str = "whatsapp.media.upload.started";
    pub const MEDIA_UPLOAD_PROGRESS: &str = "whatsapp.media.upload.progress";
    pub const MEDIA_UPLOAD_COMPLETED: &str = "whatsapp.media.upload.completed";
    pub const MEDIA_UPLOAD_FAILED: &str = "whatsapp.media.upload.failed";
}

pub mod zoom_event_types {
    pub const AUTHORIZATION_COMPLETED: &str = "zoom.authorization.completed";
    pub const RUNTIME_STATUS_CHANGED: &str = "zoom.runtime.status_changed";
    pub const TOKEN_REFRESHED: &str = "zoom.token.refreshed";
    pub const TOKEN_REFRESH_SKIPPED: &str = "zoom.token.refresh.skipped";
    pub const TOKEN_REFRESH_FAILED: &str = "zoom.token.refresh.failed";
    pub const MEETING_OBSERVED: &str = "zoom.meeting.observed";
    pub const RECORDING_OBSERVED: &str = "zoom.recording.observed";
    pub const TRANSCRIPT_OBSERVED: &str = "zoom.transcript.observed";
    pub const TRANSCRIPT_REMOVED: &str = "zoom.transcript.removed";
    pub const RECORDING_IMPORT_REMOVED: &str = "zoom.recording.import.removed";
    pub const RETENTION_CLEANUP_COMPLETED: &str = "zoom.retention.cleanup.completed";
}

pub mod yandex_telemost_event_types {
    pub const ACCOUNT_CONFIGURED: &str = "integration.yandex_telemost.account.configured";
    pub const AUTHORIZATION_COMPLETED: &str = "integration.yandex_telemost.authorization.completed";
    pub const RUNTIME_STATUS_CHANGED: &str = "integration.yandex_telemost.runtime.status_changed";
    pub const CONFERENCE_CREATED: &str = "integration.yandex_telemost.conference.created";
    pub const CONFERENCE_OBSERVED: &str = "integration.yandex_telemost.conference.observed";
    pub const CONFERENCE_UPDATED: &str = "integration.yandex_telemost.conference.updated";
    pub const COHOSTS_OBSERVED: &str = "integration.yandex_telemost.cohosts.observed";
    pub const WEBVIEW_OPEN_REQUESTED: &str = "integration.yandex_telemost.webview.open_requested";
    pub const SPEAKER_HINT_OBSERVED: &str = "integration.yandex_telemost.speaker_hint.observed";
    pub const LOCAL_CAPTURE_OBSERVED: &str = "integration.yandex_telemost.local_capture.observed";
    pub const LOCAL_RECORDING_REQUESTED: &str =
        "integration.yandex_telemost.local_recording.requested";
    pub const LOCAL_RECORDING_COMPLETED: &str =
        "integration.yandex_telemost.local_recording.completed";
    pub const RETENTION_CLEANUP_COMPLETED: &str =
        "integration.yandex_telemost.retention.cleanup.completed";
}

/// Sanitize an event payload to never include secrets or raw message bodies.
pub fn sanitize_event_payload(mut payload: Value) -> Value {
    if let Some(obj) = payload.as_object_mut() {
        obj.remove("raw_body");
        obj.remove("tdlib_raw");
        obj.remove("access_token");
        obj.remove("api_hash");
        obj.remove("session_key");
        obj.remove("bot_token");
        obj.remove("proxy_password");
        obj.remove("password");
    }
    payload
}
```

### `backend/src/platform/events/consumers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/consumers.rs`
- Size bytes / Размер в байтах: `27620`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::future::Future;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgPool;

use super::errors::EventStoreError;
use super::models::StoredEventEnvelope;
use super::store::EventStore;
use super::validation::validate_non_empty;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventConsumerConfig {
    pub consumer_name: String,
    pub batch_size: u32,
    pub max_attempts: i32,
    pub retry_base_seconds: i64,
}

impl EventConsumerConfig {
    pub fn new(consumer_name: impl Into<String>) -> Self {
        Self {
            consumer_name: consumer_name.into(),
            batch_size: 100,
            max_attempts: 5,
            retry_base_seconds: 30,
        }
    }

    fn normalized(&self) -> Self {
        Self {
            consumer_name: self.consumer_name.trim().to_owned(),
            batch_size: self.batch_size.clamp(1, 1000),
            max_attempts: self.max_attempts.max(1),
            retry_base_seconds: self.retry_base_seconds.max(0),
        }
    }
}

#[derive(Clone)]
pub struct EventConsumerRunner {
    event_store: EventStore,
    consumer_store: EventConsumerStore,
    config: EventConsumerConfig,
}

impl EventConsumerRunner {
    pub fn new(pool: PgPool, config: EventConsumerConfig) -> Self {
        Self {
            event_store: EventStore::new(pool.clone()),
            consumer_store: EventConsumerStore::new(pool),
            config: config.normalized(),
        }
    }

    pub async fn process_next_batch<F, Fut>(
        &self,
        mut handler: F,
    ) -> Result<EventConsumerRunReport, EventStoreError>
    where
        F: FnMut(StoredEventEnvelope) -> Fut,
        Fut: Future<Output = Result<(), EventStoreError>>,
    {
        validate_non_empty("consumer_name", &self.config.consumer_name)?;
        self.consumer_store
            .ensure_consumer(&self.config.consumer_name)
            .await?;

        let mut report = EventConsumerRunReport::default();
        let cursor = self
            .consumer_store
            .last_processed_position(&self.config.consumer_name)
            .await?;
        let events = self
            .event_store
            .list_after_position(cursor, self.config.batch_size)
            .await?;

        for event in events {
            let now = Utc::now();
            if self
                .consumer_store
                .has_processed_event(&self.config.consumer_name, event.position)
                .await?
            {
                self.consumer_store
                    .clear_failure(&self.config.consumer_name, event.position)
                    .await?;
                self.consumer_store
                    .save_position(&self.config.consumer_name, event.position)
                    .await?;
                report.skipped_duplicates += 1;
                report.last_processed_position = event.position;
                continue;
            }

            if self
                .consumer_store
                .next_attempt_at(&self.config.consumer_name, event.position)
                .await?
                .is_some_and(|next_attempt_at| next_attempt_at > now)
            {
                report.pending_retry = true;
                break;
            }

            match handler(event.clone()).await {
                Ok(()) => {
                    self.consumer_store
                        .record_processed(&self.config.consumer_name, &event)
                        .await?;
                    self.consumer_store
                        .mark_dead_letter_replayed_for_event(
                            &self.config.consumer_name,
                            event.position,
                        )
                        .await?;
                    self.consumer_store
                        .clear_failure(&self.config.consumer_name, event.position)
                        .await?;
                    self.consumer_store
                        .save_position(&self.config.consumer_name, event.position)
                        .await?;
                    report.processed += 1;
                    report.last_processed_position = event.position;
                }
                Err(error) => {
                    let error_message = error.to_string();
                    let attempt_count = self
                        .consumer_store
                        .record_failure(
                            &self.config.consumer_name,
                            &event,
                            &error_message,
                            next_retry_at(now, self.config.retry_base_seconds, 1),
                        )
                        .await?;
                    report.failed += 1;

                    if attempt_count >= self.config.max_attempts {
                        self.consumer_store
                            .dead_letter(
                                &self.config.consumer_name,
                                &event,
                                attempt_count,
                                &error_message,
                            )
                            .await?;
                        self.consumer_store
                            .save_position(&self.config.consumer_name, event.position)
                            .await?;
                        report.dead_lettered += 1;
                        report.last_processed_position = event.position;
                        continue;
                    }

                    let retry_at =
                        next_retry_at(now, self.config.retry_base_seconds, attempt_count);
                    self.consumer_store
                        .update_next_attempt(&self.config.consumer_name, event.position, retry_at)
                        .await?;
                    break;
                }
            }
        }

        Ok(report)
    }

    pub async fn replay_dead_letter<F, Fut>(
        &self,
        dead_letter_id: &str,
        handler: F,
    ) -> Result<(), EventStoreError>
    where
        F: FnOnce(StoredEventEnvelope) -> Fut,
        Fut: Future<Output = Result<(), EventStoreError>>,
    {
        validate_non_empty("dead_letter_id", dead_letter_id)?;
        let dead_letter = self
            .consumer_store
            .dead_letter_by_id(dead_letter_id)
            .await?;
        if dead_letter.review_state != EventDeadLetterReviewState::ReplayRequested {
            return Err(EventStoreError::DeadLetterNotReplayRequested(
                dead_letter_id.to_owned(),
            ));
        }

        handler(dead_letter.event.clone()).await?;
        self.consumer_store
            .record_processed(&dead_letter.consumer_name, &dead_letter.event)
            .await?;
        self.consumer_store
            .mark_dead_letter_replayed(dead_letter_id)
            .await?;
        self.consumer_store
            .clear_failure(&dead_letter.consumer_name, dead_letter.event_position)
            .await?;

        Ok(())
    }

    pub fn store(&self) -> &EventConsumerStore {
        &self.consumer_store
    }
}

#[derive(Clone)]
pub struct EventConsumerStore {
    pool: PgPool,
}

impl EventConsumerStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn ensure_consumer(&self, consumer_name: &str) -> Result<(), EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        sqlx::query(
            r#"
            INSERT INTO event_consumers (consumer_name, updated_at)
            VALUES ($1, now())
            ON CONFLICT (consumer_name)
            DO NOTHING
            "#,
        )
        .bind(consumer_name.trim())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn last_processed_position(
        &self,
        consumer_name: &str,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        let position = sqlx::query_scalar::<_, Option<i64>>(
            r#"
            SELECT last_processed_position
            FROM event_consumers
            WHERE consumer_name = $1
            "#,
        )
        .bind(consumer_name.trim())
        .fetch_optional(&self.pool)
        .await?;

        Ok(position.flatten().unwrap_or(0))
    }

    pub async fn save_position(
        &self,
        consumer_name: &str,
        position: i64,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        if position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(position));
        }

        let saved_position = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO event_consumers (
                consumer_name,
                last_processed_position,
                updated_at
            )
            VALUES ($1, $2, now())
            ON CONFLICT (consumer_name)
            DO UPDATE SET
                last_processed_position = GREATEST(
                    event_consumers.last_processed_position,
                    EXCLUDED.last_processed_position
                ),
                updated_at = now()
            RETURNING last_processed_position
            "#,
        )
        .bind(consumer_name.trim())
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(saved_position)
    }

    pub async fn rewind_position(
        &self,
        consumer_name: &str,
        position: i64,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        if position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(position));
        }
        self.ensure_consumer(consumer_name).await?;

        let saved_position = sqlx::query_scalar::<_, i64>(
            r#"
            UPDATE event_consumers
            SET
                last_processed_position = LEAST(
                    COALESCE(last_processed_position, $2),
                    $2
                ),
                updated_at = now()
            WHERE consumer_name = $1
            RETURNING COALESCE(last_processed_position, 0)
            "#,
        )
        .bind(consumer_name.trim())
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(saved_position)
    }

    pub async fn next_attempt_at(
        &self,
        consumer_name: &str,
        event_position: i64,
    ) -> Result<Option<DateTime<Utc>>, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        let next_attempt_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            SELECT next_attempt_at
            FROM event_consumer_failures
            WHERE consumer_name = $1 AND event_position = $2
            "#,
        )
        .bind(consumer_name.trim())
        .bind(event_position)
        .fetch_optional(&self.pool)
        .await?;

        Ok(next_attempt_at.flatten())
    }

    pub async fn record_failure(
        &self,
        consumer_name: &str,
        event: &StoredEventEnvelope,
        error_message: &str,
        next_attempt_at: DateTime<Utc>,
    ) -> Result<i32, EventStoreError> {
        validate_non_empty("consumer_name", consumer_name)?;
        validate_non_empty("error_message", error_message)?;
        self.ensure_consumer(consumer_name).await?;

        let attempt_count = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO event_consumer_failures (
                consumer_name,
                event_position,
                event_id,
                event_type,
                attempt_count,
                next_attempt_at,
                last_attempt_at,
                last_error,
                updated_at
            )
            VALUES ($1, $2, $3, $4, 1, $5, now(), $6, now())
            ON CONFLICT (consumer_name, event_position)
            DO UPDATE SET
                attempt_count = event_consumer_failures.attempt_count + 1,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/platform/events/cursors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/cursors.rs`
- Size bytes / Размер в байтах: `2918`
- Included characters / Включено символов: `2918`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::errors::EventStoreError;
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct ProjectionCursorStore {
    pool: PgPool,
}

impl ProjectionCursorStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn last_processed_position(
        &self,
        projection_name: &str,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("projection_name", projection_name)?;

        let position = sqlx::query_scalar::<_, Option<i64>>(
            r#"
            SELECT last_processed_position
            FROM projection_cursors
            WHERE projection_name = $1
            "#,
        )
        .bind(projection_name.trim())
        .fetch_optional(&self.pool)
        .await?;

        Ok(position.flatten().unwrap_or(0))
    }

    pub async fn save_position(
        &self,
        projection_name: &str,
        position: i64,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("projection_name", projection_name)?;
        if position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(position));
        }

        let saved_position = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO projection_cursors (
                projection_name,
                last_processed_position,
                updated_at
            )
            VALUES ($1, $2, now())
            ON CONFLICT (projection_name)
            DO UPDATE SET
                last_processed_position = GREATEST(
                    projection_cursors.last_processed_position,
                    EXCLUDED.last_processed_position
                ),
                updated_at = now()
            RETURNING last_processed_position
            "#,
        )
        .bind(projection_name.trim())
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(saved_position)
    }

    pub async fn rewind_position(
        &self,
        projection_name: &str,
        position: i64,
    ) -> Result<i64, EventStoreError> {
        validate_non_empty("projection_name", projection_name)?;
        if position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(position));
        }

        let saved_position = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO projection_cursors (
                projection_name,
                last_processed_position,
                updated_at
            )
            VALUES ($1, $2, now())
            ON CONFLICT (projection_name)
            DO UPDATE SET
                last_processed_position = EXCLUDED.last_processed_position,
                updated_at = now()
            RETURNING last_processed_position
            "#,
        )
        .bind(projection_name.trim())
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(saved_position)
    }
}
```

### `backend/src/platform/events/dispatcher.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/dispatcher.rs`
- Size bytes / Размер в байтах: `3779`
- Included characters / Включено символов: `3779`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Duration, Utc};

use super::errors::EventStoreError;
use super::models::DispatchableEventOutboxItem;
use super::{EventBus, EventStore, NatsJetStreamEventBus, NatsJetStreamEventBusError};

const DEFAULT_DISPATCH_BATCH_SIZE: u32 = 100;
const DEFAULT_STALE_DISPATCH_AFTER_SECONDS: i64 = 60;

#[derive(Clone)]
pub struct EventOutboxDispatcher {
    store: EventStore,
    bus: NatsJetStreamEventBus,
    realtime_bus: Option<EventBus>,
    batch_size: u32,
    stale_dispatch_after: Duration,
}

impl EventOutboxDispatcher {
    pub fn new(store: EventStore, bus: NatsJetStreamEventBus) -> Self {
        Self {
            store,
            bus,
            realtime_bus: None,
            batch_size: DEFAULT_DISPATCH_BATCH_SIZE,
            stale_dispatch_after: Duration::seconds(DEFAULT_STALE_DISPATCH_AFTER_SECONDS),
        }
    }

    pub fn with_realtime_bus(mut self, realtime_bus: EventBus) -> Self {
        self.realtime_bus = Some(realtime_bus);
        self
    }

    pub fn with_batch_size(mut self, batch_size: u32) -> Self {
        self.batch_size = batch_size.clamp(1, 1000);
        self
    }

    pub fn with_stale_dispatch_after(mut self, stale_dispatch_after: Duration) -> Self {
        self.stale_dispatch_after = stale_dispatch_after;
        self
    }

    pub async fn dispatch_pending_once(&self) -> Result<EventDispatchReport, EventDispatcherError> {
        let recovered = self
            .store
            .recover_stale_outbox_items(self.stale_dispatch_after)
            .await?;
        let items = self
            .store
            .claim_pending_outbox_batch(self.batch_size)
            .await?;

        let mut report = EventDispatchReport {
            recovered,
            claimed: u32::try_from(items.len()).unwrap_or(u32::MAX),
            ..EventDispatchReport::default()
        };

        for item in items {
            if let Err(error) = self.dispatch_item(&item).await {
                report.retried += 1;
                tracing::warn!(
                    event_id = %item.event_id,
                    subject = %item.subject,
                    error = %error,
                    "event outbox dispatch failed"
                );
            } else {
                report.published += 1;
            }
        }

        Ok(report)
    }

    async fn dispatch_item(
        &self,
        item: &DispatchableEventOutboxItem,
    ) -> Result<(), EventDispatcherError> {
        match self.bus.publish(&item.event).await {
            Ok(()) => {
                self.store.mark_outbox_published(&item.event_id).await?;
                if let Some(realtime_bus) = &self.realtime_bus {
                    let _ = realtime_bus.broadcast_stored(&item.event);
                }
                Ok(())
            }
            Err(error) => {
                let next_attempt_at = next_attempt_at(item.attempts);
                self.store
                    .mark_outbox_retry(&item.event_id, &error.to_string(), next_attempt_at)
                    .await?;
                Err(error.into())
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct EventDispatchReport {
    pub recovered: u32,
    pub claimed: u32,
    pub published: u32,
    pub retried: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum EventDispatcherError {
    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Nats(#[from] NatsJetStreamEventBusError),
}

fn next_attempt_at(attempts: i32) -> DateTime<Utc> {
    let exponent = u32::try_from(attempts.saturating_sub(1))
        .unwrap_or(0)
        .min(6);
    let delay_seconds = (5_i64 * 2_i64.pow(exponent)).min(300);
    Utc::now() + Duration::seconds(delay_seconds)
}
```

### `backend/src/platform/events/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/errors.rs`
- Size bytes / Размер в байтах: `1322`
- Included characters / Включено символов: `1322`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventEnvelopeError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("schema_version must be positive")]
    InvalidSchemaVersion,

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error(transparent)]
    Envelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("replay position must be non-negative, got {0}")]
    InvalidReplayPosition(i64),

    #[error("event handler failed: {0}")]
    ConsumerHandlerFailed(String),

    #[error("event dead letter was not found: {0}")]
    DeadLetterNotFound(String),

    #[error("event dead letter is not replay-requested: {0}")]
    DeadLetterNotReplayRequested(String),

    #[error("invalid event dead letter review state: {0}")]
    InvalidDeadLetterReviewState(String),
}

impl EventStoreError {
    pub fn is_unique_violation(&self) -> bool {
        match self {
            Self::Sqlx(sqlx::Error::Database(error)) => error.code().as_deref() == Some("23505"),
            _ => false,
        }
    }
}
```

### `backend/src/platform/events/migrations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/migrations.rs`
- Size bytes / Размер в байтах: `823`
- Included characters / Включено символов: `823`
- Truncated / Обрезано: `no`

```rust
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPool;

use super::errors::EventStoreError;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn run_migrations(pool: &PgPool) -> Result<(), EventStoreError> {
    // Keep this wrapper explicit so compile-time migration bundle file changes remain easy to validate.
    MIGRATOR.run(pool).await?;
    Ok(())
}

pub fn expected_migration_summary() -> MigrationSummary {
    let mut count = 0;
    let mut latest_version = 0;

    for migration in MIGRATOR.iter() {
        count += 1;
        latest_version = latest_version.max(migration.version);
    }

    MigrationSummary {
        count,
        latest_version,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrationSummary {
    pub count: i64,
    pub latest_version: i64,
}
```

### `backend/src/platform/events/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/models.rs`
- Size bytes / Размер в байтах: `2178`
- Included characters / Включено символов: `2178`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::builder::NewEventEnvelopeBuilder;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: i32,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub source: Value,
    pub actor: Option<Value>,
    pub subject: Value,
    pub payload: Value,
    pub provenance: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StoredEventEnvelope {
    pub position: i64,
    pub event: EventEnvelope,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventOutboxItem {
    pub event_id: String,
    pub subject: String,
    pub status: String,
    pub attempts: i32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DispatchableEventOutboxItem {
    pub event_id: String,
    pub subject: String,
    pub attempts: i32,
    pub event: EventEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewEventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: i32,
    pub occurred_at: DateTime<Utc>,
    pub source: Value,
    pub actor: Option<Value>,
    pub subject: Value,
    pub payload: Value,
    pub provenance: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl NewEventEnvelope {
    pub fn builder(
        event_id: impl Into<String>,
        event_type: impl Into<String>,
        occurred_at: DateTime<Utc>,
        source: Value,
        subject: Value,
    ) -> NewEventEnvelopeBuilder {
        NewEventEnvelopeBuilder {
            event_id: event_id.into(),
            event_type: event_type.into(),
            schema_version: 1,
            occurred_at,
            source,
            actor: None,
            subject,
            payload: json!({}),
            provenance: json!({}),
            causation_id: None,
            correlation_id: None,
        }
    }
}
```

### `backend/src/platform/events/nats.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/nats.rs`
- Size bytes / Размер в байтах: `1971`
- Included characters / Включено символов: `1971`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use super::models::EventEnvelope;

const MAKOSH_EVENTS_STREAM: &str = "makosh_events";

#[derive(Clone)]
pub struct NatsJetStreamEventBus {
    context: async_nats::jetstream::Context,
}

impl NatsJetStreamEventBus {
    pub async fn connect(server_url: &str) -> Result<Self, NatsJetStreamEventBusError> {
        let client = async_nats::connect(server_url)
            .await
            .map_err(|error| NatsJetStreamEventBusError::Connect(error.to_string()))?;
        let context = async_nats::jetstream::new(client);
        context
            .get_or_create_stream(async_nats::jetstream::stream::Config {
                name: MAKOSH_EVENTS_STREAM.to_owned(),
                subjects: vec!["signal.>".to_owned()],
                ..Default::default()
            })
            .await
            .map_err(|error| NatsJetStreamEventBusError::Stream(error.to_string()))?;

        Ok(Self { context })
    }

    pub async fn publish(&self, event: &EventEnvelope) -> Result<(), NatsJetStreamEventBusError> {
        let payload = serde_json::to_vec(event)?;
        let ack = self
            .context
            .publish(event_subject(event), payload.into())
            .await
            .map_err(|error| NatsJetStreamEventBusError::Publish(error.to_string()))?;
        ack.await
            .map_err(|error| NatsJetStreamEventBusError::PublishAck(error.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum NatsJetStreamEventBusError {
    #[error("failed to connect to NATS JetStream: {0}")]
    Connect(String),

    #[error("failed to ensure JetStream event stream: {0}")]
    Stream(String),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("failed to publish NATS JetStream event: {0}")]
    Publish(String),

    #[error("failed to confirm NATS JetStream publish: {0}")]
    PublishAck(String),
}

fn event_subject(event: &EventEnvelope) -> String {
    event.event_type.clone()
}
```

### `backend/src/platform/events/query.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/query.rs`
- Size bytes / Размер в байтах: `2366`
- Included characters / Включено символов: `2366`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventLogQuery {
    pub event_type: Option<String>,
    pub source_code: Option<String>,
    pub subject_kind: Option<String>,
    pub subject_entity_id: Option<String>,
    pub correlation_id: Option<String>,
    pub position_after: Option<i64>,
    pub position_before: Option<i64>,
    pub occurred_after: Option<DateTime<Utc>>,
    pub occurred_before: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}

impl EventLogQuery {
    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = some_non_empty(event_type);
        self
    }

    pub fn source_code(mut self, source_code: impl Into<String>) -> Self {
        self.source_code = some_non_empty(source_code);
        self
    }

    pub fn subject_kind(mut self, subject_kind: impl Into<String>) -> Self {
        self.subject_kind = some_non_empty(subject_kind);
        self
    }

    pub fn subject_entity_id(mut self, subject_entity_id: impl Into<String>) -> Self {
        self.subject_entity_id = some_non_empty(subject_entity_id);
        self
    }

    pub fn correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = some_non_empty(correlation_id);
        self
    }

    pub fn position_between(mut self, position_after: i64, position_before: i64) -> Self {
        self.position_after = Some(position_after);
        self.position_before = Some(position_before);
        self
    }

    pub fn position_after(mut self, position_after: i64) -> Self {
        self.position_after = Some(position_after);
        self
    }

    pub fn position_before(mut self, position_before: i64) -> Self {
        self.position_before = Some(position_before);
        self
    }

    pub fn occurred_between(
        mut self,
        occurred_after: DateTime<Utc>,
        occurred_before: DateTime<Utc>,
    ) -> Self {
        self.occurred_after = Some(occurred_after);
        self.occurred_before = Some(occurred_before);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

fn some_non_empty(value: impl Into<String>) -> Option<String> {
    let value = value.into();
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}
```

### `backend/src/platform/events/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/rows.rs`
- Size bytes / Размер в байтах: `1032`
- Included characters / Включено символов: `1032`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::EventStoreError;
use super::models::{EventEnvelope, StoredEventEnvelope};

pub(super) fn row_to_event(row: PgRow) -> Result<EventEnvelope, EventStoreError> {
    Ok(EventEnvelope {
        event_id: row.try_get("event_id")?,
        event_type: row.try_get("event_type")?,
        schema_version: row.try_get("schema_version")?,
        occurred_at: row.try_get("occurred_at")?,
        recorded_at: row.try_get("recorded_at")?,
        source: row.try_get("source")?,
        actor: row.try_get("actor")?,
        subject: row.try_get("subject")?,
        payload: row.try_get("payload")?,
        provenance: row.try_get("provenance")?,
        causation_id: row.try_get("causation_id")?,
        correlation_id: row.try_get("correlation_id")?,
    })
}

pub(super) fn row_to_stored_event(row: PgRow) -> Result<StoredEventEnvelope, EventStoreError> {
    Ok(StoredEventEnvelope {
        position: row.try_get("position")?,
        event: row_to_event(row)?,
    })
}
```

### `backend/src/platform/events/runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/runtime.rs`
- Size bytes / Размер в байтах: `2885`
- Included characters / Включено символов: `2885`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::postgres::PgPool;

pub async fn ensure_runtime_processing_state(
    pool: &PgPool,
    source_code: &str,
    runtime_kind: &str,
    metadata: &Value,
) -> Result<String, sqlx::Error> {
    let existing = sqlx::query_scalar::<_, String>(
        r#"
        SELECT state
        FROM signal_runtime_states
        WHERE source_code = $1
          AND connection_id IS NULL
          AND runtime_kind = $2
        "#,
    )
    .bind(source_code)
    .bind(runtime_kind)
    .fetch_optional(pool)
    .await?;

    if let Some(state) = existing {
        return Ok(state);
    }

    let default_state = source_runtime_state_from_policies(pool, source_code).await?;

    sqlx::query(
        r#"
        INSERT INTO signal_runtime_states (
            id,
            source_code,
            runtime_kind,
            state,
            last_started_at,
            metadata
        )
        VALUES (
            gen_random_uuid(),
            $1,
            $2,
            $3,
            CASE WHEN $3 = 'running' THEN now() ELSE NULL END,
            $4
        )
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(source_code)
    .bind(runtime_kind)
    .bind(default_state)
    .bind(metadata)
    .execute(pool)
    .await?;

    sqlx::query_scalar::<_, String>(
        r#"
        SELECT state
        FROM signal_runtime_states
        WHERE source_code = $1
          AND connection_id IS NULL
          AND runtime_kind = $2
        "#,
    )
    .bind(source_code)
    .bind(runtime_kind)
    .fetch_one(pool)
    .await
}

pub async fn source_runtime_state_from_policies(
    pool: &PgPool,
    source_code: &str,
) -> Result<&'static str, sqlx::Error> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT mode
        FROM signal_policies
        WHERE (expires_at IS NULL OR expires_at > now())
          AND connection_id IS NULL
          AND event_pattern IS NULL
          AND (
                (scope = 'global' AND source_code IS NULL)
             OR (scope = 'source' AND source_code = $1)
          )
        "#,
    )
    .bind(source_code)
    .fetch_all(pool)
    .await?;

    if rows.iter().any(|mode| mode == "disabled") {
        return Ok("stopped");
    }
    if rows.iter().any(|mode| mode == "paused") {
        return Ok("paused");
    }
    if rows.iter().any(|mode| mode == "muted") {
        return Ok("muted");
    }

    Ok("running")
}

pub fn runtime_state_allows_processing(state: &str) -> bool {
    matches!(state, "running" | "starting" | "reconnecting")
}

pub async fn runtime_allows_processing(
    pool: &PgPool,
    source_code: &str,
    runtime_kind: &str,
    metadata: &Value,
) -> Result<bool, sqlx::Error> {
    let state = ensure_runtime_processing_state(pool, source_code, runtime_kind, metadata).await?;
    Ok(runtime_state_allows_processing(&state))
}
```

### `backend/src/platform/events/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/store.rs`
- Size bytes / Размер в байтах: `287`
- Included characters / Включено символов: `287`
- Truncated / Обрезано: `no`

```rust
mod append;
mod outbox;
mod read;
mod replay;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
```

### `backend/src/platform/events/store/append.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/store/append.rs`
- Size bytes / Размер в байтах: `1979`
- Included characters / Включено символов: `1979`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use super::EventStore;
use crate::platform::events::errors::EventStoreError;
use crate::platform::events::models::NewEventEnvelope;

impl EventStore {
    pub async fn append(&self, event: &NewEventEnvelope) -> Result<i64, EventStoreError> {
        let mut transaction = self.pool.begin().await?;
        let position = Self::append_in_transaction(&mut transaction, event).await?;
        transaction.commit().await?;

        Ok(position)
    }

    pub async fn append_idempotent(
        &self,
        event: &NewEventEnvelope,
    ) -> Result<Option<i64>, EventStoreError> {
        match self.append(event).await {
            Ok(position) => Ok(Some(position)),
            Err(error) if error.is_unique_violation() => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub async fn append_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event: &NewEventEnvelope,
    ) -> Result<i64, EventStoreError> {
        let position = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO event_log (
                event_id,
                event_type,
                schema_version,
                occurred_at,
                source,
                actor,
                subject,
                payload,
                provenance,
                causation_id,
                correlation_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING position
            "#,
        )
        .bind(&event.event_id)
        .bind(&event.event_type)
        .bind(event.schema_version)
        .bind(event.occurred_at)
        .bind(&event.source)
        .bind(&event.actor)
        .bind(&event.subject)
        .bind(&event.payload)
        .bind(&event.provenance)
        .bind(&event.causation_id)
        .bind(&event.correlation_id)
        .fetch_one(&mut **transaction)
        .await?;

        Ok(position)
    }
}
```

### `backend/src/platform/events/store/outbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/store/outbox.rs`
- Size bytes / Размер в байтах: `7068`
- Included characters / Включено символов: `7068`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Duration, Utc};
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::EventStore;
use crate::platform::events::errors::EventStoreError;
use crate::platform::events::models::{
    DispatchableEventOutboxItem, EventEnvelope, EventOutboxItem, NewEventEnvelope,
};

impl EventStore {
    pub async fn append_for_dispatch(
        &self,
        event: &NewEventEnvelope,
    ) -> Result<i64, EventStoreError> {
        let mut transaction = self.pool.begin().await?;
        let position = Self::append_in_transaction(&mut transaction, event).await?;

        sqlx::query(
            r#"
            INSERT INTO event_outbox (event_id, subject)
            VALUES ($1, $2)
            ON CONFLICT (event_id) DO NOTHING
            "#,
        )
        .bind(&event.event_id)
        .bind(&event.event_type)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(position)
    }

    pub async fn append_for_dispatch_idempotent(
        &self,
        event: &NewEventEnvelope,
    ) -> Result<Option<i64>, EventStoreError> {
        match self.append_for_dispatch(event).await {
            Ok(position) => Ok(Some(position)),
            Err(error) if error.is_unique_violation() => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub async fn pending_outbox_batch(
        &self,
        limit: u32,
    ) -> Result<Vec<EventOutboxItem>, EventStoreError> {
        let limit = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"
            SELECT event_id, subject, status, attempts
            FROM event_outbox
            WHERE status = 'pending'
              AND next_attempt_at <= now()
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_outbox_item).collect()
    }

    pub async fn recover_stale_outbox_items(
        &self,
        stale_dispatch_after: Duration,
    ) -> Result<u32, EventStoreError> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE event_outbox
            SET
                status = 'pending',
                next_attempt_at = now(),
                last_error_redacted = COALESCE(last_error_redacted, 'dispatcher recovered stale dispatch lease'),
                updated_at = now()
            WHERE status = 'dispatching'
              AND updated_at <= $1
            "#,
        )
        .bind(Utc::now() - stale_dispatch_after)
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(u32::try_from(rows_affected).unwrap_or(u32::MAX))
    }

    pub async fn claim_pending_outbox_batch(
        &self,
        limit: u32,
    ) -> Result<Vec<DispatchableEventOutboxItem>, EventStoreError> {
        let limit = i64::from(limit.clamp(1, 1000));
        let mut transaction = self.pool.begin().await?;
        let rows = sqlx::query(
            r#"
            WITH candidates AS (
                SELECT event_id
                FROM event_outbox
                WHERE status = 'pending'
                  AND next_attempt_at <= now()
                ORDER BY created_at ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE event_outbox AS outbox
            SET
                status = 'dispatching',
                attempts = outbox.attempts + 1,
                last_error_redacted = NULL,
                updated_at = now()
            FROM candidates
            JOIN event_log ON event_log.event_id = candidates.event_id
            WHERE outbox.event_id = candidates.event_id
            RETURNING
                outbox.event_id,
                outbox.subject AS outbox_subject,
                outbox.attempts,
                event_log.event_id AS log_event_id,
                event_log.event_type,
                event_log.schema_version,
                event_log.occurred_at,
                event_log.recorded_at,
                event_log.source,
                event_log.actor,
                event_log.subject,
                event_log.payload,
                event_log.provenance,
                event_log.causation_id,
                event_log.correlation_id
            "#,
        )
        .bind(limit)
        .fetch_all(&mut *transaction)
        .await?;

        transaction.commit().await?;
        rows.into_iter()
            .map(row_to_dispatchable_outbox_item)
            .collect()
    }

    pub async fn mark_outbox_published(&self, event_id: &str) -> Result<(), EventStoreError> {
        sqlx::query(
            r#"
            UPDATE event_outbox
            SET
                status = 'published',
                published_at = now(),
                updated_at = now()
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_outbox_retry(
        &self,
        event_id: &str,
        error: &str,
        next_attempt_at: DateTime<Utc>,
    ) -> Result<(), EventStoreError> {
        sqlx::query(
            r#"
            UPDATE event_outbox
            SET
                status = 'pending',
                next_attempt_at = $2,
                last_error_redacted = $3,
                updated_at = now()
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .bind(next_attempt_at)
        .bind(truncate_redacted_error(error))
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn row_to_outbox_item(row: PgRow) -> Result<EventOutboxItem, EventStoreError> {
    Ok(EventOutboxItem {
        event_id: row.try_get("event_id")?,
        subject: row.try_get("subject")?,
        status: row.try_get("status")?,
        attempts: row.try_get("attempts")?,
    })
}

fn row_to_dispatchable_outbox_item(
    row: PgRow,
) -> Result<DispatchableEventOutboxItem, EventStoreError> {
    Ok(DispatchableEventOutboxItem {
        event_id: row.try_get("event_id")?,
        subject: row.try_get("outbox_subject")?,
        attempts: row.try_get("attempts")?,
        event: EventEnvelope {
            event_id: row.try_get("log_event_id")?,
            event_type: row.try_get("event_type")?,
            schema_version: row.try_get("schema_version")?,
            occurred_at: row.try_get("occurred_at")?,
            recorded_at: row.try_get("recorded_at")?,
            source: row.try_get("source")?,
            actor: row.try_get("actor")?,
            subject: row.try_get("subject")?,
            payload: row.try_get("payload")?,
            provenance: row.try_get("provenance")?,
            causation_id: row.try_get("causation_id")?,
            correlation_id: row.try_get("correlation_id")?,
        },
    })
}

fn truncate_redacted_error(error: &str) -> String {
    let trimmed = error.trim();
    if trimmed.chars().count() <= 500 {
        return trimmed.to_owned();
    }

    trimmed.chars().take(500).collect()
}
```

### `backend/src/platform/events/store/read.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/store/read.rs`
- Size bytes / Размер в байтах: `3573`
- Included characters / Включено символов: `3573`
- Truncated / Обрезано: `no`

```rust
use super::EventStore;
use crate::platform::events::errors::EventStoreError;
use crate::platform::events::models::{EventEnvelope, StoredEventEnvelope};
use crate::platform::events::query::EventLogQuery;
use crate::platform::events::rows::{row_to_event, row_to_stored_event};
use sqlx::{Postgres, QueryBuilder};

impl EventStore {
    pub async fn get_by_id(
        &self,
        event_id: &str,
    ) -> Result<Option<EventEnvelope>, EventStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
                event_id,
                event_type,
                schema_version,
                occurred_at,
                recorded_at,
                source,
                actor,
                subject,
                payload,
                provenance,
                causation_id,
                correlation_id
            FROM event_log
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_event).transpose()
    }

    pub async fn list_matching(
        &self,
        query: EventLogQuery,
    ) -> Result<Vec<StoredEventEnvelope>, EventStoreError> {
        let limit = i64::from(query.limit.unwrap_or(100).clamp(1, 1000));
        let mut builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                position,
                event_id,
                event_type,
                schema_version,
                occurred_at,
                recorded_at,
                source,
                actor,
                subject,
                payload,
                provenance,
                causation_id,
                correlation_id
            FROM event_log
            WHERE TRUE
            "#,
        );

        if let Some(event_type) = query.event_type {
            builder.push(" AND event_type = ");
            builder.push_bind(event_type);
        }

        if let Some(source_code) = query.source_code {
            builder.push(" AND source ->> 'source_code' = ");
            builder.push_bind(source_code);
        }

        if let Some(subject_kind) = query.subject_kind {
            builder.push(" AND subject ->> 'kind' = ");
            builder.push_bind(subject_kind);
        }

        if let Some(subject_entity_id) = query.subject_entity_id {
            builder.push(" AND subject ->> 'entity_id' = ");
            builder.push_bind(subject_entity_id);
        }

        if let Some(correlation_id) = query.correlation_id {
            builder.push(" AND correlation_id = ");
            builder.push_bind(correlation_id);
        }

        if let Some(position_after) = query.position_after {
            builder.push(" AND position >= ");
            builder.push_bind(position_after);
        }

        if let Some(position_before) = query.position_before {
            builder.push(" AND position <= ");
            builder.push_bind(position_before);
        }

        if let Some(occurred_after) = query.occurred_after {
            builder.push(" AND occurred_at >= ");
            builder.push_bind(occurred_after);
        }

        if let Some(occurred_before) = query.occurred_before {
            builder.push(" AND occurred_at <= ");
            builder.push_bind(occurred_before);
        }

        builder.push(" ORDER BY occurred_at ASC, position ASC LIMIT ");
        builder.push_bind(limit);

        let rows = builder.build().fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_stored_event).collect()
    }
}
```

### `backend/src/platform/events/store/replay.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/store/replay.rs`
- Size bytes / Размер в байтах: `1280`
- Included characters / Включено символов: `1280`
- Truncated / Обрезано: `no`

```rust
use super::EventStore;
use crate::platform::events::errors::EventStoreError;
use crate::platform::events::models::StoredEventEnvelope;
use crate::platform::events::rows::row_to_stored_event;

impl EventStore {
    pub async fn list_after_position(
        &self,
        after_position: i64,
        limit: u32,
    ) -> Result<Vec<StoredEventEnvelope>, EventStoreError> {
        if after_position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(after_position));
        }

        let limit = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"
            SELECT
                position,
                event_id,
                event_type,
                schema_version,
                occurred_at,
                recorded_at,
                source,
                actor,
                subject,
                payload,
                provenance,
                causation_id,
                correlation_id
            FROM event_log
            WHERE position > $1
            ORDER BY position ASC
            LIMIT $2
            "#,
        )
        .bind(after_position)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_stored_event).collect()
    }
}
```

### `backend/src/platform/events/trace.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/events/trace.rs`
- Size bytes / Размер в байтах: `9804`
- Included characters / Включено символов: `9804`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::errors::{EventEnvelopeError, EventStoreError};
use super::models::StoredEventEnvelope;
use super::rows::row_to_stored_event;
use super::store::EventStore;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventTrace {
    pub correlation_id: String,
    pub root_event_ids: Vec<String>,
    pub events: Vec<StoredEventEnvelope>,
    pub edges: Vec<EventTraceEdge>,
    pub orphan_event_ids: Vec<String>,
    pub missing_parent_ids: Vec<String>,
    pub consumer_annotations: Vec<EventConsumerAnnotation>,
    pub dead_letters: Vec<EventDeadLetterAnnotation>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventTraceEdge {
    pub parent_event_id: String,
    pub child_event_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventConsumerAnnotation {
    pub event_id: String,
    pub consumer_name: String,
    pub status: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub attempts: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventDeadLetterAnnotation {
    pub event_id: String,
    pub consumer_name: Option<String>,
    pub reason: String,
    pub failed_at: Option<DateTime<Utc>>,
}

impl EventStore {
    pub async fn list_by_correlation_id(
        &self,
        correlation_id: &str,
        limit: u32,
    ) -> Result<Vec<StoredEventEnvelope>, EventStoreError> {
        validate_non_empty("correlation_id", correlation_id)?;
        let sql =
            stored_event_select_sql("WHERE correlation_id = $1 ORDER BY position ASC LIMIT $2");
        let rows = sqlx::query(&sql)
            .bind(correlation_id.trim())
            .bind(trace_limit(limit))
            .fetch_all(self.pool())
            .await?;

        rows.into_iter().map(row_to_stored_event).collect()
    }

    pub async fn list_children(
        &self,
        parent_event_id: &str,
        limit: u32,
    ) -> Result<Vec<StoredEventEnvelope>, EventStoreError> {
        validate_non_empty("parent_event_id", parent_event_id)?;
        let sql = stored_event_select_sql("WHERE causation_id = $1 ORDER BY position ASC LIMIT $2");
        let rows = sqlx::query(&sql)
            .bind(parent_event_id.trim())
            .bind(trace_limit(limit))
            .fetch_all(self.pool())
            .await?;

        rows.into_iter().map(row_to_stored_event).collect()
    }

    pub async fn trace_by_event_id(
        &self,
        event_id: &str,
        limit: u32,
    ) -> Result<Option<EventTrace>, EventStoreError> {
        validate_non_empty("event_id", event_id)?;
        let Some(anchor) = self.get_stored_by_id(event_id.trim()).await? else {
            return Ok(None);
        };
        let correlation_id = resolved_correlation_id(&anchor);
        let mut events = self.list_by_correlation_id(&correlation_id, limit).await?;

        if !events
            .iter()
            .any(|event| event.event.event_id == anchor.event.event_id)
        {
            events.push(anchor);
            events.sort_by_key(|event| event.position);
        }

        self.build_trace(correlation_id, events).await.map(Some)
    }

    pub async fn trace_by_correlation_id(
        &self,
        correlation_id: &str,
        limit: u32,
    ) -> Result<EventTrace, EventStoreError> {
        validate_non_empty("correlation_id", correlation_id)?;
        let correlation_id = correlation_id.trim().to_owned();
        let events = self.list_by_correlation_id(&correlation_id, limit).await?;

        self.build_trace(correlation_id, events).await
    }

    async fn get_stored_by_id(
        &self,
        event_id: &str,
    ) -> Result<Option<StoredEventEnvelope>, EventStoreError> {
        let sql = stored_event_select_sql("WHERE event_id = $1");
        let row = sqlx::query(&sql)
            .bind(event_id)
            .fetch_optional(self.pool())
            .await?;

        row.map(row_to_stored_event).transpose()
    }

    async fn build_trace(
        &self,
        correlation_id: String,
        events: Vec<StoredEventEnvelope>,
    ) -> Result<EventTrace, EventStoreError> {
        let positions = events
            .iter()
            .map(|event| event.position)
            .collect::<Vec<_>>();
        let consumer_annotations = self.consumer_annotations_for_positions(&positions).await?;
        let dead_letters = self.dead_letters_for_positions(&positions).await?;
        let event_ids = events
            .iter()
            .map(|event| event.event.event_id.as_str())
            .collect::<HashSet<_>>();

        let mut root_event_ids = Vec::new();
        let mut edges = Vec::new();
        let mut orphan_event_ids = Vec::new();
        let mut missing_parent_ids = Vec::new();
        let mut seen_missing_parent_ids = HashSet::new();

        for stored in &events {
            match stored.event.causation_id.as_deref() {
                None => {
                    root_event_ids.push(stored.event.event_id.clone());
                    if stored.event.correlation_id.is_none() {
                        orphan_event_ids.push(stored.event.event_id.clone());
                    }
                }
                Some(parent_event_id) if event_ids.contains(parent_event_id) => {
                    edges.push(EventTraceEdge {
                        parent_event_id: parent_event_id.to_owned(),
                        child_event_id: stored.event.event_id.clone(),
                    });
                }
                Some(parent_event_id) => {
                    orphan_event_ids.push(stored.event.event_id.clone());
                    if seen_missing_parent_ids.insert(parent_event_id.to_owned()) {
                        missing_parent_ids.push(parent_event_id.to_owned());
                    }
                }
            }
        }

        Ok(EventTrace {
            correlation_id,
            root_event_ids,
            events,
            edges,
            orphan_event_ids,
            missing_parent_ids,
            consumer_annotations,
            dead_letters,
        })
    }

    async fn consumer_annotations_for_positions(
        &self,
        positions: &[i64],
    ) -> Result<Vec<EventConsumerAnnotation>, EventStoreError> {
        if positions.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT
                event_id,
                consumer_name,
                'processed'::text AS status,
                processed_at,
                NULL::integer AS attempts
            FROM event_consumer_processed_events
            WHERE event_position = ANY($1)
            UNION ALL
            SELECT
                event_id,
                consumer_name,
                'failed'::text AS status,
                NULL::timestamptz AS processed_at,
                attempt_count AS attempts
            FROM event_consumer_failures
            WHERE event_position = ANY($1)
            ORDER BY event_id ASC, consumer_name ASC, status ASC
            "#,
        )
        .bind(positions)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(EventConsumerAnnotation {
                    event_id: row.try_get("event_id")?,
                    consumer_name: row.try_get("consumer_name")?,
                    status: row.try_get("status")?,
                    processed_at: row.try_get("processed_at")?,
                    attempts: row.try_get("attempts")?,
                })
            })
            .collect()
    }

    async fn dead_letters_for_positions(
        &self,
        positions: &[i64],
    ) -> Result<Vec<EventDeadLetterAnnotation>, EventStoreError> {
        if positions.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT
                event_id,
                consumer_name,
                last_error AS reason,
                created_at AS failed_at
            FROM event_dead_letters
            WHERE event_position = ANY($1)
            ORDER BY event_id ASC, consumer_name ASC
            "#,
        )
        .bind(positions)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(EventDeadLetterAnnotation {
                    event_id: row.try_get("event_id")?,
                    consumer_name: Some(row.try_get("consumer_name")?),
                    reason: row.try_get("reason")?,
                    failed_at: row.try_get("failed_at")?,
                })
            })
            .collect()
    }
}

fn resolved_correlation_id(event: &StoredEventEnvelope) -> String {
    event
        .event
        .correlation_id
        .clone()
        .unwrap_or_else(|| event.event.event_id.clone())
}

fn trace_limit(limit: u32) -> i64 {
    i64::from(limit.clamp(1, 5000))
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), EventStoreError> {
    if value.trim().is_empty() {
        return Err(EventStoreError::Envelope(EventEnvelopeError::EmptyField(
            field_name,
        )));
    }

    Ok(())
}

fn stored_event_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT
            position,
            event_id,
            event_type,
            schema_version,
            occurred_at,
            recorded_at,
            source,
            actor,
            subject,
            payload,
            provenance,
            causation_id,
            correlation_id
        FROM event_log
        {where_clause}
        "#
    )
}
```
