//! WhatsApp runtime boundary for typed host observations.
//!
//! This crate owns no browser API, credential material, provider command
//! execution, or Communications persistence. It converts an admitted host
//! observation into an exact provider-neutral Communications outbox record.

pub mod admission;
pub mod client_port;
mod communications_outbox;
pub mod delivery_intent_consumer;
pub mod delivery_intent_execution;
pub mod delivery_intent_outbox;
pub mod delivery_intent_result;
pub mod delivery_intent_worker;
pub mod host_bridge_port;
pub mod host_bridge_transport;
pub mod managed;
pub mod settings;

use makosh_communications_ingress::{
    CommunicationEvidenceKindV1, CommunicationObservationDraft, ObservationEnvelopeBuildErrorV1,
    ObservationEnvelopeContextV1, build_observation_outbox_record_v1,
};
use makosh_whatsapp_api::host_bridge::WhatsAppHostBridgeEnvelopeV1;
use makosh_whatsapp_api::{
    WhatsAppProviderCommand, WhatsAppProviderCommandStateV1, WhatsAppProviderCommandStatusV1,
    client_wire, provider_command_account_id, provider_command_operation_id,
    validate_provider_command,
};
use makosh_whatsapp_core::{
    WhatsAppCoreError, WhatsAppOperationalProjectionError, WhatsAppOperationalProjectionV1,
    communication_observation_draft, project_host_observation,
    project_operational_host_observation,
};
use makosh_whatsapp_persistence::{
    WhatsAppClaimedCommandV1, WhatsAppDeliveryRouteLocatorV1, WhatsAppDurablePersistence,
    WhatsAppDurablePersistenceError, WhatsAppHostObservationRecordV1,
    WhatsAppOperationalObservationV1, WhatsAppProviderCommandStateV1 as PersistedCommandStateV1,
};

pub use communications_outbox::{
    WhatsAppCommunicationsOutboxRelayError, relay_communications_outbox_once,
};

pub const PACKAGE: &str = "makosh-whatsapp-runtime";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppRuntimeIdentity {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
}

#[derive(Clone)]
pub struct WhatsAppRuntimeAdmission {
    pub logical_owner_id: String,
    pub module_registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Debug)]
pub enum WhatsAppHostIngressError {
    AccountScope,
    Core(WhatsAppCoreError),
    OperationalCore(WhatsAppOperationalProjectionError),
    Envelope(ObservationEnvelopeBuildErrorV1),
    Persistence(WhatsAppDurablePersistenceError),
}

#[derive(Debug)]
pub enum WhatsAppCommandQueueError {
    InvalidCommand,
    Persistence(WhatsAppDurablePersistenceError),
    Wire,
}

#[derive(Debug)]
pub enum WhatsAppOperationalQueryError {
    AccountScope,
    Persistence(WhatsAppDurablePersistenceError),
}

#[derive(Debug)]
pub enum WhatsAppOperationalReplayError {
    AccountScope,
    Persistence(WhatsAppDurablePersistenceError),
}

impl WhatsAppRuntimeIdentity {
    pub fn observation_context(
        &self,
        recorded_at_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> ObservationEnvelopeContextV1 {
        ObservationEnvelopeContextV1 {
            runtime_instance_id: self.runtime_instance_id.clone(),
            runtime_generation: self.runtime_generation,
            module_id: "makosh-whatsapp-runtime".to_owned(),
            recorded_at_unix_seconds,
            recorded_at_nanos,
        }
    }
}

pub fn draft_host_observation(
    envelope: &WhatsAppHostBridgeEnvelopeV1,
) -> Result<CommunicationObservationDraft, WhatsAppHostIngressError> {
    communication_observation_draft(
        &project_host_observation(envelope).map_err(WhatsAppHostIngressError::Core)?,
    )
    .map_err(WhatsAppHostIngressError::Core)
}

pub async fn accept_host_observation(
    durable: &WhatsAppDurablePersistence,
    identity: &WhatsAppRuntimeIdentity,
    envelope: &WhatsAppHostBridgeEnvelopeV1,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
) -> Result<(), WhatsAppHostIngressError> {
    makosh_whatsapp_api::host_bridge::validate_host_bridge_envelope(envelope)
        .map_err(WhatsAppCoreError::HostBridge)
        .map_err(WhatsAppHostIngressError::Core)?;
    if let makosh_whatsapp_api::host_bridge::WhatsAppHostObservationV1::CommandResult {
        operation_id,
        host_claim_id,
        succeeded,
        ..
    } = &envelope.observation
    {
        return durable
            .complete_provider_command(
                operation_id,
                &envelope.account_id,
                host_claim_id,
                *succeeded,
                recorded_at_unix_seconds,
            )
            .await
            .map_err(WhatsAppHostIngressError::Persistence)
            .and_then(|completed| {
                completed
                    .then_some(())
                    .ok_or(WhatsAppHostIngressError::Persistence(
                        WhatsAppDurablePersistenceError::CommandConflict,
                    ))
            });
    }
    let operational = project_operational_host_observation(envelope)
        .map_err(WhatsAppHostIngressError::OperationalCore)?
        .map(|projection| match projection {
            WhatsAppOperationalProjectionV1::Event {
                provider_event_id,
                event,
            } => WhatsAppOperationalObservationV1::Event {
                provider_event_id,
                event,
            },
            WhatsAppOperationalProjectionV1::ResyncState {
                provider_event_id,
                account_id,
                observed_at_unix_seconds,
                complete,
            } => WhatsAppOperationalObservationV1::ResyncState {
                provider_event_id,
                account_id,
                observed_at_unix_seconds,
                complete,
            },
        });
    let communication_projection = match project_host_observation(envelope) {
        Ok(projection) => Some(projection),
        Err(WhatsAppCoreError::UnsupportedObservation) if operational.is_some() => None,
        Err(error) => return Err(WhatsAppHostIngressError::Core(error)),
    };
    let record = communication_projection
        .as_ref()
        .map(|projection| {
            let draft = communication_observation_draft(projection)
                .map_err(WhatsAppHostIngressError::Core)?;
            build_observation_outbox_record_v1(
                &draft,
                &identity.observation_context(recorded_at_unix_seconds, recorded_at_nanos),
            )
            .map_err(WhatsAppHostIngressError::Envelope)
        })
        .transpose()?;
    let delivery_route_locator = communication_projection
        .as_ref()
        .filter(|projection| projection.evidence_kind == CommunicationEvidenceKindV1::ChatMessage)
        .map(|projection| {
            let provider_chat_id = projection.provider_conversation_id.as_deref().ok_or(
                WhatsAppHostIngressError::Persistence(WhatsAppDurablePersistenceError::InvalidRow),
            )?;
            WhatsAppDeliveryRouteLocatorV1::new(
                &projection.account_id,
                provider_chat_id,
                &projection.provider_record_id,
            )
            .map_err(WhatsAppHostIngressError::Persistence)
        })
        .transpose()?;
    let observation = WhatsAppHostObservationRecordV1 {
        account_id: envelope.account_id.clone(),
        provider_event_id: envelope.provider_event_id.clone(),
        evidence_kind: communication_projection.as_ref().map_or(0, |projection| {
            evidence_kind_value(projection.evidence_kind)
        }),
        observed_at_unix_seconds: envelope.observed_at_unix_seconds,
    };
    durable
        .record_host_observation_projection_and_enqueue(
            &observation,
            operational.as_ref(),
            record.as_ref(),
            delivery_route_locator.as_ref(),
            recorded_at_unix_seconds,
        )
        .await
        .map_err(WhatsAppHostIngressError::Persistence)
        .map(|_| ())
}

pub async fn enqueue_provider_command(
    durable: &WhatsAppDurablePersistence,
    command: &WhatsAppProviderCommand,
    requested_at_unix_seconds: i64,
) -> Result<(), WhatsAppCommandQueueError> {
    validate_provider_command(command).map_err(|_| WhatsAppCommandQueueError::InvalidCommand)?;
    durable
        .enqueue_provider_command(
            provider_command_operation_id(command),
            provider_command_account_id(command),
            &client_wire::encode_command(command),
            requested_at_unix_seconds,
        )
        .await
        .map(|_| ())
        .map_err(WhatsAppCommandQueueError::Persistence)
}

pub async fn provider_command_status(
    durable: &WhatsAppDurablePersistence,
    operation_id: &str,
) -> Result<Option<WhatsAppProviderCommandStatusV1>, WhatsAppCommandQueueError> {
    durable
        .provider_command_status(operation_id)
        .await
        .map(|status| {
            status.map(|value| WhatsAppProviderCommandStatusV1 {
                operation_id: value.operation_id,
                account_id: value.account_id,
                state: match value.state {
                    PersistedCommandStateV1::Pending => WhatsAppProviderCommandStateV1::Pending,
                    PersistedCommandStateV1::Claimed => WhatsAppProviderCommandStateV1::Claimed,
                    PersistedCommandStateV1::Succeeded => WhatsAppProviderCommandStateV1::Succeeded,
                    PersistedCommandStateV1::Failed => WhatsAppProviderCommandStateV1::Failed,
                },
                requested_at_unix_seconds: value.requested_at_unix_seconds,
                completed_at_unix_seconds: value.completed_at_unix_seconds,
            })
        })
        .map_err(WhatsAppCommandQueueError::Persistence)
}

pub async fn claim_provider_commands(
    durable: &WhatsAppDurablePersistence,
    account_id: &str,
    host_claim_id: &str,
    now_unix_seconds: i64,
    lease_seconds: i64,
    limit: i64,
) -> Result<Vec<WhatsAppProviderCommand>, WhatsAppCommandQueueError> {
    let claimed = durable
        .claim_provider_commands(
            account_id,
            host_claim_id,
            now_unix_seconds,
            lease_seconds,
            limit,
        )
        .await
        .map_err(WhatsAppCommandQueueError::Persistence)?;
    decode_claimed_commands(claimed)
}

fn decode_claimed_commands(
    claimed: Vec<WhatsAppClaimedCommandV1>,
) -> Result<Vec<WhatsAppProviderCommand>, WhatsAppCommandQueueError> {
    claimed
        .into_iter()
        .map(|record| {
            let command = client_wire::decode_command(&record.exact_command_bytes)
                .map_err(|_| WhatsAppCommandQueueError::Wire)?;
            (provider_command_operation_id(&command) == record.operation_id
                && provider_command_account_id(&command) == record.account_id)
                .then_some(command)
                .ok_or(WhatsAppCommandQueueError::Wire)
        })
        .collect()
}

const fn evidence_kind_value(
    value: makosh_communications_ingress::CommunicationEvidenceKindV1,
) -> i16 {
    match value {
        makosh_communications_ingress::CommunicationEvidenceKindV1::EmailMessage => 1,
        makosh_communications_ingress::CommunicationEvidenceKindV1::ChatMessage => 2,
        makosh_communications_ingress::CommunicationEvidenceKindV1::MessageEdited => 3,
        makosh_communications_ingress::CommunicationEvidenceKindV1::MessageDeleted => 4,
        makosh_communications_ingress::CommunicationEvidenceKindV1::ReactionChanged => 5,
        makosh_communications_ingress::CommunicationEvidenceKindV1::DeliveryStateChanged => 6,
        makosh_communications_ingress::CommunicationEvidenceKindV1::ConversationStateChanged => 7,
        makosh_communications_ingress::CommunicationEvidenceKindV1::ParticipantChanged => 8,
        makosh_communications_ingress::CommunicationEvidenceKindV1::MediaChanged => 9,
        makosh_communications_ingress::CommunicationEvidenceKindV1::TopicChanged => 10,
        makosh_communications_ingress::CommunicationEvidenceKindV1::TypingChanged => 11,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_whatsapp_api::host_bridge::{
        HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION, WhatsAppHostObservationV1,
    };

    #[test]
    fn message_identity_becomes_metadata_only_chat_evidence() {
        let draft = draft_host_observation(&WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: "wa-1".to_owned(),
            provider_event_id: "event-1".to_owned(),
            observed_at_unix_seconds: 1_782_504_000,
            observation: WhatsAppHostObservationV1::MessageIdentity {
                provider_chat_id: "chat-1".to_owned(),
                provider_message_id: "message-1".to_owned(),
                sender_id: "sender-1".to_owned(),
            },
        })
        .expect("draft");

        assert_eq!(
            draft.source.provider,
            makosh_communications_ingress::ProviderProvenanceV1::WhatsAppWeb
        );
        assert_eq!(
            draft.kind,
            makosh_communications_ingress::CommunicationEvidenceKindV1::ChatMessage
        );
        assert_eq!(
            draft.body,
            makosh_communications_ingress::BodyAvailabilityV1::MetadataOnly
        );
    }

    #[test]
    fn session_material_is_not_a_communications_observation() {
        let result = draft_host_observation(&WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: "wa-1".to_owned(),
            provider_event_id: "event-1".to_owned(),
            observed_at_unix_seconds: 1_782_504_000,
            observation: WhatsAppHostObservationV1::SessionLinked {
                secret_ref: "secret-ref".to_owned(),
                revision: 1,
            },
        });

        assert!(matches!(
            result,
            Err(WhatsAppHostIngressError::Core(
                WhatsAppCoreError::UnsupportedObservation
            ))
        ));
    }
}
