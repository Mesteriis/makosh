use std::os::unix::net::UnixStream;

use makosh_communication_cross_channel_forward_api::COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1;
use makosh_communication_cross_channel_forward_persistence::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
};
use makosh_communication_delivery_intent_ingress_api::{
    communication_delivery_intent_rejected_contract_reference_v1,
    communication_delivery_intent_submitted_contract_reference_v1,
};
use makosh_communications_cross_channel_forward_source_api::{
    cross_channel_forward_source_prepared_contract_reference_v1,
    cross_channel_forward_source_rejected_contract_reference_v1,
};
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryResponseV1,
        ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
        ManagedStorageRuntimeConfigurationV1, ModuleClientRequestV1, ModuleClientResponseV1,
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

use crate::{
    CrossChannelForwardBlobTransferErrorV1, CrossChannelForwardCustodyCleanupErrorV1,
    CrossChannelForwardDeliveryResultErrorV1, CrossChannelForwardEventRelayErrorV1,
    CrossChannelForwardSourceConsumerContextV1, CrossChannelForwardSourcePrepareErrorV1,
    CrossChannelForwardSourceResultErrorV1, ManagedCrossChannelForwardBlobPortV1,
    ManagedCrossChannelForwardCustodyReleasePortV1,
    client_port::{
        get_cross_channel_forward_status_payload_v1, start_cross_channel_forward_payload_v1,
    },
    client_realtime::{
        CrossChannelForwardClientRealtimeErrorV1, CrossChannelForwardClientRealtimePublisherV1,
    },
    consume_delivery_rejected_once_v1, consume_delivery_submitted_once_v1,
    consume_source_prepared_once_v1, consume_source_rejected_once_v1,
    contracts::{
        cross_channel_forward_command_contract_v1, cross_channel_forward_query_contract_v1,
    },
    enqueue_source_prepare_once_v1, process_cross_channel_custody_cleanup_once_v1,
    relay_event_outbox_once_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardManagedRuntimeErrorV1 {
    Admission,
    Blob(CrossChannelForwardBlobTransferErrorV1),
    EventContract,
    EventUnavailable,
    InvalidTransition,
    Persistence(CrossChannelForwardPersistenceErrorV1),
    Unavailable,
}

pub struct CrossChannelForwardManagedRuntimeV1 {
    admission: CrossChannelForwardRuntimeAdmissionV1,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: CommunicationCrossChannelForwardPersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    delivery_rejected_subscription: RuntimeSubscribePermitV1,
    delivery_submitted_subscription: RuntimeSubscribePermitV1,
    source_prepared_subscription: RuntimeSubscribePermitV1,
    source_rejected_subscription: RuntimeSubscribePermitV1,
    client_realtime: CrossChannelForwardClientRealtimePublisherV1,
}

impl CrossChannelForwardManagedRuntimeV1 {
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &CrossChannelForwardRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, CrossChannelForwardManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(CrossChannelForwardManagedRuntimeErrorV1::Admission);
        }
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let vault_public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?;
        let persistence = CommunicationCrossChannelForwardPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(CrossChannelForwardManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(CrossChannelForwardManagedRuntimeErrorV1::Persistence)?;

        let mut control_channel = leases.into_route_port().into_channel();
        let event_access = request_managed_runtime_event_access_v2(
            &mut control_channel,
            &storage_configuration.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            event_credential_revision,
        )
        .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::EventUnavailable)?;
        let event_identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?;
        let (
            delivery_rejected_subscription,
            delivery_submitted_subscription,
            source_prepared_subscription,
            source_rejected_subscription,
        ) = bind_result_subscriptions(
            event_access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?,
        )?;
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            event_identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::EventUnavailable)?;
        let mut client_realtime = CrossChannelForwardClientRealtimePublisherV1::default();
        let mut dispatcher = RejectManagedControlRequestsV2;
        client_realtime
            .publish_pending(
                &persistence,
                &mut control_channel,
                &mut dispatcher,
                &admission.logical_owner_id,
            )
            .await
            .map_err(client_realtime_error)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control_channel,
            persistence,
            event_connection,
            event_publish_permit,
            delivery_rejected_subscription,
            delivery_submitted_subscription,
            source_prepared_subscription,
            source_rejected_subscription,
            client_realtime,
        })
    }

    pub async fn enqueue_source_prepare_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, CrossChannelForwardManagedRuntimeErrorV1> {
        enqueue_source_prepare_once_v1(
            &self.persistence,
            &self.admission.logical_owner_id,
            &self.admission.runtime_instance_id,
            self.admission.runtime_generation,
            now_unix_millis,
        )
        .await
        .map_err(source_prepare_error)
    }

    pub async fn relay_event_outbox_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, CrossChannelForwardManagedRuntimeErrorV1> {
        relay_event_outbox_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.event_publish_permit,
            now_unix_millis,
        )
        .await
        .map_err(event_relay_error)
    }

    pub async fn consume_source_prepared_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, CrossChannelForwardManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = {
            let mut blob_port = ManagedCrossChannelForwardBlobPortV1 {
                control_channel: &mut self.control_channel,
                dispatcher: &mut dispatcher,
            };
            consume_source_prepared_once_v1(
                &self.persistence,
                &self.event_connection,
                &self.source_prepared_subscription,
                &mut blob_port,
                &CrossChannelForwardSourceConsumerContextV1 {
                    expected_logical_owner_id: &self.admission.logical_owner_id,
                    runtime_instance_id: &self.admission.runtime_instance_id,
                    runtime_generation: self.admission.runtime_generation,
                    consumed_at_unix_millis: now_unix_millis,
                },
            )
            .await
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
        result.map_err(source_result_error)
    }

    pub async fn consume_source_rejected_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, CrossChannelForwardManagedRuntimeErrorV1> {
        consume_source_rejected_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.source_rejected_subscription,
            &self.admission.logical_owner_id,
            now_unix_millis,
        )
        .await
        .map_err(source_result_error)
    }

    pub async fn consume_delivery_submitted_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, CrossChannelForwardManagedRuntimeErrorV1> {
        consume_delivery_submitted_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.delivery_submitted_subscription,
            &self.admission.logical_owner_id,
            now_unix_millis,
        )
        .await
        .map_err(delivery_result_error)
    }

    pub async fn consume_delivery_rejected_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, CrossChannelForwardManagedRuntimeErrorV1> {
        consume_delivery_rejected_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.delivery_rejected_subscription,
            &self.admission.logical_owner_id,
            now_unix_millis,
        )
        .await
        .map_err(delivery_result_error)
    }

    pub async fn process_custody_cleanup_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, CrossChannelForwardManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = {
            let mut release_port = ManagedCrossChannelForwardCustodyReleasePortV1 {
                control_channel: &mut self.control_channel,
                dispatcher: &mut dispatcher,
            };
            process_cross_channel_custody_cleanup_once_v1(
                &self.persistence,
                &self.admission.logical_owner_id,
                now_unix_millis,
                &mut release_port,
            )
            .await
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
        result.map_err(custody_cleanup_error)
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, CrossChannelForwardManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            self.write_client_error(correlation_id, "managed_runtime_control_unexpected_request")?;
            return Ok(true);
        };
        let Some(request) = delivery
            .request
            .filter(|request| validate_module_client_request_v1(request).is_ok())
        else {
            self.write_client_error(
                correlation_id,
                "managed_runtime_control_invalid_client_delivery",
            )?;
            return Ok(true);
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
        let response = dispatch_client(
            &self.persistence,
            &self.admission.logical_owner_id,
            request,
            now_unix_millis,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
        if validate_module_client_response_v1(&response).is_err() {
            return Err(CrossChannelForwardManagedRuntimeErrorV1::Unavailable);
        }
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
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn pump_client_realtime_once(
        &mut self,
    ) -> Result<bool, CrossChannelForwardManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = self
            .client_realtime
            .publish_pending(
                &self.persistence,
                &mut self.control_channel,
                &mut dispatcher,
                &self.admission.logical_owner_id,
            )
            .await
            .map_err(client_realtime_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
        result
    }

    fn write_client_error(
        &mut self,
        correlation_id: [u8; 16],
        error_code: &str,
    ) -> Result<(), CrossChannelForwardManagedRuntimeErrorV1> {
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: error_code.to_owned(),
                },
            )
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)
    }
}

async fn dispatch_client(
    persistence: &CommunicationCrossChannelForwardPersistenceV1,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let valid_identity = request.protocol_major == 1
        && request.module_id
            == makosh_communication_cross_channel_forward_api::COMMUNICATION_CROSS_CHANNEL_FORWARD_MODULE_ID_V1
        && request.owner_id == COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1;
    let (response_payload, accepted_route) = if valid_identity {
        if request.contract.as_ref() == Some(&cross_channel_forward_command_contract_v1()) {
            (
                start_cross_channel_forward_payload_v1(
                    persistence,
                    logical_owner_id,
                    &request.request_payload,
                    now_unix_millis,
                )
                .await,
                true,
            )
        } else if request.contract.as_ref() == Some(&cross_channel_forward_query_contract_v1()) {
            (
                get_cross_channel_forward_status_payload_v1(
                    persistence,
                    logical_owner_id,
                    &request.request_payload,
                )
                .await,
                true,
            )
        } else {
            (Vec::new(), false)
        }
    } else {
        (Vec::new(), false)
    };
    ModuleClientResponseV1 {
        protocol_major: 1,
        request_id: request.request_id,
        response_payload,
        error_code: if accepted_route {
            String::new()
        } else {
            "REJECTED".to_owned()
        },
    }
}

fn bind_result_subscriptions(
    permits: Vec<RuntimeSubscribePermitV1>,
) -> Result<
    (
        RuntimeSubscribePermitV1,
        RuntimeSubscribePermitV1,
        RuntimeSubscribePermitV1,
        RuntimeSubscribePermitV1,
    ),
    CrossChannelForwardManagedRuntimeErrorV1,
> {
    if permits.len() != 4 {
        return Err(CrossChannelForwardManagedRuntimeErrorV1::Admission);
    }
    let delivery_rejected = exact_permit(
        &permits,
        &communication_delivery_intent_rejected_contract_reference_v1(),
    )?;
    let delivery_submitted = exact_permit(
        &permits,
        &communication_delivery_intent_submitted_contract_reference_v1(),
    )?;
    let prepared = exact_permit(
        &permits,
        &cross_channel_forward_source_prepared_contract_reference_v1(),
    )?;
    let rejected = exact_permit(
        &permits,
        &cross_channel_forward_source_rejected_contract_reference_v1(),
    )?;
    Ok((delivery_rejected, delivery_submitted, prepared, rejected))
}

fn exact_permit(
    permits: &[RuntimeSubscribePermitV1],
    contract: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, CrossChannelForwardManagedRuntimeErrorV1> {
    let mut matching = permits.iter().filter(|permit| {
        permit.contract().is_some_and(|actual| {
            actual.owner == contract.owner
                && actual.name == contract.name
                && actual.major == contract.major
                && actual.revision == contract.revision
                && actual.schema_sha256 == contract.schema_sha256
        })
    });
    let permit = matching
        .next()
        .cloned()
        .ok_or(CrossChannelForwardManagedRuntimeErrorV1::Admission)?;
    if matching.next().is_some() {
        return Err(CrossChannelForwardManagedRuntimeErrorV1::Admission);
    }
    Ok(permit)
}

fn validate_admission(
    admission: &CrossChannelForwardRuntimeAdmissionV1,
) -> Result<(), CrossChannelForwardManagedRuntimeErrorV1> {
    if admission.logical_owner_id.is_empty()
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(CrossChannelForwardManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &CrossChannelForwardRuntimeAdmissionV1,
) -> Result<(), CrossChannelForwardManagedRuntimeErrorV1> {
    control_channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            control_channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
    let response = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(CrossChannelForwardManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &CrossChannelForwardRuntimeAdmissionV1,
) -> Result<(), CrossChannelForwardManagedRuntimeErrorV1> {
    control_channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, CrossChannelForwardManagedRuntimeErrorV1> {
    const MAX_ATTEMPTS: usize = 20;
    for attempt in 0..MAX_ATTEMPTS {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt + 1 < MAX_ATTEMPTS {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(CrossChannelForwardManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &CrossChannelForwardRuntimeAdmissionV1,
) -> Result<StorageBindingV1, CrossChannelForwardManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.owner != COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(CrossChannelForwardManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| CrossChannelForwardManagedRuntimeErrorV1::Admission)
}

fn source_prepare_error(
    error: CrossChannelForwardSourcePrepareErrorV1,
) -> CrossChannelForwardManagedRuntimeErrorV1 {
    match error {
        CrossChannelForwardSourcePrepareErrorV1::InvalidContext => {
            CrossChannelForwardManagedRuntimeErrorV1::EventContract
        }
        CrossChannelForwardSourcePrepareErrorV1::Persistence(error) => {
            CrossChannelForwardManagedRuntimeErrorV1::Persistence(error)
        }
    }
}

fn event_relay_error(
    error: CrossChannelForwardEventRelayErrorV1,
) -> CrossChannelForwardManagedRuntimeErrorV1 {
    match error {
        CrossChannelForwardEventRelayErrorV1::InvalidTimestamp => {
            CrossChannelForwardManagedRuntimeErrorV1::EventContract
        }
        CrossChannelForwardEventRelayErrorV1::Persistence(error) => {
            CrossChannelForwardManagedRuntimeErrorV1::Persistence(error)
        }
        CrossChannelForwardEventRelayErrorV1::EventUnavailable => {
            CrossChannelForwardManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn source_result_error(
    error: CrossChannelForwardSourceResultErrorV1,
) -> CrossChannelForwardManagedRuntimeErrorV1 {
    match error {
        CrossChannelForwardSourceResultErrorV1::InvalidEnvelope
        | CrossChannelForwardSourceResultErrorV1::InvalidPayload => {
            CrossChannelForwardManagedRuntimeErrorV1::EventContract
        }
        CrossChannelForwardSourceResultErrorV1::Blob(error) => {
            CrossChannelForwardManagedRuntimeErrorV1::Blob(error)
        }
        CrossChannelForwardSourceResultErrorV1::Persistence(error) => {
            CrossChannelForwardManagedRuntimeErrorV1::Persistence(error)
        }
        CrossChannelForwardSourceResultErrorV1::EventUnavailable => {
            CrossChannelForwardManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn delivery_result_error(
    error: CrossChannelForwardDeliveryResultErrorV1,
) -> CrossChannelForwardManagedRuntimeErrorV1 {
    match error {
        CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope
        | CrossChannelForwardDeliveryResultErrorV1::InvalidPayload => {
            CrossChannelForwardManagedRuntimeErrorV1::EventContract
        }
        CrossChannelForwardDeliveryResultErrorV1::Persistence(error) => {
            CrossChannelForwardManagedRuntimeErrorV1::Persistence(error)
        }
        CrossChannelForwardDeliveryResultErrorV1::EventUnavailable => {
            CrossChannelForwardManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn custody_cleanup_error(
    error: CrossChannelForwardCustodyCleanupErrorV1,
) -> CrossChannelForwardManagedRuntimeErrorV1 {
    match error {
        CrossChannelForwardCustodyCleanupErrorV1::Blob(error) => {
            CrossChannelForwardManagedRuntimeErrorV1::Blob(error)
        }
        CrossChannelForwardCustodyCleanupErrorV1::Persistence(error) => {
            CrossChannelForwardManagedRuntimeErrorV1::Persistence(error)
        }
    }
}

fn client_realtime_error(
    error: CrossChannelForwardClientRealtimeErrorV1,
) -> CrossChannelForwardManagedRuntimeErrorV1 {
    match error {
        CrossChannelForwardClientRealtimeErrorV1::InvalidTransition => {
            CrossChannelForwardManagedRuntimeErrorV1::InvalidTransition
        }
        CrossChannelForwardClientRealtimeErrorV1::Persistence(error) => {
            CrossChannelForwardManagedRuntimeErrorV1::Persistence(error)
        }
        CrossChannelForwardClientRealtimeErrorV1::Unavailable => {
            CrossChannelForwardManagedRuntimeErrorV1::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CrossChannelForwardRuntimeAdmissionV1, validate_admission};

    #[test]
    fn admission_requires_current_runtime_and_grant_fences() {
        let mut admission = CrossChannelForwardRuntimeAdmissionV1 {
            logical_owner_id: "owner-1".to_owned(),
            registration_id: "registration-1".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        };
        assert_eq!(validate_admission(&admission), Ok(()));
        admission.grant_epoch = 0;
        assert!(validate_admission(&admission).is_err());
    }

    #[test]
    fn module_identity_is_exact_workflow_unit() {
        assert_eq!(
            makosh_communication_cross_channel_forward_api::COMMUNICATION_CROSS_CHANNEL_FORWARD_MODULE_ID_V1,
            "makosh-communication-cross-channel-forward-runtime"
        );
    }
}
