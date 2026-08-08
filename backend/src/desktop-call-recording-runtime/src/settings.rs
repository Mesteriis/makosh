use makosh_runtime_protocol::v1::SettingsSchemaV1;
use prost::Message;

pub const SETTINGS_SCHEMA_MAJOR_V1: u32 = 1;
pub const SETTINGS_SCHEMA_REVISION_V1: u32 = 1;

#[must_use]
pub fn settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: SETTINGS_SCHEMA_MAJOR_V1,
        revision: SETTINGS_SCHEMA_REVISION_V1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn settings_schema_bytes_v1() -> Vec<u8> {
    settings_schema_v1().encode_to_vec()
}
