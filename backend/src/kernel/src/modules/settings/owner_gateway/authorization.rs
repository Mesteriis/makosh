//! Owner-device and target-registration checks for public Settings mutations.

use makosh_gateway_protocol::v1::{
    PrepareOwnerModuleSettingsRequestV1, prepare_owner_module_settings_request_v1,
};
use makosh_gateway_runtime::{OwnerBrowserPrincipalV1, OwnerModuleSettingsRouteErrorV1};
use makosh_kernel_control_store::ModuleRegistrationState;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID;

use crate::platform::gateway::owner_device_proof::{self, OwnerDeviceProofErrorV1};

pub(super) struct AuthorizedSettingsTargetV1 {
    pub(super) registration_id: String,
    pub(super) grant_epoch: u64,
}

pub(super) fn authorize_target(
    store: &SqliteControlStore,
    principal: &OwnerBrowserPrincipalV1,
    request: &PrepareOwnerModuleSettingsRequestV1,
) -> Result<AuthorizedSettingsTargetV1, OwnerModuleSettingsRouteErrorV1> {
    owner_device_proof::validate_active_principal(store, principal).map_err(map_proof_error)?;
    if request.operation_id.len() != 16 || request.operation_id.iter().all(|byte| *byte == 0) {
        return Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument);
    }
    let registration_id = match request.operation.as_ref() {
        Some(prepare_owner_module_settings_request_v1::Operation::UpdateDesired(update)) => {
            update_registration_id(update)?
        }
        Some(prepare_owner_module_settings_request_v1::Operation::ApplyManagedIntegration(
            apply,
        )) => apply_registration_id(apply)?,
        Some(prepare_owner_module_settings_request_v1::Operation::ApplyManagedWorkflow(apply)) => {
            apply_workflow_registration_id(apply)?
        }
        Some(prepare_owner_module_settings_request_v1::Operation::ExportEffective(export)) => {
            export_registration_id(export)?
        }
        Some(prepare_owner_module_settings_request_v1::Operation::CreateConfigurationTarget(
            create,
        )) => create_registration_id(create)?,
        None => return Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument),
    };
    let snapshot = store
        .module_grant_snapshot(registration_id)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::NotFound)?;
    let registration = snapshot.registration();
    let grants = snapshot
        .effective_grants()
        .ok_or(OwnerModuleSettingsRouteErrorV1::PermissionDenied)?;
    if registration.state() != ModuleRegistrationState::Approved
        || registration.grant_epoch() != grants.grant_epoch()
    {
        return Err(OwnerModuleSettingsRouteErrorV1::PermissionDenied);
    }
    if matches!(
        request.operation.as_ref(),
        Some(prepare_owner_module_settings_request_v1::Operation::CreateConfigurationTarget(_))
    ) && !grants
        .capability_ids()
        .iter()
        .any(|capability_id| capability_id == SETTINGS_CONFIGURATION_CATALOG_CAPABILITY_ID)
    {
        return Err(OwnerModuleSettingsRouteErrorV1::PermissionDenied);
    }
    store
        .settings_schema_binding(registration_id)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::NotFound)?;
    if let Some(configuration_instance_id) = operation_configuration_instance_id(request) {
        store
            .settings_configuration_target(registration_id, configuration_instance_id)
            .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
            .ok_or(OwnerModuleSettingsRouteErrorV1::NotFound)?;
    }
    Ok(AuthorizedSettingsTargetV1 {
        registration_id: registration_id.to_owned(),
        grant_epoch: grants.grant_epoch(),
    })
}

fn update_registration_id(
    update: &makosh_gateway_protocol::v1::UpdateOwnerModuleSettingsV1,
) -> Result<&str, OwnerModuleSettingsRouteErrorV1> {
    if update.registration_id.is_empty()
        || update.configuration_instance_id.is_empty()
        || update.expected_desired_revision == u64::MAX
        || update.values.len() > 256
    {
        return Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument);
    }
    Ok(&update.registration_id)
}

fn apply_registration_id(
    apply: &makosh_gateway_protocol::v1::ApplyOwnerManagedIntegrationSettingsV1,
) -> Result<&str, OwnerModuleSettingsRouteErrorV1> {
    if apply.registration_id.is_empty()
        || apply.storage_capability_id.is_empty()
        || apply.configuration_instance_id.is_empty()
        || apply.expected_desired_revision == 0
    {
        return Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument);
    }
    Ok(&apply.registration_id)
}

fn apply_workflow_registration_id(
    apply: &makosh_gateway_protocol::v1::ApplyOwnerManagedWorkflowSettingsV1,
) -> Result<&str, OwnerModuleSettingsRouteErrorV1> {
    if apply.registration_id.is_empty()
        || apply.storage_capability_id.is_empty()
        || apply.configuration_instance_id.is_empty()
        || apply.expected_desired_revision == 0
    {
        return Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument);
    }
    Ok(&apply.registration_id)
}

fn export_registration_id(
    export: &makosh_gateway_protocol::v1::ExportEffectiveOwnerModuleSettingsV1,
) -> Result<&str, OwnerModuleSettingsRouteErrorV1> {
    if export.registration_id.is_empty()
        || export.configuration_instance_id.is_empty()
        || export.expected_effective_revision == 0
    {
        return Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument);
    }
    Ok(&export.registration_id)
}

fn create_registration_id(
    create: &makosh_gateway_protocol::v1::CreateOwnerModuleSettingsTargetV1,
) -> Result<&str, OwnerModuleSettingsRouteErrorV1> {
    if create.registration_id.is_empty() {
        return Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument);
    }
    Ok(&create.registration_id)
}

fn operation_configuration_instance_id(
    request: &PrepareOwnerModuleSettingsRequestV1,
) -> Option<&str> {
    match request.operation.as_ref()? {
        prepare_owner_module_settings_request_v1::Operation::UpdateDesired(update) => {
            Some(&update.configuration_instance_id)
        }
        prepare_owner_module_settings_request_v1::Operation::ApplyManagedIntegration(apply) => {
            Some(&apply.configuration_instance_id)
        }
        prepare_owner_module_settings_request_v1::Operation::ApplyManagedWorkflow(apply) => {
            Some(&apply.configuration_instance_id)
        }
        prepare_owner_module_settings_request_v1::Operation::ExportEffective(export) => {
            Some(&export.configuration_instance_id)
        }
        prepare_owner_module_settings_request_v1::Operation::CreateConfigurationTarget(_) => None,
    }
}

pub(super) fn map_proof_error(error: OwnerDeviceProofErrorV1) -> OwnerModuleSettingsRouteErrorV1 {
    match error {
        OwnerDeviceProofErrorV1::InvalidArgument => {
            OwnerModuleSettingsRouteErrorV1::InvalidArgument
        }
        OwnerDeviceProofErrorV1::PermissionDenied => {
            OwnerModuleSettingsRouteErrorV1::PermissionDenied
        }
        OwnerDeviceProofErrorV1::Internal => OwnerModuleSettingsRouteErrorV1::Internal,
    }
}
