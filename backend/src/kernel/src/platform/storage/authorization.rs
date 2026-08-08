//! Fenced authorization facts that Storage Control turns into a binding.

use makosh_kernel_control_store::{ModuleRegistryStore, ModuleStorageRequestV1, RuntimeTrustStore};
use makosh_kernel_control_store_sqlite::StoreError;

use crate::modules::capability::router::{self, ExternalCapabilityRouteRequest};

/// Credential-free authorization for one exact descriptor-declared Storage request.
pub(crate) struct StorageBindingAuthorizationV1 {
    registration_id: String,
    capability_id: String,
    owner_id: String,
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    connection_budget: u16,
    statement_timeout_millis: u32,
}

impl StorageBindingAuthorizationV1 {
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }

    #[must_use]
    pub const fn connection_budget(&self) -> u16 {
        self.connection_budget
    }

    #[must_use]
    pub const fn statement_timeout_millis(&self) -> u32 {
        self.statement_timeout_millis
    }
}

pub(crate) fn authorize_binding<S>(
    store: &S,
    registration_id: &str,
    runtime_id: &str,
    runtime_generation: u64,
    capability_id: &str,
) -> Result<StorageBindingAuthorizationV1, String>
where
    S: ModuleRegistryStore<Error = StoreError> + RuntimeTrustStore<Error = StoreError>,
{
    let request = read_exact_request(store, registration_id, capability_id)?;
    let route = ExternalCapabilityRouteRequest::new(
        registration_id,
        runtime_id,
        runtime_generation,
        capability_id,
    );
    let authorization = router::authorize_external_route(store, &route)?;
    Ok(StorageBindingAuthorizationV1 {
        registration_id: registration_id.to_owned(),
        capability_id: capability_id.to_owned(),
        owner_id: request.owner_id().to_owned(),
        runtime_id: runtime_id.to_owned(),
        runtime_generation,
        grant_epoch: authorization.grant_epoch(),
        connection_budget: request.connection_budget(),
        statement_timeout_millis: request.statement_timeout_millis(),
    })
}

pub(crate) fn authorize_managed_binding<S>(
    store: &S,
    registration_id: &str,
    runtime_instance_id: &str,
    runtime_generation: u64,
    capability_id: &str,
) -> Result<StorageBindingAuthorizationV1, String>
where
    S: ModuleRegistryStore<Error = StoreError> + RuntimeTrustStore<Error = StoreError>,
{
    let request = read_exact_request(store, registration_id, capability_id)?;
    let snapshot = store
        .module_grant_snapshot(registration_id)
        .map_err(|_| "module registration is unavailable".to_owned())?
        .ok_or_else(|| "module registration is not approved".to_owned())?;
    let grants = snapshot
        .effective_grants()
        .ok_or_else(|| "module registration is not approved".to_owned())?;
    if grants
        .capability_ids()
        .binary_search_by(|item| item.as_str().cmp(capability_id))
        .is_err()
    {
        return Err("capability is not granted to this registration".to_owned());
    }
    let launch = store
        .effective_managed_launch_record(registration_id)
        .map_err(|_| "managed runtime is unavailable".to_owned())?
        .ok_or_else(|| "managed runtime requires a current launch".to_owned())?;
    if launch.runtime_instance_id() != runtime_instance_id
        || launch.runtime_generation() != runtime_generation
        || launch.grant_epoch() != grants.grant_epoch()
    {
        return Err("managed runtime launch is stale".to_owned());
    }
    Ok(StorageBindingAuthorizationV1 {
        registration_id: registration_id.to_owned(),
        capability_id: capability_id.to_owned(),
        owner_id: request.owner_id().to_owned(),
        runtime_id: runtime_instance_id.to_owned(),
        runtime_generation,
        grant_epoch: grants.grant_epoch(),
        connection_budget: request.connection_budget(),
        statement_timeout_millis: request.statement_timeout_millis(),
    })
}

fn read_exact_request<S>(
    store: &S,
    registration_id: &str,
    capability_id: &str,
) -> Result<ModuleStorageRequestV1, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    store
        .module_storage_request(registration_id, capability_id)
        .map_err(|_| "Storage request is unavailable".to_owned())?
        .ok_or_else(|| "capability did not declare a Storage request".to_owned())
}
