//! Fresh-proof projection of one current client-visible Settings snapshot.

use makosh_gateway_protocol::v1::{
    CommitOwnerModuleSettingsResponseV1, ExportEffectiveOwnerModuleSettingsReceiptV1,
    ExportEffectiveOwnerModuleSettingsV1, commit_owner_module_settings_response_v1,
};
use makosh_gateway_runtime::OwnerModuleSettingsRouteErrorV1;
use makosh_kernel_control_store::SettingsApplyState;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::validation::descriptor::{
    decode_settings_schema_v1, decode_settings_snapshot_v1,
    validate_settings_snapshot_against_schema_v1,
};
use sha2::{Digest, Sha256};

use super::values::visible_public_values;

pub(super) fn effective(
    store: &SqliteControlStore,
    operation_id: Vec<u8>,
    export: ExportEffectiveOwnerModuleSettingsV1,
) -> Result<CommitOwnerModuleSettingsResponseV1, OwnerModuleSettingsRouteErrorV1> {
    let binding = store
        .settings_schema_binding(&export.registration_id)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::NotFound)?;
    let target = store
        .settings_configuration_target(&export.registration_id, &export.configuration_instance_id)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::NotFound)?;
    if target.desired_revision() != target.effective_revision()
        || target.effective_revision() != export.expected_effective_revision
        || target.apply_state() != SettingsApplyState::Current
    {
        return Err(OwnerModuleSettingsRouteErrorV1::Conflict);
    }
    let schema_bytes = store
        .settings_schema_artifact(&export.registration_id)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::NotFound)?;
    let schema_sha256: [u8; 32] = Sha256::digest(&schema_bytes).into();
    if schema_sha256 != *binding.schema_sha256() {
        return Err(OwnerModuleSettingsRouteErrorV1::Conflict);
    }
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Conflict)?;
    if schema.major != binding.schema_major() || schema.revision != binding.schema_revision() {
        return Err(OwnerModuleSettingsRouteErrorV1::Conflict);
    }
    let (revision, snapshot_bytes) = store
        .desired_settings_snapshot_for_target(
            &export.registration_id,
            &export.configuration_instance_id,
        )
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
        .ok_or(OwnerModuleSettingsRouteErrorV1::NotFound)?;
    let snapshot = decode_settings_snapshot_v1(&snapshot_bytes)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Conflict)?;
    if revision != target.effective_revision()
        || snapshot.target_id != export.configuration_instance_id
        || snapshot.revision != target.effective_revision()
    {
        return Err(OwnerModuleSettingsRouteErrorV1::Conflict);
    }
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Conflict)?;
    let values = visible_public_values(&schema, snapshot.values)?;
    Ok(CommitOwnerModuleSettingsResponseV1 {
        major: 1,
        operation_id,
        result: Some(commit_owner_module_settings_response_v1::Result::Exported(
            ExportEffectiveOwnerModuleSettingsReceiptV1 {
                registration_id: export.registration_id,
                schema_major: binding.schema_major(),
                schema_revision: binding.schema_revision(),
                effective_revision: target.effective_revision(),
                values,
                configuration_instance_id: export.configuration_instance_id,
            },
        )),
    })
}
