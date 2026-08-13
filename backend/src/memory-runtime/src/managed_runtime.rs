use crate::{
    MemoryExecutionContextV1, MemoryExecutionErrorV1, MemorySourceV1,
    dispatch_memory_client_request_v1, process_memory_source_event_v1,
};
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePullDeliveryErrorV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
    try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_memory_api::MEMORY_OWNER_ID_V1;
use makosh_memory_persistence::{MemoryPersistenceErrorV1, MemoryPersistenceV1};
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
pub struct MemoryRuntimeAdmissionV1 {
    pub module_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(MemoryPersistenceErrorV1),
    ControlClosed,
    Unavailable,
}
pub struct MemoryManagedRuntimeV1 {
    admission: MemoryRuntimeAdmissionV1,
    control: ManagedControlChannelV2<UnixStream>,
    persistence: MemoryPersistenceV1,
    events: RuntimeJetStreamConnection,
    subscriptions: Vec<(MemorySourceV1, RuntimeSubscribePermitV1)>,
    next: usize,
    generation: u64,
}
impl MemoryManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control: UnixStream,
        descriptor: Vec<u8>,
        settings: Vec<u8>,
        admission: &MemoryRuntimeAdmissionV1,
        storage: ManagedStorageRuntimeConfigurationV1,
        event_endpoint: &str,
        event_revision: u64,
        now: i64,
    ) -> Result<Self, MemoryManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_endpoint.is_empty() || event_revision == 0 || now <= 0 {
            return Err(MemoryManagedRuntimeErrorV1::Admission);
        }
        let mut control = ManagedControlChannelV2::new(control);
        authenticate(&mut control, descriptor, settings, admission)?;
        let binding = storage_binding(&storage, admission)?;
        let public_key = storage
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?;
        let context = StorageVaultRouteContextV1::new(
            storage.vault_instance_id.clone(),
            storage.vault_runtime_generation,
            public_key,
        )
        .map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?;
        let mut leases =
            StorageVaultLeaseAdapterV1::new(InheritedKernelVaultRouteV2::new(control), context);
        let password = resolve(&mut leases, &binding).await?;
        let password =
            std::str::from_utf8(&password).map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?;
        let persistence = MemoryPersistenceV1::connect_runtime(
            &binding,
            &storage.database_id,
            &storage.pgbouncer_host,
            storage.pgbouncer_port,
            password,
        )
        .await
        .map_err(MemoryManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(MemoryManagedRuntimeErrorV1::Persistence)?;
        let generation = persistence
            .ensure_live_generation(&admission.logical_human_owner_id, now)
            .await
            .map_err(MemoryManagedRuntimeErrorV1::Persistence)?;
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
        .map_err(|_| MemoryManagedRuntimeErrorV1::EventUnavailable)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?;
        let subscriptions = bind(
            access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?,
        )?;
        let events = JetStreamClient::connect_runtime_with_jwt(
            event_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| MemoryManagedRuntimeErrorV1::EventUnavailable)?;
        for (_, permit) in &subscriptions {
            events
                .open_pull_consumer(permit)
                .await
                .map_err(|_| MemoryManagedRuntimeErrorV1::EventContract)?;
        }
        signal_ready(&mut control, admission)?;
        control
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| MemoryManagedRuntimeErrorV1::Unavailable)?;
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
    pub async fn service_once(&mut self, now: i64) -> Result<bool, MemoryManagedRuntimeErrorV1> {
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
                .map_err(|_| MemoryManagedRuntimeErrorV1::EventContract)?;
            let context = MemoryExecutionContextV1 {
                logical_owner_id: self.admission.logical_human_owner_id.clone(),
                projection_generation: self.generation,
                now_unix_millis: now,
            };
            match process_memory_source_event_v1(&self.persistence, &record, *source, &context)
                .await
            {
                Ok(_) => {}
                Err(error) if bounded(error) => {}
                Err(error) => return Err(execution_error(error)),
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
    ) -> Result<bool, MemoryManagedRuntimeErrorV1> {
        let deadline = tokio::time::Instant::now() + delay;
        loop {
            match self.pump().await {
                Ok(_) => {}
                Err(MemoryManagedRuntimeErrorV1::ControlClosed) => return Ok(false),
                Err(error) => return Err(error),
            }
            if tokio::time::Instant::now() >= deadline {
                return Ok(true);
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await
        }
    }
    async fn pump(&mut self) -> Result<bool, MemoryManagedRuntimeErrorV1> {
        let Some((id, request)) = self.control.try_receive_request().map_err(control_error)? else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            self.write_error(id, "managed_runtime_control_unexpected_request")?;
            return Ok(true);
        };
        let Some(request) = delivery
            .request
            .filter(|value| validate_module_client_request_v1(value).is_ok())
        else {
            self.write_error(id, "managed_runtime_control_invalid_client_delivery")?;
            return Ok(true);
        };
        let response = dispatch_memory_client_request_v1(
            &self.persistence,
            &self.admission.logical_human_owner_id,
            request,
        )
        .await;
        validate_module_client_response_v1(&response)
            .map_err(|_| MemoryManagedRuntimeErrorV1::Unavailable)?;
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
    fn write_error(&mut self, id: [u8; 16], code: &str) -> Result<(), MemoryManagedRuntimeErrorV1> {
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
) -> Result<Vec<(MemorySourceV1, RuntimeSubscribePermitV1)>, MemoryManagedRuntimeErrorV1> {
    let mut result = Vec::new();
    for permit in permits {
        let source = source_for(
            permit
                .contract()
                .ok_or(MemoryManagedRuntimeErrorV1::Admission)?,
        )
        .ok_or(MemoryManagedRuntimeErrorV1::Admission)?;
        if result.iter().any(|(v, _)| *v == source) {
            return Err(MemoryManagedRuntimeErrorV1::Admission);
        }
        result.push((source, permit));
    }
    if result.len() != 2 {
        return Err(MemoryManagedRuntimeErrorV1::Admission);
    }
    result.sort_by_key(|(v, _)| *v as u8);
    Ok(result)
}
fn source_for(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> Option<MemorySourceV1> {
    use makosh_decisions_api::decisions_lifecycle_event_contract_reference_v1;
    use makosh_knowledge_command_api::knowledge_lifecycle_event_contract_reference_v1;
    [
        (
            MemorySourceV1::Decisions,
            decisions_lifecycle_event_contract_reference_v1(),
        ),
        (
            MemorySourceV1::Knowledge,
            knowledge_lifecycle_event_contract_reference_v1(),
        ),
    ]
    .into_iter()
    .find_map(|(source, expected)| (contract == &expected).then_some(source))
}
fn validate_admission(value: &MemoryRuntimeAdmissionV1) -> Result<(), MemoryManagedRuntimeErrorV1> {
    if value.module_owner_id != MEMORY_OWNER_ID_V1
        || value.logical_human_owner_id.is_empty()
        || value.registration_id.is_empty()
        || value.runtime_instance_id.is_empty()
        || value.runtime_generation == 0
        || value.grant_epoch == 0
    {
        Err(MemoryManagedRuntimeErrorV1::Admission)
    } else {
        Ok(())
    }
}
fn authenticate(
    control: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &MemoryRuntimeAdmissionV1,
) -> Result<(), MemoryManagedRuntimeErrorV1> {
    control
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            control
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| MemoryManagedRuntimeErrorV1::Unavailable)?;
    let response = control
        .describe_managed_runtime(descriptor, settings)
        .map_err(control_error)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(MemoryManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}
fn signal_ready(
    control: &mut ManagedControlChannelV2<UnixStream>,
    admission: &MemoryRuntimeAdmissionV1,
) -> Result<(), MemoryManagedRuntimeErrorV1> {
    control
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(control_error)?;
    control
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control.inner_mut().set_write_timeout(None))
        .map_err(|_| MemoryManagedRuntimeErrorV1::Unavailable)
}
async fn resolve(
    adapter: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, MemoryManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(value) = adapter.ensure_runtime_credential(binding).await {
            return Ok(value);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await
        }
    }
    Err(MemoryManagedRuntimeErrorV1::Unavailable)
}
fn storage_binding(
    c: &ManagedStorageRuntimeConfigurationV1,
    a: &MemoryRuntimeAdmissionV1,
) -> Result<StorageBindingV1, MemoryManagedRuntimeErrorV1> {
    if c.runtime_instance_id != a.runtime_instance_id
        || c.logical_owner_id != MEMORY_OWNER_ID_V1
        || c.owner != MEMORY_OWNER_ID_V1
        || c.storage_bundle_digest.len() != 32
        || c.storage_generation == 0
        || c.credential_revision == 0
        || c.role_epoch == 0
        || c.storage_bundle_revision == 0
    {
        return Err(MemoryManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        c.storage_instance_id.clone(),
        c.database_id.clone(),
        c.owner.clone(),
        a.registration_id.clone(),
        c.runtime_instance_id.clone(),
    )
    .map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        c.storage_generation,
        a.runtime_generation,
        a.grant_epoch,
        c.role_epoch,
        c.credential_revision,
        c.storage_bundle_revision,
    )
    .map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(c.max_connections).map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?,
        c.statement_timeout_millis,
    )
    .map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        c.runtime_principal.clone(),
        c.pool_alias.clone(),
        budgets,
        c.storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| MemoryManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| MemoryManagedRuntimeErrorV1::Admission)
}
const fn bounded(e: MemoryExecutionErrorV1) -> bool {
    matches!(
        e,
        MemoryExecutionErrorV1::InvalidEnvelope
            | MemoryExecutionErrorV1::InvalidPayload
            | MemoryExecutionErrorV1::Persistence(
                MemoryPersistenceErrorV1::InvalidInput
                    | MemoryPersistenceErrorV1::Conflict
                    | MemoryPersistenceErrorV1::RevisionConflict
            )
    )
}
const fn execution_error(e: MemoryExecutionErrorV1) -> MemoryManagedRuntimeErrorV1 {
    match e {
        MemoryExecutionErrorV1::Persistence(v) => MemoryManagedRuntimeErrorV1::Persistence(v),
        _ => MemoryManagedRuntimeErrorV1::EventContract,
    }
}
const fn event_error(_: RuntimePullDeliveryErrorV1) -> MemoryManagedRuntimeErrorV1 {
    MemoryManagedRuntimeErrorV1::EventUnavailable
}
fn control_error(e: ManagedControlTransportErrorV2) -> MemoryManagedRuntimeErrorV1 {
    if matches!(e, ManagedControlTransportErrorV2::PeerClosed) {
        MemoryManagedRuntimeErrorV1::ControlClosed
    } else {
        MemoryManagedRuntimeErrorV1::Unavailable
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_sources_are_bound() {
        assert_eq!(
            source_for(&makosh_decisions_api::decisions_lifecycle_event_contract_reference_v1()),
            Some(MemorySourceV1::Decisions)
        );
        assert_eq!(
            source_for(
                &makosh_knowledge_command_api::knowledge_lifecycle_event_contract_reference_v1()
            ),
            Some(MemorySourceV1::Knowledge)
        );
    }
}
