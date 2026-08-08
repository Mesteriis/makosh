use std::{
    os::unix::net::UnixStream,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_MODULE_ID_V1, ATTACHMENT_PREVIEW_OWNER_V1,
    wire::AttachmentPreviewErrorCodeV1,
};
use makosh_attachment_preview_ingress::{
    AttachmentPreviewCustodyEnvelopeContextV1,
    attachment_preview_custody_delegated_contract_reference_v1,
    attachment_preview_custody_delegation_rejected_contract_reference_v1,
    attachment_preview_custody_delegation_request_id_v1,
    build_request_attachment_preview_custody_delegation_outbox_record_v1,
    wire::RequestAttachmentPreviewCustodyDelegationV1,
};
use makosh_attachment_preview_persistence::{
    AttachmentPreviewPersistenceErrorV1, AttachmentPreviewPersistenceV1,
    PersistAttachmentPreviewCustodyDelegationV1, PreviewTargetBlobReceiptV1,
    RenderedAttachmentPreviewArtifactV1,
};
use makosh_attachment_preview_renderer_contract::AttachmentPreviewRendererErrorV1;
use makosh_attachment_security_contract::admission::attachment_security_scan_candidate_observed_contract_reference_v1;
use makosh_communications_attachment_contract::admission::communication_attachment_safety_state_changed_contract_reference_v1;
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
    try_receive_runtime_pull_delivery,
};
use makosh_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;
use makosh_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
    },
    v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryResponseV1,
        ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
        ManagedStorageRuntimeConfigurationV1, ModuleClientResponseV1,
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
use zeroize::Zeroizing;

use crate::{
    AttachmentPreviewRendererRuntimeV1, attachment_preview_renderer_identity_v1,
    blob::{read_source_v1, transfer_source_v1, write_derived_v1},
    client_port::dispatch_attachment_preview_client_request_v1,
    client_realtime::{ClientRealtimeErrorV1, ClientRealtimePublisherV1},
    event_decode::{
        DecodedCustodyResultV1, decode_candidate_v1, decode_custody_result_v1, decode_safety_v1,
    },
    outbox::relay_custody_outbox_once_v1,
};

const WORKER_ID: &str = "attachment-preview-runtime";
const JOB_LEASE_MILLIS: u64 = 180_000;

pub struct AttachmentPreviewRuntimeAdmissionV1 {
    pub module_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

pub struct AttachmentPreviewManagedRuntimeV1 {
    control_channel: ManagedControlChannelV2<UnixStream>,
    connection: RuntimeJetStreamConnection,
    permits: SubscribePermitsV1,
    next_consumer: ConsumerV1,
    publish_permit: RuntimePublishPermitV1,
    persistence: AttachmentPreviewPersistenceV1,
    client_realtime: ClientRealtimePublisherV1,
    renderer: AttachmentPreviewRendererRuntimeV1,
    logical_human_owner_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

struct AttachmentPreviewNestedRequestDispatcherV1<'a> {
    persistence: &'a AttachmentPreviewPersistenceV1,
    runtime_generation: u64,
    grant_epoch: u64,
}

impl ManagedControlRequestDispatcherV2<UnixStream>
    for AttachmentPreviewNestedRequestDispatcherV1<'_>
{
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: makosh_runtime_protocol::v1::ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    let response = current_runtime_time_v1()
                        .ok()
                        .map(|(now_unix_millis, _)| {
                            tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(
                                    dispatch_attachment_preview_client_request_v1(
                                        self.persistence,
                                        self.runtime_generation,
                                        self.grant_epoch,
                                        &request,
                                        now_unix_millis,
                                    ),
                                )
                            })
                        })
                        .unwrap_or_else(|| client_unavailable_response_v1(request.request_id));
                    client_delivery_response_v1(response)
                }
                Some(request) => {
                    client_delivery_response_v1(client_rejected_response_v1(request.request_id))
                }
                None => ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                },
            },
            _ => ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        };
        channel.write_response(correlation_id, response)
    }
}

impl AttachmentPreviewManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &AttachmentPreviewRuntimeAdmissionV1,
        event_hub_endpoint: &str,
        credential_revision: u64,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        renderer: AttachmentPreviewRendererRuntimeV1,
    ) -> Result<Self, AttachmentPreviewRuntimeErrorV1> {
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
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        let permits = SubscribePermitsV1::bind(
            access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?,
        )?;
        let publish_permit = access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?;
        let connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            public_key,
        )
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?;
        let persistence = AttachmentPreviewPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        let mut control_channel = leases.into_route_port().into_channel();
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            control_channel,
            connection,
            permits,
            next_consumer: ConsumerV1::Candidate,
            publish_permit,
            persistence,
            client_realtime: ClientRealtimePublisherV1::default(),
            renderer,
            logical_human_owner_id: admission.logical_human_owner_id.clone(),
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
    }

    pub async fn consume_next(
        &mut self,
        consumed_at_unix_millis: i64,
    ) -> Result<bool, AttachmentPreviewRuntimeErrorV1> {
        let consumer = self.next_consumer;
        self.next_consumer = consumer.successor();
        let delivery = match try_receive_runtime_pull_delivery(
            &self.connection,
            self.permits.for_consumer(consumer),
        )
        .await
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?
        {
            None => return Ok(false),
            Some(delivery) => delivery,
        };
        match consumer {
            ConsumerV1::Candidate => {
                let decoded = decode_candidate_v1(delivery.exact_bytes())
                    .map_err(|_| AttachmentPreviewRuntimeErrorV1::InvalidDelivery)?;
                self.persistence
                    .persist_scan_candidate(
                        &self.logical_human_owner_id,
                        &decoded.fact,
                        decoded.payload_sha256,
                        consumed_at_unix_millis,
                    )
                    .await
                    .map_err(persistence_error)?;
            }
            ConsumerV1::Safety => {
                if let Some(decoded) = decode_safety_v1(delivery.exact_bytes())
                    .map_err(|_| AttachmentPreviewRuntimeErrorV1::InvalidDelivery)?
                {
                    self.persistence
                        .persist_safety_fact(
                            &self.logical_human_owner_id,
                            decoded.fact,
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
                    .map_err(|_| AttachmentPreviewRuntimeErrorV1::InvalidDelivery)?
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
        }
        delivery
            .acknowledge()
            .await
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn materialize_pending_custody_requests(
        &self,
        now_unix_millis: i64,
        now_nanos: i32,
    ) -> Result<usize, AttachmentPreviewRuntimeErrorV1> {
        let pending = self
            .persistence
            .pending_custody_delegations(&self.logical_human_owner_id, 64)
            .await
            .map_err(persistence_error)?;
        for item in &pending {
            let request_id = attachment_preview_custody_delegation_request_id_v1(
                item.intent.run_id,
                item.intent.candidate_message_id,
                item.intent.safety_message_id,
            );
            let seconds = now_unix_millis / 1_000;
            let record = build_request_attachment_preview_custody_delegation_outbox_record_v1(
                RequestAttachmentPreviewCustodyDelegationV1 {
                    request_id: request_id.to_vec(),
                    preview_run_id: item.intent.run_id.to_vec(),
                    attachment_anchor_id: item.intent.attachment_anchor_id.to_vec(),
                    candidate_message_id: item.intent.candidate_message_id.to_vec(),
                    candidate_envelope_sha256: item.intent.candidate_envelope_sha256.to_vec(),
                    safety_message_id: item.intent.safety_message_id.to_vec(),
                    safety_evidence_id: item.intent.safety_evidence_id.to_vec(),
                    logical_owner_id: self.logical_human_owner_id.clone(),
                },
                seconds
                    .checked_add(300)
                    .ok_or(AttachmentPreviewRuntimeErrorV1::Unavailable)?,
                &AttachmentPreviewCustodyEnvelopeContextV1 {
                    module_id: ATTACHMENT_PREVIEW_MODULE_ID_V1.to_owned(),
                    runtime_instance_id: self.runtime_instance_id.clone(),
                    runtime_generation: self.runtime_generation,
                    recorded_at_unix_seconds: seconds,
                    recorded_at_nanos: now_nanos,
                },
            )
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::InvalidJob)?;
            self.persistence
                .store_custody_delegation_outbox(
                    &self.logical_human_owner_id,
                    &PersistAttachmentPreviewCustodyDelegationV1 {
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
    ) -> Result<usize, AttachmentPreviewRuntimeErrorV1> {
        relay_custody_outbox_once_v1(
            &self.persistence,
            &self.logical_human_owner_id,
            &self.connection,
            &self.publish_permit,
            now_unix_millis,
        )
        .await
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)
    }

    pub async fn process_next_job(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<JobTickV1, AttachmentPreviewRuntimeErrorV1> {
        self.persistence
            .recover_expired_jobs(&self.logical_human_owner_id, now_unix_millis)
            .await
            .inspect_err(|_| developer_job_stage("recover_expired"))
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
            .inspect_err(|_| developer_job_stage("claim"))
            .map_err(persistence_error)?
        else {
            return Ok(JobTickV1::Idle);
        };
        let source_receipt = transfer_source_v1(&mut self.control_channel, &claimed)
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        let source = read_source_v1(&mut self.control_channel, &claimed, source_receipt)
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        let rendered = match self.renderer.render(&source) {
            Ok(value) => value,
            Err(error) => {
                let error = renderer_error(error);
                self.persistence
                    .fail_job(
                        &claimed.logical_owner_id,
                        claimed.job_id,
                        &claimed.lease,
                        error,
                        now_unix_millis,
                    )
                    .await
                    .map_err(persistence_error)?;
                return Ok(JobTickV1::Rejected(error));
            }
        };
        let renderer_identity_sha256 = attachment_preview_renderer_identity_v1();
        let (derived_reference_id, derived_receipt_sha256, preview_size_bytes) = write_derived_v1(
            &mut self.control_channel,
            &claimed,
            Zeroizing::new(rendered.bytes),
            renderer_identity_sha256,
        )
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        let target_blob_receipt = PreviewTargetBlobReceiptV1 {
            reference_id: derived_reference_id,
            receipt_sha256: derived_receipt_sha256,
        };
        self.persistence
            .record_target_blob_receipt(
                &claimed.logical_owner_id,
                claimed.job_id,
                &claimed.lease,
                target_blob_receipt,
                now_unix_millis,
            )
            .await
            .inspect_err(|_| developer_job_stage("record_target_receipt"))
            .map_err(persistence_error)?;
        self.persistence
            .complete_job(
                &claimed.logical_owner_id,
                claimed.job_id,
                &claimed.lease,
                RenderedAttachmentPreviewArtifactV1 {
                    target_blob_receipt,
                    renderer_identity_sha256,
                    preview_kind: rendered.preview_kind,
                    content_type: rendered.content_type,
                    preview_size_bytes,
                    truncated: rendered.truncated,
                },
                now_unix_millis,
            )
            .await
            .inspect_err(|_| developer_job_stage("complete"))
            .map_err(persistence_error)?;
        Ok(JobTickV1::Completed)
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, AttachmentPreviewRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?
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
        let response = dispatch_attachment_preview_client_request_v1(
            &self.persistence,
            self.runtime_generation,
            self.grant_epoch,
            &request,
            now_unix_millis,
        )
        .await;
        if validate_module_client_response_v1(&response).is_err() {
            return Err(AttachmentPreviewRuntimeErrorV1::Unavailable);
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
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn pump_client_realtime_once(
        &mut self,
    ) -> Result<bool, AttachmentPreviewRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = AttachmentPreviewNestedRequestDispatcherV1 {
            persistence: &self.persistence,
            runtime_generation: self.runtime_generation,
            grant_epoch: self.grant_epoch,
        };
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
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        result
    }

    fn write_client_error(
        &mut self,
        correlation_id: [u8; 16],
    ) -> Result<bool, AttachmentPreviewRuntimeErrorV1> {
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_request".to_owned(),
                },
            )
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }
}

fn client_delivery_response_v1(
    response: ModuleClientResponseV1,
) -> ManagedRuntimeControlResponseV1 {
    ManagedRuntimeControlResponseV1 {
        result: Some(ControlResult::ClientDelivery(
            ManagedRuntimeClientDeliveryResponseV1 {
                response: Some(response),
            },
        )),
        error_code: String::new(),
    }
}

fn client_rejected_response_v1(request_id: u64) -> ModuleClientResponseV1 {
    client_error_response_v1(request_id, "REJECTED")
}

fn client_unavailable_response_v1(request_id: u64) -> ModuleClientResponseV1 {
    client_error_response_v1(request_id, "UNAVAILABLE")
}

fn client_error_response_v1(request_id: u64, error_code: &str) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: 1,
        request_id,
        response_payload: Vec::new(),
        error_code: error_code.to_owned(),
    }
}

fn developer_job_stage(stage: &str) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_attachment_preview_job_denied stage={stage}");
    }
}

struct SubscribePermitsV1 {
    candidate: RuntimeSubscribePermitV1,
    safety: RuntimeSubscribePermitV1,
    delegated: RuntimeSubscribePermitV1,
    rejected: RuntimeSubscribePermitV1,
}

impl SubscribePermitsV1 {
    fn bind(
        permits: Vec<RuntimeSubscribePermitV1>,
    ) -> Result<Self, AttachmentPreviewRuntimeErrorV1> {
        let expected = [
            attachment_security_scan_candidate_observed_contract_reference_v1(),
            communication_attachment_safety_state_changed_contract_reference_v1(),
            attachment_preview_custody_delegated_contract_reference_v1(),
            attachment_preview_custody_delegation_rejected_contract_reference_v1(),
        ];
        let mut selected: [Option<RuntimeSubscribePermitV1>; 4] = [None, None, None, None];
        for permit in permits {
            let Some(contract) = permit.contract() else {
                return Err(AttachmentPreviewRuntimeErrorV1::Admission);
            };
            let Some(index) = expected
                .iter()
                .position(|expected| exact_contract(contract, expected))
            else {
                return Err(AttachmentPreviewRuntimeErrorV1::Admission);
            };
            if selected[index].replace(permit).is_some() {
                return Err(AttachmentPreviewRuntimeErrorV1::Admission);
            }
        }
        let [candidate, safety, delegated, rejected] = selected;
        Ok(Self {
            candidate: candidate.ok_or(AttachmentPreviewRuntimeErrorV1::Admission)?,
            safety: safety.ok_or(AttachmentPreviewRuntimeErrorV1::Admission)?,
            delegated: delegated.ok_or(AttachmentPreviewRuntimeErrorV1::Admission)?,
            rejected: rejected.ok_or(AttachmentPreviewRuntimeErrorV1::Admission)?,
        })
    }

    fn for_consumer(&self, consumer: ConsumerV1) -> &RuntimeSubscribePermitV1 {
        match consumer {
            ConsumerV1::Candidate => &self.candidate,
            ConsumerV1::Safety => &self.safety,
            ConsumerV1::Delegated => &self.delegated,
            ConsumerV1::Rejected => &self.rejected,
        }
    }
}

#[derive(Clone, Copy)]
enum ConsumerV1 {
    Candidate,
    Safety,
    Delegated,
    Rejected,
}

impl ConsumerV1 {
    const fn successor(self) -> Self {
        match self {
            Self::Candidate => Self::Safety,
            Self::Safety => Self::Delegated,
            Self::Delegated => Self::Rejected,
            Self::Rejected => Self::Candidate,
        }
    }
}

fn validate_open(
    descriptor: &[u8],
    settings: &[u8],
    admission: &AttachmentPreviewRuntimeAdmissionV1,
    event_hub_endpoint: &str,
    credential_revision: u64,
) -> Result<(), AttachmentPreviewRuntimeErrorV1> {
    if descriptor.is_empty()
        || settings.is_empty()
        || admission.module_owner_id != ATTACHMENT_PREVIEW_OWNER_V1
        || !valid_owner_id(&admission.logical_human_owner_id)
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
        || credential_revision == 0
        || event_hub_endpoint.is_empty()
    {
        return Err(AttachmentPreviewRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &AttachmentPreviewRuntimeAdmissionV1,
) -> Result<(), AttachmentPreviewRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(AttachmentPreviewRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &AttachmentPreviewRuntimeAdmissionV1,
) -> Result<(), AttachmentPreviewRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<Zeroizing<Vec<u8>>, AttachmentPreviewRuntimeErrorV1> {
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
    Err(AttachmentPreviewRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &AttachmentPreviewRuntimeAdmissionV1,
) -> Result<StorageBindingV1, AttachmentPreviewRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != ATTACHMENT_PREVIEW_OWNER_V1
        || configuration.owner != ATTACHMENT_PREVIEW_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(AttachmentPreviewRuntimeErrorV1::Admission);
    }
    StorageBindingV1::new(
        StorageBindingIdentityV1::new(
            configuration.storage_instance_id.clone(),
            configuration.database_id.clone(),
            configuration.owner.clone(),
            admission.registration_id.clone(),
            configuration.runtime_instance_id.clone(),
        )
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?,
        StorageBindingFencesV1::new(
            configuration.storage_generation,
            admission.runtime_generation,
            admission.grant_epoch,
            configuration.role_epoch,
            configuration.credential_revision,
            configuration.storage_bundle_revision,
        )
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?,
        StorageBindingAccessV1::new(
            configuration.runtime_principal.clone(),
            configuration.pool_alias.clone(),
            StorageEffectiveBudgetsV1::new(
                u16::try_from(configuration.max_connections)
                    .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?,
                configuration.statement_timeout_millis,
            )
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?,
            configuration
                .storage_bundle_digest
                .as_slice()
                .try_into()
                .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?,
        )
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| AttachmentPreviewRuntimeErrorV1::Admission)
}

fn persistence_error(
    error: AttachmentPreviewPersistenceErrorV1,
) -> AttachmentPreviewRuntimeErrorV1 {
    match error {
        AttachmentPreviewPersistenceErrorV1::StorageUnavailable => {
            AttachmentPreviewRuntimeErrorV1::Unavailable
        }
        AttachmentPreviewPersistenceErrorV1::InvalidInput
        | AttachmentPreviewPersistenceErrorV1::InvalidRow
        | AttachmentPreviewPersistenceErrorV1::EvidenceConflict
        | AttachmentPreviewPersistenceErrorV1::NotFound
        | AttachmentPreviewPersistenceErrorV1::TicketExpired
        | AttachmentPreviewPersistenceErrorV1::TicketUsed
        | AttachmentPreviewPersistenceErrorV1::StaleFence => {
            AttachmentPreviewRuntimeErrorV1::InvalidJob
        }
    }
}

fn client_realtime_error(error: ClientRealtimeErrorV1) -> AttachmentPreviewRuntimeErrorV1 {
    match error {
        ClientRealtimeErrorV1::Unavailable => AttachmentPreviewRuntimeErrorV1::Unavailable,
        ClientRealtimeErrorV1::Persistence(error) => persistence_error(error),
        ClientRealtimeErrorV1::InvalidTransition => AttachmentPreviewRuntimeErrorV1::InvalidJob,
    }
}

const fn renderer_error(error: AttachmentPreviewRendererErrorV1) -> AttachmentPreviewErrorCodeV1 {
    match error {
        AttachmentPreviewRendererErrorV1::Unsupported => AttachmentPreviewErrorCodeV1::Unsupported,
        AttachmentPreviewRendererErrorV1::SourceTooLarge => {
            AttachmentPreviewErrorCodeV1::SourceTooLarge
        }
        AttachmentPreviewRendererErrorV1::Empty
        | AttachmentPreviewRendererErrorV1::InvalidContent => {
            AttachmentPreviewErrorCodeV1::InvalidContent
        }
        AttachmentPreviewRendererErrorV1::OutputTooLarge
        | AttachmentPreviewRendererErrorV1::Failed => AttachmentPreviewErrorCodeV1::RendererFailed,
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
    Rejected(AttachmentPreviewErrorCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewRuntimeErrorV1 {
    Admission,
    InvalidDelivery,
    InvalidJob,
    Unavailable,
}

impl AttachmentPreviewRuntimeErrorV1 {
    #[must_use]
    pub const fn sanitized_reason_code(self) -> &'static str {
        match self {
            Self::Admission => "attachment_preview_runtime_admission_rejected",
            Self::InvalidDelivery => "attachment_preview_runtime_invalid_delivery",
            Self::InvalidJob => "attachment_preview_runtime_invalid_job",
            Self::Unavailable => "attachment_preview_runtime_unavailable",
        }
    }
}

pub fn current_runtime_time_v1() -> Result<(i64, i32), AttachmentPreviewRuntimeErrorV1> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?;
    Ok((
        i64::try_from(now.as_millis()).map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?,
        i32::try_from(now.subsec_nanos())
            .map_err(|_| AttachmentPreviewRuntimeErrorV1::Unavailable)?,
    ))
}
