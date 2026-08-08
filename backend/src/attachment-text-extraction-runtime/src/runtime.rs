use std::{
    os::unix::net::UnixStream,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use makosh_attachment_security_contract::admission::attachment_security_scan_candidate_observed_contract_reference_v1;
use makosh_attachment_text_extraction_api::{
    ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1, ATTACHMENT_TEXT_EXTRACTION_OWNER_V1,
    wire::AttachmentTextExtractionErrorCodeV1 as WireError,
};
use makosh_attachment_text_extraction_core::AttachmentTextExtractionErrorV1;
use makosh_attachment_text_extraction_ingress::{
    AttachmentTextCustodyEnvelopeContextV1,
    attachment_text_custody_delegated_contract_reference_v1,
    attachment_text_custody_delegation_rejected_contract_reference_v1,
    attachment_text_custody_delegation_request_id_v1,
    build_request_attachment_text_custody_delegation_outbox_record_v1,
    wire::RequestAttachmentTextCustodyDelegationV1,
};
use makosh_attachment_text_extraction_persistence::{
    AttachmentTextExtractionPersistenceErrorV1, AttachmentTextExtractionPersistenceV1,
    PersistAttachmentTextCustodyDelegationV1, PersistedAttachmentTextArtifactV1,
};
use makosh_attachment_translation_ingress::attachment_translation_source_requested_contract_reference_v1;
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
use zeroize::Zeroizing;

use crate::{
    AttachmentTextExtractionParserRuntimeV1, AttachmentTextRuntimeParseErrorV1,
    blob::{read_artifact_v1, read_source_v1, transfer_source_v1, write_derived_v1},
    client_port::{
        ClientDispatchV1, dispatch_client_request_v1, read_text_error_response_v1,
        read_text_response_v1,
    },
    client_realtime::{ClientRealtimeErrorV1, ClientRealtimePublisherV1},
    event_decode::{
        DecodedCustodyResultV1, decode_candidate_v1, decode_custody_result_v1, decode_safety_v1,
    },
    outbox::{relay_custody_outbox_once_v1, relay_translation_source_outbox_once_v1},
    translation_source::process_translation_source_delivery_v1,
};

const WORKER_ID: &str = "attachment-text-extraction-runtime";
const JOB_LEASE_MILLIS: u64 = 180_000;

pub struct AttachmentTextExtractionRuntimeAdmissionV1 {
    pub module_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

pub struct AttachmentTextExtractionManagedRuntimeV1 {
    control_channel: ManagedControlChannelV2<UnixStream>,
    connection: RuntimeJetStreamConnection,
    permits: SubscribePermitsV1,
    next_consumer: ConsumerV1,
    publish_permit: RuntimePublishPermitV1,
    persistence: AttachmentTextExtractionPersistenceV1,
    client_realtime: ClientRealtimePublisherV1,
    parser: AttachmentTextExtractionParserRuntimeV1,
    logical_human_owner_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

impl AttachmentTextExtractionManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &AttachmentTextExtractionRuntimeAdmissionV1,
        event_hub_endpoint: &str,
        credential_revision: u64,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        parser: AttachmentTextExtractionParserRuntimeV1,
    ) -> Result<Self, AttachmentTextExtractionRuntimeErrorV1> {
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
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        let permits = SubscribePermitsV1::bind(
            access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?,
        )?;
        let publish_permit = access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?;
        let connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            public_key,
        )
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?;
        let persistence = AttachmentTextExtractionPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        let mut control_channel = leases.into_route_port().into_channel();
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            control_channel,
            connection,
            permits,
            next_consumer: ConsumerV1::Candidate,
            publish_permit,
            persistence,
            client_realtime: ClientRealtimePublisherV1::default(),
            parser,
            logical_human_owner_id: admission.logical_human_owner_id.clone(),
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
    }

    pub async fn consume_next(
        &mut self,
        consumed_at_unix_millis: i64,
    ) -> Result<bool, AttachmentTextExtractionRuntimeErrorV1> {
        let consumer = self.next_consumer;
        self.next_consumer = consumer.successor();
        let delivery = match try_receive_runtime_pull_delivery(
            &self.connection,
            self.permits.for_consumer(consumer),
        )
        .await
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?
        {
            None => return Ok(false),
            Some(delivery) => delivery,
        };
        match consumer {
            ConsumerV1::Candidate => {
                let decoded = decode_candidate_v1(delivery.exact_bytes())
                    .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::InvalidDelivery)?;
                self.persistence
                    .persist_scan_candidate(
                        &self.logical_human_owner_id,
                        &decoded.fact,
                        decoded.envelope_sha256,
                        decoded.payload_sha256,
                        consumed_at_unix_millis,
                    )
                    .await
                    .map_err(persistence_error)?;
            }
            ConsumerV1::Safety => {
                if let Some(decoded) = decode_safety_v1(delivery.exact_bytes())
                    .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::InvalidDelivery)?
                {
                    self.persistence
                        .persist_canonical_safety_fact(
                            &self.logical_human_owner_id,
                            &decoded.fact,
                            decoded.envelope_sha256,
                            decoded.payload_sha256,
                            consumed_at_unix_millis,
                        )
                        .await
                        .map_err(persistence_error)?;
                }
            }
            ConsumerV1::Delegated | ConsumerV1::Rejected => {
                match decode_custody_result_v1(delivery.exact_bytes())
                    .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::InvalidDelivery)?
                {
                    DecodedCustodyResultV1::Delegated {
                        message_id,
                        envelope_sha256,
                        command_message_id,
                        payload,
                    } => self
                        .persistence
                        .persist_custody_delegated_result(
                            message_id,
                            envelope_sha256,
                            command_message_id,
                            &payload,
                            consumed_at_unix_millis,
                        )
                        .await
                        .map_err(persistence_error)?,
                    DecodedCustodyResultV1::Rejected {
                        message_id,
                        envelope_sha256,
                        command_message_id,
                        payload,
                    } => self
                        .persistence
                        .persist_custody_delegation_rejected_result(
                            message_id,
                            envelope_sha256,
                            command_message_id,
                            &payload,
                            consumed_at_unix_millis,
                        )
                        .await
                        .map_err(persistence_error)?,
                };
            }
            ConsumerV1::TranslationSource => {
                process_translation_source_delivery_v1(
                    &self.persistence,
                    &mut self.control_channel,
                    delivery.exact_bytes(),
                    &self.logical_human_owner_id,
                    &self.runtime_instance_id,
                    self.runtime_generation,
                    consumed_at_unix_millis,
                )
                .await
                .map_err(|error| match error {
                    crate::translation_source::TranslationSourceDeliveryErrorV1::Unavailable => {
                        AttachmentTextExtractionRuntimeErrorV1::Unavailable
                    }
                    crate::translation_source::TranslationSourceDeliveryErrorV1::InvalidEnvelope
                    | crate::translation_source::TranslationSourceDeliveryErrorV1::InvalidPayload => {
                        AttachmentTextExtractionRuntimeErrorV1::InvalidDelivery
                    }
                    crate::translation_source::TranslationSourceDeliveryErrorV1::Persistence => {
                        AttachmentTextExtractionRuntimeErrorV1::InvalidJob
                    }
                })?;
            }
        }
        delivery
            .acknowledge()
            .await
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn materialize_pending_custody_requests(
        &self,
        now_unix_millis: i64,
        now_nanos: i32,
    ) -> Result<usize, AttachmentTextExtractionRuntimeErrorV1> {
        let pending = self
            .persistence
            .pending_custody_delegation_intents(&self.logical_human_owner_id, 64)
            .await
            .map_err(persistence_error)?;
        for item in &pending {
            let request_id = attachment_text_custody_delegation_request_id_v1(
                item.intent.run_id,
                item.intent.candidate_message_id,
                item.intent.safety_message_id,
            );
            let seconds = now_unix_millis / 1_000;
            let record = build_request_attachment_text_custody_delegation_outbox_record_v1(
                RequestAttachmentTextCustodyDelegationV1 {
                    request_id: request_id.to_vec(),
                    extraction_run_id: item.intent.run_id.to_vec(),
                    attachment_anchor_id: item.intent.attachment_anchor_id.to_vec(),
                    candidate_message_id: item.intent.candidate_message_id.to_vec(),
                    candidate_envelope_sha256: item.candidate_envelope_sha256.to_vec(),
                    safety_message_id: item.intent.safety_message_id.to_vec(),
                    safety_evidence_id: item.intent.safety_evidence_id.to_vec(),
                    logical_owner_id: self.logical_human_owner_id.clone(),
                },
                seconds
                    .checked_add(300)
                    .ok_or(AttachmentTextExtractionRuntimeErrorV1::Unavailable)?,
                &AttachmentTextCustodyEnvelopeContextV1 {
                    module_id: ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1.to_owned(),
                    runtime_instance_id: self.runtime_instance_id.clone(),
                    runtime_generation: self.runtime_generation,
                    recorded_at_unix_seconds: seconds,
                    recorded_at_nanos: now_nanos,
                },
            )
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::InvalidJob)?;
            self.persistence
                .store_custody_delegation_outbox(
                    &self.logical_human_owner_id,
                    &PersistAttachmentTextCustodyDelegationV1 {
                        request_id,
                        run_id: item.intent.run_id,
                        candidate_message_id: item.intent.candidate_message_id,
                        safety_message_id: item.intent.safety_message_id,
                        envelope_sha256: *record.envelope_sha256(),
                        exact_envelope_bytes: record.exact_bytes().to_vec(),
                        created_at_unix_millis: now_unix_millis,
                    },
                )
                .await
                .map_err(persistence_error)?;
        }
        Ok(pending.len())
    }

    pub async fn relay_custody_outbox(
        &self,
        now_unix_millis: i64,
    ) -> Result<usize, AttachmentTextExtractionRuntimeErrorV1> {
        relay_custody_outbox_once_v1(
            &self.persistence,
            &self.logical_human_owner_id,
            &self.connection,
            &self.publish_permit,
            now_unix_millis,
        )
        .await
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)
    }

    pub async fn relay_translation_source_outbox(
        &self,
        now_unix_millis: i64,
    ) -> Result<usize, AttachmentTextExtractionRuntimeErrorV1> {
        relay_translation_source_outbox_once_v1(
            &self.persistence,
            &self.logical_human_owner_id,
            &self.connection,
            &self.publish_permit,
            now_unix_millis,
        )
        .await
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)
    }

    pub async fn process_next_job(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<JobTickV1, AttachmentTextExtractionRuntimeErrorV1> {
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
            return Ok(JobTickV1::Idle);
        };
        let receipt = match claimed.target_blob_receipt {
            Some(receipt) => receipt,
            None => {
                let receipt = transfer_source_v1(&mut self.control_channel, &claimed)
                    .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
                self.persistence
                    .record_target_blob_receipt(&claimed, receipt, now_unix_millis)
                    .await
                    .map_err(persistence_error)?;
                receipt
            }
        };
        let source = read_source_v1(&mut self.control_channel, &claimed, receipt)
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        let parsed = match self.parser.extract(&source) {
            Ok(value) => value,
            Err(error) => {
                let error = parser_error(error);
                self.persistence
                    .reject_job(&claimed, error, now_unix_millis)
                    .await
                    .map_err(persistence_error)?;
                return Ok(JobTickV1::Rejected(error));
            }
        };
        let (derived_reference_id, derived_receipt_sha256, extracted_size_bytes) =
            write_derived_v1(
                &mut self.control_channel,
                &claimed,
                Zeroizing::new(parsed.text_utf8),
                parsed.parser_identity_sha256,
            )
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        self.persistence
            .complete_job(
                &claimed,
                PersistedAttachmentTextArtifactV1 {
                    run_id: claimed.request.run_id,
                    derived_reference_id,
                    derived_receipt_sha256,
                    source_receipt_sha256: claimed.source_receipt_sha256,
                    parser_identity_sha256: parsed.parser_identity_sha256,
                    format: parsed.format,
                    extracted_size_bytes,
                    extraction_truncated: parsed.extraction_truncated,
                },
                now_unix_millis,
            )
            .await
            .map_err(persistence_error)?;
        Ok(JobTickV1::Completed)
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, AttachmentTextExtractionRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            return self.write_client_error(correlation_id);
        };
        let Some(request) = delivery
            .request
            .filter(|request| validate_module_client_request_v1(request).is_ok())
        else {
            return self.write_client_error(correlation_id);
        };
        let response = match dispatch_client_request_v1(
            &self.persistence,
            &self.logical_human_owner_id,
            request,
            now_unix_millis,
        )
        .await
        {
            ClientDispatchV1::Response(response) => response,
            ClientDispatchV1::ReadText { request_id, run_id } => {
                match self
                    .persistence
                    .find_artifact(&self.logical_human_owner_id, run_id)
                    .await
                    .map_err(persistence_error)?
                {
                    None => read_text_error_response_v1(request_id, run_id, WireError::NotFound),
                    Some(artifact)
                        if !self.parser.matches_artifact_identity_v1(
                            artifact.format,
                            artifact.parser_identity_sha256,
                        ) =>
                    {
                        read_text_error_response_v1(request_id, run_id, WireError::Unavailable)
                    }
                    Some(artifact) => {
                        match read_artifact_v1(&mut self.control_channel, &artifact) {
                            Ok(text) => read_text_response_v1(
                                request_id,
                                run_id,
                                &text,
                                artifact.extracted_size_bytes,
                            ),
                            Err(_) => read_text_error_response_v1(
                                request_id,
                                run_id,
                                WireError::Unavailable,
                            ),
                        }
                    }
                }
            }
        };
        if validate_module_client_response_v1(&response).is_err() {
            return Err(AttachmentTextExtractionRuntimeErrorV1::Unavailable);
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
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn pump_client_realtime_once(
        &mut self,
    ) -> Result<bool, AttachmentTextExtractionRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
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
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        result
    }

    fn write_client_error(
        &mut self,
        correlation_id: [u8; 16],
    ) -> Result<bool, AttachmentTextExtractionRuntimeErrorV1> {
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_request".to_owned(),
                },
            )
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }
}

struct SubscribePermitsV1 {
    candidate: RuntimeSubscribePermitV1,
    safety: RuntimeSubscribePermitV1,
    delegated: RuntimeSubscribePermitV1,
    rejected: RuntimeSubscribePermitV1,
    translation_source: RuntimeSubscribePermitV1,
}

impl SubscribePermitsV1 {
    fn bind(
        permits: Vec<RuntimeSubscribePermitV1>,
    ) -> Result<Self, AttachmentTextExtractionRuntimeErrorV1> {
        let expected = [
            attachment_security_scan_candidate_observed_contract_reference_v1(),
            communication_attachment_safety_state_changed_contract_reference_v1(),
            attachment_text_custody_delegated_contract_reference_v1(),
            attachment_text_custody_delegation_rejected_contract_reference_v1(),
            attachment_translation_source_requested_contract_reference_v1(),
        ];
        let mut selected: [Option<RuntimeSubscribePermitV1>; 5] = [None, None, None, None, None];
        for permit in permits {
            let Some(contract) = permit.contract() else {
                return Err(AttachmentTextExtractionRuntimeErrorV1::Admission);
            };
            let Some(index) = expected
                .iter()
                .position(|expected| exact_contract(contract, expected))
            else {
                return Err(AttachmentTextExtractionRuntimeErrorV1::Admission);
            };
            if selected[index].replace(permit).is_some() {
                return Err(AttachmentTextExtractionRuntimeErrorV1::Admission);
            }
        }
        let [candidate, safety, delegated, rejected, translation_source] = selected;
        Ok(Self {
            candidate: candidate.ok_or(AttachmentTextExtractionRuntimeErrorV1::Admission)?,
            safety: safety.ok_or(AttachmentTextExtractionRuntimeErrorV1::Admission)?,
            delegated: delegated.ok_or(AttachmentTextExtractionRuntimeErrorV1::Admission)?,
            rejected: rejected.ok_or(AttachmentTextExtractionRuntimeErrorV1::Admission)?,
            translation_source: translation_source
                .ok_or(AttachmentTextExtractionRuntimeErrorV1::Admission)?,
        })
    }

    fn for_consumer(&self, consumer: ConsumerV1) -> &RuntimeSubscribePermitV1 {
        match consumer {
            ConsumerV1::Candidate => &self.candidate,
            ConsumerV1::Safety => &self.safety,
            ConsumerV1::Delegated => &self.delegated,
            ConsumerV1::Rejected => &self.rejected,
            ConsumerV1::TranslationSource => &self.translation_source,
        }
    }
}

#[derive(Clone, Copy)]
enum ConsumerV1 {
    Candidate,
    Safety,
    Delegated,
    Rejected,
    TranslationSource,
}

impl ConsumerV1 {
    const fn successor(self) -> Self {
        match self {
            Self::Candidate => Self::Safety,
            Self::Safety => Self::Delegated,
            Self::Delegated => Self::Rejected,
            Self::Rejected => Self::TranslationSource,
            Self::TranslationSource => Self::Candidate,
        }
    }
}

fn validate_open(
    descriptor: &[u8],
    settings: &[u8],
    admission: &AttachmentTextExtractionRuntimeAdmissionV1,
    event_hub_endpoint: &str,
    credential_revision: u64,
) -> Result<(), AttachmentTextExtractionRuntimeErrorV1> {
    if descriptor.is_empty()
        || settings.is_empty()
        || admission.module_owner_id != ATTACHMENT_TEXT_EXTRACTION_OWNER_V1
        || !valid_owner_id(&admission.logical_human_owner_id)
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
        || credential_revision == 0
        || event_hub_endpoint.is_empty()
    {
        return Err(AttachmentTextExtractionRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &AttachmentTextExtractionRuntimeAdmissionV1,
) -> Result<(), AttachmentTextExtractionRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(AttachmentTextExtractionRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &AttachmentTextExtractionRuntimeAdmissionV1,
) -> Result<(), AttachmentTextExtractionRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<Zeroizing<Vec<u8>>, AttachmentTextExtractionRuntimeErrorV1> {
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
    Err(AttachmentTextExtractionRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &AttachmentTextExtractionRuntimeAdmissionV1,
) -> Result<StorageBindingV1, AttachmentTextExtractionRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != ATTACHMENT_TEXT_EXTRACTION_OWNER_V1
        || configuration.owner != ATTACHMENT_TEXT_EXTRACTION_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(AttachmentTextExtractionRuntimeErrorV1::Admission);
    }
    StorageBindingV1::new(
        StorageBindingIdentityV1::new(
            configuration.storage_instance_id.clone(),
            configuration.database_id.clone(),
            configuration.owner.clone(),
            admission.registration_id.clone(),
            configuration.runtime_instance_id.clone(),
        )
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?,
        StorageBindingFencesV1::new(
            configuration.storage_generation,
            admission.runtime_generation,
            admission.grant_epoch,
            configuration.role_epoch,
            configuration.credential_revision,
            configuration.storage_bundle_revision,
        )
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?,
        StorageBindingAccessV1::new(
            configuration.runtime_principal.clone(),
            configuration.pool_alias.clone(),
            StorageEffectiveBudgetsV1::new(
                u16::try_from(configuration.max_connections)
                    .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?,
                configuration.statement_timeout_millis,
            )
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?,
            configuration
                .storage_bundle_digest
                .as_slice()
                .try_into()
                .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?,
        )
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Admission)
}

fn persistence_error(
    error: AttachmentTextExtractionPersistenceErrorV1,
) -> AttachmentTextExtractionRuntimeErrorV1 {
    match error {
        AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable => {
            AttachmentTextExtractionRuntimeErrorV1::Unavailable
        }
        AttachmentTextExtractionPersistenceErrorV1::InvalidInput
        | AttachmentTextExtractionPersistenceErrorV1::InvalidRow
        | AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict => {
            AttachmentTextExtractionRuntimeErrorV1::InvalidJob
        }
    }
}

fn client_realtime_error(error: ClientRealtimeErrorV1) -> AttachmentTextExtractionRuntimeErrorV1 {
    match error {
        ClientRealtimeErrorV1::Unavailable => AttachmentTextExtractionRuntimeErrorV1::Unavailable,
        ClientRealtimeErrorV1::Persistence(error) => persistence_error(error),
        ClientRealtimeErrorV1::InvalidTransition => {
            AttachmentTextExtractionRuntimeErrorV1::InvalidJob
        }
    }
}

const fn parser_error(error: AttachmentTextRuntimeParseErrorV1) -> AttachmentTextExtractionErrorV1 {
    match error {
        AttachmentTextRuntimeParseErrorV1::Unsupported => {
            AttachmentTextExtractionErrorV1::Unsupported
        }
        AttachmentTextRuntimeParseErrorV1::SourceTooLarge => {
            AttachmentTextExtractionErrorV1::SourceTooLarge
        }
        AttachmentTextRuntimeParseErrorV1::InvalidContent => {
            AttachmentTextExtractionErrorV1::InvalidContent
        }
        AttachmentTextRuntimeParseErrorV1::ParserUnavailable => {
            AttachmentTextExtractionErrorV1::ParserUnavailable
        }
        AttachmentTextRuntimeParseErrorV1::ParserFailed => {
            AttachmentTextExtractionErrorV1::ParserFailed
        }
    }
}

fn exact_contract(left: &ContractReferenceV1, right: &ContractReferenceV1) -> bool {
    left.owner == right.owner
        && left.name == right.name
        && left.major == right.major
        && left.revision == right.revision
        && left.schema_sha256 == right.schema_sha256
}

fn valid_owner_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobTickV1 {
    Idle,
    Completed,
    Rejected(AttachmentTextExtractionErrorV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionRuntimeErrorV1 {
    Admission,
    InvalidDelivery,
    InvalidJob,
    Unavailable,
}

pub fn current_runtime_time_v1() -> Result<(i64, i32), AttachmentTextExtractionRuntimeErrorV1> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?;
    Ok((
        i64::try_from(now.as_millis())
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?,
        i32::try_from(now.subsec_nanos())
            .map_err(|_| AttachmentTextExtractionRuntimeErrorV1::Unavailable)?,
    ))
}
