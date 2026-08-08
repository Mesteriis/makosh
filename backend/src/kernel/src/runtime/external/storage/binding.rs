//! Resolves one external runtime's current Storage configuration without secrets.

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use prost::Message;

use crate::platform::storage::topology;
use crate::platform::vault::status as vault_status;
use crate::runtime::external::sessions::ExternalRuntimeSessions;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

pub(crate) struct ExternalStorageBindingV1 {
    storage_binding_v1: Vec<u8>,
    pgbouncer_host: String,
    pgbouncer_port: u32,
    vault_instance_id: String,
    vault_runtime_generation: u64,
    vault_hpke_public_key_x25519: Vec<u8>,
}

impl ExternalStorageBindingV1 {
    pub(crate) fn storage_binding_v1(&self) -> &[u8] {
        &self.storage_binding_v1
    }

    pub(crate) fn pgbouncer_host(&self) -> &str {
        &self.pgbouncer_host
    }

    pub(crate) const fn pgbouncer_port(&self) -> u32 {
        self.pgbouncer_port
    }

    pub(crate) fn vault_instance_id(&self) -> &str {
        &self.vault_instance_id
    }

    pub(crate) const fn vault_runtime_generation(&self) -> u64 {
        self.vault_runtime_generation
    }

    pub(crate) fn vault_hpke_public_key_x25519(&self) -> &[u8] {
        &self.vault_hpke_public_key_x25519
    }
}

pub(crate) fn current_binding(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut ExternalRuntimeSessions,
    session_id: &str,
    capability_id: &str,
) -> Result<ExternalStorageBindingV1, String> {
    let runtime = sessions.authorize_storage_binding(store, session_id, capability_id)?;
    let binding = store
        .platform_storage_binding(runtime.registration_id(), capability_id)
        .map_err(|_| "Storage binding is unavailable".to_owned())?
        .ok_or_else(|| "Storage binding is unavailable".to_owned())?;
    validate_binding(&binding, &runtime)?;
    let topology = topology::current(store)?;
    let runtime_topology = topology::to_runtime(&topology)?;
    let storage_binding = topology::to_runtime_binding(&runtime_topology, &binding)?;
    let vault = vault_status::read_current(store, &supervisor.relay_port())?;
    Ok(ExternalStorageBindingV1 {
        storage_binding_v1: storage_binding.encode_to_vec(),
        pgbouncer_host: runtime_topology.pgbouncer_host,
        pgbouncer_port: runtime_topology.pgbouncer_port,
        vault_instance_id: store.snapshot().instance_id().to_owned(),
        vault_runtime_generation: vault.runtime_generation(),
        vault_hpke_public_key_x25519: vault.hpke_public_key_x25519().to_vec(),
    })
}

fn validate_binding(
    binding: &makosh_kernel_control_store::PlatformStorageBindingV1,
    runtime: &crate::runtime::external::sessions::AuthorizedExternalRuntimeV1,
) -> Result<(), String> {
    (binding.state() == PlatformStorageBindingStateV1::Active
        && binding.runtime_instance_id() == runtime.runtime_id()
        && binding.runtime_generation() == runtime.runtime_generation()
        && binding.grant_epoch() == runtime.grant_epoch())
    .then_some(())
    .ok_or_else(|| "Storage binding is stale or unauthorized".to_owned())
}
