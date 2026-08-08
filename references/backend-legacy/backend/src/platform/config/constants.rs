pub(crate) const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:8080";
pub(crate) const DEFAULT_SERVICE_NAME: &str = "makosh-backend";
// Local-development fallback per ADR-0056: the API binds to 127.0.0.1 only, so
// a well-known default lets `cargo run` / `make dev` work without env setup.
// Packaged desktop builds override it with a per-build random secret.
pub(crate) const DEFAULT_LOCAL_API_SECRET: &str = "change-me-local-api-secret";
pub(crate) const DEFAULT_OLLAMA_BASE_URL: &str = "http://192.168.1.2:11434";
pub(crate) const DEFAULT_OLLAMA_CHAT_MODEL: &str = "qwen3:4b";
pub(crate) const DEFAULT_OLLAMA_EMBED_MODEL: &str = "qwen3-embedding:4b";
pub(crate) const DEFAULT_OLLAMA_TIMEOUT_SECONDS: u64 = 120;
pub(crate) const DEFAULT_OMNIROUTE_BASE_URL: &str = "https://ai.sh-inc.ru/v1";
pub(crate) const DEFAULT_OMNIROUTE_CHAT_MODEL: &str = "codex/gpt-5.5";
pub(crate) const DEFAULT_OMNIROUTE_EMBED_MODEL: &str =
    "openai-compatible-chat-ollama-pve/qwen3-embedding:4b";
pub(crate) const DEFAULT_OMNIROUTE_TIMEOUT_SECONDS: u64 = 120;
