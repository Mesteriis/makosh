use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePullDeliveryErrorV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
    try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_managed_vault_client::owner_derived_key::{
    ManagedOwnerDerivedKeyContextV1, ensure_managed_owner_derived_key_v2,
};
use makosh_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlTransportErrorV2, RejectManagedControlRequestsV2,
    },
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
use makosh_search_api::SEARCH_OWNER_ID_V1;
use makosh_search_persistence::{SearchPersistenceErrorV1, SearchPersistenceV1};
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};
use zeroize::Zeroizing;

use crate::{
    SearchExecutionContextV1, SearchExecutionErrorV1, SearchSourceV1,
    admission::{
        SEARCH_OWNER_KEY_PURPOSE_ID_V1, SEARCH_OWNER_KEY_SCHEMA_REVISION_V1,
        SEARCH_OWNER_KEY_TTL_SECONDS_V1,
    },
    dispatch_search_client_request_v1, process_search_source_event_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchRuntimeAdmissionV1 {
    pub module_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(SearchPersistenceErrorV1),
    ControlClosed,
    Unavailable,
}

pub struct SearchManagedRuntimeV1 {
    admission: SearchRuntimeAdmissionV1,
    control: ManagedControlChannelV2<UnixStream>,
    persistence: SearchPersistenceV1,
    events: RuntimeJetStreamConnection,
    subscriptions: Vec<(SearchSourceV1, RuntimeSubscribePermitV1)>,
    next_subscription: usize,
    projection_generation: u64,
    owner_derived_key: Zeroizing<[u8; 32]>,
}

impl SearchManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control: UnixStream,
        descriptor: Vec<u8>,
        settings: Vec<u8>,
        admission: &SearchRuntimeAdmissionV1,
        storage: ManagedStorageRuntimeConfigurationV1,
        event_endpoint: &str,
        event_revision: u64,
        now_unix_millis: i64,
    ) -> Result<Self, SearchManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_endpoint.is_empty() || event_revision == 0 || now_unix_millis <= 0 {
            return Err(SearchManagedRuntimeErrorV1::Admission);
        }
        let mut control = ManagedControlChannelV2::new(control);
        authenticate(&mut control, descriptor, settings, admission)?;
        let binding = storage_binding(&storage, admission)?;
        let vault_public_key = storage
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage.vault_instance_id.clone(),
            storage.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password =
            std::str::from_utf8(&password).map_err(|_| SearchManagedRuntimeErrorV1::Admission)?;
        let persistence = SearchPersistenceV1::connect_runtime(
            &binding,
            &storage.database_id,
            &storage.pgbouncer_host,
            storage.pgbouncer_port,
            password,
        )
        .await
        .map_err(SearchManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(SearchManagedRuntimeErrorV1::Persistence)?;
        let projection_generation = persistence
            .ensure_live_generation(&admission.logical_human_owner_id, now_unix_millis)
            .await
            .map_err(SearchManagedRuntimeErrorV1::Persistence)?;
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
        .map_err(|_| SearchManagedRuntimeErrorV1::EventUnavailable)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?;
        let permits = access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?;
        let subscriptions = bind_subscriptions(permits)?;
        let events = JetStreamClient::connect_runtime_with_jwt(
            event_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| SearchManagedRuntimeErrorV1::EventUnavailable)?;
        for (_, permit) in &subscriptions {
            events
                .open_pull_consumer(permit)
                .await
                .map_err(|_| SearchManagedRuntimeErrorV1::EventContract)?;
        }
        let owner_key_context = ManagedOwnerDerivedKeyContextV1 {
            vault_instance_id: storage.vault_instance_id,
            vault_runtime_generation: storage.vault_runtime_generation,
            vault_public_key_x25519: vault_public_key,
            logical_owner_id: storage.logical_owner_id,
            registration_id: admission.registration_id.clone(),
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        };
        let mut reject = RejectManagedControlRequestsV2;
        let owner_derived_key: [u8; 32] = ensure_managed_owner_derived_key_v2(
            &mut control,
            &mut reject,
            &owner_key_context,
            makosh_search_api::SEARCH_PROJECTION_CAPABILITY_ID_V1,
            SEARCH_OWNER_KEY_PURPOSE_ID_V1,
            SEARCH_OWNER_KEY_SCHEMA_REVISION_V1,
            SEARCH_OWNER_KEY_TTL_SECONDS_V1,
        )
        .map_err(|_| SearchManagedRuntimeErrorV1::Unavailable)?
        .as_slice()
        .try_into()
        .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?;
        signal_ready(&mut control, admission)?;
        control
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| SearchManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control,
            persistence,
            events,
            subscriptions,
            next_subscription: 0,
            projection_generation,
            owner_derived_key: Zeroizing::new(owner_derived_key),
        })
    }

    pub async fn service_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, SearchManagedRuntimeErrorV1> {
        if self.pump_control_once().await? {
            return Ok(true);
        }
        let count = self.subscriptions.len();
        for offset in 0..count {
            let index = (self.next_subscription + offset) % count;
            let (source, permit) = &self.subscriptions[index];
            let Some(delivery) = try_receive_runtime_pull_delivery(&self.events, permit)
                .await
                .map_err(event_error)?
            else {
                continue;
            };
            self.next_subscription = (index + 1) % count;
            let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
                .map_err(|_| SearchManagedRuntimeErrorV1::EventContract)?;
            let context = SearchExecutionContextV1 {
                logical_owner_id: self.admission.logical_human_owner_id.clone(),
                projection_generation: self.projection_generation,
                owner_derived_key: *self.owner_derived_key,
                now_unix_millis,
            };
            match process_search_source_event_v1(&self.persistence, &record, *source, &context)
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
    ) -> Result<bool, SearchManagedRuntimeErrorV1> {
        let deadline = tokio::time::Instant::now() + delay;
        loop {
            match self.pump_control_once().await {
                Ok(_) => {}
                Err(SearchManagedRuntimeErrorV1::ControlClosed) => return Ok(false),
                Err(error) => return Err(error),
            }
            if tokio::time::Instant::now() >= deadline {
                return Ok(true);
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    async fn pump_control_once(&mut self) -> Result<bool, SearchManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) =
            self.control.try_receive_request().map_err(control_error)?
        else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            self.write_control_error(correlation_id, "managed_runtime_control_unexpected_request")?;
            return Ok(true);
        };
        let Some(request) = delivery
            .request
            .filter(|value| validate_module_client_request_v1(value).is_ok())
        else {
            self.write_control_error(
                correlation_id,
                "managed_runtime_control_invalid_client_delivery",
            )?;
            return Ok(true);
        };
        let response = dispatch_search_client_request_v1(
            &self.persistence,
            &self.admission.logical_human_owner_id,
            &self.owner_derived_key,
            request,
        )
        .await;
        validate_module_client_response_v1(&response)
            .map_err(|_| SearchManagedRuntimeErrorV1::Unavailable)?;
        self.control
            .write_response(
                correlation_id,
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

    fn write_control_error(
        &mut self,
        id: [u8; 16],
        code: &str,
    ) -> Result<(), SearchManagedRuntimeErrorV1> {
        self.control
            .write_response(
                id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: code.to_owned(),
                },
            )
            .map_err(control_error)
    }
}

fn bind_subscriptions(
    permits: Vec<RuntimeSubscribePermitV1>,
) -> Result<Vec<(SearchSourceV1, RuntimeSubscribePermitV1)>, SearchManagedRuntimeErrorV1> {
    let mut bound = Vec::with_capacity(permits.len());
    for permit in permits {
        let contract = permit
            .contract()
            .ok_or(SearchManagedRuntimeErrorV1::Admission)?;
        let source = source_for_contract(contract).ok_or(SearchManagedRuntimeErrorV1::Admission)?;
        if bound.iter().any(|(existing, _)| *existing == source) {
            return Err(SearchManagedRuntimeErrorV1::Admission);
        }
        bound.push((source, permit));
    }
    if bound.len() != 10 {
        return Err(SearchManagedRuntimeErrorV1::Admission);
    }
    bound.sort_by_key(|(source, _)| *source as u8);
    Ok(bound)
}

fn source_for_contract(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> Option<SearchSourceV1> {
    use makosh_calendar_api::calendar_lifecycle_event_contract_reference_v1;
    use makosh_decisions_api::decisions_lifecycle_event_contract_reference_v1;
    use makosh_documents_api::documents_lifecycle_event_contract_reference_v1;
    use makosh_knowledge_command_api::knowledge_lifecycle_event_contract_reference_v1;
    use makosh_obligations_api::obligations_lifecycle_event_contract_reference_v1;
    use makosh_organizations_api::organizations_lifecycle_event_contract_reference_v1;
    use makosh_persons_api::persons_owner_event_contract_reference_v1;
    use makosh_projects_api::projects_lifecycle_event_contract_reference_v1;
    use makosh_relationships_api::relationships_lifecycle_event_contract_reference_v1;
    use makosh_tasks_command_api::tasks_lifecycle_event_contract_reference_v1;
    [
        (
            SearchSourceV1::Persons,
            persons_owner_event_contract_reference_v1(),
        ),
        (
            SearchSourceV1::Organizations,
            organizations_lifecycle_event_contract_reference_v1(),
        ),
        (
            SearchSourceV1::Relationships,
            relationships_lifecycle_event_contract_reference_v1(),
        ),
        (
            SearchSourceV1::Projects,
            projects_lifecycle_event_contract_reference_v1(),
        ),
        (
            SearchSourceV1::Tasks,
            tasks_lifecycle_event_contract_reference_v1(),
        ),
        (
            SearchSourceV1::Obligations,
            obligations_lifecycle_event_contract_reference_v1(),
        ),
        (
            SearchSourceV1::Decisions,
            decisions_lifecycle_event_contract_reference_v1(),
        ),
        (
            SearchSourceV1::Calendar,
            calendar_lifecycle_event_contract_reference_v1(),
        ),
        (
            SearchSourceV1::Documents,
            documents_lifecycle_event_contract_reference_v1(),
        ),
        (
            SearchSourceV1::Knowledge,
            knowledge_lifecycle_event_contract_reference_v1(),
        ),
    ]
    .into_iter()
    .find_map(|(source, expected)| (contract == &expected).then_some(source))
}

fn validate_admission(value: &SearchRuntimeAdmissionV1) -> Result<(), SearchManagedRuntimeErrorV1> {
    if value.module_owner_id != SEARCH_OWNER_ID_V1
        || value.logical_human_owner_id.is_empty()
        || value.registration_id.is_empty()
        || value.runtime_instance_id.is_empty()
        || value.runtime_generation == 0
        || value.grant_epoch == 0
    {
        Err(SearchManagedRuntimeErrorV1::Admission)
    } else {
        Ok(())
    }
}

fn authenticate(
    control: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &SearchRuntimeAdmissionV1,
) -> Result<(), SearchManagedRuntimeErrorV1> {
    control
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            control
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| SearchManagedRuntimeErrorV1::Unavailable)?;
    let response = control
        .describe_managed_runtime(descriptor, settings)
        .map_err(control_error)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(SearchManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    control: &mut ManagedControlChannelV2<UnixStream>,
    admission: &SearchRuntimeAdmissionV1,
) -> Result<(), SearchManagedRuntimeErrorV1> {
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
        .map_err(|_| SearchManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    adapter: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, SearchManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(value) = adapter.ensure_runtime_credential(binding).await {
            return Ok(value);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(SearchManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &SearchRuntimeAdmissionV1,
) -> Result<StorageBindingV1, SearchManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != SEARCH_OWNER_ID_V1
        || configuration.owner != SEARCH_OWNER_ID_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(SearchManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| SearchManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| SearchManagedRuntimeErrorV1::Admission)
}

const fn bounded(error: SearchExecutionErrorV1) -> bool {
    matches!(
        error,
        SearchExecutionErrorV1::InvalidEnvelope
            | SearchExecutionErrorV1::InvalidPayload
            | SearchExecutionErrorV1::Persistence(
                SearchPersistenceErrorV1::InvalidInput
                    | SearchPersistenceErrorV1::Conflict
                    | SearchPersistenceErrorV1::RevisionConflict
            )
    )
}

const fn execution_error(error: SearchExecutionErrorV1) -> SearchManagedRuntimeErrorV1 {
    match error {
        SearchExecutionErrorV1::Persistence(value) => {
            SearchManagedRuntimeErrorV1::Persistence(value)
        }
        SearchExecutionErrorV1::InvalidContext
        | SearchExecutionErrorV1::InvalidEnvelope
        | SearchExecutionErrorV1::InvalidPayload => SearchManagedRuntimeErrorV1::EventContract,
    }
}

const fn event_error(_: RuntimePullDeliveryErrorV1) -> SearchManagedRuntimeErrorV1 {
    SearchManagedRuntimeErrorV1::EventUnavailable
}
fn control_error(error: ManagedControlTransportErrorV2) -> SearchManagedRuntimeErrorV1 {
    match error {
        ManagedControlTransportErrorV2::PeerClosed => SearchManagedRuntimeErrorV1::ControlClosed,
        _ => SearchManagedRuntimeErrorV1::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_sources_are_exact_and_fairly_bound() {
        assert_eq!(
            source_for_contract(&makosh_persons_api::persons_owner_event_contract_reference_v1()),
            Some(SearchSourceV1::Persons)
        );
        assert_eq!(
            source_for_contract(
                &makosh_organizations_api::organizations_lifecycle_event_contract_reference_v1()
            ),
            Some(SearchSourceV1::Organizations)
        );
    }
}
