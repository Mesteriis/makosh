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
