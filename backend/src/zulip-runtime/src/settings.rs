//! Zulip-owned decoding of an admitted generic settings snapshot.

use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use makosh_zulip_api::{ZulipAccountV1, validate_account};
use prost::Message;

const ACCOUNT_ID: &str = "zulip.account_id";
const REALM_URL: &str = "zulip.realm_url";
const ACCOUNT_EMAIL: &str = "zulip.account_email";

pub const ZULIP_SETTINGS_SCHEMA_MAJOR_V3: u32 = 3;
pub const ZULIP_SETTINGS_SCHEMA_REVISION_V3: u32 = 1;

#[must_use]
pub fn zulip_settings_schema_v3() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: ZULIP_SETTINGS_SCHEMA_MAJOR_V3,
        revision: ZULIP_SETTINGS_SCHEMA_REVISION_V3,
        definitions: vec![
            definition(
                ACCOUNT_EMAIL,
                SettingValueTypeV1::String,
                "Zulip account email",
            ),
            definition(ACCOUNT_ID, SettingValueTypeV1::String, "Zulip account ID"),
            definition(REALM_URL, SettingValueTypeV1::String, "Zulip realm URL"),
        ],
    }
}

#[must_use]
pub fn zulip_settings_schema_bytes_v3() -> Vec<u8> {
    zulip_settings_schema_v3().encode_to_vec()
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

pub struct ZulipRuntimeSettingsV1 {
    pub account: ZulipAccountV1,
}

pub fn decode(snapshot: &SettingsSnapshotV1) -> Result<ZulipRuntimeSettingsV1, String> {
    let account = ZulipAccountV1 {
        account_id: required_string(snapshot, ACCOUNT_ID)?,
        realm_url: required_string(snapshot, REALM_URL)?,
        account_email: required_string(snapshot, ACCOUNT_EMAIL)?,
    };
    if !validate_account(&account) {
        return Err(invalid_settings());
    }
    Ok(ZulipRuntimeSettingsV1 { account })
}

fn required_string(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<String, String> {
    match value(snapshot, setting_id)? {
        Value::StringValue(value) if !value.trim().is_empty() => Ok(value.clone()),
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
    "Zulip runtime settings are invalid".to_owned()
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::{
        v1::{
            SettingClientVisibilityV1, SettingTargetScopeV1, SettingValueV1, SettingsSnapshotV1,
            SettingsValueEntryV1, setting_value_v1::Value,
        },
        validation::descriptor::validate_settings_schema_v1,
    };

    use super::{decode, zulip_settings_schema_v3};

    #[test]
    fn canonical_schema_is_configuration_scoped_editable_and_non_secret() {
        let schema = zulip_settings_schema_v3();

        assert_eq!(validate_settings_schema_v1(&schema), Ok(()));
        assert_eq!(
            schema
                .definitions
                .iter()
                .map(|definition| definition.setting_id.as_str())
                .collect::<Vec<_>>(),
            ["zulip.account_email", "zulip.account_id", "zulip.realm_url",]
        );
        assert!(schema.definitions.iter().all(|definition| {
            definition.client_visibility == SettingClientVisibilityV1::Editable as i32
                && definition.target_scope == SettingTargetScopeV1::ConfigurationInstance as i32
                && definition.fresh_owner_proof_required
        }));
    }

    #[test]
    fn decoder_accepts_only_the_canonical_account_snapshot() {
        let settings = decode(&SettingsSnapshotV1 {
            target_id: "zulip-account-1".to_owned(),
            revision: 1,
            values: vec![
                entry(
                    "zulip.account_id",
                    Value::StringValue("account-1".to_owned()),
                ),
                entry(
                    "zulip.account_email",
                    Value::StringValue("account@example.com".to_owned()),
                ),
                entry(
                    "zulip.realm_url",
                    Value::StringValue("https://zulip.example.com".to_owned()),
                ),
            ],
        })
        .expect("decode canonical Zulip settings");

        assert_eq!(settings.account.account_id, "account-1");
    }

    fn entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 { value: Some(value) }),
        }
    }
}
