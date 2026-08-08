//! Settings schema and snapshot contract validation.

use super::common::*;

#[test]
fn settings_schema_requires_ordered_typed_non_secret_definitions() {
    let schema = SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: vec![SettingDefinitionV1 {
            setting_id: "sync.interval".into(),
            capability_id: "capability.read".into(),
            value_type: SettingValueTypeV1::Duration as i32,
            mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
            target_scope: SettingTargetScopeV1::ModuleRegistration as i32,
            apply_mode: SettingApplyModeV1::HotReload as i32,
            client_visibility: SettingClientVisibilityV1::Editable as i32,
            fresh_owner_proof_required: true,
            kernel_controller_id: String::new(),
            display_name: "Sync interval".into(),
            default_value: None,
            optional: false,
        }],
    };
    assert!(decode_settings_schema_v1(&schema.encode_to_vec()).is_ok());
    let mut defaulted = schema.clone();
    defaulted.definitions[0].default_value = Some(SettingValueV1 {
        value: Some(makosh_runtime_protocol::v1::setting_value_v1::Value::DurationMillis(5_000)),
    });
    assert!(decode_settings_schema_v1(&defaulted.encode_to_vec()).is_ok());
    defaulted.definitions[0].default_value = Some(SettingValueV1 {
        value: Some(
            makosh_runtime_protocol::v1::setting_value_v1::Value::StringValue("wrong".into()),
        ),
    });
    assert!(decode_settings_schema_v1(&defaulted.encode_to_vec()).is_err());
    let mut invalid = schema;
    invalid.definitions[0].mutation_authority = SettingMutationAuthorityV1::KernelManaged as i32;
    invalid.definitions[0].client_visibility = SettingClientVisibilityV1::Editable as i32;
    invalid.definitions[0].kernel_controller_id = "controller".into();
    assert!(decode_settings_schema_v1(&invalid.encode_to_vec()).is_err());
}

#[test]
fn settings_snapshot_requires_a_canonical_typed_value_set() {
    let snapshot = duration_settings_snapshot();
    assert!(decode_settings_snapshot_v1(&snapshot.encode_to_vec()).is_ok());
    let schema = duration_settings_schema();
    assert!(validate_settings_snapshot_against_schema_v1(&schema, &snapshot).is_ok());
    let mut wrong_type = snapshot.clone();
    wrong_type.values[0].value = Some(SettingValueV1 {
        value: Some(
            makosh_runtime_protocol::v1::setting_value_v1::Value::StringValue(
                "not a duration".into(),
            ),
        ),
    });
    assert!(validate_settings_snapshot_against_schema_v1(&schema, &wrong_type).is_err());
    let mut unknown_setting = snapshot.clone();
    unknown_setting.values[0].setting_id = "unknown.setting".into();
    assert!(validate_settings_snapshot_against_schema_v1(&schema, &unknown_setting).is_err());
    assert_oversized_setting_is_rejected(&schema, &snapshot);
    let mut missing = snapshot;
    missing.values[0].value = None;
    assert!(decode_settings_snapshot_v1(&missing.encode_to_vec()).is_err());
}

fn duration_settings_snapshot() -> SettingsSnapshotV1 {
    SettingsSnapshotV1 {
        target_id: "registration-1".into(),
        revision: 1,
        values: vec![SettingsValueEntryV1 {
            setting_id: "sync.interval".into(),
            value: Some(SettingValueV1 {
                value: Some(
                    makosh_runtime_protocol::v1::setting_value_v1::Value::DurationMillis(1000),
                ),
            }),
        }],
    }
}

fn duration_settings_schema() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: vec![SettingDefinitionV1 {
            setting_id: "sync.interval".into(),
            capability_id: String::new(),
            value_type: SettingValueTypeV1::Duration as i32,
            mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
            target_scope: SettingTargetScopeV1::ModuleRegistration as i32,
            apply_mode: SettingApplyModeV1::HotReload as i32,
            client_visibility: SettingClientVisibilityV1::Editable as i32,
            fresh_owner_proof_required: false,
            kernel_controller_id: String::new(),
            display_name: "Sync interval".into(),
            default_value: None,
            optional: false,
        }],
    }
}

fn assert_oversized_setting_is_rejected(schema: &SettingsSchemaV1, snapshot: &SettingsSnapshotV1) {
    let mut oversized = snapshot.clone();
    oversized.values[0].value = Some(SettingValueV1 {
        value: Some(
            makosh_runtime_protocol::v1::setting_value_v1::Value::StringValue("x".repeat(8193)),
        ),
    });
    let string_schema = SettingsSchemaV1 {
        definitions: vec![SettingDefinitionV1 {
            value_type: SettingValueTypeV1::String as i32,
            ..schema.definitions[0].clone()
        }],
        ..schema.clone()
    };
    assert!(validate_settings_snapshot_against_schema_v1(&string_schema, &oversized).is_err());
}
