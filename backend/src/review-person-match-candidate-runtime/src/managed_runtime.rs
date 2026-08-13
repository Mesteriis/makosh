use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    DurableSubjectV1, JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity,
    RuntimePublishPermitV1, RuntimeSubscribePermitV1, StreamKindV1,
    request_managed_runtime_event_access_v2,
};
use makosh_identity_resolution_api::identity_resolution_person_match_candidate_contract_reference_v1;
use makosh_review_person_match_candidate_api::{
    REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1,
    review_person_match_candidate_approved_contract_reference_v1,
    review_person_match_candidate_decision_contract_reference_v1,
    review_person_match_candidate_submission_rejected_contract_reference_v1,
    review_person_match_candidate_submitted_contract_reference_v1,
};
use makosh_review_person_match_candidate_persistence::{
    ReviewPersonMatchCandidatePersistenceErrorV1, ReviewPersonMatchCandidatePersistenceV1,
};
use makosh_review_person_match_candidate_promotion_api::review_person_match_candidate_promotion_result_contract_reference_v1;
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
    ReviewPersonMatchCandidateExecutionContextV1, ReviewPersonMatchCandidateExecutionErrorV1,
    consume_person_match_candidate_decision_once_v1,
    consume_person_match_candidate_promotion_result_once_v1,
    consume_persons_review_candidate_once_v1,
    dispatch_review_person_match_candidate_client_request_v1,
};

const MAX_OUTBOX_RELAY_PER_SERVICE_V1: usize = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewPersonMatchCandidateRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewPersonMatchCandidateManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(ReviewPersonMatchCandidatePersistenceErrorV1),
    ControlClosed,
    Unavailable,
}

pub struct ReviewPersonMatchCandidateManagedRuntimeV1 {
    admission: ReviewPersonMatchCandidateRuntimeAdmissionV1,
    control: ManagedControlChannelV2<UnixStream>,
    persistence: ReviewPersonMatchCandidatePersistenceV1,
    events: RuntimeJetStreamConnection,
    publish_permit: RuntimePublishPermitV1,
    subscriptions: Vec<RuntimeSubscribePermitV1>,
}

impl ReviewPersonMatchCandidateManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control: UnixStream,
        descriptor: Vec<u8>,
        settings: Vec<u8>,
        admission: &ReviewPersonMatchCandidateRuntimeAdmissionV1,
        storage: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission);
        }
        let mut control = ManagedControlChannelV2::new(control);
        authenticate(&mut control, descriptor, settings, admission)?;
        let binding = storage_binding(&storage, admission)?;
        let public_key = storage
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage.vault_instance_id.clone(),
            storage.vault_runtime_generation,
            public_key,
        )
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
        let mut control = leases.into_route_port().into_channel();
        let persistence = tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control) => return Err(closed),
            value = ReviewPersonMatchCandidatePersistenceV1::connect_runtime(
                &binding,
                &storage.database_id,
                &storage.pgbouncer_host,
                storage.pgbouncer_port,
                password,
            ) => value.map_err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Persistence)?,
        };
        tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control) => return Err(closed),
            ready = persistence.verify_storage_ready() => {
                ready.map_err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Persistence)?;
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
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::EventUnavailable)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
        let publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
        validate_exact_publish_permit(&publish_permit)?;
        let mut available = event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
        let mut subscriptions = Vec::new();
        for expected in expected_subscriptions() {
            subscriptions.push(take_exact_subscription(&mut available, &expected)?);
        }
        if !available.is_empty() {
            return Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission);
        }
        let events = tokio::select! {
            biased;
            closed = wait_for_bootstrap_control_close(&mut control) => return Err(closed),
            value = JetStreamClient::connect_runtime_with_jwt(
                event_hub_endpoint,
                identity,
                event_access.into_credential(),
            ) => value.map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::EventUnavailable)?,
        };
        for subscription in &subscriptions {
            tokio::select! {
                biased;
                closed = wait_for_bootstrap_control_close(&mut control) => return Err(closed),
                consumer = events.open_pull_consumer(subscription) => {
                    consumer.map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::EventContract)?;
                }
            }
        }
        signal_ready(&mut control, admission)?;
        control
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)?;
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
    ) -> Result<bool, ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
        let context = ReviewPersonMatchCandidateExecutionContextV1 {
            logical_owner_id: self.admission.logical_human_owner_id.clone(),
            runtime_instance_id: self.admission.runtime_instance_id.clone(),
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis,
        };
        let mut progressed = false;
        for index in 0..3 {
            let consumed = tokio::select! {
                biased;
                control = wait_for_control(&mut self.control, &self.persistence, &self.admission, now_unix_millis) => return control,
                value = consume_by_index(index, &self.persistence, &self.events, &self.subscriptions[index], &context) => value.map_err(execution_error)?,
            };
            progressed |= consumed;
        }
        for _ in 0..MAX_OUTBOX_RELAY_PER_SERVICE_V1 {
            let claim = self
                .persistence
                .claim_next_pending_outbox(&self.admission.logical_human_owner_id)
                .await
                .map_err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Persistence)?;
            let Some(claim) = claim else { break };
            let envelope_sha256 = claim.record().record.envelope_sha256;
            tokio::select! {
                biased;
                control = wait_for_control(&mut self.control, &self.persistence, &self.admission, now_unix_millis) => return control,
                published = self.events.publish_exact(&self.publish_permit, &claim.record().record.envelope_bytes) => {
                    published.map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::EventUnavailable)?;
                }
            }
            claim
                .mark_published(envelope_sha256, now_unix_millis)
                .await
                .map_err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Persistence)?;
            progressed = true;
        }
        if !progressed {
            tokio::select! {
                biased;
                control = wait_for_control(&mut self.control, &self.persistence, &self.admission, now_unix_millis) => return control,
                () = tokio::time::sleep(std::time::Duration::from_millis(25)) => {}
            }
        }
        Ok(progressed)
    }

    pub async fn wait_retry_delay(
        &mut self,
        delay: std::time::Duration,
    ) -> Result<bool, ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
        let now_unix_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|value| i64::try_from(value.as_millis()).ok())
            .ok_or(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)?;
        tokio::select! {
            biased;
            control = wait_for_control(&mut self.control, &self.persistence, &self.admission, now_unix_millis) => match control {
                Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::ControlClosed) => Ok(false),
                Err(error) => Err(error),
                Ok(_) => Ok(true),
            },
            () = tokio::time::sleep(delay) => Ok(true),
        }
    }
}

async fn consume_by_index(
    index: usize,
    persistence: &ReviewPersonMatchCandidatePersistenceV1,
    events: &RuntimeJetStreamConnection,
    subscription: &RuntimeSubscribePermitV1,
    context: &ReviewPersonMatchCandidateExecutionContextV1,
) -> Result<bool, ReviewPersonMatchCandidateExecutionErrorV1> {
    match index {
        0 => {
            consume_persons_review_candidate_once_v1(persistence, events, subscription, context)
                .await
        }
        1 => {
            consume_person_match_candidate_decision_once_v1(
                persistence,
                events,
                subscription,
                context,
            )
            .await
        }
        2 => {
            consume_person_match_candidate_promotion_result_once_v1(
                persistence,
                events,
                subscription,
                context,
            )
            .await
        }
        _ => Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidContext),
    }
}

fn expected_subscriptions() -> Vec<ContractReferenceV1> {
    vec![
        identity_resolution_person_match_candidate_contract_reference_v1(),
        review_person_match_candidate_decision_contract_reference_v1(),
        review_person_match_candidate_promotion_result_contract_reference_v1(),
    ]
}

fn validate_exact_publish_permit(
    permit: &RuntimePublishPermitV1,
) -> Result<(), ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    let subjects = [
        review_person_match_candidate_approved_contract_reference_v1(),
        review_person_match_candidate_submission_rejected_contract_reference_v1(),
        review_person_match_candidate_submitted_contract_reference_v1(),
    ]
    .into_iter()
    .map(|contract| {
        DurableSubjectV1::new(
            StreamKindV1::Event,
            contract.owner,
            contract.name,
            contract.major,
        )
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)
    })
    .collect::<Result<Vec<_>, _>>()?;
    permit
        .permits_exact_subjects(&subjects)
        .then_some(())
        .ok_or(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)
}

fn take_exact_subscription(
    available: &mut Vec<RuntimeSubscribePermitV1>,
    expected: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    let index = available
        .iter()
        .position(|permit| permit.contract().is_some_and(|actual| actual == expected))
        .ok_or(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
    Ok(available.remove(index))
}

fn validate_admission(
    admission: &ReviewPersonMatchCandidateRuntimeAdmissionV1,
) -> Result<(), ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    if admission.logical_owner_id != REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1
        || admission.logical_human_owner_id.is_empty()
        || admission.logical_human_owner_id == REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)
    } else {
        Ok(())
    }
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &ReviewPersonMatchCandidateRuntimeAdmissionV1,
) -> Result<(), ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &ReviewPersonMatchCandidateRuntimeAdmissionV1,
) -> Result<(), ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if control_peer_closed(leases.route_port_mut().channel_mut())? {
            return Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::ControlClosed);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)
}

async fn wait_for_control(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    persistence: &ReviewPersonMatchCandidatePersistenceV1,
    admission: &ReviewPersonMatchCandidateRuntimeAdmissionV1,
    now_unix_millis: i64,
) -> Result<bool, ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    loop {
        if pump_control(channel, persistence, admission, now_unix_millis).await? {
            return Ok(true);
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}

async fn wait_for_bootstrap_control_close(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> ReviewPersonMatchCandidateManagedRuntimeErrorV1 {
    loop {
        match control_peer_closed(channel) {
            Ok(true) => return ReviewPersonMatchCandidateManagedRuntimeErrorV1::ControlClosed,
            Ok(false) => tokio::time::sleep(std::time::Duration::from_millis(5)).await,
            Err(error) => return error,
        }
    }
}

async fn pump_control(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    persistence: &ReviewPersonMatchCandidatePersistenceV1,
    admission: &ReviewPersonMatchCandidateRuntimeAdmissionV1,
    now_unix_millis: i64,
) -> Result<bool, ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    let Some((correlation_id, request)) = channel.try_receive_request().map_err(|error| {
        if matches!(error, ManagedControlTransportErrorV2::PeerClosed) {
            ReviewPersonMatchCandidateManagedRuntimeErrorV1::ControlClosed
        } else {
            ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable
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
        let response = dispatch_review_person_match_candidate_client_request_v1(
            persistence,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            &admission.logical_human_owner_id,
            request,
            now_unix_millis,
        )
        .await;
        validate_module_client_response_v1(&response)
            .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)?;
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
            .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)?;
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
) -> Result<bool, ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: error_code.to_owned(),
            },
        )
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)?;
    Ok(true)
}

fn control_peer_closed(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<bool, ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    channel
        .peer_closed_preserving_frames()
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &ReviewPersonMatchCandidateRuntimeAdmissionV1,
) -> Result<StorageBindingV1, ReviewPersonMatchCandidateManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1
        || configuration.owner != REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| ReviewPersonMatchCandidateManagedRuntimeErrorV1::Admission)
}

fn execution_error(
    error: ReviewPersonMatchCandidateExecutionErrorV1,
) -> ReviewPersonMatchCandidateManagedRuntimeErrorV1 {
    match error {
        ReviewPersonMatchCandidateExecutionErrorV1::EventUnavailable => {
            ReviewPersonMatchCandidateManagedRuntimeErrorV1::EventUnavailable
        }
        ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope
        | ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload
        | ReviewPersonMatchCandidateExecutionErrorV1::InvalidContext => {
            ReviewPersonMatchCandidateManagedRuntimeErrorV1::EventContract
        }
        ReviewPersonMatchCandidateExecutionErrorV1::Persistence(error) => {
            ReviewPersonMatchCandidateManagedRuntimeErrorV1::Persistence(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_contour_has_exact_three_subscriptions_and_bounded_relay() {
        assert_eq!(expected_subscriptions().len(), 3);
        assert_eq!(MAX_OUTBOX_RELAY_PER_SERVICE_V1, 4);
    }
}
