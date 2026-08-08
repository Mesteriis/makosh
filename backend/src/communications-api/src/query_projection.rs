//! Typed conversion from owner query values to the public Protobuf contract.

use crate::{
    AttachmentDispositionV1, AttachmentSafetyStateV1, CommunicationAccountSummaryV1,
    CommunicationAttachmentAnchorSummaryV1, CommunicationBodyStateV1,
    CommunicationConversationIdV1, CommunicationConversationSummaryV1, CommunicationDirectionV1,
    CommunicationMessageIdV1, CommunicationMessageLifecycleStateV1,
    CommunicationMessageReferenceKindV1, CommunicationMessageReferenceSummaryV1,
    CommunicationMessageSummaryV1, CommunicationObservationIdV1,
    CommunicationObservedParticipantSummaryV1, CommunicationProviderProvenanceV1,
    CommunicationSourceCursorV1, query_wire,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsQueryProjectionErrorV1 {
    InvalidIdentifier,
    InvalidProvider,
    InvalidBodyState,
    InvalidDirection,
    InvalidLifecycleState,
}

impl From<&crate::CommunicationSummary> for query_wire::EvidenceSummaryV1 {
    fn from(value: &crate::CommunicationSummary) -> Self {
        Self {
            evidence_id: value.evidence_id.bytes().to_vec(),
            provider: provider_value(value.provider),
            direction: direction_value(value.direction),
            kind: evidence_kind_value(value.kind),
            body_state: body_state_value(value.body),
            body_admission_failure: body_admission_failure_value(value.body_admission_failure),
            observed_at_unix_seconds: value.observed_at_unix_seconds,
            causation_message_id: value
                .causation_message_id
                .map_or_else(Vec::new, |id| id.bytes().to_vec()),
            correlation_id: value.correlation_id.bytes().to_vec(),
            recorded_at_unix_seconds: value.recorded_at_unix_seconds,
            recorded_at_nanos: value.recorded_at_nanos,
        }
    }
}

impl From<&CommunicationAccountSummaryV1> for query_wire::AccountSummaryV1 {
    fn from(value: &CommunicationAccountSummaryV1) -> Self {
        Self {
            account_id: value.account_id.bytes().to_vec(),
            account_cursor_sha256: value.account_cursor.bytes().to_vec(),
            provider: provider_value(value.provider),
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: value.last_evidence_id.bytes().to_vec(),
        }
    }
}

impl From<&CommunicationConversationSummaryV1> for query_wire::ConversationSummaryV1 {
    fn from(value: &CommunicationConversationSummaryV1) -> Self {
        Self {
            conversation_id: value.conversation_id.bytes().to_vec(),
            account_cursor_sha256: value.account_cursor.bytes().to_vec(),
            conversation_cursor_sha256: value.conversation_cursor.bytes().to_vec(),
            provider: provider_value(value.provider),
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: value.last_evidence_id.bytes().to_vec(),
        }
    }
}

impl From<&CommunicationMessageSummaryV1> for query_wire::MessageSummaryV1 {
    fn from(value: &CommunicationMessageSummaryV1) -> Self {
        Self {
            message_id: value.message_id.bytes().to_vec(),
            conversation_id: value.conversation_id.bytes().to_vec(),
            source_cursor_sha256: value.source_cursor.bytes().to_vec(),
            body_state: body_state_value(value.body),
            direction: direction_value(value.direction),
            lifecycle_state: lifecycle_state_value(value.lifecycle_state),
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: value.last_evidence_id.bytes().to_vec(),
        }
    }
}

impl TryFrom<query_wire::ConversationSummaryV1> for CommunicationConversationSummaryV1 {
    type Error = CommunicationsQueryProjectionErrorV1;

    fn try_from(value: query_wire::ConversationSummaryV1) -> Result<Self, Self::Error> {
        Ok(Self {
            conversation_id: CommunicationConversationIdV1::new(id16(&value.conversation_id)?),
            account_cursor: CommunicationSourceCursorV1::new(id32(&value.account_cursor_sha256)?),
            conversation_cursor: CommunicationSourceCursorV1::new(id32(
                &value.conversation_cursor_sha256,
            )?),
            provider: provider_from_value(value.provider)?,
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: CommunicationObservationIdV1::new(id16(&value.last_evidence_id)?),
        })
    }
}

impl TryFrom<query_wire::MessageSummaryV1> for CommunicationMessageSummaryV1 {
    type Error = CommunicationsQueryProjectionErrorV1;

    fn try_from(value: query_wire::MessageSummaryV1) -> Result<Self, Self::Error> {
        Ok(Self {
            message_id: CommunicationMessageIdV1::new(id16(&value.message_id)?),
            conversation_id: CommunicationConversationIdV1::new(id16(&value.conversation_id)?),
            source_cursor: CommunicationSourceCursorV1::new(id32(&value.source_cursor_sha256)?),
            body: body_state_from_value(value.body_state)?,
            direction: direction_from_value(value.direction)?,
            lifecycle_state: lifecycle_state_from_value(value.lifecycle_state)?,
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: CommunicationObservationIdV1::new(id16(&value.last_evidence_id)?),
        })
    }
}

impl From<&CommunicationObservedParticipantSummaryV1> for query_wire::ObservedParticipantSummaryV1 {
    fn from(value: &CommunicationObservedParticipantSummaryV1) -> Self {
        Self {
            participant_id: value.participant_id.bytes().to_vec(),
            conversation_id: value.conversation_id.bytes().to_vec(),
            participant_cursor_sha256: value.participant_cursor.bytes().to_vec(),
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: value.last_evidence_id.bytes().to_vec(),
            display_label: value.display_label.clone(),
        }
    }
}

impl From<&CommunicationAttachmentAnchorSummaryV1> for query_wire::AttachmentAnchorSummaryV1 {
    fn from(value: &CommunicationAttachmentAnchorSummaryV1) -> Self {
        Self {
            attachment_anchor_id: value.attachment_anchor_id.bytes().to_vec(),
            message_id: value.message_id.bytes().to_vec(),
            media_cursor_sha256: value.media_cursor.bytes().to_vec(),
            state: attachment_state_value(value.state),
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: value.last_evidence_id.bytes().to_vec(),
            has_descriptor: value.descriptor.is_some(),
            filename: value
                .descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.filename())
                .unwrap_or_default()
                .to_owned(),
            has_filename: value
                .descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.filename())
                .is_some(),
            media_type: value
                .descriptor
                .as_ref()
                .map(|descriptor| descriptor.media_type())
                .unwrap_or_default()
                .to_owned(),
            declared_bytes: value
                .descriptor
                .as_ref()
                .map_or(0, |descriptor| descriptor.declared_bytes()),
            sha256: value
                .descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.sha256())
                .map_or_else(Vec::new, |value| value.to_vec()),
            disposition: value.descriptor.as_ref().map_or(0, |descriptor| {
                match descriptor.disposition() {
                    AttachmentDispositionV1::Attachment => 1,
                    AttachmentDispositionV1::Inline => 2,
                    AttachmentDispositionV1::Unknown => 3,
                }
            }),
        }
    }
}

impl From<&CommunicationMessageReferenceSummaryV1> for query_wire::MessageReferenceSummaryV1 {
    fn from(value: &CommunicationMessageReferenceSummaryV1) -> Self {
        Self {
            source_message_id: value.source_message_id.bytes().to_vec(),
            kind: reference_kind_value(value.kind),
            target_source_cursor_sha256: value.target_source_cursor.bytes().to_vec(),
            target_message_id: value
                .target_message_id
                .map_or_else(Vec::new, |id| id.bytes().to_vec()),
            observed_at_unix_seconds: value.observed_at_unix_seconds,
            evidence_id: value.evidence_id.bytes().to_vec(),
        }
    }
}

const fn provider_value(value: CommunicationProviderProvenanceV1) -> u32 {
    match value {
        CommunicationProviderProvenanceV1::MailImap => 1,
        CommunicationProviderProvenanceV1::Telegram => 2,
        CommunicationProviderProvenanceV1::WhatsAppWeb => 3,
        CommunicationProviderProvenanceV1::MailSmtp => 4,
        CommunicationProviderProvenanceV1::Zulip => 5,
        CommunicationProviderProvenanceV1::MailGmail => 6,
    }
}

const fn provider_from_value(
    value: u32,
) -> Result<CommunicationProviderProvenanceV1, CommunicationsQueryProjectionErrorV1> {
    match value {
        1 => Ok(CommunicationProviderProvenanceV1::MailImap),
        2 => Ok(CommunicationProviderProvenanceV1::Telegram),
        3 => Ok(CommunicationProviderProvenanceV1::WhatsAppWeb),
        4 => Ok(CommunicationProviderProvenanceV1::MailSmtp),
        5 => Ok(CommunicationProviderProvenanceV1::Zulip),
        6 => Ok(CommunicationProviderProvenanceV1::MailGmail),
        _ => Err(CommunicationsQueryProjectionErrorV1::InvalidProvider),
    }
}

const fn direction_value(value: CommunicationDirectionV1) -> u32 {
    match value {
        CommunicationDirectionV1::Incoming => 1,
        CommunicationDirectionV1::Outgoing => 2,
        CommunicationDirectionV1::Unknown => 3,
    }
}

const fn direction_from_value(
    value: u32,
) -> Result<CommunicationDirectionV1, CommunicationsQueryProjectionErrorV1> {
    match value {
        1 => Ok(CommunicationDirectionV1::Incoming),
        2 => Ok(CommunicationDirectionV1::Outgoing),
        3 => Ok(CommunicationDirectionV1::Unknown),
        _ => Err(CommunicationsQueryProjectionErrorV1::InvalidDirection),
    }
}

const fn body_state_value(value: crate::CommunicationBodyStateV1) -> u32 {
    match value {
        crate::CommunicationBodyStateV1::MetadataOnly => 1,
        crate::CommunicationBodyStateV1::PendingBlob => 2,
        crate::CommunicationBodyStateV1::Unavailable => 3,
        crate::CommunicationBodyStateV1::AdmittedBlob => 4,
    }
}

const fn body_state_from_value(
    value: u32,
) -> Result<CommunicationBodyStateV1, CommunicationsQueryProjectionErrorV1> {
    match value {
        1 => Ok(CommunicationBodyStateV1::MetadataOnly),
        2 => Ok(CommunicationBodyStateV1::PendingBlob),
        3 => Ok(CommunicationBodyStateV1::Unavailable),
        4 => Ok(CommunicationBodyStateV1::AdmittedBlob),
        _ => Err(CommunicationsQueryProjectionErrorV1::InvalidBodyState),
    }
}

const fn body_admission_failure_value(
    value: Option<crate::CommunicationBodyAdmissionFailureV1>,
) -> u32 {
    match value {
        None => 0,
        Some(crate::CommunicationBodyAdmissionFailureV1::SourceUnavailable) => 1,
        Some(crate::CommunicationBodyAdmissionFailureV1::SizeLimitExceeded) => 2,
        Some(crate::CommunicationBodyAdmissionFailureV1::IntegrityMismatch) => 3,
        Some(crate::CommunicationBodyAdmissionFailureV1::PolicyRejected) => 4,
    }
}

const fn evidence_kind_value(value: crate::CanonicalCommunicationEvidenceKindV1) -> u32 {
    match value {
        crate::CanonicalCommunicationEvidenceKindV1::EmailMessage => 1,
        crate::CanonicalCommunicationEvidenceKindV1::ChatMessage => 2,
        crate::CanonicalCommunicationEvidenceKindV1::MessageEdited => 3,
        crate::CanonicalCommunicationEvidenceKindV1::MessageDeleted => 4,
        crate::CanonicalCommunicationEvidenceKindV1::ReactionChanged => 5,
        crate::CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged => 6,
        crate::CanonicalCommunicationEvidenceKindV1::ConversationStateChanged => 7,
        crate::CanonicalCommunicationEvidenceKindV1::ParticipantChanged => 8,
        crate::CanonicalCommunicationEvidenceKindV1::MediaChanged => 9,
        crate::CanonicalCommunicationEvidenceKindV1::TopicChanged => 10,
        crate::CanonicalCommunicationEvidenceKindV1::TypingChanged => 11,
    }
}

const fn lifecycle_state_value(value: CommunicationMessageLifecycleStateV1) -> u32 {
    match value {
        CommunicationMessageLifecycleStateV1::Active => 1,
        CommunicationMessageLifecycleStateV1::Deleted => 2,
    }
}

const fn lifecycle_state_from_value(
    value: u32,
) -> Result<CommunicationMessageLifecycleStateV1, CommunicationsQueryProjectionErrorV1> {
    match value {
        1 => Ok(CommunicationMessageLifecycleStateV1::Active),
        2 => Ok(CommunicationMessageLifecycleStateV1::Deleted),
        _ => Err(CommunicationsQueryProjectionErrorV1::InvalidLifecycleState),
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsQueryProjectionErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsQueryProjectionErrorV1::InvalidIdentifier)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsQueryProjectionErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsQueryProjectionErrorV1::InvalidIdentifier)
}

const fn reference_kind_value(value: CommunicationMessageReferenceKindV1) -> u32 {
    match value {
        CommunicationMessageReferenceKindV1::Reply => 1,
        CommunicationMessageReferenceKindV1::Forward => 2,
    }
}

const fn attachment_state_value(value: AttachmentSafetyStateV1) -> u32 {
    match value {
        AttachmentSafetyStateV1::DescriptorOnly => 1,
        AttachmentSafetyStateV1::BlobPending => 2,
        AttachmentSafetyStateV1::BlobAdmitted => 3,
        AttachmentSafetyStateV1::Quarantined => 4,
        AttachmentSafetyStateV1::SafeForDelivery => 5,
        AttachmentSafetyStateV1::Rejected => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CanonicalCommunicationEvidenceKindV1, CommunicationBodyAdmissionFailureV1,
        CommunicationBodyBlobReferenceV1, CommunicationBodyStateV1, CommunicationDirectionV1,
        CommunicationObservationIdV1, CommunicationProviderProvenanceV1,
        CommunicationSourceCursorV1, CommunicationSummary,
    };

    #[test]
    fn evidence_summary_exposes_only_canonical_metadata() {
        let summary = CommunicationSummary {
            evidence_id: CommunicationObservationIdV1::new([1; 16]),
            observation_id: CommunicationObservationIdV1::new([2; 16]),
            causation_message_id: Some(CommunicationObservationIdV1::new([3; 16])),
            correlation_id: CommunicationObservationIdV1::new([4; 16]),
            source_cursor: CommunicationSourceCursorV1::new([3; 32]),
            account_cursor: Some(CommunicationSourceCursorV1::new([4; 32])),
            conversation_cursor: Some(CommunicationSourceCursorV1::new([5; 32])),
            participant_cursor: None,
            participant_display_label: None,
            message_subject: None,
            media_cursor: None,
            reply_to_source_cursor: None,
            forward_origin_source_cursor: None,
            provider: CommunicationProviderProvenanceV1::MailImap,
            direction: CommunicationDirectionV1::Incoming,
            kind: CanonicalCommunicationEvidenceKindV1::EmailMessage,
            body: CommunicationBodyStateV1::AdmittedBlob,
            body_blob: Some(CommunicationBodyBlobReferenceV1 {
                blob_ref: "private-blob-locator".to_owned(),
                reference_id: [6; 16],
                declared_bytes: 64,
                sha256: [7; 32],
            }),
            body_media_type: Some("text/plain".to_owned()),
            body_admission_failure: Some(CommunicationBodyAdmissionFailureV1::PolicyRejected),
            attachment_descriptor: None,
            observed_at_unix_seconds: 8,
            recorded_at_unix_seconds: 9,
            recorded_at_nanos: 10,
        };

        let wire: query_wire::EvidenceSummaryV1 = (&summary).into();

        assert_eq!(wire.evidence_id, vec![1; 16]);
        assert_eq!(wire.provider, 1);
        assert_eq!(wire.direction, 1);
        assert_eq!(wire.kind, 1);
        assert_eq!(wire.body_state, 4);
        assert_eq!(wire.body_admission_failure, 4);
        assert_eq!(wire.observed_at_unix_seconds, 8);
        assert_eq!(wire.causation_message_id, vec![3; 16]);
        assert_eq!(wire.correlation_id, vec![4; 16]);
        assert_eq!(wire.recorded_at_unix_seconds, 9);
        assert_eq!(wire.recorded_at_nanos, 10);
    }

    #[test]
    fn public_query_summaries_round_trip_through_strict_domain_values() {
        let conversation = CommunicationConversationSummaryV1 {
            conversation_id: CommunicationConversationIdV1::new([1; 16]),
            account_cursor: CommunicationSourceCursorV1::new([2; 32]),
            conversation_cursor: CommunicationSourceCursorV1::new([3; 32]),
            provider: CommunicationProviderProvenanceV1::Telegram,
            first_observed_at_unix_seconds: 10,
            last_observed_at_unix_seconds: 20,
            last_evidence_id: CommunicationObservationIdV1::new([4; 16]),
        };
        let wire = query_wire::ConversationSummaryV1::from(&conversation);
        assert_eq!(
            CommunicationConversationSummaryV1::try_from(wire),
            Ok(conversation)
        );

        let message = CommunicationMessageSummaryV1 {
            message_id: CommunicationMessageIdV1::new([5; 16]),
            conversation_id: CommunicationConversationIdV1::new([1; 16]),
            source_cursor: CommunicationSourceCursorV1::new([6; 32]),
            body: CommunicationBodyStateV1::AdmittedBlob,
            direction: CommunicationDirectionV1::Outgoing,
            lifecycle_state: CommunicationMessageLifecycleStateV1::Active,
            first_observed_at_unix_seconds: 11,
            last_observed_at_unix_seconds: 21,
            last_evidence_id: CommunicationObservationIdV1::new([7; 16]),
        };
        let wire = query_wire::MessageSummaryV1::from(&message);
        assert_eq!(CommunicationMessageSummaryV1::try_from(wire), Ok(message));
    }

    #[test]
    fn public_query_summary_decode_rejects_unknown_values_and_wrong_ids() {
        let mut conversation = query_wire::ConversationSummaryV1 {
            conversation_id: vec![1; 16],
            account_cursor_sha256: vec![2; 32],
            conversation_cursor_sha256: vec![3; 32],
            provider: 99,
            first_observed_at_unix_seconds: 10,
            last_observed_at_unix_seconds: 20,
            last_evidence_id: vec![4; 16],
        };
        assert_eq!(
            CommunicationConversationSummaryV1::try_from(conversation.clone()),
            Err(CommunicationsQueryProjectionErrorV1::InvalidProvider)
        );
        conversation.provider = 2;
        conversation.conversation_id.clear();
        assert_eq!(
            CommunicationConversationSummaryV1::try_from(conversation),
            Err(CommunicationsQueryProjectionErrorV1::InvalidIdentifier)
        );
    }
}
