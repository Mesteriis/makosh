use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;

use makosh_hub_backend::platform::config::{
    ai::AiRuntimeProvider, app_config::AppConfig, errors::ConfigError,
};

#[test]
fn default_config_binds_to_localhost_without_database_url() {
    let config = AppConfig::default();

    assert_eq!(
        config.http_addr(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
    );
    assert_eq!(config.service_name(), "makosh-backend");
    assert_eq!(config.database_url(), None);
    assert_eq!(
        config.local_api_secret(),
        Some("change-me-local-api-secret")
    );
    assert_eq!(config.secret_vault_path(), None);
    assert_eq!(config.secret_vault_key(), None);
    assert_eq!(config.tdjson_path(), None);
    assert!(config.zoom_token_maintenance_scheduler_enabled());
    assert!(config.zoom_recording_sync_scheduler_enabled());
    assert!(config.zoom_retention_cleanup_scheduler_enabled());
}

#[test]
fn config_from_pairs_overrides_http_addr_database_url_and_local_api_secret() {
    let config = AppConfig::from_pairs([
        ("MAKOSH_HTTP_ADDR", "127.0.0.1:9090"),
        (
            "DATABASE_URL",
            "postgres://makosh:local-dev-password@postgres:5432/makosh_hub",
        ),
        ("MAKOSH_LOCAL_API_SECRET", "local-dev-api-secret"),
    ])
    .expect("valid config");

    assert_eq!(
        config.http_addr(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9090)
    );
    assert_eq!(
        config.database_url(),
        Some("postgres://makosh:local-dev-password@postgres:5432/makosh_hub")
    );
    assert_eq!(config.local_api_secret(), Some("local-dev-api-secret"));
}

#[test]
fn config_from_pairs_accepts_secret_vault_path_and_key() {
    let config = AppConfig::from_pairs([
        (
            "MAKOSH_SECRET_VAULT_PATH",
            "docker/data/secrets/makosh.vault.json",
        ),
        ("MAKOSH_SECRET_VAULT_KEY", "local-vault-key"),
    ])
    .expect("valid secret vault config");

    assert_eq!(
        config.secret_vault_path(),
        Some(Path::new("docker/data/secrets/makosh.vault.json"))
    );
    assert_eq!(
        config
            .secret_vault_key()
            .expect("vault key")
            .expose_for_runtime(),
        "local-vault-key"
    );
    assert_eq!(
        format!("{:?}", config.secret_vault_key().expect("vault key")),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[test]
fn config_from_pairs_accepts_ollama_runtime_overrides() {
    let config = AppConfig::from_pairs([
        ("MAKOSH_OLLAMA_BASE_URL", "http://192.168.1.2:11434"),
        ("MAKOSH_OLLAMA_CHAT_MODEL", "qwen3:4b"),
        ("MAKOSH_OLLAMA_EMBED_MODEL", "qwen3-embedding:4b"),
        ("MAKOSH_OLLAMA_TIMEOUT_SECONDS", "120"),
    ])
    .expect("valid Ollama config");

    assert_eq!(config.ollama_base_url(), "http://192.168.1.2:11434");
    assert_eq!(config.ollama_chat_model(), "qwen3:4b");
    assert_eq!(config.ollama_embed_model(), "qwen3-embedding:4b");
    assert_eq!(config.ollama_timeout_seconds(), 120);
}

#[test]
fn config_from_pairs_accepts_omniroute_runtime_overrides_without_printing_key() {
    let config = AppConfig::from_pairs([
        ("MAKOSH_AI_PROVIDER", "omniroute"),
        ("MAKOSH_OMNIROUTE_BASE_URL", "https://ai.sh-inc.ru/v1/"),
        ("MAKOSH_OMNIROUTE_CHAT_MODEL", "codex/gpt-5.5"),
        (
            "MAKOSH_OMNIROUTE_EMBED_MODEL",
            "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
        ),
        ("MAKOSH_OMNIROUTE_TIMEOUT_SECONDS", "90"),
        ("MAKOSH_OMNIROUTE_API_KEY", "omniroute-test-key"),
    ])
    .expect("valid OmniRoute config");

    assert_eq!(config.ai_provider(), AiRuntimeProvider::OmniRoute);
    assert_eq!(config.omniroute_base_url(), "https://ai.sh-inc.ru/v1");
    assert_eq!(config.omniroute_chat_model(), "codex/gpt-5.5");
    assert_eq!(
        config.omniroute_embed_model(),
        "openai-compatible-chat-ollama-pve/qwen3-embedding:4b"
    );
    assert_eq!(config.omniroute_timeout_seconds(), 90);
    assert_eq!(
        config
            .omniroute_api_key()
            .expect("OmniRoute API key")
            .expose_for_runtime(),
        "omniroute-test-key"
    );
    assert_eq!(
        format!(
            "{:?}",
            config.omniroute_api_key().expect("OmniRoute API key")
        ),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[test]
fn config_from_pairs_accepts_tdjson_runtime_path() {
    let config =
        AppConfig::from_pairs([("MAKOSH_TDJSON_PATH", "/opt/homebrew/lib/libtdjson.dylib")])
            .expect("valid TDLib JSON runtime config");

    assert_eq!(
        config.tdjson_path(),
        Some(Path::new("/opt/homebrew/lib/libtdjson.dylib"))
    );
}

#[test]
fn config_from_pairs_accepts_telegram_app_credentials() {
    let config = AppConfig::from_pairs([
        ("MAKOSH_TELEGRAM_API_ID", "12345"),
        ("MAKOSH_TELEGRAM_API_HASH", "telegram-api-hash"),
    ])
    .expect("valid Telegram app credential config");

    assert_eq!(config.telegram_api_id(), Some(12345));
    assert_eq!(
        config
            .telegram_api_hash()
            .expect("Telegram API hash")
            .expose_for_runtime(),
        "telegram-api-hash"
    );
    assert_eq!(
        format!(
            "{:?}",
            config.telegram_api_hash().expect("Telegram API hash")
        ),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[test]
fn config_from_pairs_accepts_zoom_token_maintenance_scheduler_toggle() {
    let config =
        AppConfig::from_pairs([("MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED", "false")])
            .expect("valid Zoom token maintenance scheduler config");

    assert!(!config.zoom_token_maintenance_scheduler_enabled());
}

#[test]
fn config_from_pairs_accepts_zoom_recording_sync_scheduler_toggle() {
    let config = AppConfig::from_pairs([("MAKOSH_ZOOM_RECORDING_SYNC_SCHEDULER_ENABLED", "false")])
        .expect("valid Zoom recording sync scheduler config");

    assert!(!config.zoom_recording_sync_scheduler_enabled());
}

#[test]
fn config_from_pairs_accepts_zoom_retention_cleanup_scheduler_toggle() {
    let config =
        AppConfig::from_pairs([("MAKOSH_ZOOM_RETENTION_CLEANUP_SCHEDULER_ENABLED", "false")])
            .expect("valid Zoom retention cleanup scheduler config");

    assert!(!config.zoom_retention_cleanup_scheduler_enabled());
}

#[test]
fn default_config_uses_local_ollama_and_qwen_models() {
    let config = AppConfig::default();

    assert_eq!(config.ai_provider(), AiRuntimeProvider::Ollama);
    assert_eq!(config.ollama_base_url(), "http://192.168.1.2:11434");
    assert_eq!(config.ollama_chat_model(), "qwen3:4b");
    assert_eq!(config.ollama_embed_model(), "qwen3-embedding:4b");
    assert_eq!(config.ollama_timeout_seconds(), 120);
    assert_eq!(config.omniroute_base_url(), "https://ai.sh-inc.ru/v1");
    assert_eq!(config.omniroute_chat_model(), "codex/gpt-5.5");
    assert_eq!(
        config.omniroute_embed_model(),
        "openai-compatible-chat-ollama-pve/qwen3-embedding:4b"
    );
    assert_eq!(config.omniroute_timeout_seconds(), 120);
    assert_eq!(config.omniroute_api_key(), None);
}

#[test]
fn config_from_pairs_rejects_invalid_http_addr() {
    let error = AppConfig::from_pairs([("MAKOSH_HTTP_ADDR", "not-a-socket")])
        .expect_err("invalid socket address must fail");

    assert!(matches!(error, ConfigError::InvalidHttpAddr { .. }));
}

#[test]
fn config_from_pairs_rejects_empty_database_url() {
    let error =
        AppConfig::from_pairs([("DATABASE_URL", "   ")]).expect_err("empty database URL must fail");

    assert!(matches!(error, ConfigError::EmptyDatabaseUrl));
}

#[test]
fn config_from_pairs_rejects_empty_local_api_secret() {
    let error = AppConfig::from_pairs([("MAKOSH_LOCAL_API_SECRET", "   ")])
        .expect_err("empty local API secret must fail");

    assert!(matches!(error, ConfigError::EmptyLocalApiSecret));
}

#[test]
fn config_from_pairs_rejects_empty_secret_vault_path() {
    let error = AppConfig::from_pairs([("MAKOSH_SECRET_VAULT_PATH", "   ")])
        .expect_err("empty secret vault path must fail");

    assert!(matches!(error, ConfigError::EmptySecretVaultPath));
}

#[test]
fn config_from_pairs_rejects_empty_secret_vault_key() {
    let error = AppConfig::from_pairs([("MAKOSH_SECRET_VAULT_KEY", "   ")])
        .expect_err("empty secret vault key must fail");

    assert!(matches!(error, ConfigError::EmptySecretVaultKey));
}

#[test]
fn config_from_pairs_rejects_empty_tdjson_path() {
    let error = AppConfig::from_pairs([("MAKOSH_TDJSON_PATH", "   ")])
        .expect_err("empty TDLib JSON runtime path must fail");

    assert!(matches!(error, ConfigError::EmptyTdjsonPath));
}

#[test]
fn config_from_pairs_rejects_invalid_telegram_app_credentials() {
    let error = AppConfig::from_pairs([("MAKOSH_TELEGRAM_API_ID", "0")])
        .expect_err("zero Telegram API ID must fail");
    assert!(matches!(error, ConfigError::InvalidTelegramApiId { .. }));

    let error = AppConfig::from_pairs([("MAKOSH_TELEGRAM_API_ID", "not-a-number")])
        .expect_err("non-numeric Telegram API ID must fail");
    assert!(matches!(error, ConfigError::InvalidTelegramApiId { .. }));

    let error = AppConfig::from_pairs([("MAKOSH_TELEGRAM_API_HASH", "   ")])
        .expect_err("empty Telegram API hash must fail");
    assert!(matches!(error, ConfigError::EmptyTelegramApiHash));
}

#[test]
fn config_from_pairs_rejects_invalid_ollama_values() {
    let error = AppConfig::from_pairs([("MAKOSH_OLLAMA_BASE_URL", "   ")])
        .expect_err("empty Ollama base URL must fail");
    assert!(matches!(error, ConfigError::EmptyOllamaBaseUrl));

    let error = AppConfig::from_pairs([("MAKOSH_OLLAMA_CHAT_MODEL", "   ")])
        .expect_err("empty Ollama chat model must fail");
    assert!(matches!(error, ConfigError::EmptyOllamaChatModel));

    let error = AppConfig::from_pairs([("MAKOSH_OLLAMA_EMBED_MODEL", "   ")])
        .expect_err("empty Ollama embed model must fail");
    assert!(matches!(error, ConfigError::EmptyOllamaEmbedModel));

    let error = AppConfig::from_pairs([("MAKOSH_OLLAMA_TIMEOUT_SECONDS", "0")])
        .expect_err("zero Ollama timeout must fail");
    assert!(matches!(error, ConfigError::InvalidOllamaTimeout { .. }));
}

#[test]
fn config_from_pairs_rejects_invalid_omniroute_values() {
    let error = AppConfig::from_pairs([("MAKOSH_AI_PROVIDER", "cloudy")])
        .expect_err("unknown AI provider must fail");
    assert!(matches!(error, ConfigError::InvalidAiProvider { .. }));

    let error = AppConfig::from_pairs([("MAKOSH_OMNIROUTE_BASE_URL", "   ")])
        .expect_err("empty OmniRoute base URL must fail");
    assert!(matches!(error, ConfigError::EmptyOmniRouteBaseUrl));

    let error = AppConfig::from_pairs([("MAKOSH_OMNIROUTE_CHAT_MODEL", "   ")])
        .expect_err("empty OmniRoute chat model must fail");
    assert!(matches!(error, ConfigError::EmptyOmniRouteChatModel));

    let error = AppConfig::from_pairs([("MAKOSH_OMNIROUTE_EMBED_MODEL", "   ")])
        .expect_err("empty OmniRoute embed model must fail");
    assert!(matches!(error, ConfigError::EmptyOmniRouteEmbedModel));

    let error = AppConfig::from_pairs([("MAKOSH_OMNIROUTE_TIMEOUT_SECONDS", "0")])
        .expect_err("zero OmniRoute timeout must fail");
    assert!(matches!(error, ConfigError::InvalidOmniRouteTimeout { .. }));

    let error = AppConfig::from_pairs([("MAKOSH_OMNIROUTE_API_KEY", "   ")])
        .expect_err("empty OmniRoute API key must fail");
    assert!(matches!(error, ConfigError::EmptyOmniRouteApiKey));
}
