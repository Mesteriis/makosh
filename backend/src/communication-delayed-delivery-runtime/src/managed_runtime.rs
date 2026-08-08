use std::os::unix::net::UnixStream;

use crate::{
    COMMUNICATION_DELAYED_DELIVERY_BLOB_CAPABILITY_ID_V1,
    body_cleanup::process_pending_body_cleanup_v1,
    client_port::{
        DelayedDeliveryClientContextV1, cancel_delayed_delivery_payload_v1,
        get_delayed_delivery_status_payload_v1, schedule_delayed_delivery_payload_v1,
    },
    client_realtime::{
        DelayedDeliveryClientRealtimeErrorV1, DelayedDeliveryClientRealtimePublisherV1,
    },
    contracts::{
        delayed_delivery_cancel_command_contract_v1, delayed_delivery_query_contract_v1,
        delayed_delivery_schedule_command_contract_v1,
    },
    due_execution::{
        DelayedDeliveryDueExecutionContextV1, DelayedDeliveryDueExecutionErrorV1,
        consume_due_delivery_v1,
    },
    scheduler_outbox::{
        DelayedDeliverySchedulerOutboxErrorV1, relay_scheduler_commands_v1,
        relay_scheduler_receipts_v1,
    },
    scheduler_results::{DelayedDeliverySchedulerResultErrorV1, consume_scheduler_result_v1},
};
use makosh_communication_delayed_delivery_api::{
    COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1, COMMUNICATION_DELAYED_DELIVERY_OWNER_V1,
};
use makosh_communication_delayed_delivery_execution::DelayedDeliveryCleanupErrorV1;
use makosh_communication_delayed_delivery_persistence::{
    CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryPersistenceErrorV1,
};
use makosh_communication_delayed_delivery_runtime_adapters::ManagedDelayedDeliveryRuntimePortV1;
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::{
        ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeReadyRequestV1, ManagedStorageRuntimeConfigurationV1, ModuleClientRequestV1,
        ModuleClientResponseV1, managed_runtime_control_request_v1::Operation,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryManagedRuntimeErrorV1 {
    Admission,
    Persistence(DelayedDeliveryPersistenceErrorV1),
    InvalidTransition,
    Unavailable,
}

pub struct DelayedDeliveryManagedRuntimeV1 {
    admission: DelayedDeliveryRuntimeAdmissionV1,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: CommunicationDelayedDeliveryPersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    schedule_result_subscription: RuntimeSubscribePermitV1,
    due_subscription: RuntimeSubscribePermitV1,
    client_realtime: DelayedDeliveryClientRealtimePublisherV1,
}

impl DelayedDeliveryManagedRuntimeV1 {
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &DelayedDeliveryRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, DelayedDeliveryManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(DelayedDeliveryManagedRuntimeErrorV1::Admission);
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
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
        let persistence = CommunicationDelayedDeliveryPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(DelayedDeliveryManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(DelayedDeliveryManagedRuntimeErrorV1::Persistence)?;

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
        .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
        let event_identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
        let (schedule_result_subscription, due_subscription) = bind_subscribe_permits(
            event_access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?,
        )?;
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            event_identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
        let mut client_realtime = DelayedDeliveryClientRealtimePublisherV1::default();
        let mut dispatcher = RejectManagedControlRequestsV2;
        client_realtime
            .publish_pending(
                &persistence,
                &mut control_channel,
                &mut dispatcher,
                &admission.logical_owner_id,
            )
            .await
            .map_err(realtime_error)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;

        Ok(Self {
            admission: admission.clone(),
            control_channel,
            persistence,
            event_connection,
            event_publish_permit,
            schedule_result_subscription,
            due_subscription,
            client_realtime,
        })
    }

    pub async fn pump_control_once(
        &mut self,
        authoritative_now_unix_millis: u64,
    ) -> Result<bool, DelayedDeliveryManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            self.control_channel
                .write_response(
                    correlation_id,
                    ManagedRuntimeControlResponseV1 {
                        result: None,
                        error_code: "managed_runtime_control_unexpected_request".to_owned(),
                    },
                )
                .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
            return Ok(true);
        };
        let Some(request) = delivery
            .request
            .filter(|request| validate_module_client_request_v1(request).is_ok())
        else {
            self.control_channel
                .write_response(
                    correlation_id,
                    ManagedRuntimeControlResponseV1 {
                        result: None,
                        error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                    },
                )
                .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
            return Ok(true);
        };

        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let response = dispatch_client(
            &self.persistence,
            &mut self.control_channel,
            &mut dispatcher,
            &self.admission,
            request,
            authoritative_now_unix_millis,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
        if validate_module_client_response_v1(&response).is_err() {
            return Err(DelayedDeliveryManagedRuntimeErrorV1::Unavailable);
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
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn pump_client_realtime_once(
        &mut self,
    ) -> Result<bool, DelayedDeliveryManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
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
            .map_err(realtime_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
        result
    }

    pub async fn relay_scheduler_outbox_once(
        &self,
        published_at_unix_millis: u64,
    ) -> Result<bool, DelayedDeliveryManagedRuntimeErrorV1> {
        let commands = relay_scheduler_commands_v1(
            &self.persistence,
            &self.event_connection,
            &self.event_publish_permit,
            &self.admission.logical_owner_id,
            published_at_unix_millis,
        )
        .await
        .map_err(scheduler_outbox_error)?;
        let receipts = relay_scheduler_receipts_v1(
            &self.persistence,
            &self.event_connection,
            &self.event_publish_permit,
            &self.admission.logical_owner_id,
            published_at_unix_millis,
        )
        .await
        .map_err(scheduler_outbox_error)?;
        Ok(commands + receipts > 0)
    }

    pub async fn consume_scheduler_result_once(
        &self,
        received_at_unix_millis: u64,
    ) -> Result<bool, DelayedDeliveryManagedRuntimeErrorV1> {
        consume_scheduler_result_v1(
            &self.persistence,
            &self.event_connection,
            &self.schedule_result_subscription,
            &self.admission.logical_owner_id,
            received_at_unix_millis,
        )
        .await
        .map_err(scheduler_result_error)
    }

    pub async fn consume_due_delivery_once(
        &mut self,
        now_unix_millis: u64,
    ) -> Result<bool, DelayedDeliveryManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = consume_due_delivery_v1(
            &self.persistence,
            &self.event_connection,
            &self.due_subscription,
            &mut self.control_channel,
            &mut dispatcher,
            &DelayedDeliveryDueExecutionContextV1 {
                logical_owner_id: self.admission.logical_owner_id.clone(),
                runtime_instance_id: runtime_source_reference(&self.admission.runtime_instance_id)
                    .expect("validated runtime instance identity"),
                runtime_generation: self.admission.runtime_generation,
                grant_epoch: self.admission.grant_epoch,
            },
            now_unix_millis,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
        result.map_err(due_execution_error)
    }

    pub async fn process_body_cleanup_once(
        &mut self,
        now_unix_millis: u64,
    ) -> Result<bool, DelayedDeliveryManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = process_pending_body_cleanup_v1(
            &self.persistence,
            &mut self.control_channel,
            &mut dispatcher,
            &self.admission.logical_owner_id,
            now_unix_millis,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
        result
            .map(|outcome| {
                !matches!(
                    outcome,
                    makosh_communication_delayed_delivery_execution::DelayedDeliveryCleanupOutcomeV1::Idle
                )
            })
            .map_err(body_cleanup_error)
    }
}

async fn dispatch_client(
    persistence: &CommunicationDelayedDeliveryPersistenceV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut RejectManagedControlRequestsV2,
    admission: &DelayedDeliveryRuntimeAdmissionV1,
    request: ModuleClientRequestV1,
    authoritative_now_unix_millis: u64,
) -> ModuleClientResponseV1 {
    let valid_identity = request.protocol_major == 1
        && request.module_id == COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1
        && request.owner_id == COMMUNICATION_DELAYED_DELIVERY_OWNER_V1
        && request.logical_owner_id == admission.logical_owner_id;
    let context = DelayedDeliveryClientContextV1 {
        logical_owner_id: admission.logical_owner_id.clone(),
        runtime_instance_id: runtime_source_reference(&admission.runtime_instance_id)
            .expect("validated runtime instance identity"),
        runtime_generation: admission.runtime_generation,
        grant_epoch: admission.grant_epoch,
        authoritative_now_unix_millis,
    };
    let (payload, accepted_route) = if !valid_identity {
        (Vec::new(), false)
    } else if request.contract.as_ref() == Some(&delayed_delivery_schedule_command_contract_v1()) {
        let payload = match ManagedDelayedDeliveryRuntimePortV1::new(
            control_channel,
            dispatcher,
            COMMUNICATION_DELAYED_DELIVERY_BLOB_CAPABILITY_ID_V1,
        ) {
            Ok(mut custody) => {
                schedule_delayed_delivery_payload_v1(
                    persistence,
                    &mut custody,
                    &context,
                    &request.request_payload,
                )
                .await
            }
            Err(_) => Vec::new(),
        };
        (payload, true)
    } else if request.contract.as_ref() == Some(&delayed_delivery_cancel_command_contract_v1()) {
        (
            cancel_delayed_delivery_payload_v1(persistence, &context, &request.request_payload)
                .await,
            true,
        )
    } else if request.contract.as_ref() == Some(&delayed_delivery_query_contract_v1()) {
        (
            get_delayed_delivery_status_payload_v1(
                persistence,
                &admission.logical_owner_id,
                &request.request_payload,
            )
            .await,
            true,
        )
    } else {
        (Vec::new(), false)
    };

    ModuleClientResponseV1 {
        protocol_major: 1,
        request_id: request.request_id,
        response_payload: payload,
        error_code: if accepted_route {
            String::new()
        } else {
            "REJECTED".to_owned()
        },
    }
}

fn runtime_source_reference(runtime_instance_id: &str) -> Option<[u8; 16]> {
    if runtime_instance_id.len() != 32 {
        return None;
    }
    let mut bytes = [0; 16];
    for (index, item) in bytes.iter_mut().enumerate() {
        *item = u8::from_str_radix(&runtime_instance_id[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

fn validate_admission(
    admission: &DelayedDeliveryRuntimeAdmissionV1,
) -> Result<(), DelayedDeliveryManagedRuntimeErrorV1> {
    if admission.logical_owner_id.is_empty()
        || admission.registration_id.is_empty()
        || runtime_source_reference(&admission.runtime_instance_id).is_none()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(DelayedDeliveryManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &DelayedDeliveryRuntimeAdmissionV1,
) -> Result<(), DelayedDeliveryManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(DelayedDeliveryManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &DelayedDeliveryRuntimeAdmissionV1,
) -> Result<(), DelayedDeliveryManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, DelayedDeliveryManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(DelayedDeliveryManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &DelayedDeliveryRuntimeAdmissionV1,
) -> Result<StorageBindingV1, DelayedDeliveryManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(DelayedDeliveryManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| DelayedDeliveryManagedRuntimeErrorV1::Admission)
}

fn realtime_error(
    error: DelayedDeliveryClientRealtimeErrorV1,
) -> DelayedDeliveryManagedRuntimeErrorV1 {
    match error {
        DelayedDeliveryClientRealtimeErrorV1::InvalidTransition => {
            DelayedDeliveryManagedRuntimeErrorV1::InvalidTransition
        }
        DelayedDeliveryClientRealtimeErrorV1::Persistence(error) => {
            DelayedDeliveryManagedRuntimeErrorV1::Persistence(error)
        }
        DelayedDeliveryClientRealtimeErrorV1::Unavailable => {
            DelayedDeliveryManagedRuntimeErrorV1::Unavailable
        }
    }
}

fn scheduler_outbox_error(
    error: DelayedDeliverySchedulerOutboxErrorV1,
) -> DelayedDeliveryManagedRuntimeErrorV1 {
    match error {
        DelayedDeliverySchedulerOutboxErrorV1::Persistence(error) => {
            DelayedDeliveryManagedRuntimeErrorV1::Persistence(error)
        }
        DelayedDeliverySchedulerOutboxErrorV1::EventUnavailable => {
            DelayedDeliveryManagedRuntimeErrorV1::Unavailable
        }
    }
}

fn scheduler_result_error(
    error: DelayedDeliverySchedulerResultErrorV1,
) -> DelayedDeliveryManagedRuntimeErrorV1 {
    match error {
        DelayedDeliverySchedulerResultErrorV1::InvalidResult => {
            eprintln!("developer_delayed_delivery_broker_rejected=invalid_scheduler_result");
            DelayedDeliveryManagedRuntimeErrorV1::InvalidTransition
        }
        DelayedDeliverySchedulerResultErrorV1::Persistence(error) => {
            DelayedDeliveryManagedRuntimeErrorV1::Persistence(error)
        }
        DelayedDeliverySchedulerResultErrorV1::EventUnavailable => {
            DelayedDeliveryManagedRuntimeErrorV1::Unavailable
        }
    }
}

fn due_execution_error(
    error: DelayedDeliveryDueExecutionErrorV1,
) -> DelayedDeliveryManagedRuntimeErrorV1 {
    match error {
        DelayedDeliveryDueExecutionErrorV1::InvalidCommand => {
            eprintln!("developer_delayed_delivery_broker_rejected=invalid_due_command");
            DelayedDeliveryManagedRuntimeErrorV1::InvalidTransition
        }
        DelayedDeliveryDueExecutionErrorV1::Store(_)
        | DelayedDeliveryDueExecutionErrorV1::EventUnavailable => {
            DelayedDeliveryManagedRuntimeErrorV1::Unavailable
        }
    }
}

fn body_cleanup_error(
    error: DelayedDeliveryCleanupErrorV1,
) -> DelayedDeliveryManagedRuntimeErrorV1 {
    match error {
        DelayedDeliveryCleanupErrorV1::InvalidInput => {
            DelayedDeliveryManagedRuntimeErrorV1::InvalidTransition
        }
        DelayedDeliveryCleanupErrorV1::Store(error) => match error {
            makosh_communication_delayed_delivery_execution::ExecutionStoreErrorV1::InvalidInput
            | makosh_communication_delayed_delivery_execution::ExecutionStoreErrorV1::Conflict
            | makosh_communication_delayed_delivery_execution::ExecutionStoreErrorV1::ClaimLost
            | makosh_communication_delayed_delivery_execution::ExecutionStoreErrorV1::NotFound => {
                DelayedDeliveryManagedRuntimeErrorV1::InvalidTransition
            }
            makosh_communication_delayed_delivery_execution::ExecutionStoreErrorV1::Unavailable => {
                DelayedDeliveryManagedRuntimeErrorV1::Unavailable
            }
        },
    }
}

fn bind_subscribe_permits(
    permits: Vec<RuntimeSubscribePermitV1>,
) -> Result<
    (RuntimeSubscribePermitV1, RuntimeSubscribePermitV1),
    DelayedDeliveryManagedRuntimeErrorV1,
> {
    if permits.len() != 2 {
        return Err(DelayedDeliveryManagedRuntimeErrorV1::Admission);
    }
    let schedule = permits
        .iter()
        .find(|permit| {
            permit.contract().is_some_and(|contract| {
                contract.owner == "scheduler" && contract.name == "schedule_control"
            })
        })
        .cloned()
        .ok_or(DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
    let due = permits
        .iter()
        .find(|permit| {
            permit.contract().is_some_and(|contract| {
                contract.owner == "communication_delayed_delivery" && contract.name == "execute"
            })
        })
        .cloned()
        .ok_or(DelayedDeliveryManagedRuntimeErrorV1::Admission)?;
    Ok((schedule, due))
}

#[cfg(test)]
mod tests {
    use super::runtime_source_reference;

    #[test]
    fn runtime_source_reference_preserves_the_exact_managed_identity() {
        assert_eq!(
            runtime_source_reference("5a903dfad7f58b794797c33b781d84b1"),
            Some([
                0x5a, 0x90, 0x3d, 0xfa, 0xd7, 0xf5, 0x8b, 0x79, 0x47, 0x97, 0xc3, 0x3b, 0x78, 0x1d,
                0x84, 0xb1,
            ])
        );
    }

    #[test]
    fn runtime_source_reference_rejects_non_exact_identity_text() {
        assert_eq!(runtime_source_reference("runtime-instance"), None);
        assert_eq!(
            runtime_source_reference("5a903dfad7f58b794797c33b781d84bz"),
            None
        );
    }
}
