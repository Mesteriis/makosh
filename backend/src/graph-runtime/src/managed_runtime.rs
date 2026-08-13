use crate::{
    GraphExecutionContextV1, GraphExecutionErrorV1, GraphSourceV1,
    dispatch_graph_client_request_v1, process_graph_source_event_v1,
};
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePullDeliveryErrorV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
    try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_graph_api::GRAPH_OWNER_ID_V1;
use makosh_graph_persistence::{GraphPersistenceErrorV1, GraphPersistenceV1};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlTransportErrorV2},
    v1::{
        ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeReadyRequestV1, ManagedStorageRuntimeConfigurationV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_client::{
        validate_module_client_request_v1, validate_module_client_response_v1,
    },
};
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};
use std::os::unix::net::UnixStream;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphRuntimeAdmissionV1 {
    pub module_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(GraphPersistenceErrorV1),
    ControlClosed,
    Unavailable,
}
pub struct GraphManagedRuntimeV1 {
    admission: GraphRuntimeAdmissionV1,
    control: ManagedControlChannelV2<UnixStream>,
    persistence: GraphPersistenceV1,
    events: RuntimeJetStreamConnection,
    subscriptions: Vec<(GraphSourceV1, RuntimeSubscribePermitV1)>,
    next: usize,
    generation: u64,
}
impl GraphManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control: UnixStream,
        descriptor: Vec<u8>,
        settings: Vec<u8>,
        admission: &GraphRuntimeAdmissionV1,
        storage: ManagedStorageRuntimeConfigurationV1,
        event_endpoint: &str,
        event_revision: u64,
        now: i64,
    ) -> Result<Self, GraphManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_endpoint.is_empty() || event_revision == 0 || now <= 0 {
            return Err(GraphManagedRuntimeErrorV1::Admission);
        }
        let mut control = ManagedControlChannelV2::new(control);
        authenticate(&mut control, descriptor, settings, admission)?;
        let binding = storage_binding(&storage, admission)?;
        let key = storage
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| GraphManagedRuntimeErrorV1::Admission)?;
        let context = StorageVaultRouteContextV1::new(
            storage.vault_instance_id.clone(),
            storage.vault_runtime_generation,
            key,
        )
        .map_err(|_| GraphManagedRuntimeErrorV1::Admission)?;
        let mut leases =
            StorageVaultLeaseAdapterV1::new(InheritedKernelVaultRouteV2::new(control), context);
        let password = resolve(&mut leases, &binding).await?;
        let password =
            std::str::from_utf8(&password).map_err(|_| GraphManagedRuntimeErrorV1::Admission)?;
        let persistence = GraphPersistenceV1::connect_runtime(
            &binding,
            &storage.database_id,
            &storage.pgbouncer_host,
            storage.pgbouncer_port,
            password,
        )
        .await
        .map_err(GraphManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(GraphManagedRuntimeErrorV1::Persistence)?;
        let generation = persistence
            .ensure_live_generation(&admission.logical_human_owner_id, now)
            .await
            .map_err(GraphManagedRuntimeErrorV1::Persistence)?;
        let mut control = leases.into_route_port().into_channel();
        let access = request_managed_runtime_event_access_v2(
            &mut control,
            &storage.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            event_revision,
        )
        .map_err(|_| GraphManagedRuntimeErrorV1::EventUnavailable)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| GraphManagedRuntimeErrorV1::Admission)?;
        let subscriptions = bind(
            access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| GraphManagedRuntimeErrorV1::Admission)?,
        )?;
        let events = JetStreamClient::connect_runtime_with_jwt(
            event_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| GraphManagedRuntimeErrorV1::EventUnavailable)?;
        for (_, permit) in &subscriptions {
            events
                .open_pull_consumer(permit)
                .await
                .map_err(|_| GraphManagedRuntimeErrorV1::EventContract)?;
        }
        signal_ready(&mut control, admission)?;
        control
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| GraphManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control,
            persistence,
            events,
            subscriptions,
            next: 0,
            generation,
        })
    }
    pub async fn service_once(&mut self, now: i64) -> Result<bool, GraphManagedRuntimeErrorV1> {
        if self.pump().await? {
            return Ok(true);
        }
        let count = self.subscriptions.len();
        for offset in 0..count {
            let index = (self.next + offset) % count;
            let (source, permit) = &self.subscriptions[index];
            let Some(delivery) = try_receive_runtime_pull_delivery(&self.events, permit)
                .await
                .map_err(event_error)?
            else {
                continue;
            };
            self.next = (index + 1) % count;
            let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
                .map_err(|_| GraphManagedRuntimeErrorV1::EventContract)?;
            let context = GraphExecutionContextV1 {
                logical_owner_id: self.admission.logical_human_owner_id.clone(),
                projection_generation: self.generation,
                now_unix_millis: now,
            };
            match process_graph_source_event_v1(&self.persistence, &record, *source, &context).await
            {
                Ok(_) => {}
                Err(e) if bounded(e) => {}
                Err(e) => return Err(execution_error(e)),
            }
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        Ok(false)
    }
    pub async fn wait_retry_delay(
        &mut self,
        delay: std::time::Duration,
    ) -> Result<bool, GraphManagedRuntimeErrorV1> {
        let deadline = tokio::time::Instant::now() + delay;
        loop {
            match self.pump().await {
                Ok(_) => {}
                Err(GraphManagedRuntimeErrorV1::ControlClosed) => return Ok(false),
                Err(e) => return Err(e),
            }
            if tokio::time::Instant::now() >= deadline {
                return Ok(true);
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await
        }
    }
    async fn pump(&mut self) -> Result<bool, GraphManagedRuntimeErrorV1> {
        let Some((id, request)) = self.control.try_receive_request().map_err(control_error)? else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            self.write_error(id, "managed_runtime_control_unexpected_request")?;
            return Ok(true);
        };
        let Some(request) = delivery
            .request
            .filter(|v| validate_module_client_request_v1(v).is_ok())
        else {
            self.write_error(id, "managed_runtime_control_invalid_client_delivery")?;
            return Ok(true);
        };
        let response = dispatch_graph_client_request_v1(
            &self.persistence,
            &self.admission.logical_human_owner_id,
            request,
        )
        .await;
        validate_module_client_response_v1(&response)
            .map_err(|_| GraphManagedRuntimeErrorV1::Unavailable)?;
        self.control
            .write_response(
                id,
                ManagedRuntimeControlResponseV1 {
                    result: Some(ControlResult::ClientDelivery(
                        ManagedRuntimeClientDeliveryResponseV1 {
                            response: Some(response),
                        },
                    )),
                    error_code: String::new(),
                },
            )
            .map_err(control_error)?;
        Ok(true)
    }
    fn write_error(&mut self, id: [u8; 16], code: &str) -> Result<(), GraphManagedRuntimeErrorV1> {
        self.control
            .write_response(
                id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: code.into(),
                },
            )
            .map_err(control_error)
    }
}
fn bind(
    permits: Vec<RuntimeSubscribePermitV1>,
) -> Result<Vec<(GraphSourceV1, RuntimeSubscribePermitV1)>, GraphManagedRuntimeErrorV1> {
    let mut result = Vec::new();
    for permit in permits {
        let source = source_for(
            permit
                .contract()
                .ok_or(GraphManagedRuntimeErrorV1::Admission)?,
        )
        .ok_or(GraphManagedRuntimeErrorV1::Admission)?;
        if result.iter().any(|(v, _)| *v == source) {
            return Err(GraphManagedRuntimeErrorV1::Admission);
        }
        result.push((source, permit));
    }
    if result.len() != 2 {
        return Err(GraphManagedRuntimeErrorV1::Admission);
    }
    result.sort_by_key(|(v, _)| *v as u8);
    Ok(result)
}
fn source_for(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> Option<GraphSourceV1> {
    if contract == &makosh_persons_api::persons_owner_event_contract_reference_v1() {
        Some(GraphSourceV1::Persons)
    } else if contract
        == &makosh_relationships_api::relationships_lifecycle_event_contract_reference_v1()
    {
        Some(GraphSourceV1::Relationships)
    } else {
        None
    }
}
fn validate_admission(v: &GraphRuntimeAdmissionV1) -> Result<(), GraphManagedRuntimeErrorV1> {
    if v.module_owner_id != GRAPH_OWNER_ID_V1
        || v.logical_human_owner_id.is_empty()
        || v.registration_id.is_empty()
        || v.runtime_instance_id.is_empty()
        || v.runtime_generation == 0
        || v.grant_epoch == 0
    {
        Err(GraphManagedRuntimeErrorV1::Admission)
    } else {
        Ok(())
    }
}
fn authenticate(
    c: &mut ManagedControlChannelV2<UnixStream>,
    d: Vec<u8>,
    s: Vec<u8>,
    a: &GraphRuntimeAdmissionV1,
) -> Result<(), GraphManagedRuntimeErrorV1> {
    c.inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            c.inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| GraphManagedRuntimeErrorV1::Unavailable)?;
    let r = c.describe_managed_runtime(d, s).map_err(control_error)?;
    if r.registration_id != a.registration_id
        || r.runtime_generation != a.runtime_generation
        || r.grant_epoch != a.grant_epoch
    {
        return Err(GraphManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}
fn signal_ready(
    c: &mut ManagedControlChannelV2<UnixStream>,
    a: &GraphRuntimeAdmissionV1,
) -> Result<(), GraphManagedRuntimeErrorV1> {
    c.signal_ready(ManagedRuntimeReadyRequestV1 {
        registration_id: a.registration_id.clone(),
        runtime_generation: a.runtime_generation,
        grant_epoch: a.grant_epoch,
    })
    .map_err(control_error)?;
    c.inner_mut()
        .set_read_timeout(None)
        .and_then(|_| c.inner_mut().set_write_timeout(None))
        .map_err(|_| GraphManagedRuntimeErrorV1::Unavailable)
}
async fn resolve(
    a: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    b: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, GraphManagedRuntimeErrorV1> {
    for i in 0..20 {
        if let Ok(v) = a.ensure_runtime_credential(b).await {
            return Ok(v);
        }
        if i < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await
        }
    }
    Err(GraphManagedRuntimeErrorV1::Unavailable)
}
fn storage_binding(
    c: &ManagedStorageRuntimeConfigurationV1,
    a: &GraphRuntimeAdmissionV1,
) -> Result<StorageBindingV1, GraphManagedRuntimeErrorV1> {
    if c.runtime_instance_id != a.runtime_instance_id
        || c.logical_owner_id != GRAPH_OWNER_ID_V1
        || c.owner != GRAPH_OWNER_ID_V1
        || c.storage_bundle_digest.len() != 32
        || c.storage_generation == 0
        || c.credential_revision == 0
        || c.role_epoch == 0
        || c.storage_bundle_revision == 0
    {
        return Err(GraphManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        c.storage_instance_id.clone(),
        c.database_id.clone(),
        c.owner.clone(),
        a.registration_id.clone(),
        c.runtime_instance_id.clone(),
    )
    .map_err(|_| GraphManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        c.storage_generation,
        a.runtime_generation,
        a.grant_epoch,
        c.role_epoch,
        c.credential_revision,
        c.storage_bundle_revision,
    )
    .map_err(|_| GraphManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(c.max_connections).map_err(|_| GraphManagedRuntimeErrorV1::Admission)?,
        c.statement_timeout_millis,
    )
    .map_err(|_| GraphManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        c.runtime_principal.clone(),
        c.pool_alias.clone(),
        budgets,
        c.storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| GraphManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| GraphManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| GraphManagedRuntimeErrorV1::Admission)
}
const fn bounded(e: GraphExecutionErrorV1) -> bool {
    matches!(
        e,
        GraphExecutionErrorV1::InvalidEnvelope
            | GraphExecutionErrorV1::InvalidPayload
            | GraphExecutionErrorV1::Persistence(
                GraphPersistenceErrorV1::InvalidInput
                    | GraphPersistenceErrorV1::Conflict
                    | GraphPersistenceErrorV1::RevisionConflict
            )
    )
}
const fn execution_error(e: GraphExecutionErrorV1) -> GraphManagedRuntimeErrorV1 {
    match e {
        GraphExecutionErrorV1::Persistence(v) => GraphManagedRuntimeErrorV1::Persistence(v),
        _ => GraphManagedRuntimeErrorV1::EventContract,
    }
}
const fn event_error(_: RuntimePullDeliveryErrorV1) -> GraphManagedRuntimeErrorV1 {
    GraphManagedRuntimeErrorV1::EventUnavailable
}
fn control_error(e: ManagedControlTransportErrorV2) -> GraphManagedRuntimeErrorV1 {
    if matches!(e, ManagedControlTransportErrorV2::PeerClosed) {
        GraphManagedRuntimeErrorV1::ControlClosed
    } else {
        GraphManagedRuntimeErrorV1::Unavailable
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_two_sources() {
        assert_eq!(
            source_for(&makosh_persons_api::persons_owner_event_contract_reference_v1()),
            Some(GraphSourceV1::Persons)
        );
        assert_eq!(
            source_for(
                &makosh_relationships_api::relationships_lifecycle_event_contract_reference_v1()
            ),
            Some(GraphSourceV1::Relationships)
        );
    }
}
