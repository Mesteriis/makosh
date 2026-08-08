//! Owner-authorized lifecycle commands for the managed Blob platform service.

use std::path::Path;

use makosh_gateway_protocol::v1::{
    BindPlatformBlobReleaseRequestV1, BindPlatformBlobReleaseResponseV1,
    StartPlatformBlobRuntimeRequestV1, StartPlatformBlobRuntimeResponseV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use super::{OwnerControlSessions, OwnerResult, stop_if_active};
use crate::platform::blob::binding;
use crate::platform::blob::launch;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

pub(super) fn bind_release(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: BindPlatformBlobReleaseRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let binding = binding::bind_current_installed_release(store)?;
        stop_if_active(supervisor, binding::BLOB_PROCESS_ID)?;
        Ok(binding)
    })()
    .map(|binding| {
        OwnerResult::BindPlatformBlobRelease(BindPlatformBlobReleaseResponseV1 {
            process_id: binding.process_id().to_owned(),
            binding_revision: binding.binding_revision(),
            distribution_id: binding.distribution_id().to_owned(),
            artifact_id: binding.artifact_id().to_owned(),
        })
    })
}

pub(super) fn start(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartPlatformBlobRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        launch::start(supervisor, store, data_dir, runtime_dir)
    })()
    .map(|runtime_generation| {
        OwnerResult::StartPlatformBlobRuntime(StartPlatformBlobRuntimeResponseV1 {
            process_id: binding::BLOB_PROCESS_ID.to_owned(),
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}
