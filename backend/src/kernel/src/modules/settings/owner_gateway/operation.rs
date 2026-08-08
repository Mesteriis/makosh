//! Canonical Settings mutation and managed-integration apply execution.

use std::path::Path;

use makosh_gateway_protocol::v1::{
    ApplyOwnerManagedIntegrationSettingsReceiptV1, ApplyOwnerManagedIntegrationSettingsV1,
    ApplyOwnerManagedWorkflowSettingsReceiptV1, ApplyOwnerManagedWorkflowSettingsV1,
    CommitOwnerModuleSettingsResponseV1, UpdateOwnerModuleSettingsReceiptV1,
    UpdateOwnerModuleSettingsV1, commit_owner_module_settings_response_v1,
};
use makosh_gateway_runtime::OwnerModuleSettingsRouteErrorV1;
use makosh_kernel_control_store::SettingsApplyState;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use prost::Message;

use super::values::canonical_snapshot;
use crate::modules::settings::{managed_application, mutation};
use crate::runtime::lifecycle::{
    integration_launch, supervisor::ManagedRuntimeSupervisor, workflow_launch,
};

pub(super) fn update_desired(
    store: &SqliteControlStore,
    operation_id: Vec<u8>,
    update: UpdateOwnerModuleSettingsV1,
) -> Result<CommitOwnerModuleSettingsResponseV1, OwnerModuleSettingsRouteErrorV1> {
    let snapshot = canonical_snapshot(
        &update.configuration_instance_id,
        update.expected_desired_revision,
        update.values,
    )?;
    let desired_revision = mutation::commit_after_owner_authorization_for_target(
        store,
        &update.registration_id,
        &update.configuration_instance_id,
        update.expected_desired_revision,
        &snapshot.encode_to_vec(),
    )
    .map_err(map_mutation_error)?;
    let target = store
        .settings_configuration_target(&update.registration_id, &update.configuration_instance_id)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::Internal)?;
    Ok(CommitOwnerModuleSettingsResponseV1 {
        major: 1,
        operation_id,
        result: Some(commit_owner_module_settings_response_v1::Result::Updated(
            UpdateOwnerModuleSettingsReceiptV1 {
                registration_id: update.registration_id,
                desired_revision,
                apply_state: target.apply_state().as_str().to_owned(),
                configuration_instance_id: update.configuration_instance_id,
            },
        )),
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_managed_integration(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    operation_id: Vec<u8>,
    apply: ApplyOwnerManagedIntegrationSettingsV1,
) -> Result<CommitOwnerModuleSettingsResponseV1, OwnerModuleSettingsRouteErrorV1> {
    let prepared = managed_application::prepare(
        store,
        supervisor,
        &apply.registration_id,
        &apply.configuration_instance_id,
        &apply.storage_capability_id,
        apply.expected_desired_revision,
    )
    .map_err(map_apply_preparation_error)?;
    let launch = integration_launch::launch_reserved(
        store,
        data_dir,
        runtime_dir,
        supervisor,
        &apply.registration_id,
        &apply.storage_capability_id,
        &apply.configuration_instance_id,
        apply.request_host_bridge,
        Some(prepared.snapshot_bytes().to_vec()),
    );
    let (runtime_generation, host_bridge_socket_path) = match launch {
        Ok(launch) => launch,
        Err(_) => {
            managed_application::block_after_launch_failure(
                store,
                &apply.registration_id,
                &apply.configuration_instance_id,
                prepared.revision(),
            );
            return Err(OwnerModuleSettingsRouteErrorV1::Unavailable);
        }
    };
    managed_application::wait_for_ready_and_confirm(
        store,
        supervisor,
        &apply.registration_id,
        &apply.configuration_instance_id,
        prepared.revision(),
    )
    .map_err(|_| OwnerModuleSettingsRouteErrorV1::Unavailable)?;
    let target = store
        .settings_configuration_target(&apply.registration_id, &apply.configuration_instance_id)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::Internal)?;
    if target.effective_revision() != prepared.revision()
        || target.apply_state() != SettingsApplyState::Current
    {
        return Err(OwnerModuleSettingsRouteErrorV1::Internal);
    }
    Ok(CommitOwnerModuleSettingsResponseV1 {
        major: 1,
        operation_id,
        result: Some(commit_owner_module_settings_response_v1::Result::Applied(
            ApplyOwnerManagedIntegrationSettingsReceiptV1 {
                registration_id: apply.registration_id,
                effective_revision: target.effective_revision(),
                runtime_generation,
                apply_state: target.apply_state().as_str().to_owned(),
                host_bridge_socket_path,
                configuration_instance_id: apply.configuration_instance_id,
            },
        )),
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_managed_workflow(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    logical_owner_id: &str,
    operation_id: Vec<u8>,
    apply: ApplyOwnerManagedWorkflowSettingsV1,
) -> Result<CommitOwnerModuleSettingsResponseV1, OwnerModuleSettingsRouteErrorV1> {
    workflow_launch::require_bound_workflow_kind(store, runtime_dir, &apply.registration_id)
        .map_err(map_workflow_kind_error)?;
    let prepared = managed_application::prepare(
        store,
        supervisor,
        &apply.registration_id,
        &apply.configuration_instance_id,
        &apply.storage_capability_id,
        apply.expected_desired_revision,
    )
    .map_err(map_apply_preparation_error)?;
    let launch = workflow_launch::launch_reserved(
        store,
        runtime_dir,
        supervisor,
        logical_owner_id,
        &apply.registration_id,
        &apply.storage_capability_id,
        &apply.configuration_instance_id,
        Some(prepared.snapshot_bytes().to_vec()),
    );
    let runtime_generation = match launch {
        Ok(runtime_generation) => runtime_generation,
        Err(_) => {
            managed_application::block_after_launch_failure(
                store,
                &apply.registration_id,
                &apply.configuration_instance_id,
                prepared.revision(),
            );
            return Err(OwnerModuleSettingsRouteErrorV1::Unavailable);
        }
    };
    managed_application::wait_for_ready_and_confirm(
        store,
        supervisor,
        &apply.registration_id,
        &apply.configuration_instance_id,
        prepared.revision(),
    )
    .map_err(|_| OwnerModuleSettingsRouteErrorV1::Unavailable)?;
    let target = store
        .settings_configuration_target(&apply.registration_id, &apply.configuration_instance_id)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::Internal)?;
    if target.effective_revision() != prepared.revision()
        || target.apply_state() != SettingsApplyState::Current
    {
        return Err(OwnerModuleSettingsRouteErrorV1::Internal);
    }
    Ok(CommitOwnerModuleSettingsResponseV1 {
        major: 1,
        operation_id,
        result: Some(
            commit_owner_module_settings_response_v1::Result::WorkflowApplied(
                ApplyOwnerManagedWorkflowSettingsReceiptV1 {
                    registration_id: apply.registration_id,
                    configuration_instance_id: apply.configuration_instance_id,
                    effective_revision: target.effective_revision(),
                    runtime_generation,
                    apply_state: target.apply_state().as_str().to_owned(),
                },
            ),
        ),
    })
}

fn map_mutation_error(error: String) -> OwnerModuleSettingsRouteErrorV1 {
    if error.contains("conflict") {
        OwnerModuleSettingsRouteErrorV1::Conflict
    } else if error.contains("not admitted") || error.contains("unavailable") {
        OwnerModuleSettingsRouteErrorV1::NotFound
    } else {
        OwnerModuleSettingsRouteErrorV1::InvalidArgument
    }
}

fn map_apply_preparation_error(error: String) -> OwnerModuleSettingsRouteErrorV1 {
    if error.contains("revision") || error.contains("active") {
        OwnerModuleSettingsRouteErrorV1::Conflict
    } else if error.contains("unavailable") {
        OwnerModuleSettingsRouteErrorV1::NotFound
    } else {
        OwnerModuleSettingsRouteErrorV1::InvalidArgument
    }
}

fn map_workflow_kind_error(error: String) -> OwnerModuleSettingsRouteErrorV1 {
    if error.contains("kind") {
        OwnerModuleSettingsRouteErrorV1::InvalidArgument
    } else if error.contains("unavailable") {
        OwnerModuleSettingsRouteErrorV1::NotFound
    } else {
        OwnerModuleSettingsRouteErrorV1::Internal
    }
}
