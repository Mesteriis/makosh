//! Authorizes an external runtime capability route against current fenced state.

use makosh_kernel_control_store::{ModuleRegistryStore, RuntimeTrustStore};
use makosh_kernel_control_store_sqlite::StoreError;
use makosh_runtime_protocol::{
    v1::{
        ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeControlRequestV1,
        ManagedRuntimeControlResponseV1, ModuleClientRequestV1,
    },
    validation::module_client::{
        validate_module_client_request_v1, validate_module_client_response_v1,
    },
};
use prost::Message;

use crate::modules::capability::policy::permits_external_route;
use crate::runtime::lifecycle::fence::current_managed_runtime_matches;

/// Delivers a capability request to the exact fenced managed runtime.
pub trait ManagedRuntimeRelay: Send + Sync {
    fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String>;

    fn relay_with_timeout(
        &self,
        registration_id: &str,
        payload: Vec<u8>,
        _timeout: std::time::Duration,
    ) -> Result<Vec<u8>, String> {
        self.relay(registration_id, payload)
    }
}

pub struct ExternalCapabilityRouteRequest<'a> {
    registration_id: &'a str,
    runtime_id: &'a str,
    runtime_generation: u64,
    capability_id: &'a str,
}

impl<'a> ExternalCapabilityRouteRequest<'a> {
    #[must_use]
    pub fn new(
        registration_id: &'a str,
        runtime_id: &'a str,
        runtime_generation: u64,
        capability_id: &'a str,
    ) -> Self {
        Self {
            registration_id,
            runtime_id,
            runtime_generation,
            capability_id,
        }
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        self.registration_id
    }

    #[must_use]
    pub fn runtime_id(&self) -> &str {
        self.runtime_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
}

pub struct AuthorizedExternalCapabilityRoute {
    grant_epoch: u64,
}

impl AuthorizedExternalCapabilityRoute {
    #[must_use]
    pub fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

pub fn authorize_external_route<S>(
    store: &S,
    request: &ExternalCapabilityRouteRequest<'_>,
) -> Result<AuthorizedExternalCapabilityRoute, String>
where
    S: ModuleRegistryStore<Error = StoreError> + RuntimeTrustStore<Error = StoreError>,
{
    if !permits_external_route(request.capability_id) {
        return Err("capability route is prohibited by Kernel policy".to_owned());
    }
    let snapshot = store
        .module_grant_snapshot(request.registration_id)
        .map_err(|_| "module registration is unavailable".to_owned())?
        .ok_or_else(|| "module registration is not approved".to_owned())?;
    let grants = snapshot
        .effective_grants()
        .ok_or_else(|| "module registration is not approved".to_owned())?;
    if grants
        .capability_ids()
        .binary_search_by(|candidate| candidate.as_str().cmp(request.capability_id))
        .is_err()
    {
        return Err("capability is not granted to this registration".to_owned());
    }
    let runtime = store
        .effective_external_runtime_attestation(request.registration_id)
        .map_err(|_| "external runtime is unavailable".to_owned())?
        .ok_or_else(|| "external runtime requires a current attestation".to_owned())?;
    if runtime.runtime_id() != request.runtime_id
        || runtime.runtime_generation() != request.runtime_generation
        || runtime.grant_epoch() != grants.grant_epoch()
    {
        return Err("external runtime attestation is stale".to_owned());
    }
    Ok(AuthorizedExternalCapabilityRoute {
        grant_epoch: grants.grant_epoch(),
    })
}

/// Current managed-runtime fence supplied by the Core delivery caller.
pub struct ManagedCapabilityRouteRequest<'a> {
    registration_id: &'a str,
    runtime_instance_id: &'a str,
    runtime_generation: u64,
    grant_epoch: u64,
    capability_id: &'a str,
    request_bytes: &'a [u8],
}

impl<'a> ManagedCapabilityRouteRequest<'a> {
    #[must_use]
    pub fn new(
        registration_id: &'a str,
        runtime_instance_id: &'a str,
        runtime_generation: u64,
        grant_epoch: u64,
        capability_id: &'a str,
        request_bytes: &'a [u8],
    ) -> Self {
        Self {
            registration_id,
            runtime_instance_id,
            runtime_generation,
            grant_epoch,
            capability_id,
            request_bytes,
        }
    }
}

/// Delivers one opaque client request only to the exact current managed runtime.
pub fn route_managed_client_request<S, R>(
    store: &S,
    relay: &R,
    route: &ManagedCapabilityRouteRequest<'_>,
) -> Result<Vec<u8>, String>
where
    S: ModuleRegistryStore<Error = StoreError> + RuntimeTrustStore<Error = StoreError>,
    R: ManagedRuntimeRelay,
{
    if !permits_external_route(route.capability_id) {
        return Err("capability route is prohibited by Kernel policy".to_owned());
    }
    let request = ModuleClientRequestV1::decode(route.request_bytes)
        .map_err(|_| "managed client request is invalid".to_owned())?;
    validate_module_client_request_v1(&request)
        .map_err(|_| "managed client request is invalid".to_owned())?;
    let snapshot = store
        .module_grant_snapshot(route.registration_id)
        .map_err(|_| "module registration is unavailable".to_owned())?
        .ok_or_else(|| "module registration is not approved".to_owned())?;
    let grants = snapshot
        .effective_grants()
        .ok_or_else(|| "module registration is not approved".to_owned())?;
    if grants
        .capability_ids()
        .binary_search_by(|candidate| candidate.as_str().cmp(route.capability_id))
        .is_err()
    {
        return Err("capability is not granted to this registration".to_owned());
    }
    if route.grant_epoch != grants.grant_epoch()
        || !current_managed_runtime_matches(
            store,
            route.registration_id,
            route.runtime_instance_id,
            route.runtime_generation,
            route.grant_epoch,
        )
        .map_err(|_| "managed runtime is unavailable".to_owned())?
    {
        return Err("managed runtime fence is stale".to_owned());
    }
    let response = relay.relay(
        route.registration_id,
        ManagedRuntimeControlRequestV1 {
            operation: Some(
                makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::ClientDelivery(
                    ManagedRuntimeClientDeliveryRequestV1 {
                        request: Some(request.clone()),
                    },
                ),
            ),
        }
        .encode_to_vec(),
    )?;
    let response = ManagedRuntimeControlResponseV1::decode(response.as_slice())
        .map_err(|_| "managed client delivery response is invalid".to_owned())?
        .result
        .and_then(|result| match result {
            makosh_runtime_protocol::v1::managed_runtime_control_response_v1::Result::ClientDelivery(
                delivery,
            ) => delivery.response,
            _ => None,
        })
        .ok_or_else(|| "managed client delivery response is missing".to_owned())?;
    validate_module_client_response_v1(&response)
        .map_err(|_| "managed client response is rejected".to_owned())?;
    if response.request_id != request.request_id {
        return Err("managed client response does not match request".to_owned());
    }
    Ok(response.encode_to_vec())
}
