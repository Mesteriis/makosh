//! WhatsApp provider policy over the sanitized host bridge contract.
//!
//! This package has no WebView, socket, database, event broker or business
//! domain dependency. It preserves only the evidence category allowed by the
//! host bridge; provider payload and session material never enter this type.

mod operational;

pub use operational::{
    WhatsAppOperationalProjectionError, WhatsAppOperationalProjectionV1,
    project_operational_host_observation,
};

use makosh_communications_ingress::{
    AttachmentDescriptorV1, AttachmentDispositionV1, BodyAvailabilityV1, CommunicationDirectionV1,
    CommunicationEvidenceKindV1, CommunicationObservationDraft, IngressDraftError,
    ProviderProvenanceV1, SourceEnvelope, SourceScopeEnvelope,
    new_scoped_communication_observation_draft, with_attachment_descriptor,
};
use makosh_whatsapp_api::host_bridge::{
    WhatsAppHostBridgeEnvelopeV1, WhatsAppHostBridgeError, WhatsAppHostObservationV1,
    validate_host_bridge_envelope,
};

pub const PACKAGE: &str = "makosh-whatsapp-core";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppHostObservationProjection {
    pub account_id: String,
    pub provider_event_id: String,
    pub provider_record_id: String,
    pub provider_conversation_id: Option<String>,
    pub provider_participant_id: Option<String>,
    pub provider_media_id: Option<String>,
    pub attachment_descriptor: Option<AttachmentDescriptorV1>,
    pub observed_at_unix_seconds: i64,
    pub evidence_kind: CommunicationEvidenceKindV1,
}

#[derive(Debug)]
pub enum WhatsAppCoreError {
    HostBridge(WhatsAppHostBridgeError),
    UnsupportedObservation,
    Draft(IngressDraftError),
}

pub fn project_host_observation(
    envelope: &WhatsAppHostBridgeEnvelopeV1,
) -> Result<WhatsAppHostObservationProjection, WhatsAppCoreError> {
    validate_host_bridge_envelope(envelope).map_err(WhatsAppCoreError::HostBridge)?;
    let (
        evidence_kind,
        provider_conversation_id,
        provider_participant_id,
        provider_media_id,
        provider_record_id,
        attachment_descriptor,
    ) = match &envelope.observation {
        WhatsAppHostObservationV1::MessageIdentity {
            provider_chat_id,
            provider_message_id,
            sender_id,
        } => (
            CommunicationEvidenceKindV1::ChatMessage,
            Some(provider_chat_id.clone()),
            Some(sender_id.clone()),
            None,
            provider_message_id.clone(),
            None,
        ),
        WhatsAppHostObservationV1::OperationalMessage(value) => (
            CommunicationEvidenceKindV1::ChatMessage,
            Some(value.provider_chat_id.clone()),
            Some(value.sender_id.clone()),
            None,
            value.provider_message_id.clone(),
            None,
        ),
        WhatsAppHostObservationV1::MessageUpdated {
            provider_chat_id,
            provider_message_id,
        } => (
            CommunicationEvidenceKindV1::MessageEdited,
            Some(provider_chat_id.clone()),
            None,
            None,
            provider_message_id.clone(),
            None,
        ),
        WhatsAppHostObservationV1::MessageDeleted {
            provider_chat_id,
            provider_message_id,
        } => (
            CommunicationEvidenceKindV1::MessageDeleted,
            Some(provider_chat_id.clone()),
            None,
            None,
            provider_message_id.clone(),
            None,
        ),
        WhatsAppHostObservationV1::Receipt {
            provider_chat_id,
            provider_message_id,
            ..
        } => (
            CommunicationEvidenceKindV1::DeliveryStateChanged,
            Some(provider_chat_id.clone()),
            None,
            None,
            provider_message_id.clone(),
            None,
        ),
        WhatsAppHostObservationV1::Reaction {
            provider_chat_id,
            provider_message_id,
            actor_id,
            ..
        } => (
            CommunicationEvidenceKindV1::ReactionChanged,
            Some(provider_chat_id.clone()),
            Some(actor_id.clone()),
            None,
            provider_message_id.clone(),
            None,
        ),
        WhatsAppHostObservationV1::Participant {
            provider_chat_id,
            provider_identity_id,
            ..
        } => (
            CommunicationEvidenceKindV1::ParticipantChanged,
            Some(provider_chat_id.clone()),
            Some(provider_identity_id.clone()),
            None,
            envelope.provider_event_id.clone(),
            None,
        ),
        WhatsAppHostObservationV1::OperationalParticipant(value) => (
            CommunicationEvidenceKindV1::ParticipantChanged,
            Some(value.provider_chat_id.clone()),
            Some(value.provider_identity_id.clone()),
            None,
            envelope.provider_event_id.clone(),
            None,
        ),
        WhatsAppHostObservationV1::OperationalParticipantRemoved {
            provider_chat_id,
            provider_identity_id,
        } => (
            CommunicationEvidenceKindV1::ParticipantChanged,
            Some(provider_chat_id.clone()),
            Some(provider_identity_id.clone()),
            None,
            envelope.provider_event_id.clone(),
            None,
        ),
        WhatsAppHostObservationV1::MediaMetadata {
            provider_chat_id,
            provider_message_id,
            provider_media_id,
            filename,
            content_type,
            declared_size,
            ..
        } => {
            let descriptor = match (content_type, declared_size) {
                (Some(media_type), Some(declared_bytes)) => Some(AttachmentDescriptorV1 {
                    filename: filename.clone(),
                    media_type: media_type.clone(),
                    declared_bytes: *declared_bytes,
                    sha256: None,
                    disposition: AttachmentDispositionV1::Attachment,
                }),
                _ => None,
            };
            (
                CommunicationEvidenceKindV1::MediaChanged,
                Some(provider_chat_id.clone()),
                None,
                Some(provider_media_id.clone()),
                provider_message_id.clone(),
                descriptor,
            )
        }
        WhatsAppHostObservationV1::Dialog { .. }
        | WhatsAppHostObservationV1::OperationalDialog(_)
        | WhatsAppHostObservationV1::RuntimeState { .. }
        | WhatsAppHostObservationV1::Presence { .. }
        | WhatsAppHostObservationV1::CallMetadata { .. }
        | WhatsAppHostObservationV1::StatusMetadata { .. }
        | WhatsAppHostObservationV1::StatusViewMetadata { .. }
        | WhatsAppHostObservationV1::StatusDeletedMetadata { .. } => (
            CommunicationEvidenceKindV1::ConversationStateChanged,
            None,
            None,
            None,
            envelope.provider_event_id.clone(),
            None,
        ),
        WhatsAppHostObservationV1::SessionLinked { .. }
        | WhatsAppHostObservationV1::SessionRevoked
        | WhatsAppHostObservationV1::CommandResult { .. }
        | WhatsAppHostObservationV1::OperationalResyncState { .. } => {
            return Err(WhatsAppCoreError::UnsupportedObservation);
        }
    };
    Ok(WhatsAppHostObservationProjection {
        account_id: envelope.account_id.clone(),
        provider_event_id: envelope.provider_event_id.clone(),
        provider_record_id,
        provider_conversation_id,
        provider_participant_id,
        provider_media_id,
        attachment_descriptor,
        observed_at_unix_seconds: envelope.observed_at_unix_seconds,
        evidence_kind,
    })
}

pub fn communication_observation_draft(
    projection: &WhatsAppHostObservationProjection,
) -> Result<CommunicationObservationDraft, WhatsAppCoreError> {
    let draft = new_scoped_communication_observation_draft(
        format!(
            "whatsapp:{}:{}",
            projection.account_id, projection.provider_event_id
        ),
        SourceEnvelope {
            provider: ProviderProvenanceV1::WhatsAppWeb,
            external_record_id: projection.provider_record_id.clone(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: projection.account_id.clone(),
                external_conversation_id: projection.provider_conversation_id.clone(),
                external_participant_id: projection.provider_participant_id.clone(),
                external_media_id: projection.provider_media_id.clone(),
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        projection.evidence_kind,
        BodyAvailabilityV1::MetadataOnly,
        CommunicationDirectionV1::Unknown,
        Some(projection.observed_at_unix_seconds),
    )
    .map_err(WhatsAppCoreError::Draft)?;
    match &projection.attachment_descriptor {
        Some(descriptor) => with_attachment_descriptor(draft, descriptor.clone()),
        None => Ok(draft),
    }
    .map_err(WhatsAppCoreError::Draft)
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_whatsapp_api::host_bridge::{
        HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION,
    };

    #[test]
    fn message_identity_projects_without_body_content() {
        let projection = project_host_observation(&WhatsAppHostBridgeEnvelopeV1 {
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
        .expect("projection");

        assert_eq!(
            projection.evidence_kind,
            CommunicationEvidenceKindV1::ChatMessage
        );
        assert_eq!(
            communication_observation_draft(&projection)
                .expect("draft")
                .body,
            BodyAvailabilityV1::MetadataOnly
        );
    }

    #[test]
    fn provider_command_result_is_not_communications_evidence() {
        let result = project_host_observation(&WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: "wa-1".to_owned(),
            provider_event_id: "command-result-1".to_owned(),
            observed_at_unix_seconds: 1_782_504_001,
            observation: WhatsAppHostObservationV1::CommandResult {
                operation_id: "operation-1".to_owned(),
                provider_request_id: Some("provider-request-1".to_owned()),
                succeeded: true,
                host_claim_id: "host-claim-1".to_owned(),
            },
        });

        assert!(matches!(
            result,
            Err(WhatsAppCoreError::UnsupportedObservation)
        ));
    }
}
