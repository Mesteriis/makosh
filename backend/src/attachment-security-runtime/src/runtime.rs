//! Kernel-fenced runtime composition for durable scan execution.

use std::{
    os::unix::net::UnixStream,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use makosh_attachment_archive_inspection_ingress::archive_inspection_custody_delegation_requested_contract_reference_v1;
use makosh_attachment_preview_ingress::attachment_preview_custody_delegation_requested_contract_reference_v1;
use makosh_attachment_security_contract::admission::attachment_security_scan_candidate_observed_contract_reference_v1;
use makosh_attachment_security_core::{
    AttachmentSecurityJoinPolicyV1, AttachmentSecurityVerdictV1,
    decide_attachment_security_verdict_v1,
};
use makosh_attachment_security_persistence::{
    AttachmentSecurityPersistenceErrorV1, AttachmentSecurityPersistenceV1,
    AttachmentSecurityRetryPolicyV1, ClaimedAttachmentSecurityArchiveDelegationV1,
    ClaimedAttachmentSecurityPreviewDelegationV1, ClaimedAttachmentSecurityScanJobV1,
    ClaimedAttachmentSecurityTextDelegationV1,
};
use makosh_attachment_text_extraction_ingress::attachment_text_custody_delegation_requested_contract_reference_v1;
use makosh_communications_attachment_contract::{
    AttachmentObservationEnvelopeContextV1, AttachmentSafetyExpectedStateV1,
    AttachmentSafetyVerdictFactV1,
    AttachmentSafetyVerdictV1 as CommunicationsAttachmentSafetyVerdictV1,
    admission::communication_attachment_safety_state_changed_contract_reference_v1,
    build_attachment_safety_verdict_outbox_record_v1,
};
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
    try_receive_runtime_pull_delivery,
};
use makosh_runtime_protocol::{
    managed_control::ManagedControlChannelV2,
    v1::{ContractReferenceV1, ManagedRuntimeReadyRequestV1, ManagedStorageRuntimeConfigurationV1},
};
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};

use crate::{
    admission::{ATTACHMENT_SECURITY_MODULE_ID, ATTACHMENT_SECURITY_OWNER_ID},
    delegation::{
        AttachmentSecurityArchiveDelegationErrorV1, materialize_archive_delegation_exhausted_v1,
        materialize_archive_delegation_result_v1,
    },
    event_decode::{decode_canonical_state_v1, decode_scan_candidate_v1},
    outbox::{
        relay_attachment_security_archive_delegation_outbox_once_v1,
        relay_attachment_security_preview_delegation_outbox_once_v1,
        relay_attachment_security_text_delegation_outbox_once_v1,
        relay_attachment_security_verdict_outbox_once_v1,
    },
    preview_delegation::{
        AttachmentSecurityPreviewDelegationErrorV1, materialize_preview_delegation_exhausted_v1,
        materialize_preview_delegation_result_v1,
    },
    scan::AttachmentSecurityScannerV1,
    settings::AttachmentSecurityRuntimeSettingsV1,
    text_delegation::{
        AttachmentSecurityTextDelegationErrorV1, materialize_text_delegation_exhausted_v1,
        materialize_text_delegation_result_v1,
    },
};

const SCAN_JOB_WORKER_ID: &str = "attachment-security-runtime";
const SCAN_JOB_MAX_ATTEMPTS: u32 = 8;
const SCAN_JOB_LEASE_SECONDS: i64 = 180;
const ARCHIVE_DELEGATION_WORKER_ID: &str = "attachment-security-archive-delegation";
const ARCHIVE_DELEGATION_LEASE_SECONDS: i64 = 30;
const TEXT_DELEGATION_WORKER_ID: &str = "attachment-security-text-delegation";
const TEXT_DELEGATION_LEASE_SECONDS: i64 = 30;
const PREVIEW_DELEGATION_WORKER_ID: &str = "attachment-security-preview-delegation";
const PREVIEW_DELEGATION_LEASE_SECONDS: i64 = 30;

pub struct AttachmentSecurityRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

pub struct AttachmentSecurityRuntimeV1 {
    control_channel: ManagedControlChannelV2<UnixStream>,
    connection: RuntimeJetStreamConnection,
    permits: AttachmentSecuritySubscribePermitsV1,
    next_consumer: AttachmentSecurityConsumerV1,
    verdict_publish_permit: RuntimePublishPermitV1,
    persistence: AttachmentSecurityPersistenceV1,
    scanner: AttachmentSecurityScannerV1,
    join_policy: AttachmentSecurityJoinPolicyV1,
    retry_policy: AttachmentSecurityRetryPolicyV1,
    runtime_instance_id: String,
    runtime_generation: u64,
}

impl AttachmentSecurityRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &AttachmentSecurityRuntimeAdmissionV1,
        event_hub_endpoint: &str,
        credential_revision: u64,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        settings: AttachmentSecurityRuntimeSettingsV1,
    ) -> Result<Self, AttachmentSecurityRuntimeErrorV1> {
        validate_open_input(
            &descriptor_bytes,
            &settings_schema_bytes,
            admission,
            event_hub_endpoint,
            credential_revision,
        )?;
        let join_policy = AttachmentSecurityJoinPolicyV1::new(settings.max_scan_bytes)
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
        let retry_policy = AttachmentSecurityRetryPolicyV1::new(SCAN_JOB_MAX_ATTEMPTS)
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
        let scanner = AttachmentSecurityScannerV1::new(settings)
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate_managed_runtime_v2(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let access = request_managed_runtime_event_access_v2(
            &mut control_channel,
            &admission.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            credential_revision,
        )
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        let permits = access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
        let permits = AttachmentSecuritySubscribePermitsV1::bind(permits)?;
        let verdict_publish_permit = access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
        let connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let vault_public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_runtime_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
        let persistence = AttachmentSecurityPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        persistence
            .reconcile_retry_policies_v3()
            .await
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        let mut control_channel = leases.into_route_port().into_channel();
        signal_managed_runtime_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            control_channel,
            connection,
            permits,
            next_consumer: AttachmentSecurityConsumerV1::Candidate,
            verdict_publish_permit,
            persistence,
            scanner,
            join_policy,
            retry_policy,
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
        })
    }

    pub async fn consume_next(
        &mut self,
        consumed_at_unix_seconds: i64,
    ) -> Result<bool, AttachmentSecurityRuntimeErrorV1> {
        let consumer = self.next_consumer;
        self.next_consumer = consumer.successor();
        let permit = match consumer {
            AttachmentSecurityConsumerV1::Candidate => &self.permits.candidate,
            AttachmentSecurityConsumerV1::CanonicalState => &self.permits.canonical_state,
            AttachmentSecurityConsumerV1::ArchiveDelegation => &self.permits.archive_delegation,
            AttachmentSecurityConsumerV1::PreviewDelegation => &self.permits.preview_delegation,
            AttachmentSecurityConsumerV1::TextDelegation => &self.permits.text_delegation,
        };
        let delivery = match try_receive_runtime_pull_delivery(&self.connection, permit)
            .await
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?
        {
            None => return Ok(false),
            Some(delivery) => delivery,
        };
        let result = match consumer {
            AttachmentSecurityConsumerV1::Candidate => {
                let decoded = decode_scan_candidate_v1(delivery.exact_bytes())
                    .map_err(|_| AttachmentSecurityRuntimeErrorV1::InvalidDelivery)?;
                Some(
                    self.persistence
                        .persist_scan_candidate(
                            &decoded.fact,
                            decoded.envelope_sha256,
                            self.join_policy,
                            self.retry_policy,
                            consumed_at_unix_seconds,
                        )
                        .await
                        .map(|_| ()),
                )
            }
            AttachmentSecurityConsumerV1::CanonicalState => {
                let decoded = decode_canonical_state_v1(delivery.exact_bytes())
                    .map_err(|_| AttachmentSecurityRuntimeErrorV1::InvalidDelivery)?;
                match decoded {
                    Some(decoded) => Some(
                        self.persistence
                            .persist_canonical_state(
                                &decoded.fact,
                                decoded.envelope_sha256,
                                self.join_policy,
                                self.retry_policy,
                                consumed_at_unix_seconds,
                            )
                            .await
                            .map(|_| ()),
                    ),
                    None => None,
                }
            }
            AttachmentSecurityConsumerV1::ArchiveDelegation => Some(
                self.persistence
                    .persist_archive_delegation_request(
                        delivery.exact_bytes(),
                        consumed_at_unix_seconds,
                    )
                    .await
                    .map(|_| ()),
            ),
            AttachmentSecurityConsumerV1::PreviewDelegation => Some(
                self.persistence
                    .persist_preview_delegation_request(
                        delivery.exact_bytes(),
                        consumed_at_unix_seconds,
                    )
                    .await
                    .map(|_| ()),
            ),
            AttachmentSecurityConsumerV1::TextDelegation => Some(
                self.persistence
                    .persist_text_delegation_request(
                        delivery.exact_bytes(),
                        consumed_at_unix_seconds,
                    )
                    .await
                    .map(|_| ()),
            ),
        };
        if let Some(result) = result {
            result.map_err(persistence_error)?;
        }
        delivery
            .acknowledge()
            .await
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn process_next_scan_job(
        &mut self,
        observed_at_unix_seconds: i64,
        observed_at_nanos: i32,
    ) -> Result<AttachmentSecurityScanTickV1, AttachmentSecurityRuntimeErrorV1> {
        let lease_expires_at = observed_at_unix_seconds
            .checked_add(SCAN_JOB_LEASE_SECONDS)
            .ok_or(AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        let Some(claimed) = self
            .persistence
            .claim_next_scan_job(
                SCAN_JOB_WORKER_ID,
                observed_at_unix_seconds,
                lease_expires_at,
            )
            .await
            .map_err(persistence_error)?
        else {
            return Ok(AttachmentSecurityScanTickV1::Idle);
        };
        let target_blob = match claimed.target_blob_receipt {
            Some(receipt) => receipt,
            None => {
                let receipt = match self
                    .scanner
                    .transfer_claimed_blob(&mut self.control_channel, &claimed)
                {
                    Ok(receipt) => receipt,
                    Err(error) => {
                        return self
                            .retry_claim(&claimed, observed_at_unix_seconds, error)
                            .await;
                    }
                };
                self.persistence
                    .record_target_blob_receipt(&claimed, receipt, observed_at_unix_seconds)
                    .await
                    .map_err(persistence_error)?;
                receipt
            }
        };
        let scanner_outcome =
            match self
                .scanner
                .scan_claimed(&mut self.control_channel, &claimed, target_blob)
            {
                Ok(outcome) => outcome,
                Err(error) => {
                    return self
                        .retry_claim(&claimed, observed_at_unix_seconds, error)
                        .await;
                }
            };
        let decision = decide_attachment_security_verdict_v1(
            &claimed.job,
            scanner_outcome,
            observed_at_unix_seconds,
        )
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::InvalidJob)?;
        let fact = AttachmentSafetyVerdictFactV1 {
            attachment_anchor_id: decision.attachment_anchor_id,
            evidence_id: decision.evidence_id,
            causation_message_id: decision.causation_message_id,
            correlation_id: decision.correlation_id,
            expected_state: AttachmentSafetyExpectedStateV1::BlobAdmitted,
            verdict: match decision.verdict {
                AttachmentSecurityVerdictV1::SafeForDelivery => {
                    CommunicationsAttachmentSafetyVerdictV1::SafeForDelivery
                }
                AttachmentSecurityVerdictV1::Quarantined => {
                    CommunicationsAttachmentSafetyVerdictV1::Quarantined
                }
            },
            observed_at_unix_seconds: decision.observed_at_unix_seconds,
        };
        let context = AttachmentObservationEnvelopeContextV1 {
            runtime_instance_id: self.runtime_instance_id.clone(),
            runtime_generation: self.runtime_generation,
            module_id: ATTACHMENT_SECURITY_MODULE_ID.to_owned(),
            recorded_at_unix_seconds: observed_at_unix_seconds,
            recorded_at_nanos: observed_at_nanos,
        };
        let record = build_attachment_safety_verdict_outbox_record_v1(&fact, &context)
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::InvalidJob)?;
        self.persistence
            .complete_scan_job_with_outbox(&claimed, &record, observed_at_unix_seconds)
            .await
            .map_err(persistence_error)?;
        Ok(AttachmentSecurityScanTickV1::Completed)
    }

    pub async fn relay_verdict_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, AttachmentSecurityRuntimeErrorV1> {
        relay_attachment_security_verdict_outbox_once_v1(
            &self.persistence,
            &self.connection,
            &self.verdict_publish_permit,
            published_at_unix_seconds,
        )
        .await
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)
    }

    pub async fn process_next_archive_delegation(
        &mut self,
        observed_at_unix_seconds: i64,
        observed_at_nanos: i32,
    ) -> Result<AttachmentSecurityArchiveDelegationTickV1, AttachmentSecurityRuntimeErrorV1> {
        let lease_expires_at = observed_at_unix_seconds
            .checked_add(ARCHIVE_DELEGATION_LEASE_SECONDS)
            .ok_or(AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        let Some(claimed) = self
            .persistence
            .claim_next_archive_delegation(
                ARCHIVE_DELEGATION_WORKER_ID,
                observed_at_unix_seconds,
                lease_expires_at,
            )
            .await
            .map_err(persistence_error)?
        else {
            return Ok(AttachmentSecurityArchiveDelegationTickV1::Idle);
        };
        let result = materialize_archive_delegation_result_v1(
            &mut self.control_channel,
            &claimed,
            &self.runtime_instance_id,
            self.runtime_generation,
            observed_at_unix_seconds,
            observed_at_nanos,
        );
        let record = match result {
            Ok(record) => record,
            Err(AttachmentSecurityArchiveDelegationErrorV1::Unavailable) => {
                return self
                    .retry_archive_delegation(&claimed, observed_at_unix_seconds, observed_at_nanos)
                    .await;
            }
            Err(AttachmentSecurityArchiveDelegationErrorV1::Rejected) => {
                materialize_archive_delegation_exhausted_v1(
                    &claimed,
                    &self.runtime_instance_id,
                    self.runtime_generation,
                    observed_at_unix_seconds,
                    observed_at_nanos,
                )
                .map_err(|_| AttachmentSecurityRuntimeErrorV1::InvalidJob)?
            }
            Err(AttachmentSecurityArchiveDelegationErrorV1::InvalidEvidence) => {
                return Err(AttachmentSecurityRuntimeErrorV1::InvalidJob);
            }
        };
        self.persistence
            .complete_archive_delegation_with_outbox(&claimed, &record, observed_at_unix_seconds)
            .await
            .map_err(persistence_error)?;
        Ok(AttachmentSecurityArchiveDelegationTickV1::Completed)
    }

    pub async fn relay_archive_delegation_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, AttachmentSecurityRuntimeErrorV1> {
        relay_attachment_security_archive_delegation_outbox_once_v1(
            &self.persistence,
            &self.connection,
            &self.verdict_publish_permit,
            published_at_unix_seconds,
        )
        .await
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)
    }

    pub async fn process_next_text_delegation(
        &mut self,
        observed_at_unix_seconds: i64,
        observed_at_nanos: i32,
    ) -> Result<AttachmentSecurityTextDelegationTickV1, AttachmentSecurityRuntimeErrorV1> {
        let lease_expires_at = observed_at_unix_seconds
            .checked_add(TEXT_DELEGATION_LEASE_SECONDS)
            .ok_or(AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        let Some(claimed) = self
            .persistence
            .claim_next_text_delegation(
                TEXT_DELEGATION_WORKER_ID,
                observed_at_unix_seconds,
                lease_expires_at,
            )
            .await
            .map_err(persistence_error)?
        else {
            return Ok(AttachmentSecurityTextDelegationTickV1::Idle);
        };
        let result = materialize_text_delegation_result_v1(
            &mut self.control_channel,
            &claimed,
            &self.runtime_instance_id,
            self.runtime_generation,
            observed_at_unix_seconds,
            observed_at_nanos,
        );
        let record = match result {
            Ok(record) => record,
            Err(AttachmentSecurityTextDelegationErrorV1::Unavailable) => {
                return self
                    .retry_text_delegation(&claimed, observed_at_unix_seconds, observed_at_nanos)
                    .await;
            }
            Err(AttachmentSecurityTextDelegationErrorV1::Rejected) => {
                materialize_text_delegation_exhausted_v1(
                    &claimed,
                    &self.runtime_instance_id,
                    self.runtime_generation,
                    observed_at_unix_seconds,
                    observed_at_nanos,
                )
                .map_err(|_| AttachmentSecurityRuntimeErrorV1::InvalidJob)?
            }
            Err(AttachmentSecurityTextDelegationErrorV1::InvalidEvidence) => {
                return Err(AttachmentSecurityRuntimeErrorV1::InvalidJob);
            }
        };
        self.persistence
            .complete_text_delegation_with_outbox(&claimed, &record, observed_at_unix_seconds)
            .await
            .map_err(persistence_error)?;
        Ok(AttachmentSecurityTextDelegationTickV1::Completed)
    }

    pub async fn process_next_preview_delegation(
        &mut self,
        observed_at_unix_seconds: i64,
        observed_at_nanos: i32,
    ) -> Result<AttachmentSecurityPreviewDelegationTickV1, AttachmentSecurityRuntimeErrorV1> {
        let lease_expires_at = observed_at_unix_seconds
            .checked_add(PREVIEW_DELEGATION_LEASE_SECONDS)
            .ok_or(AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        let Some(claimed) = self
            .persistence
            .claim_next_preview_delegation(
                PREVIEW_DELEGATION_WORKER_ID,
                observed_at_unix_seconds,
                lease_expires_at,
            )
            .await
            .map_err(persistence_error)?
        else {
            return Ok(AttachmentSecurityPreviewDelegationTickV1::Idle);
        };
        let result = materialize_preview_delegation_result_v1(
            &mut self.control_channel,
            &claimed,
            &self.runtime_instance_id,
            self.runtime_generation,
            observed_at_unix_seconds,
            observed_at_nanos,
        );
        let record = match result {
            Ok(record) => record,
            Err(AttachmentSecurityPreviewDelegationErrorV1::Unavailable) => {
                return self
                    .retry_preview_delegation(&claimed, observed_at_unix_seconds, observed_at_nanos)
                    .await;
            }
            Err(AttachmentSecurityPreviewDelegationErrorV1::Rejected) => {
                materialize_preview_delegation_exhausted_v1(
                    &claimed,
                    &self.runtime_instance_id,
                    self.runtime_generation,
                    observed_at_unix_seconds,
                    observed_at_nanos,
                )
                .map_err(|_| AttachmentSecurityRuntimeErrorV1::InvalidJob)?
            }
            Err(AttachmentSecurityPreviewDelegationErrorV1::InvalidEvidence) => {
                return Err(AttachmentSecurityRuntimeErrorV1::InvalidJob);
            }
        };
        self.persistence
            .complete_preview_delegation_with_outbox(&claimed, &record, observed_at_unix_seconds)
            .await
            .map_err(persistence_error)?;
        Ok(AttachmentSecurityPreviewDelegationTickV1::Completed)
    }

    pub async fn relay_preview_delegation_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, AttachmentSecurityRuntimeErrorV1> {
        relay_attachment_security_preview_delegation_outbox_once_v1(
            &self.persistence,
            &self.connection,
            &self.verdict_publish_permit,
            published_at_unix_seconds,
        )
        .await
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)
    }

    pub async fn relay_text_delegation_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, AttachmentSecurityRuntimeErrorV1> {
        relay_attachment_security_text_delegation_outbox_once_v1(
            &self.persistence,
            &self.connection,
            &self.verdict_publish_permit,
            published_at_unix_seconds,
        )
        .await
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)
    }

    async fn retry_archive_delegation(
        &self,
        claimed: &ClaimedAttachmentSecurityArchiveDelegationV1,
        recorded_at_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> Result<AttachmentSecurityArchiveDelegationTickV1, AttachmentSecurityRuntimeErrorV1> {
        let exponent = claimed.attempt_count.saturating_sub(1).min(6);
        let next_attempt = recorded_at_unix_seconds
            .checked_add((5_i64 << exponent).min(300))
            .ok_or(AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        match self
            .persistence
            .retry_archive_delegation(claimed, recorded_at_unix_seconds, next_attempt)
            .await
            .map_err(persistence_error)?
        {
            makosh_attachment_security_persistence::RetryAttachmentSecurityArchiveDelegationOutcomeV1::Scheduled => {
                Ok(AttachmentSecurityArchiveDelegationTickV1::RetryScheduled)
            }
            makosh_attachment_security_persistence::RetryAttachmentSecurityArchiveDelegationOutcomeV1::Exhausted => {
                let record = materialize_archive_delegation_exhausted_v1(
                    claimed,
                    &self.runtime_instance_id,
                    self.runtime_generation,
                    recorded_at_unix_seconds,
                    recorded_at_nanos,
                )
                .map_err(|_| AttachmentSecurityRuntimeErrorV1::InvalidJob)?;
                self.persistence
                    .complete_archive_delegation_with_outbox(
                        claimed,
                        &record,
                        recorded_at_unix_seconds,
                    )
                    .await
                    .map_err(persistence_error)?;
                Ok(AttachmentSecurityArchiveDelegationTickV1::Rejected)
            }
        }
    }

    async fn retry_text_delegation(
        &self,
        claimed: &ClaimedAttachmentSecurityTextDelegationV1,
        recorded_at_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> Result<AttachmentSecurityTextDelegationTickV1, AttachmentSecurityRuntimeErrorV1> {
        let exponent = claimed.attempt_count.saturating_sub(1).min(6);
        let next_attempt = recorded_at_unix_seconds
            .checked_add((5_i64 << exponent).min(300))
            .ok_or(AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        match self
            .persistence
            .retry_text_delegation(claimed, recorded_at_unix_seconds, next_attempt)
            .await
            .map_err(persistence_error)?
        {
            makosh_attachment_security_persistence::RetryAttachmentSecurityTextDelegationOutcomeV1::Scheduled => {
                Ok(AttachmentSecurityTextDelegationTickV1::RetryScheduled)
            }
            makosh_attachment_security_persistence::RetryAttachmentSecurityTextDelegationOutcomeV1::Exhausted => {
                let record = materialize_text_delegation_exhausted_v1(
                    claimed,
                    &self.runtime_instance_id,
                    self.runtime_generation,
                    recorded_at_unix_seconds,
                    recorded_at_nanos,
                )
                .map_err(|_| AttachmentSecurityRuntimeErrorV1::InvalidJob)?;
                self.persistence
                    .complete_text_delegation_with_outbox(
                        claimed,
                        &record,
                        recorded_at_unix_seconds,
                    )
                    .await
                    .map_err(persistence_error)?;
                Ok(AttachmentSecurityTextDelegationTickV1::Rejected)
            }
        }
    }

    async fn retry_preview_delegation(
        &self,
        claimed: &ClaimedAttachmentSecurityPreviewDelegationV1,
        recorded_at_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> Result<AttachmentSecurityPreviewDelegationTickV1, AttachmentSecurityRuntimeErrorV1> {
        let exponent = claimed.attempt_count.saturating_sub(1).min(6);
        let next_attempt = recorded_at_unix_seconds
            .checked_add((5_i64 << exponent).min(300))
            .ok_or(AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        match self
            .persistence
            .retry_preview_delegation(claimed, recorded_at_unix_seconds, next_attempt)
            .await
            .map_err(persistence_error)?
        {
            makosh_attachment_security_persistence::RetryAttachmentSecurityPreviewDelegationOutcomeV1::Scheduled => {
                Ok(AttachmentSecurityPreviewDelegationTickV1::RetryScheduled)
            }
            makosh_attachment_security_persistence::RetryAttachmentSecurityPreviewDelegationOutcomeV1::Exhausted => {
                let record = materialize_preview_delegation_exhausted_v1(
                    claimed,
                    &self.runtime_instance_id,
                    self.runtime_generation,
                    recorded_at_unix_seconds,
                    recorded_at_nanos,
                )
                .map_err(|_| AttachmentSecurityRuntimeErrorV1::InvalidJob)?;
                self.persistence
                    .complete_preview_delegation_with_outbox(
                        claimed,
                        &record,
                        recorded_at_unix_seconds,
                    )
                    .await
                    .map_err(persistence_error)?;
                Ok(AttachmentSecurityPreviewDelegationTickV1::Rejected)
            }
        }
    }

    async fn retry_claim(
        &self,
        claimed: &ClaimedAttachmentSecurityScanJobV1,
        recorded_at_unix_seconds: i64,
        error: crate::scan::AttachmentSecurityScanAdapterErrorV1,
    ) -> Result<AttachmentSecurityScanTickV1, AttachmentSecurityRuntimeErrorV1> {
        let exponent = claimed.attempt_count.saturating_sub(1).min(6);
        let delay_seconds = (5_i64 << exponent).min(300);
        let next_attempt = recorded_at_unix_seconds
            .checked_add(delay_seconds)
            .ok_or(AttachmentSecurityRuntimeErrorV1::Unavailable)?;
        let outcome = self
            .persistence
            .retry_scan_job(claimed, recorded_at_unix_seconds, next_attempt)
            .await
            .map_err(persistence_error)?;
        Ok(
            match outcome {
                makosh_attachment_security_persistence::RetryAttachmentSecurityScanJobOutcomeV1::Scheduled => {
                    AttachmentSecurityScanTickV1::RetryScheduled(error)
                }
                makosh_attachment_security_persistence::RetryAttachmentSecurityScanJobOutcomeV1::Exhausted => {
                    AttachmentSecurityScanTickV1::Exhausted(error)
                }
            },
        )
    }
}

struct AttachmentSecuritySubscribePermitsV1 {
    candidate: RuntimeSubscribePermitV1,
    canonical_state: RuntimeSubscribePermitV1,
    archive_delegation: RuntimeSubscribePermitV1,
    preview_delegation: RuntimeSubscribePermitV1,
    text_delegation: RuntimeSubscribePermitV1,
}

impl AttachmentSecuritySubscribePermitsV1 {
    fn bind(
        permits: Vec<RuntimeSubscribePermitV1>,
    ) -> Result<Self, AttachmentSecurityRuntimeErrorV1> {
        let candidate = attachment_security_scan_candidate_observed_contract_reference_v1();
        let canonical_state = communication_attachment_safety_state_changed_contract_reference_v1();
        let archive_delegation =
            archive_inspection_custody_delegation_requested_contract_reference_v1();
        let preview_delegation =
            attachment_preview_custody_delegation_requested_contract_reference_v1();
        let text_delegation = attachment_text_custody_delegation_requested_contract_reference_v1();
        let mut candidate_permit = None;
        let mut canonical_state_permit = None;
        let mut archive_delegation_permit = None;
        let mut preview_delegation_permit = None;
        let mut text_delegation_permit = None;
        for permit in permits {
            let Some(contract) = permit.contract() else {
                return Err(AttachmentSecurityRuntimeErrorV1::Admission);
            };
            if exact_contract(contract, &candidate) {
                replace_once(&mut candidate_permit, permit)?;
            } else if exact_contract(contract, &canonical_state) {
                replace_once(&mut canonical_state_permit, permit)?;
            } else if exact_contract(contract, &archive_delegation) {
                replace_once(&mut archive_delegation_permit, permit)?;
            } else if exact_contract(contract, &preview_delegation) {
                replace_once(&mut preview_delegation_permit, permit)?;
            } else if exact_contract(contract, &text_delegation) {
                replace_once(&mut text_delegation_permit, permit)?;
            } else {
                return Err(AttachmentSecurityRuntimeErrorV1::Admission);
            }
        }
        Ok(Self {
            candidate: candidate_permit.ok_or(AttachmentSecurityRuntimeErrorV1::Admission)?,
            canonical_state: canonical_state_permit
                .ok_or(AttachmentSecurityRuntimeErrorV1::Admission)?,
            archive_delegation: archive_delegation_permit
                .ok_or(AttachmentSecurityRuntimeErrorV1::Admission)?,
            preview_delegation: preview_delegation_permit
                .ok_or(AttachmentSecurityRuntimeErrorV1::Admission)?,
            text_delegation: text_delegation_permit
                .ok_or(AttachmentSecurityRuntimeErrorV1::Admission)?,
        })
    }
}

#[derive(Clone, Copy)]
enum AttachmentSecurityConsumerV1 {
    Candidate,
    CanonicalState,
    ArchiveDelegation,
    PreviewDelegation,
    TextDelegation,
}

impl AttachmentSecurityConsumerV1 {
    const fn successor(self) -> Self {
        match self {
            Self::Candidate => Self::CanonicalState,
            Self::CanonicalState => Self::ArchiveDelegation,
            Self::ArchiveDelegation => Self::PreviewDelegation,
            Self::PreviewDelegation => Self::TextDelegation,
            Self::TextDelegation => Self::Candidate,
        }
    }
}

fn validate_open_input(
    descriptor_bytes: &[u8],
    settings_schema_bytes: &[u8],
    admission: &AttachmentSecurityRuntimeAdmissionV1,
    event_hub_endpoint: &str,
    credential_revision: u64,
) -> Result<(), AttachmentSecurityRuntimeErrorV1> {
    if descriptor_bytes.is_empty()
        || settings_schema_bytes.is_empty()
        || admission.logical_owner_id != ATTACHMENT_SECURITY_OWNER_ID
        || admission.registration_id.trim().is_empty()
        || admission.runtime_instance_id.trim().is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
        || credential_revision == 0
        || event_hub_endpoint.trim().is_empty()
    {
        return Err(AttachmentSecurityRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn replace_once(
    selected: &mut Option<RuntimeSubscribePermitV1>,
    permit: RuntimeSubscribePermitV1,
) -> Result<(), AttachmentSecurityRuntimeErrorV1> {
    if selected.replace(permit).is_some() {
        return Err(AttachmentSecurityRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn exact_contract(left: &ContractReferenceV1, right: &ContractReferenceV1) -> bool {
    left.owner == right.owner
        && left.name == right.name
        && left.major == right.major
        && left.revision == right.revision
        && left.schema_sha256 == right.schema_sha256
}

fn persistence_error(
    error: AttachmentSecurityPersistenceErrorV1,
) -> AttachmentSecurityRuntimeErrorV1 {
    match error {
        AttachmentSecurityPersistenceErrorV1::InvalidInput
        | AttachmentSecurityPersistenceErrorV1::InvalidRow
        | AttachmentSecurityPersistenceErrorV1::OutboxHashConflict
        | AttachmentSecurityPersistenceErrorV1::EvidenceConflict => {
            AttachmentSecurityRuntimeErrorV1::InvalidJob
        }
        AttachmentSecurityPersistenceErrorV1::StorageUnavailable
        | AttachmentSecurityPersistenceErrorV1::ClaimLost => {
            AttachmentSecurityRuntimeErrorV1::Unavailable
        }
    }
}

async fn resolve_storage_runtime_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, AttachmentSecurityRuntimeErrorV1> {
    const MAX_ATTEMPTS: usize = 20;
    for attempt in 0..MAX_ATTEMPTS {
        if let Ok(lease_id) = leases.issue_runtime_credential(binding).await
            && let Ok(password) = leases.resolve_runtime_credential(binding, lease_id).await
        {
            return Ok(password);
        }
        if attempt + 1 < MAX_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    Err(AttachmentSecurityRuntimeErrorV1::Unavailable)
}

fn authenticate_managed_runtime_v2(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &AttachmentSecurityRuntimeAdmissionV1,
) -> Result<(), AttachmentSecurityRuntimeErrorV1> {
    control_channel
        .inner_mut()
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| {
            control_channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
    let response = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(AttachmentSecurityRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_managed_runtime_ready(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &AttachmentSecurityRuntimeAdmissionV1,
) -> Result<(), AttachmentSecurityRuntimeErrorV1> {
    control_channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &AttachmentSecurityRuntimeAdmissionV1,
) -> Result<StorageBindingV1, AttachmentSecurityRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != ATTACHMENT_SECURITY_OWNER_ID
        || configuration.owner != ATTACHMENT_SECURITY_OWNER_ID
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(AttachmentSecurityRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Admission)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityScanTickV1 {
    Idle,
    Completed,
    RetryScheduled(crate::scan::AttachmentSecurityScanAdapterErrorV1),
    Exhausted(crate::scan::AttachmentSecurityScanAdapterErrorV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityArchiveDelegationTickV1 {
    Idle,
    Completed,
    RetryScheduled,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityTextDelegationTickV1 {
    Idle,
    Completed,
    RetryScheduled,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityPreviewDelegationTickV1 {
    Idle,
    Completed,
    RetryScheduled,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityRuntimeErrorV1 {
    Admission,
    InvalidDelivery,
    InvalidJob,
    Unavailable,
}

pub fn current_runtime_time_v1() -> Result<(i64, i32), AttachmentSecurityRuntimeErrorV1> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?;
    Ok((
        i64::try_from(now.as_secs()).map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?,
        i32::try_from(now.subsec_nanos())
            .map_err(|_| AttachmentSecurityRuntimeErrorV1::Unavailable)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumers_alternate_without_one_route_owning_the_loop() {
        assert!(matches!(
            AttachmentSecurityConsumerV1::Candidate.successor(),
            AttachmentSecurityConsumerV1::CanonicalState
        ));
        assert!(matches!(
            AttachmentSecurityConsumerV1::CanonicalState.successor(),
            AttachmentSecurityConsumerV1::ArchiveDelegation
        ));
        assert!(matches!(
            AttachmentSecurityConsumerV1::ArchiveDelegation.successor(),
            AttachmentSecurityConsumerV1::PreviewDelegation
        ));
        assert!(matches!(
            AttachmentSecurityConsumerV1::PreviewDelegation.successor(),
            AttachmentSecurityConsumerV1::TextDelegation
        ));
        assert!(matches!(
            AttachmentSecurityConsumerV1::TextDelegation.successor(),
            AttachmentSecurityConsumerV1::Candidate
        ));
    }
}
