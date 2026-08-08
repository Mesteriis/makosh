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

- Chunk ID / ID чанка: `072-source-backend-part-052`
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

### `backend/src/platform/secrets/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/secrets/validation.rs`
- Size bytes / Размер в байтах: `1363`
- Included characters / Включено символов: `1363`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::database_vault::DatabaseEncryptedVaultError;
use super::errors::{SecretReferenceError, SecretResolutionError};
use super::file_vault::EncryptedVaultError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), SecretReferenceError> {
    if value.trim().is_empty() {
        return Err(SecretReferenceError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), SecretReferenceError> {
    if !value.is_object() {
        return Err(SecretReferenceError::NonObjectJson(field_name));
    }

    Ok(())
}

pub(super) fn validate_secret_resolution_ref(value: &str) -> Result<(), SecretResolutionError> {
    if value.trim().is_empty() {
        return Err(SecretResolutionError::EmptySecretRef);
    }

    Ok(())
}

pub(super) fn validate_vault_field(
    field: &'static str,
    value: &str,
) -> Result<(), EncryptedVaultError> {
    if value.trim().is_empty() {
        return Err(EncryptedVaultError::EmptyField(field));
    }
    Ok(())
}

pub(super) fn validate_database_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), DatabaseEncryptedVaultError> {
    if value.trim().is_empty() {
        return Err(DatabaseEncryptedVaultError::EmptyField(field));
    }

    Ok(())
}
```

### `backend/src/platform/settings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings.rs`
- Size bytes / Размер в байтах: `318`
- Included characters / Включено символов: `318`
- Truncated / Обрезано: `no`

```rust
mod ai_runtime;
mod constants;
mod definitions;
mod errors;
mod models;
mod persistence;
mod store;
mod validation;

pub use ai_runtime::AiRuntimeSettings;
pub use errors::SettingsError;
pub use models::{ApplicationSetting, ApplicationSettingsRepairSummary, SettingValueKind};
pub use store::ApplicationSettingsStore;
```

### `backend/src/platform/settings/ai_runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/ai_runtime.rs`
- Size bytes / Размер в байтах: `4604`
- Included characters / Включено символов: `4604`
- Truncated / Обрезано: `no`

```rust
use crate::platform::config::{AiRuntimeProvider, AppConfig};

use super::models::ApplicationSetting;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiRuntimeSettings {
    pub provider: AiRuntimeProvider,
    pub base_url: String,
    pub chat_model: String,
    pub embedding_model: String,
    pub timeout_seconds: u64,
}

impl AiRuntimeSettings {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            provider: config.ai_provider(),
            base_url: match config.ai_provider() {
                AiRuntimeProvider::Ollama => config.ollama_base_url().to_owned(),
                AiRuntimeProvider::OmniRoute => config.omniroute_base_url().to_owned(),
            },
            chat_model: match config.ai_provider() {
                AiRuntimeProvider::Ollama => config.ollama_chat_model().to_owned(),
                AiRuntimeProvider::OmniRoute => config.omniroute_chat_model().to_owned(),
            },
            embedding_model: match config.ai_provider() {
                AiRuntimeProvider::Ollama => config.ollama_embed_model().to_owned(),
                AiRuntimeProvider::OmniRoute => config.omniroute_embed_model().to_owned(),
            },
            timeout_seconds: match config.ai_provider() {
                AiRuntimeProvider::Ollama => config.ollama_timeout_seconds(),
                AiRuntimeProvider::OmniRoute => config.omniroute_timeout_seconds(),
            },
        }
    }
}

pub(crate) fn runtime_settings_from_values(
    settings: &[ApplicationSetting],
    fallback: &AppConfig,
) -> AiRuntimeSettings {
    AiRuntimeSettings {
        provider: ai_provider_value(settings).unwrap_or_else(|| fallback.ai_provider()),
        base_url: ai_base_url_value(settings, fallback),
        chat_model: ai_chat_model_value(settings, fallback),
        embedding_model: ai_embedding_model_value(settings, fallback),
        timeout_seconds: integer_value(settings, "ai.timeout_seconds")
            .and_then(|value| u64::try_from(value).ok())
            .filter(|value| *value > 0)
            .unwrap_or_else(|| {
                match ai_provider_value(settings).unwrap_or_else(|| fallback.ai_provider()) {
                    AiRuntimeProvider::Ollama => fallback.ollama_timeout_seconds(),
                    AiRuntimeProvider::OmniRoute => fallback.omniroute_timeout_seconds(),
                }
            }),
    }
}

fn string_value(settings: &[ApplicationSetting], setting_key: &str) -> Option<String> {
    settings
        .iter()
        .find(|setting| setting.setting_key == setting_key)
        .and_then(|setting| setting.value.as_str())
        .map(str::to_owned)
}

fn ai_provider_value(settings: &[ApplicationSetting]) -> Option<AiRuntimeProvider> {
    string_value(settings, "ai.provider")
        .as_deref()
        .and_then(|value| AiRuntimeProvider::try_from(value).ok())
}

fn ai_base_url_value(settings: &[ApplicationSetting], fallback: &AppConfig) -> String {
    match ai_provider_value(settings).unwrap_or_else(|| fallback.ai_provider()) {
        AiRuntimeProvider::Ollama => string_value(settings, "ai.ollama_base_url")
            .unwrap_or_else(|| fallback.ollama_base_url().to_owned()),
        AiRuntimeProvider::OmniRoute => string_value(settings, "ai.omniroute_base_url")
            .unwrap_or_else(|| fallback.omniroute_base_url().to_owned()),
    }
}

fn ai_chat_model_value(settings: &[ApplicationSetting], fallback: &AppConfig) -> String {
    match ai_provider_value(settings).unwrap_or_else(|| fallback.ai_provider()) {
        AiRuntimeProvider::Ollama => string_value(settings, "ai.chat_model")
            .unwrap_or_else(|| fallback.ollama_chat_model().to_owned()),
        AiRuntimeProvider::OmniRoute => string_value(settings, "ai.omniroute_chat_model")
            .unwrap_or_else(|| fallback.omniroute_chat_model().to_owned()),
    }
}

fn ai_embedding_model_value(settings: &[ApplicationSetting], fallback: &AppConfig) -> String {
    match ai_provider_value(settings).unwrap_or_else(|| fallback.ai_provider()) {
        AiRuntimeProvider::Ollama => string_value(settings, "ai.embedding_model")
            .unwrap_or_else(|| fallback.ollama_embed_model().to_owned()),
        AiRuntimeProvider::OmniRoute => string_value(settings, "ai.omniroute_embedding_model")
            .unwrap_or_else(|| fallback.omniroute_embed_model().to_owned()),
    }
}

fn integer_value(settings: &[ApplicationSetting], setting_key: &str) -> Option<i64> {
    settings
        .iter()
        .find(|setting| setting.setting_key == setting_key)
        .and_then(|setting| setting.value.as_i64())
}
```

### `backend/src/platform/settings/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/constants.rs`
- Size bytes / Размер в байтах: `289`
- Included characters / Включено символов: `289`
- Truncated / Обрезано: `no`

```rust
pub(crate) const SECRET_LIKE_MARKERS: [&str; 5] =
    ["secret", "password", "token", "credential", "private_key"];
pub(crate) const UI_STATE_FORBIDDEN_KEYS: [&str; 7] =
    ["body", "html", "raw", "text", "password", "token", "secret"];
pub(crate) const UI_STATE_MAX_BYTES: u64 = 65_536;
```

### `backend/src/platform/settings/definitions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions.rs`
- Size bytes / Размер в байтах: `905`
- Included characters / Включено символов: `905`
- Truncated / Обрезано: `no`

```rust
mod ai;
mod frontend;
mod privacy;
mod server;
#[cfg(test)]
mod tests;
mod ui;

use super::models::DeclaredApplicationSetting;

pub(crate) fn declared_setting_keys() -> Vec<String> {
    declared_application_settings()
        .into_iter()
        .map(|setting| setting.setting_key.to_owned())
        .collect()
}

pub(crate) fn declared_setting(setting_key: &str) -> Option<DeclaredApplicationSetting> {
    declared_application_settings()
        .into_iter()
        .find(|setting| setting.setting_key == setting_key)
}

pub(crate) fn declared_application_settings() -> Vec<DeclaredApplicationSetting> {
    let mut settings = Vec::new();
    settings.extend(server::declared_settings());
    settings.extend(frontend::declared_settings());
    settings.extend(privacy::declared_settings());
    settings.extend(ai::declared_settings());
    settings.extend(ui::declared_settings());
    settings
}
```

### `backend/src/platform/settings/definitions/ai.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/ai.rs`
- Size bytes / Размер в байтах: `369`
- Included characters / Включено символов: `369`
- Truncated / Обрезано: `no`

```rust
mod models;
mod provider;
mod runtime;

use super::super::models::DeclaredApplicationSetting;

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    let mut settings = Vec::new();
    settings.extend(provider::declared_settings());
    settings.extend(models::declared_settings());
    settings.extend(runtime::declared_settings());
    settings
}
```

### `backend/src/platform/settings/definitions/ai/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/ai/models.rs`
- Size bytes / Размер в байтах: `2462`
- Included characters / Включено символов: `2462`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![
        DeclaredApplicationSetting {
            setting_key: "ai.chat_model",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("qwen3:4b"),
            label: "Chat model",
            description: "Ollama model used for chat and source-backed answers.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "qwen3:4b"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.omniroute_chat_model",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("codex/gpt-5.5"),
            label: "OmniRoute chat model",
            description: "OpenAI-compatible OmniRoute model used for chat and source-backed answers when ai.provider is omniroute.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "codex/gpt-5.5"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.embedding_model",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("qwen3-embedding:4b"),
            label: "Embedding model",
            description: "Ollama model used for semantic embeddings.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "qwen3-embedding:4b"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.omniroute_embedding_model",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("openai-compatible-chat-ollama-pve/qwen3-embedding:4b"),
            label: "OmniRoute embedding model",
            description: "OpenAI-compatible OmniRoute embedding model. It must return 2560 dimensions until the semantic index shape changes.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
                "required_dimension": 2560
            }),
            is_editable: true,
        },
    ]
}
```

### `backend/src/platform/settings/definitions/ai/provider.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/ai/provider.rs`
- Size bytes / Размер в байтах: `2008`
- Included characters / Включено символов: `2008`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![
        DeclaredApplicationSetting {
            setting_key: "ai.provider",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("ollama"),
            label: "AI provider",
            description: "AI runtime provider. Ollama is local by default; OmniRoute is explicit opt-in and uses an env-backed API key.",
            metadata: json!({
                "ui_control": "select",
                "allowed_values": ["ollama", "omniroute"],
                "stores_secret": false
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.ollama_base_url",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("http://127.0.0.1:11434"),
            label: "Ollama base URL",
            description: "Local Ollama HTTP endpoint used by AI runtime requests.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "http://127.0.0.1:11434"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.omniroute_base_url",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("https://ai.sh-inc.ru/v1"),
            label: "OmniRoute base URL",
            description: "OpenAI-compatible OmniRoute endpoint. API key is read from MAKOSH_OMNIROUTE_API_KEY, never from application settings.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "https://ai.sh-inc.ru/v1",
                "stores_secret": false,
                "key_env": "MAKOSH_OMNIROUTE_API_KEY"
            }),
            is_editable: true,
        },
    ]
}
```

### `backend/src/platform/settings/definitions/ai/runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/ai/runtime.rs`
- Size bytes / Размер в байтах: `647`
- Included characters / Включено символов: `647`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![DeclaredApplicationSetting {
        setting_key: "ai.timeout_seconds",
        category: "ai",
        value_kind: SettingValueKind::Integer,
        default_value: json!(120),
        label: "AI request timeout",
        description: "Timeout in seconds for Ollama HTTP requests.",
        metadata: json!({
            "ui_control": "number",
            "min": 1,
            "max": 600,
            "step": 1
        }),
        is_editable: true,
    }]
}
```

### `backend/src/platform/settings/definitions/frontend.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/frontend.rs`
- Size bytes / Размер в байтах: `437`
- Included characters / Включено символов: `437`
- Truncated / Обрезано: `no`

```rust
mod appearance;
mod bootstrap;
mod layout;
mod state;

use super::super::models::DeclaredApplicationSetting;

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    let mut settings = Vec::new();
    settings.extend(bootstrap::declared_settings());
    settings.extend(layout::declared_settings());
    settings.extend(appearance::declared_settings());
    settings.extend(state::declared_settings());
    settings
}
```

### `backend/src/platform/settings/definitions/frontend/appearance.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/frontend/appearance.rs`
- Size bytes / Размер в байтах: `1728`
- Included characters / Включено символов: `1728`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![DeclaredApplicationSetting {
        setting_key: "frontend.theme",
        category: "frontend",
        value_kind: SettingValueKind::Json,
        default_value: json!({
            "schemaVersion": 1,
            "shellBackground": "network-mesh",
            "backgroundBrightness": 70,
            "accentColor": "teal",
            "panelOpacity": 70,
            "panelBlur": 12
        }),
        label: "Frontend appearance",
        description: "Desktop shell background, image brightness, panel transparency, panel blur and accent color. Stores visual preferences only, never message bodies, document text or secrets.",
        metadata: json!({
            "ui_control": "appearance",
            "schema_version": 1,
            "allowed_backgrounds": [
                "none",
                "network-mesh",
                "data-stream",
                "node-frame",
                "eclipse-grid",
                "dna-blueprint",
                "forest-network",
                "forest-stream",
                "knowledge-map",
                "rune-gold",
                "rune-teal"
            ],
            "allowed_brightness": [30, 40, 50, 60, 70, 80, 90, 100],
            "allowed_accent_colors": ["teal", "cyan", "blue", "violet", "amber", "rose"],
            "allowed_panel_opacity": [40, 50, 60, 70, 80, 90, 100],
            "allowed_panel_blur": [0, 4, 8, 12, 16, 20, 24],
            "stores_private_content": false,
            "restart_required": false
        }),
        is_editable: true,
    }]
}
```

### `backend/src/platform/settings/definitions/frontend/bootstrap.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/frontend/bootstrap.rs`
- Size bytes / Размер в байтах: `770`
- Included characters / Включено символов: `770`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![DeclaredApplicationSetting {
        setting_key: "frontend.api_base_url",
        category: "frontend",
        value_kind: SettingValueKind::String,
        default_value: json!("http://127.0.0.1:8080"),
        label: "Frontend API base URL",
        description: "Backend URL used by the desktop shell after it has loaded local settings.",
        metadata: json!({
            "ui_control": "text",
            "placeholder": "http://127.0.0.1:8080",
            "bootstrap": true,
            "env_var": "VITE_MAKOSH_API_BASE_URL"
        }),
        is_editable: true,
    }]
}
```

### `backend/src/platform/settings/definitions/frontend/layout.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/frontend/layout.rs`
- Size bytes / Размер в байтах: `2609`
- Included characters / Включено символов: `2609`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![layout_setting(), sidebar_setting()]
}

fn layout_setting() -> DeclaredApplicationSetting {
    DeclaredApplicationSetting {
        setting_key: "frontend.layout",
        category: "frontend",
        value_kind: SettingValueKind::Json,
        default_value: json!({
            "schemaVersion": 2,
            "views": {}
        }),
        label: "Frontend layout",
        description: "Desktop widget layout preset selections and user overrides. Stores layout metadata only, never message bodies, document text or secrets.",
        metadata: json!({
            "ui_control": "json",
            "schema_version": 2,
            "stores_private_content": false,
            "restart_required": false
        }),
        is_editable: true,
    }
}

fn sidebar_setting() -> DeclaredApplicationSetting {
    DeclaredApplicationSetting {
        setting_key: "frontend.sidebar",
        category: "frontend",
        value_kind: SettingValueKind::Json,
        default_value: json!({
            "schemaVersion": 3,
            "rootItemIds": [
                "home",
                "group:communications",
                "persons",
                "projects",
                "tasks",
                "calendar",
                "documents",
                "notes",
                "knowledge",
                "agents"
            ],
            "groups": [
                {
                    "id": "communications",
                    "label": "Communications",
                    "icon": "tabler:messages",
                    "itemIds": [
                        "communications.mail",
                        "communications.telegram",
                        "communications.whatsapp",
                        "communications.calls",
                        "communications.meetings",
                        "timeline"
                    ],
                    "separatorBeforeItemIds": []
                }
            ],
            "hiddenItemIds": []
        }),
        label: "Frontend sidebar",
        description: "Desktop sidebar grouping, item order and hidden workspace metadata. Stores navigation preferences only, never message bodies, document text or secrets.",
        metadata: json!({
            "ui_control": "json",
            "schema_version": 3,
            "stores_private_content": false,
            "restart_required": false
        }),
        is_editable: true,
    }
}
```

### `backend/src/platform/settings/definitions/frontend/state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/frontend/state.rs`
- Size bytes / Размер в байтах: `1741`
- Included characters / Включено символов: `1741`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::super::constants::{UI_STATE_FORBIDDEN_KEYS, UI_STATE_MAX_BYTES};
use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![locale_setting(), ui_state_setting()]
}

fn locale_setting() -> DeclaredApplicationSetting {
    DeclaredApplicationSetting {
        setting_key: "frontend.locale",
        category: "frontend",
        value_kind: SettingValueKind::String,
        default_value: json!("en"),
        label: "Frontend locale",
        description: "Desktop interface language preference. Stores only the selected locale code.",
        metadata: json!({
            "ui_control": "language",
            "allowed_values": ["en", "ru"],
            "stores_private_content": false,
            "restart_required": false
        }),
        is_editable: true,
    }
}

fn ui_state_setting() -> DeclaredApplicationSetting {
    DeclaredApplicationSetting {
        setting_key: "frontend.ui_state",
        category: "frontend",
        value_kind: SettingValueKind::Json,
        default_value: json!({
            "schemaVersion": 1
        }),
        label: "Frontend UI state",
        description: "Transient desktop UI state for restoring visible workspace context. Stores non-authoritative UI metadata only, never message bodies, document text or secrets.",
        metadata: json!({
            "ui_control": "hidden",
            "schema_version": 1,
            "stores_private_content": false,
            "restart_required": false,
            "max_bytes": UI_STATE_MAX_BYTES,
            "forbidden_keys": UI_STATE_FORBIDDEN_KEYS
        }),
        is_editable: true,
    }
}
```

### `backend/src/platform/settings/definitions/privacy.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/privacy.rs`
- Size bytes / Размер в байтах: `4780`
- Included characters / Включено символов: `4780`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![
        DeclaredApplicationSetting {
            setting_key: "privacy.zoom_remote_transcript_download_enabled",
            category: "privacy",
            value_kind: SettingValueKind::Boolean,
            default_value: json!(false),
            label: "Zoom remote transcript downloads",
            description: "Allow Макошь to fetch transcript-like text files directly from Zoom recording URLs during webhook processing and manual provider sync.",
            metadata: json!({
                "ui_control": "checkbox",
                "stores_private_content": false,
                "scope": "zoom",
                "policy_kind": "owner_visible_opt_in",
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "privacy.zoom_remote_recording_download_enabled",
            category: "privacy",
            value_kind: SettingValueKind::Boolean,
            default_value: json!(false),
            label: "Zoom remote recording downloads",
            description: "Allow Макошь to fetch non-transcript Zoom recording files directly from Zoom recording URLs during manual provider sync and scheduled recording sync.",
            metadata: json!({
                "ui_control": "checkbox",
                "stores_private_content": true,
                "scope": "zoom",
                "policy_kind": "owner_visible_opt_in",
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "privacy.zoom_recording_import_retention_days",
            category: "privacy",
            value_kind: SettingValueKind::Integer,
            default_value: json!(0),
            label: "Zoom recording import retention (days)",
            description: "Owner-visible retention policy for imported Zoom recording blobs. Set to 0 to retain local imports until explicit removal.",
            metadata: json!({
                "ui_control": "number",
                "stores_private_content": true,
                "scope": "zoom",
                "policy_kind": "owner_visible_retention",
                "min": 0,
                "max": 3650,
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "privacy.zoom_transcript_retention_days",
            category: "privacy",
            value_kind: SettingValueKind::Integer,
            default_value: json!(0),
            label: "Zoom transcript retention (days)",
            description: "Owner-visible retention policy for Zoom transcript evidence imported or observed by Макошь. Set to 0 to retain transcript evidence until explicit review or later manual cleanup.",
            metadata: json!({
                "ui_control": "number",
                "stores_private_content": true,
                "scope": "zoom",
                "policy_kind": "owner_visible_retention",
                "min": 0,
                "max": 3650,
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "privacy.yandex_telemost_recording_retention_days",
            category: "privacy",
            value_kind: SettingValueKind::Integer,
            default_value: json!(0),
            label: "Yandex Telemost recording retention (days)",
            description: "Owner-visible retention policy for local Telemost MP3 capture artifacts. Set to 0 to disable automatic cleanup.",
            metadata: json!({
                "ui_control": "number",
                "stores_private_content": true,
                "scope": "yandex_telemost",
                "policy_kind": "owner_visible_retention",
                "min": 0,
                "max": 3650,
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "privacy.yandex_telemost_speaker_timeline_retention_days",
            category: "privacy",
            value_kind: SettingValueKind::Integer,
            default_value: json!(0),
            label: "Yandex Telemost speaker hint retention (days)",
            description: "Owner-visible retention policy for local Telemost speaker timeline hint files. Set to 0 to disable automatic cleanup.",
            metadata: json!({
                "ui_control": "number",
                "stores_private_content": true,
                "scope": "yandex_telemost",
                "policy_kind": "owner_visible_retention",
                "min": 0,
                "max": 3650,
            }),
            is_editable: true,
        },
    ]
}
```

### `backend/src/platform/settings/definitions/server.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/server.rs`
- Size bytes / Размер в байтах: `1355`
- Included characters / Включено символов: `1355`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![
        DeclaredApplicationSetting {
            setting_key: "server.http_addr",
            category: "server",
            value_kind: SettingValueKind::String,
            default_value: json!("127.0.0.1:8080"),
            label: "Backend HTTP bind",
            description: "Backend HTTP address used when the local server starts. Changes require a backend restart.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "127.0.0.1:8080",
                "restart_required": true,
                "bootstrap": true,
                "env_var": "MAKOSH_HTTP_ADDR"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "signal_hub.active_profile",
            category: "signal_hub",
            value_kind: SettingValueKind::String,
            default_value: json!("production"),
            label: "Active Signal Hub profile",
            description: "Operational Signal Hub profile applied to managed source policies.",
            metadata: json!({
                "ui_control": "hidden"
            }),
            is_editable: true,
        },
    ]
}
```

### `backend/src/platform/settings/definitions/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/tests.rs`
- Size bytes / Размер в байтах: `7359`
- Included characters / Включено символов: `7359`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::models::SettingValueKind;
use super::*;
use crate::platform::settings::SettingsError;

#[test]
fn frontend_locale_setting_is_declared_as_editable_string() {
    let setting = declared_setting("frontend.locale").expect("frontend locale setting");

    assert_eq!(setting.category, "frontend");
    assert_eq!(setting.value_kind, SettingValueKind::String);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!("en"));
    assert_eq!(setting.metadata["ui_control"], json!("language"));
    assert_eq!(setting.metadata["allowed_values"], json!(["en", "ru"]));
    assert_eq!(setting.metadata["stores_private_content"], json!(false));
}

#[test]
fn frontend_ui_state_setting_is_declared_as_hidden_json() {
    let setting = declared_setting("frontend.ui_state").expect("frontend ui state setting");

    assert_eq!(setting.category, "frontend");
    assert_eq!(setting.value_kind, SettingValueKind::Json);
    assert!(setting.is_editable);
    assert_eq!(setting.metadata["ui_control"], json!("hidden"));
    assert_eq!(setting.metadata["schema_version"], json!(1));
    assert_eq!(setting.metadata["stores_private_content"], json!(false));
    assert_eq!(setting.default_value["schemaVersion"], json!(1));
}

#[test]
fn frontend_ui_state_rejects_private_content_keys() {
    let setting = declared_setting("frontend.ui_state").expect("frontend ui state setting");
    let value = json!({
        "schemaVersion": 1,
        "savedAt": "2026-06-11T12:00:00Z",
        "expiresAt": "2026-06-18T12:00:00Z",
        "communications": {
            "selectedMessageId": "msg-1",
            "compose": {
                "draftId": "draft-1",
                "body": "private draft body"
            }
        }
    });

    let error = setting
        .value_kind
        .validate_value(&value, &setting.metadata)
        .expect_err("private body key rejected");

    assert!(matches!(error, SettingsError::InvalidValue(_)));
}

#[test]
fn frontend_ui_state_rejects_oversized_snapshots() {
    let setting = declared_setting("frontend.ui_state").expect("frontend ui state setting");
    let value = json!({
        "schemaVersion": 1,
        "savedAt": "2026-06-11T12:00:00Z",
        "expiresAt": "2026-06-18T12:00:00Z",
        "shell": {
            "expandedSidebarGroupIds": vec!["communications"; 10_000]
        }
    });

    let error = setting
        .value_kind
        .validate_value(&value, &setting.metadata)
        .expect_err("oversized snapshot rejected");

    assert!(matches!(error, SettingsError::InvalidValue(_)));
}

#[test]
fn zoom_remote_transcript_download_setting_is_declared_as_opt_in_boolean() {
    let setting = declared_setting("privacy.zoom_remote_transcript_download_enabled")
        .expect("zoom transcript download policy setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Boolean);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(false));
    assert_eq!(setting.metadata["ui_control"], json!("checkbox"));
    assert_eq!(setting.metadata["scope"], json!("zoom"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_opt_in")
    );
}

#[test]
fn zoom_remote_recording_download_setting_is_declared_as_opt_in_boolean() {
    let setting = declared_setting("privacy.zoom_remote_recording_download_enabled")
        .expect("zoom recording download policy setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Boolean);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(false));
    assert_eq!(setting.metadata["ui_control"], json!("checkbox"));
    assert_eq!(setting.metadata["scope"], json!("zoom"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_opt_in")
    );
    assert_eq!(setting.metadata["stores_private_content"], json!(true));
}

#[test]
fn zoom_recording_import_retention_setting_is_declared_as_editable_integer() {
    let setting = declared_setting("privacy.zoom_recording_import_retention_days")
        .expect("zoom recording import retention setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Integer);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(0));
    assert_eq!(setting.metadata["ui_control"], json!("number"));
    assert_eq!(setting.metadata["scope"], json!("zoom"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_retention")
    );
    assert_eq!(setting.metadata["min"], json!(0));
    assert_eq!(setting.metadata["max"], json!(3650));
    assert_eq!(setting.metadata["stores_private_content"], json!(true));
}

#[test]
fn zoom_transcript_retention_setting_is_declared_as_editable_integer() {
    let setting = declared_setting("privacy.zoom_transcript_retention_days")
        .expect("zoom transcript retention setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Integer);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(0));
    assert_eq!(setting.metadata["ui_control"], json!("number"));
    assert_eq!(setting.metadata["scope"], json!("zoom"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_retention")
    );
    assert_eq!(setting.metadata["min"], json!(0));
    assert_eq!(setting.metadata["max"], json!(3650));
    assert_eq!(setting.metadata["stores_private_content"], json!(true));
}

#[test]
fn yandex_telemost_recording_retention_setting_is_declared_as_editable_integer() {
    let setting = declared_setting("privacy.yandex_telemost_recording_retention_days")
        .expect("yandex telemost recording retention setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Integer);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(0));
    assert_eq!(setting.metadata["ui_control"], json!("number"));
    assert_eq!(setting.metadata["scope"], json!("yandex_telemost"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_retention")
    );
    assert_eq!(setting.metadata["min"], json!(0));
    assert_eq!(setting.metadata["max"], json!(3650));
    assert_eq!(setting.metadata["stores_private_content"], json!(true));
}

#[test]
fn yandex_telemost_speaker_timeline_retention_setting_is_declared_as_editable_integer() {
    let setting = declared_setting("privacy.yandex_telemost_speaker_timeline_retention_days")
        .expect("yandex telemost speaker timeline retention setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Integer);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(0));
    assert_eq!(setting.metadata["ui_control"], json!("number"));
    assert_eq!(setting.metadata["scope"], json!("yandex_telemost"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_retention")
    );
    assert_eq!(setting.metadata["min"], json!(0));
    assert_eq!(setting.metadata["max"], json!(3650));
    assert_eq!(setting.metadata["stores_private_content"], json!(true));
}
```

### `backend/src/platform/settings/definitions/ui.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/definitions/ui.rs`
- Size bytes / Размер в байтах: `1162`
- Included characters / Включено символов: `1162`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![
        DeclaredApplicationSetting {
            setting_key: "ui.theme",
            category: "ui",
            value_kind: SettingValueKind::String,
            default_value: json!("system"),
            label: "Theme",
            description: "Desktop shell color theme preference.",
            metadata: json!({
                "ui_control": "select",
                "allowed_values": ["system", "dark", "light"]
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ui.density",
            category: "ui",
            value_kind: SettingValueKind::String,
            default_value: json!("comfortable"),
            label: "UI density",
            description: "Desktop shell spacing density preference.",
            metadata: json!({
                "ui_control": "select",
                "allowed_values": ["comfortable", "compact"]
            }),
            is_editable: true,
        },
    ]
}
```

### `backend/src/platform/settings/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/errors.rs`
- Size bytes / Размер в байтах: `1128`
- Included characters / Включено символов: `1128`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("unsupported setting value kind: {0}")]
    UnsupportedValueKind(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("setting key has invalid format")]
    InvalidSettingKey,

    #[error("setting key must not refer to secrets or credentials")]
    SecretLikeSettingKey,

    #[error("invalid setting value: {0}")]
    InvalidValue(&'static str),

    #[error("application setting was not found: {setting_key}")]
    SettingNotFound { setting_key: String },

    #[error("application setting is read-only: {setting_key}")]
    ReadOnlySetting { setting_key: String },
}

impl SettingsError {
    pub fn is_invalid_request(&self) -> bool {
        matches!(
            self,
            Self::UnsupportedValueKind(_)
                | Self::EmptyField(_)
                | Self::InvalidSettingKey
                | Self::SecretLikeSettingKey
                | Self::InvalidValue(_)
                | Self::ReadOnlySetting { .. }
        )
    }
}
```

### `backend/src/platform/settings/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/models.rs`
- Size bytes / Размер в байтах: `4127`
- Included characters / Включено символов: `4127`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::SettingsError;
use super::validation::validate_json_metadata_constraints;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApplicationSettingsRepairSummary {
    pub inserted: u64,
    pub repaired: u64,
    pub reset_values: u64,
}

impl ApplicationSettingsRepairSummary {
    pub fn changed(&self) -> bool {
        self.inserted > 0 || self.repaired > 0 || self.reset_values > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeclaredApplicationSetting {
    pub(crate) setting_key: &'static str,
    pub(crate) category: &'static str,
    pub(crate) value_kind: SettingValueKind,
    pub(crate) default_value: Value,
    pub(crate) label: &'static str,
    pub(crate) description: &'static str,
    pub(crate) metadata: Value,
    pub(crate) is_editable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApplicationSetting {
    pub setting_key: String,
    pub category: String,
    pub value_kind: SettingValueKind,
    pub value: Value,
    pub label: String,
    pub description: String,
    pub metadata: Value,
    pub is_editable: bool,
    pub updated_by_actor_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingValueKind {
    Boolean,
    Integer,
    String,
    Json,
}

impl SettingValueKind {
    pub(crate) fn db_value(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::String => "string",
            Self::Json => "json",
        }
    }

    pub(crate) fn validate_value(
        self,
        value: &Value,
        metadata: &Value,
    ) -> Result<(), SettingsError> {
        match self {
            Self::Boolean if !value.is_boolean() => {
                return Err(SettingsError::InvalidValue("value must be a boolean"));
            }
            Self::Integer if value.as_i64().is_none() => {
                return Err(SettingsError::InvalidValue("value must be an integer"));
            }
            Self::String if value.as_str().is_none() => {
                return Err(SettingsError::InvalidValue("value must be a string"));
            }
            Self::Json if !(value.is_object() || value.is_array()) => {
                return Err(SettingsError::InvalidValue(
                    "value must be a JSON object or array",
                ));
            }
            _ => {}
        }

        if let Some(allowed_values) = metadata.get("allowed_values").and_then(Value::as_array) {
            let is_allowed = allowed_values.iter().any(|allowed| allowed == value);
            if !is_allowed {
                return Err(SettingsError::InvalidValue(
                    "value is not allowed for this setting",
                ));
            }
        }

        if let Some(value) = value.as_i64() {
            if let Some(min) = metadata.get("min").and_then(Value::as_i64)
                && value < min
            {
                return Err(SettingsError::InvalidValue(
                    "value is below the allowed minimum",
                ));
            }
            if let Some(max) = metadata.get("max").and_then(Value::as_i64)
                && value > max
            {
                return Err(SettingsError::InvalidValue(
                    "value is above the allowed maximum",
                ));
            }
        }

        validate_json_metadata_constraints(value, metadata)?;

        Ok(())
    }
}

impl TryFrom<&str> for SettingValueKind {
    type Error = SettingsError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "boolean" => Ok(Self::Boolean),
            "integer" => Ok(Self::Integer),
            "string" => Ok(Self::String),
            "json" => Ok(Self::Json),
            _ => Err(SettingsError::UnsupportedValueKind(value.to_owned())),
        }
    }
}
```

### `backend/src/platform/settings/persistence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/persistence.rs`
- Size bytes / Размер в байтах: `6579`
- Included characters / Включено символов: `6579`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::SettingsError;
use super::models::{ApplicationSetting, DeclaredApplicationSetting, SettingValueKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExistingApplicationSettingRow {
    pub(crate) category: String,
    pub(crate) value_kind: String,
    pub(crate) value: Value,
    pub(crate) label: String,
    pub(crate) description: String,
    pub(crate) metadata: Value,
    pub(crate) is_editable: bool,
}

impl ExistingApplicationSettingRow {
    pub(crate) fn is_value_compatible_with(&self, declared: &DeclaredApplicationSetting) -> bool {
        SettingValueKind::try_from(self.value_kind.as_str())
            .is_ok_and(|value_kind| value_kind == declared.value_kind)
            && declared
                .value_kind
                .validate_value(&self.value, &declared.metadata)
                .is_ok()
    }

    pub(crate) fn needs_repair(
        &self,
        declared: &DeclaredApplicationSetting,
        next_value: &Value,
    ) -> bool {
        self.category != declared.category
            || self.value_kind != declared.value_kind.db_value()
            || &self.value != next_value
            || self.label != declared.label
            || self.description != declared.description
            || self.metadata != declared.metadata
            || self.is_editable != declared.is_editable
    }
}

pub(crate) async fn ensure_application_settings_table(pool: &PgPool) -> Result<(), SettingsError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS application_settings (
            setting_key TEXT PRIMARY KEY,
            category TEXT NOT NULL,
            value_kind TEXT NOT NULL,
            value JSONB NOT NULL,
            label TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
            is_editable BOOLEAN NOT NULL DEFAULT true,
            updated_by_actor_id TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

            CONSTRAINT application_settings_key_not_empty CHECK (length(trim(setting_key)) > 0),
            CONSTRAINT application_settings_key_format CHECK (setting_key ~ '^[a-z0-9][a-z0-9_.-]*[a-z0-9]$'),
            CONSTRAINT application_settings_key_not_secret_like CHECK (
                setting_key !~* '(secret|password|token|credential|private_key)'
            ),
            CONSTRAINT application_settings_category_not_empty CHECK (length(trim(category)) > 0),
            CONSTRAINT application_settings_label_not_empty CHECK (length(trim(label)) > 0),
            CONSTRAINT application_settings_value_kind CHECK (
                value_kind IN ('boolean', 'integer', 'string', 'json')
            ),
            CONSTRAINT application_settings_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS application_settings_category_idx
            ON application_settings (category, setting_key)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn fetch_existing_setting_row(
    pool: &PgPool,
    setting_key: &str,
) -> Result<Option<ExistingApplicationSettingRow>, SettingsError> {
    let Some(row) = sqlx::query(
        r#"
        SELECT
            category,
            value_kind,
            value,
            label,
            description,
            metadata,
            is_editable
        FROM application_settings
        WHERE setting_key = $1
        "#,
    )
    .bind(setting_key)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(ExistingApplicationSettingRow {
        category: row.try_get("category")?,
        value_kind: row.try_get("value_kind")?,
        value: row.try_get("value")?,
        label: row.try_get("label")?,
        description: row.try_get("description")?,
        metadata: row.try_get("metadata")?,
        is_editable: row.try_get("is_editable")?,
    }))
}

pub(crate) async fn insert_declared_setting(
    pool: &PgPool,
    declared: &DeclaredApplicationSetting,
) -> Result<(), SettingsError> {
    sqlx::query(
        r#"
        INSERT INTO application_settings (
            setting_key,
            category,
            value_kind,
            value,
            label,
            description,
            metadata,
            is_editable,
            updated_by_actor_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'system:settings_repair')
        "#,
    )
    .bind(declared.setting_key)
    .bind(declared.category)
    .bind(declared.value_kind.db_value())
    .bind(&declared.default_value)
    .bind(declared.label)
    .bind(declared.description)
    .bind(&declared.metadata)
    .bind(declared.is_editable)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn update_declared_setting(
    pool: &PgPool,
    declared: &DeclaredApplicationSetting,
    next_value: &Value,
) -> Result<(), SettingsError> {
    sqlx::query(
        r#"
        UPDATE application_settings
        SET
            category = $2,
            value_kind = $3,
            value = $4,
            label = $5,
            description = $6,
            metadata = $7,
            is_editable = $8,
            updated_by_actor_id = 'system:settings_repair',
            updated_at = now()
        WHERE setting_key = $1
        "#,
    )
    .bind(declared.setting_key)
    .bind(declared.category)
    .bind(declared.value_kind.db_value())
    .bind(next_value)
    .bind(declared.label)
    .bind(declared.description)
    .bind(&declared.metadata)
    .bind(declared.is_editable)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) fn row_to_setting(row: PgRow) -> Result<ApplicationSetting, SettingsError> {
    let value_kind = SettingValueKind::try_from(row.try_get::<String, _>("value_kind")?.as_str())?;

    Ok(ApplicationSetting {
        setting_key: row.try_get("setting_key")?,
        category: row.try_get("category")?,
        value_kind,
        value: row.try_get("value")?,
        label: row.try_get("label")?,
        description: row.try_get("description")?,
        metadata: row.try_get("metadata")?,
        is_editable: row.try_get("is_editable")?,
        updated_by_actor_id: row.try_get("updated_by_actor_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/platform/settings/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/store.rs`
- Size bytes / Размер в байтах: `5899`
- Included characters / Включено символов: `5899`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::platform::config::AppConfig;

use super::ai_runtime::{AiRuntimeSettings, runtime_settings_from_values};
use super::definitions::{declared_application_settings, declared_setting, declared_setting_keys};
use super::errors::SettingsError;
use super::models::{ApplicationSetting, ApplicationSettingsRepairSummary};
use super::persistence::{
    ensure_application_settings_table, fetch_existing_setting_row, insert_declared_setting,
    row_to_setting, update_declared_setting,
};
use super::validation::{validate_declared_setting, validate_non_empty, validate_setting_key};

#[derive(Clone)]
pub struct ApplicationSettingsStore {
    pool: PgPool,
}

impl ApplicationSettingsStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_settings(&self) -> Result<Vec<ApplicationSetting>, SettingsError> {
        let setting_keys = declared_setting_keys();
        let rows = sqlx::query(
            r#"
            SELECT
                setting_key,
                category,
                value_kind,
                value,
                label,
                description,
                metadata,
                is_editable,
                updated_by_actor_id,
                created_at,
                updated_at
            FROM application_settings
            WHERE setting_key = ANY($1)
            ORDER BY category ASC, setting_key ASC
            "#,
        )
        .bind(&setting_keys)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_setting).collect()
    }

    pub async fn setting(
        &self,
        setting_key: &str,
    ) -> Result<Option<ApplicationSetting>, SettingsError> {
        validate_setting_key(setting_key)?;
        let setting_key = setting_key.trim();
        if declared_setting(setting_key).is_none() {
            return Ok(None);
        }

        let row = sqlx::query(
            r#"
            SELECT
                setting_key,
                category,
                value_kind,
                value,
                label,
                description,
                metadata,
                is_editable,
                updated_by_actor_id,
                created_at,
                updated_at
            FROM application_settings
            WHERE setting_key = $1
            "#,
        )
        .bind(setting_key)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_setting).transpose()
    }

    pub async fn update_setting_value(
        &self,
        setting_key: &str,
        value: &Value,
        actor_id: &str,
    ) -> Result<ApplicationSetting, SettingsError> {
        validate_setting_key(setting_key)?;
        validate_non_empty("actor_id", actor_id)?;
        let setting_key = setting_key.trim();

        if declared_setting(setting_key).is_none() {
            return Err(SettingsError::SettingNotFound {
                setting_key: setting_key.to_owned(),
            });
        };

        let existing = match self.setting(setting_key).await? {
            Some(setting) => setting,
            None => {
                self.repair_declared_settings().await?;
                self.setting(setting_key)
                    .await?
                    .ok_or_else(|| SettingsError::SettingNotFound {
                        setting_key: setting_key.to_owned(),
                    })?
            }
        };

        if !existing.is_editable {
            return Err(SettingsError::ReadOnlySetting {
                setting_key: existing.setting_key,
            });
        }
        existing
            .value_kind
            .validate_value(value, &existing.metadata)?;

        let row = sqlx::query(
            r#"
            UPDATE application_settings
            SET
                value = $2,
                updated_by_actor_id = $3,
                updated_at = now()
            WHERE setting_key = $1
            RETURNING
                setting_key,
                category,
                value_kind,
                value,
                label,
                description,
                metadata,
                is_editable,
                updated_by_actor_id,
                created_at,
                updated_at
            "#,
        )
        .bind(setting_key)
        .bind(value)
        .bind(actor_id.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_setting(row)
    }

    pub async fn ai_runtime_settings(
        &self,
        fallback: &AppConfig,
    ) -> Result<AiRuntimeSettings, SettingsError> {
        let settings = self.list_settings().await?;

        Ok(runtime_settings_from_values(&settings, fallback))
    }

    pub async fn repair_declared_settings(
        &self,
    ) -> Result<ApplicationSettingsRepairSummary, SettingsError> {
        ensure_application_settings_table(&self.pool).await?;

        let mut summary = ApplicationSettingsRepairSummary::default();
        for declared in declared_application_settings() {
            validate_declared_setting(&declared)?;
            let Some(existing) =
                fetch_existing_setting_row(&self.pool, declared.setting_key).await?
            else {
                insert_declared_setting(&self.pool, &declared).await?;
                summary.inserted += 1;
                continue;
            };

            let next_value = if existing.is_value_compatible_with(&declared) {
                existing.value.clone()
            } else {
                summary.reset_values += 1;
                declared.default_value.clone()
            };

            if existing.needs_repair(&declared, &next_value) {
                update_declared_setting(&self.pool, &declared, &next_value).await?;
                summary.repaired += 1;
            }
        }

        Ok(summary)
    }
}
```

### `backend/src/platform/settings/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/settings/validation.rs`
- Size bytes / Размер в байтах: `3839`
- Included characters / Включено символов: `3839`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::constants::SECRET_LIKE_MARKERS;
use super::errors::SettingsError;
use super::models::DeclaredApplicationSetting;

pub(crate) fn validate_declared_setting(
    declared: &DeclaredApplicationSetting,
) -> Result<(), SettingsError> {
    validate_setting_key(declared.setting_key)?;
    validate_non_empty("category", declared.category)?;
    validate_non_empty("label", declared.label)?;
    if !declared.metadata.is_object() {
        return Err(SettingsError::InvalidValue(
            "metadata must be a JSON object",
        ));
    }
    declared
        .value_kind
        .validate_value(&declared.default_value, &declared.metadata)?;

    Ok(())
}

pub(crate) fn validate_json_metadata_constraints(
    value: &Value,
    metadata: &Value,
) -> Result<(), SettingsError> {
    if let Some(max_bytes) = metadata.get("max_bytes").and_then(Value::as_u64)
        && (value.to_string().len() as u64) > max_bytes
    {
        return Err(SettingsError::InvalidValue(
            "JSON value exceeds maximum size",
        ));
    }

    let forbidden_keys = metadata
        .get("forbidden_keys")
        .and_then(Value::as_array)
        .map(|keys| {
            keys.iter()
                .filter_map(Value::as_str)
                .map(str::to_ascii_lowercase)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !forbidden_keys.is_empty() && json_value_has_forbidden_key(value, &forbidden_keys) {
        return Err(SettingsError::InvalidValue(
            "JSON value contains private content keys",
        ));
    }

    Ok(())
}

fn json_value_has_forbidden_key(value: &Value, forbidden_keys: &[String]) -> bool {
    match value {
        Value::Object(object) => object.iter().any(|(key, child)| {
            is_forbidden_json_key(key, forbidden_keys)
                || json_value_has_forbidden_key(child, forbidden_keys)
        }),
        Value::Array(items) => items
            .iter()
            .any(|item| json_value_has_forbidden_key(item, forbidden_keys)),
        _ => false,
    }
}

fn is_forbidden_json_key(key: &str, forbidden_keys: &[String]) -> bool {
    let key = key.to_ascii_lowercase();
    forbidden_keys.iter().any(|marker| {
        key == *marker
            || key.starts_with(marker)
            || key.contains(&format!("_{marker}"))
            || key.contains(&format!("{marker}_"))
            || key.contains(&format!("-{marker}"))
            || key.contains(&format!("{marker}-"))
            || key.contains(&format!(".{marker}"))
            || key.contains(&format!("{marker}."))
            || (marker != "text" && key.ends_with(marker))
    })
}

pub(crate) fn validate_setting_key(setting_key: &str) -> Result<(), SettingsError> {
    validate_non_empty("setting_key", setting_key)?;
    let setting_key = setting_key.trim();
    let has_valid_format = setting_key.chars().all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '_' | '-' | '.')
    }) && setting_key
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        && setting_key
            .chars()
            .last()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit());
    if !has_valid_format {
        return Err(SettingsError::InvalidSettingKey);
    }

    if SECRET_LIKE_MARKERS
        .iter()
        .any(|marker| setting_key.contains(marker))
    {
        return Err(SettingsError::SecretLikeSettingKey);
    }

    Ok(())
}

pub(crate) fn validate_non_empty(field: &'static str, value: &str) -> Result<(), SettingsError> {
    if value.trim().is_empty() {
        return Err(SettingsError::EmptyField(field));
    }

    Ok(())
}
```

### `backend/src/platform/storage/communication_media.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/storage/communication_media.rs`
- Size bytes / Размер в байтах: `13104`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::future::Future;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::StorageError;

const LOCAL_FS_STORAGE_KIND: &str = "local_fs";
const SHA256_PREFIX: &str = "sha256:";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalBlobRecord {
    pub storage_kind: String,
    pub storage_path: String,
    pub sha256: String,
    pub size_bytes: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredBlobRecord {
    pub blob_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyScanStatus {
    NotScanned,
    Clean,
    Suspicious,
    Malicious,
    Failed,
}

impl SafetyScanStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotScanned => "not_scanned",
            Self::Clean => "clean",
            Self::Suspicious => "suspicious",
            Self::Malicious => "malicious",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SafetyScanReport {
    pub status: SafetyScanStatus,
    pub engine: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    pub summary: Option<String>,
    pub metadata: Value,
}

impl SafetyScanReport {
    pub fn not_scanned() -> Self {
        Self {
            status: SafetyScanStatus::NotScanned,
            engine: None,
            checked_at: None,
            summary: None,
            metadata: json!({}),
        }
    }
}

pub struct SafetyScanRequest<'a> {
    pub filename: Option<&'a str>,
    pub content_type: &'a str,
    pub size_bytes: i64,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedAttachmentRecord {
    pub attachment_id: String,
    pub account_id: Option<String>,
    pub channel_kind: Option<String>,
    pub blob_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub source_kind: String,
    pub imported_by: String,
    pub scan_status: SafetyScanStatus,
    pub scan_engine: Option<String>,
    pub scan_checked_at: Option<DateTime<Utc>>,
    pub scan_summary: Option<String>,
    pub scan_metadata: Value,
    pub metadata: Value,
    pub storage_kind: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedAttachmentRemovalResult {
    pub imported_attachment: ImportedAttachmentRecord,
    pub blob_metadata_removed: bool,
}

#[derive(Clone, Debug)]
pub struct ImportedAttachmentUpsert {
    pub attachment_id: String,
    pub account_id: String,
    pub channel_kind: String,
    pub blob_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub source_kind: String,
    pub imported_by: String,
    pub scan_report: SafetyScanReport,
    pub metadata: Value,
}

pub trait ImportedAttachmentStoragePort: Send + Sync {
    fn upsert_blob_record<'a>(
        &'a self,
        blob: &'a LocalBlobRecord,
        content_type: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<StoredBlobRecord, StorageError>> + Send + 'a>>;

    fn upsert_imported_attachment_record<'a>(
        &'a self,
        import: &'a ImportedAttachmentUpsert,
    ) -> Pin<Box<dyn Future<Output = Result<ImportedAttachmentRecord, StorageError>> + Send + 'a>>;

    fn list_imported_attachment_records<'a>(
        &'a self,
        account_id: &'a str,
        source_kind: &'a str,
        limit: i64,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<ImportedAttachmentRecord>, StorageError>> + Send + 'a>,
    >;

    fn list_expired_imported_attachment_records<'a>(
        &'a self,
        account_id: &'a str,
        source_kind: &'a str,
        limit: i64,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<ImportedAttachmentRecord>, StorageError>> + Send + 'a>,
    >;

    fn remove_imported_attachment_record<'a>(
        &'a self,
        attachment_id: &'a str,
        account_id: &'a str,
        source_kind: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ImportedAttachmentRemovalResult>, StorageError>>
                + Send
                + 'a,
        >,
    >;
}

pub fn new_attachment_import_id(seed: &str) -> String {
    format!("att-import:v1:{}:{}", seed.len(), seed)
}

pub async fn put_local_blob(root: &str, bytes: &[u8]) -> Result<LocalBlobRecord, StorageError> {
    let size_bytes = i64::try_from(bytes.len())
        .map_err(|_| StorageError::Invalid("blob too large".to_owned()))?;
    let digest_hex = sha256_hex(bytes);
    let storage_path = format!("sha256/{}/{}.blob", &digest_hex[..2], digest_hex);
    let absolute_path = Path::new(root).join(&storage_path);

    if let Some(parent) = absolute_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    if !path_exists(&absolute_path).await? {
        let temp_path = absolute_path.with_extension(format!(
            "tmp-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        tokio::fs::write(&temp_path, bytes).await?;
        tokio::fs::rename(&temp_path, &absolute_path).await?;
    }

    let actual_size = i64::try_from(tokio::fs::metadata(&absolute_path).await?.len())
        .map_err(|_| StorageError::Invalid("blob too large".to_owned()))?;
    if actual_size != size_bytes {
        return Err(StorageError::Invalid(format!(
            "blob size mismatch for {}: expected {}, actual {}",
            absolute_path.display(),
            size_bytes,
            actual_size
        )));
    }

    Ok(LocalBlobRecord {
        storage_kind: LOCAL_FS_STORAGE_KIND.to_owned(),
        storage_path,
        sha256: format!("{SHA256_PREFIX}{digest_hex}"),
        size_bytes,
    })
}

pub async fn delete_local_blob(root: &str, storage_path: &str) -> Result<bool, StorageError> {
    let storage_path = validate_storage_path(storage_path)?;
    let absolute_path = Path::new(root).join(&storage_path);
    match tokio::fs::remove_file(&absolute_path).await {
        Ok(()) => {
            prune_empty_parent_dirs(Path::new(root), &absolute_path).await?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub fn scan_attachment(request: &SafetyScanRequest<'_>) -> Result<SafetyScanReport, StorageError> {
    let extension = normalized_extension(request.filename);
    let content_type = normalized_content_type(request.content_type);
    let mut reasons = Vec::new();
    let mut status = SafetyScanStatus::NotScanned;

    if has_executable_magic(request.bytes) {
        status = SafetyScanStatus::Malicious;
        reasons.push("executable_magic");
    }

    if let Some(extension) = extension.as_deref() {
        if is_active_content_extension(extension) {
            status = SafetyScanStatus::Malicious;
            reasons.push("active_content_extension");
        } else if is_macro_document_extension(extension) {
            status = max_scan_status(status, SafetyScanStatus::Suspicious);
            reasons.push("macro_enabled_document_extension");
        }
    }

    if let Some(extension) = extension.as_deref()
        && is_mime_extension_mismatch(&content_type, extension)
    {
        status = max_scan_status(status, SafetyScanStatus::Suspicious);
        reasons.push("mime_extension_mismatch");
    }

    if status == SafetyScanStatus::NotScanned {
        return Ok(SafetyScanReport::not_scanned());
    }

    Ok(SafetyScanReport {
        status,
        engine: Some("makosh_heuristic_v1".to_owned()),
        checked_at: Some(Utc::now()),
        summary: Some(scan_summary(status).to_owned()),
        metadata: json!({
            "reasons": reasons,
            "content_type": content_type,
            "filename_extension": extension,
            "size_bytes": request.size_bytes,
        }),
    })
}

fn validate_storage_path(value: &str) -> Result<String, StorageError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(StorageError::Invalid(
            "storage_path must not be empty".to_owned(),
        ));
    }
    let path = Path::new(value.as_str());
    if path.is_absolute() || value.contains('\\') {
        return Err(StorageError::Invalid(format!(
            "storage_path must be relative and stay inside mail blob root: {value}"
        )));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => {
                return Err(StorageError::Invalid(format!(
                    "storage_path must be relative and stay inside mail blob root: {value}"
                )));
            }
        }
    }

    Ok(value)
}

async fn path_exists(path: &Path) -> Result<bool, std::io::Error> {
    match tokio::fs::metadata(path).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

async fn prune_empty_parent_dirs(root: &Path, path: &Path) -> Result<(), std::io::Error> {
    let mut current = path.parent();
    while let Some(dir) = current {
        if dir == root {
            break;
        }
        match tokio::fs::remove_dir(dir).await {
            Ok(()) => current = dir.parent(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(hex_char(byte >> 4));
        encoded.push(hex_char(byte & 0x0f));
    }
    encoded
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("hex nibble must fit in 0..=15"),
    }
}

fn normalized_extension(filename: Option<&str>) -> Option<String> {
    let filename = filename?.trim();
    let basename = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filename)
        .trim();
    let (_, extension) = basename.rsplit_once('.')?;
    let extension = extension.trim().to_ascii_lowercase();
    (!extension.is_empty()).then_some(extension)
}

fn normalized_content_type(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase()
}

fn has_executable_magic(bytes: &[u8]) -> bool {
    bytes.starts_with(b"MZ") || bytes.starts_with(b"\x7fELF")
}

fn is_active_content_extension(extension: &str) -> bool {
    matches!(
        extension,
        "app"
            | "bat"
            | "cmd"
            | "com"
            | "dll"
            | "dmg"
            | "exe"
            | "hta"
            | "jar"
            | "jse"
            | "js"
            | "msi"
            | "ps1"
            | "scr"
            | "vbe"
            | "vbs"
            | "wsf"
    )
}

fn is_macro_document_extension(extension: &str) -> bool {
    matches!(
        extension,
        "docm" | "dotm" | "xlsm" | "xltm" | "pptm" | "potm"
    )
}

fn is_mime_extension_mismatch(content_type: &str, extension: &str) -> bool {
    let expected = expected_extensions_for_content_type(content_type);
    !expected.is_empty() && !expected.contains(&extension)
}

fn expected_extensions_for_content_type(content_type: &str) -> &'static [&'static str] {
    match content_type {
        "application/pdf" => &["pdf"],
        "application/zip" => &["zip"],
        "image/jpeg" => &["jpg", "jpeg"],
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/platform/storage/database.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/storage/database.rs`
- Size bytes / Размер в байтах: `4345`
- Included characters / Включено символов: `4345`
- Truncated / Обрезано: `no`

```rust
use serde::Serialize;
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::platform::events::{EventStoreError, expected_migration_summary, run_migrations};
use crate::platform::settings::{ApplicationSettingsStore, SettingsError};
use crate::platform::storage::errors::StorageError;
use crate::platform::storage::models::{DatabaseReadiness, MigrationReadiness};

#[derive(Clone)]
pub struct Database {
    pool: Option<PgPool>,
    database_url: Option<String>,
}

impl Database {
    pub async fn connect(database_url: Option<&str>) -> Result<Self, StorageError> {
        let Some(database_url) = database_url else {
            return Ok(Self::disabled());
        };

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        run_migrations(&pool).await?;
        let settings_repair = ApplicationSettingsStore::new(pool.clone())
            .repair_declared_settings()
            .await?;
        if settings_repair.changed() {
            tracing::warn!(
                inserted = settings_repair.inserted,
                repaired = settings_repair.repaired,
                reset_values = settings_repair.reset_values,
                "application settings were repaired during database startup"
            );
        }

        Ok(Self {
            pool: Some(pool),
            database_url: Some(database_url.to_owned()),
        })
    }

    pub fn disabled() -> Self {
        Self {
            pool: None,
            database_url: None,
        }
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn from_test_pool(pool: PgPool, database_url: impl Into<String>) -> Self {
        Self {
            pool: Some(pool),
            database_url: Some(database_url.into()),
        }
    }

    pub fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    pub(crate) fn database_url(&self) -> Option<&str> {
        self.database_url.as_deref()
    }

    pub async fn readiness(&self) -> DatabaseReadiness {
        let Some(pool) = &self.pool else {
            return DatabaseReadiness::not_configured();
        };

        match sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await
        {
            Ok(1) => DatabaseReadiness::ok(),
            Ok(_) => DatabaseReadiness::unavailable(
                "database readiness query returned unexpected result",
            ),
            Err(error) => {
                tracing::warn!(error = %error, "database readiness check failed");
                DatabaseReadiness::unavailable("database readiness query failed")
            }
        }
    }

    pub async fn migration_readiness(&self) -> MigrationReadiness {
        let Some(pool) = &self.pool else {
            return MigrationReadiness::not_configured();
        };

        let expected = expected_migration_summary();
        let result = sqlx::query_as::<_, AppliedMigrationSummary>(
            r#"
            SELECT
                count(*) FILTER (WHERE success) AS applied_count,
                COALESCE(max(version) FILTER (WHERE success), 0) AS latest_version,
                count(*) FILTER (WHERE NOT success) AS failed_count
            FROM _sqlx_migrations
            "#,
        )
        .fetch_one(pool)
        .await;

        match result {
            Ok(summary) if summary.matches(expected) => MigrationReadiness::ok(),
            Ok(summary) if summary.failed_count > 0 => {
                MigrationReadiness::unavailable("database migrations contain failed entries")
            }
            Ok(_) => MigrationReadiness::unavailable("required database migrations are incomplete"),
            Err(error) => {
                tracing::warn!(error = %error, "database migration readiness check failed");
                MigrationReadiness::unavailable("database migration readiness query failed")
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct AppliedMigrationSummary {
    applied_count: i64,
    latest_version: i64,
    failed_count: i64,
}

impl AppliedMigrationSummary {
    fn matches(&self, expected: crate::platform::events::MigrationSummary) -> bool {
        self.failed_count == 0
            && self.applied_count == expected.count
            && self.latest_version == expected.latest_version
    }
}
```
