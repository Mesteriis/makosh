//! Typed, owner-controlled runtime settings for the loopback scanner adapter.

use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use prost::Message;

const CONNECT_TIMEOUT_MILLIS: &str = "attachment_security.clamav.connect_timeout_millis";
const IO_TIMEOUT_MILLIS: &str = "attachment_security.clamav.io_timeout_millis";
const MAX_SCAN_BYTES: &str = "attachment_security.max_scan_bytes";
const PORT: &str = "attachment_security.clamav.port";

pub const ATTACHMENT_SECURITY_SETTINGS_SCHEMA_MAJOR_V1: u32 = 1;
pub const ATTACHMENT_SECURITY_SETTINGS_SCHEMA_REVISION_V1: u32 = 1;
pub const ATTACHMENT_SECURITY_HARD_MAX_SCAN_BYTES_V1: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityRuntimeSettingsV1 {
    pub clamav_connect_timeout_millis: u64,
    pub clamav_io_timeout_millis: u64,
    pub clamav_port: u16,
    pub max_scan_bytes: u64,
}

#[must_use]
pub fn attachment_security_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: ATTACHMENT_SECURITY_SETTINGS_SCHEMA_MAJOR_V1,
        revision: ATTACHMENT_SECURITY_SETTINGS_SCHEMA_REVISION_V1,
        definitions: vec![
            definition(
                CONNECT_TIMEOUT_MILLIS,
                "ClamAV loopback connect timeout in milliseconds",
            ),
            definition(
                IO_TIMEOUT_MILLIS,
                "ClamAV loopback I/O timeout in milliseconds",
            ),
            definition(PORT, "ClamAV loopback TCP port"),
            definition(MAX_SCAN_BYTES, "Maximum admitted attachment scan bytes"),
        ],
    }
}

#[must_use]
pub fn attachment_security_settings_schema_bytes_v1() -> Vec<u8> {
    attachment_security_settings_schema_v1().encode_to_vec()
}

pub fn decode_attachment_security_settings_v1(
    snapshot: &SettingsSnapshotV1,
    registration_id: &str,
    expected_revision: u64,
) -> Result<AttachmentSecurityRuntimeSettingsV1, AttachmentSecuritySettingsErrorV1> {
    if registration_id.is_empty()
        || snapshot.target_id != registration_id
        || expected_revision == 0
        || snapshot.revision != expected_revision
        || snapshot.values.len() != 4
    {
        return Err(AttachmentSecuritySettingsErrorV1::InvalidSnapshot);
    }
    let connect_timeout = required_unsigned(snapshot, CONNECT_TIMEOUT_MILLIS)?;
    let io_timeout = required_unsigned(snapshot, IO_TIMEOUT_MILLIS)?;
    let max_scan_bytes = required_unsigned(snapshot, MAX_SCAN_BYTES)?;
    let port = required_unsigned(snapshot, PORT)?;
    if !(1..=30_000).contains(&connect_timeout)
        || !(1..=120_000).contains(&io_timeout)
        || !(1..=ATTACHMENT_SECURITY_HARD_MAX_SCAN_BYTES_V1).contains(&max_scan_bytes)
        || !(1..=u64::from(u16::MAX)).contains(&port)
    {
        return Err(AttachmentSecuritySettingsErrorV1::InvalidSnapshot);
    }
    Ok(AttachmentSecurityRuntimeSettingsV1 {
        clamav_connect_timeout_millis: connect_timeout,
        clamav_io_timeout_millis: io_timeout,
        clamav_port: u16::try_from(port)
            .map_err(|_| AttachmentSecuritySettingsErrorV1::InvalidSnapshot)?,
        max_scan_bytes,
    })
}

fn definition(setting_id: &str, display_name: &str) -> SettingDefinitionV1 {
    let default = match setting_id {
        CONNECT_TIMEOUT_MILLIS => 2_000,
        IO_TIMEOUT_MILLIS => 30_000,
        PORT => 3_310,
        MAX_SCAN_BYTES => 8 * 1024 * 1024,
        _ => unreachable!("all Attachment Security settings are exhaustively declared"),
    };
    SettingDefinitionV1 {
        setting_id: setting_id.to_owned(),
        capability_id: String::new(),
        value_type: SettingValueTypeV1::UnsignedInteger as i32,
        mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
        target_scope: SettingTargetScopeV1::ModuleRegistration as i32,
        apply_mode: SettingApplyModeV1::RestartModule as i32,
        client_visibility: SettingClientVisibilityV1::Hidden as i32,
        fresh_owner_proof_required: true,
        kernel_controller_id: String::new(),
        display_name: display_name.to_owned(),
        default_value: Some(makosh_runtime_protocol::v1::SettingValueV1 {
            value: Some(Value::UnsignedIntegerValue(default)),
        }),
        optional: false,
    }
}

fn required_unsigned(
    snapshot: &SettingsSnapshotV1,
    setting_id: &str,
) -> Result<u64, AttachmentSecuritySettingsErrorV1> {
    let mut selected = None;
    for entry in &snapshot.values {
        if entry.setting_id == setting_id {
            let value = entry.value.as_ref().and_then(|value| value.value.as_ref());
            if selected.replace(value).is_some() {
                return Err(AttachmentSecuritySettingsErrorV1::InvalidSnapshot);
            }
        }
    }
    match selected.flatten() {
        Some(Value::UnsignedIntegerValue(value)) => Ok(*value),
        _ => Err(AttachmentSecuritySettingsErrorV1::InvalidSnapshot),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecuritySettingsErrorV1 {
    InvalidSnapshot,
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
    fn schema_is_registration_scoped_hidden_and_restart_applied() {
        let schema = attachment_security_settings_schema_v1();
        assert_eq!(validate_settings_schema_v1(&schema), Ok(()));
        assert_eq!(
            schema
                .definitions
                .iter()
                .map(|definition| definition.setting_id.as_str())
                .collect::<Vec<_>>(),
            [
                CONNECT_TIMEOUT_MILLIS,
                IO_TIMEOUT_MILLIS,
                PORT,
                MAX_SCAN_BYTES,
            ]
        );
        assert!(schema.definitions.iter().all(|definition| {
            definition.target_scope == SettingTargetScopeV1::ModuleRegistration as i32
                && definition.apply_mode == SettingApplyModeV1::RestartModule as i32
                && definition.client_visibility == SettingClientVisibilityV1::Hidden as i32
                && definition.fresh_owner_proof_required
        }));
    }

    #[test]
    fn decoder_requires_exact_target_revision_and_bounded_values() {
        let snapshot = snapshot();
        assert_eq!(
            validate_settings_snapshot_against_schema_v1(
                &attachment_security_settings_schema_v1(),
                &snapshot,
            ),
            Ok(())
        );
        assert_eq!(
            decode_attachment_security_settings_v1(&snapshot, "attachment-security", 7),
            Ok(AttachmentSecurityRuntimeSettingsV1 {
                clamav_connect_timeout_millis: 2_000,
                clamav_io_timeout_millis: 30_000,
                clamav_port: 3_310,
                max_scan_bytes: 8 * 1024 * 1024,
            })
        );
        assert_eq!(
            decode_attachment_security_settings_v1(&snapshot, "other", 7),
            Err(AttachmentSecuritySettingsErrorV1::InvalidSnapshot)
        );
    }

    fn snapshot() -> SettingsSnapshotV1 {
        SettingsSnapshotV1 {
            target_id: "attachment-security".to_owned(),
            revision: 7,
            values: vec![
                entry(CONNECT_TIMEOUT_MILLIS, 2_000),
                entry(IO_TIMEOUT_MILLIS, 30_000),
                entry(PORT, 3_310),
                entry(MAX_SCAN_BYTES, 8 * 1024 * 1024),
            ],
        }
    }

    fn entry(setting_id: &str, value: u64) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 {
                value: Some(Value::UnsignedIntegerValue(value)),
            }),
        }
    }
}
