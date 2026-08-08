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
            default_value: json!("http://192.168.1.2:11434"),
            label: "Ollama base URL",
            description: "Local Ollama HTTP endpoint used by AI runtime requests.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "http://192.168.1.2:11434"
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
