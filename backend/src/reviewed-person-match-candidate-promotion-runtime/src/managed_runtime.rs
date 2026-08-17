use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    DurableSubjectV1, JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity,
    RuntimePublishPermitV1, RuntimeSubscribePermitV1, StreamKindV1,
    request_managed_runtime_event_access_v2,
};
use makosh_persons_api::{
    persons_command_contract_reference_v1, persons_command_rejected_contract_reference_v1,
    persons_command_succeeded_contract_reference_v1,
};
use makosh_review_person_match_candidate_api::review_person_match_candidate_approved_contract_reference_v1;
use makosh_review_person_match_candidate_promotion_api::review_person_match_candidate_promotion_result_contract_reference_v1;
use makosh_reviewed_person_match_candidate_promotion_persistence::{
    ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    ReviewedPersonMatchCandidatePromotionPersistenceV1,
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

use crate::{
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1,
    ReviewedPersonMatchCandidatePromotionExecutionContextV1,
    ReviewedPersonMatchCandidatePromotionExecutionErrorV1,
    consume_person_match_candidate_approval_once_v1, consume_persons_rejected_terminal_once_v1,
    consume_persons_succeeded_terminal_once_v1,
};

const MAX_OUTBOX_RELAY_PER_SERVICE_V1: usize = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedPersonMatchCandidatePromotionRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1),
    ControlClosed,
    Unavailable,
}

pub struct ReviewedPersonMatchCandidatePromotionManagedRuntimeV1 {
    admission: ReviewedPersonMatchCandidatePromotionRuntimeAdmissionV1,
    control: ManagedControlChannelV2<UnixStream>,
    persistence: ReviewedPersonMatchCandidatePromotionPersistenceV1,
    events: RuntimeJetStreamConnection,
    publish_permit: RuntimePublishPermitV1,
    subscriptions: Vec<RuntimeSubscribePermitV1>,
}

impl ReviewedPersonMatchCandidatePromotionManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control: UnixStream,
        descriptor: Vec<u8>,
        settings: Vec<u8>,
        admission: &ReviewedPersonMatchCandidatePromotionRuntimeAdmissionV1,
        storage: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission);
        }
        let mut control = ManagedControlChannelV2::new(control);
        authenticate(&mut control, descriptor, settings, admission)?;
        let binding = storage_binding(&storage, admission)?;
        let public_key = storage
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage.vault_instance_id.clone(),
            storage.vault_runtime_generation,
            public_key,
        )
        .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let mut control = leases.into_route_port().into_channel();
        let persistence = tokio::select! {biased; closed=wait_for_bootstrap_control_close(&mut control)=>return Err(closed), value=ReviewedPersonMatchCandidatePromotionPersistenceV1::connect_runtime(&binding,&storage.database_id,&storage.pgbouncer_host,storage.pgbouncer_port,password)=>value.map_err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Persistence)?};
        tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control) => return Err(closed),
            ready = persistence.verify_storage_ready() => {
                ready.map_err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Persistence)?;
            }
        }
        let event_access = request_managed_runtime_event_access_v2(
            &mut control,
            &storage.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            event_credential_revision,
        )
        .map_err(|_| {
            ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::EventUnavailable
        })?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        validate_exact_publish_permit(&publish_permit)?;
        let mut available = event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let mut subscriptions = Vec::new();
        for expected in expected_subscriptions() {
            subscriptions.push(take_exact_subscription(&mut available, &expected)?);
        }
        if !available.is_empty() {
            return Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission);
        }
        let events = tokio::select! {biased; closed=wait_for_bootstrap_control_close(&mut control)=>return Err(closed), value=JetStreamClient::connect_runtime_with_jwt(event_hub_endpoint,identity,event_access.into_credential())=>value.map_err(|_|ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::EventUnavailable)?};
        for subscription in &subscriptions {
            tokio::select! {
                biased;
                closed = wait_for_bootstrap_control_close(&mut control) => return Err(closed),
                consumer = events.open_pull_consumer(subscription) => {
                    consumer.map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::EventContract)?;
                }
            }
        }
        signal_ready(&mut control, admission)?;
        control
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control,
            persistence,
            events,
            publish_permit,
            subscriptions,
        })
    }

    pub async fn service_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
        let context = ReviewedPersonMatchCandidatePromotionExecutionContextV1 {
            logical_owner_id: self.admission.logical_human_owner_id.clone(),
            runtime_instance_id: self.admission.runtime_instance_id.clone(),
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis,
        };
        let mut progressed = false;
        for index in 0..3 {
            let consumed = tokio::select! {biased; control=wait_for_control(&mut self.control)=>return control, value=consume_by_index(index,&self.persistence,&self.events,&self.subscriptions[index],&context)=>value.map_err(execution_error)?};
            progressed |= consumed;
        }
        for _ in 0..MAX_OUTBOX_RELAY_PER_SERVICE_V1 {
            let claim = self
                .persistence
                .claim_next_pending_outbox(&self.admission.logical_human_owner_id)
                .await
                .map_err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Persistence)?;
            let Some(claim) = claim else { break };
            let envelope_sha256 = claim.record().record.envelope_sha256;
            tokio::select! {
                biased;
                control = wait_for_control(&mut self.control) => return control,
                published = self.events.publish_exact(&self.publish_permit,&claim.record().record.envelope_bytes) => {
                    published.map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::EventUnavailable)?;
                }
            }
            claim
                .mark_published(envelope_sha256, now_unix_millis)
                .await
                .map_err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Persistence)?;
            progressed = true;
        }
        Ok(progressed)
    }

    pub async fn wait_retry_delay(
        &mut self,
        delay: std::time::Duration,
    ) -> Result<bool, ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
        tokio::select! {biased; control=wait_for_control(&mut self.control)=>match control{Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::ControlClosed)=>Ok(false),Err(error)=>Err(error),Ok(_)=>Ok(true)}, ()=tokio::time::sleep(delay)=>Ok(true)}
    }
}

async fn consume_by_index(
    index: usize,
    persistence: &ReviewedPersonMatchCandidatePromotionPersistenceV1,
    events: &RuntimeJetStreamConnection,
    subscription: &RuntimeSubscribePermitV1,
    context: &ReviewedPersonMatchCandidatePromotionExecutionContextV1,
) -> Result<bool, ReviewedPersonMatchCandidatePromotionExecutionErrorV1> {
    match index {
        0 => {
            consume_person_match_candidate_approval_once_v1(
                persistence,
                events,
                subscription,
                context,
            )
            .await
        }
        1 => {
            consume_persons_rejected_terminal_once_v1(persistence, events, subscription, context)
                .await
        }
        2 => {
            consume_persons_succeeded_terminal_once_v1(persistence, events, subscription, context)
                .await
        }
        _ => Err(ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidContext),
    }
}
fn expected_subscriptions() -> Vec<ContractReferenceV1> {
    vec![
        review_person_match_candidate_approved_contract_reference_v1(),
        persons_command_rejected_contract_reference_v1(),
        persons_command_succeeded_contract_reference_v1(),
    ]
}
fn validate_exact_publish_permit(
    permit: &RuntimePublishPermitV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
    let subjects = [
        (
            StreamKindV1::Command,
            persons_command_contract_reference_v1(),
        ),
        (
            StreamKindV1::Result,
            review_person_match_candidate_promotion_result_contract_reference_v1(),
        ),
    ]
    .into_iter()
    .map(|(kind, c)| {
        DurableSubjectV1::new(kind, c.owner, c.name, c.major)
            .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)
    })
    .collect::<Result<Vec<_>, _>>()?;
    permit
        .permits_exact_subjects(&subjects)
        .then_some(())
        .ok_or(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)
}
fn take_exact_subscription(
    available: &mut Vec<RuntimeSubscribePermitV1>,
    expected: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
    let index = available
        .iter()
        .position(|permit| permit.contract().is_some_and(|actual| actual == expected))
        .ok_or(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
    Ok(available.remove(index))
}
fn validate_admission(
    admission: &ReviewedPersonMatchCandidatePromotionRuntimeAdmissionV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
    if admission.logical_owner_id != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1
        || admission.logical_human_owner_id.is_empty()
        || admission.logical_human_owner_id == REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)
    } else {
        Ok(())
    }
}
fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &ReviewedPersonMatchCandidatePromotionRuntimeAdmissionV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}
fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &ReviewedPersonMatchCandidatePromotionRuntimeAdmissionV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Unavailable)
}
async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1>
{
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if control_peer_closed(leases.route_port_mut().channel_mut())? {
            return Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::ControlClosed);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Unavailable)
}
async fn wait_for_control(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<bool, ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
    loop {
        if pump_control(channel)? {
            return Ok(true);
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}
async fn wait_for_bootstrap_control_close(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1 {
    loop {
        match control_peer_closed(channel) {
            Ok(true) => {
                return ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::ControlClosed;
            }
            Ok(false) => tokio::time::sleep(std::time::Duration::from_millis(5)).await,
            Err(error) => return error,
        }
    }
}
fn pump_control(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<bool, ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
    let Some((correlation_id, _)) = channel.try_receive_request().map_err(|error| {
        if matches!(error, ManagedControlTransportErrorV2::PeerClosed) {
            ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::ControlClosed
        } else {
            ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Unavailable
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
                error_code: "managed_runtime_control_unexpected_request".into(),
            },
        )
        .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
    Ok(true)
}
fn control_peer_closed(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<bool, ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
    channel
        .peer_closed_preserving_frames()
        .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Unavailable)
}
fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &ReviewedPersonMatchCandidatePromotionRuntimeAdmissionV1,
) -> Result<StorageBindingV1, ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1
        || configuration.owner != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Admission)
}
fn execution_error(
    error: ReviewedPersonMatchCandidatePromotionExecutionErrorV1,
) -> ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1 {
    match error {
        ReviewedPersonMatchCandidatePromotionExecutionErrorV1::EventUnavailable => {
            ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::EventUnavailable
        }
        ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidContext
        | ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidEnvelope
        | ReviewedPersonMatchCandidatePromotionExecutionErrorV1::InvalidPayload
        | ReviewedPersonMatchCandidatePromotionExecutionErrorV1::Action(_) => {
            ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::EventContract
        }
        ReviewedPersonMatchCandidatePromotionExecutionErrorV1::Persistence(error) => {
            ReviewedPersonMatchCandidatePromotionManagedRuntimeErrorV1::Persistence(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_topology_and_bounded_fair_relay() {
        assert_eq!(expected_subscriptions().len(), 3);
        assert_eq!(MAX_OUTBOX_RELAY_PER_SERVICE_V1, 4);
    }
}
