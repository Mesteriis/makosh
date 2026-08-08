use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_review_note_candidate_api::{
    REVIEW_NOTE_CANDIDATE_MODULE_ID_V1, REVIEW_NOTE_CANDIDATE_OWNER_V1,
    review_note_candidate_submit_contract_reference_v1,
};
use makosh_review_note_candidate_persistence::{
    ReviewNoteCandidatePersistenceErrorV1, ReviewNoteCandidatePersistenceV1,
};
use makosh_review_note_candidate_promotion_api::review_note_candidate_promotion_result_contract_reference_v1;
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryResponseV1,
        ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
        ManagedStorageRuntimeConfigurationV1, ModuleClientRequestV1, ModuleClientResponseV1,
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
    client_port::{ReviewNoteCandidateClientRuntimeContextV1, decide_payload_v1, get_payload_v1},
    client_realtime::{
        ReviewNoteCandidateClientRealtimeErrorV1, ReviewNoteCandidateClientRealtimePublisherV1,
    },
    contracts::{command_contract_v1, query_contract_v1},
    event_outbox::{
        ReviewNoteCandidateEventRelayErrorV1, relay_review_note_candidate_outbox_once_v1,
    },
    promotion_result::{
        ReviewNoteCandidatePromotionResultErrorV1,
        consume_review_note_candidate_promotion_result_once_v1,
    },
    submission::{
        ReviewNoteCandidateSubmissionErrorV1, ReviewNoteCandidateSubmissionRuntimeContextV1,
        consume_review_note_candidate_submission_once_v1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewNoteCandidateRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewNoteCandidateManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    InvalidTransition,
    Persistence(ReviewNoteCandidatePersistenceErrorV1),
    Unavailable,
}

pub struct ReviewNoteCandidateManagedRuntimeV1 {
    admission: ReviewNoteCandidateRuntimeAdmissionV1,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: ReviewNoteCandidatePersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    submission_subscription: RuntimeSubscribePermitV1,
    promotion_result_subscription: RuntimeSubscribePermitV1,
    client_realtime: ReviewNoteCandidateClientRealtimePublisherV1,
}

impl ReviewNoteCandidateManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &ReviewNoteCandidateRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, ReviewNoteCandidateManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(ReviewNoteCandidateManagedRuntimeErrorV1::Admission);
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
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
        let persistence = ReviewNoteCandidatePersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(ReviewNoteCandidateManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(ReviewNoteCandidateManagedRuntimeErrorV1::Persistence)?;

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
        .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::EventUnavailable)?;
        let event_identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
        let permits = event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
        let submission_subscription = exact_permit(
            &permits,
            &review_note_candidate_submit_contract_reference_v1(),
        )?;
        let promotion_result_subscription = exact_permit(
            &permits,
            &review_note_candidate_promotion_result_contract_reference_v1(),
        )?;
        if permits.len() != 2 {
            return Err(ReviewNoteCandidateManagedRuntimeErrorV1::Admission);
        }
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            event_identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::EventUnavailable)?;
        let mut client_realtime = ReviewNoteCandidateClientRealtimePublisherV1::default();
        let mut dispatcher = RejectManagedControlRequestsV2;
        client_realtime
            .publish_pending(
                &persistence,
                &mut control_channel,
                &mut dispatcher,
                &admission.logical_human_owner_id,
            )
            .await
            .map_err(client_realtime_error)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control_channel,
            persistence,
            event_connection,
            event_publish_permit,
            submission_subscription,
            promotion_result_subscription,
            client_realtime,
        })
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, ReviewNoteCandidateManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?
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
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let response = dispatch_client(
            &self.persistence,
            &mut self.control_channel,
            &mut dispatcher,
            &self.admission,
            request,
            now_unix_millis,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
        if validate_module_client_response_v1(&response).is_err() {
            return Err(ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable);
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
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn consume_submission_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, ReviewNoteCandidateManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = consume_review_note_candidate_submission_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.submission_subscription,
            &mut self.control_channel,
            &mut dispatcher,
            &ReviewNoteCandidateSubmissionRuntimeContextV1 {
                logical_owner_id: &self.admission.logical_human_owner_id,
                runtime_instance_id: &self.admission.runtime_instance_id,
                runtime_generation: self.admission.runtime_generation,
                now_unix_millis,
            },
        )
        .await
        .map_err(submission_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
        result
    }

    pub async fn relay_outbox_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, ReviewNoteCandidateManagedRuntimeErrorV1> {
        relay_review_note_candidate_outbox_once_v1(
            &self.persistence,
            &self.admission.logical_human_owner_id,
            &self.event_connection,
            &self.event_publish_permit,
            now_unix_millis,
        )
        .await
        .map_err(event_relay_error)
    }

    pub async fn consume_promotion_result_once(
        &self,
    ) -> Result<bool, ReviewNoteCandidateManagedRuntimeErrorV1> {
        consume_review_note_candidate_promotion_result_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.promotion_result_subscription,
            &self.admission.logical_human_owner_id,
        )
        .await
        .map_err(promotion_result_error)
    }

    pub async fn pump_client_realtime_once(
        &mut self,
    ) -> Result<bool, ReviewNoteCandidateManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = self
            .client_realtime
            .publish_pending(
                &self.persistence,
                &mut self.control_channel,
                &mut dispatcher,
                &self.admission.logical_human_owner_id,
            )
            .await
            .map_err(client_realtime_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
        result
    }

    fn write_control_error(
        &mut self,
        correlation_id: [u8; 16],
        error_code: &str,
    ) -> Result<(), ReviewNoteCandidateManagedRuntimeErrorV1> {
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: error_code.to_owned(),
                },
            )
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)
    }
}

async fn dispatch_client(
    persistence: &ReviewNoteCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut RejectManagedControlRequestsV2,
    admission: &ReviewNoteCandidateRuntimeAdmissionV1,
    request: ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let valid_identity = request.protocol_major == 1
        && request.module_id == REVIEW_NOTE_CANDIDATE_MODULE_ID_V1
        && request.owner_id == REVIEW_NOTE_CANDIDATE_OWNER_V1
        && request.logical_owner_id == admission.logical_human_owner_id
        && !request.authenticated_device_id.is_empty();
    let (payload, accepted_route) = if valid_identity {
        if request.contract.as_ref() == Some(&command_contract_v1()) {
            (
                decide_payload_v1(
                    persistence,
                    channel,
                    dispatcher,
                    &ReviewNoteCandidateClientRuntimeContextV1 {
                        logical_owner_id: &admission.logical_human_owner_id,
                        authenticated_device_id: &request.authenticated_device_id,
                        runtime_instance_id: &admission.runtime_instance_id,
                        runtime_generation: admission.runtime_generation,
                        now_unix_millis,
                    },
                    &request.request_payload,
                )
                .await,
                true,
            )
        } else if request.contract.as_ref() == Some(&query_contract_v1()) {
            (
                get_payload_v1(
                    persistence,
                    &admission.logical_human_owner_id,
                    &request.request_payload,
                )
                .await,
                true,
            )
        } else {
            (Vec::new(), false)
        }
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

fn exact_permit(
    permits: &[RuntimeSubscribePermitV1],
    contract: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, ReviewNoteCandidateManagedRuntimeErrorV1> {
    let mut matching = permits.iter().filter(|permit| {
        permit.contract().is_some_and(|actual| {
            actual.owner == contract.owner
                && actual.name == contract.name
                && actual.major == contract.major
                && actual.revision == contract.revision
                && actual.schema_sha256 == contract.schema_sha256
        })
    });
    let permit = matching
        .next()
        .cloned()
        .ok_or(ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
    if matching.next().is_some() {
        return Err(ReviewNoteCandidateManagedRuntimeErrorV1::Admission);
    }
    Ok(permit)
}

fn validate_admission(
    admission: &ReviewNoteCandidateRuntimeAdmissionV1,
) -> Result<(), ReviewNoteCandidateManagedRuntimeErrorV1> {
    if admission.logical_owner_id != REVIEW_NOTE_CANDIDATE_OWNER_V1
        || admission.logical_human_owner_id.is_empty()
        || admission.logical_human_owner_id == admission.logical_owner_id
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(ReviewNoteCandidateManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &ReviewNoteCandidateRuntimeAdmissionV1,
) -> Result<(), ReviewNoteCandidateManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(ReviewNoteCandidateManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &ReviewNoteCandidateRuntimeAdmissionV1,
) -> Result<(), ReviewNoteCandidateManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ReviewNoteCandidateManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &ReviewNoteCandidateRuntimeAdmissionV1,
) -> Result<StorageBindingV1, ReviewNoteCandidateManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != REVIEW_NOTE_CANDIDATE_OWNER_V1
        || configuration.owner != REVIEW_NOTE_CANDIDATE_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(ReviewNoteCandidateManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| ReviewNoteCandidateManagedRuntimeErrorV1::Admission)
}

fn event_relay_error(
    error: ReviewNoteCandidateEventRelayErrorV1,
) -> ReviewNoteCandidateManagedRuntimeErrorV1 {
    match error {
        ReviewNoteCandidateEventRelayErrorV1::InvalidTimestamp => {
            ReviewNoteCandidateManagedRuntimeErrorV1::EventContract
        }
        ReviewNoteCandidateEventRelayErrorV1::Persistence(error) => {
            ReviewNoteCandidateManagedRuntimeErrorV1::Persistence(error)
        }
        ReviewNoteCandidateEventRelayErrorV1::EventUnavailable => {
            ReviewNoteCandidateManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn submission_error(
    error: ReviewNoteCandidateSubmissionErrorV1,
) -> ReviewNoteCandidateManagedRuntimeErrorV1 {
    match error {
        ReviewNoteCandidateSubmissionErrorV1::InvalidEnvelope
        | ReviewNoteCandidateSubmissionErrorV1::InvalidPayload
        | ReviewNoteCandidateSubmissionErrorV1::Blob(
            crate::blob_materialization::ReviewNoteCandidateBlobErrorV1::InvalidReceipt,
        ) => ReviewNoteCandidateManagedRuntimeErrorV1::EventContract,
        ReviewNoteCandidateSubmissionErrorV1::Blob(
            crate::blob_materialization::ReviewNoteCandidateBlobErrorV1::Unavailable,
        ) => ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable,
        ReviewNoteCandidateSubmissionErrorV1::Persistence(error) => {
            ReviewNoteCandidateManagedRuntimeErrorV1::Persistence(error)
        }
        ReviewNoteCandidateSubmissionErrorV1::EventUnavailable => {
            ReviewNoteCandidateManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn promotion_result_error(
    error: ReviewNoteCandidatePromotionResultErrorV1,
) -> ReviewNoteCandidateManagedRuntimeErrorV1 {
    match error {
        ReviewNoteCandidatePromotionResultErrorV1::InvalidEnvelope
        | ReviewNoteCandidatePromotionResultErrorV1::InvalidPayload => {
            ReviewNoteCandidateManagedRuntimeErrorV1::EventContract
        }
        ReviewNoteCandidatePromotionResultErrorV1::Persistence(error) => {
            ReviewNoteCandidateManagedRuntimeErrorV1::Persistence(error)
        }
        ReviewNoteCandidatePromotionResultErrorV1::EventUnavailable => {
            ReviewNoteCandidateManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn client_realtime_error(
    error: ReviewNoteCandidateClientRealtimeErrorV1,
) -> ReviewNoteCandidateManagedRuntimeErrorV1 {
    match error {
        ReviewNoteCandidateClientRealtimeErrorV1::InvalidTransition => {
            ReviewNoteCandidateManagedRuntimeErrorV1::InvalidTransition
        }
        ReviewNoteCandidateClientRealtimeErrorV1::Persistence(error) => {
            ReviewNoteCandidateManagedRuntimeErrorV1::Persistence(error)
        }
        ReviewNoteCandidateClientRealtimeErrorV1::Unavailable => {
            ReviewNoteCandidateManagedRuntimeErrorV1::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admission() -> ReviewNoteCandidateRuntimeAdmissionV1 {
        ReviewNoteCandidateRuntimeAdmissionV1 {
            logical_owner_id: REVIEW_NOTE_CANDIDATE_OWNER_V1.to_owned(),
            logical_human_owner_id: "owner-1".to_owned(),
            registration_id: "registration-1".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        }
    }

    #[test]
    fn admission_separates_review_owner_from_human_owner() {
        assert_eq!(validate_admission(&admission()), Ok(()));
        let mut invalid = admission();
        invalid.logical_human_owner_id = REVIEW_NOTE_CANDIDATE_OWNER_V1.to_owned();
        assert_eq!(
            validate_admission(&invalid),
            Err(ReviewNoteCandidateManagedRuntimeErrorV1::Admission)
        );
    }
}
