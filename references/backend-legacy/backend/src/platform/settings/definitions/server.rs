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
