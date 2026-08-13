//! Kernel-fenced Event Hub consumer for the Communications domain.

use std::{
    os::unix::net::UnixStream,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use makosh_communications_ai_source_api::{
    communication_explanation_source_prepare_contract_reference_v1,
    communication_reply_source_prepare_contract_reference_v1,
    communication_summary_source_prepare_contract_reference_v1,
    communication_translation_source_prepare_contract_reference_v1,
};
use makosh_communications_attachment_contract::admission::{
    communication_attachment_blob_admission_observed_contract_reference_v1,
    communication_attachment_safety_verdict_observed_contract_reference_v1,
};
use makosh_communications_call_evidence_ingress::call_evidence_observed_contract_reference_v1;
use makosh_communications_call_evidence_persistence::{
    CallEvidenceConsumeOutcomeV1, CallEvidencePersistenceErrorV1,
    CommunicationsCallEvidencePersistenceV1,
};
use makosh_communications_cross_channel_forward_source_api::cross_channel_forward_source_prepare_contract_reference_v1;
use makosh_communications_domain::COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1;
use makosh_communications_evidence_export_source_api::evidence_export_prepare_contract_reference_v1;
use makosh_communications_ingress::admission::communication_observed_contract_reference_v1;
use makosh_communications_note_source_api::communication_note_source_prepare_contract_reference_v1;
use makosh_communications_persistence::CommunicationsDurablePersistence;
use makosh_communications_recipient_source_api::communication_recipient_source_prepare_contract_reference_v1;
use makosh_communications_retained_evidence_replay_contract::communications_replay_command_contract_reference_v1;
use makosh_communications_retained_evidence_replay_persistence::{
    CommunicationsRetainedEvidenceReplayPersistenceV1, RetainedCommunicationsReplayErrorV1,
};
use makosh_communications_task_source_api::communication_task_source_prepare_contract_reference_v1;
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_runtime_protocol::managed_control::ManagedControlChannelV2;
use makosh_runtime_protocol::managed_control::{
    ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
    RejectManagedControlRequestsV2,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use makosh_runtime_protocol::v1::{
    ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeReadyRequestV1, ManagedStorageRuntimeConfigurationV1, ModuleClientResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;
use makosh_runtime_protocol::validation::module_client::{
    validate_module_client_request_v1, validate_module_client_response_v1,
};
use makosh_runtime_protocol::validation::module_query::validate_module_query_response_v1;
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};

use crate::{
    ai_source::{CommunicationsAiSourceDeliveryErrorV1, consume_next_ai_source_prepare_v1},
    attachment_observation_consumer::{
        consume_next_attachment_blob_admission_observation_v1,
        consume_next_attachment_safety_verdict_observation_v1,
    },
    call_evidence_client_port::handle_call_evidence_module_query_delivery_v1,
    call_evidence_consumer::consume_next_call_evidence_observation_v1,
    call_evidence_realtime::{
        CallEvidenceClientRealtimeErrorV1, CallEvidenceClientRealtimePublisherV1,
    },
    canonical_outbox::CanonicalEventContextV1,
    client_port::{CommunicationsClientRequestDependenciesV1, dispatch_module_client_request_v1},
    consumer::{
        CommunicationsDeliveryErrorV1, CommunicationsEventConsumeErrorV1,
        consume_next_observation_v1,
    },
    content_ticket_store::CommunicationsContentTicketStoreV1,
    cross_channel_forward_source::{
        CommunicationsCrossChannelForwardSourceDeliveryErrorV1,
        consume_next_cross_channel_forward_source_prepare_v1,
    },
    custody_worker::{CommunicationsCustodyWorkerErrorV1, process_next_body_custody_transfer_v1},
    domain_outbox::{CommunicationsDomainOutboxRelayErrorV1, relay_domain_outbox_once},
    evidence_export_source::{
        CommunicationsEvidenceExportDeliveryErrorV1, consume_next_evidence_export_prepare_v1,
    },
    explanation_source::{
        CommunicationsExplanationSourceDeliveryErrorV1, consume_next_explanation_source_prepare_v1,
    },
    note_source::{CommunicationsNoteSourceDeliveryErrorV1, consume_next_note_source_prepare_v1},
    query_module_port::handle_module_query_delivery_v1,
    recipient_source::{
        CommunicationsRecipientSourceDeliveryErrorV1, consume_next_recipient_source_prepare_v1,
    },
    retained_evidence_replay_consumer::{
        CommunicationsReplayCommandConsumeErrorV1, CommunicationsReplayConsumerContextV1,
        consume_next_communications_replay_command_v1,
    },
    retained_evidence_replay_result::{
        CommunicationsReplayResultRelayErrorV1, relay_communications_replay_result_once_v1,
    },
    search_access::CommunicationsSearchAccessV1,
    search_worker::process_next_derived_index_job_v1,
    summary_source::{
        CommunicationsSummarySourceDeliveryErrorV1, consume_next_summary_source_prepare_v1,
    },
    task_source::{CommunicationsTaskSourceDeliveryErrorV1, consume_next_task_source_prepare_v1},
    translation_source::{
        CommunicationsTranslationSourceDeliveryErrorV1, consume_next_translation_source_prepare_v1,
    },
};

pub struct CommunicationsRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

pub struct CommunicationsEventRuntimeV1 {
    control_channel: ManagedControlChannelV2<UnixStream>,
    connection: RuntimeJetStreamConnection,
    permits: CommunicationsSubscribePermitsV1,
    consumer_schedule: CommunicationsConsumerScheduleV1,
    domain_publish_permit: RuntimePublishPermitV1,
    persistence: CommunicationsDurablePersistence,
    call_evidence_persistence: CommunicationsCallEvidencePersistenceV1,
    replay_persistence: CommunicationsRetainedEvidenceReplayPersistenceV1,
    call_evidence_realtime: CallEvidenceClientRealtimePublisherV1,
    call_evidence_realtime_pending: bool,
    search_access: CommunicationsSearchAccessV1,
    content_tickets: Arc<CommunicationsContentTicketStoreV1>,
    runtime_instance_id: String,
    runtime_generation: u64,
    logical_owner_id: String,
    logical_human_owner_id: String,
    registration_id: String,
    grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsEventRuntimeErrorV1 {
    Admission,
    Unavailable,
}

struct CommunicationsSubscribePermitsV1 {
    observation: RuntimeSubscribePermitV1,
    call_evidence: RuntimeSubscribePermitV1,
    attachment_blob_admission: RuntimeSubscribePermitV1,
    attachment_safety_verdict: RuntimeSubscribePermitV1,
    evidence_export_prepare: RuntimeSubscribePermitV1,
    cross_channel_forward_source_prepare: RuntimeSubscribePermitV1,
    ai_source_prepare: RuntimeSubscribePermitV1,
    summary_source_prepare: RuntimeSubscribePermitV1,
    translation_source_prepare: RuntimeSubscribePermitV1,
    explanation_source_prepare: RuntimeSubscribePermitV1,
    note_source_prepare: RuntimeSubscribePermitV1,
    recipient_source_prepare: RuntimeSubscribePermitV1,
    task_source_prepare: RuntimeSubscribePermitV1,
    replay_command: RuntimeSubscribePermitV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommunicationsConsumerV1 {
    Observation,
    CallEvidence,
    AttachmentBlobAdmission,
    AttachmentSafetyVerdict,
    EvidenceExportPrepare,
    CrossChannelForwardSourcePrepare,
    AiSourcePrepare,
    SummarySourcePrepare,
    TranslationSourcePrepare,
    ExplanationSourcePrepare,
    NoteSourcePrepare,
    RecipientSourcePrepare,
    TaskSourcePrepare,
    ReplayCommand,
}

const COMMUNICATIONS_OBSERVATION_BURST_V1: u16 = 512;

struct CommunicationsConsumerScheduleV1 {
    next: CommunicationsConsumerV1,
    next_secondary: CommunicationsConsumerV1,
    observation_burst_remaining: u16,
}

impl CommunicationsConsumerScheduleV1 {
    const fn new() -> Self {
        Self {
            next: CommunicationsConsumerV1::Observation,
            next_secondary: CommunicationsConsumerV1::CallEvidence,
            observation_burst_remaining: COMMUNICATIONS_OBSERVATION_BURST_V1,
        }
    }

    const fn selected(&self) -> CommunicationsConsumerV1 {
        self.next
    }

    fn complete_attempt(&mut self, delivered: bool) {
        if self.next == CommunicationsConsumerV1::Observation {
            if delivered {
                self.observation_burst_remaining =
                    self.observation_burst_remaining.saturating_sub(1);
                if self.observation_burst_remaining > 0 {
                    return;
                }
            }
            self.next = self.next_secondary;
            self.next_secondary = match self.next_secondary.successor() {
                CommunicationsConsumerV1::Observation => CommunicationsConsumerV1::CallEvidence,
                successor => successor,
            };
            self.observation_burst_remaining = COMMUNICATIONS_OBSERVATION_BURST_V1;
            return;
        }
        self.next = CommunicationsConsumerV1::Observation;
        self.observation_burst_remaining = COMMUNICATIONS_OBSERVATION_BURST_V1;
    }
}

impl CommunicationsConsumerV1 {
    const fn successor(self) -> Self {
        match self {
            Self::Observation => Self::CallEvidence,
            Self::CallEvidence => Self::AttachmentBlobAdmission,
            Self::AttachmentBlobAdmission => Self::AttachmentSafetyVerdict,
            Self::AttachmentSafetyVerdict => Self::EvidenceExportPrepare,
            Self::EvidenceExportPrepare => Self::CrossChannelForwardSourcePrepare,
            Self::CrossChannelForwardSourcePrepare => Self::AiSourcePrepare,
            Self::AiSourcePrepare => Self::SummarySourcePrepare,
            Self::SummarySourcePrepare => Self::TranslationSourcePrepare,
            Self::TranslationSourcePrepare => Self::ExplanationSourcePrepare,
            Self::ExplanationSourcePrepare => Self::NoteSourcePrepare,
            Self::NoteSourcePrepare => Self::RecipientSourcePrepare,
            Self::RecipientSourcePrepare => Self::TaskSourcePrepare,
            Self::TaskSourcePrepare => Self::ReplayCommand,
            Self::ReplayCommand => Self::Observation,
        }
    }
}

#[cfg(test)]
mod consumer_schedule_tests {
    use super::{
        COMMUNICATIONS_OBSERVATION_BURST_V1, CommunicationsConsumerScheduleV1,
        CommunicationsConsumerV1,
    };

    #[test]
    fn observation_backlog_drains_in_a_bounded_burst_before_fair_rotation() {
        let mut schedule = CommunicationsConsumerScheduleV1::new();
        for _ in 1..COMMUNICATIONS_OBSERVATION_BURST_V1 {
            assert_eq!(schedule.selected(), CommunicationsConsumerV1::Observation);
            schedule.complete_attempt(true);
        }
        assert_eq!(schedule.selected(), CommunicationsConsumerV1::Observation);
        schedule.complete_attempt(true);
        assert_eq!(schedule.selected(), CommunicationsConsumerV1::CallEvidence);
        schedule.complete_attempt(false);
        assert_eq!(schedule.selected(), CommunicationsConsumerV1::Observation);
    }

    #[test]
    fn empty_observation_consumer_rotates_without_spinning() {
        let mut schedule = CommunicationsConsumerScheduleV1::new();
        schedule.complete_attempt(false);
        assert_eq!(schedule.selected(), CommunicationsConsumerV1::CallEvidence);
        schedule.complete_attempt(false);
        assert_eq!(schedule.selected(), CommunicationsConsumerV1::Observation);
        schedule.complete_attempt(false);
        assert_eq!(
            schedule.selected(),
            CommunicationsConsumerV1::AttachmentBlobAdmission
        );
    }
}

impl CommunicationsSubscribePermitsV1 {
    fn bind(
        permits: Vec<RuntimeSubscribePermitV1>,
    ) -> Result<Self, CommunicationsEventRuntimeErrorV1> {
        let observation = communication_observed_contract_reference_v1();
        let call_evidence = call_evidence_observed_contract_reference_v1();
        let attachment_blob_admission =
            communication_attachment_blob_admission_observed_contract_reference_v1();
        let attachment_safety_verdict =
            communication_attachment_safety_verdict_observed_contract_reference_v1();
        let evidence_export_prepare = evidence_export_prepare_contract_reference_v1();
        let cross_channel_forward_source_prepare =
            cross_channel_forward_source_prepare_contract_reference_v1();
        let ai_source_prepare = communication_reply_source_prepare_contract_reference_v1();
        let summary_source_prepare = communication_summary_source_prepare_contract_reference_v1();
        let translation_source_prepare =
            communication_translation_source_prepare_contract_reference_v1();
        let explanation_source_prepare =
            communication_explanation_source_prepare_contract_reference_v1();
        let note_source_prepare = communication_note_source_prepare_contract_reference_v1();
        let recipient_source_prepare =
            communication_recipient_source_prepare_contract_reference_v1();
        let task_source_prepare = communication_task_source_prepare_contract_reference_v1();
        let replay_command = communications_replay_command_contract_reference_v1();
        let mut observation_permit = None;
        let mut call_evidence_permit = None;
        let mut attachment_blob_admission_permit = None;
        let mut attachment_safety_verdict_permit = None;
        let mut evidence_export_prepare_permit = None;
        let mut cross_channel_forward_source_prepare_permit = None;
        let mut ai_source_prepare_permit = None;
        let mut summary_source_prepare_permit = None;
        let mut translation_source_prepare_permit = None;
        let mut explanation_source_prepare_permit = None;
        let mut note_source_prepare_permit = None;
        let mut recipient_source_prepare_permit = None;
        let mut task_source_prepare_permit = None;
        let mut replay_command_permit = None;
        for permit in permits {
            let Some(contract) = permit.contract() else {
                return Err(CommunicationsEventRuntimeErrorV1::Admission);
            };
            if exact_contract(contract, &observation) {
                replace_once(&mut observation_permit, permit)?;
            } else if exact_contract(contract, &call_evidence) {
                replace_once(&mut call_evidence_permit, permit)?;
            } else if exact_contract(contract, &attachment_blob_admission) {
                replace_once(&mut attachment_blob_admission_permit, permit)?;
            } else if exact_contract(contract, &attachment_safety_verdict) {
                replace_once(&mut attachment_safety_verdict_permit, permit)?;
            } else if exact_contract(contract, &evidence_export_prepare) {
                replace_once(&mut evidence_export_prepare_permit, permit)?;
            } else if exact_contract(contract, &cross_channel_forward_source_prepare) {
                replace_once(&mut cross_channel_forward_source_prepare_permit, permit)?;
            } else if exact_contract(contract, &ai_source_prepare) {
                replace_once(&mut ai_source_prepare_permit, permit)?;
            } else if exact_contract(contract, &summary_source_prepare) {
                replace_once(&mut summary_source_prepare_permit, permit)?;
            } else if exact_contract(contract, &translation_source_prepare) {
                replace_once(&mut translation_source_prepare_permit, permit)?;
            } else if exact_contract(contract, &explanation_source_prepare) {
                replace_once(&mut explanation_source_prepare_permit, permit)?;
            } else if exact_contract(contract, &note_source_prepare) {
                replace_once(&mut note_source_prepare_permit, permit)?;
            } else if exact_contract(contract, &recipient_source_prepare) {
                replace_once(&mut recipient_source_prepare_permit, permit)?;
            } else if exact_contract(contract, &task_source_prepare) {
                replace_once(&mut task_source_prepare_permit, permit)?;
            } else if exact_contract(contract, &replay_command) {
                replace_once(&mut replay_command_permit, permit)?;
            } else {
                return Err(CommunicationsEventRuntimeErrorV1::Admission);
            }
        }
        Ok(Self {
            observation: observation_permit.ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            call_evidence: call_evidence_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            attachment_blob_admission: attachment_blob_admission_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            attachment_safety_verdict: attachment_safety_verdict_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            evidence_export_prepare: evidence_export_prepare_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            cross_channel_forward_source_prepare: cross_channel_forward_source_prepare_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            ai_source_prepare: ai_source_prepare_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            summary_source_prepare: summary_source_prepare_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            translation_source_prepare: translation_source_prepare_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            explanation_source_prepare: explanation_source_prepare_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            note_source_prepare: note_source_prepare_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            recipient_source_prepare: recipient_source_prepare_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            task_source_prepare: task_source_prepare_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
            replay_command: replay_command_permit
                .ok_or(CommunicationsEventRuntimeErrorV1::Admission)?,
        })
    }
}

fn replace_once(
    slot: &mut Option<RuntimeSubscribePermitV1>,
    permit: RuntimeSubscribePermitV1,
) -> Result<(), CommunicationsEventRuntimeErrorV1> {
    slot.replace(permit)
        .is_none()
        .then_some(())
        .ok_or(CommunicationsEventRuntimeErrorV1::Admission)
}

fn exact_contract(left: &ContractReferenceV1, right: &ContractReferenceV1) -> bool {
    left.owner == right.owner
        && left.name == right.name
        && left.major == right.major
        && left.revision == right.revision
        && left.schema_sha256 == right.schema_sha256
}

struct CommunicationsNestedRequestDispatcher<'a> {
    persistence: &'a CommunicationsDurablePersistence,
    call_evidence_persistence: &'a CommunicationsCallEvidencePersistenceV1,
    logical_owner_id: &'a str,
    search_access: &'a mut CommunicationsSearchAccessV1,
    content_tickets: &'a Arc<CommunicationsContentTicketStoreV1>,
}

impl ManagedControlRequestDispatcherV2<UnixStream> for CommunicationsNestedRequestDispatcher<'_> {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: makosh_runtime_protocol::v1::ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    let mut reject_nested_request = RejectManagedControlRequestsV2;
                    let dependencies = CommunicationsClientRequestDependenciesV1 {
                        persistence: self.persistence,
                        call_evidence_persistence: self.call_evidence_persistence,
                        logical_human_owner_id: self.logical_owner_id,
                        tickets: self.content_tickets,
                    };
                    let response = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(
                            dispatch_module_client_request_v1(
                                &dependencies,
                                self.search_access,
                                channel,
                                &mut reject_nested_request,
                                &request,
                            ),
                        )
                    });
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(response),
                            },
                        )),
                        error_code: String::new(),
                    }
                }
                Some(request) => ManagedRuntimeControlResponseV1 {
                    result: Some(ControlResult::ClientDelivery(
                        ManagedRuntimeClientDeliveryResponseV1 {
                            response: Some(ModuleClientResponseV1 {
                                protocol_major: 1,
                                request_id: request.request_id,
                                response_payload: Vec::new(),
                                error_code: "REJECTED".to_owned(),
                            }),
                        },
                    )),
                    error_code: String::new(),
                },
                None => ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                },
            },
            Some(Operation::DeliverModuleQuery(delivery)) => {
                let mut reject_nested_request = RejectManagedControlRequestsV2;
                let response = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(handle_module_query_delivery_v1(
                        self.persistence,
                        self.search_access,
                        channel,
                        &mut reject_nested_request,
                        delivery,
                    ))
                });
                ManagedRuntimeControlResponseV1 {
                    result: Some(ControlResult::ModuleQueryDelivery(response)),
                    error_code: String::new(),
                }
            }
            _ => ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        };
        channel.write_response(correlation_id, response)
    }
}

impl CommunicationsEventRuntimeV1 {
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &CommunicationsRuntimeAdmissionV1,
        event_hub_endpoint: &str,
        credential_revision: u64,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
    ) -> Result<Self, CommunicationsEventRuntimeErrorV1> {
        if descriptor_bytes.is_empty()
            || settings_schema_bytes.is_empty()
            || admission.logical_owner_id.trim().is_empty()
            || admission.logical_human_owner_id.trim().is_empty()
            || admission.registration_id.trim().is_empty()
            || admission.runtime_instance_id.trim().is_empty()
            || admission.runtime_generation == 0
            || admission.grant_epoch == 0
            || credential_revision == 0
            || event_hub_endpoint.trim().is_empty()
        {
            return Err(CommunicationsEventRuntimeErrorV1::Admission);
        }
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate_managed_runtime_v2(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )
        .map_err(|_| admission_at("runtime_authentication"))?;
        let access = request_managed_runtime_event_access_v2(
            &mut control_channel,
            &admission.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            credential_revision,
        )
        .map_err(|_| unavailable_at("event_access"))?;
        let permits = access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| admission_at("event_subscribe_permits"))?;
        let permits = CommunicationsSubscribePermitsV1::bind(permits)
            .map_err(|_| admission_at("event_subscribe_permit_binding"))?;
        let domain_publish_permit = access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| admission_at("event_publish_permit"))?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| admission_at("event_identity"))?;
        let connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| unavailable_at("event_connection"))?;
        let binding = storage_binding(&storage_configuration, admission)
            .map_err(|_| admission_at("storage_binding"))?;
        let vault_public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| admission_at("vault_public_key"))?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| admission_at("vault_context"))?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_runtime_credential(&mut leases, &binding)
            .await
            .map_err(|_| unavailable_at("storage_credential"))?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| admission_at("storage_credential_encoding"))?;
        let persistence = CommunicationsDurablePersistence::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(|_| unavailable_at("storage_connection"))?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(|_| unavailable_at("storage_readiness"))?;
        let call_evidence_persistence =
            CommunicationsCallEvidencePersistenceV1::from_owner_local_pool(
                persistence.owner_local_pool_handle(),
            );
        call_evidence_persistence
            .verify_storage_ready()
            .await
            .map_err(|_| unavailable_at("call_evidence_storage_readiness"))?;
        let replay_persistence =
            CommunicationsRetainedEvidenceReplayPersistenceV1::from_owner_local_pool(
                persistence.owner_local_pool_handle(),
            );
        replay_persistence
            .verify_storage_ready()
            .await
            .map_err(|_| unavailable_at("replay_storage_readiness"))?;
        let mut control_channel = leases.into_route_port().into_channel();
        let started_at_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        persistence
            .reconcile_search_projection_jobs(
                COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1,
                i64::try_from(started_at_unix_seconds.as_secs())
                    .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?,
            )
            .await
            .map_err(|_| unavailable_at("search_projection"))?;
        let search_access = CommunicationsSearchAccessV1::open(admission, &storage_configuration)
            .map_err(|_| admission_at("search_access"))?;
        let content_tickets = Arc::new(CommunicationsContentTicketStoreV1::new());
        let mut search_access = search_access;
        let mut ready_dispatcher = CommunicationsNestedRequestDispatcher {
            persistence: &persistence,
            call_evidence_persistence: &call_evidence_persistence,
            logical_owner_id: &admission.logical_human_owner_id,
            search_access: &mut search_access,
            content_tickets: &content_tickets,
        };
        signal_managed_runtime_ready(&mut control_channel, admission, &mut ready_dispatcher)
            .map_err(|error| match error {
                CommunicationsEventRuntimeErrorV1::Admission => admission_at("ready_signal"),
                CommunicationsEventRuntimeErrorV1::Unavailable => unavailable_at("ready_signal"),
            })?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            control_channel,
            connection,
            permits,
            consumer_schedule: CommunicationsConsumerScheduleV1::new(),
            domain_publish_permit,
            persistence,
            call_evidence_persistence,
            replay_persistence,
            call_evidence_realtime: CallEvidenceClientRealtimePublisherV1::default(),
            call_evidence_realtime_pending: true,
            search_access,
            content_tickets,
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
            logical_owner_id: admission.logical_owner_id.clone(),
            logical_human_owner_id: admission.logical_human_owner_id.clone(),
            registration_id: admission.registration_id.clone(),
            grant_epoch: admission.grant_epoch,
        })
    }

    pub async fn try_handle_control_delivery(
        &mut self,
    ) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| unavailable_at("client_receive"))?
        else {
            return Ok(false);
        };
        let operation = match request.operation {
            Some(Operation::DeliverModuleQuery(delivery)) => {
                let request_id = delivery.request_id.clone();
                let mut nested_search_access = self.search_access.clone();
                let mut nested_dispatcher = CommunicationsNestedRequestDispatcher {
                    persistence: &self.persistence,
                    call_evidence_persistence: &self.call_evidence_persistence,
                    logical_owner_id: &self.logical_owner_id,
                    search_access: &mut nested_search_access,
                    content_tickets: &self.content_tickets,
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| unavailable_at("module_query_blocking"))?;
                let response = if delivery.contract.as_ref()
                    == Some(
                        &crate::admission::communications_call_evidence_query_contract_reference_v1(
                        ),
                    ) {
                    handle_call_evidence_module_query_delivery_v1(
                        &self.call_evidence_persistence,
                        &self.logical_human_owner_id,
                        delivery,
                    )
                    .await
                } else {
                    handle_module_query_delivery_v1(
                        &self.persistence,
                        &mut self.search_access,
                        &mut self.control_channel,
                        &mut nested_dispatcher,
                        delivery,
                    )
                    .await
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| unavailable_at("module_query_nonblocking"))?;
                validate_module_query_response_v1(&response)
                    .map_err(|_| unavailable_at("module_query_response_validate"))?;
                if response.request_id != request_id {
                    return Err(admission_at("module_query_response_request_id"));
                }
                self.control_channel
                    .write_response(
                        correlation_id,
                        ManagedRuntimeControlResponseV1 {
                            result: Some(ControlResult::ModuleQueryDelivery(response)),
                            error_code: String::new(),
                        },
                    )
                    .map_err(|_| unavailable_at("module_query_write"))?;
                return Ok(true);
            }
            operation => operation,
        };
        let request = match operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) => request,
                None => {
                    self.control_channel
                        .write_response(
                            correlation_id,
                            ManagedRuntimeControlResponseV1 {
                                result: None,
                                error_code: "managed_runtime_control_invalid_client_delivery"
                                    .to_owned(),
                            },
                        )
                        .map_err(|_| unavailable_at("client_invalid_write"))?;
                    return Ok(true);
                }
            },
            _ => {
                self.control_channel
                    .write_response(
                        correlation_id,
                        ManagedRuntimeControlResponseV1 {
                            result: None,
                            error_code: "managed_runtime_control_unexpected_request".to_owned(),
                        },
                    )
                    .map_err(|_| unavailable_at("client_unexpected_write"))?;
                return Ok(true);
            }
        };
        if validate_module_client_request_v1(&request).is_err() {
            self.control_channel
                .write_response(
                    correlation_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(ModuleClientResponseV1 {
                                    protocol_major: 1,
                                    request_id: request.request_id,
                                    response_payload: Vec::new(),
                                    error_code: "REJECTED".to_owned(),
                                }),
                            },
                        )),
                        error_code: String::new(),
                    },
                )
                .map_err(|_| unavailable_at("client_rejected_write"))?;
            return Ok(true);
        }
        let mut nested_search_access = self.search_access.clone();
        let mut nested_dispatcher = CommunicationsNestedRequestDispatcher {
            persistence: &self.persistence,
            call_evidence_persistence: &self.call_evidence_persistence,
            logical_owner_id: &self.logical_owner_id,
            search_access: &mut nested_search_access,
            content_tickets: &self.content_tickets,
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| unavailable_at("client_blocking"))?;
        let dependencies = CommunicationsClientRequestDependenciesV1 {
            persistence: &self.persistence,
            call_evidence_persistence: &self.call_evidence_persistence,
            logical_human_owner_id: &self.logical_human_owner_id,
            tickets: &self.content_tickets,
        };
        let response = dispatch_module_client_request_v1(
            &dependencies,
            &mut self.search_access,
            &mut self.control_channel,
            &mut nested_dispatcher,
            &request,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| unavailable_at("client_nonblocking"))?;
        validate_module_client_response_v1(&response)
            .map_err(|_| unavailable_at("client_response_validate"))?;
        if response.request_id != request.request_id {
            return Err(admission_at("client_response_request_id"));
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
            .map_err(|_| unavailable_at("client_write"))?;
        Ok(true)
    }

    pub async fn consume_next(&mut self) -> Result<(), CommunicationsDeliveryErrorV1> {
        let canonical_event_context = self.canonical_event_context()?;
        let consumer = self.consumer_schedule.selected();
        let result = match consumer {
            CommunicationsConsumerV1::Observation => consume_next_observation_v1(
                &self.persistence,
                &self.connection,
                &self.permits.observation,
                &canonical_event_context,
            )
            .await
            .map(|_| ()),
            CommunicationsConsumerV1::CallEvidence => consume_next_call_evidence_observation_v1(
                &self.call_evidence_persistence,
                &self.connection,
                &self.permits.call_evidence,
                &self.logical_human_owner_id,
                canonical_event_context.recorded_at_unix_seconds,
            )
            .await
            .map(|outcome| {
                if matches!(outcome, CallEvidenceConsumeOutcomeV1::Applied { .. }) {
                    self.call_evidence_realtime_pending = true;
                }
            }),
            CommunicationsConsumerV1::AttachmentBlobAdmission => {
                consume_next_attachment_blob_admission_observation_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.attachment_blob_admission,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
            }
            CommunicationsConsumerV1::AttachmentSafetyVerdict => {
                consume_next_attachment_safety_verdict_observation_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.attachment_safety_verdict,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
            }
            CommunicationsConsumerV1::EvidenceExportPrepare => {
                let mut nested_search_access = self.search_access.clone();
                let mut dispatcher = CommunicationsNestedRequestDispatcher {
                    persistence: &self.persistence,
                    call_evidence_persistence: &self.call_evidence_persistence,
                    logical_owner_id: &self.logical_owner_id,
                    search_access: &mut nested_search_access,
                    content_tickets: &self.content_tickets,
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                let result = consume_next_evidence_export_prepare_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.evidence_export_prepare,
                    &mut self.control_channel,
                    &mut dispatcher,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
                .map_err(evidence_export_delivery_error);
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                result
            }
            CommunicationsConsumerV1::CrossChannelForwardSourcePrepare => {
                let mut nested_search_access = self.search_access.clone();
                let mut dispatcher = CommunicationsNestedRequestDispatcher {
                    persistence: &self.persistence,
                    call_evidence_persistence: &self.call_evidence_persistence,
                    logical_owner_id: &self.logical_owner_id,
                    search_access: &mut nested_search_access,
                    content_tickets: &self.content_tickets,
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                let result = consume_next_cross_channel_forward_source_prepare_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.cross_channel_forward_source_prepare,
                    &mut self.control_channel,
                    &mut dispatcher,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
                .map_err(cross_channel_forward_source_delivery_error);
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                result
            }
            CommunicationsConsumerV1::AiSourcePrepare => {
                let mut nested_search_access = self.search_access.clone();
                let mut dispatcher = CommunicationsNestedRequestDispatcher {
                    persistence: &self.persistence,
                    call_evidence_persistence: &self.call_evidence_persistence,
                    logical_owner_id: &self.logical_owner_id,
                    search_access: &mut nested_search_access,
                    content_tickets: &self.content_tickets,
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                let result = consume_next_ai_source_prepare_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.ai_source_prepare,
                    &mut self.control_channel,
                    &mut dispatcher,
                    &self.logical_human_owner_id,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
                .map_err(ai_source_delivery_error);
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                result
            }
            CommunicationsConsumerV1::SummarySourcePrepare => {
                let mut nested_search_access = self.search_access.clone();
                let mut dispatcher = CommunicationsNestedRequestDispatcher {
                    persistence: &self.persistence,
                    call_evidence_persistence: &self.call_evidence_persistence,
                    logical_owner_id: &self.logical_owner_id,
                    search_access: &mut nested_search_access,
                    content_tickets: &self.content_tickets,
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                let result = consume_next_summary_source_prepare_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.summary_source_prepare,
                    &mut self.control_channel,
                    &mut dispatcher,
                    &self.logical_human_owner_id,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
                .map_err(summary_source_delivery_error);
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                result
            }
            CommunicationsConsumerV1::TranslationSourcePrepare => {
                let mut nested_search_access = self.search_access.clone();
                let mut dispatcher = CommunicationsNestedRequestDispatcher {
                    persistence: &self.persistence,
                    call_evidence_persistence: &self.call_evidence_persistence,
                    logical_owner_id: &self.logical_owner_id,
                    search_access: &mut nested_search_access,
                    content_tickets: &self.content_tickets,
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                let result = consume_next_translation_source_prepare_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.translation_source_prepare,
                    &mut self.control_channel,
                    &mut dispatcher,
                    &self.logical_human_owner_id,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
                .map_err(translation_source_delivery_error);
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                result
            }
            CommunicationsConsumerV1::ExplanationSourcePrepare => {
                let mut nested_search_access = self.search_access.clone();
                let mut dispatcher = CommunicationsNestedRequestDispatcher {
                    persistence: &self.persistence,
                    call_evidence_persistence: &self.call_evidence_persistence,
                    logical_owner_id: &self.logical_owner_id,
                    search_access: &mut nested_search_access,
                    content_tickets: &self.content_tickets,
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                let result = consume_next_explanation_source_prepare_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.explanation_source_prepare,
                    &mut self.control_channel,
                    &mut dispatcher,
                    &self.logical_human_owner_id,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
                .map_err(explanation_source_delivery_error);
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                result
            }
            CommunicationsConsumerV1::NoteSourcePrepare => {
                let mut nested_search_access = self.search_access.clone();
                let mut dispatcher = CommunicationsNestedRequestDispatcher {
                    persistence: &self.persistence,
                    call_evidence_persistence: &self.call_evidence_persistence,
                    logical_owner_id: &self.logical_owner_id,
                    search_access: &mut nested_search_access,
                    content_tickets: &self.content_tickets,
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                let result = consume_next_note_source_prepare_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.note_source_prepare,
                    &mut self.control_channel,
                    &mut dispatcher,
                    &self.logical_human_owner_id,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
                .map_err(note_source_delivery_error);
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                result
            }
            CommunicationsConsumerV1::RecipientSourcePrepare => {
                let mut nested_search_access = self.search_access.clone();
                let mut dispatcher = CommunicationsNestedRequestDispatcher {
                    persistence: &self.persistence,
                    call_evidence_persistence: &self.call_evidence_persistence,
                    logical_owner_id: &self.logical_owner_id,
                    search_access: &mut nested_search_access,
                    content_tickets: &self.content_tickets,
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                let result = consume_next_recipient_source_prepare_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.recipient_source_prepare,
                    &mut self.control_channel,
                    &mut dispatcher,
                    &self.logical_human_owner_id,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
                .map_err(recipient_source_delivery_error);
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                result
            }
            CommunicationsConsumerV1::TaskSourcePrepare => {
                let mut nested_search_access = self.search_access.clone();
                let mut dispatcher = CommunicationsNestedRequestDispatcher {
                    persistence: &self.persistence,
                    call_evidence_persistence: &self.call_evidence_persistence,
                    logical_owner_id: &self.logical_owner_id,
                    search_access: &mut nested_search_access,
                    content_tickets: &self.content_tickets,
                };
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(false)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                let result = consume_next_task_source_prepare_v1(
                    &self.persistence,
                    &self.connection,
                    &self.permits.task_source_prepare,
                    &mut self.control_channel,
                    &mut dispatcher,
                    &self.logical_human_owner_id,
                    &canonical_event_context,
                )
                .await
                .map(|_| ())
                .map_err(task_source_delivery_error);
                self.control_channel
                    .inner_mut()
                    .set_nonblocking(true)
                    .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
                result
            }
            CommunicationsConsumerV1::ReplayCommand => {
                let event_context = self.canonical_event_context()?;
                consume_next_communications_replay_command_v1(
                    &self.replay_persistence,
                    &self.connection,
                    &self.permits.replay_command,
                    &self.domain_publish_permit,
                    &CommunicationsReplayConsumerContextV1 {
                        logical_owner_id: self.logical_human_owner_id.clone(),
                        producer_registration_id: self.registration_id.clone(),
                        runtime_instance_id: self.runtime_instance_id.clone(),
                        runtime_generation: self.runtime_generation,
                        grant_epoch: self.grant_epoch,
                        execution_attempt: 1,
                        completed_at_unix_seconds: event_context.recorded_at_unix_seconds,
                        completed_at_nanos: event_context.recorded_at_nanos,
                    },
                )
                .await
                .map(|_| ())
                .map_err(replay_command_error)
            }
        };
        self.consumer_schedule.complete_attempt(result.is_ok());
        result
    }

    pub async fn publish_call_evidence_realtime(
        &mut self,
    ) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
        if !self.call_evidence_realtime_pending {
            return Ok(false);
        }
        let mut nested_search_access = self.search_access.clone();
        let mut dispatcher = CommunicationsNestedRequestDispatcher {
            persistence: &self.persistence,
            call_evidence_persistence: &self.call_evidence_persistence,
            logical_owner_id: &self.logical_owner_id,
            search_access: &mut nested_search_access,
            content_tickets: &self.content_tickets,
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| unavailable_at("call_evidence_realtime_blocking"))?;
        let result = self
            .call_evidence_realtime
            .publish_pending(
                &self.call_evidence_persistence,
                &mut self.control_channel,
                &mut dispatcher,
                &self.logical_human_owner_id,
            )
            .await
            .map_err(call_evidence_realtime_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| unavailable_at("call_evidence_realtime_nonblocking"))?;
        match result {
            Ok(outcome) => {
                self.call_evidence_realtime_pending = !outcome.drained;
                Ok(outcome.published)
            }
            Err(error) => Err(error),
        }
    }

    pub async fn process_next_body_custody_transfer(
        &mut self,
    ) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
        let context = self
            .canonical_event_context()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = CommunicationsNestedRequestDispatcher {
            persistence: &self.persistence,
            call_evidence_persistence: &self.call_evidence_persistence,
            logical_owner_id: &self.logical_owner_id,
            search_access: &mut self.search_access,
            content_tickets: &self.content_tickets,
        };
        custody_worker_outcome(
            process_next_body_custody_transfer_v1(
                &mut self.control_channel,
                &mut dispatcher,
                &self.persistence,
                &format!("{}:{}", self.runtime_instance_id, self.runtime_generation),
                context.recorded_at_unix_seconds,
            )
            .await,
        )
    }

    pub async fn process_next_derived_index_job(
        &mut self,
    ) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
        let context = self
            .canonical_event_context()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        let mut nested_search_access = self.search_access.clone();
        let mut nested_dispatcher = CommunicationsNestedRequestDispatcher {
            persistence: &self.persistence,
            call_evidence_persistence: &self.call_evidence_persistence,
            logical_owner_id: &self.logical_owner_id,
            search_access: &mut nested_search_access,
            content_tickets: &self.content_tickets,
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| unavailable_at("search_worker_blocking"))?;
        let result = process_next_derived_index_job_v1(
            &self.persistence,
            &mut self.search_access,
            &mut self.control_channel,
            &mut nested_dispatcher,
            &format!("{}:{}", self.runtime_instance_id, self.runtime_generation),
            context.recorded_at_unix_seconds,
        )
        .await
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| unavailable_at("search_worker_nonblocking"))?;
        result
    }

    pub async fn reconcile_search_projection_jobs(
        &self,
    ) -> Result<usize, CommunicationsEventRuntimeErrorV1> {
        let context = self
            .canonical_event_context()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
        self.persistence
            .reconcile_search_projection_jobs(
                COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1,
                context.recorded_at_unix_seconds,
            )
            .await
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)
    }

    fn canonical_event_context(
        &self,
    ) -> Result<CanonicalEventContextV1, CommunicationsDeliveryErrorV1> {
        let recorded_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?;
        Ok(CanonicalEventContextV1 {
            runtime_instance_id: self.runtime_instance_id.clone(),
            runtime_generation: self.runtime_generation,
            recorded_at_unix_seconds: i64::try_from(recorded_at.as_secs())
                .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?,
            recorded_at_nanos: i32::try_from(recorded_at.subsec_nanos())
                .map_err(|_| CommunicationsDeliveryErrorV1::Unavailable)?,
        })
    }

    pub async fn relay_domain_outbox(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<usize, CommunicationsDomainOutboxRelayErrorV1> {
        relay_domain_outbox_once(
            &self.persistence,
            &self.connection,
            &self.domain_publish_permit,
            published_at_unix_seconds,
        )
        .await
    }

    pub async fn relay_replay_result(
        &self,
        published_at_unix_seconds: i64,
    ) -> Result<bool, CommunicationsReplayResultRelayErrorV1> {
        relay_communications_replay_result_once_v1(
            &self.replay_persistence,
            &self.connection,
            &self.domain_publish_permit,
            published_at_unix_seconds,
        )
        .await
    }

    pub async fn index_retained_attachment_safety_events(
        &self,
        indexed_at_unix_seconds: i64,
    ) -> Result<usize, RetainedCommunicationsReplayErrorV1> {
        self.replay_persistence
            .index_existing_attachment_safety_events(256, indexed_at_unix_seconds)
            .await
    }
}

fn replay_command_error(
    error: CommunicationsReplayCommandConsumeErrorV1,
) -> CommunicationsDeliveryErrorV1 {
    match error {
        CommunicationsReplayCommandConsumeErrorV1::EventUnavailable
        | CommunicationsReplayCommandConsumeErrorV1::ReplayRetryable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
        CommunicationsReplayCommandConsumeErrorV1::Decode(_) => {
            CommunicationsDeliveryErrorV1::InvalidEnvelope
        }
        CommunicationsReplayCommandConsumeErrorV1::Persistence(_) => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
        CommunicationsReplayCommandConsumeErrorV1::ResultEnvelope => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::InvalidPayload,
            )
        }
    }
}

fn custody_worker_outcome(
    outcome: Result<bool, CommunicationsCustodyWorkerErrorV1>,
) -> Result<bool, CommunicationsEventRuntimeErrorV1> {
    match outcome {
        Ok(processed) => Ok(processed),
        Err(
            error @ (CommunicationsCustodyWorkerErrorV1::RetryPending
            | CommunicationsCustodyWorkerErrorV1::StorageUnavailable),
        ) => {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_communications_custody_retry={error:?}");
            }
            Ok(false)
        }
    }
}

fn evidence_export_delivery_error(
    error: CommunicationsEvidenceExportDeliveryErrorV1,
) -> CommunicationsDeliveryErrorV1 {
    match error {
        CommunicationsEvidenceExportDeliveryErrorV1::Unavailable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
        CommunicationsEvidenceExportDeliveryErrorV1::InvalidEnvelope => {
            CommunicationsDeliveryErrorV1::InvalidEnvelope
        }
        CommunicationsEvidenceExportDeliveryErrorV1::InvalidPayload => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::InvalidPayload,
            )
        }
        CommunicationsEvidenceExportDeliveryErrorV1::Persistence => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
    }
}

fn cross_channel_forward_source_delivery_error(
    error: CommunicationsCrossChannelForwardSourceDeliveryErrorV1,
) -> CommunicationsDeliveryErrorV1 {
    match error {
        CommunicationsCrossChannelForwardSourceDeliveryErrorV1::Unavailable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
        CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidEnvelope => {
            CommunicationsDeliveryErrorV1::InvalidEnvelope
        }
        CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidPayload => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::InvalidPayload,
            )
        }
        CommunicationsCrossChannelForwardSourceDeliveryErrorV1::Persistence => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
    }
}

fn ai_source_delivery_error(
    error: CommunicationsAiSourceDeliveryErrorV1,
) -> CommunicationsDeliveryErrorV1 {
    match error {
        CommunicationsAiSourceDeliveryErrorV1::Unavailable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
        CommunicationsAiSourceDeliveryErrorV1::InvalidEnvelope => {
            CommunicationsDeliveryErrorV1::InvalidEnvelope
        }
        CommunicationsAiSourceDeliveryErrorV1::InvalidPayload => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::InvalidPayload,
            )
        }
        CommunicationsAiSourceDeliveryErrorV1::Persistence => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
    }
}

fn summary_source_delivery_error(
    error: CommunicationsSummarySourceDeliveryErrorV1,
) -> CommunicationsDeliveryErrorV1 {
    match error {
        CommunicationsSummarySourceDeliveryErrorV1::Unavailable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
        CommunicationsSummarySourceDeliveryErrorV1::InvalidEnvelope => {
            CommunicationsDeliveryErrorV1::InvalidEnvelope
        }
        CommunicationsSummarySourceDeliveryErrorV1::InvalidPayload => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::InvalidPayload,
            )
        }
        CommunicationsSummarySourceDeliveryErrorV1::Persistence => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
    }
}

fn translation_source_delivery_error(
    error: CommunicationsTranslationSourceDeliveryErrorV1,
) -> CommunicationsDeliveryErrorV1 {
    match error {
        CommunicationsTranslationSourceDeliveryErrorV1::Unavailable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
        CommunicationsTranslationSourceDeliveryErrorV1::InvalidEnvelope => {
            CommunicationsDeliveryErrorV1::InvalidEnvelope
        }
        CommunicationsTranslationSourceDeliveryErrorV1::InvalidPayload => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::InvalidPayload,
            )
        }
        CommunicationsTranslationSourceDeliveryErrorV1::Persistence => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
    }
}

fn explanation_source_delivery_error(
    error: CommunicationsExplanationSourceDeliveryErrorV1,
) -> CommunicationsDeliveryErrorV1 {
    match error {
        CommunicationsExplanationSourceDeliveryErrorV1::Unavailable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
        CommunicationsExplanationSourceDeliveryErrorV1::InvalidEnvelope => {
            CommunicationsDeliveryErrorV1::InvalidEnvelope
        }
        CommunicationsExplanationSourceDeliveryErrorV1::InvalidPayload => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::InvalidPayload,
            )
        }
        CommunicationsExplanationSourceDeliveryErrorV1::Persistence => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
    }
}

fn recipient_source_delivery_error(
    error: CommunicationsRecipientSourceDeliveryErrorV1,
) -> CommunicationsDeliveryErrorV1 {
    match error {
        CommunicationsRecipientSourceDeliveryErrorV1::Unavailable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
        CommunicationsRecipientSourceDeliveryErrorV1::InvalidEnvelope => {
            CommunicationsDeliveryErrorV1::InvalidEnvelope
        }
        CommunicationsRecipientSourceDeliveryErrorV1::InvalidPayload => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::InvalidPayload,
            )
        }
        CommunicationsRecipientSourceDeliveryErrorV1::Persistence => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
    }
}

fn note_source_delivery_error(
    error: CommunicationsNoteSourceDeliveryErrorV1,
) -> CommunicationsDeliveryErrorV1 {
    match error {
        CommunicationsNoteSourceDeliveryErrorV1::Unavailable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
        CommunicationsNoteSourceDeliveryErrorV1::InvalidEnvelope => {
            CommunicationsDeliveryErrorV1::InvalidEnvelope
        }
        CommunicationsNoteSourceDeliveryErrorV1::InvalidPayload => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::InvalidPayload,
            )
        }
        CommunicationsNoteSourceDeliveryErrorV1::Persistence => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
    }
}

fn task_source_delivery_error(
    error: CommunicationsTaskSourceDeliveryErrorV1,
) -> CommunicationsDeliveryErrorV1 {
    match error {
        CommunicationsTaskSourceDeliveryErrorV1::Unavailable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
        CommunicationsTaskSourceDeliveryErrorV1::InvalidEnvelope => {
            CommunicationsDeliveryErrorV1::InvalidEnvelope
        }
        CommunicationsTaskSourceDeliveryErrorV1::InvalidPayload => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::InvalidPayload,
            )
        }
        CommunicationsTaskSourceDeliveryErrorV1::Persistence => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
    }
}

const fn call_evidence_realtime_error(
    error: CallEvidenceClientRealtimeErrorV1,
) -> CommunicationsEventRuntimeErrorV1 {
    match error {
        CallEvidenceClientRealtimeErrorV1::InvalidRecord
        | CallEvidenceClientRealtimeErrorV1::Persistence(
            CallEvidencePersistenceErrorV1::InvalidInput
            | CallEvidencePersistenceErrorV1::InvalidRow
            | CallEvidencePersistenceErrorV1::InboxHashConflict,
        ) => CommunicationsEventRuntimeErrorV1::Admission,
        CallEvidenceClientRealtimeErrorV1::Persistence(
            CallEvidencePersistenceErrorV1::StorageUnavailable,
        )
        | CallEvidenceClientRealtimeErrorV1::Unavailable => {
            CommunicationsEventRuntimeErrorV1::Unavailable
        }
    }
}

fn unavailable_at(stage: &str) -> CommunicationsEventRuntimeErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_communications_runtime_startup_unavailable stage={stage}");
    }
    CommunicationsEventRuntimeErrorV1::Unavailable
}

fn admission_at(stage: &str) -> CommunicationsEventRuntimeErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_communications_runtime_admission stage={stage}");
    }
    CommunicationsEventRuntimeErrorV1::Admission
}

async fn resolve_storage_runtime_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, CommunicationsEventRuntimeErrorV1> {
    const MAX_ATTEMPTS: usize = 20;
    for attempt in 0..MAX_ATTEMPTS {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt + 1 < MAX_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    Err(CommunicationsEventRuntimeErrorV1::Unavailable)
}

fn authenticate_managed_runtime_v2(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    admission: &CommunicationsRuntimeAdmissionV1,
) -> Result<(), CommunicationsEventRuntimeErrorV1> {
    control_channel
        .inner_mut()
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| {
            control_channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    let response = control_channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    let registration_id = response.registration_id;
    let runtime_generation = response.runtime_generation;
    let grant_epoch = response.grant_epoch;
    if registration_id != admission.registration_id
        || runtime_generation != admission.runtime_generation
        || grant_epoch != admission.grant_epoch
    {
        return Err(CommunicationsEventRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_managed_runtime_ready(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &CommunicationsRuntimeAdmissionV1,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
) -> Result<(), CommunicationsEventRuntimeErrorV1> {
    control_channel
        .signal_ready_with_dispatch(
            ManagedRuntimeReadyRequestV1 {
                registration_id: admission.registration_id.clone(),
                runtime_generation: admission.runtime_generation,
                grant_epoch: admission.grant_epoch,
            },
            dispatcher,
        )
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)?;
    control_channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| control_channel.inner_mut().set_write_timeout(None))
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &CommunicationsRuntimeAdmissionV1,
) -> Result<StorageBindingV1, CommunicationsEventRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(CommunicationsEventRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| CommunicationsEventRuntimeErrorV1::Admission)
}

#[cfg(test)]
mod tests {
    use super::{
        CommunicationsConsumerV1, CommunicationsCustodyWorkerErrorV1, custody_worker_outcome,
    };

    #[test]
    fn transient_custody_dependencies_keep_the_runtime_available_for_retry() {
        assert_eq!(
            custody_worker_outcome(Err(CommunicationsCustodyWorkerErrorV1::RetryPending)),
            Ok(false)
        );
        assert_eq!(
            custody_worker_outcome(Err(CommunicationsCustodyWorkerErrorV1::StorageUnavailable)),
            Ok(false)
        );
    }

    #[test]
    fn event_consumers_advance_without_empty_consumer_starvation() {
        let first = CommunicationsConsumerV1::Observation;
        let second = first.successor();
        let third = second.successor();
        let fourth = third.successor();
        let fifth = fourth.successor();
        let sixth = fifth.successor();
        let seventh = sixth.successor();
        let eighth = seventh.successor();
        let ninth = eighth.successor();
        let tenth = ninth.successor();
        let eleventh = tenth.successor();
        let twelfth = eleventh.successor();
        let thirteenth = twelfth.successor();
        let fourteenth = thirteenth.successor();

        assert_eq!(
            [
                first,
                second,
                third,
                fourth,
                fifth,
                sixth,
                seventh,
                eighth,
                ninth,
                tenth,
                eleventh,
                twelfth,
                thirteenth,
                fourteenth,
                fourteenth.successor()
            ],
            [
                CommunicationsConsumerV1::Observation,
                CommunicationsConsumerV1::CallEvidence,
                CommunicationsConsumerV1::AttachmentBlobAdmission,
                CommunicationsConsumerV1::AttachmentSafetyVerdict,
                CommunicationsConsumerV1::EvidenceExportPrepare,
                CommunicationsConsumerV1::CrossChannelForwardSourcePrepare,
                CommunicationsConsumerV1::AiSourcePrepare,
                CommunicationsConsumerV1::SummarySourcePrepare,
                CommunicationsConsumerV1::TranslationSourcePrepare,
                CommunicationsConsumerV1::ExplanationSourcePrepare,
                CommunicationsConsumerV1::NoteSourcePrepare,
                CommunicationsConsumerV1::RecipientSourcePrepare,
                CommunicationsConsumerV1::TaskSourcePrepare,
                CommunicationsConsumerV1::ReplayCommand,
                CommunicationsConsumerV1::Observation,
            ]
        );
    }
}
