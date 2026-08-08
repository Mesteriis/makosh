//! Kernel-fenced managed process and owner-local Storage bootstrap.

use std::os::unix::net::UnixStream;

use makosh_communication_delivery_intent_core::{
    DeliveryIntentDraftV1, DeliveryIntentPlanErrorV1, PlannedDeliveryIntentV1,
    plan_delivery_intent_v1,
};
use makosh_communication_delivery_intent_persistence::{
    CommunicationDeliveryIntentPersistenceV1, CreateDeliveryIntentOutcomeV1,
    DeliveryIntentPersistenceErrorV1, DeliveryIntentStatusRecordV1,
};
use makosh_events_jetstream::{
    JetStreamClient, ManagedRuntimeEventAccessErrorV1, RuntimeJetStreamConnection,
    RuntimeNatsIdentity, RuntimePublishPermitV1, RuntimeSubscribePermitV1,
    request_managed_runtime_event_access_v2,
};
use makosh_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, RejectManagedControlRequestsV2,
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
    validation::module_request::validate_module_request_response_v1,
};
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};

use crate::{
    body_materializer::ManagedDeliveryIntentBodyMaterializerV1,
    client_port::dispatch_delivery_intent_client_request_v1,
    client_realtime::{
        DeliveryIntentClientRealtimeErrorV1, DeliveryIntentClientRealtimePublisherV1,
    },
    communications_query_client::{
        CommunicationsQueryClientErrorV1, ManagedCommunicationsQueryClientV1,
    },
    coordinator::{DeliveryIntentCoordinatorErrorV1, prepare_create_delivery_intent_v1},
    event_ingress::bind_delivery_intent_ingress_subscription,
    event_runtime::{ProviderTerminalSubscriptionV1, bind_terminal_subscriptions},
    module_request_port::handle_module_request_delivery_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryIntentRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentRuntimeErrorV1 {
    Admission,
    Coordinator(DeliveryIntentCoordinatorErrorV1),
    Persistence(DeliveryIntentPersistenceErrorV1),
    EventContract,
    InvalidRequest,
    RouteUnavailable,
    Unavailable,
}

pub struct DeliveryIntentManagedRuntimeV1 {
    pub(crate) logical_owner_id: String,
    pub(crate) control_channel: ManagedControlChannelV2<UnixStream>,
    pub(crate) persistence: CommunicationDeliveryIntentPersistenceV1,
    pub(crate) runtime_instance_id: String,
    pub(crate) runtime_generation: u64,
    pub(crate) event_connection: RuntimeJetStreamConnection,
    pub(crate) event_publish_permit: RuntimePublishPermitV1,
    pub(crate) event_ingress_subscription: RuntimeSubscribePermitV1,
    pub(crate) terminal_subscriptions: Vec<ProviderTerminalSubscriptionV1>,
    pub(crate) next_terminal_subscription: usize,
    client_realtime: DeliveryIntentClientRealtimePublisherV1,
}

impl DeliveryIntentManagedRuntimeV1 {
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &DeliveryIntentRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, DeliveryIntentRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(DeliveryIntentRuntimeErrorV1::Admission);
        }
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate_managed_runtime_v2(
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
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_runtime_credential(&mut leases, &binding).await?;
        let password =
            std::str::from_utf8(&password).map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let persistence = CommunicationDeliveryIntentPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(persistence_error)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(persistence_error)?;
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
        .map_err(event_access_error)?;
        let event_identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let mut subscribe_permits = event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
        let event_ingress_subscription =
            bind_delivery_intent_ingress_subscription(&mut subscribe_permits)?;
        let terminal_subscriptions = bind_terminal_subscriptions(subscribe_permits)?;
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            event_identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        let mut client_realtime = DeliveryIntentClientRealtimePublisherV1::default();
        let mut bootstrap_dispatcher = RejectManagedControlRequestsV2;
        client_realtime
            .publish_pending(
                &persistence,
                &mut control_channel,
                &mut bootstrap_dispatcher,
                &admission.logical_owner_id,
            )
            .await
            .map_err(client_realtime_error)?;
        signal_managed_runtime_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            logical_owner_id: admission.logical_owner_id.clone(),
            control_channel,
            persistence,
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
            event_connection,
            event_publish_permit,
            event_ingress_subscription,
            terminal_subscriptions,
            next_terminal_subscription: 0,
            client_realtime,
        })
    }

    pub fn persistence(&self) -> &CommunicationDeliveryIntentPersistenceV1 {
        &self.persistence
    }

    pub async fn create_delivery_intent_v1(
        &mut self,
        planned: PlannedDeliveryIntentV1,
        created_at_unix_seconds: i64,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    ) -> Result<CreateDeliveryIntentOutcomeV1, DeliveryIntentRuntimeErrorV1> {
        let command = {
            let mut materializer = ManagedDeliveryIntentBodyMaterializerV1 {
                control_channel: &mut self.control_channel,
                dispatcher,
            };
            prepare_create_delivery_intent_v1(
                self.logical_owner_id.clone(),
                planned,
                created_at_unix_seconds,
                &mut materializer,
            )
            .map_err(DeliveryIntentRuntimeErrorV1::Coordinator)?
        };
        self.persistence
            .create_intent(&command)
            .await
            .map_err(DeliveryIntentRuntimeErrorV1::Persistence)
    }

    pub async fn submit_delivery_intent_v1(
        &mut self,
        draft: DeliveryIntentDraftV1,
        created_at_unix_seconds: i64,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    ) -> Result<CreateDeliveryIntentOutcomeV1, DeliveryIntentRuntimeErrorV1> {
        let (conversation, reply) = {
            let mut query_client = ManagedCommunicationsQueryClientV1 {
                control_channel: &mut self.control_channel,
                dispatcher,
            };
            query_client
                .resolve_route_sources(
                    draft.operation_id,
                    draft.conversation_id,
                    draft.reply_to_message_id,
                )
                .map_err(query_error)?
        };
        let planned =
            plan_delivery_intent_v1(draft, &conversation, reply.as_ref()).map_err(plan_error)?;
        self.create_delivery_intent_v1(planned, created_at_unix_seconds, dispatcher)
            .await
    }

    pub async fn delivery_intent_status_v1(
        &self,
        intent_id: [u8; 16],
    ) -> Result<Option<DeliveryIntentStatusRecordV1>, DeliveryIntentRuntimeErrorV1> {
        self.persistence
            .status(&self.logical_owner_id, intent_id)
            .await
            .map_err(DeliveryIntentRuntimeErrorV1::Persistence)
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_seconds: i64,
    ) -> Result<bool, DeliveryIntentRuntimeErrorV1> {
        let Some((correlation_id, control_request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let delivery = match control_request.operation {
            Some(Operation::DeliverModuleRequest(delivery)) => {
                let request_id = delivery.request_id.clone();
                let mut dispatcher = RejectManagedControlRequestsV2;
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
                let response = handle_module_request_delivery_v1(
                    self,
                    &mut dispatcher,
                    delivery,
                    now_unix_seconds,
                )
                .await;
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
                if validate_module_request_response_v1(&response).is_err()
                    || response.request_id != request_id
                {
                    return Err(DeliveryIntentRuntimeErrorV1::Unavailable);
                }
                self.control_channel
                    .write_response(
                        correlation_id,
                        ManagedRuntimeControlResponseV1 {
                            result: Some(ControlResult::ModuleRequestDelivery(response)),
                            error_code: String::new(),
                        },
                    )
                    .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
                return Ok(true);
            }
            Some(Operation::ClientDelivery(delivery)) => delivery,
            _ => {
                self.control_channel
                    .write_response(
                        correlation_id,
                        ManagedRuntimeControlResponseV1 {
                            result: None,
                            error_code: "managed_runtime_control_unexpected_request".to_owned(),
                        },
                    )
                    .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
                return Ok(true);
            }
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
                .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
            return Ok(true);
        };
        let mut dispatcher = RejectManagedControlRequestsV2;
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        let response = dispatch_delivery_intent_client_request_v1(
            self,
            &mut dispatcher,
            &request,
            now_unix_seconds,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        if validate_module_client_response_v1(&response).is_err()
            || response.request_id != request.request_id
        {
            return Err(DeliveryIntentRuntimeErrorV1::Unavailable);
        }
        let response = ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::ClientDelivery(
                ManagedRuntimeClientDeliveryResponseV1 {
                    response: Some(response),
                },
            )),
            error_code: String::new(),
        };
        self.control_channel
            .write_response(correlation_id, response)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn pump_client_realtime_once(
        &mut self,
    ) -> Result<bool, DeliveryIntentRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
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
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        result
    }
}

fn event_access_error(error: ManagedRuntimeEventAccessErrorV1) -> DeliveryIntentRuntimeErrorV1 {
    match error {
        ManagedRuntimeEventAccessErrorV1::Rejected => DeliveryIntentRuntimeErrorV1::EventContract,
        ManagedRuntimeEventAccessErrorV1::Unavailable => DeliveryIntentRuntimeErrorV1::Unavailable,
    }
}

fn validate_admission(
    admission: &DeliveryIntentRuntimeAdmissionV1,
) -> Result<(), DeliveryIntentRuntimeErrorV1> {
    if admission.logical_owner_id.is_empty()
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(DeliveryIntentRuntimeErrorV1::Admission);
    }
    Ok(())
}

async fn resolve_storage_runtime_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, DeliveryIntentRuntimeErrorV1> {
    const MAX_ATTEMPTS: usize = 20;
    for attempt in 0..MAX_ATTEMPTS {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt + 1 < MAX_ATTEMPTS {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(DeliveryIntentRuntimeErrorV1::Unavailable)
}

fn authenticate_managed_runtime_v2(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &DeliveryIntentRuntimeAdmissionV1,
) -> Result<(), DeliveryIntentRuntimeErrorV1> {
    control_channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            control_channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
    let response = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(DeliveryIntentRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_managed_runtime_ready(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &DeliveryIntentRuntimeAdmissionV1,
) -> Result<(), DeliveryIntentRuntimeErrorV1> {
    control_channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &DeliveryIntentRuntimeAdmissionV1,
) -> Result<StorageBindingV1, DeliveryIntentRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(DeliveryIntentRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| DeliveryIntentRuntimeErrorV1::Admission)
}

fn persistence_error(_: DeliveryIntentPersistenceErrorV1) -> DeliveryIntentRuntimeErrorV1 {
    DeliveryIntentRuntimeErrorV1::Unavailable
}

const fn client_realtime_error(
    error: DeliveryIntentClientRealtimeErrorV1,
) -> DeliveryIntentRuntimeErrorV1 {
    match error {
        DeliveryIntentClientRealtimeErrorV1::InvalidTransition => {
            DeliveryIntentRuntimeErrorV1::EventContract
        }
        DeliveryIntentClientRealtimeErrorV1::Persistence(error) => {
            DeliveryIntentRuntimeErrorV1::Persistence(error)
        }
        DeliveryIntentClientRealtimeErrorV1::Unavailable => {
            DeliveryIntentRuntimeErrorV1::Unavailable
        }
    }
}

const fn query_error(_: CommunicationsQueryClientErrorV1) -> DeliveryIntentRuntimeErrorV1 {
    DeliveryIntentRuntimeErrorV1::RouteUnavailable
}

const fn plan_error(_: DeliveryIntentPlanErrorV1) -> DeliveryIntentRuntimeErrorV1 {
    DeliveryIntentRuntimeErrorV1::InvalidRequest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_requires_current_runtime_fences() {
        let mut admission = DeliveryIntentRuntimeAdmissionV1 {
            logical_owner_id: "owner:test".to_owned(),
            registration_id: "delivery-intent".to_owned(),
            runtime_instance_id: "delivery-intent-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        };
        assert_eq!(validate_admission(&admission), Ok(()));
        admission.grant_epoch = 0;
        assert_eq!(
            validate_admission(&admission),
            Err(DeliveryIntentRuntimeErrorV1::Admission)
        );
    }
}
