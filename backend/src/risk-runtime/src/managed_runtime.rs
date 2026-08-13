use crate::{
    RiskExecutionContextV1, RiskExecutionErrorV1, RiskSourceV1, dispatch_risk_client_request_v1,
    process_risk_source_event_v1,
};
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePullDeliveryErrorV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
    try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_risk_api::RISK_OWNER_ID_V1;
use makosh_risk_persistence::{RiskPersistenceErrorV1, RiskPersistenceV1};
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
pub struct RiskRuntimeAdmissionV1 {
    pub module_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(RiskPersistenceErrorV1),
    ControlClosed,
    Unavailable,
}
pub struct RiskManagedRuntimeV1 {
    admission: RiskRuntimeAdmissionV1,
    control: ManagedControlChannelV2<UnixStream>,
    persistence: RiskPersistenceV1,
    events: RuntimeJetStreamConnection,
    subscriptions: Vec<(RiskSourceV1, RuntimeSubscribePermitV1)>,
    next: usize,
    generation: u64,
}
impl RiskManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control: UnixStream,
        descriptor: Vec<u8>,
        settings: Vec<u8>,
        admission: &RiskRuntimeAdmissionV1,
        storage: ManagedStorageRuntimeConfigurationV1,
        event_endpoint: &str,
        event_revision: u64,
        now: i64,
    ) -> Result<Self, RiskManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_endpoint.is_empty() || event_revision == 0 || now <= 0 {
            return Err(RiskManagedRuntimeErrorV1::Admission);
        }
        let mut control = ManagedControlChannelV2::new(control);
        authenticate(&mut control, descriptor, settings, admission)?;
        let binding = storage_binding(&storage, admission)?;
        let public_key = storage
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| RiskManagedRuntimeErrorV1::Admission)?;
        let context = StorageVaultRouteContextV1::new(
            storage.vault_instance_id.clone(),
            storage.vault_runtime_generation,
            public_key,
        )
        .map_err(|_| RiskManagedRuntimeErrorV1::Admission)?;
        let mut leases =
            StorageVaultLeaseAdapterV1::new(InheritedKernelVaultRouteV2::new(control), context);
        let password = resolve(&mut leases, &binding).await?;
        let password =
            std::str::from_utf8(&password).map_err(|_| RiskManagedRuntimeErrorV1::Admission)?;
        let persistence = RiskPersistenceV1::connect_runtime(
            &binding,
            &storage.database_id,
            &storage.pgbouncer_host,
            storage.pgbouncer_port,
            password,
        )
        .await
        .map_err(RiskManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(RiskManagedRuntimeErrorV1::Persistence)?;
        let generation = persistence
            .ensure_live_generation(&admission.logical_human_owner_id, now)
            .await
            .map_err(RiskManagedRuntimeErrorV1::Persistence)?;
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
        .map_err(|_| RiskManagedRuntimeErrorV1::EventUnavailable)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| RiskManagedRuntimeErrorV1::Admission)?;
        let subscriptions = bind(
            access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| RiskManagedRuntimeErrorV1::Admission)?,
        )?;
        let events = JetStreamClient::connect_runtime_with_jwt(
            event_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| RiskManagedRuntimeErrorV1::EventUnavailable)?;
        for (_, permit) in &subscriptions {
            events
                .open_pull_consumer(permit)
                .await
                .map_err(|_| RiskManagedRuntimeErrorV1::EventContract)?;
        }
        signal_ready(&mut control, admission)?;
        control
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| RiskManagedRuntimeErrorV1::Unavailable)?;
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
    pub async fn service_once(&mut self, now: i64) -> Result<bool, RiskManagedRuntimeErrorV1> {
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
                .map_err(|_| RiskManagedRuntimeErrorV1::EventContract)?;
            let context = RiskExecutionContextV1 {
                logical_owner_id: self.admission.logical_human_owner_id.clone(),
                projection_generation: self.generation,
                now_unix_millis: now,
            };
            match process_risk_source_event_v1(&self.persistence, &record, *source, &context).await
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
    ) -> Result<bool, RiskManagedRuntimeErrorV1> {
        let deadline = tokio::time::Instant::now() + delay;
        loop {
            match self.pump().await {
                Ok(_) => {}
                Err(RiskManagedRuntimeErrorV1::ControlClosed) => return Ok(false),
                Err(error) => return Err(error),
            }
            if tokio::time::Instant::now() >= deadline {
                return Ok(true);
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await
        }
    }
    async fn pump(&mut self) -> Result<bool, RiskManagedRuntimeErrorV1> {
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
        let response = dispatch_risk_client_request_v1(
            &self.persistence,
            &self.admission.logical_human_owner_id,
            request,
        )
        .await;
        validate_module_client_response_v1(&response)
            .map_err(|_| RiskManagedRuntimeErrorV1::Unavailable)?;
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
    fn write_error(&mut self, id: [u8; 16], code: &str) -> Result<(), RiskManagedRuntimeErrorV1> {
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
) -> Result<Vec<(RiskSourceV1, RuntimeSubscribePermitV1)>, RiskManagedRuntimeErrorV1> {
    let mut result = Vec::new();
    for permit in permits {
        let source = source_for(
            permit
                .contract()
                .ok_or(RiskManagedRuntimeErrorV1::Admission)?,
        )
        .ok_or(RiskManagedRuntimeErrorV1::Admission)?;
        if result.iter().any(|(v, _)| *v == source) {
            return Err(RiskManagedRuntimeErrorV1::Admission);
        }
        result.push((source, permit));
    }
    if result.len() != 2 {
        return Err(RiskManagedRuntimeErrorV1::Admission);
    }
    result.sort_by_key(|(v, _)| *v as u8);
    Ok(result)
}
fn source_for(contract: &makosh_runtime_protocol::v1::ContractReferenceV1) -> Option<RiskSourceV1> {
    use makosh_obligations_api::obligations_lifecycle_event_contract_reference_v1;
    use makosh_tasks_command_api::tasks_lifecycle_event_contract_reference_v1;
    [
        (
            RiskSourceV1::Tasks,
            tasks_lifecycle_event_contract_reference_v1(),
        ),
        (
            RiskSourceV1::Obligations,
            obligations_lifecycle_event_contract_reference_v1(),
        ),
    ]
    .into_iter()
    .find_map(|(source, expected)| (contract == &expected).then_some(source))
}
fn validate_admission(value: &RiskRuntimeAdmissionV1) -> Result<(), RiskManagedRuntimeErrorV1> {
    if value.module_owner_id != RISK_OWNER_ID_V1
        || value.logical_human_owner_id.is_empty()
        || value.registration_id.is_empty()
        || value.runtime_instance_id.is_empty()
        || value.runtime_generation == 0
        || value.grant_epoch == 0
    {
        Err(RiskManagedRuntimeErrorV1::Admission)
    } else {
        Ok(())
    }
}
fn authenticate(
    control: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &RiskRuntimeAdmissionV1,
) -> Result<(), RiskManagedRuntimeErrorV1> {
    control
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            control
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| RiskManagedRuntimeErrorV1::Unavailable)?;
    let response = control
        .describe_managed_runtime(descriptor, settings)
        .map_err(control_error)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(RiskManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}
fn signal_ready(
    control: &mut ManagedControlChannelV2<UnixStream>,
    admission: &RiskRuntimeAdmissionV1,
) -> Result<(), RiskManagedRuntimeErrorV1> {
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
        .map_err(|_| RiskManagedRuntimeErrorV1::Unavailable)
}
async fn resolve(
    adapter: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, RiskManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(value) = adapter.ensure_runtime_credential(binding).await {
            return Ok(value);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await
        }
    }
    Err(RiskManagedRuntimeErrorV1::Unavailable)
}
fn storage_binding(
    c: &ManagedStorageRuntimeConfigurationV1,
    a: &RiskRuntimeAdmissionV1,
) -> Result<StorageBindingV1, RiskManagedRuntimeErrorV1> {
    if c.runtime_instance_id != a.runtime_instance_id
        || c.logical_owner_id != RISK_OWNER_ID_V1
        || c.owner != RISK_OWNER_ID_V1
        || c.storage_bundle_digest.len() != 32
        || c.storage_generation == 0
        || c.credential_revision == 0
        || c.role_epoch == 0
        || c.storage_bundle_revision == 0
    {
        return Err(RiskManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        c.storage_instance_id.clone(),
        c.database_id.clone(),
        c.owner.clone(),
        a.registration_id.clone(),
        c.runtime_instance_id.clone(),
    )
    .map_err(|_| RiskManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        c.storage_generation,
        a.runtime_generation,
        a.grant_epoch,
        c.role_epoch,
        c.credential_revision,
        c.storage_bundle_revision,
    )
    .map_err(|_| RiskManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(c.max_connections).map_err(|_| RiskManagedRuntimeErrorV1::Admission)?,
        c.statement_timeout_millis,
    )
    .map_err(|_| RiskManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        c.runtime_principal.clone(),
        c.pool_alias.clone(),
        budgets,
        c.storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| RiskManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| RiskManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| RiskManagedRuntimeErrorV1::Admission)
}
const fn bounded(e: RiskExecutionErrorV1) -> bool {
    matches!(
        e,
        RiskExecutionErrorV1::InvalidEnvelope
            | RiskExecutionErrorV1::InvalidPayload
            | RiskExecutionErrorV1::Persistence(
                RiskPersistenceErrorV1::InvalidInput
                    | RiskPersistenceErrorV1::Conflict
                    | RiskPersistenceErrorV1::RevisionConflict
            )
    )
}
const fn execution_error(e: RiskExecutionErrorV1) -> RiskManagedRuntimeErrorV1 {
    match e {
        RiskExecutionErrorV1::Persistence(v) => RiskManagedRuntimeErrorV1::Persistence(v),
        _ => RiskManagedRuntimeErrorV1::EventContract,
    }
}
const fn event_error(_: RuntimePullDeliveryErrorV1) -> RiskManagedRuntimeErrorV1 {
    RiskManagedRuntimeErrorV1::EventUnavailable
}
fn control_error(e: ManagedControlTransportErrorV2) -> RiskManagedRuntimeErrorV1 {
    if matches!(e, ManagedControlTransportErrorV2::PeerClosed) {
        RiskManagedRuntimeErrorV1::ControlClosed
    } else {
        RiskManagedRuntimeErrorV1::Unavailable
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_sources_are_bound() {
        assert_eq!(
            source_for(&makosh_tasks_command_api::tasks_lifecycle_event_contract_reference_v1()),
            Some(RiskSourceV1::Tasks)
        );
        assert_eq!(
            source_for(
                &makosh_obligations_api::obligations_lifecycle_event_contract_reference_v1()
            ),
            Some(RiskSourceV1::Obligations)
        );
    }
}
