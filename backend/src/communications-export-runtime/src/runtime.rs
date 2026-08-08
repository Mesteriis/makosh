//! Kernel-fenced independently restartable Communications Export workflow.

use std::{
    os::unix::net::UnixStream,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use makosh_communications_evidence_export_source_api::{
    evidence_export_prepared_contract_reference_v1, evidence_export_rejected_contract_reference_v1,
};
use makosh_communications_export_persistence::CommunicationsExportPersistenceV1;
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
    },
    v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryResponseV1,
        ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
        ManagedStorageRuntimeConfigurationV1, managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::{
        managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES,
        module_client::{validate_module_client_request_v1, validate_module_client_response_v1},
    },
};
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};

use crate::{
    client_port::dispatch_communications_export_client_request_v1,
    client_realtime::{
        CommunicationsExportClientRealtimeErrorV1, CommunicationsExportClientRealtimePublisherV1,
    },
    event_consumer::{
        CommunicationsExportEventConsumerErrorV1, consume_next_prepared_result_v1,
        consume_next_rejected_result_v1,
    },
    materializer::{
        CommunicationsExportMaterializerErrorV1, process_next_communications_export_v1,
    },
    outbox::{CommunicationsExportOutboxErrorV1, relay_communications_export_outbox_v1},
    ticket_store::CommunicationsExportTicketStoreV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsExportRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsExportRuntimeErrorV1 {
    Admission,
    Unavailable,
    InvalidDelivery,
}

pub struct CommunicationsExportRuntimeV1 {
    control_channel: ManagedControlChannelV2<UnixStream>,
    connection: RuntimeJetStreamConnection,
    permits: CommunicationsExportSubscribePermitsV1,
    next_consumer: CommunicationsExportConsumerV1,
    publish_permit: RuntimePublishPermitV1,
    persistence: CommunicationsExportPersistenceV1,
    tickets: Arc<CommunicationsExportTicketStoreV1>,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    logical_owner_id: String,
    client_realtime: CommunicationsExportClientRealtimePublisherV1,
}

struct CommunicationsExportSubscribePermitsV1 {
    prepared: RuntimeSubscribePermitV1,
    rejected: RuntimeSubscribePermitV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommunicationsExportConsumerV1 {
    Prepared,
    Rejected,
}

impl CommunicationsExportConsumerV1 {
    const fn successor(self) -> Self {
        match self {
            Self::Prepared => Self::Rejected,
            Self::Rejected => Self::Prepared,
        }
    }
}

impl CommunicationsExportSubscribePermitsV1 {
    fn bind(
        permits: Vec<RuntimeSubscribePermitV1>,
    ) -> Result<Self, CommunicationsExportRuntimeErrorV1> {
        let prepared = evidence_export_prepared_contract_reference_v1();
        let rejected = evidence_export_rejected_contract_reference_v1();
        let mut prepared_permit = None;
        let mut rejected_permit = None;
        for permit in permits {
            let Some(contract) = permit.contract() else {
                return Err(CommunicationsExportRuntimeErrorV1::Admission);
            };
            if exact_contract(contract, &prepared) {
                replace_once(&mut prepared_permit, permit)?;
            } else if exact_contract(contract, &rejected) {
                replace_once(&mut rejected_permit, permit)?;
            } else {
                return Err(CommunicationsExportRuntimeErrorV1::Admission);
            }
        }
        Ok(Self {
            prepared: prepared_permit.ok_or(CommunicationsExportRuntimeErrorV1::Admission)?,
            rejected: rejected_permit.ok_or(CommunicationsExportRuntimeErrorV1::Admission)?,
        })
    }
}

struct CommunicationsExportNestedDispatcherV1<'a> {
    persistence: &'a CommunicationsExportPersistenceV1,
    tickets: &'a Arc<CommunicationsExportTicketStoreV1>,
    runtime_instance_id: &'a str,
    runtime_generation: u64,
    grant_epoch: u64,
}

impl ManagedControlRequestDispatcherV2<UnixStream> for CommunicationsExportNestedDispatcherV1<'_> {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: makosh_runtime_protocol::v1::ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    let response = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(
                            dispatch_communications_export_client_request_v1(
                                self.persistence,
                                self.tickets,
                                self.runtime_instance_id,
                                self.runtime_generation,
                                self.grant_epoch,
                                &request,
                            ),
                        )
                    });
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(response),
                            },
                        )),
                        error_code: String::new(),
                    }
                }
                _ => ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                },
            },
            _ => ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        };
        channel.write_response(correlation_id, response)
    }
}

impl CommunicationsExportRuntimeV1 {
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &CommunicationsExportRuntimeAdmissionV1,
        event_hub_endpoint: &str,
        credential_revision: u64,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
    ) -> Result<Self, CommunicationsExportRuntimeErrorV1> {
        validate_open(
            &descriptor_bytes,
            &settings_schema_bytes,
            admission,
            event_hub_endpoint,
            credential_revision,
        )?;
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate_managed_runtime_v2(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let access = request_managed_runtime_event_access_v2(
            &mut control_channel,
            &storage_configuration.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            credential_revision,
        )
        .map_err(|error| {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_communications_export_event_access_error={error:?}");
            }
            unavailable_at("event_access")
        })?;
        let permits = CommunicationsExportSubscribePermitsV1::bind(
            access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?,
        )?;
        let publish_permit = access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?;
        let connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| unavailable_at("event_connection"))?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let vault_public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_runtime_credential(&mut leases, &binding)
            .await
            .map_err(|_| unavailable_at("storage_credential"))?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?;
        let persistence = CommunicationsExportPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(|_| unavailable_at("storage_connection"))?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(|_| unavailable_at("storage_readiness"))?;
        let mut control_channel = leases.into_route_port().into_channel();
        let tickets = Arc::new(
            CommunicationsExportTicketStoreV1::new(
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?,
        );
        let mut client_realtime = CommunicationsExportClientRealtimePublisherV1::default();
        let mut dispatcher = CommunicationsExportNestedDispatcherV1 {
            persistence: &persistence,
            tickets: &tickets,
            runtime_instance_id: &admission.runtime_instance_id,
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        };
        client_realtime
            .publish_pending(
                &persistence,
                &mut control_channel,
                &mut dispatcher,
                &admission.logical_owner_id,
            )
            .await
            .map_err(client_realtime_error)?;
        signal_managed_runtime_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            control_channel,
            connection,
            permits,
            next_consumer: CommunicationsExportConsumerV1::Prepared,
            publish_permit,
            persistence,
            tickets,
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
            logical_owner_id: admission.logical_owner_id.clone(),
            client_realtime,
        })
    }

    pub async fn try_handle_client_delivery(
        &mut self,
    ) -> Result<bool, CommunicationsExportRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            return Err(CommunicationsExportRuntimeErrorV1::InvalidDelivery);
        };
        let request = delivery
            .request
            .filter(|request| validate_module_client_request_v1(request).is_ok())
            .ok_or(CommunicationsExportRuntimeErrorV1::InvalidDelivery)?;
        let response = dispatch_communications_export_client_request_v1(
            &self.persistence,
            &self.tickets,
            &self.runtime_instance_id,
            self.runtime_generation,
            self.grant_epoch,
            &request,
        )
        .await;
        validate_module_client_response_v1(&response)
            .map_err(|_| CommunicationsExportRuntimeErrorV1::InvalidDelivery)?;
        self.control_channel
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
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn consume_next(&mut self) -> Result<(), CommunicationsExportRuntimeErrorV1> {
        let now = now_unix_seconds()?;
        let consumer = self.next_consumer;
        self.next_consumer = consumer.successor();
        let result = match consumer {
            CommunicationsExportConsumerV1::Prepared => {
                consume_next_prepared_result_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.prepared,
                    now,
                )
                .await
            }
            CommunicationsExportConsumerV1::Rejected => {
                consume_next_rejected_result_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.rejected,
                    now,
                )
                .await
            }
        };
        result.map_err(event_consumer_error)
    }

    pub async fn process_next_materialization(
        &mut self,
    ) -> Result<bool, CommunicationsExportRuntimeErrorV1> {
        let now = now_unix_seconds()?;
        let mut dispatcher = CommunicationsExportNestedDispatcherV1 {
            persistence: &self.persistence,
            tickets: &self.tickets,
            runtime_instance_id: &self.runtime_instance_id,
            runtime_generation: self.runtime_generation,
            grant_epoch: self.grant_epoch,
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?;
        let result = process_next_communications_export_v1(
            &self.persistence,
            &mut self.control_channel,
            &mut dispatcher,
            &format!(
                "communications-export-{}-{}",
                self.runtime_generation, self.grant_epoch
            ),
            now,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?;
        match result {
            Ok(processed) => Ok(processed),
            Err(CommunicationsExportMaterializerErrorV1::RetryPending) => Ok(false),
            Err(CommunicationsExportMaterializerErrorV1::StorageUnavailable) => {
                Err(CommunicationsExportRuntimeErrorV1::Unavailable)
            }
        }
    }

    pub async fn relay_outbox(&self) -> Result<usize, CommunicationsExportRuntimeErrorV1> {
        relay_communications_export_outbox_v1(
            &self.persistence,
            &self.connection,
            &self.publish_permit,
            now_unix_seconds()?,
        )
        .await
        .map_err(outbox_error)
    }

    pub async fn pump_client_realtime_once(
        &mut self,
    ) -> Result<bool, CommunicationsExportRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = CommunicationsExportNestedDispatcherV1 {
            persistence: &self.persistence,
            tickets: &self.tickets,
            runtime_instance_id: &self.runtime_instance_id,
            runtime_generation: self.runtime_generation,
            grant_epoch: self.grant_epoch,
        };
        let result = self
            .client_realtime
            .publish_pending(
                &self.persistence,
                &mut self.control_channel,
                &mut dispatcher,
                &self.logical_owner_id,
            )
            .await
            .map_err(client_realtime_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?;
        result
    }
}

fn unavailable_at(stage: &'static str) -> CommunicationsExportRuntimeErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_communications_export_startup_unavailable stage={stage}");
    }
    CommunicationsExportRuntimeErrorV1::Unavailable
}

fn validate_open(
    descriptor_bytes: &[u8],
    settings_schema_bytes: &[u8],
    admission: &CommunicationsExportRuntimeAdmissionV1,
    event_hub_endpoint: &str,
    credential_revision: u64,
) -> Result<(), CommunicationsExportRuntimeErrorV1> {
    if descriptor_bytes.is_empty()
        || settings_schema_bytes.is_empty()
        || admission.logical_owner_id.is_empty()
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
        || event_hub_endpoint.is_empty()
        || credential_revision == 0
    {
        return Err(CommunicationsExportRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn replace_once(
    slot: &mut Option<RuntimeSubscribePermitV1>,
    permit: RuntimeSubscribePermitV1,
) -> Result<(), CommunicationsExportRuntimeErrorV1> {
    slot.replace(permit)
        .is_none()
        .then_some(())
        .ok_or(CommunicationsExportRuntimeErrorV1::Admission)
}

fn exact_contract(left: &ContractReferenceV1, right: &ContractReferenceV1) -> bool {
    left.owner == right.owner
        && left.name == right.name
        && left.major == right.major
        && left.revision == right.revision
        && left.schema_sha256 == right.schema_sha256
}

fn event_consumer_error(
    error: CommunicationsExportEventConsumerErrorV1,
) -> CommunicationsExportRuntimeErrorV1 {
    match error {
        CommunicationsExportEventConsumerErrorV1::Unavailable => {
            CommunicationsExportRuntimeErrorV1::Unavailable
        }
        _ => CommunicationsExportRuntimeErrorV1::InvalidDelivery,
    }
}

fn outbox_error(_: CommunicationsExportOutboxErrorV1) -> CommunicationsExportRuntimeErrorV1 {
    CommunicationsExportRuntimeErrorV1::Unavailable
}

fn client_realtime_error(
    _: CommunicationsExportClientRealtimeErrorV1,
) -> CommunicationsExportRuntimeErrorV1 {
    CommunicationsExportRuntimeErrorV1::Unavailable
}

fn now_unix_seconds() -> Result<i64, CommunicationsExportRuntimeErrorV1> {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?
            .as_secs(),
    )
    .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_runtime_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, CommunicationsExportRuntimeErrorV1> {
    const MAX_ATTEMPTS: usize = 20;
    for attempt in 0..MAX_ATTEMPTS {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt + 1 < MAX_ATTEMPTS {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(CommunicationsExportRuntimeErrorV1::Unavailable)
}

fn authenticate_managed_runtime_v2(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &CommunicationsExportRuntimeAdmissionV1,
) -> Result<(), CommunicationsExportRuntimeErrorV1> {
    control_channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            control_channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?;
    let response = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(CommunicationsExportRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_managed_runtime_ready(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &CommunicationsExportRuntimeAdmissionV1,
) -> Result<(), CommunicationsExportRuntimeErrorV1> {
    control_channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .map_err(|_| CommunicationsExportRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &CommunicationsExportRuntimeAdmissionV1,
) -> Result<StorageBindingV1, CommunicationsExportRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(CommunicationsExportRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| CommunicationsExportRuntimeErrorV1::Admission)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumers_fairly_alternate() {
        assert_eq!(
            CommunicationsExportConsumerV1::Prepared.successor(),
            CommunicationsExportConsumerV1::Rejected
        );
        assert_eq!(
            CommunicationsExportConsumerV1::Rejected.successor(),
            CommunicationsExportConsumerV1::Prepared
        );
    }
}
