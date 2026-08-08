//! Managed Archive Inspection worker composition with event-only owner boundaries.

use std::{
    os::unix::net::UnixStream,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use makosh_attachment_archive_inspection_api::{
    ATTACHMENT_ARCHIVE_INSPECTION_MODULE_ID_V1, ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1,
};
use makosh_attachment_archive_inspection_core::{
    ArchiveInspectionErrorV1, ArchiveInspectionLimitsV1, ArchiveInspectionPolicyErrorV1,
};
use makosh_attachment_archive_inspection_ingress::{
    ArchiveInspectionCustodyEnvelopeContextV1,
    archive_inspection_custody_delegated_contract_reference_v1,
    archive_inspection_custody_delegation_rejected_contract_reference_v1,
    build_request_archive_inspection_custody_delegation_outbox_record_v1,
};
use makosh_attachment_archive_inspection_persistence::{
    ArchiveInspectionPersistenceErrorV1, AttachmentArchiveInspectionPersistenceV1,
};
use makosh_attachment_archive_inspection_zip::inspect_zip_bytes_v1;
use makosh_attachment_security_contract::admission::attachment_security_scan_candidate_observed_contract_reference_v1;
use makosh_communications_attachment_contract::admission::communication_attachment_safety_state_changed_contract_reference_v1;
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
    try_receive_runtime_pull_delivery,
};
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
    blob::{read_archive_blob_v1, transfer_archive_blob_v1},
    client_port::dispatch_archive_inspection_client_request_v1,
    client_realtime::{
        ArchiveInspectionClientRealtimeErrorV1, ArchiveInspectionClientRealtimePublisherV1,
    },
    event_decode::{
        DecodedArchiveCustodyResultV1, decode_archive_candidate_v1,
        decode_archive_custody_result_v1, decode_archive_safety_v1,
    },
    outbox::relay_archive_custody_outbox_once_v1,
};

const WORKER_ID: &str = "attachment-archive-inspection-runtime";
const JOB_LEASE_MILLIS: u64 = 180_000;

pub struct ArchiveInspectionRuntimeAdmissionV1 {
    pub module_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

pub struct AttachmentArchiveInspectionRuntimeV1 {
    control_channel: ManagedControlChannelV2<UnixStream>,
    connection: RuntimeJetStreamConnection,
    permits: ArchiveInspectionSubscribePermitsV1,
    next_consumer: ArchiveInspectionConsumerV1,
    publish_permit: RuntimePublishPermitV1,
    persistence: AttachmentArchiveInspectionPersistenceV1,
    client_realtime: ArchiveInspectionClientRealtimePublisherV1,
    limits: ArchiveInspectionLimitsV1,
    logical_human_owner_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

impl AttachmentArchiveInspectionRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &ArchiveInspectionRuntimeAdmissionV1,
        event_hub_endpoint: &str,
        credential_revision: u64,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        limits: ArchiveInspectionLimitsV1,
    ) -> Result<Self, ArchiveInspectionRuntimeErrorV1> {
        validate_open(
            &descriptor_bytes,
            &settings_schema_bytes,
            admission,
            event_hub_endpoint,
            credential_revision,
        )?;
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let access = request_managed_runtime_event_access_v2(
            &mut control_channel,
            &admission.module_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            credential_revision,
        )
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
        let permits = ArchiveInspectionSubscribePermitsV1::bind(
            access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?,
        )?;
        let publish_permit = access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?;
        let connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            public_key,
        )
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?;
        let persistence = AttachmentArchiveInspectionPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
        let mut control_channel = leases.into_route_port().into_channel();
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            control_channel,
            connection,
            permits,
            next_consumer: ArchiveInspectionConsumerV1::Candidate,
            publish_permit,
            persistence,
            client_realtime: ArchiveInspectionClientRealtimePublisherV1::default(),
            limits,
            logical_human_owner_id: admission.logical_human_owner_id.clone(),
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
    }

    pub async fn consume_next(
        &mut self,
        consumed_at_unix_millis: i64,
    ) -> Result<bool, ArchiveInspectionRuntimeErrorV1> {
        let consumer = self.next_consumer;
        self.next_consumer = consumer.successor();
        let permit = self.permits.for_consumer(consumer);
        let delivery = match try_receive_runtime_pull_delivery(&self.connection, permit)
            .await
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?
        {
            None => return Ok(false),
            Some(delivery) => delivery,
        };
        match consumer {
            ArchiveInspectionConsumerV1::Candidate => {
                let decoded = decode_archive_candidate_v1(delivery.exact_bytes())
                    .map_err(|_| ArchiveInspectionRuntimeErrorV1::InvalidDelivery)?;
                self.persistence
                    .persist_scan_candidate(
                        &self.logical_human_owner_id,
                        &decoded.fact,
                        decoded.envelope_sha256,
                        consumed_at_unix_millis,
                    )
                    .await
                    .map_err(persistence_error)?;
            }
            ArchiveInspectionConsumerV1::Safety => {
                if let Some(decoded) = decode_archive_safety_v1(delivery.exact_bytes())
                    .map_err(|_| ArchiveInspectionRuntimeErrorV1::InvalidDelivery)?
                {
                    self.persistence
                        .persist_canonical_safety_fact(
                            &self.logical_human_owner_id,
                            &decoded.fact,
                            decoded.envelope_sha256,
                            consumed_at_unix_millis,
                        )
                        .await
                        .map_err(persistence_error)?;
                }
            }
            ArchiveInspectionConsumerV1::Delegated | ArchiveInspectionConsumerV1::Rejected => {
                let decoded = decode_archive_custody_result_v1(delivery.exact_bytes())
                    .map_err(|_| ArchiveInspectionRuntimeErrorV1::InvalidDelivery)?;
                match decoded {
                    DecodedArchiveCustodyResultV1::Delegated {
                        message_id,
                        envelope_sha256,
                        command_message_id,
                        payload,
                    } => {
                        self.persistence
                            .persist_custody_delegated_result(
                                message_id,
                                envelope_sha256,
                                command_message_id,
                                &payload,
                                consumed_at_unix_millis,
                            )
                            .await
                            .map_err(persistence_error)?;
                    }
                    DecodedArchiveCustodyResultV1::Rejected {
                        message_id,
                        envelope_sha256,
                        command_message_id,
                        payload,
                    } => {
                        self.persistence
                            .persist_custody_delegation_rejected_result(
                                message_id,
                                envelope_sha256,
                                command_message_id,
                                &payload,
                                consumed_at_unix_millis,
                            )
                            .await
                            .map_err(persistence_error)?;
                    }
                }
            }
        }
        delivery
            .acknowledge()
            .await
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn materialize_pending_custody_requests(
        &self,
        now_unix_millis: i64,
        now_nanos: i32,
    ) -> Result<usize, ArchiveInspectionRuntimeErrorV1> {
        let pending = self
            .persistence
            .pending_custody_delegation_requests(&self.logical_human_owner_id, 64)
            .await
            .map_err(persistence_error)?;
        for item in &pending {
            let recorded_at_unix_seconds = now_unix_millis / 1_000;
            let record = build_request_archive_inspection_custody_delegation_outbox_record_v1(
                item.request.clone(),
                recorded_at_unix_seconds
                    .checked_add(300)
                    .ok_or(ArchiveInspectionRuntimeErrorV1::Unavailable)?,
                &ArchiveInspectionCustodyEnvelopeContextV1 {
                    module_id: ATTACHMENT_ARCHIVE_INSPECTION_MODULE_ID_V1.to_owned(),
                    runtime_instance_id: self.runtime_instance_id.clone(),
                    runtime_generation: self.runtime_generation,
                    recorded_at_unix_seconds,
                    recorded_at_nanos: now_nanos,
                },
            )
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::InvalidJob)?;
            self.persistence
                .store_custody_delegation_outbox(
                    &self.logical_human_owner_id,
                    &record,
                    now_unix_millis,
                )
                .await
                .map_err(persistence_error)?;
        }
        Ok(pending.len())
    }

    pub async fn relay_custody_outbox(
        &self,
        now_unix_millis: i64,
    ) -> Result<usize, ArchiveInspectionRuntimeErrorV1> {
        relay_archive_custody_outbox_once_v1(
            &self.persistence,
            &self.logical_human_owner_id,
            &self.connection,
            &self.publish_permit,
            now_unix_millis,
        )
        .await
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)
    }

    pub async fn process_next_job(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<ArchiveInspectionJobTickV1, ArchiveInspectionRuntimeErrorV1> {
        self.persistence
            .recover_expired_jobs(&self.logical_human_owner_id, now_unix_millis)
            .await
            .map_err(persistence_error)?;
        let Some(claimed) = self
            .persistence
            .claim_next_job(
                &self.logical_human_owner_id,
                WORKER_ID,
                self.runtime_generation,
                self.grant_epoch,
                now_unix_millis,
                JOB_LEASE_MILLIS,
            )
            .await
            .map_err(persistence_error)?
        else {
            return Ok(ArchiveInspectionJobTickV1::Idle);
        };
        let receipt = match claimed.target_blob_receipt {
            Some(receipt) => receipt,
            None => {
                let receipt = transfer_archive_blob_v1(&mut self.control_channel, &claimed)
                    .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
                self.persistence
                    .record_target_blob_receipt(&claimed, receipt, now_unix_millis)
                    .await
                    .map_err(persistence_error)?;
                receipt
            }
        };
        let bytes = read_archive_blob_v1(&mut self.control_channel, &claimed, receipt)
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
        if !zip_signature(&bytes) {
            self.persistence
                .reject_job(&claimed, ArchiveInspectionErrorV1::NotZip, now_unix_millis)
                .await
                .map_err(persistence_error)?;
            return Ok(ArchiveInspectionJobTickV1::Rejected(
                ArchiveInspectionErrorV1::NotZip,
            ));
        }
        match inspect_zip_bytes_v1(&bytes, self.limits) {
            Ok(report) => {
                self.persistence
                    .complete_job(&claimed, &report, now_unix_millis)
                    .await
                    .map_err(persistence_error)?;
                Ok(ArchiveInspectionJobTickV1::Completed)
            }
            Err(error) => {
                let error = policy_error(error);
                self.persistence
                    .reject_job(&claimed, error, now_unix_millis)
                    .await
                    .map_err(persistence_error)?;
                Ok(ArchiveInspectionJobTickV1::Rejected(error))
            }
        }
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, ArchiveInspectionRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?
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
        let response = dispatch_archive_inspection_client_request_v1(
            &self.persistence,
            &self.logical_human_owner_id,
            request,
            now_unix_millis,
        )
        .await;
        if validate_module_client_response_v1(&response).is_err() {
            return Err(ArchiveInspectionRuntimeErrorV1::Unavailable);
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
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn pump_client_realtime_once(
        &mut self,
    ) -> Result<bool, ArchiveInspectionRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = self
            .client_realtime
            .publish_pending(
                &self.persistence,
                &mut self.control_channel,
                &mut dispatcher,
                &self.logical_human_owner_id,
            )
            .await
            .map_err(client_realtime_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
        result
    }

    fn write_client_error(
        &mut self,
        correlation_id: [u8; 16],
        error_code: &str,
    ) -> Result<(), ArchiveInspectionRuntimeErrorV1> {
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: error_code.to_owned(),
                },
            )
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)
    }
}

struct ArchiveInspectionSubscribePermitsV1 {
    candidate: RuntimeSubscribePermitV1,
    safety: RuntimeSubscribePermitV1,
    delegated: RuntimeSubscribePermitV1,
    rejected: RuntimeSubscribePermitV1,
}

impl ArchiveInspectionSubscribePermitsV1 {
    fn bind(
        permits: Vec<RuntimeSubscribePermitV1>,
    ) -> Result<Self, ArchiveInspectionRuntimeErrorV1> {
        let expected = [
            attachment_security_scan_candidate_observed_contract_reference_v1(),
            communication_attachment_safety_state_changed_contract_reference_v1(),
            archive_inspection_custody_delegated_contract_reference_v1(),
            archive_inspection_custody_delegation_rejected_contract_reference_v1(),
        ];
        let mut selected: [Option<RuntimeSubscribePermitV1>; 4] = [None, None, None, None];
        for permit in permits {
            let Some(contract) = permit.contract() else {
                return Err(ArchiveInspectionRuntimeErrorV1::Admission);
            };
            let Some(index) = expected
                .iter()
                .position(|expected| exact_contract(contract, expected))
            else {
                return Err(ArchiveInspectionRuntimeErrorV1::Admission);
            };
            if selected[index].replace(permit).is_some() {
                return Err(ArchiveInspectionRuntimeErrorV1::Admission);
            }
        }
        let [candidate, safety, delegated, rejected] = selected;
        Ok(Self {
            candidate: candidate.ok_or(ArchiveInspectionRuntimeErrorV1::Admission)?,
            safety: safety.ok_or(ArchiveInspectionRuntimeErrorV1::Admission)?,
            delegated: delegated.ok_or(ArchiveInspectionRuntimeErrorV1::Admission)?,
            rejected: rejected.ok_or(ArchiveInspectionRuntimeErrorV1::Admission)?,
        })
    }

    fn for_consumer(&self, consumer: ArchiveInspectionConsumerV1) -> &RuntimeSubscribePermitV1 {
        match consumer {
            ArchiveInspectionConsumerV1::Candidate => &self.candidate,
            ArchiveInspectionConsumerV1::Safety => &self.safety,
            ArchiveInspectionConsumerV1::Delegated => &self.delegated,
            ArchiveInspectionConsumerV1::Rejected => &self.rejected,
        }
    }
}

#[derive(Clone, Copy)]
enum ArchiveInspectionConsumerV1 {
    Candidate,
    Safety,
    Delegated,
    Rejected,
}

impl ArchiveInspectionConsumerV1 {
    const fn successor(self) -> Self {
        match self {
            Self::Candidate => Self::Safety,
            Self::Safety => Self::Delegated,
            Self::Delegated => Self::Rejected,
            Self::Rejected => Self::Candidate,
        }
    }
}

fn zip_signature(bytes: &[u8]) -> bool {
    matches!(
        bytes.get(..4),
        Some(b"PK\x03\x04" | b"PK\x05\x06" | b"PK\x07\x08")
    )
}

fn policy_error(error: ArchiveInspectionPolicyErrorV1) -> ArchiveInspectionErrorV1 {
    match error {
        ArchiveInspectionPolicyErrorV1::MalformedArchive => {
            ArchiveInspectionErrorV1::CorruptArchive
        }
        ArchiveInspectionPolicyErrorV1::InvalidLimits => ArchiveInspectionErrorV1::Unavailable,
        _ => ArchiveInspectionErrorV1::PolicyRejected,
    }
}

fn persistence_error(
    error: ArchiveInspectionPersistenceErrorV1,
) -> ArchiveInspectionRuntimeErrorV1 {
    match error {
        ArchiveInspectionPersistenceErrorV1::InvalidInput
        | ArchiveInspectionPersistenceErrorV1::InvalidRow
        | ArchiveInspectionPersistenceErrorV1::EvidenceConflict => {
            ArchiveInspectionRuntimeErrorV1::InvalidJob
        }
        ArchiveInspectionPersistenceErrorV1::StorageUnavailable
        | ArchiveInspectionPersistenceErrorV1::ClaimLost => {
            ArchiveInspectionRuntimeErrorV1::Unavailable
        }
    }
}

fn client_realtime_error(
    error: ArchiveInspectionClientRealtimeErrorV1,
) -> ArchiveInspectionRuntimeErrorV1 {
    match error {
        ArchiveInspectionClientRealtimeErrorV1::InvalidTransition => {
            ArchiveInspectionRuntimeErrorV1::InvalidJob
        }
        ArchiveInspectionClientRealtimeErrorV1::Persistence(error) => persistence_error(error),
        ArchiveInspectionClientRealtimeErrorV1::Unavailable => {
            ArchiveInspectionRuntimeErrorV1::Unavailable
        }
    }
}

fn validate_open(
    descriptor: &[u8],
    settings: &[u8],
    admission: &ArchiveInspectionRuntimeAdmissionV1,
    event_hub_endpoint: &str,
    credential_revision: u64,
) -> Result<(), ArchiveInspectionRuntimeErrorV1> {
    if descriptor.is_empty()
        || settings.is_empty()
        || admission.module_owner_id != ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1
        || !valid_owner_id(&admission.logical_human_owner_id)
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
        || credential_revision == 0
        || event_hub_endpoint.is_empty()
    {
        return Err(ArchiveInspectionRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn valid_owner_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &ArchiveInspectionRuntimeAdmissionV1,
) -> Result<(), ArchiveInspectionRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(ArchiveInspectionRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &ArchiveInspectionRuntimeAdmissionV1,
) -> Result<(), ArchiveInspectionRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ArchiveInspectionRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(lease_id) = leases.issue_runtime_credential(binding).await
            && let Ok(password) = leases.resolve_runtime_credential(binding, lease_id).await
        {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    Err(ArchiveInspectionRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &ArchiveInspectionRuntimeAdmissionV1,
) -> Result<StorageBindingV1, ArchiveInspectionRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1
        || configuration.owner != ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(ArchiveInspectionRuntimeErrorV1::Admission);
    }
    StorageBindingV1::new(
        StorageBindingIdentityV1::new(
            configuration.storage_instance_id.clone(),
            configuration.database_id.clone(),
            configuration.owner.clone(),
            admission.registration_id.clone(),
            configuration.runtime_instance_id.clone(),
        )
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?,
        StorageBindingFencesV1::new(
            configuration.storage_generation,
            admission.runtime_generation,
            admission.grant_epoch,
            configuration.role_epoch,
            configuration.credential_revision,
            configuration.storage_bundle_revision,
        )
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?,
        StorageBindingAccessV1::new(
            configuration.runtime_principal.clone(),
            configuration.pool_alias.clone(),
            StorageEffectiveBudgetsV1::new(
                u16::try_from(configuration.max_connections)
                    .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?,
                configuration.statement_timeout_millis,
            )
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?,
            configuration
                .storage_bundle_digest
                .as_slice()
                .try_into()
                .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?,
        )
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| ArchiveInspectionRuntimeErrorV1::Admission)
}

fn exact_contract(left: &ContractReferenceV1, right: &ContractReferenceV1) -> bool {
    left.owner == right.owner
        && left.name == right.name
        && left.major == right.major
        && left.revision == right.revision
        && left.schema_sha256 == right.schema_sha256
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionJobTickV1 {
    Idle,
    Completed,
    Rejected(ArchiveInspectionErrorV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionRuntimeErrorV1 {
    Admission,
    InvalidDelivery,
    InvalidJob,
    Unavailable,
}

pub fn current_runtime_time_v1() -> Result<(i64, i32), ArchiveInspectionRuntimeErrorV1> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?;
    Ok((
        i64::try_from(now.as_millis()).map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?,
        i32::try_from(now.subsec_nanos())
            .map_err(|_| ArchiveInspectionRuntimeErrorV1::Unavailable)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumer_rotation_covers_each_exact_route() {
        let mut consumer = ArchiveInspectionConsumerV1::Candidate;
        consumer = consumer.successor();
        assert!(matches!(consumer, ArchiveInspectionConsumerV1::Safety));
        consumer = consumer.successor();
        assert!(matches!(consumer, ArchiveInspectionConsumerV1::Delegated));
        consumer = consumer.successor();
        assert!(matches!(consumer, ArchiveInspectionConsumerV1::Rejected));
        assert!(matches!(
            consumer.successor(),
            ArchiveInspectionConsumerV1::Candidate
        ));
    }

    #[test]
    fn zip_signature_is_bounded_and_exact() {
        assert!(zip_signature(b"PK\x03\x04rest"));
        assert!(zip_signature(b"PK\x05\x06"));
        assert!(!zip_signature(b"not zip"));
    }
}
