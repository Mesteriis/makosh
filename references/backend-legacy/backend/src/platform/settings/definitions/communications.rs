use serde_json::json;

use super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![
        DeclaredApplicationSetting {
            setting_key: "communications.mail.consecutive_failures_before_degraded",
            category: "communications",
            value_kind: SettingValueKind::Integer,
            default_value: json!(3),
            label: "Mail failures before degradation",
            description: "Number of consecutive provider sync failures before Макошь marks mail sync as degraded. Successful or skipped runs reset the counter.",
            metadata: json!({
                "ui_control": "number",
                "min": 1,
                "max": 10,
                "scope": "mail_sync",
                "stores_private_content": false
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "communications.telegram.read_receipt_reports_enabled",
            category: "communications",
            value_kind: SettingValueKind::Boolean,
            default_value: json!(true),
            label: "Send Telegram read reports",
            description: "When disabled, Макошь keeps read state locally and does not enqueue Telegram view/read commands. A chat-level policy can override this default.",
            metadata: json!({
                "ui_control": "checkbox",
                "scope": "telegram",
                "policy_kind": "provider_read_receipt",
                "stores_private_content": false
            }),
            is_editable: true,
        },
    ]
}
