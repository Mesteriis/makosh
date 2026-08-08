//! Pure validation and enum-decoding helpers for persisted records.

use makosh_kernel_control_store::{
    ModuleRegistrationState, OwnerPinnedArtifactBinding, SettingsApplyState,
    SettingsConfigurationTarget, SettingsSchemaBinding,
};

pub(crate) fn valid_identity_token(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

pub(crate) fn module_registration_state_from_str(value: &str) -> Option<ModuleRegistrationState> {
    match value {
        "pending" => Some(ModuleRegistrationState::Pending),
        "approved" => Some(ModuleRegistrationState::Approved),
        "suspended" => Some(ModuleRegistrationState::Suspended),
        "revoked" => Some(ModuleRegistrationState::Revoked),
        "blocked_incompatible" => Some(ModuleRegistrationState::BlockedIncompatible),
        _ => None,
    }
}

pub(crate) fn settings_apply_state_from_str(value: &str) -> Option<SettingsApplyState> {
    match value {
        "current" => Some(SettingsApplyState::Current),
        "pending_validation" => Some(SettingsApplyState::PendingValidation),
        "pending_apply" => Some(SettingsApplyState::PendingApply),
        "applying" => Some(SettingsApplyState::Applying),
        "awaiting_external_restart" => Some(SettingsApplyState::AwaitingExternalRestart),
        "blocked_config" => Some(SettingsApplyState::BlockedConfig),
        _ => None,
    }
}

pub(crate) fn valid_sanitized_reason_code(value: Option<&str>) -> bool {
    value.is_none_or(|code| {
        !code.is_empty()
            && code.len() <= 128
            && code
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    })
}

pub(crate) fn valid_settings_binding_state(binding: &SettingsSchemaBinding) -> bool {
    valid_settings_state(
        binding.desired_revision(),
        binding.effective_revision(),
        binding.apply_state(),
        binding.sanitized_reason_code(),
    )
}

pub(crate) fn valid_settings_configuration_target(target: &SettingsConfigurationTarget) -> bool {
    valid_identity_token(target.registration_id())
        && valid_identity_token(target.configuration_instance_id())
        && valid_settings_state(
            target.desired_revision(),
            target.effective_revision(),
            target.apply_state(),
            target.sanitized_reason_code(),
        )
}

fn valid_settings_state(
    desired_revision: u64,
    effective_revision: u64,
    apply_state: SettingsApplyState,
    sanitized_reason_code: Option<&str>,
) -> bool {
    effective_revision <= desired_revision
        && valid_sanitized_reason_code(sanitized_reason_code)
        && match apply_state {
            SettingsApplyState::Current => {
                desired_revision == effective_revision && sanitized_reason_code.is_none()
            }
            SettingsApplyState::BlockedConfig => sanitized_reason_code.is_some(),
            SettingsApplyState::PendingValidation
            | SettingsApplyState::PendingApply
            | SettingsApplyState::Applying
            | SettingsApplyState::AwaitingExternalRestart => {
                desired_revision > effective_revision && sanitized_reason_code.is_none()
            }
        }
}

pub(crate) fn valid_owner_pinned_artifact_binding(binding: &OwnerPinnedArtifactBinding) -> bool {
    valid_identity_token(binding.registration_id())
        && binding.binding_revision() > 0
        && binding.canonical_artifact_path().starts_with('/')
        && binding.canonical_artifact_path().len() <= 4096
        && binding.artifact_size() > 0
        && binding.artifact_inode() > 0
}

pub(crate) fn valid_capability_ids(capability_ids: &[String]) -> bool {
    capability_ids.windows(2).all(|pair| pair[0] < pair[1])
        && capability_ids.iter().all(|id| valid_identity_token(id))
}
