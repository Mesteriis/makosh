use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    DurableSubjectV1, JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity,
    RuntimePublishPermitV1, RuntimeSubscribePermitV1, StreamKindV1,
    request_managed_runtime_event_access_v2,
};
use makosh_persons_api::{
    PERSONS_OWNER_ID_V1, persons_command_contract_reference_v1,
    persons_command_rejected_contract_reference_v1,
    persons_command_succeeded_contract_reference_v1, persons_owner_event_contract_reference_v1,
    persons_review_candidate_contract_reference_v1,
};
use makosh_persons_core::MAX_REVIEW_CANDIDATES_PER_COMMAND_V1;
use makosh_persons_persistence::{
    PERSONS_OUTBOX_READ_LIMIT_V1, PersonsPersistenceErrorV1, PersonsPersistenceV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlTransportErrorV2},
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
    consumer::{PersonsCommandConsumerErrorV1, consume_persons_command_once_v1},
    dispatch_persons_client_request_v1,
    event_outbox::{PersonsEventRelayErrorV1, relay_persons_outbox_once_v1},
    execution::PersonsCommandRuntimeContextV1,
};

// One source observation can emit a terminal result, Person/source owner
// events, and at most the core-bound Review candidates. Keep the fair drain
// coupled to that public core bound and below one persistence read page.
const MAX_OUTBOX_RELAY_PER_SERVICE_V1: usize = MAX_REVIEW_CANDIDATES_PER_COMMAND_V1 + 3;
const _: () = assert!(MAX_OUTBOX_RELAY_PER_SERVICE_V1 <= PERSONS_OUTBOX_READ_LIMIT_V1 as usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonsRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(PersonsPersistenceErrorV1),
    ControlClosed,
    Unavailable,
}

pub struct PersonsManagedRuntimeV1 {
    admission: PersonsRuntimeAdmissionV1,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: PersonsPersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    command_subscription: RuntimeSubscribePermitV1,
}

impl PersonsManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &PersonsRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, PersonsManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(PersonsManagedRuntimeErrorV1::Admission);
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
            .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password =
            std::str::from_utf8(&password).map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?;
        let mut control_channel = leases.into_route_port().into_channel();
        let persistence = tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control_channel) => return Err(closed),
            persistence = PersonsPersistenceV1::connect_runtime(
                &binding,
                &storage_configuration.database_id,
                &storage_configuration.pgbouncer_host,
                storage_configuration.pgbouncer_port,
                password,
            ) => persistence.map_err(PersonsManagedRuntimeErrorV1::Persistence)?,
        };
        tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control_channel) => return Err(closed),
            ready = persistence.verify_storage_ready() => {
                ready.map_err(PersonsManagedRuntimeErrorV1::Persistence)?;
            }
        }

        let event_access = request_managed_runtime_event_access_v2(
            &mut control_channel,
            &storage_configuration.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            event_credential_revision,
        )
        .map_err(|_| PersonsManagedRuntimeErrorV1::EventUnavailable)?;
        let event_identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?;
        validate_exact_publish_permit(&event_publish_permit)?;
        let mut subscriptions = event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?;
        let command_subscription =
            take_exact_subscription(&mut subscriptions, &persons_command_contract_reference_v1())?;
        if !subscriptions.is_empty() {
            return Err(PersonsManagedRuntimeErrorV1::Admission);
        }
        let event_connection = tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control_channel) => return Err(closed),
            connection = JetStreamClient::connect_runtime_with_jwt(
                event_hub_endpoint,
                event_identity,
                event_access.into_credential(),
            ) => connection.map_err(|_| PersonsManagedRuntimeErrorV1::EventUnavailable)?,
        };
        tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control_channel) => return Err(closed),
            consumer = event_connection.open_pull_consumer(&command_subscription) => {
                consumer.map_err(|_| PersonsManagedRuntimeErrorV1::EventContract)?;
            }
        }
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| PersonsManagedRuntimeErrorV1::Unavailable)?;
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
    ) -> Result<bool, PersonsManagedRuntimeErrorV1> {
        pump_control(
            &mut self.control_channel,
            &self.persistence,
            &self.admission,
            now_unix_millis,
        )
        .await
    }

    pub async fn service_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, PersonsManagedRuntimeErrorV1> {
        let runtime = PersonsCommandRuntimeContextV1 {
            logical_owner_id: self.admission.logical_human_owner_id.clone(),
            runtime_instance_id: self.admission.runtime_instance_id.clone(),
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis,
        };
        let control_channel = &mut self.control_channel;
        let persistence = &self.persistence;
        let event_connection = &self.event_connection;
        let command_subscription = &self.command_subscription;
        let event_publish_permit = &self.event_publish_permit;
        let logical_owner_id = self.admission.logical_human_owner_id.clone();
        let consumed = tokio::select! {
            biased;
            control = wait_for_control(control_channel, persistence, &self.admission, now_unix_millis) => control,
            consumed = consume_persons_command_once_v1(
                persistence,
                event_connection,
                command_subscription,
                &runtime,
            ) => consumed.map_err(command_error),
        }?;
        let mut relayed = false;
        for _ in 0..MAX_OUTBOX_RELAY_PER_SERVICE_V1 {
            let progressed = tokio::select! {
                biased;
                control = wait_for_control(control_channel, persistence, &self.admission, now_unix_millis) => control,
                relayed = relay_persons_outbox_once_v1(
                    persistence,
                    &logical_owner_id,
                    event_connection,
                    event_publish_permit,
                    now_unix_millis,
                ) => relayed.map_err(event_relay_error),
            }?;
            if !progressed {
                break;
            }
            relayed = true;
        }
        Ok(consumed || relayed)
    }

    #[must_use]
    pub fn generation(&self) -> u64 {
        self.admission.runtime_generation
    }
}

async fn wait_for_control(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    persistence: &PersonsPersistenceV1,
    admission: &PersonsRuntimeAdmissionV1,
    now_unix_millis: i64,
) -> Result<bool, PersonsManagedRuntimeErrorV1> {
    loop {
        if pump_control(channel, persistence, admission, now_unix_millis).await? {
            return Ok(true);
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}

async fn pump_control(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    persistence: &PersonsPersistenceV1,
    admission: &PersonsRuntimeAdmissionV1,
    now_unix_millis: i64,
) -> Result<bool, PersonsManagedRuntimeErrorV1> {
    let Some((correlation_id, request)) = channel.try_receive_request().map_err(|error| {
        if matches!(error, ManagedControlTransportErrorV2::PeerClosed) {
            PersonsManagedRuntimeErrorV1::ControlClosed
        } else {
            PersonsManagedRuntimeErrorV1::Unavailable
        }
    })?
    else {
        return Ok(false);
    };
    if let Some(Operation::ClientDelivery(delivery)) = request.operation {
        let Some(request) = delivery
            .request
            .filter(|request| validate_module_client_request_v1(request).is_ok())
        else {
            return write_control_error(
                channel,
                correlation_id,
                "managed_runtime_control_invalid_client_delivery",
            );
        };
        let response = dispatch_persons_client_request_v1(
            persistence,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            &admission.logical_human_owner_id,
            request,
            now_unix_millis,
        )
        .await;
        validate_module_client_response_v1(&response)
            .map_err(|_| PersonsManagedRuntimeErrorV1::Unavailable)?;
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
            .map_err(|_| PersonsManagedRuntimeErrorV1::Unavailable)?;
        return Ok(true);
    }
    write_control_error(
        channel,
        correlation_id,
        "managed_runtime_control_unexpected_request",
    )
}

fn write_control_error(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; 16],
    error_code: &str,
) -> Result<bool, PersonsManagedRuntimeErrorV1> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: error_code.to_owned(),
            },
        )
        .map_err(|_| PersonsManagedRuntimeErrorV1::Unavailable)?;
    Ok(true)
}

fn take_exact_subscription(
    permits: &mut Vec<RuntimeSubscribePermitV1>,
    contract: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, PersonsManagedRuntimeErrorV1> {
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
        .ok_or(PersonsManagedRuntimeErrorV1::Admission)?;
    Ok(permits.remove(index))
}

fn validate_exact_publish_permit(
    permit: &RuntimePublishPermitV1,
) -> Result<(), PersonsManagedRuntimeErrorV1> {
    let contracts = [
        (
            StreamKindV1::Result,
            persons_command_rejected_contract_reference_v1(),
        ),
        (
            StreamKindV1::Result,
            persons_command_succeeded_contract_reference_v1(),
        ),
        (
            StreamKindV1::Event,
            persons_owner_event_contract_reference_v1(),
        ),
        (
            StreamKindV1::Event,
            persons_review_candidate_contract_reference_v1(),
        ),
    ];
    let subjects = contracts
        .into_iter()
        .map(|(kind, contract)| {
            DurableSubjectV1::new(kind, contract.owner, contract.name, contract.major)
                .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if permit.permits_exact_subjects(&subjects) {
        Ok(())
    } else {
        Err(PersonsManagedRuntimeErrorV1::Admission)
    }
}

fn validate_admission(
    admission: &PersonsRuntimeAdmissionV1,
) -> Result<(), PersonsManagedRuntimeErrorV1> {
    if admission.logical_owner_id != PERSONS_OWNER_ID_V1
        || admission.logical_human_owner_id.is_empty()
        || admission.logical_human_owner_id == admission.logical_owner_id
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(PersonsManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &PersonsRuntimeAdmissionV1,
) -> Result<(), PersonsManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| PersonsManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| PersonsManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(PersonsManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &PersonsRuntimeAdmissionV1,
) -> Result<(), PersonsManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| PersonsManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| PersonsManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, PersonsManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if control_peer_closed(leases.route_port_mut().channel_mut())? {
            return Err(PersonsManagedRuntimeErrorV1::ControlClosed);
        }
        if attempt < 19 {
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(100);
            loop {
                if control_peer_closed(leases.route_port_mut().channel_mut())? {
                    return Err(PersonsManagedRuntimeErrorV1::ControlClosed);
                }
                if tokio::time::Instant::now() >= deadline {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        }
    }
    Err(PersonsManagedRuntimeErrorV1::Unavailable)
}

async fn wait_for_bootstrap_control_close(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> PersonsManagedRuntimeErrorV1 {
    loop {
        match control_peer_closed(channel) {
            Ok(true) => return PersonsManagedRuntimeErrorV1::ControlClosed,
            Ok(false) => tokio::time::sleep(std::time::Duration::from_millis(5)).await,
            Err(error) => return error,
        }
    }
}

fn control_peer_closed(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<bool, PersonsManagedRuntimeErrorV1> {
    channel
        .peer_closed_preserving_frames()
        .map_err(|_| PersonsManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &PersonsRuntimeAdmissionV1,
) -> Result<StorageBindingV1, PersonsManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != PERSONS_OWNER_ID_V1
        || configuration.owner != PERSONS_OWNER_ID_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(PersonsManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| PersonsManagedRuntimeErrorV1::Admission)
}

fn command_error(error: PersonsCommandConsumerErrorV1) -> PersonsManagedRuntimeErrorV1 {
    match error {
        PersonsCommandConsumerErrorV1::InvalidEnvelope
        | PersonsCommandConsumerErrorV1::InvalidPayload => {
            PersonsManagedRuntimeErrorV1::EventContract
        }
        PersonsCommandConsumerErrorV1::Persistence(error) => {
            PersonsManagedRuntimeErrorV1::Persistence(error)
        }
        PersonsCommandConsumerErrorV1::EventUnavailable => {
            PersonsManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn event_relay_error(error: PersonsEventRelayErrorV1) -> PersonsManagedRuntimeErrorV1 {
    match error {
        PersonsEventRelayErrorV1::InvalidTimestamp => PersonsManagedRuntimeErrorV1::EventContract,
        PersonsEventRelayErrorV1::Persistence(error) => {
            PersonsManagedRuntimeErrorV1::Persistence(error)
        }
        PersonsEventRelayErrorV1::EventUnavailable => {
            PersonsManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fair_relay_bound_tracks_core_candidate_bound_and_one_storage_page() {
        assert_eq!(
            MAX_OUTBOX_RELAY_PER_SERVICE_V1,
            MAX_REVIEW_CANDIDATES_PER_COMMAND_V1 + 3
        );
        assert!(MAX_OUTBOX_RELAY_PER_SERVICE_V1 <= PERSONS_OUTBOX_READ_LIMIT_V1 as usize);
    }

    #[test]
    fn admission_rejects_wrong_owner_and_stale_fences() {
        let valid = PersonsRuntimeAdmissionV1 {
            logical_owner_id: "persons".to_owned(),
            logical_human_owner_id: "owner-a".to_owned(),
            registration_id: "persons_registration".to_owned(),
            runtime_instance_id: "persons-runtime-1".to_owned(),
            runtime_generation: 7,
            grant_epoch: 8,
        };
        assert_eq!(validate_admission(&valid), Ok(()));
        for invalid in [
            PersonsRuntimeAdmissionV1 {
                logical_owner_id: "contacts".to_owned(),
                ..valid.clone()
            },
            PersonsRuntimeAdmissionV1 {
                runtime_generation: 0,
                ..valid.clone()
            },
            PersonsRuntimeAdmissionV1 {
                grant_epoch: 0,
                ..valid
            },
        ] {
            assert_eq!(
                validate_admission(&invalid),
                Err(PersonsManagedRuntimeErrorV1::Admission)
            );
        }
    }

    #[test]
    fn publish_permit_requires_exact_four_persons_contracts() {
        let expected = [
            (
                StreamKindV1::Result,
                persons_command_rejected_contract_reference_v1(),
            ),
            (
                StreamKindV1::Result,
                persons_command_succeeded_contract_reference_v1(),
            ),
            (
                StreamKindV1::Event,
                persons_owner_event_contract_reference_v1(),
            ),
            (
                StreamKindV1::Event,
                persons_review_candidate_contract_reference_v1(),
            ),
        ]
        .into_iter()
        .map(|(kind, contract)| {
            DurableSubjectV1::new(kind, contract.owner, contract.name, contract.major)
                .expect("subject")
        })
        .collect::<Vec<_>>();
        let exact = RuntimePublishPermitV1::new("registration", "runtime", 1, 1, expected.clone())
            .expect("exact permit");
        assert_eq!(validate_exact_publish_permit(&exact), Ok(()));
        let missing =
            RuntimePublishPermitV1::new("registration", "runtime", 1, 1, expected[..3].to_vec())
                .expect("missing permit");
        assert_eq!(
            validate_exact_publish_permit(&missing),
            Err(PersonsManagedRuntimeErrorV1::Admission)
        );
        let mut with_extra = expected;
        with_extra.push(
            DurableSubjectV1::new(StreamKindV1::Event, "persons", "extra", 1)
                .expect("extra subject"),
        );
        let extra = RuntimePublishPermitV1::new("registration", "runtime", 1, 1, with_extra)
            .expect("extra permit");
        assert_eq!(
            validate_exact_publish_permit(&extra),
            Err(PersonsManagedRuntimeErrorV1::Admission)
        );
    }

    #[test]
    fn closed_control_peer_is_distinct_from_retryable_unavailability() {
        let (runtime, kernel) = UnixStream::pair().expect("control pair");
        runtime.set_nonblocking(true).expect("nonblocking control");
        let mut channel = ManagedControlChannelV2::new(runtime);
        drop(kernel);
        assert!(matches!(
            channel.try_receive_request(),
            Err(ManagedControlTransportErrorV2::PeerClosed)
        ));
    }

    #[test]
    fn storage_binding_rejects_missing_or_stale_owner_runtime_and_fences() {
        let admission = PersonsRuntimeAdmissionV1 {
            logical_owner_id: PERSONS_OWNER_ID_V1.to_owned(),
            logical_human_owner_id: "owner-a".to_owned(),
            registration_id: "persons_registration".to_owned(),
            runtime_instance_id: "persons-runtime-1".to_owned(),
            runtime_generation: 7,
            grant_epoch: 8,
        };
        let valid = ManagedStorageRuntimeConfigurationV1 {
            database_id: "persons-db".to_owned(),
            pgbouncer_host: "127.0.0.1".to_owned(),
            pgbouncer_port: 6432,
            runtime_principal: "persons_runtime_role".to_owned(),
            storage_generation: 1,
            credential_revision: 1,
            storage_instance_id: "storage-1".to_owned(),
            owner: PERSONS_OWNER_ID_V1.to_owned(),
            role_epoch: 1,
            pool_alias: "runtime_persons_registration_7".to_owned(),
            max_connections: 4,
            statement_timeout_millis: 5_000,
            storage_bundle_revision: 3,
            storage_bundle_digest: vec![9; 32],
            vault_instance_id: "vault-1".to_owned(),
            vault_runtime_generation: 1,
            vault_hpke_public_key_x25519: vec![8; 32],
            runtime_instance_id: admission.runtime_instance_id.clone(),
            logical_owner_id: PERSONS_OWNER_ID_V1.to_owned(),
        };
        assert!(storage_binding(&valid, &admission).is_ok());
        for invalid in [
            ManagedStorageRuntimeConfigurationV1 {
                runtime_instance_id: "stale-runtime".to_owned(),
                ..valid.clone()
            },
            ManagedStorageRuntimeConfigurationV1 {
                owner: "contacts".to_owned(),
                ..valid.clone()
            },
            ManagedStorageRuntimeConfigurationV1 {
                credential_revision: 0,
                ..valid.clone()
            },
            ManagedStorageRuntimeConfigurationV1 {
                storage_bundle_digest: Vec::new(),
                ..valid
            },
        ] {
            assert_eq!(
                storage_binding(&invalid, &admission),
                Err(PersonsManagedRuntimeErrorV1::Admission)
            );
        }
    }
}
