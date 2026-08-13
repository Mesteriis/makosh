use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    DurableSubjectV1, JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity,
    RuntimePublishPermitV1, RuntimeSubscribePermitV1, StreamKindV1,
    request_managed_runtime_event_access_v2,
};
use makosh_mail_address_book_contract::MailPersonSourceContractV1;
use makosh_mail_persons_sync_api::{MAIL_PERSONS_SYNC_OWNER_V1, MailPersonsSyncContractV1};
use makosh_mail_persons_sync_persistence::{
    MAIL_PERSONS_SYNC_OUTBOX_READ_LIMIT_V1, MailPersonsSyncPersistenceErrorV1,
    MailPersonsSyncPersistenceV1,
};
use makosh_persons_api::{
    persons_command_contract_reference_v1, persons_command_rejected_contract_reference_v1,
    persons_command_succeeded_contract_reference_v1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlTransportErrorV2},
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

use crate::account_binding::{
    MailPersonsSyncAccountBindingContextV1, MailPersonsSyncAccountBindingErrorV1,
    consume_mail_person_source_account_lifecycle_once_v1,
};
use crate::admission::{
    scheduler_job_contract_v1, scheduler_receipt_contract_v1,
    scheduler_schedule_control_contract_v1,
};
use crate::execution::{
    MailPersonsSyncExecutionContextV1, MailPersonsSyncExecutionErrorV1,
    consume_mail_person_source_once_v1,
};
use crate::page::{
    MailPersonsSyncPageContextV1, MailPersonsSyncPageErrorV1,
    consume_mail_person_source_page_once_v1,
};
use crate::persons_terminal::{
    MailPersonsSyncPersonsTerminalContextV1, MailPersonsSyncPersonsTerminalErrorV1,
    MailPersonsSyncPersonsTerminalKindV1, consume_mail_persons_sync_persons_terminal_once_v1,
};
use crate::scheduler::{
    MailPersonsSyncSchedulerContextV1, MailPersonsSyncSchedulerErrorV1,
    consume_mail_persons_sync_due_once_v1,
};

const MAX_OUTBOX_RELAY_PER_SERVICE_V1: usize = 8;
const _: () =
    assert!(MAX_OUTBOX_RELAY_PER_SERVICE_V1 <= MAIL_PERSONS_SYNC_OUTBOX_READ_LIMIT_V1 as usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(MailPersonsSyncPersistenceErrorV1),
    ControlClosed,
    Unavailable,
}

pub struct MailPersonsSyncManagedRuntimeV1 {
    admission: MailPersonsSyncRuntimeAdmissionV1,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: MailPersonsSyncPersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    subscriptions: Vec<RuntimeSubscribePermitV1>,
}

impl MailPersonsSyncManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &MailPersonsSyncRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, MailPersonsSyncManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(MailPersonsSyncManagedRuntimeErrorV1::Admission);
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
            .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
        let mut control_channel = leases.into_route_port().into_channel();
        let persistence = tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control_channel) => return Err(closed),
            persistence = MailPersonsSyncPersistenceV1::connect_runtime(
                &binding,
                &storage_configuration.database_id,
                &storage_configuration.pgbouncer_host,
                storage_configuration.pgbouncer_port,
                password,
            ) => persistence.map_err(MailPersonsSyncManagedRuntimeErrorV1::Persistence)?,
        };
        tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control_channel) => return Err(closed),
            ready = persistence.verify_storage_ready() => {
                ready.map_err(MailPersonsSyncManagedRuntimeErrorV1::Persistence)?;
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
        .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
        let publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
        validate_exact_publish_permit(&publish_permit)?;
        let mut available = event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
        let expected = expected_subscriptions();
        let mut subscriptions = Vec::with_capacity(expected.len());
        for contract in expected {
            subscriptions.push(take_exact_subscription(&mut available, &contract)?);
        }
        if !available.is_empty() {
            return Err(MailPersonsSyncManagedRuntimeErrorV1::Admission);
        }
        let event_connection = tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control_channel) => return Err(closed),
            connection = JetStreamClient::connect_runtime_with_jwt(
                event_hub_endpoint,
                identity,
                event_access.into_credential(),
            ) => connection.map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable)?,
        };
        for subscription in &subscriptions {
            tokio::select! {
                biased;
                closed = wait_for_bootstrap_control_close(&mut control_channel) => return Err(closed),
                consumer = event_connection.open_pull_consumer(subscription) => {
                    consumer.map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::EventContract)?;
                }
            }
        }
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control_channel,
            persistence,
            event_connection,
            event_publish_permit: publish_permit,
            subscriptions,
        })
    }

    pub fn pump_control_once(&mut self) -> Result<bool, MailPersonsSyncManagedRuntimeErrorV1> {
        pump_control(&mut self.control_channel)
    }

    pub async fn service_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, MailPersonsSyncManagedRuntimeErrorV1> {
        if now_unix_millis <= 0 {
            return Err(MailPersonsSyncManagedRuntimeErrorV1::EventContract);
        }
        let context = MailPersonsSyncExecutionContextV1 {
            logical_owner_id: self.admission.logical_owner_id.clone(),
            runtime_instance_id: self.admission.runtime_instance_id.clone(),
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis,
        };
        let mut progressed = false;
        let scheduler_context = MailPersonsSyncSchedulerContextV1 {
            logical_owner_id: self.admission.logical_owner_id.clone(),
            runtime_instance_id: self.admission.runtime_instance_id.clone(),
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis,
        };
        let scheduler_consumed = tokio::select! {
            biased;
            control = wait_for_control(&mut self.control_channel) => return control,
            consumed = consume_mail_persons_sync_due_once_v1(
                &self.persistence,
                &self.event_connection,
                &self.subscriptions[7],
                &scheduler_context,
            ) => consumed.map_err(scheduler_error)?,
        };
        progressed |= scheduler_consumed;
        let binding_context = MailPersonsSyncAccountBindingContextV1 {
            logical_owner_id: self.admission.logical_owner_id.clone(),
            runtime_instance_id: self.admission.runtime_instance_id.clone(),
            runtime_generation: self.admission.runtime_generation,
            grant_epoch: self.admission.grant_epoch,
            now_unix_millis,
        };
        for (index, contract) in [
            MailPersonSourceContractV1::AccountReady,
            MailPersonSourceContractV1::AccountRetired,
        ]
        .into_iter()
        .enumerate()
        {
            let consumed = tokio::select! {
                biased;
                control = wait_for_control(&mut self.control_channel) => return control,
                consumed = consume_mail_person_source_account_lifecycle_once_v1(
                    &self.persistence, &self.event_connection, &self.subscriptions[index + 8],
                    contract, &binding_context,
                ) => consumed.map_err(account_binding_error)?,
            };
            progressed |= consumed;
        }
        for (index, contract) in [
            MailPersonSourceContractV1::SourceObserved,
            MailPersonSourceContractV1::SourceUpdated,
            MailPersonSourceContractV1::SourceRemoved,
        ]
        .into_iter()
        .enumerate()
        {
            let consumed = tokio::select! {
                biased;
                control = wait_for_control(&mut self.control_channel) => return control,
                consumed = consume_mail_person_source_once_v1(
                    &self.persistence,
                    &self.event_connection,
                    &self.subscriptions[index],
                    contract,
                    &context,
                ) => consumed.map_err(execution_error)?,
            };
            progressed |= consumed;
        }
        let persons_context = MailPersonsSyncPersonsTerminalContextV1 {
            logical_owner_id: self.admission.logical_owner_id.clone(),
            runtime_instance_id: self.admission.runtime_instance_id.clone(),
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis,
        };
        for (index, kind) in [
            MailPersonsSyncPersonsTerminalKindV1::Succeeded,
            MailPersonsSyncPersonsTerminalKindV1::Rejected,
        ]
        .into_iter()
        .enumerate()
        {
            let consumed = tokio::select! {
                biased;
                control = wait_for_control(&mut self.control_channel) => return control,
                consumed = consume_mail_persons_sync_persons_terminal_once_v1(
                    &self.persistence,
                    &self.event_connection,
                    &self.subscriptions[index + 5],
                    kind,
                    &persons_context,
                ) => consumed.map_err(persons_terminal_error)?,
            };
            progressed |= consumed;
        }
        let page_context = MailPersonsSyncPageContextV1 {
            logical_owner_id: self.admission.logical_owner_id.clone(),
            runtime_instance_id: self.admission.runtime_instance_id.clone(),
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis,
        };
        for (index, contract) in [
            MailPersonSourceContractV1::PageCompleted,
            MailPersonSourceContractV1::PageRejected,
        ]
        .into_iter()
        .enumerate()
        {
            let consumed = tokio::select! {
                biased;
                control = wait_for_control(&mut self.control_channel) => return control,
                consumed = consume_mail_person_source_page_once_v1(
                    &self.persistence,
                    &self.event_connection,
                    &self.subscriptions[index + 3],
                    contract,
                    &page_context,
                ) => consumed.map_err(page_error)?,
            };
            progressed |= consumed;
        }
        for _ in 0..MAX_OUTBOX_RELAY_PER_SERVICE_V1 {
            let claim = tokio::select! {
                biased;
                control = wait_for_control(&mut self.control_channel) => return control,
                claim = self.persistence.claim_next_pending_outbox(&self.admission.logical_owner_id) => {
                    claim.map_err(MailPersonsSyncManagedRuntimeErrorV1::Persistence)?
                }
            };
            let Some(claim) = claim else {
                break;
            };
            let envelope_sha256 = claim.record().record.envelope_sha256;
            tokio::select! {
                biased;
                control = wait_for_control(&mut self.control_channel) => return control,
                published = self.event_connection.publish_exact(
                    &self.event_publish_permit,
                    &claim.record().record.envelope_bytes,
                ) => published.map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable)?,
            };
            claim
                .mark_published(envelope_sha256, now_unix_millis)
                .await
                .map_err(MailPersonsSyncManagedRuntimeErrorV1::Persistence)?;
            progressed = true;
        }
        if let Some(outbox) = self
            .persistence
            .load_pending_schedule_control(&self.admission.logical_owner_id)
            .await
            .map_err(MailPersonsSyncManagedRuntimeErrorV1::Persistence)?
        {
            tokio::select! {
                biased;
                control = wait_for_control(&mut self.control_channel) => return control,
                published = self.event_connection.publish_exact(&self.event_publish_permit, &outbox.record.envelope_bytes) => {
                    published.map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable)?;
                }
            }
            self.persistence
                .mark_schedule_control_published(
                    &self.admission.logical_owner_id,
                    outbox.record.message_id,
                    outbox.record.envelope_sha256,
                    now_unix_millis,
                )
                .await
                .map_err(MailPersonsSyncManagedRuntimeErrorV1::Persistence)?;
            progressed = true;
        }
        if !progressed {
            tokio::select! {
                biased;
                control = wait_for_control(&mut self.control_channel) => return control,
                () = tokio::time::sleep(std::time::Duration::from_millis(25)) => {}
            }
        }
        Ok(progressed)
    }

    /// Waits between transient retries while retaining prompt Kernel-control
    /// responsiveness. A closed inherited control peer is terminal.
    pub async fn wait_retry_delay(
        &mut self,
        delay: std::time::Duration,
    ) -> Result<bool, MailPersonsSyncManagedRuntimeErrorV1> {
        tokio::select! {
            biased;
            control = wait_for_control(&mut self.control_channel) => match control {
                Err(MailPersonsSyncManagedRuntimeErrorV1::ControlClosed) => Ok(false),
                Err(error) => Err(error),
                Ok(_) => Ok(true),
            },
            () = tokio::time::sleep(delay) => Ok(true),
        }
    }

    #[must_use]
    pub const fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }
}

fn execution_error(error: MailPersonsSyncExecutionErrorV1) -> MailPersonsSyncManagedRuntimeErrorV1 {
    match error {
        MailPersonsSyncExecutionErrorV1::InvalidEnvelope
        | MailPersonsSyncExecutionErrorV1::InvalidPayload
        | MailPersonsSyncExecutionErrorV1::PageNotReady => {
            MailPersonsSyncManagedRuntimeErrorV1::EventContract
        }
        MailPersonsSyncExecutionErrorV1::Persistence(error) => {
            MailPersonsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailPersonsSyncExecutionErrorV1::EventUnavailable => {
            MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn scheduler_error(error: MailPersonsSyncSchedulerErrorV1) -> MailPersonsSyncManagedRuntimeErrorV1 {
    match error {
        MailPersonsSyncSchedulerErrorV1::InvalidEnvelope
        | MailPersonsSyncSchedulerErrorV1::InvalidPayload => {
            MailPersonsSyncManagedRuntimeErrorV1::EventContract
        }
        MailPersonsSyncSchedulerErrorV1::Persistence(error) => {
            MailPersonsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailPersonsSyncSchedulerErrorV1::EventUnavailable => {
            MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn page_error(error: MailPersonsSyncPageErrorV1) -> MailPersonsSyncManagedRuntimeErrorV1 {
    match error {
        MailPersonsSyncPageErrorV1::InvalidEnvelope
        | MailPersonsSyncPageErrorV1::InvalidPayload => {
            MailPersonsSyncManagedRuntimeErrorV1::EventContract
        }
        MailPersonsSyncPageErrorV1::Persistence(error) => {
            MailPersonsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailPersonsSyncPageErrorV1::EventUnavailable => {
            MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn persons_terminal_error(
    error: MailPersonsSyncPersonsTerminalErrorV1,
) -> MailPersonsSyncManagedRuntimeErrorV1 {
    match error {
        MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope
        | MailPersonsSyncPersonsTerminalErrorV1::InvalidPayload => {
            MailPersonsSyncManagedRuntimeErrorV1::EventContract
        }
        MailPersonsSyncPersonsTerminalErrorV1::Persistence(error) => {
            MailPersonsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailPersonsSyncPersonsTerminalErrorV1::EventUnavailable => {
            MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn account_binding_error(
    error: MailPersonsSyncAccountBindingErrorV1,
) -> MailPersonsSyncManagedRuntimeErrorV1 {
    match error {
        MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope
        | MailPersonsSyncAccountBindingErrorV1::InvalidPayload => {
            MailPersonsSyncManagedRuntimeErrorV1::EventContract
        }
        MailPersonsSyncAccountBindingErrorV1::Persistence(error) => {
            MailPersonsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailPersonsSyncAccountBindingErrorV1::EventUnavailable => {
            MailPersonsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn expected_subscriptions() -> Vec<ContractReferenceV1> {
    vec![
        MailPersonSourceContractV1::SourceObserved.reference(),
        MailPersonSourceContractV1::SourceUpdated.reference(),
        MailPersonSourceContractV1::SourceRemoved.reference(),
        MailPersonSourceContractV1::PageCompleted.reference(),
        MailPersonSourceContractV1::PageRejected.reference(),
        persons_command_succeeded_contract_reference_v1(),
        persons_command_rejected_contract_reference_v1(),
        scheduler_job_contract_v1(),
        MailPersonSourceContractV1::AccountReady.reference(),
        MailPersonSourceContractV1::AccountRetired.reference(),
        scheduler_schedule_control_contract_v1(),
    ]
}

fn validate_exact_publish_permit(
    permit: &RuntimePublishPermitV1,
) -> Result<(), MailPersonsSyncManagedRuntimeErrorV1> {
    let contracts = [
        (
            StreamKindV1::Command,
            MailPersonSourceContractV1::FetchPageCommand.reference(),
        ),
        (
            StreamKindV1::Command,
            persons_command_contract_reference_v1(),
        ),
        (
            StreamKindV1::Result,
            MailPersonsSyncContractV1::PageReceipt.reference(),
        ),
        (
            StreamKindV1::Result,
            MailPersonsSyncContractV1::RunResult.reference(),
        ),
        (StreamKindV1::Ack, scheduler_receipt_contract_v1()),
        (StreamKindV1::Result, scheduler_receipt_contract_v1()),
        (
            StreamKindV1::Command,
            scheduler_schedule_control_contract_v1(),
        ),
    ];
    let subjects = contracts
        .into_iter()
        .map(|(kind, contract)| {
            DurableSubjectV1::new(kind, contract.owner, contract.name, contract.major)
                .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)
        })
        .collect::<Result<Vec<_>, _>>()?;
    permit
        .permits_exact_subjects(&subjects)
        .then_some(())
        .ok_or(MailPersonsSyncManagedRuntimeErrorV1::Admission)
}

fn take_exact_subscription(
    permits: &mut Vec<RuntimeSubscribePermitV1>,
    contract: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, MailPersonsSyncManagedRuntimeErrorV1> {
    let index = permits
        .iter()
        .position(|permit| permit.contract().is_some_and(|actual| actual == contract))
        .ok_or(MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
    Ok(permits.remove(index))
}

fn validate_admission(
    admission: &MailPersonsSyncRuntimeAdmissionV1,
) -> Result<(), MailPersonsSyncManagedRuntimeErrorV1> {
    if admission.logical_owner_id.is_empty()
        || admission.logical_owner_id == MAIL_PERSONS_SYNC_OWNER_V1
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        Err(MailPersonsSyncManagedRuntimeErrorV1::Admission)
    } else {
        Ok(())
    }
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &MailPersonsSyncRuntimeAdmissionV1,
) -> Result<(), MailPersonsSyncManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(MailPersonsSyncManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &MailPersonsSyncRuntimeAdmissionV1,
) -> Result<(), MailPersonsSyncManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, MailPersonsSyncManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if control_peer_closed(leases.route_port_mut().channel_mut())? {
            return Err(MailPersonsSyncManagedRuntimeErrorV1::ControlClosed);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(MailPersonsSyncManagedRuntimeErrorV1::Unavailable)
}

async fn wait_for_control(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<bool, MailPersonsSyncManagedRuntimeErrorV1> {
    loop {
        if pump_control(channel)? {
            return Ok(true);
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}

async fn wait_for_bootstrap_control_close(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> MailPersonsSyncManagedRuntimeErrorV1 {
    loop {
        match control_peer_closed(channel) {
            Ok(true) => return MailPersonsSyncManagedRuntimeErrorV1::ControlClosed,
            Ok(false) => tokio::time::sleep(std::time::Duration::from_millis(5)).await,
            Err(error) => return error,
        }
    }
}

fn pump_control(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<bool, MailPersonsSyncManagedRuntimeErrorV1> {
    let Some((correlation_id, _request)) = channel.try_receive_request().map_err(|error| {
        if matches!(error, ManagedControlTransportErrorV2::PeerClosed) {
            MailPersonsSyncManagedRuntimeErrorV1::ControlClosed
        } else {
            MailPersonsSyncManagedRuntimeErrorV1::Unavailable
        }
    })?
    else {
        return Ok(false);
    };
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        )
        .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Unavailable)?;
    Ok(true)
}

fn control_peer_closed(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<bool, MailPersonsSyncManagedRuntimeErrorV1> {
    channel
        .peer_closed_preserving_frames()
        .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &MailPersonsSyncRuntimeAdmissionV1,
) -> Result<StorageBindingV1, MailPersonsSyncManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != MAIL_PERSONS_SYNC_OWNER_V1
        || configuration.owner != MAIL_PERSONS_SYNC_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(MailPersonsSyncManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| MailPersonsSyncManagedRuntimeErrorV1::Admission)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_subscription_and_publish_sets_are_bounded() {
        assert_eq!(expected_subscriptions().len(), 11);
        assert_eq!(MAX_OUTBOX_RELAY_PER_SERVICE_V1, 8);
    }

    #[test]
    fn admission_rejects_module_owner_and_stale_fences() {
        let valid = MailPersonsSyncRuntimeAdmissionV1 {
            logical_owner_id: "owner-a".to_owned(),
            registration_id: "mail-persons-sync-registration".to_owned(),
            runtime_instance_id: "mail-persons-sync-runtime-1".to_owned(),
            runtime_generation: 2,
            grant_epoch: 3,
        };
        assert_eq!(validate_admission(&valid), Ok(()));
        assert_eq!(
            validate_admission(&MailPersonsSyncRuntimeAdmissionV1 {
                logical_owner_id: MAIL_PERSONS_SYNC_OWNER_V1.to_owned(),
                ..valid.clone()
            }),
            Err(MailPersonsSyncManagedRuntimeErrorV1::Admission)
        );
        assert_eq!(
            validate_admission(&MailPersonsSyncRuntimeAdmissionV1 {
                runtime_generation: 0,
                ..valid
            }),
            Err(MailPersonsSyncManagedRuntimeErrorV1::Admission)
        );
    }
}
