//! Creates one opaque Kernel Settings configuration target.

use makosh_gateway_protocol::v1::{
    CommitOwnerModuleSettingsResponseV1, CreateOwnerModuleSettingsTargetReceiptV1,
    CreateOwnerModuleSettingsTargetV1, commit_owner_module_settings_response_v1,
};
use makosh_gateway_runtime::OwnerModuleSettingsRouteErrorV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use sha2::{Digest, Sha256};

use crate::modules::settings::schema;

pub(super) fn create(
    store: &SqliteControlStore,
    operation_id: Vec<u8>,
    create: CreateOwnerModuleSettingsTargetV1,
) -> Result<CommitOwnerModuleSettingsResponseV1, OwnerModuleSettingsRouteErrorV1> {
    let operation_id_array: [u8; 16] = operation_id
        .as_slice()
        .try_into()
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::InvalidArgument)?;
    let configuration_instance_id =
        configuration_instance_id(store, &create.registration_id, &operation_id_array);
    let target = schema::materialize_configuration_target(
        store,
        &create.registration_id,
        &configuration_instance_id,
        operation_id_array,
    )
    .map_err(map_create_error)?;
    Ok(CommitOwnerModuleSettingsResponseV1 {
        major: 1,
        operation_id,
        result: Some(commit_owner_module_settings_response_v1::Result::Created(
            CreateOwnerModuleSettingsTargetReceiptV1 {
                registration_id: create.registration_id,
                configuration_instance_id,
                desired_revision: target.desired_revision(),
                apply_state: target.apply_state().as_str().to_owned(),
            },
        )),
    })
}

fn configuration_instance_id(
    store: &SqliteControlStore,
    registration_id: &str,
    operation_id: &[u8; 16],
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"makosh.settings.configuration-target.v1");
    digest.update(store.snapshot().instance_id());
    digest.update(registration_id.as_bytes());
    digest.update(operation_id);
    let digest: [u8; 32] = digest.finalize().into();
    format!(
        "cfg-{}",
        digest[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn map_create_error(error: String) -> OwnerModuleSettingsRouteErrorV1 {
    if error.contains("RevisionConflict") {
        OwnerModuleSettingsRouteErrorV1::Conflict
    } else if error.contains("does not admit") {
        OwnerModuleSettingsRouteErrorV1::InvalidArgument
    } else {
        OwnerModuleSettingsRouteErrorV1::Internal
    }
}
