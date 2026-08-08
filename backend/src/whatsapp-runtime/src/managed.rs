//! Kernel-admitted WhatsApp runtime composition. It receives no browser state
//! or provider credential material; the host owns that boundary.

use std::io::ErrorKind;
use std::os::unix::{
    fs::PermissionsExt,
    net::{UnixListener, UnixStream},
};
use std::time::Duration;

use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_runtime_protocol::managed_control::ManagedControlChannelV2;
use makosh_runtime_protocol::v1::{
    ManagedIntegrationHostBridgeConfigurationV1, ManagedRuntimeClientDeliveryResponseV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    ManagedStorageRuntimeConfigurationV1, ModuleClientResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_runtime_protocol::validation::integration_host_bridge::validate_managed_integration_host_bridge_configuration;
use makosh_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;
use makosh_runtime_protocol::validation::module_client::{
    validate_module_client_request_v1, validate_module_client_response_v1,
};
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};

use crate::{
    WhatsAppCommandQueueError, WhatsAppOperationalQueryError, WhatsAppOperationalReplayError,
    WhatsAppRuntimeAdmission, WhatsAppRuntimeIdentity, accept_host_observation,
    claim_provider_commands,
    delivery_intent_consumer::{
        WhatsAppDeliveryIntentConsumeErrorV1, WhatsAppDeliveryIntentResultContextV1,
        consume_next_whatsapp_delivery_intent_v1,
    },
    delivery_intent_outbox::{
        WhatsAppDeliveryIntentOutboxRelayErrorV1, relay_whatsapp_delivery_intent_outbox_once_v1,
    },
    delivery_intent_worker::{
        WhatsAppDeliveryIntentWorkerContextV1, WhatsAppDeliveryIntentWorkerErrorV1,
        process_next_whatsapp_delivery_intent_v1,
    },
    enqueue_provider_command, provider_command_status, relay_communications_outbox_once,
    settings::WhatsAppRuntimeSettingsV1,
};
use makosh_whatsapp_api::{
    WhatsAppProviderCommand, WhatsAppProviderCommandStatusV1,
    host_bridge::{WhatsAppHostBridgeEnvelopeV1, WhatsAppHostBridgeHandshakeV1},
    operational::{
        WhatsAppOperationalQueryResponseV1, WhatsAppOperationalQueryV1,
        operational_query_account_id,
    },
    provider_command_account_id, provider_command_operation_id,
    realtime::{WhatsAppOperationalReplayRequestV1, WhatsAppOperationalReplayResponseV1},
};
use makosh_whatsapp_delivery_intent_contract::whatsapp_delivery_intent_execute_contract_reference_v1;
use makosh_whatsapp_persistence::WhatsAppDurablePersistence;
use prost::Message;

const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

pub struct WhatsAppAdmittedRuntime {
    pub control_channel: ManagedControlChannelV2<UnixStream>,
    pub durable: WhatsAppDurablePersistence,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    delivery_intent_subscribe_permit: RuntimeSubscribePermitV1,
    identity: WhatsAppRuntimeIdentity,
    logical_owner_id: String,
    account_id: String,
    host_bridge_socket_path: String,
    host_bridge_route_binding: [u8; 32],
}

#[derive(Debug, Eq, PartialEq)]
pub enum WhatsAppBootstrapError {
    Admission,
    Control,
    HostBridge,
    Storage,
    Credential,
    Persistence,
    EventHub,
}

#[allow(clippy::too_many_arguments)]
pub async fn open_admitted_runtime(
    control_channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    settings: &WhatsAppRuntimeSettingsV1,
    admission: &WhatsAppRuntimeAdmission,
    storage_configuration: ManagedStorageRuntimeConfigurationV1,
    host_bridge_configuration: ManagedIntegrationHostBridgeConfigurationV1,
    event_hub_endpoint: &str,
    event_credential_revision: u64,
) -> Result<WhatsAppAdmittedRuntime, WhatsAppBootstrapError> {
    if descriptor_bytes.is_empty()
        || settings_schema_bytes.is_empty()
        || admission.logical_owner_id.trim().is_empty()
        || admission.logical_owner_id.len() > 128
        || !admission.logical_owner_id.is_ascii()
        || admission.runtime_instance_id.trim().is_empty()
        || settings.account_id.trim().is_empty()
        || event_hub_endpoint.trim().is_empty()
        || event_credential_revision == 0
    {
        return Err(WhatsAppBootstrapError::Admission);
    }
    control_channel
        .set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| control_channel.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|_| WhatsAppBootstrapError::Control)?;
    let mut control_channel = ManagedControlChannelV2::new(control_channel);
    let identity = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| WhatsAppBootstrapError::Control)?;
    let registration_id = identity.registration_id;
    let runtime_generation = identity.runtime_generation;
    let grant_epoch = identity.grant_epoch;
    if registration_id != admission.module_registration_id
        || runtime_generation != admission.runtime_generation
        || grant_epoch != admission.grant_epoch
    {
        return Err(WhatsAppBootstrapError::Admission);
    }

    let binding = storage_binding(&storage_configuration, admission)?;
    let (host_bridge_socket_path, host_bridge_route_binding) =
        host_bridge_route(&host_bridge_configuration, admission)?;
    let storage_context = StorageVaultRouteContextV1::new(
        storage_configuration.vault_instance_id.clone(),
        storage_configuration.vault_runtime_generation,
        storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| WhatsAppBootstrapError::Storage)?,
    )
    .map_err(|_| WhatsAppBootstrapError::Storage)?;
    let mut storage_leases = StorageVaultLeaseAdapterV1::new(
        InheritedKernelVaultRouteV2::new(control_channel),
        storage_context,
    );
    let lease_id = storage_leases
        .issue_runtime_credential(&binding)
        .await
        .map_err(|_| WhatsAppBootstrapError::Credential)?;
    let password = storage_leases
        .resolve_runtime_credential(&binding, lease_id)
        .await
        .map_err(|_| WhatsAppBootstrapError::Credential)?;
    let mut control_channel = storage_leases.into_route_port().into_channel();
    let password =
        std::str::from_utf8(&password).map_err(|_| WhatsAppBootstrapError::Credential)?;
    let durable = WhatsAppDurablePersistence::connect_runtime(
        &binding,
        &storage_configuration.database_id,
        &storage_configuration.pgbouncer_host,
        storage_configuration.pgbouncer_port,
        password,
    )
    .await
    .map_err(|_| WhatsAppBootstrapError::Persistence)?;

    let event_access = request_managed_runtime_event_access_v2(
        &mut control_channel,
        &admission.logical_owner_id,
        &admission.module_registration_id,
        &admission.runtime_instance_id,
        admission.runtime_generation,
        admission.grant_epoch,
        event_credential_revision,
    )
    .map_err(|_| WhatsAppBootstrapError::EventHub)?;
    let identity = RuntimeNatsIdentity::new(
        admission.runtime_instance_id.clone(),
        admission.runtime_generation,
        admission.grant_epoch,
    )
    .map_err(|_| WhatsAppBootstrapError::EventHub)?;
    let event_publish_permit = event_access
        .publish_permit(
            &admission.module_registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| WhatsAppBootstrapError::EventHub)?;
    let delivery_intent_subscribe_permit = bind_delivery_intent_subscribe_permit(
        event_access
            .subscribe_permits(
                &admission.module_registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| WhatsAppBootstrapError::EventHub)?,
    )?;
    let event_connection = JetStreamClient::connect_runtime_with_jwt(
        event_hub_endpoint,
        identity,
        event_access.into_credential(),
    )
    .await
    .map_err(|_| WhatsAppBootstrapError::EventHub)?;
    control_channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id,
            runtime_generation,
            grant_epoch,
        })
        .map_err(|_| WhatsAppBootstrapError::Control)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .and_then(|_| control_channel.inner_mut().set_nonblocking(true))
        .map_err(|_| WhatsAppBootstrapError::Control)?;
    Ok(WhatsAppAdmittedRuntime {
        control_channel,
        durable,
        event_connection,
        event_publish_permit,
        delivery_intent_subscribe_permit,
        identity: WhatsAppRuntimeIdentity {
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
        },
        logical_owner_id: admission.logical_owner_id.clone(),
        account_id: settings.account_id.clone(),
        host_bridge_socket_path,
        host_bridge_route_binding,
    })
}

fn bind_delivery_intent_subscribe_permit(
    mut permits: Vec<RuntimeSubscribePermitV1>,
) -> Result<RuntimeSubscribePermitV1, WhatsAppBootstrapError> {
    if permits.len() != 1 {
        return Err(WhatsAppBootstrapError::EventHub);
    }
    let permit = permits.pop().ok_or(WhatsAppBootstrapError::EventHub)?;
    let expected = whatsapp_delivery_intent_execute_contract_reference_v1();
    if permit.contract().is_none_or(|contract| {
        contract.owner != expected.owner
            || contract.name != expected.name
            || contract.major != expected.major
            || contract.revision != expected.revision
            || contract.schema_sha256 != expected.schema_sha256
    }) {
        return Err(WhatsAppBootstrapError::EventHub);
    }
    Ok(permit)
}

impl WhatsAppAdmittedRuntime {
    pub async fn consume_next_delivery_intent(
        &self,
        now_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDeliveryIntentConsumeErrorV1> {
        let outcome = consume_next_whatsapp_delivery_intent_v1(
            &self.durable.delivery_intent_store(),
            &self.event_connection,
            &self.delivery_intent_subscribe_permit,
            &self.logical_owner_id,
            &WhatsAppDeliveryIntentResultContextV1 {
                runtime_instance_id: self.identity.runtime_instance_id.clone(),
                runtime_generation: self.identity.runtime_generation,
                completed_at_unix_seconds: now_unix_seconds,
                completed_at_nanos: 0,
            },
        )
        .await?;
        Ok(matches!(
            outcome,
            makosh_whatsapp_persistence::WhatsAppDeliveryIntentInboxOutcomeV1::Pending
                | makosh_whatsapp_persistence::WhatsAppDeliveryIntentInboxOutcomeV1::RouteNotFound
        ))
    }

    pub async fn process_next_delivery_intent(
        &mut self,
        now_unix_seconds: i64,
    ) -> Result<bool, WhatsAppDeliveryIntentWorkerErrorV1> {
        process_next_whatsapp_delivery_intent_v1(
            &mut self.control_channel,
            &self.durable,
            &WhatsAppDeliveryIntentWorkerContextV1 {
                runtime_instance_id: self.identity.runtime_instance_id.clone(),
                runtime_generation: self.identity.runtime_generation,
            },
            now_unix_seconds,
        )
        .await
    }

    pub async fn relay_delivery_intent_outbox(
        &self,
        now_unix_seconds: i64,
    ) -> Result<usize, WhatsAppDeliveryIntentOutboxRelayErrorV1> {
        relay_whatsapp_delivery_intent_outbox_once_v1(
            &self.durable.delivery_intent_store(),
            &self.event_connection,
            &self.event_publish_permit,
            now_unix_seconds,
        )
        .await
    }

    pub async fn try_handle_client_delivery(
        &mut self,
        requested_at_unix_seconds: i64,
    ) -> Result<bool, WhatsAppBootstrapError> {
        let Some((correlation_id, control_request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| WhatsAppBootstrapError::Control)?
        else {
            return Ok(false);
        };
        let request = match control_request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => request,
                _ => {
                    write_control_error(
                        &mut self.control_channel,
                        correlation_id,
                        "managed_runtime_control_invalid_client_delivery",
                    )?;
                    return Ok(true);
                }
            },
            _ => {
                write_control_error(
                    &mut self.control_channel,
                    correlation_id,
                    "managed_runtime_control_unexpected_request",
                )?;
                return Ok(true);
            }
        };
        let response = match crate::client_port::handle_client_request(
            self,
            &request.encode_to_vec(),
            requested_at_unix_seconds,
        )
        .await
        {
            Ok(payload) => {
                let response = ModuleClientResponseV1::decode(payload.as_slice())
                    .map_err(|_| WhatsAppBootstrapError::Admission)?;
                validate_module_client_response_v1(&response)
                    .map_err(|_| WhatsAppBootstrapError::Admission)?;
                response
            }
            Err(error) => ModuleClientResponseV1 {
                protocol_major: 1,
                request_id: request.request_id,
                response_payload: Vec::new(),
                error_code: match error {
                    crate::client_port::WhatsAppClientPortErrorV1::Protocol => "INVALID_ARGUMENT",
                    crate::client_port::WhatsAppClientPortErrorV1::Runtime => "RUNTIME_UNAVAILABLE",
                }
                .to_owned(),
            },
        };
        write_client_delivery_response(&mut self.control_channel, correlation_id, response)?;
        Ok(true)
    }

    pub async fn submit_command(
        &self,
        command: &WhatsAppProviderCommand,
        requested_at_unix_seconds: i64,
    ) -> Result<String, WhatsAppCommandQueueError> {
        if provider_command_account_id(command) != self.account_id {
            return Err(WhatsAppCommandQueueError::InvalidCommand);
        }
        let operation_id = provider_command_operation_id(command).to_owned();
        enqueue_provider_command(&self.durable, command, requested_at_unix_seconds).await?;
        Ok(operation_id)
    }

    pub async fn command_operation_status(
        &self,
        operation_id: &str,
    ) -> Result<Option<WhatsAppProviderCommandStatusV1>, WhatsAppCommandQueueError> {
        provider_command_status(&self.durable, operation_id)
            .await
            .map(|status| status.filter(|value| value.account_id == self.account_id))
    }

    pub async fn operational_query(
        &self,
        query: &WhatsAppOperationalQueryV1,
    ) -> Result<WhatsAppOperationalQueryResponseV1, WhatsAppOperationalQueryError> {
        if operational_query_account_id(query) != self.account_id {
            return Err(WhatsAppOperationalQueryError::AccountScope);
        }
        self.durable
            .execute_operational_query(query)
            .await
            .map_err(WhatsAppOperationalQueryError::Persistence)
    }

    pub async fn operational_replay(
        &self,
        request: &WhatsAppOperationalReplayRequestV1,
    ) -> Result<WhatsAppOperationalReplayResponseV1, WhatsAppOperationalReplayError> {
        if request.account_id != self.account_id {
            return Err(WhatsAppOperationalReplayError::AccountScope);
        }
        self.durable
            .replay_operational_events(request)
            .await
            .map_err(WhatsAppOperationalReplayError::Persistence)
    }

    /// Binds the exact host bridge endpoint staged by Kernel. The caller owns
    /// scheduling and shutdown; this runtime never invents a socket path or
    /// removes an existing endpoint.
    pub fn bind_host_bridge_listener(&self) -> Result<UnixListener, WhatsAppBootstrapError> {
        let listener = UnixListener::bind(&self.host_bridge_socket_path)
            .map_err(|_| WhatsAppBootstrapError::HostBridge)?;
        std::fs::set_permissions(
            &self.host_bridge_socket_path,
            std::fs::Permissions::from_mode(0o600),
        )
        .map_err(|_| WhatsAppBootstrapError::HostBridge)?;
        Ok(listener)
    }

    pub fn serve_host_bridge_once(
        &self,
        listener: &UnixListener,
        handle: &tokio::runtime::Handle,
    ) -> Result<(), WhatsAppBootstrapError> {
        let (stream, _) = listener
            .accept()
            .map_err(|_| WhatsAppBootstrapError::HostBridge)?;
        crate::host_bridge_transport::serve_connection(stream, self, handle)
            .map_err(|_| WhatsAppBootstrapError::HostBridge)
    }

    /// Serves one short-lived host connection when one is already pending.
    /// The process root owns scheduling and continues relaying the durable
    /// Communications outbox when no host request is waiting.
    pub fn try_serve_host_bridge_once(
        &self,
        listener: &UnixListener,
        handle: &tokio::runtime::Handle,
    ) -> Result<bool, WhatsAppBootstrapError> {
        let (stream, _) = match listener.accept() {
            Ok(value) => value,
            Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(false),
            Err(_) => return Err(WhatsAppBootstrapError::HostBridge),
        };
        crate::host_bridge_transport::serve_connection(stream, self, handle)
            .map_err(|_| WhatsAppBootstrapError::HostBridge)?;
        Ok(true)
    }

    pub async fn accept_host_observation(
        &self,
        envelope: &WhatsAppHostBridgeEnvelopeV1,
        recorded_at_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> Result<(), crate::WhatsAppHostIngressError> {
        if envelope.account_id != self.account_id {
            return Err(crate::WhatsAppHostIngressError::AccountScope);
        }
        accept_host_observation(
            &self.durable,
            &self.identity,
            envelope,
            recorded_at_unix_seconds,
            recorded_at_nanos,
        )
        .await
    }

    pub fn accepts_host_bridge_handshake(&self, handshake: &WhatsAppHostBridgeHandshakeV1) -> bool {
        handshake.route_binding_sha256 == self.host_bridge_route_binding
    }

    pub async fn claim_host_commands(
        &self,
        account_id: &str,
        host_claim_id: &str,
        now_unix_seconds: i64,
        lease_seconds: i64,
        limit: i64,
    ) -> Result<Vec<WhatsAppProviderCommand>, WhatsAppCommandQueueError> {
        if account_id != self.account_id {
            return Err(WhatsAppCommandQueueError::InvalidCommand);
        }
        claim_provider_commands(
            &self.durable,
            account_id,
            host_claim_id,
            now_unix_seconds,
            lease_seconds,
            limit,
        )
        .await
    }

    pub async fn relay_communications_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, crate::WhatsAppCommunicationsOutboxRelayError> {
        relay_communications_outbox_once(
            &self.durable,
            &self.event_connection,
            &self.event_publish_permit,
            published_at_unix_seconds,
        )
        .await
    }
}

fn write_client_delivery_response(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    response: ModuleClientResponseV1,
) -> Result<(), WhatsAppBootstrapError> {
    channel
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
        .map_err(|_| WhatsAppBootstrapError::Control)
}

fn write_control_error(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    error_code: &str,
) -> Result<(), WhatsAppBootstrapError> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: error_code.to_owned(),
            },
        )
        .map_err(|_| WhatsAppBootstrapError::Control)
}

fn host_bridge_route(
    configuration: &ManagedIntegrationHostBridgeConfigurationV1,
    admission: &WhatsAppRuntimeAdmission,
) -> Result<(String, [u8; 32]), WhatsAppBootstrapError> {
    validate_managed_integration_host_bridge_configuration(configuration)
        .map_err(|_| WhatsAppBootstrapError::Admission)?;
    if configuration.owner_id != admission.logical_owner_id
        || configuration.registration_id != admission.module_registration_id
        || configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.runtime_generation != admission.runtime_generation
        || configuration.grant_epoch != admission.grant_epoch
    {
        return Err(WhatsAppBootstrapError::Admission);
    }
    let route_binding = configuration
        .route_binding_sha256
        .as_slice()
        .try_into()
        .map_err(|_| WhatsAppBootstrapError::Admission)?;
    Ok((configuration.socket_path.clone(), route_binding))
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &WhatsAppRuntimeAdmission,
) -> Result<StorageBindingV1, WhatsAppBootstrapError> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(WhatsAppBootstrapError::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.module_registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| WhatsAppBootstrapError::Storage)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| WhatsAppBootstrapError::Storage)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| WhatsAppBootstrapError::Storage)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| WhatsAppBootstrapError::Storage)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| WhatsAppBootstrapError::Storage)?,
    )
    .map_err(|_| WhatsAppBootstrapError::Storage)?;
    StorageBindingV1::new(identity, fences, access).map_err(|_| WhatsAppBootstrapError::Storage)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admission() -> WhatsAppRuntimeAdmission {
        WhatsAppRuntimeAdmission {
            logical_owner_id: "whatsapp".to_owned(),
            module_registration_id: "whatsapp_runtime".to_owned(),
            runtime_instance_id: "whatsapp_runtime_1".to_owned(),
            runtime_generation: 2,
            grant_epoch: 3,
        }
    }

    #[test]
    fn accepts_only_a_kernel_fenced_whatsapp_host_route() {
        let configuration = ManagedIntegrationHostBridgeConfigurationV1 {
            major: 1,
            kernel_instance_id: "kernel_1".to_owned(),
            owner_id: "whatsapp".to_owned(),
            registration_id: "whatsapp_runtime".to_owned(),
            runtime_instance_id: "whatsapp_runtime_1".to_owned(),
            runtime_generation: 2,
            grant_epoch: 3,
            socket_path: "/private/tmp/makosh/whatsapp.sock".to_owned(),
            route_binding_sha256: vec![1; 32],
        };

        assert_eq!(
            host_bridge_route(&configuration, &admission()),
            Ok((configuration.socket_path.clone(), [1; 32]))
        );
    }

    #[test]
    fn rejects_a_stale_host_route() {
        let configuration = ManagedIntegrationHostBridgeConfigurationV1 {
            major: 1,
            kernel_instance_id: "kernel_1".to_owned(),
            owner_id: "whatsapp".to_owned(),
            registration_id: "whatsapp_runtime".to_owned(),
            runtime_instance_id: "whatsapp_runtime_1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 3,
            socket_path: "/private/tmp/makosh/whatsapp.sock".to_owned(),
            route_binding_sha256: vec![1; 32],
        };

        assert!(matches!(
            host_bridge_route(&configuration, &admission()),
            Err(WhatsAppBootstrapError::Admission)
        ));
    }
}
