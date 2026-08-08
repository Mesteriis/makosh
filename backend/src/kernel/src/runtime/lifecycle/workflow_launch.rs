//! Managed workflow launch composition below owner-control transports.

use std::path::Path;

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{
        ManagedWorkflowRuntimeConfigurationV1, ModuleKindV1, SettingTargetScopeV1, SettingsSchemaV1,
    },
    validation::{
        descriptor::{decode_settings_schema_v1, decode_settings_snapshot_v1},
        managed_workflow_runtime::validate_managed_workflow_runtime_configuration,
    },
};

pub(crate) fn require_bound_workflow_kind(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    registration_id: &str,
) -> Result<(), String> {
    macos_managed_runtime_launch::require_bound_module_kind(
        store,
        runtime_dir,
        registration_id,
        ModuleKindV1::Workflow,
    )
}

use crate::platform::macos::managed_launch as macos_managed_runtime_launch;
use crate::runtime::lifecycle::{
    integration_launch::{
        admitted_workflow_configuration_instances, startup_configuration_instance,
    },
    supervisor::ManagedRuntimeSupervisor,
};

pub(crate) fn configuration_required_but_unavailable(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<bool, String> {
    let schema_bytes = store
        .settings_schema_artifact(registration_id)
        .map_err(|_| "managed workflow settings schema is unavailable".to_owned())?
        .ok_or_else(|| "managed workflow settings schema is unavailable".to_owned())?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "managed workflow settings schema is invalid".to_owned())?;
    Ok(schema_requires_configuration(&schema)
        && startup_configuration_instance(store, registration_id, "")?.is_none())
}

fn schema_requires_configuration(schema: &SettingsSchemaV1) -> bool {
    schema.definitions.iter().any(|definition| {
        definition.target_scope == SettingTargetScopeV1::ConfigurationInstance as i32
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn launch_reserved(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    logical_owner_id: &str,
    registration_id: &str,
    storage_capability_id: &str,
    configuration_instance_id: &str,
    applying_settings_snapshot_bytes: Option<Vec<u8>>,
) -> Result<u64, String> {
    let reservation = macos_managed_runtime_launch::load(supervisor, store, registration_id)?;
    let _registration = store
        .module_registration(registration_id)
        .map_err(|_| "managed workflow registration is unavailable".to_owned())?
        .ok_or_else(|| "managed workflow registration is unavailable".to_owned())?;
    let binding = store
        .platform_storage_binding(registration_id, storage_capability_id)
        .map_err(|_| "managed workflow Storage binding is unavailable".to_owned())?
        .filter(|value| value.state() == PlatformStorageBindingStateV1::Active)
        .ok_or_else(|| "managed workflow Storage binding is unavailable".to_owned())?;
    let storage_topology = crate::platform::storage::topology::current(store)?;
    let vault = crate::platform::vault::status::read_current(store, &supervisor.relay_port())?;
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &storage_topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )?;
    let event_topology = store
        .platform_event_hub_topology()
        .map_err(|_| "Event Hub topology is unavailable".to_owned())?
        .ok_or_else(|| "Event Hub topology is unavailable".to_owned())?;
    let granted_capability_ids = effective_granted_capability_ids(store, registration_id)?;
    let selected_settings = select_settings(
        store,
        registration_id,
        configuration_instance_id,
        applying_settings_snapshot_bytes,
    )?;
    let configuration_instances = match selected_settings.as_ref() {
        Some(snapshot) => admitted_workflow_configuration_instances(
            store,
            registration_id,
            configuration_instance_id,
            &snapshot.bytes,
        )?,
        None => Vec::new(),
    };
    let configuration = ManagedWorkflowRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: logical_owner_id.to_owned(),
        registration_id: registration_id.to_owned(),
        runtime_instance_id: reservation.runtime_instance_id().to_owned(),
        runtime_generation: reservation.runtime_generation(),
        grant_epoch: reservation.grant_epoch(),
        storage: Some(storage),
        event_hub_endpoint: event_topology.nats_endpoint().to_owned(),
        event_credential_revision: event_topology.credential_revision(),
        runtime_artifacts: Vec::new(),
        configuration_instance_id: configuration_instance_id.to_owned(),
        settings_revision: selected_settings.as_ref().map_or(0, |value| value.revision),
        configuration_instances,
    };
    validate_managed_workflow_runtime_configuration(&configuration)
        .map_err(|_| "managed workflow runtime configuration is invalid".to_owned())?;
    match selected_settings {
        Some(snapshot) => macos_managed_runtime_launch::start_reserved_workflow_with_settings(
            supervisor,
            runtime_dir,
            reservation,
            configuration,
            snapshot.bytes,
            &granted_capability_ids,
        ),
        None => macos_managed_runtime_launch::start_reserved_workflow(
            supervisor,
            runtime_dir,
            reservation,
            configuration,
            &granted_capability_ids,
        ),
    }
}

struct SelectedWorkflowSettingsV1 {
    revision: u64,
    bytes: Vec<u8>,
}

fn select_settings(
    store: &SqliteControlStore,
    registration_id: &str,
    configuration_instance_id: &str,
    applying_settings_snapshot_bytes: Option<Vec<u8>>,
) -> Result<Option<SelectedWorkflowSettingsV1>, String> {
    if let Some(bytes) = applying_settings_snapshot_bytes {
        let snapshot = decode_settings_snapshot_v1(&bytes)
            .map_err(|_| "managed workflow applying settings are invalid".to_owned())?;
        if configuration_instance_id.is_empty()
            || snapshot.target_id != configuration_instance_id
            || snapshot.revision == 0
        {
            return Err("managed workflow applying settings are invalid".to_owned());
        }
        return Ok(Some(SelectedWorkflowSettingsV1 {
            revision: snapshot.revision,
            bytes,
        }));
    }
    if configuration_instance_id.is_empty() {
        return Ok(None);
    }
    let snapshot =
        crate::runtime::lifecycle::integration_launch::admitted_settings_snapshot_for_target(
            store,
            registration_id,
            configuration_instance_id,
        )?;
    Ok(Some(SelectedWorkflowSettingsV1 {
        revision: snapshot.revision,
        bytes: snapshot.bytes,
    }))
}

fn effective_granted_capability_ids(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<Vec<String>, String> {
    store
        .module_grant_snapshot(registration_id)
        .map_err(|_| "managed workflow grants are unavailable".to_owned())?
        .and_then(|snapshot| {
            snapshot
                .effective_grants()
                .map(|grants| grants.capability_ids().to_vec())
        })
        .ok_or_else(|| "managed workflow grants are unavailable".to_owned())
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::{
        SettingDefinitionV1, SettingTargetScopeV1, SettingsSchemaV1,
    };

    use super::schema_requires_configuration;

    #[test]
    fn only_configuration_scoped_workflow_settings_defer_initial_launch() {
        let configuration_scoped = SettingsSchemaV1 {
            definitions: vec![SettingDefinitionV1 {
                target_scope: SettingTargetScopeV1::ConfigurationInstance as i32,
                ..Default::default()
            }],
            ..Default::default()
        };
        let registration_scoped = SettingsSchemaV1 {
            definitions: vec![SettingDefinitionV1 {
                target_scope: SettingTargetScopeV1::ModuleRegistration as i32,
                ..Default::default()
            }],
            ..Default::default()
        };

        assert!(schema_requires_configuration(&configuration_scoped));
        assert!(!schema_requires_configuration(&registration_scoped));
        assert!(!schema_requires_configuration(&SettingsSchemaV1::default()));
    }
}
