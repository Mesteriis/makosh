//! Telegram-owned decoding of one admitted generic settings snapshot.

use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use prost::Message;

const ACCOUNT_ID: &str = "telegram.account_id";
const API_ID: &str = "telegram.api_id";

pub const TELEGRAM_SETTINGS_SCHEMA_MAJOR_V1: u32 = 1;
pub const TELEGRAM_SETTINGS_SCHEMA_REVISION_V1: u32 = 2;

#[must_use]
pub fn telegram_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: TELEGRAM_SETTINGS_SCHEMA_MAJOR_V1,
        revision: TELEGRAM_SETTINGS_SCHEMA_REVISION_V1,
        definitions: vec![
            definition(ACCOUNT_ID, SettingValueTypeV1::String, "Telegram account"),
            definition(API_ID, SettingValueTypeV1::SignedInteger, "Telegram API ID"),
        ],
    }
}

#[must_use]
pub fn telegram_settings_schema_bytes_v1() -> Vec<u8> {
    telegram_settings_schema_v1().encode_to_vec()
}

fn definition(
    setting_id: &str,
    value_type: SettingValueTypeV1,
    display_name: &str,
) -> SettingDefinitionV1 {
    SettingDefinitionV1 {
        setting_id: setting_id.to_owned(),
        capability_id: String::new(),
        value_type: value_type as i32,
        mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
        target_scope: SettingTargetScopeV1::ConfigurationInstance as i32,
        apply_mode: SettingApplyModeV1::RestartModule as i32,
        client_visibility: SettingClientVisibilityV1::Editable as i32,
        fresh_owner_proof_required: true,
        kernel_controller_id: String::new(),
        display_name: display_name.to_owned(),
        default_value: None,
        optional: false,
    }
}

pub struct TelegramRuntimeSettingsV1 {
    pub account_id: String,
    pub api_id: i64,
}

pub fn decode(snapshot: &SettingsSnapshotV1) -> Result<TelegramRuntimeSettingsV1, String> {
    let account_id = required_string(snapshot, ACCOUNT_ID)?;
    let api_id = required_signed(snapshot, API_ID)?;
    if api_id <= 0 {
        return Err(invalid_settings());
    }
    Ok(TelegramRuntimeSettingsV1 { account_id, api_id })
}

fn required_string(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<String, String> {
    match value(snapshot, setting_id)? {
        Value::StringValue(value) if !value.trim().is_empty() => Ok(value.clone()),
        _ => Err(invalid_settings()),
    }
}

fn required_signed(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<i64, String> {
    match value(snapshot, setting_id)? {
        Value::SignedIntegerValue(value) => Ok(*value),
        _ => Err(invalid_settings()),
    }
}

fn value<'a>(snapshot: &'a SettingsSnapshotV1, setting_id: &str) -> Result<&'a Value, String> {
    let mut selected = None;
    for entry in &snapshot.values {
        if entry.setting_id == setting_id {
            let value = entry.value.as_ref().and_then(|value| value.value.as_ref());
            if selected.replace(value).is_some() {
                return Err(invalid_settings());
            }
        }
    }
    selected.flatten().ok_or_else(invalid_settings)
}

fn invalid_settings() -> String {
    "Telegram runtime settings are invalid".to_owned()
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::{
        v1::{
            SettingClientVisibilityV1, SettingValueV1, SettingsSnapshotV1, SettingsValueEntryV1,
            setting_value_v1::Value,
        },
        validation::descriptor::validate_settings_schema_v1,
    };

    use super::{decode, telegram_settings_schema_v1};

    #[test]
    fn canonical_schema_contains_only_non_secret_operator_configuration() {
        let schema = telegram_settings_schema_v1();

        assert_eq!(validate_settings_schema_v1(&schema), Ok(()));
        assert_eq!(schema.revision, 2);
        assert_eq!(
            schema
                .definitions
                .iter()
                .map(|definition| definition.setting_id.as_str())
                .collect::<Vec<_>>(),
            ["telegram.account_id", "telegram.api_id"]
        );
        assert!(schema.definitions.iter().all(|definition| {
            definition.client_visibility == SettingClientVisibilityV1::Editable as i32
        }));
    }

    #[test]
    fn decoder_requires_no_path_or_secret_revision_setting() {
        let settings = decode(&SettingsSnapshotV1 {
            target_id: "telegram-account-1".to_owned(),
            revision: 1,
            values: vec![
                entry(
                    "telegram.account_id",
                    Value::StringValue("account-1".to_owned()),
                ),
                entry("telegram.api_id", Value::SignedIntegerValue(42)),
            ],
        })
        .expect("decode canonical Telegram settings");

        assert_eq!(settings.account_id, "account-1");
        assert_eq!(settings.api_id, 42);
    }

    fn entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 { value: Some(value) }),
        }
    }
}
