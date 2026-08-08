//! Owner-authorized reservation of an external Storage binding replacement.

use makosh_gateway_protocol::v1::{
    BeginExternalStorageBindingRevocationRequestV1, BeginExternalStorageBindingRevocationResponseV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use super::{OwnerControlSessions, OwnerResult};

pub(super) fn begin(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: BeginExternalStorageBindingRevocationRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let current = store
            .platform_storage_binding(&request.registration_id, &request.capability_id)
            .map_err(|_| "Storage binding is unavailable".to_owned())?
            .ok_or_else(|| "Storage binding is unavailable".to_owned())?;
        if current.runtime_instance_id() != request.runtime_instance_id
            || current.runtime_generation() != request.runtime_generation
        {
            return Err("External Storage binding does not match runtime".to_owned());
        }
        store
            .begin_platform_storage_binding_revocation(
                &request.registration_id,
                &request.capability_id,
                request.binding_revision,
            )
            .map_err(|_| "Storage binding cannot be reserved for revocation".to_owned())
    })()
    .map(|binding| {
        OwnerResult::BeginExternalStorageBindingRevocation(
            BeginExternalStorageBindingRevocationResponseV1 {
                registration_id: binding.registration_id().to_owned(),
                capability_id: binding.capability_id().to_owned(),
                binding_revision: binding.binding_revision(),
            },
        )
    })
}
