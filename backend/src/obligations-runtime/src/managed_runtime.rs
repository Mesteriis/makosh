use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_obligations_api::{
    OBLIGATIONS_OWNER_ID_V1, create_obligation_from_reviewed_candidate_contract_reference_v1,
};
use makosh_obligations_persistence::{ObligationsPersistenceErrorV1, ObligationsPersistenceV1};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryResponseV1,
        ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
        ManagedStorageRuntimeConfigurationV1, managed_runtime_control_request_v1::Operation,
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
    client::dispatch_obligations_client_request_v1,
    command::{
        ObligationsCommandErrorV1, ObligationsCommandRuntimeContextV1,
        consume_obligation_command_once_v1, recover_obligation_command_once_v1,
    },
    event_outbox::{ObligationsEventRelayErrorV1, relay_obligations_outbox_once_v1},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationsRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObligationsManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(ObligationsPersistenceErrorV1),
    Unavailable,
}

pub struct ObligationsManagedRuntimeV1 {
    admission: ObligationsRuntimeAdmissionV1,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: ObligationsPersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    command_subscription: RuntimeSubscribePermitV1,
}

impl ObligationsManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &ObligationsRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, ObligationsManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(ObligationsManagedRuntimeErrorV1::Admission);
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
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?;
        let persistence = ObligationsPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(ObligationsManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(ObligationsManagedRuntimeErrorV1::Persistence)?;

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
        .map_err(|_| ObligationsManagedRuntimeErrorV1::EventUnavailable)?;
        let event_identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?;
        let command_subscription = exact_subscription(
            event_access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?,
            &create_obligation_from_reviewed_candidate_contract_reference_v1(),
        )?;
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            event_identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| ObligationsManagedRuntimeErrorV1::EventUnavailable)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control_channel,
            persistence,
            event_connection,
            event_publish_permit,
            command_subscription,
        })
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, ObligationsManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            self.write_control_error(correlation_id, "managed_runtime_control_unexpected_request")?;
            return Ok(true);
        };
        let Some(request) = delivery
            .request
            .filter(|request| validate_module_client_request_v1(request).is_ok())
        else {
            self.write_control_error(
                correlation_id,
                "managed_runtime_control_invalid_client_delivery",
            )?;
            return Ok(true);
        };
        let response = dispatch_obligations_client_request_v1(
            &self.persistence,
            &self.admission.runtime_instance_id,
            self.admission.runtime_generation,
            &self.admission.logical_human_owner_id,
            request,
            now_unix_millis,
        )
        .await;
        if validate_module_client_response_v1(&response).is_err() {
            return Err(ObligationsManagedRuntimeErrorV1::Unavailable);
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
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    fn write_control_error(
        &mut self,
        correlation_id: [u8; 16],
        error_code: &str,
    ) -> Result<(), ObligationsManagedRuntimeErrorV1> {
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: error_code.to_owned(),
                },
            )
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Unavailable)
    }

    pub async fn recover_command_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, ObligationsManagedRuntimeErrorV1> {
        self.with_blocking_control(now_unix_millis, false).await
    }

    pub async fn consume_command_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, ObligationsManagedRuntimeErrorV1> {
        self.with_blocking_control(now_unix_millis, true).await
    }

    async fn with_blocking_control(
        &mut self,
        now_unix_millis: i64,
        consume: bool,
    ) -> Result<bool, ObligationsManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let context = ObligationsCommandRuntimeContextV1 {
            logical_owner_id: &self.admission.logical_human_owner_id,
            runtime_instance_id: &self.admission.runtime_instance_id,
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis,
        };
        let result = if consume {
            consume_obligation_command_once_v1(
                &self.persistence,
                &self.event_connection,
                &self.command_subscription,
                &mut self.control_channel,
                &mut dispatcher,
                &context,
            )
            .await
        } else {
            recover_obligation_command_once_v1(
                &self.persistence,
                &mut self.control_channel,
                &mut dispatcher,
                &context,
            )
            .await
        }
        .map_err(command_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Unavailable)?;
        result
    }

    pub async fn relay_outbox_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, ObligationsManagedRuntimeErrorV1> {
        relay_obligations_outbox_once_v1(
            &self.persistence,
            &self.admission.logical_human_owner_id,
            &self.event_connection,
            &self.event_publish_permit,
            now_unix_millis,
        )
        .await
        .map_err(event_relay_error)
    }
}

fn exact_subscription(
    permits: Vec<RuntimeSubscribePermitV1>,
    contract: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, ObligationsManagedRuntimeErrorV1> {
    if permits.len() != 1 {
        return Err(ObligationsManagedRuntimeErrorV1::Admission);
    }
    let permit = permits
        .into_iter()
        .next()
        .ok_or(ObligationsManagedRuntimeErrorV1::Admission)?;
    if permit.contract().is_none_or(|actual| {
        actual.owner != contract.owner
            || actual.name != contract.name
            || actual.major != contract.major
            || actual.revision != contract.revision
            || actual.schema_sha256 != contract.schema_sha256
    }) {
        return Err(ObligationsManagedRuntimeErrorV1::Admission);
    }
    Ok(permit)
}

fn validate_admission(
    admission: &ObligationsRuntimeAdmissionV1,
) -> Result<(), ObligationsManagedRuntimeErrorV1> {
    if admission.logical_owner_id != OBLIGATIONS_OWNER_ID_V1
        || admission.logical_human_owner_id.is_empty()
        || admission.logical_human_owner_id == admission.logical_owner_id
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(ObligationsManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &ObligationsRuntimeAdmissionV1,
) -> Result<(), ObligationsManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| ObligationsManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| ObligationsManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(ObligationsManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &ObligationsRuntimeAdmissionV1,
) -> Result<(), ObligationsManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| ObligationsManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| ObligationsManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ObligationsManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(ObligationsManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &ObligationsRuntimeAdmissionV1,
) -> Result<StorageBindingV1, ObligationsManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != OBLIGATIONS_OWNER_ID_V1
        || configuration.owner != OBLIGATIONS_OWNER_ID_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(ObligationsManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| ObligationsManagedRuntimeErrorV1::Admission)
}

fn command_error(error: ObligationsCommandErrorV1) -> ObligationsManagedRuntimeErrorV1 {
    match error {
        ObligationsCommandErrorV1::InvalidEnvelope
        | ObligationsCommandErrorV1::InvalidPayload
        | ObligationsCommandErrorV1::Blob(ObligationsBlobErrorV1::InvalidReceipt) => {
            ObligationsManagedRuntimeErrorV1::EventContract
        }
        ObligationsCommandErrorV1::Blob(ObligationsBlobErrorV1::Unavailable) => {
            ObligationsManagedRuntimeErrorV1::Unavailable
        }
        ObligationsCommandErrorV1::Persistence(error) => {
            ObligationsManagedRuntimeErrorV1::Persistence(error)
        }
        ObligationsCommandErrorV1::EventUnavailable => {
            ObligationsManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn event_relay_error(error: ObligationsEventRelayErrorV1) -> ObligationsManagedRuntimeErrorV1 {
    match error {
        ObligationsEventRelayErrorV1::InvalidTimestamp => {
            ObligationsManagedRuntimeErrorV1::EventContract
        }
        ObligationsEventRelayErrorV1::Persistence(error) => {
            ObligationsManagedRuntimeErrorV1::Persistence(error)
        }
        ObligationsEventRelayErrorV1::EventUnavailable => {
            ObligationsManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

use crate::blob::ObligationsBlobErrorV1;
