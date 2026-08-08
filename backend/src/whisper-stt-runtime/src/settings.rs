use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use prost::Message;

const ALLOWED_LANGUAGES_MASK: &str = "whisper_stt.allowed_languages_mask";
const MAXIMUM_SOURCE_BYTES: &str = "whisper_stt.maximum_source_bytes";
const MAXIMUM_TRANSCRIPT_BYTES: &str = "whisper_stt.maximum_transcript_bytes";
const THREAD_COUNT: &str = "whisper_stt.thread_count";
const TIMEOUT_MILLIS: &str = "whisper_stt.timeout_millis";

pub const WHISPER_STT_SETTINGS_SCHEMA_MAJOR_V1: u32 = 1;
pub const WHISPER_STT_SETTINGS_SCHEMA_REVISION_V1: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttRuntimeSettingsV1 {
    pub allowed_languages_mask: u32,
    pub maximum_source_bytes: u64,
    pub maximum_transcript_bytes: u32,
    pub thread_count: u32,
    pub timeout_millis: u64,
    pub settings_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhisperSttSettingsErrorV1 {
    InvalidSnapshot,
}

#[must_use]
pub fn whisper_stt_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: WHISPER_STT_SETTINGS_SCHEMA_MAJOR_V1,
        revision: WHISPER_STT_SETTINGS_SCHEMA_REVISION_V1,
        definitions: vec![
            definition(ALLOWED_LANGUAGES_MASK, 14, "Allowed language mask"),
            definition(MAXIMUM_SOURCE_BYTES, 512 * 1024 * 1024, "Maximum WAV bytes"),
            definition(
                MAXIMUM_TRANSCRIPT_BYTES,
                4 * 1024 * 1024,
                "Maximum transcript bytes",
            ),
            definition(THREAD_COUNT, 4, "Native thread count"),
            definition(TIMEOUT_MILLIS, 120_000, "Native timeout milliseconds"),
        ],
    }
}

#[must_use]
pub fn whisper_stt_settings_schema_bytes_v1() -> Vec<u8> {
    whisper_stt_settings_schema_v1().encode_to_vec()
}

pub fn decode_whisper_stt_settings_v1(
    snapshot: &SettingsSnapshotV1,
    configuration_instance_id: &str,
) -> Result<WhisperSttRuntimeSettingsV1, WhisperSttSettingsErrorV1> {
    if configuration_instance_id.is_empty()
        || snapshot.target_id != configuration_instance_id
        || snapshot.revision == 0
        || snapshot.values.len() != 5
    {
        return Err(WhisperSttSettingsErrorV1::InvalidSnapshot);
    }
    let allowed_languages_mask = required(snapshot, ALLOWED_LANGUAGES_MASK)?;
    let maximum_source_bytes = required(snapshot, MAXIMUM_SOURCE_BYTES)?;
    let maximum_transcript_bytes = required(snapshot, MAXIMUM_TRANSCRIPT_BYTES)?;
    let thread_count = required(snapshot, THREAD_COUNT)?;
    let timeout_millis = required(snapshot, TIMEOUT_MILLIS)?;
    if allowed_languages_mask == 0
        || allowed_languages_mask & !14 != 0
        || !(44..=512 * 1024 * 1024).contains(&maximum_source_bytes)
        || !(1..=4 * 1024 * 1024).contains(&maximum_transcript_bytes)
        || !(1..=32).contains(&thread_count)
        || !(1_000..=30 * 60 * 1_000).contains(&timeout_millis)
    {
        return Err(WhisperSttSettingsErrorV1::InvalidSnapshot);
    }
    Ok(WhisperSttRuntimeSettingsV1 {
        allowed_languages_mask: u32::try_from(allowed_languages_mask)
            .map_err(|_| WhisperSttSettingsErrorV1::InvalidSnapshot)?,
        maximum_source_bytes,
        maximum_transcript_bytes: u32::try_from(maximum_transcript_bytes)
            .map_err(|_| WhisperSttSettingsErrorV1::InvalidSnapshot)?,
        thread_count: u32::try_from(thread_count)
            .map_err(|_| WhisperSttSettingsErrorV1::InvalidSnapshot)?,
        timeout_millis,
        settings_revision: snapshot.revision,
    })
}

fn definition(setting_id: &str, default: u64, display_name: &str) -> SettingDefinitionV1 {
    SettingDefinitionV1 {
        setting_id: setting_id.to_owned(),
        capability_id: String::new(),
        value_type: SettingValueTypeV1::UnsignedInteger as i32,
        mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
        target_scope: SettingTargetScopeV1::ConfigurationInstance as i32,
        apply_mode: SettingApplyModeV1::RestartModule as i32,
        client_visibility: SettingClientVisibilityV1::Editable as i32,
        fresh_owner_proof_required: true,
        kernel_controller_id: String::new(),
        display_name: display_name.to_owned(),
        default_value: Some(makosh_runtime_protocol::v1::SettingValueV1 {
            value: Some(Value::UnsignedIntegerValue(default)),
        }),
        optional: false,
    }
}

fn required(
    snapshot: &SettingsSnapshotV1,
    setting_id: &str,
) -> Result<u64, WhisperSttSettingsErrorV1> {
    let mut selected = None;
    for entry in &snapshot.values {
        if entry.setting_id == setting_id
            && selected
                .replace(entry.value.as_ref().and_then(|value| value.value.as_ref()))
                .is_some()
        {
            return Err(WhisperSttSettingsErrorV1::InvalidSnapshot);
        }
    }
    match selected.flatten() {
        Some(Value::UnsignedIntegerValue(value)) => Ok(*value),
        _ => Err(WhisperSttSettingsErrorV1::InvalidSnapshot),
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::validate_settings_schema_v1;

    use super::*;

    #[test]
    fn schema_contains_only_bounded_non_secret_execution_policy() {
        let schema = whisper_stt_settings_schema_v1();
        validate_settings_schema_v1(&schema).expect("settings schema");
        assert_eq!(schema.definitions.len(), 5);
        assert!(schema.definitions.iter().all(|definition| {
            definition.apply_mode == SettingApplyModeV1::RestartModule as i32
                && definition.fresh_owner_proof_required
                && !definition.setting_id.contains("path")
                && !definition.setting_id.contains("model")
        }));
    }
}
