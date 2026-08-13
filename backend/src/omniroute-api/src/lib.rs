#![forbid(unsafe_code)]
use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingValueV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use prost::Message;
pub const PACKAGE: &str = "makosh-omniroute-api";
pub const OMNIROUTE_OWNER_ID_V1: &str = "omniroute";
pub const OMNIROUTE_MODULE_ID_V1: &str = "makosh-omniroute-runtime";
pub const OMNIROUTE_CREDENTIAL_PROVISION_CAPABILITY_ID_V1: &str =
    "omniroute.ai.credential.provision.v1";
pub const OMNIROUTE_CREDENTIAL_RESOLVE_CAPABILITY_ID_V1: &str =
    "omniroute.ai.credential.resolve.v1";
pub const OMNIROUTE_STORAGE_CAPABILITY_ID_V1: &str = "omniroute.ai.storage.v1";
pub const OMNIROUTE_CREDENTIAL_PURPOSE_ID_V1: &str = "omniroute.api-key.v1";
const BASE_URL: &str = "omniroute.base_url";
const MODEL: &str = "omniroute.model";
const TIMEOUT: &str = "omniroute.timeout_millis";
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OmniRouteSettingsV1 {
    pub base_url: String,
    pub model: String,
    pub timeout_millis: u64,
    pub settings_revision: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmniRouteSettingsErrorV1 {
    InvalidSnapshot,
}
#[must_use]
pub fn omniroute_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: vec![
            definition(
                BASE_URL,
                SettingValueTypeV1::String,
                Value::StringValue("https://api.omniroute.invalid/v1".into()),
            ),
            definition(
                MODEL,
                SettingValueTypeV1::String,
                Value::StringValue("operator-selected".into()),
            ),
            definition(
                TIMEOUT,
                SettingValueTypeV1::UnsignedInteger,
                Value::UnsignedIntegerValue(30_000),
            ),
        ],
    }
}
#[must_use]
pub fn omniroute_settings_schema_bytes_v1() -> Vec<u8> {
    omniroute_settings_schema_v1().encode_to_vec()
}
pub fn decode_omniroute_settings_v1(
    snapshot: &SettingsSnapshotV1,
    target: &str,
) -> Result<OmniRouteSettingsV1, OmniRouteSettingsErrorV1> {
    if target.is_empty()
        || snapshot.target_id != target
        || snapshot.revision == 0
        || snapshot.values.len() != 3
    {
        return Err(OmniRouteSettingsErrorV1::InvalidSnapshot);
    }
    let base_url = string(snapshot, BASE_URL)?;
    let model = string(snapshot, MODEL)?;
    let timeout_millis = unsigned(snapshot, TIMEOUT)?;
    if !valid_base_url(&base_url) || !atom(&model, 128) || !(1..=60_000).contains(&timeout_millis) {
        return Err(OmniRouteSettingsErrorV1::InvalidSnapshot);
    }
    Ok(OmniRouteSettingsV1 {
        base_url,
        model,
        timeout_millis,
        settings_revision: snapshot.revision,
    })
}
fn definition(id: &str, kind: SettingValueTypeV1, value: Value) -> SettingDefinitionV1 {
    SettingDefinitionV1 {
        setting_id: id.into(),
        capability_id: String::new(),
        value_type: kind as i32,
        mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
        target_scope: SettingTargetScopeV1::ConfigurationInstance as i32,
        apply_mode: SettingApplyModeV1::RestartModule as i32,
        client_visibility: SettingClientVisibilityV1::Editable as i32,
        fresh_owner_proof_required: true,
        kernel_controller_id: String::new(),
        display_name: id.into(),
        default_value: Some(SettingValueV1 { value: Some(value) }),
        optional: false,
    }
}
fn value<'a>(s: &'a SettingsSnapshotV1, id: &str) -> Result<&'a Value, OmniRouteSettingsErrorV1> {
    let mut found = None;
    for e in &s.values {
        if e.setting_id == id {
            if found.is_some() {
                return Err(OmniRouteSettingsErrorV1::InvalidSnapshot);
            }
            found = e.value.as_ref().and_then(|v| v.value.as_ref());
        }
    }
    found.ok_or(OmniRouteSettingsErrorV1::InvalidSnapshot)
}
fn string(s: &SettingsSnapshotV1, id: &str) -> Result<String, OmniRouteSettingsErrorV1> {
    match value(s, id)? {
        Value::StringValue(v) => Ok(v.clone()),
        _ => Err(OmniRouteSettingsErrorV1::InvalidSnapshot),
    }
}
fn unsigned(s: &SettingsSnapshotV1, id: &str) -> Result<u64, OmniRouteSettingsErrorV1> {
    match value(s, id)? {
        Value::UnsignedIntegerValue(v) => Ok(*v),
        _ => Err(OmniRouteSettingsErrorV1::InvalidSnapshot),
    }
}
fn valid_base_url(v: &str) -> bool {
    v.len() <= 512
        && v.starts_with("https://")
        && !v.contains('@')
        && !v.contains('?')
        && !v.contains('#')
        && !v.ends_with('/')
}
fn atom(v: &str, max: usize) -> bool {
    !v.is_empty()
        && v.len() <= max
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b':' | b'/'))
}
#[cfg(test)]
mod tests {
    use super::*;
    use makosh_runtime_protocol::v1::SettingsValueEntryV1;
    #[test]
    fn settings_are_bounded_non_secret_and_https_only() {
        let s = SettingsSnapshotV1 {
            target_id: "route-1".into(),
            revision: 1,
            values: vec![
                entry(
                    BASE_URL,
                    Value::StringValue("https://gateway.example/v1".into()),
                ),
                entry(MODEL, Value::StringValue("route/model".into())),
                entry(TIMEOUT, Value::UnsignedIntegerValue(5000)),
            ],
        };
        assert!(decode_omniroute_settings_v1(&s, "route-1").is_ok());
        let schema = format!("{:?}", omniroute_settings_schema_v1()).to_ascii_lowercase();
        assert!(!schema.contains("api_key"));
        assert!(!schema.contains("credential"));
    }
    fn entry(id: &str, v: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: id.into(),
            value: Some(SettingValueV1 { value: Some(v) }),
        }
    }
}
