use makosh_mail_contacts_sync_core::MailContactsSyncDirectionV1;
use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use prost::Message;

const ACCOUNT_ID: &str = "mail_contacts_sync.account_id";
const DIRECTION: &str = "mail_contacts_sync.direction";
const ENABLED: &str = "mail_contacts_sync.enabled";
const INTERVAL_SECONDS: &str = "mail_contacts_sync.interval_seconds";
const REMOTE_WRITE_ENABLED: &str = "mail_contacts_sync.remote_write_enabled";
const MIN_INTERVAL_SECONDS: u64 = 300;
const MAX_INTERVAL_SECONDS: u64 = 604_800;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncRuntimeSettingsV1 {
    pub account_id: String,
    pub direction: MailContactsSyncDirectionV1,
    pub enabled: bool,
    pub interval_seconds: u64,
    pub remote_write_enabled: bool,
}

#[must_use]
pub fn mail_contacts_sync_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: vec![
            definition(ACCOUNT_ID, SettingValueTypeV1::String, "Mail account ID"),
            definition(DIRECTION, SettingValueTypeV1::Enum, "Sync direction"),
            definition(
                ENABLED,
                SettingValueTypeV1::Boolean,
                "Scheduled sync enabled",
            ),
            definition(
                INTERVAL_SECONDS,
                SettingValueTypeV1::UnsignedInteger,
                "Sync interval seconds",
            ),
            definition(
                REMOTE_WRITE_ENABLED,
                SettingValueTypeV1::Boolean,
                "Remote address-book writes enabled",
            ),
        ],
    }
}

#[must_use]
pub fn mail_contacts_sync_settings_schema_bytes_v1() -> Vec<u8> {
    mail_contacts_sync_settings_schema_v1().encode_to_vec()
}

pub fn decode_mail_contacts_sync_settings_v1(
    snapshot: &SettingsSnapshotV1,
) -> Result<MailContactsSyncRuntimeSettingsV1, &'static str> {
    if snapshot.revision == 0 || snapshot.target_id.trim().is_empty() || snapshot.values.len() != 5
    {
        return Err("mail_contacts_sync_settings_invalid");
    }
    let account_id = required_string(snapshot, ACCOUNT_ID)?;
    if account_id.len() > 256 || !account_id.is_ascii() || account_id.trim() != account_id {
        return Err("mail_contacts_sync_settings_invalid");
    }
    let direction = match required_enum(snapshot, DIRECTION)? {
        "provider_to_contacts" => MailContactsSyncDirectionV1::ProviderToContacts,
        "bidirectional" => MailContactsSyncDirectionV1::Bidirectional,
        _ => return Err("mail_contacts_sync_settings_invalid"),
    };
    let enabled = required_boolean(snapshot, ENABLED)?;
    let interval_seconds = required_unsigned(snapshot, INTERVAL_SECONDS)?;
    let remote_write_enabled = required_boolean(snapshot, REMOTE_WRITE_ENABLED)?;
    if !(MIN_INTERVAL_SECONDS..=MAX_INTERVAL_SECONDS).contains(&interval_seconds)
        || (remote_write_enabled && direction != MailContactsSyncDirectionV1::Bidirectional)
    {
        return Err("mail_contacts_sync_settings_invalid");
    }
    Ok(MailContactsSyncRuntimeSettingsV1 {
        account_id,
        direction,
        enabled,
        interval_seconds,
        remote_write_enabled,
    })
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

fn required_value<'a>(
    snapshot: &'a SettingsSnapshotV1,
    setting_id: &str,
) -> Result<&'a Value, &'static str> {
    let mut selected = None;
    for entry in &snapshot.values {
        if entry.setting_id == setting_id
            && selected
                .replace(entry.value.as_ref().and_then(|value| value.value.as_ref()))
                .is_some()
        {
            return Err("mail_contacts_sync_settings_invalid");
        }
    }
    selected
        .flatten()
        .ok_or("mail_contacts_sync_settings_invalid")
}

fn required_string(
    snapshot: &SettingsSnapshotV1,
    setting_id: &str,
) -> Result<String, &'static str> {
    match required_value(snapshot, setting_id)? {
        Value::StringValue(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err("mail_contacts_sync_settings_invalid"),
    }
}

fn required_boolean(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<bool, &'static str> {
    match required_value(snapshot, setting_id)? {
        Value::BooleanValue(value) => Ok(*value),
        _ => Err("mail_contacts_sync_settings_invalid"),
    }
}

fn required_enum<'a>(
    snapshot: &'a SettingsSnapshotV1,
    setting_id: &str,
) -> Result<&'a str, &'static str> {
    match required_value(snapshot, setting_id)? {
        Value::EnumValue(value) if !value.is_empty() => Ok(value),
        _ => Err("mail_contacts_sync_settings_invalid"),
    }
}

fn required_unsigned(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<u64, &'static str> {
    match required_value(snapshot, setting_id)? {
        Value::UnsignedIntegerValue(value) => Ok(*value),
        _ => Err("mail_contacts_sync_settings_invalid"),
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::{
        v1::{SettingValueV1, SettingsValueEntryV1},
        validation::descriptor::{
            validate_settings_schema_v1, validate_settings_snapshot_against_schema_v1,
        },
    };

    use super::*;

    #[test]
    fn schema_is_typed_editable_and_configuration_scoped() {
        let schema = mail_contacts_sync_settings_schema_v1();
        validate_settings_schema_v1(&schema).expect("settings schema");
        assert_eq!(schema.definitions.len(), 5);
        assert!(schema.definitions.iter().all(|definition| {
            definition.target_scope == SettingTargetScopeV1::ConfigurationInstance as i32
                && definition.client_visibility == SettingClientVisibilityV1::Editable as i32
                && definition.fresh_owner_proof_required
        }));
    }

    #[test]
    fn decoder_fences_direction_interval_and_remote_write() {
        let valid = snapshot("bidirectional", 900, true);
        validate_settings_snapshot_against_schema_v1(
            &mail_contacts_sync_settings_schema_v1(),
            &valid,
        )
        .expect("typed settings snapshot");
        assert_eq!(
            decode_mail_contacts_sync_settings_v1(&valid)
                .expect("valid")
                .direction,
            MailContactsSyncDirectionV1::Bidirectional
        );
        assert!(
            decode_mail_contacts_sync_settings_v1(&snapshot("provider_to_contacts", 900, true))
                .is_err()
        );
        assert!(
            decode_mail_contacts_sync_settings_v1(&snapshot("bidirectional", 299, false)).is_err()
        );
    }

    fn snapshot(direction: &str, interval: u64, remote_write: bool) -> SettingsSnapshotV1 {
        SettingsSnapshotV1 {
            target_id: "sync-account-1".to_owned(),
            revision: 1,
            values: vec![
                entry(ACCOUNT_ID, Value::StringValue("mail-account-1".to_owned())),
                entry(DIRECTION, Value::EnumValue(direction.to_owned())),
                entry(ENABLED, Value::BooleanValue(true)),
                entry(INTERVAL_SECONDS, Value::UnsignedIntegerValue(interval)),
                entry(REMOTE_WRITE_ENABLED, Value::BooleanValue(remote_write)),
            ],
        }
    }

    fn entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 { value: Some(value) }),
        }
    }
}
