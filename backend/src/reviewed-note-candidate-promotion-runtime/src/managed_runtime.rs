use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_knowledge_command_api::{
    knowledge_note_created_from_reviewed_candidate_contract_reference_v1,
    knowledge_note_creation_from_reviewed_candidate_rejected_contract_reference_v1,
};
use makosh_review_note_candidate_api::review_note_candidate_approved_contract_reference_v1;
use makosh_reviewed_note_candidate_promotion_core::REVIEWED_NOTE_CANDIDATE_PROMOTION_OWNER_V1;
use makosh_reviewed_note_candidate_promotion_persistence::{
    ReviewedNoteCandidatePromotionPersistenceErrorV1, ReviewedNoteCandidatePromotionPersistenceV1,
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
    approval::consume_approval_once_v1,
    event_outbox::{PromotionEventRelayErrorV1, relay_promotion_outbox_once_v1},
    note_results::{
        ReviewedNoteCandidatePromotionEventErrorV1, ReviewedNoteCandidatePromotionRuntimeContextV1,
        consume_note_created_once_v1, consume_note_rejected_once_v1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedNoteCandidatePromotionRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedNoteCandidatePromotionManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(ReviewedNoteCandidatePromotionPersistenceErrorV1),
    Unavailable,
}

pub struct ReviewedNoteCandidatePromotionManagedRuntimeV1 {
    admission: ReviewedNoteCandidatePromotionRuntimeAdmissionV1,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: ReviewedNoteCandidatePromotionPersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    approval_subscription: RuntimeSubscribePermitV1,
    note_created_subscription: RuntimeSubscribePermitV1,
    note_rejected_subscription: RuntimeSubscribePermitV1,
}

impl ReviewedNoteCandidatePromotionManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &ReviewedNoteCandidatePromotionRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission);
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
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let persistence = ReviewedNoteCandidatePromotionPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Persistence)?;

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
        .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::EventUnavailable)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let permits = event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
        let approval_subscription = exact_permit(
            &permits,
            &review_note_candidate_approved_contract_reference_v1(),
        )?;
        let note_created_subscription = exact_permit(
            &permits,
            &knowledge_note_created_from_reviewed_candidate_contract_reference_v1(),
        )?;
        let note_rejected_subscription = exact_permit(
            &permits,
            &knowledge_note_creation_from_reviewed_candidate_rejected_contract_reference_v1(),
        )?;
        if permits.len() != 3 {
            return Err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission);
        }
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::EventUnavailable)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control_channel,
            persistence,
            event_connection,
            event_publish_permit,
            approval_subscription,
            note_created_subscription,
            note_rejected_subscription,
        })
    }

    pub fn pump_control_once(
        &mut self,
    ) -> Result<bool, ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
        let Some((correlation_id, _request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)?
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
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn consume_approval_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
        let context = ReviewedNoteCandidatePromotionRuntimeContextV1 {
            logical_human_owner_id: &self.admission.logical_owner_id,
            runtime_instance_id: &self.admission.runtime_instance_id,
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis,
        };
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = consume_approval_once_v1(
            &self.persistence,
            &mut self.control_channel,
            &mut dispatcher,
            &self.event_connection,
            &self.approval_subscription,
            &context,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
        result.map_err(event_error)
    }

    pub async fn consume_note_created_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
        consume_note_created_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.note_created_subscription,
            &self.context(now_unix_millis),
        )
        .await
        .map_err(event_error)
    }

    pub async fn consume_note_rejected_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
        consume_note_rejected_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.note_rejected_subscription,
            &self.context(now_unix_millis),
        )
        .await
        .map_err(event_error)
    }

    pub async fn relay_outbox_once(
        &self,
        now_unix_millis: i64,
    ) -> Result<bool, ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
        relay_promotion_outbox_once_v1(
            &self.persistence,
            &self.admission.logical_owner_id,
            &self.event_connection,
            &self.event_publish_permit,
            now_unix_millis,
        )
        .await
        .map_err(relay_error)
    }

    fn context(&self, now_unix_millis: i64) -> ReviewedNoteCandidatePromotionRuntimeContextV1<'_> {
        ReviewedNoteCandidatePromotionRuntimeContextV1 {
            logical_human_owner_id: &self.admission.logical_owner_id,
            runtime_instance_id: &self.admission.runtime_instance_id,
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis,
        }
    }
}

fn exact_permit(
    permits: &[RuntimeSubscribePermitV1],
    contract: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
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
        .ok_or(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
    if matching.next().is_some() {
        return Err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission);
    }
    Ok(permit)
}

fn validate_admission(
    admission: &ReviewedNoteCandidatePromotionRuntimeAdmissionV1,
) -> Result<(), ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
    if admission.logical_owner_id.is_empty()
        || admission.logical_owner_id == REVIEWED_NOTE_CANDIDATE_PROMOTION_OWNER_V1
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &ReviewedNoteCandidatePromotionRuntimeAdmissionV1,
) -> Result<(), ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &ReviewedNoteCandidatePromotionRuntimeAdmissionV1,
) -> Result<(), ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &ReviewedNoteCandidatePromotionRuntimeAdmissionV1,
) -> Result<StorageBindingV1, ReviewedNoteCandidatePromotionManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.owner != REVIEWED_NOTE_CANDIDATE_PROMOTION_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Admission)
}

fn event_error(
    error: ReviewedNoteCandidatePromotionEventErrorV1,
) -> ReviewedNoteCandidatePromotionManagedRuntimeErrorV1 {
    match error {
        ReviewedNoteCandidatePromotionEventErrorV1::InvalidEnvelope
        | ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload => {
            ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::EventContract
        }
        ReviewedNoteCandidatePromotionEventErrorV1::Persistence(error) => {
            ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Persistence(error)
        }
        ReviewedNoteCandidatePromotionEventErrorV1::EventUnavailable => {
            ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn relay_error(
    error: PromotionEventRelayErrorV1,
) -> ReviewedNoteCandidatePromotionManagedRuntimeErrorV1 {
    match error {
        PromotionEventRelayErrorV1::InvalidTimestamp => {
            ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::EventContract
        }
        PromotionEventRelayErrorV1::Persistence(error) => {
            ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::Persistence(error)
        }
        PromotionEventRelayErrorV1::EventUnavailable => {
            ReviewedNoteCandidatePromotionManagedRuntimeErrorV1::EventUnavailable
        }
    }
}
