//! Module-kind-neutral Settings revision, successor fencing and readiness application.

use std::time::{Duration, Instant};

use makosh_kernel_control_store::{
    ModuleRegistrationState, PlatformStorageBindingStateV1, SettingsApplyState,
};
use makosh_kernel_control_store_sqlite::{SqliteControlStore, StoreError};
use makosh_runtime_protocol::v1::SettingApplyModeV1;
use makosh_runtime_protocol::validation::descriptor::{
    decode_settings_schema_v1, decode_settings_snapshot_v1,
    validate_settings_snapshot_against_schema_v1,
};
use sha2::{Digest, Sha256};

use super::application::{self, ApplyAcknowledgement};
use crate::platform::storage::{provisioning, successor};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const READY_DEADLINE: Duration = Duration::from_secs(10);
const READY_POLL_INTERVAL: Duration = Duration::from_millis(20);
const VALIDATION_FAILED: &str = "settings_validation_failed";
const REPLACEMENT_FAILED: &str = "managed_replacement_failed";
const READINESS_FAILED: &str = "managed_readiness_failed";

pub(crate) struct PreparedManagedSettingsV1 {
    revision: u64,
    snapshot_bytes: Vec<u8>,
}

impl PreparedManagedSettingsV1 {
    #[must_use]
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub(crate) fn snapshot_bytes(&self) -> &[u8] {
        &self.snapshot_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ManagedApplyKindV1 {
    InitialLaunch,
    Replacement,
}

pub(crate) fn prepare(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    configuration_instance_id: &str,
    storage_capability_id: &str,
    expected_desired_revision: u64,
) -> Result<PreparedManagedSettingsV1, String> {
    let apply_kind = validate_current_target(
        store,
        supervisor,
        registration_id,
        configuration_instance_id,
        storage_capability_id,
        expected_desired_revision,
    )?;
    let snapshot_bytes = match validate_desired_snapshot(
        store,
        registration_id,
        configuration_instance_id,
        expected_desired_revision,
    ) {
        Ok(bytes) => bytes,
        Err(error) => {
            block(
                store,
                registration_id,
                configuration_instance_id,
                expected_desired_revision,
                VALIDATION_FAILED,
            );
            return Err(error);
        }
    };
    application::acknowledge_target(
        store,
        registration_id,
        configuration_instance_id,
        expected_desired_revision,
        ApplyAcknowledgement::ValidationAccepted,
    )?;
    application::acknowledge_target(
        store,
        registration_id,
        configuration_instance_id,
        expected_desired_revision,
        ApplyAcknowledgement::ApplyStarted,
    )?;
    if apply_kind == ManagedApplyKindV1::Replacement
        && let Err(error) =
            reserve_successor_storage(store, supervisor, registration_id, storage_capability_id)
    {
        block(
            store,
            registration_id,
            configuration_instance_id,
            expected_desired_revision,
            REPLACEMENT_FAILED,
        );
        return Err(error);
    }
    Ok(PreparedManagedSettingsV1 {
        revision: expected_desired_revision,
        snapshot_bytes,
    })
}

pub(crate) fn wait_for_ready_and_confirm(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    configuration_instance_id: &str,
    revision: u64,
) -> Result<(), String> {
    let deadline = Instant::now() + READY_DEADLINE;
    loop {
        if supervisor.relay_port().is_ready(registration_id) == Ok(true) {
            return application::acknowledge_target(
                store,
                registration_id,
                configuration_instance_id,
                revision,
                ApplyAcknowledgement::RuntimeApplied,
            );
        }
        if Instant::now() >= deadline
            || (!supervisor.is_active(registration_id)?
                && supervisor.last_failure(registration_id)?.is_some())
        {
            let _ = supervisor.stop_if_active(registration_id);
            block(
                store,
                registration_id,
                configuration_instance_id,
                revision,
                READINESS_FAILED,
            );
            return Err("managed module did not acknowledge the desired settings".to_owned());
        }
        std::thread::sleep(READY_POLL_INTERVAL);
    }
}

pub(crate) fn block_after_launch_failure(
    store: &SqliteControlStore,
    registration_id: &str,
    configuration_instance_id: &str,
    revision: u64,
) {
    block(
        store,
        registration_id,
        configuration_instance_id,
        revision,
        REPLACEMENT_FAILED,
    );
}

fn validate_current_target(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    configuration_instance_id: &str,
    storage_capability_id: &str,
    expected_desired_revision: u64,
) -> Result<ManagedApplyKindV1, String> {
    let registration = store
        .module_registration(registration_id)
        .map_err(store_error)?
        .filter(|registration| registration.state() == ModuleRegistrationState::Approved)
        .ok_or_else(|| "managed module registration is unavailable".to_owned())?;
    if registration.registration_id() != registration_id
        || storage_capability_id.trim().is_empty()
        || expected_desired_revision == 0
    {
        return Err("managed module settings target is unavailable".to_owned());
    }
    let active = supervisor.is_active(registration_id)?;
    let settings = store
        .settings_configuration_target(registration_id, configuration_instance_id)
        .map_err(store_error)?
        .ok_or_else(|| "managed module settings are unavailable".to_owned())?;
    let retries_readiness_failure =
        retryable_readiness_failure(settings.apply_state(), settings.sanitized_reason_code());
    if settings.desired_revision() != expected_desired_revision
        || settings.effective_revision() >= expected_desired_revision
        || (settings.apply_state() != SettingsApplyState::PendingValidation
            && !retries_readiness_failure)
    {
        return Err("managed module settings revision is stale".to_owned());
    }
    let storage = store
        .platform_storage_binding(registration_id, storage_capability_id)
        .map_err(store_error)?
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .ok_or_else(|| "managed module Storage binding is unavailable".to_owned())?;
    if storage.registration_id() != registration_id
        || storage.capability_id() != storage_capability_id
    {
        return Err("managed module Storage binding is unavailable".to_owned());
    }
    if retries_readiness_failure {
        store
            .transition_settings_apply_state_for_target(
                registration_id,
                configuration_instance_id,
                expected_desired_revision,
                SettingsApplyState::PendingValidation,
                None,
            )
            .map_err(store_error)?;
    }
    Ok(classify_apply_kind(settings.effective_revision(), active))
}

fn retryable_readiness_failure(state: SettingsApplyState, reason: Option<&str>) -> bool {
    state == SettingsApplyState::BlockedConfig && reason == Some(READINESS_FAILED)
}

fn classify_apply_kind(effective_revision: u64, runtime_active: bool) -> ManagedApplyKindV1 {
    match (effective_revision, runtime_active) {
        (_, false) => ManagedApplyKindV1::InitialLaunch,
        (_, true) => ManagedApplyKindV1::Replacement,
    }
}

fn validate_desired_snapshot(
    store: &SqliteControlStore,
    registration_id: &str,
    configuration_instance_id: &str,
    revision: u64,
) -> Result<Vec<u8>, String> {
    let binding = store
        .settings_schema_binding(registration_id)
        .map_err(store_error)?
        .ok_or_else(|| "managed module settings are unavailable".to_owned())?;
    let schema_bytes = store
        .settings_schema_artifact(registration_id)
        .map_err(store_error)?
        .ok_or_else(|| "managed module settings schema is unavailable".to_owned())?;
    let schema_sha256: [u8; 32] = Sha256::digest(&schema_bytes).into();
    if schema_sha256 != *binding.schema_sha256() {
        return Err("managed module settings schema binding is stale".to_owned());
    }
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "managed module settings schema is invalid".to_owned())?;
    if schema.definitions.is_empty()
        || schema
            .definitions
            .iter()
            .any(|definition| definition.apply_mode != SettingApplyModeV1::RestartModule as i32)
    {
        return Err("managed module settings require an unsupported apply mode".to_owned());
    }
    let (stored_revision, snapshot_bytes) = store
        .desired_settings_snapshot_for_target(registration_id, configuration_instance_id)
        .map_err(store_error)?
        .ok_or_else(|| "managed module desired settings are unavailable".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(&snapshot_bytes)
        .map_err(|_| "managed module desired settings are invalid".to_owned())?;
    if stored_revision != revision
        || snapshot.revision != revision
        || snapshot.target_id != configuration_instance_id
        || validate_settings_snapshot_against_schema_v1(&schema, &snapshot).is_err()
    {
        return Err("managed module desired settings are stale".to_owned());
    }
    Ok(snapshot_bytes)
}

fn reserve_successor_storage(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    storage_capability_id: &str,
) -> Result<(), String> {
    let predecessor = store
        .platform_storage_binding(registration_id, storage_capability_id)
        .map_err(store_error)?
        .ok_or_else(|| "managed module Storage binding is unavailable".to_owned())?;
    let issue = successor::issue_after(&predecessor)?;
    let (_, binding) = successor::reserve(
        supervisor,
        store,
        registration_id,
        storage_capability_id,
        issue,
    )?;
    provisioning::apply_reserved_binding(supervisor, store, &binding)
}

fn block(
    store: &SqliteControlStore,
    registration_id: &str,
    configuration_instance_id: &str,
    revision: u64,
    reason: &str,
) {
    let _ = store.transition_settings_apply_state_for_target(
        registration_id,
        configuration_instance_id,
        revision,
        SettingsApplyState::BlockedConfig,
        Some(reason),
    );
}

fn store_error(error: StoreError) -> String {
    format!("{error:?}")
}

#[cfg(test)]
mod tests {
    use makosh_kernel_control_store::SettingsApplyState;

    use super::{ManagedApplyKindV1, classify_apply_kind, retryable_readiness_failure};

    #[test]
    fn inactive_runtime_relaunches_for_pending_successor_settings() {
        assert_eq!(
            classify_apply_kind(0, false),
            ManagedApplyKindV1::InitialLaunch
        );
        assert_eq!(
            classify_apply_kind(3, false),
            ManagedApplyKindV1::InitialLaunch
        );
        assert_eq!(
            classify_apply_kind(3, true),
            ManagedApplyKindV1::Replacement
        );
    }

    #[test]
    fn only_readiness_failure_is_retryable_without_a_new_settings_revision() {
        assert!(retryable_readiness_failure(
            SettingsApplyState::BlockedConfig,
            Some("managed_readiness_failed"),
        ));
        assert!(!retryable_readiness_failure(
            SettingsApplyState::BlockedConfig,
            Some("settings_validation_failed"),
        ));
        assert!(!retryable_readiness_failure(
            SettingsApplyState::PendingValidation,
            None,
        ));
    }
}
