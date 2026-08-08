//! Capability-fenced publication of managed owner events into the shared Gateway SSE source.

use std::sync::Arc;

use makosh_gateway_protocol::v1::{
    ClientRealtimeEventV1, ClientRealtimeFrameV1, client_realtime_frame_v1::Frame,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::ModuleClientRealtimeRouteV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{
        ContractReferenceV1, ManagedRuntimeClientRealtimePublishRequestV1,
        ManagedRuntimeClientRealtimePublishResponseV1,
    },
    validation::client_realtime::validate_managed_client_realtime_publish_request_v1,
};

use crate::runtime::lifecycle::{
    control::{ManagedRuntimeClientRealtimeHandler, ManagedRuntimeExpectation},
    fence::current_managed_runtime_matches,
};

pub(crate) struct ClientRealtimePublishHandlerV1 {
    store: Arc<SqliteControlStore>,
    source: InMemoryBrowserRealtimeSource,
}

impl ClientRealtimePublishHandlerV1 {
    #[must_use]
    pub(crate) fn new(
        store: Arc<SqliteControlStore>,
        source: InMemoryBrowserRealtimeSource,
    ) -> Self {
        Self { store, source }
    }
}

impl ManagedRuntimeClientRealtimeHandler for ClientRealtimePublishHandlerV1 {
    fn publish_client_realtime(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeClientRealtimePublishRequestV1,
    ) -> Result<ManagedRuntimeClientRealtimePublishResponseV1, String> {
        validate_managed_client_realtime_publish_request_v1(&request)
            .map_err(|_| "managed ClientRealtime publication is invalid".to_owned())?;
        ensure_current_runtime(&self.store, expectation)?;
        ensure_logical_owner(&self.store, &request.logical_owner_id)?;
        let route = exact_granted_route(&self.store, expectation, request.contract.as_ref())?;
        let contract = request
            .contract
            .as_ref()
            .ok_or_else(|| "managed ClientRealtime contract is invalid".to_owned())?;
        if route.owner() != contract.owner
            || route.contract_name() != contract.name
            || route.contract_major() != contract.major
            || route.contract_revision() != contract.revision
            || route.contract_schema_sha256().as_slice() != contract.schema_sha256
        {
            return Err("managed ClientRealtime contract is prohibited".to_owned());
        }
        let cursor = request.cursor.clone();
        let frame = ClientRealtimeFrameV1 {
            frame: Some(Frame::Event(ClientRealtimeEventV1 {
                event_id: request.event_id,
                cursor: request.cursor,
                contract_name: contract.name.clone(),
                contract_version: contract.major,
                event_kind: request.event_kind,
                occurred_at_unix_millis: request.occurred_at_unix_millis,
                causation_id: request.causation_id,
                correlation_id: request.correlation_id,
                trace_id: request.trace_id,
                payload: request.payload,
            })),
        };
        self.source
            .admit_owner(request.logical_owner_id)?
            .publish(frame)?;
        ensure_current_runtime(&self.store, expectation)?;
        Ok(ManagedRuntimeClientRealtimePublishResponseV1 {
            accepted_cursor: cursor,
        })
    }
}

fn ensure_current_runtime(
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
    .map_err(|_| "managed ClientRealtime publisher is unavailable".to_owned())?
    .then_some(())
    .ok_or_else(|| "managed ClientRealtime publisher fence is stale".to_owned())
}

fn ensure_logical_owner(store: &SqliteControlStore, requested_owner: &str) -> Result<(), String> {
    let owner = store
        .initial_owner_identity()
        .map_err(|_| "managed ClientRealtime logical owner is unavailable".to_owned())?
        .ok_or_else(|| "managed ClientRealtime logical owner is unavailable".to_owned())?;
    (owner.owner_id() == requested_owner)
        .then_some(())
        .ok_or_else(|| "managed ClientRealtime logical owner is prohibited".to_owned())
}

fn exact_granted_route(
    store: &SqliteControlStore,
    expectation: &ManagedRuntimeExpectation,
    contract: Option<&ContractReferenceV1>,
) -> Result<ModuleClientRealtimeRouteV1, String> {
    let contract =
        contract.ok_or_else(|| "managed ClientRealtime contract is invalid".to_owned())?;
    let grants = store
        .module_grant_snapshot(expectation.registration_id())
        .map_err(|_| "managed ClientRealtime grants are unavailable".to_owned())?
        .and_then(|snapshot| snapshot.effective_grants().cloned())
        .ok_or_else(|| "managed ClientRealtime publisher is not approved".to_owned())?;
    let mut matches = store
        .approved_module_client_realtime_routes()
        .map_err(|_| "managed ClientRealtime routes are unavailable".to_owned())?
        .into_iter()
        .filter(|route| {
            route.registration_id() == expectation.registration_id()
                && grants
                    .capability_ids()
                    .binary_search_by(|candidate| candidate.as_str().cmp(route.capability_id()))
                    .is_ok()
                && route.owner() == contract.owner
                && route.contract_name() == contract.name
                && route.contract_major() == contract.major
                && route.contract_revision() == contract.revision
                && route.contract_schema_sha256().as_slice() == contract.schema_sha256
        });
    let route = matches
        .next()
        .ok_or_else(|| "managed ClientRealtime contract is prohibited".to_owned())?;
    if matches.next().is_some() {
        return Err("managed ClientRealtime contract route is ambiguous".to_owned());
    }
    Ok(route)
}
