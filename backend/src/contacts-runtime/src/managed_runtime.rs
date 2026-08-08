use std::os::unix::net::UnixStream;

use makosh_contacts_command_api::{
    CONTACTS_OWNER_ID_V1, bind_mail_address_book_provider_link_contract_reference_v1,
    upsert_contact_command_contract_reference_v1,
};
use makosh_contacts_mail_sync_source_api::contact_mail_sync_source_prepare_contract_reference_v1;
use makosh_contacts_persistence::{ContactsPersistenceErrorV1, ContactsPersistenceV1};
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::{
        ContractReferenceV1, ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
        ManagedStorageRuntimeConfigurationV1,
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
    command::{
        ContactsCommandErrorV1, ContactsCommandRuntimeContextV1, consume_contacts_command_once_v1,
    },
    event_outbox::{ContactsEventRelayErrorV1, relay_contacts_outbox_once_v1},
    provider_link::{
        ContactsProviderLinkErrorV1, ContactsProviderLinkRuntimeContextV1,
        consume_bind_mail_provider_link_once_v1,
    },
    source::{
        ContactsSourceErrorV1, ContactsSourceRuntimeContextV1,
        consume_contact_mail_sync_source_once_v1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactsRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactsManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(ContactsPersistenceErrorV1),
    Unavailable,
}

pub struct ContactsManagedRuntimeV1 {
    admission: ContactsRuntimeAdmissionV1,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: ContactsPersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    command_subscription: RuntimeSubscribePermitV1,
    provider_link_subscription: RuntimeSubscribePermitV1,
    source_subscription: RuntimeSubscribePermitV1,
}

impl ContactsManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &ContactsRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, ContactsManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(ContactsManagedRuntimeErrorV1::Admission);
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
            .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password =
            std::str::from_utf8(&password).map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?;
        let persistence = ContactsPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(ContactsManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(ContactsManagedRuntimeErrorV1::Persistence)?;

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
        .map_err(|_| ContactsManagedRuntimeErrorV1::EventUnavailable)?;
        let event_identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?;
        let mut subscriptions = event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?;
        let command_subscription = take_exact_subscription(
            &mut subscriptions,
            &upsert_contact_command_contract_reference_v1(),
        )?;
        let provider_link_subscription = take_exact_subscription(
            &mut subscriptions,
            &bind_mail_address_book_provider_link_contract_reference_v1(),
        )?;
        let source_subscription = take_exact_subscription(
            &mut subscriptions,
            &contact_mail_sync_source_prepare_contract_reference_v1(),
        )?;
        if !subscriptions.is_empty() {
            return Err(ContactsManagedRuntimeErrorV1::Admission);
        }
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            event_identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| ContactsManagedRuntimeErrorV1::EventUnavailable)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ContactsManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control_channel,
            persistence,
            event_connection,
            event_publish_permit,
            command_subscription,
            provider_link_subscription,
            source_subscription,
        })
    }

    pub fn pump_control_once(&mut self) -> Result<bool, ContactsManagedRuntimeErrorV1> {
        let Some((correlation_id, _request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| ContactsManagedRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_unexpected_request".to_owned(),
                },
            )
            .map_err(|_| ContactsManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn consume_command_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, ContactsManagedRuntimeErrorV1> {
        consume_contacts_command_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.command_subscription,
            &ContactsCommandRuntimeContextV1 {
                logical_owner_id: &self.admission.logical_human_owner_id,
                runtime_instance_id: &self.admission.runtime_instance_id,
                runtime_generation: self.admission.runtime_generation,
                now_unix_millis,
            },
        )
        .await
        .map_err(command_error)
    }

    pub async fn relay_outbox_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, ContactsManagedRuntimeErrorV1> {
        relay_contacts_outbox_once_v1(
            &self.persistence,
            &self.admission.logical_human_owner_id,
            &self.event_connection,
            &self.event_publish_permit,
            now_unix_millis,
        )
        .await
        .map_err(event_relay_error)
    }

    pub async fn consume_provider_link_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, ContactsManagedRuntimeErrorV1> {
        consume_bind_mail_provider_link_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.provider_link_subscription,
            &ContactsProviderLinkRuntimeContextV1 {
                logical_owner_id: &self.admission.logical_human_owner_id,
                runtime_instance_id: &self.admission.runtime_instance_id,
                runtime_generation: self.admission.runtime_generation,
                now_unix_millis,
            },
        )
        .await
        .map_err(provider_link_error)
    }

    pub async fn consume_source_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, ContactsManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| ContactsManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = consume_contact_mail_sync_source_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.source_subscription,
            &mut self.control_channel,
            &mut dispatcher,
            &ContactsSourceRuntimeContextV1 {
                logical_owner_id: &self.admission.logical_human_owner_id,
                runtime_instance_id: &self.admission.runtime_instance_id,
                runtime_generation: self.admission.runtime_generation,
                now_unix_millis,
            },
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ContactsManagedRuntimeErrorV1::Unavailable)?;
        result.map_err(source_error)
    }
}

fn take_exact_subscription(
    permits: &mut Vec<RuntimeSubscribePermitV1>,
    contract: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, ContactsManagedRuntimeErrorV1> {
    let index = permits
        .iter()
        .position(|permit| {
            permit.contract().is_some_and(|actual| {
                actual.owner == contract.owner
                    && actual.name == contract.name
                    && actual.major == contract.major
                    && actual.revision == contract.revision
                    && actual.schema_sha256 == contract.schema_sha256
            })
        })
        .ok_or(ContactsManagedRuntimeErrorV1::Admission)?;
    Ok(permits.remove(index))
}

fn validate_admission(
    admission: &ContactsRuntimeAdmissionV1,
) -> Result<(), ContactsManagedRuntimeErrorV1> {
    if admission.logical_owner_id != CONTACTS_OWNER_ID_V1
        || admission.logical_human_owner_id.is_empty()
        || admission.logical_human_owner_id == admission.logical_owner_id
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(ContactsManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &ContactsRuntimeAdmissionV1,
) -> Result<(), ContactsManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| ContactsManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| ContactsManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(ContactsManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &ContactsRuntimeAdmissionV1,
) -> Result<(), ContactsManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| ContactsManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| ContactsManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ContactsManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(ContactsManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &ContactsRuntimeAdmissionV1,
) -> Result<StorageBindingV1, ContactsManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != CONTACTS_OWNER_ID_V1
        || configuration.owner != CONTACTS_OWNER_ID_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(ContactsManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| ContactsManagedRuntimeErrorV1::Admission)
}

fn command_error(error: ContactsCommandErrorV1) -> ContactsManagedRuntimeErrorV1 {
    match error {
        ContactsCommandErrorV1::InvalidEnvelope | ContactsCommandErrorV1::InvalidPayload => {
            ContactsManagedRuntimeErrorV1::EventContract
        }
        ContactsCommandErrorV1::Persistence(error) => {
            ContactsManagedRuntimeErrorV1::Persistence(error)
        }
        ContactsCommandErrorV1::EventUnavailable => ContactsManagedRuntimeErrorV1::EventUnavailable,
    }
}

fn source_error(error: ContactsSourceErrorV1) -> ContactsManagedRuntimeErrorV1 {
    match error {
        ContactsSourceErrorV1::InvalidEnvelope | ContactsSourceErrorV1::InvalidPayload => {
            ContactsManagedRuntimeErrorV1::EventContract
        }
        ContactsSourceErrorV1::Persistence(error) => {
            ContactsManagedRuntimeErrorV1::Persistence(error)
        }
        ContactsSourceErrorV1::EventUnavailable | ContactsSourceErrorV1::BlobUnavailable => {
            ContactsManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn provider_link_error(error: ContactsProviderLinkErrorV1) -> ContactsManagedRuntimeErrorV1 {
    match error {
        ContactsProviderLinkErrorV1::InvalidEnvelope
        | ContactsProviderLinkErrorV1::InvalidPayload => {
            ContactsManagedRuntimeErrorV1::EventContract
        }
        ContactsProviderLinkErrorV1::Persistence(error) => {
            ContactsManagedRuntimeErrorV1::Persistence(error)
        }
        ContactsProviderLinkErrorV1::EventUnavailable => {
            ContactsManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn event_relay_error(error: ContactsEventRelayErrorV1) -> ContactsManagedRuntimeErrorV1 {
    match error {
        ContactsEventRelayErrorV1::InvalidTimestamp => ContactsManagedRuntimeErrorV1::EventContract,
        ContactsEventRelayErrorV1::Persistence(error) => {
            ContactsManagedRuntimeErrorV1::Persistence(error)
        }
        ContactsEventRelayErrorV1::EventUnavailable => {
            ContactsManagedRuntimeErrorV1::EventUnavailable
        }
    }
}
