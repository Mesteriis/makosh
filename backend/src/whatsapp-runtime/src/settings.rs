//! WhatsApp-owned decoding of an admitted generic settings snapshot.

use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use makosh_whatsapp_api::MAX_ID_LEN;
use prost::Message;

const ACCOUNT_ID: &str = "whatsapp.account_id";

pub const WHATSAPP_SETTINGS_SCHEMA_MAJOR_V1: u32 = 1;
pub const WHATSAPP_SETTINGS_SCHEMA_REVISION_V1: u32 = 1;

#[must_use]
pub fn whatsapp_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: WHATSAPP_SETTINGS_SCHEMA_MAJOR_V1,
        revision: WHATSAPP_SETTINGS_SCHEMA_REVISION_V1,
        definitions: vec![SettingDefinitionV1 {
            setting_id: ACCOUNT_ID.to_owned(),
            capability_id: String::new(),
            value_type: SettingValueTypeV1::String as i32,
            mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
            target_scope: SettingTargetScopeV1::ConfigurationInstance as i32,
            apply_mode: SettingApplyModeV1::RestartModule as i32,
            client_visibility: SettingClientVisibilityV1::Hidden as i32,
            fresh_owner_proof_required: true,
            kernel_controller_id: String::new(),
            display_name: "WhatsApp account ID".to_owned(),
            default_value: None,
            optional: false,
        }],
    }
}

#[must_use]
pub fn whatsapp_settings_schema_bytes_v1() -> Vec<u8> {
    whatsapp_settings_schema_v1().encode_to_vec()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppRuntimeSettingsV1 {
    pub account_id: String,
}

pub fn decode(snapshot: &SettingsSnapshotV1) -> Result<WhatsAppRuntimeSettingsV1, String> {
    if snapshot.values.len() != 1 {
        return Err(invalid_settings());
    }
    let account_id = match snapshot.values[0]
        .value
        .as_ref()
        .and_then(|value| value.value.as_ref())
    {
        Some(Value::StringValue(value))
            if snapshot.values[0].setting_id == ACCOUNT_ID
                && !value.trim().is_empty()
                && value.len() <= MAX_ID_LEN =>
        {
            value.clone()
        }
        _ => return Err(invalid_settings()),
    };
    Ok(WhatsAppRuntimeSettingsV1 { account_id })
}

fn invalid_settings() -> String {
    "WhatsApp runtime settings are invalid".to_owned()
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::{
        v1::{
            SettingApplyModeV1, SettingClientVisibilityV1, SettingMutationAuthorityV1,
            SettingTargetScopeV1, SettingValueV1, SettingsSnapshotV1, SettingsValueEntryV1,
            setting_value_v1::Value,
        },
        validation::descriptor::validate_settings_schema_v1,
    };

    use super::{decode, whatsapp_settings_schema_v1};

    #[test]
    fn canonical_schema_is_configuration_scoped_hidden_and_requires_owner_proof() {
        let schema = whatsapp_settings_schema_v1();

        assert_eq!(validate_settings_schema_v1(&schema), Ok(()));
        assert_eq!(schema.definitions.len(), 1);
        let account = &schema.definitions[0];
        assert_eq!(account.setting_id, "whatsapp.account_id");
        assert_eq!(
            account.mutation_authority,
            SettingMutationAuthorityV1::OperatorManaged as i32
        );
        assert_eq!(
            account.target_scope,
            SettingTargetScopeV1::ConfigurationInstance as i32
        );
        assert_eq!(account.apply_mode, SettingApplyModeV1::RestartModule as i32);
        assert_eq!(
            account.client_visibility,
            SettingClientVisibilityV1::Hidden as i32
        );
        assert!(account.fresh_owner_proof_required);
    }

    #[test]
    fn decoder_accepts_only_one_exact_bounded_account_setting() {
        let canonical = SettingsSnapshotV1 {
            target_id: "whatsapp-account-1".to_owned(),
            revision: 1,
            values: vec![entry(
                "whatsapp.account_id",
                Value::StringValue("account-1".to_owned()),
            )],
        };

        assert_eq!(
            decode(&canonical),
            Ok(super::WhatsAppRuntimeSettingsV1 {
                account_id: "account-1".to_owned(),
            })
        );

        for invalid in [
            SettingsSnapshotV1 {
                values: Vec::new(),
                ..canonical.clone()
            },
            SettingsSnapshotV1 {
                values: vec![entry(
                    "whatsapp.other",
                    Value::StringValue("account-1".to_owned()),
                )],
                ..canonical.clone()
            },
            SettingsSnapshotV1 {
                values: vec![entry(
                    "whatsapp.account_id",
                    Value::StringValue(" ".to_owned()),
                )],
                ..canonical.clone()
            },
            SettingsSnapshotV1 {
                values: vec![canonical.values[0].clone(), canonical.values[0].clone()],
                ..canonical.clone()
            },
        ] {
            assert!(decode(&invalid).is_err());
        }
    }

    fn entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 { value: Some(value) }),
        }
    }
}
