//! Typed bounded ZIP inspection settings, applied only by supervised restart.

use makosh_attachment_archive_inspection_core::{
    ArchiveInspectionLimitsV1, DEFAULT_MAX_ARCHIVE_BYTES_V1, DEFAULT_MAX_DEPTH_V1,
    DEFAULT_MAX_ENTRY_UNCOMPRESSED_BYTES_V1, DEFAULT_MAX_TOTAL_UNCOMPRESSED_BYTES_V1,
};
use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use prost::Message;

const MAX_ARCHIVE_BYTES: &str = "attachment_archive_inspection.max_archive_bytes";
const MAX_DEPTH: &str = "attachment_archive_inspection.max_depth";
const MAX_ENTRIES: &str = "attachment_archive_inspection.max_entries";
const MAX_ENTRY_BYTES: &str = "attachment_archive_inspection.max_entry_uncompressed_bytes";
const MAX_PATH_BYTES: &str = "attachment_archive_inspection.max_path_bytes";
const MAX_TOTAL_BYTES: &str = "attachment_archive_inspection.max_total_uncompressed_bytes";

pub const ATTACHMENT_ARCHIVE_INSPECTION_SETTINGS_SCHEMA_MAJOR_V1: u32 = 1;
pub const ATTACHMENT_ARCHIVE_INSPECTION_SETTINGS_SCHEMA_REVISION_V1: u32 = 1;

#[must_use]
pub fn attachment_archive_inspection_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: ATTACHMENT_ARCHIVE_INSPECTION_SETTINGS_SCHEMA_MAJOR_V1,
        revision: ATTACHMENT_ARCHIVE_INSPECTION_SETTINGS_SCHEMA_REVISION_V1,
        definitions: vec![
            definition(MAX_ARCHIVE_BYTES, DEFAULT_MAX_ARCHIVE_BYTES_V1),
            definition(MAX_DEPTH, DEFAULT_MAX_DEPTH_V1 as u64),
            definition(
                MAX_ENTRIES,
                makosh_attachment_archive_inspection_api::ATTACHMENT_ARCHIVE_INSPECTION_MAX_REPORT_ENTRIES_V1
                    as u64,
            ),
            definition(MAX_ENTRY_BYTES, DEFAULT_MAX_ENTRY_UNCOMPRESSED_BYTES_V1),
            definition(
                MAX_PATH_BYTES,
                makosh_attachment_archive_inspection_api::ATTACHMENT_ARCHIVE_INSPECTION_MAX_PATH_BYTES_V1
                    as u64,
            ),
            definition(MAX_TOTAL_BYTES, DEFAULT_MAX_TOTAL_UNCOMPRESSED_BYTES_V1),
        ],
    }
}

#[must_use]
pub fn attachment_archive_inspection_settings_schema_bytes_v1() -> Vec<u8> {
    attachment_archive_inspection_settings_schema_v1().encode_to_vec()
}

pub fn decode_attachment_archive_inspection_settings_v1(
    snapshot: &SettingsSnapshotV1,
    registration_id: &str,
    expected_revision: u64,
) -> Result<ArchiveInspectionLimitsV1, ArchiveInspectionSettingsErrorV1> {
    if registration_id.is_empty()
        || snapshot.target_id != registration_id
        || expected_revision == 0
        || snapshot.revision != expected_revision
        || snapshot.values.len() != 6
    {
        return Err(ArchiveInspectionSettingsErrorV1::InvalidSnapshot);
    }
    ArchiveInspectionLimitsV1::new(
        required(snapshot, MAX_ARCHIVE_BYTES)?,
        required(snapshot, MAX_TOTAL_BYTES)?,
        required(snapshot, MAX_ENTRY_BYTES)?,
        usize::try_from(required(snapshot, MAX_ENTRIES)?)
            .map_err(|_| ArchiveInspectionSettingsErrorV1::InvalidSnapshot)?,
        usize::try_from(required(snapshot, MAX_DEPTH)?)
            .map_err(|_| ArchiveInspectionSettingsErrorV1::InvalidSnapshot)?,
        usize::try_from(required(snapshot, MAX_PATH_BYTES)?)
            .map_err(|_| ArchiveInspectionSettingsErrorV1::InvalidSnapshot)?,
    )
    .map_err(|_| ArchiveInspectionSettingsErrorV1::InvalidSnapshot)
}

fn definition(setting_id: &str, default: u64) -> SettingDefinitionV1 {
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
        display_name: setting_id.to_owned(),
        default_value: Some(makosh_runtime_protocol::v1::SettingValueV1 {
            value: Some(Value::UnsignedIntegerValue(default)),
        }),
        optional: false,
    }
}

fn required(
    snapshot: &SettingsSnapshotV1,
    setting_id: &str,
) -> Result<u64, ArchiveInspectionSettingsErrorV1> {
    let mut found = None;
    for entry in &snapshot.values {
        if entry.setting_id == setting_id
            && found
                .replace(entry.value.as_ref().and_then(|value| value.value.as_ref()))
                .is_some()
        {
            return Err(ArchiveInspectionSettingsErrorV1::InvalidSnapshot);
        }
    }
    match found.flatten() {
        Some(Value::UnsignedIntegerValue(value)) => Ok(*value),
        _ => Err(ArchiveInspectionSettingsErrorV1::InvalidSnapshot),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionSettingsErrorV1 {
    InvalidSnapshot,
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::validate_settings_schema_v1;

    use super::*;

    #[test]
    fn settings_schema_is_hidden_bounded_and_restart_applied() {
        let schema = attachment_archive_inspection_settings_schema_v1();
        assert_eq!(validate_settings_schema_v1(&schema), Ok(()));
        assert_eq!(schema.definitions.len(), 6);
        assert!(schema.definitions.iter().all(|definition| {
            definition.apply_mode == SettingApplyModeV1::RestartModule as i32
                && definition.client_visibility == SettingClientVisibilityV1::Hidden as i32
                && definition.fresh_owner_proof_required
        }));
    }
}
