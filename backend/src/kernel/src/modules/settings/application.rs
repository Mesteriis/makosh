//! Persists Settings validation and apply lifecycle acknowledgements.

use makosh_kernel_control_store::{SettingsApplyState, SettingsRegistryStore};
use makosh_kernel_control_store_sqlite::StoreError;

pub enum ApplyAcknowledgement {
    ValidationAccepted,
    ValidationRejected { reason_code: String },
    ApplyStarted,
    ExternalRestartRequired,
    RuntimeApplied,
}

pub fn parse_acknowledgement(
    value: &str,
    reason_code: Option<&str>,
) -> Result<ApplyAcknowledgement, String> {
    match value {
        "validation_accepted" if reason_code.is_none() => {
            Ok(ApplyAcknowledgement::ValidationAccepted)
        }
        "validation_rejected" => reason_code
            .filter(|item| !item.is_empty())
            .map(|item| ApplyAcknowledgement::ValidationRejected {
                reason_code: item.to_owned(),
            })
            .ok_or_else(|| "validation_rejected requires a reason code".to_owned()),
        "apply_started" if reason_code.is_none() => Ok(ApplyAcknowledgement::ApplyStarted),
        "external_restart_required" if reason_code.is_none() => {
            Ok(ApplyAcknowledgement::ExternalRestartRequired)
        }
        "runtime_applied" if reason_code.is_none() => Ok(ApplyAcknowledgement::RuntimeApplied),
        _ => Err("settings lifecycle acknowledgement is invalid".to_owned()),
    }
}

pub fn acknowledge<S>(
    store: &S,
    registration_id: &str,
    revision: u64,
    acknowledgement: ApplyAcknowledgement,
) -> Result<(), String>
where
    S: SettingsRegistryStore<Error = StoreError>,
{
    acknowledge_target(
        store,
        registration_id,
        registration_id,
        revision,
        acknowledgement,
    )
}

pub fn acknowledge_target<S>(
    store: &S,
    registration_id: &str,
    configuration_instance_id: &str,
    revision: u64,
    acknowledgement: ApplyAcknowledgement,
) -> Result<(), String>
where
    S: SettingsRegistryStore<Error = StoreError>,
{
    match acknowledgement {
        ApplyAcknowledgement::ValidationAccepted => transition(
            store,
            registration_id,
            configuration_instance_id,
            revision,
            SettingsApplyState::PendingApply,
            None,
        ),
        ApplyAcknowledgement::ValidationRejected { reason_code } => transition(
            store,
            registration_id,
            configuration_instance_id,
            revision,
            SettingsApplyState::BlockedConfig,
            Some(&reason_code),
        ),
        ApplyAcknowledgement::ApplyStarted => transition(
            store,
            registration_id,
            configuration_instance_id,
            revision,
            SettingsApplyState::Applying,
            None,
        ),
        ApplyAcknowledgement::ExternalRestartRequired => transition(
            store,
            registration_id,
            configuration_instance_id,
            revision,
            SettingsApplyState::AwaitingExternalRestart,
            None,
        ),
        ApplyAcknowledgement::RuntimeApplied => store
            .confirm_effective_settings_revision_for_target(
                registration_id,
                configuration_instance_id,
                revision,
            )
            .map_err(|error| format!("{error:?}")),
    }
}

fn transition<S>(
    store: &S,
    registration_id: &str,
    configuration_instance_id: &str,
    revision: u64,
    next: SettingsApplyState,
    reason_code: Option<&str>,
) -> Result<(), String>
where
    S: SettingsRegistryStore<Error = StoreError>,
{
    store
        .transition_settings_apply_state_for_target(
            registration_id,
            configuration_instance_id,
            revision,
            next,
            reason_code,
        )
        .map_err(|error| format!("{error:?}"))
}
