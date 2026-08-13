use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    request_managed_runtime_event_access_v2,
};
use makosh_organizations_api::ORGANIZATIONS_OWNER_ID_V1;
use makosh_organizations_persistence::{
    OrganizationsPersistenceErrorV1, OrganizationsPersistenceV1,
};
use makosh_runtime_protocol::{
    managed_control::ManagedControlChannelV2,
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

use crate::{
    OrganizationsClientRuntimeContextV1, dispatch_organizations_client_request_v1,
    event_outbox::{OrganizationsEventRelayErrorV1, relay_organizations_outbox_once_v1},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizationsRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizationsManagedRuntimeErrorV1 {
    Admission,
    EventUnavailable,
    Persistence(OrganizationsPersistenceErrorV1),
    Unavailable,
}

pub struct OrganizationsManagedRuntimeV1 {
    admission: OrganizationsRuntimeAdmissionV1,
    runtime_instance_id: [u8; 16],
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: OrganizationsPersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
}

impl OrganizationsManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &OrganizationsRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, OrganizationsManagedRuntimeErrorV1> {
        let runtime_instance_id = validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(OrganizationsManagedRuntimeErrorV1::Admission);
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
            .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?;
        let persistence = OrganizationsPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(OrganizationsManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(OrganizationsManagedRuntimeErrorV1::Persistence)?;

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
        .map_err(|_| OrganizationsManagedRuntimeErrorV1::EventUnavailable)?;
        let event_identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?;
        let subscriptions = event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?;
        if !subscriptions.is_empty() {
            return Err(OrganizationsManagedRuntimeErrorV1::Admission);
        }
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            event_identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| OrganizationsManagedRuntimeErrorV1::EventUnavailable)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| OrganizationsManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            runtime_instance_id,
            control_channel,
            persistence,
            event_connection,
            event_publish_permit,
        })
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, OrganizationsManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| OrganizationsManagedRuntimeErrorV1::Unavailable)?
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
        let response = dispatch_organizations_client_request_v1(
            &self.persistence,
            &self.admission.logical_human_owner_id,
            request,
            OrganizationsClientRuntimeContextV1 {
                runtime_instance_id: self.runtime_instance_id,
                runtime_generation: self.admission.runtime_generation,
                now_unix_millis,
            },
        )
        .await;
        if validate_module_client_response_v1(&response).is_err() {
            return Err(OrganizationsManagedRuntimeErrorV1::Unavailable);
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
            .map_err(|_| OrganizationsManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn relay_outbox_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, OrganizationsManagedRuntimeErrorV1> {
        relay_organizations_outbox_once_v1(
            &self.persistence,
            &self.admission.logical_human_owner_id,
            &self.event_connection,
            &self.event_publish_permit,
            now_unix_millis,
        )
        .await
        .map_err(relay_error)
    }

    fn write_control_error(
        &mut self,
        correlation_id: [u8; 16],
        error_code: &str,
    ) -> Result<(), OrganizationsManagedRuntimeErrorV1> {
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: error_code.to_owned(),
                },
            )
            .map_err(|_| OrganizationsManagedRuntimeErrorV1::Unavailable)
    }
}

fn validate_admission(
    admission: &OrganizationsRuntimeAdmissionV1,
) -> Result<[u8; 16], OrganizationsManagedRuntimeErrorV1> {
    if admission.logical_owner_id != ORGANIZATIONS_OWNER_ID_V1
        || admission.logical_human_owner_id.is_empty()
        || admission.logical_human_owner_id == admission.logical_owner_id
        || admission.registration_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(OrganizationsManagedRuntimeErrorV1::Admission);
    }
    runtime_source_reference(&admission.runtime_instance_id)
        .ok_or(OrganizationsManagedRuntimeErrorV1::Admission)
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &OrganizationsRuntimeAdmissionV1,
) -> Result<(), OrganizationsManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| OrganizationsManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| OrganizationsManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(OrganizationsManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &OrganizationsRuntimeAdmissionV1,
) -> Result<(), OrganizationsManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| OrganizationsManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| OrganizationsManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, OrganizationsManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(OrganizationsManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &OrganizationsRuntimeAdmissionV1,
) -> Result<StorageBindingV1, OrganizationsManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != ORGANIZATIONS_OWNER_ID_V1
        || configuration.owner != ORGANIZATIONS_OWNER_ID_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(OrganizationsManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| OrganizationsManagedRuntimeErrorV1::Admission)
}

fn runtime_source_reference(value: &str) -> Option<[u8; 16]> {
    if value.len() != 32 {
        return None;
    }
    let mut bytes = [0; 16];
    for (index, item) in bytes.iter_mut().enumerate() {
        *item = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    bytes.iter().any(|byte| *byte != 0).then_some(bytes)
}

fn relay_error(error: OrganizationsEventRelayErrorV1) -> OrganizationsManagedRuntimeErrorV1 {
    match error {
        OrganizationsEventRelayErrorV1::InvalidTimestamp => {
            OrganizationsManagedRuntimeErrorV1::Admission
        }
        OrganizationsEventRelayErrorV1::Persistence(error) => {
            OrganizationsManagedRuntimeErrorV1::Persistence(error)
        }
        OrganizationsEventRelayErrorV1::EventUnavailable => {
            OrganizationsManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_requires_exact_owner_and_runtime_source() {
        let valid = OrganizationsRuntimeAdmissionV1 {
            logical_owner_id: ORGANIZATIONS_OWNER_ID_V1.to_owned(),
            logical_human_owner_id: "owner-1".to_owned(),
            registration_id: "registration".to_owned(),
            runtime_instance_id: "01010101010101010101010101010101".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        };
        assert_eq!(validate_admission(&valid), Ok([1; 16]));
        let mut invalid = valid;
        invalid.logical_owner_id = "calendar".to_owned();
        assert_eq!(
            validate_admission(&invalid),
            Err(OrganizationsManagedRuntimeErrorV1::Admission)
        );
    }
}
