use std::{future::Future, os::unix::net::UnixStream, time::Duration};

use makosh_attachment_preview_evidence_replay_api::ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1;
use makosh_attachment_preview_evidence_replay_persistence::{
    AttachmentPreviewEvidenceReplayPersistenceV1, ReplayPersistenceErrorV1,
};
use makosh_communications_retained_evidence_replay_contract::communications_replay_result_contract_reference_v1;
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_mail_retained_evidence_replay_contract::mail_replay_result_contract_reference_v1;
use makosh_runtime_protocol::{
    managed_control::ManagedControlChannelV2,
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
    client_port::{ReplayClientRuntimeContextV1, dispatch_replay_client_request_v1},
    outbox::{ReplayCommandRelayErrorV1, relay_replay_commands_once_v1},
    result_consumer::{
        ReplayResultConsumerErrorV1, consume_next_communications_replay_result_v1,
        consume_next_mail_replay_result_v1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewEvidenceReplayRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(ReplayPersistenceErrorV1),
    Unavailable,
}

pub struct AttachmentPreviewEvidenceReplayManagedRuntimeV1 {
    admission: AttachmentPreviewEvidenceReplayRuntimeAdmissionV1,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: AttachmentPreviewEvidenceReplayPersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    communications_result_subscription: RuntimeSubscribePermitV1,
    mail_result_subscription: RuntimeSubscribePermitV1,
}

impl AttachmentPreviewEvidenceReplayManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &AttachmentPreviewEvidenceReplayRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission);
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
            .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
        let persistence = AttachmentPreviewEvidenceReplayPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Persistence)?;

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
        .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::EventUnavailable)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
        let permits = event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
        let communications_result_subscription = exact_permit(
            &permits,
            &communications_replay_result_contract_reference_v1(),
        )?;
        let mail_result_subscription =
            exact_permit(&permits, &mail_replay_result_contract_reference_v1())?;
        if permits.len() != 2 {
            return Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission);
        }
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::EventUnavailable)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control_channel,
            persistence,
            event_connection,
            event_publish_permit,
            communications_result_subscription,
            mail_result_subscription,
        })
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_seconds: i64,
        now_nanos: i32,
    ) -> Result<bool, AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            self.write_client_error(correlation_id, "managed_runtime_control_unexpected_request")?;
            return Ok(true);
        };
        let Some(request) = delivery
            .request
            .filter(|request| validate_module_client_request_v1(request).is_ok())
        else {
            self.write_client_error(
                correlation_id,
                "managed_runtime_control_invalid_client_delivery",
            )?;
            return Ok(true);
        };
        let response = dispatch_replay_client_request_v1(
            &self.persistence,
            &request,
            &ReplayClientRuntimeContextV1 {
                runtime_instance_id: self.admission.runtime_instance_id.clone(),
                runtime_generation: self.admission.runtime_generation,
                grant_epoch: self.admission.grant_epoch,
                now_unix_seconds,
                now_nanos,
            },
        )
        .await;
        if validate_module_client_response_v1(&response).is_err() {
            return Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable);
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
            .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn consume_communications_result_once(
        &self,
        now_unix_seconds: i64,
    ) -> Result<bool, AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
        bounded_consume(consume_next_communications_replay_result_v1(
            &self.persistence,
            &self.event_connection,
            &self.communications_result_subscription,
            now_unix_seconds,
        ))
        .await
    }

    pub async fn consume_mail_result_once(
        &self,
        now_unix_seconds: i64,
    ) -> Result<bool, AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
        bounded_consume(consume_next_mail_replay_result_v1(
            &self.persistence,
            &self.event_connection,
            &self.mail_result_subscription,
            now_unix_seconds,
        ))
        .await
    }

    pub async fn relay_commands_once(
        &self,
        now_unix_seconds: i64,
    ) -> Result<bool, AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
        relay_replay_commands_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.event_publish_permit,
            &self.event_publish_permit,
            now_unix_seconds,
        )
        .await
        .map(|published| published > 0)
        .map_err(relay_error)
    }

    fn write_client_error(
        &mut self,
        correlation_id: [u8; 16],
        error_code: &str,
    ) -> Result<(), AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: error_code.to_owned(),
                },
            )
            .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable)
    }
}

async fn bounded_consume<F>(
    future: F,
) -> Result<bool, AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1>
where
    F: Future<Output = Result<bool, ReplayResultConsumerErrorV1>>,
{
    // The Event Hub pull adapter owns the bounded 500 ms delivery deadline.
    // Cancelling it here before its server-side pull expires can strand an
    // already assigned message as unacknowledged until JetStream redelivery.
    future.await.map_err(consumer_error)
}

fn exact_permit(
    permits: &[RuntimeSubscribePermitV1],
    contract: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
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
        .ok_or(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
    if matching.next().is_some() {
        return Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission);
    }
    Ok(permit)
}

fn validate_admission(
    admission: &AttachmentPreviewEvidenceReplayRuntimeAdmissionV1,
) -> Result<(), AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
    if admission.logical_owner_id.is_empty()
        || admission.logical_owner_id == ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &AttachmentPreviewEvidenceReplayRuntimeAdmissionV1,
) -> Result<(), AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &AttachmentPreviewEvidenceReplayRuntimeAdmissionV1,
) -> Result<(), AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &AttachmentPreviewEvidenceReplayRuntimeAdmissionV1,
) -> Result<StorageBindingV1, AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.owner != ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Admission)
}

fn consumer_error(
    error: ReplayResultConsumerErrorV1,
) -> AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1 {
    match error {
        ReplayResultConsumerErrorV1::InvalidTimestamp
        | ReplayResultConsumerErrorV1::InvalidEnvelope => {
            AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::EventContract
        }
        ReplayResultConsumerErrorV1::Persistence(error) => {
            AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Persistence(error)
        }
        ReplayResultConsumerErrorV1::EventUnavailable => {
            AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn relay_error(
    error: ReplayCommandRelayErrorV1,
) -> AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1 {
    match error {
        ReplayCommandRelayErrorV1::InvalidTimestamp => {
            AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::EventContract
        }
        ReplayCommandRelayErrorV1::Persistence(error) => {
            AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::Persistence(error)
        }
        ReplayCommandRelayErrorV1::EventUnavailable => {
            AttachmentPreviewEvidenceReplayManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn result_consume_is_not_cancelled_before_the_transport_deadline() {
        let consumed = bounded_consume(async {
            tokio::time::sleep(Duration::from_millis(30)).await;
            Ok::<bool, ReplayResultConsumerErrorV1>(true)
        })
        .await;

        assert_eq!(consumed, Ok(true));
    }
}
