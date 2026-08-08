//! Authorizes and relays one exact descriptor-declared managed module request.

use std::sync::Arc;
use std::time::Duration;

use makosh_kernel_control_store::ModuleRequestContractV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{
        ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeModuleRequestDeliveryV1, ManagedRuntimeModuleRequestRequestV1,
        ManagedRuntimeModuleRequestResponseV1, managed_runtime_control_request_v1,
        managed_runtime_control_response_v1,
    },
    validation::module_request::{
        validate_module_request_delivery_v1, validate_module_request_request_v1,
        validate_module_request_response_v1,
    },
};
use prost::Message;

use crate::runtime::lifecycle::{
    control::{ManagedRuntimeExpectation, ManagedRuntimeModuleRequestHandler},
    fence::current_managed_runtime_matches,
    supervisor::ManagedRuntimeRelay,
};

pub(crate) struct ModuleRequestRouteHandlerV1<R> {
    store: Arc<SqliteControlStore>,
    relay: R,
}

pub(crate) struct ResolvedModuleRequestProviderV1 {
    pub(crate) route: ModuleRequestContractV1,
    pub(crate) registration: makosh_kernel_control_store::ModuleRegistration,
    pub(crate) launch: makosh_kernel_control_store::ManagedLaunchRecord,
}

impl<R> ModuleRequestRouteHandlerV1<R>
where
    R: ManagedRuntimeRelay,
{
    pub(crate) fn new(store: Arc<SqliteControlStore>, relay: R) -> Self {
        Self { store, relay }
    }
}

impl<R> ManagedRuntimeModuleRequestHandler for ModuleRequestRouteHandlerV1<R>
where
    R: ManagedRuntimeRelay,
{
    fn route_module_request(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeModuleRequestRequestV1,
    ) -> Result<ManagedRuntimeModuleRequestResponseV1, String> {
        validate_module_request_request_v1(&request)
            .map_err(|_| "managed module request is denied".to_owned())?;
        let logical_owner = self
            .store
            .initial_owner_identity()
            .map_err(|_| "managed module request logical owner is unavailable".to_owned())?
            .ok_or_else(|| "managed module request logical owner is unavailable".to_owned())?;
        let contract = request
            .contract
            .as_ref()
            .ok_or_else(|| "managed module request contract is missing".to_owned())?;
        let resolved = resolve_provider_for_caller(&self.store, expectation, contract)?;
        let provider = resolved.route;
        let provider_launch = resolved.launch;
        let response_target = resolve_response_blob_target(
            &self.store,
            expectation,
            &request.response_blob_capability_id,
        )?;

        let delivery = ManagedRuntimeModuleRequestDeliveryV1 {
            request_id: request.request_id.clone(),
            logical_owner_id: logical_owner.owner_id().to_owned(),
            contract: request.contract.clone(),
            request_payload: request.request_payload,
            response_blob_target_owner_id: response_target.owner_id,
            response_blob_target_module_id: response_target.module_id,
            response_blob_target_capability_id: response_target.capability_id,
        };
        validate_module_request_delivery_v1(&delivery)
            .map_err(|_| "managed module request delivery is denied".to_owned())?;
        let response_bytes = ManagedRuntimeRelay::relay_with_timeout(
            &self.relay,
            provider.registration_id(),
            ManagedRuntimeControlRequestV1 {
                operation: Some(
                    managed_runtime_control_request_v1::Operation::DeliverModuleRequest(delivery),
                ),
            }
            .encode_to_vec(),
            Duration::from_millis(u64::from(request.deadline_millis)),
        )?;
        let response = ManagedRuntimeControlResponseV1::decode(response_bytes.as_slice())
            .map_err(|_| "managed module request provider response is invalid".to_owned())?
            .result
            .and_then(|result| match result {
                managed_runtime_control_response_v1::Result::ModuleRequestDelivery(response) => {
                    Some(response)
                }
                _ => None,
            })
            .ok_or_else(|| "managed module request provider response is missing".to_owned())?;
        validate_module_request_response_v1(&response)
            .map_err(|_| "managed module request provider response is rejected".to_owned())?;
        if response.request_id != request.request_id {
            return Err(
                "managed module request provider response does not match request".to_owned(),
            );
        }

        ensure_caller_fence(&self.store, expectation)?;
        ensure_provider_fence(&self.store, &provider, &provider_launch)?;
        Ok(response)
    }
}

struct ModuleRequestResponseBlobTargetV1 {
    owner_id: String,
    module_id: String,
    capability_id: String,
}

fn resolve_response_blob_target(
    store: &SqliteControlStore,
    expectation: &ManagedRuntimeExpectation,
    capability_id: &str,
) -> Result<ModuleRequestResponseBlobTargetV1, String> {
    if capability_id.is_empty() {
        return Ok(ModuleRequestResponseBlobTargetV1 {
            owner_id: String::new(),
            module_id: String::new(),
            capability_id: String::new(),
        });
    }
    let entry = crate::platform::blob::catalog::resolve(store)?
        .into_iter()
        .find(|entry| {
            entry.registration_id() == expectation.registration_id()
                && entry.module_id() == expectation.module_id()
                && entry.grant_epoch() == expectation.grant_epoch()
                && entry.capability_id() == capability_id
        })
        .ok_or_else(|| "managed module request response Blob target is denied".to_owned())?;
    Ok(ModuleRequestResponseBlobTargetV1 {
        owner_id: entry.request().owner_id().to_owned(),
        module_id: entry.module_id().to_owned(),
        capability_id: entry.capability_id().to_owned(),
    })
}

pub(crate) fn resolve_provider_for_caller(
    store: &SqliteControlStore,
    expectation: &ManagedRuntimeExpectation,
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> Result<ResolvedModuleRequestProviderV1, String> {
    ensure_caller_fence(store, expectation)?;
    let caller = store
        .module_registration(expectation.registration_id())
        .map_err(|_| "managed module request caller is unavailable".to_owned())?
        .ok_or_else(|| "managed module request caller is unavailable".to_owned())?;
    if caller.grant_epoch() != expectation.grant_epoch() {
        return Err("managed module request caller fence is stale".to_owned());
    }
    let caller_grants = store
        .module_grant_snapshot(expectation.registration_id())
        .map_err(|_| "managed module request caller grants are unavailable".to_owned())?
        .and_then(|snapshot| snapshot.effective_grants().cloned())
        .ok_or_else(|| "managed module request caller is not approved".to_owned())?;
    resolve_caller_capability(
        store,
        expectation.registration_id(),
        caller_grants.capability_ids(),
        contract,
    )?;

    let route = resolve_provider(store, contract)?;
    let provider_grants = store
        .module_grant_snapshot(route.registration_id())
        .map_err(|_| "managed module request provider grants are unavailable".to_owned())?
        .and_then(|snapshot| snapshot.effective_grants().cloned())
        .ok_or_else(|| "managed module request provider is not approved".to_owned())?;
    if provider_grants
        .capability_ids()
        .binary_search_by(|candidate| candidate.as_str().cmp(route.capability_id()))
        .is_err()
    {
        return Err("managed module request provider capability is not granted".to_owned());
    }
    let launch = current_provider_launch(store, &route)?;
    let registration = store
        .module_registration(route.registration_id())
        .map_err(|_| "managed module request provider is unavailable".to_owned())?
        .ok_or_else(|| "managed module request provider is unavailable".to_owned())?;
    if registration.grant_epoch() != launch.grant_epoch() {
        return Err("managed module request provider fence is stale".to_owned());
    }
    Ok(ResolvedModuleRequestProviderV1 {
        route,
        registration,
        launch,
    })
}

fn resolve_caller_capability(
    store: &SqliteControlStore,
    registration_id: &str,
    granted_capabilities: &[String],
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> Result<(), String> {
    for capability_id in granted_capabilities {
        let dependencies = store
            .module_contract_dependencies(registration_id, capability_id)
            .map_err(|_| "managed module request dependencies are unavailable".to_owned())?;
        if dependencies
            .iter()
            .any(|dependency| exact_dependency_matches(dependency, contract))
        {
            return Ok(());
        }
    }
    Err("managed module request dependency is not granted".to_owned())
}

fn resolve_provider(
    store: &SqliteControlStore,
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> Result<ModuleRequestContractV1, String> {
    let routes = store
        .approved_module_request_rpc_routes()
        .map_err(|_| "managed module request providers are unavailable".to_owned())?;
    let mut matches = routes
        .into_iter()
        .filter(|route| exact_provider_matches(route, contract));
    let provider = matches
        .next()
        .ok_or_else(|| "managed module request provider is unavailable".to_owned())?;
    if matches.next().is_some() {
        return Err("managed module request provider is ambiguous".to_owned());
    }
    Ok(provider)
}

fn exact_dependency_matches(
    expected: &makosh_kernel_control_store::ModuleQueryContractV1,
    actual: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> bool {
    expected.owner() == actual.owner
        && expected.name() == actual.name
        && expected.major() == actual.major
        && expected.revision() == actual.revision
        && expected.schema_sha256().as_slice() == actual.schema_sha256
}

fn exact_provider_matches(
    expected: &ModuleRequestContractV1,
    actual: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> bool {
    expected.owner() == actual.owner
        && expected.name() == actual.name
        && expected.major() == actual.major
        && expected.revision() == actual.revision
        && expected.schema_sha256().as_slice() == actual.schema_sha256
}

fn ensure_caller_fence(
    store: &SqliteControlStore,
    expectation: &ManagedRuntimeExpectation,
) -> Result<(), String> {
    current_managed_runtime_matches(
        store,
        expectation.registration_id(),
        expectation.runtime_instance_id(),
        expectation.runtime_generation(),
        expectation.grant_epoch(),
    )
    .map_err(|_| "managed module request caller is unavailable".to_owned())?
    .then_some(())
    .ok_or_else(|| "managed module request caller fence is stale".to_owned())
}

fn current_provider_launch(
    store: &SqliteControlStore,
    provider: &ModuleRequestContractV1,
) -> Result<makosh_kernel_control_store::ManagedLaunchRecord, String> {
    let launch = store
        .effective_managed_launch_record(provider.registration_id())
        .map_err(|_| "managed module request provider is unavailable".to_owned())?
        .ok_or_else(|| "managed module request provider is unavailable".to_owned())?;
    ensure_provider_fence(store, provider, &launch)?;
    Ok(launch)
}

fn ensure_provider_fence(
    store: &SqliteControlStore,
    provider: &ModuleRequestContractV1,
    launch: &makosh_kernel_control_store::ManagedLaunchRecord,
) -> Result<(), String> {
    current_managed_runtime_matches(
        store,
        provider.registration_id(),
        launch.runtime_instance_id(),
        launch.runtime_generation(),
        launch.grant_epoch(),
    )
    .map_err(|_| "managed module request provider is unavailable".to_owned())?
    .then_some(())
    .ok_or_else(|| "managed module request provider fence is stale".to_owned())
}
