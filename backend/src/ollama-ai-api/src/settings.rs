use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use prost::Message;

use crate::{OLLAMA_AI_MAX_TIMEOUT_MILLIS_V1, valid_ollama_model_name_v1};

const CHAT_MODEL: &str = "ollama.chat_model";
const PORT: &str = "ollama.port";
const TIMEOUT_MILLIS: &str = "ollama.timeout_millis";

pub const OLLAMA_AI_SETTINGS_SCHEMA_MAJOR_V1: u32 = 1;
pub const OLLAMA_AI_SETTINGS_SCHEMA_REVISION_V1: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaAiRuntimeSettingsV1 {
    pub chat_model: String,
    pub port: u16,
    pub timeout_millis: u64,
    pub settings_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OllamaAiSettingsErrorV1 {
    InvalidSnapshot,
}

#[must_use]
pub fn ollama_ai_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: OLLAMA_AI_SETTINGS_SCHEMA_MAJOR_V1,
        revision: OLLAMA_AI_SETTINGS_SCHEMA_REVISION_V1,
        definitions: vec![
            definition(CHAT_MODEL, SettingValueTypeV1::String, "Ollama chat model"),
            definition(
                PORT,
                SettingValueTypeV1::UnsignedInteger,
                "Ollama loopback port",
            ),
            definition(
                TIMEOUT_MILLIS,
                SettingValueTypeV1::UnsignedInteger,
                "Ollama request timeout in milliseconds",
            ),
        ],
    }
}

#[must_use]
pub fn ollama_ai_settings_schema_bytes_v1() -> Vec<u8> {
    ollama_ai_settings_schema_v1().encode_to_vec()
}

pub fn decode_ollama_ai_settings_v1(
    snapshot: &SettingsSnapshotV1,
    configuration_instance_id: &str,
) -> Result<OllamaAiRuntimeSettingsV1, OllamaAiSettingsErrorV1> {
    if configuration_instance_id.is_empty()
        || snapshot.target_id != configuration_instance_id
        || snapshot.revision == 0
        || snapshot.values.len() != 3
    {
        return Err(OllamaAiSettingsErrorV1::InvalidSnapshot);
    }
    let chat_model = required_string(snapshot, CHAT_MODEL)?;
    let port = required_unsigned(snapshot, PORT)?;
    let timeout_millis = required_unsigned(snapshot, TIMEOUT_MILLIS)?;
    if !valid_ollama_model_name_v1(&chat_model)
        || !(1..=u64::from(u16::MAX)).contains(&port)
        || !(1..=OLLAMA_AI_MAX_TIMEOUT_MILLIS_V1).contains(&timeout_millis)
    {
        return Err(OllamaAiSettingsErrorV1::InvalidSnapshot);
    }
    Ok(OllamaAiRuntimeSettingsV1 {
        chat_model,
        port: u16::try_from(port).map_err(|_| OllamaAiSettingsErrorV1::InvalidSnapshot)?,
        timeout_millis,
        settings_revision: snapshot.revision,
    })
}

fn definition(
    setting_id: &str,
    value_type: SettingValueTypeV1,
    display_name: &str,
) -> SettingDefinitionV1 {
    let default_value = match setting_id {
        CHAT_MODEL => Value::StringValue("qwen3:4b".to_owned()),
        PORT => Value::UnsignedIntegerValue(11_434),
        TIMEOUT_MILLIS => Value::UnsignedIntegerValue(30_000),
        _ => unreachable!("the Ollama settings schema is exact"),
    };
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
        default_value: Some(makosh_runtime_protocol::v1::SettingValueV1 {
            value: Some(default_value),
        }),
        optional: false,
    }
}

fn required_string(
    snapshot: &SettingsSnapshotV1,
    setting_id: &str,
) -> Result<String, OllamaAiSettingsErrorV1> {
    match value(snapshot, setting_id)? {
        Value::StringValue(value) => Ok(value.clone()),
        _ => Err(OllamaAiSettingsErrorV1::InvalidSnapshot),
    }
}

fn required_unsigned(
    snapshot: &SettingsSnapshotV1,
    setting_id: &str,
) -> Result<u64, OllamaAiSettingsErrorV1> {
    match value(snapshot, setting_id)? {
        Value::UnsignedIntegerValue(value) => Ok(*value),
        _ => Err(OllamaAiSettingsErrorV1::InvalidSnapshot),
    }
}

fn value<'a>(
    snapshot: &'a SettingsSnapshotV1,
    setting_id: &str,
) -> Result<&'a Value, OllamaAiSettingsErrorV1> {
    let mut selected = None;
    for entry in &snapshot.values {
        if entry.setting_id == setting_id {
            let value = entry.value.as_ref().and_then(|value| value.value.as_ref());
            if selected.replace(value).is_some() {
                return Err(OllamaAiSettingsErrorV1::InvalidSnapshot);
            }
        }
    }
    selected
        .flatten()
        .ok_or(OllamaAiSettingsErrorV1::InvalidSnapshot)
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
    fn schema_is_exact_editable_non_secret_and_restart_applied() {
        let schema = ollama_ai_settings_schema_v1();
        validate_settings_schema_v1(&schema).expect("settings schema");
        assert_eq!(
            schema
                .definitions
                .iter()
                .map(|definition| definition.setting_id.as_str())
                .collect::<Vec<_>>(),
            [CHAT_MODEL, PORT, TIMEOUT_MILLIS]
        );
        assert!(schema.definitions.iter().all(|definition| {
            definition.target_scope == SettingTargetScopeV1::ConfigurationInstance as i32
                && definition.apply_mode == SettingApplyModeV1::RestartModule as i32
                && definition.client_visibility == SettingClientVisibilityV1::Editable as i32
                && definition.fresh_owner_proof_required
        }));
    }

    #[test]
    fn decoder_accepts_only_exact_loopback_runtime_values() {
        let snapshot = SettingsSnapshotV1 {
            target_id: "ollama-local".to_owned(),
            revision: 7,
            values: vec![
                entry(CHAT_MODEL, Value::StringValue("qwen3:4b".to_owned())),
                entry(PORT, Value::UnsignedIntegerValue(11_434)),
                entry(TIMEOUT_MILLIS, Value::UnsignedIntegerValue(30_000)),
            ],
        };
        validate_settings_snapshot_against_schema_v1(&ollama_ai_settings_schema_v1(), &snapshot)
            .expect("snapshot");
        assert_eq!(
            decode_ollama_ai_settings_v1(&snapshot, "ollama-local"),
            Ok(OllamaAiRuntimeSettingsV1 {
                chat_model: "qwen3:4b".to_owned(),
                port: 11_434,
                timeout_millis: 30_000,
                settings_revision: 7,
            })
        );
    }

    fn entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 { value: Some(value) }),
        }
    }
}
