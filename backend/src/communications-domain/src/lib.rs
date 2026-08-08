//! Canonical evidence decisions owned exclusively by Communications.

use hermes_communications_api::PACKAGE as API_PACKAGE;
use hermes_communications_api::{
    AttachmentSafetyTransitionCommandV1, AttachmentSafetyTransitionDecisionV1,
    CanonicalAccountProjectionV1, CanonicalAttachmentAnchorProjectionV1,
    CanonicalCommunicationEvidenceKindV1, CanonicalCommunicationProjectionV1,
    CanonicalConversationProjectionV1, CanonicalMessageMutationV1, CanonicalMessageProjectionV1,
    CanonicalMessageReferenceProjectionV1, CanonicalObservedParticipantProjectionV1,
    CommunicationAccountIdV1, CommunicationAttachmentAnchorIdV1, CommunicationConversationIdV1,
    CommunicationDirectionV1, CommunicationMessageIdV1, CommunicationMessageReferenceKindV1,
    CommunicationObservationIdV1, CommunicationParticipantIdV1, CommunicationSenderIdV1,
    CommunicationSummary, CommunicationsClientError, RecordCommunicationEvidenceV1,
};
use sha2::{Digest, Sha256};

mod saved_search;
mod search;
pub use saved_search::{
    COMMUNICATIONS_SAVED_SEARCH_MAX_DESCRIPTION_BYTES_V1,
    COMMUNICATIONS_SAVED_SEARCH_MAX_NAME_BYTES_V1, CommunicationsSavedSearchDraftErrorV1,
    CommunicationsSavedSearchDraftV1, validate_saved_search_draft_v1,
};
pub use search::{
    COMMUNICATIONS_SEARCH_MAX_DOCUMENT_BYTES_V1, COMMUNICATIONS_SEARCH_MAX_DOCUMENT_TOKENS_V1,
    COMMUNICATIONS_SEARCH_MAX_QUERY_BYTES_V1, COMMUNICATIONS_SEARCH_MAX_QUERY_TOKENS_V1,
    COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1, CommunicationsSearchDocumentV1,
    CommunicationsSearchIndexDecisionV1, CommunicationsSearchIndexJobV1,
    CommunicationsSearchIndexRejectionV1, CommunicationsSearchQueryV1,
    CommunicationsSearchTokenErrorV1, decide_search_index_v1, normalize_search_document_tokens_v1,
    normalize_search_query_v1,
};

pub const PACKAGE: &str = "hermes-communications-domain";
pub fn dependency() -> &'static str {
    API_PACKAGE
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCommunication {
    pub evidence_id: CommunicationObservationIdV1,
    pub summary: CommunicationSummary,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDomainError {
    InvalidObservedTime,
    InvalidRecordedTime,
    MissingMessageScope,
    InvalidAttachmentScope,
    InvalidParticipantMetadata,
    InvalidMessageSubject,
    InvalidAttachmentSafetyTransition,
}

pub fn accept_command(
    command: RecordCommunicationEvidenceV1,
) -> Result<CommunicationSummary, CommunicationsDomainError> {
    validate_body_admission(&command)?;
    if command.attachment_descriptor.is_some()
        && (command.kind != CanonicalCommunicationEvidenceKindV1::MediaChanged
            || command.media_cursor.is_none())
    {
        return Err(CommunicationsDomainError::InvalidAttachmentScope);
    }
    if command
        .participant_display_label
        .as_ref()
        .is_some_and(|label| {
            command.participant_cursor.is_none()
                || label.is_empty()
                || label.len() > 256
                || label.chars().any(char::is_control)
        })
    {
        return Err(CommunicationsDomainError::InvalidParticipantMetadata);
    }
    if command.message_subject.as_ref().is_some_and(|subject| {
        !requires_message_scope(command.kind)
            || subject.is_empty()
            || subject.len() > 998
            || subject.trim() != subject
            || subject.chars().any(char::is_control)
    }) {
        return Err(CommunicationsDomainError::InvalidMessageSubject);
    }
    if requires_message_scope(command.kind)
        && (command.account_cursor.is_none() || command.conversation_cursor.is_none())
    {
        return Err(CommunicationsDomainError::MissingMessageScope);
    }
    (valid_timestamp(command.observed_at_unix_seconds, 0)
        && valid_timestamp(command.recorded_at_unix_seconds, command.recorded_at_nanos))
    .then_some(CommunicationSummary {
        evidence_id: command.observation_id,
        observation_id: command.observation_id,
        causation_message_id: command.causation_message_id,
        correlation_id: command.correlation_id,
        source_cursor: command.source_cursor,
        provider: command.provider,
        direction: command.direction,
        kind: command.kind,
        account_cursor: command.account_cursor,
        conversation_cursor: command.conversation_cursor,
        participant_cursor: command.participant_cursor,
        participant_display_label: command.participant_display_label,
        message_subject: command.message_subject,
        media_cursor: command.media_cursor,
        reply_to_source_cursor: command.reply_to_source_cursor,
        forward_origin_source_cursor: command.forward_origin_source_cursor,
        body: command.body,
        body_blob: command.body_blob,
        body_media_type: command.body_media_type,
        body_admission_failure: command.body_admission_failure,
        attachment_descriptor: command.attachment_descriptor,
        observed_at_unix_seconds: command.observed_at_unix_seconds,
        recorded_at_unix_seconds: command.recorded_at_unix_seconds,
        recorded_at_nanos: command.recorded_at_nanos,
    })
    .ok_or_else(|| {
        if valid_timestamp(command.observed_at_unix_seconds, 0) {
            CommunicationsDomainError::InvalidRecordedTime
        } else {
            CommunicationsDomainError::InvalidObservedTime
        }
    })
}

const fn requires_message_scope(kind: CanonicalCommunicationEvidenceKindV1) -> bool {
    matches!(
        kind,
        CanonicalCommunicationEvidenceKindV1::EmailMessage
            | CanonicalCommunicationEvidenceKindV1::ChatMessage
            | CanonicalCommunicationEvidenceKindV1::MessageEdited
            | CanonicalCommunicationEvidenceKindV1::MessageDeleted
            | CanonicalCommunicationEvidenceKindV1::ReactionChanged
            | CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged
            | CanonicalCommunicationEvidenceKindV1::MediaChanged
    )
}

const fn valid_timestamp(seconds: i64, nanos: i32) -> bool {
    seconds >= -62_135_596_800 && seconds <= 253_402_300_799 && nanos >= 0 && nanos < 1_000_000_000
}

fn validate_body_admission(
    command: &RecordCommunicationEvidenceV1,
) -> Result<(), CommunicationsDomainError> {
    let media_type_is_valid = command
        .body_media_type
        .as_deref()
        .is_some_and(|value| matches!(value, "text/plain" | "text/html"));
    if matches!(
        command.body,
        hermes_communications_api::CommunicationBodyStateV1::PendingBlob
            | hermes_communications_api::CommunicationBodyStateV1::AdmittedBlob
    ) != media_type_is_valid
    {
        return Err(CommunicationsDomainError::InvalidAttachmentScope);
    }
    if command.body == hermes_communications_api::CommunicationBodyStateV1::AdmittedBlob {
        let Some(receipt) = command.body_blob.as_ref() else {
            return Err(CommunicationsDomainError::InvalidAttachmentScope);
        };
        if receipt.blob_ref.trim().is_empty()
            || receipt.blob_ref.len() > 512
            || !receipt.blob_ref.is_ascii()
            || receipt.reference_id.iter().all(|byte| *byte == 0)
            || !(1..=64 * 1024 * 1024).contains(&receipt.declared_bytes)
            || command.body_admission_failure.is_some()
        {
            return Err(CommunicationsDomainError::InvalidAttachmentScope);
        }
    } else if command.body_blob.is_some() {
        return Err(CommunicationsDomainError::InvalidAttachmentScope);
    }
    Ok(())
}

pub fn decide_attachment_safety_transition(
    command: AttachmentSafetyTransitionCommandV1,
) -> Result<AttachmentSafetyTransitionDecisionV1, CommunicationsDomainError> {
    if !(-62_135_596_800..=253_402_300_799).contains(&command.observed_at_unix_seconds) {
        return Err(CommunicationsDomainError::InvalidObservedTime);
    }
    let next_state = command
        .current_state
        .transition(command.transition)
        .map_err(|_| CommunicationsDomainError::InvalidAttachmentSafetyTransition)?;
    Ok(AttachmentSafetyTransitionDecisionV1 {
        attachment_anchor_id: command.attachment_anchor_id,
        expected_state: command.current_state,
        next_state,
        evidence_id: command.evidence_id,
        observed_at_unix_seconds: command.observed_at_unix_seconds,
    })
}

pub fn canonicalize_communication(
    summary: &CommunicationSummary,
) -> Result<CanonicalCommunicationProjectionV1, CommunicationsDomainError> {
    let account = summary
        .account_cursor
        .map(|account_cursor| CanonicalAccountProjectionV1 {
            account_id: CommunicationAccountIdV1::new(identifier(
                b"hermes.communications.account.v1\0",
                &[&account_cursor.bytes()],
            )),
            account_cursor,
            provider: summary.provider,
            observed_at_unix_seconds: summary.observed_at_unix_seconds,
        });
    let conversation = match (summary.account_cursor, summary.conversation_cursor) {
        (Some(account_cursor), Some(conversation_cursor)) => {
            Some(CanonicalConversationProjectionV1 {
                conversation_id: CommunicationConversationIdV1::new(identifier(
                    b"hermes.communications.conversation.v1\0",
                    &[&account_cursor.bytes(), &conversation_cursor.bytes()],
                )),
                account_cursor,
                conversation_cursor,
                provider: summary.provider,
                observed_at_unix_seconds: summary.observed_at_unix_seconds,
            })
        }
        _ => None,
    };
    let message = match summary.kind {
        CanonicalCommunicationEvidenceKindV1::EmailMessage
        | CanonicalCommunicationEvidenceKindV1::ChatMessage => {
            let conversation = conversation
                .as_ref()
                .ok_or(CommunicationsDomainError::MissingMessageScope)?;
            Some(CanonicalMessageProjectionV1 {
                message_id: CommunicationMessageIdV1::new(identifier(
                    b"hermes.communications.message.v1\0",
                    &[&summary.source_cursor.bytes()],
                )),
                conversation_id: conversation.conversation_id,
                source_cursor: summary.source_cursor,
                body: summary.body,
                direction: summary.direction,
                observed_at_unix_seconds: summary.observed_at_unix_seconds,
                mutation: CanonicalMessageMutationV1::Create,
            })
        }
        CanonicalCommunicationEvidenceKindV1::MessageEdited
        | CanonicalCommunicationEvidenceKindV1::ReactionChanged
        | CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged
        | CanonicalCommunicationEvidenceKindV1::MediaChanged => {
            conversation
                .as_ref()
                .map(|conversation| CanonicalMessageProjectionV1 {
                    message_id: CommunicationMessageIdV1::new(identifier(
                        b"hermes.communications.message.v1\0",
                        &[&summary.source_cursor.bytes()],
                    )),
                    conversation_id: conversation.conversation_id,
                    source_cursor: summary.source_cursor,
                    body: summary.body,
                    direction: summary.direction,
                    observed_at_unix_seconds: summary.observed_at_unix_seconds,
                    mutation: CanonicalMessageMutationV1::Update,
                })
        }
        CanonicalCommunicationEvidenceKindV1::MessageDeleted => {
            conversation
                .as_ref()
                .map(|conversation| CanonicalMessageProjectionV1 {
                    message_id: CommunicationMessageIdV1::new(identifier(
                        b"hermes.communications.message.v1\0",
                        &[&summary.source_cursor.bytes()],
                    )),
                    conversation_id: conversation.conversation_id,
                    source_cursor: summary.source_cursor,
                    body: summary.body,
                    direction: summary.direction,
                    observed_at_unix_seconds: summary.observed_at_unix_seconds,
                    mutation: CanonicalMessageMutationV1::Delete,
                })
        }
        CanonicalCommunicationEvidenceKindV1::ConversationStateChanged
        | CanonicalCommunicationEvidenceKindV1::ParticipantChanged
        | CanonicalCommunicationEvidenceKindV1::TopicChanged
        | CanonicalCommunicationEvidenceKindV1::TypingChanged => None,
    };
    let participant = match (&conversation, summary.participant_cursor) {
        (Some(conversation), Some(participant_cursor)) => {
            let sender_id = message
                .as_ref()
                .filter(|message| message.direction == CommunicationDirectionV1::Incoming)
                .map(|_| {
                    CommunicationSenderIdV1::new(identifier(
                        b"hermes.communications.sender.v1\0",
                        &[&participant_cursor.bytes()],
                    ))
                });
            Some(CanonicalObservedParticipantProjectionV1 {
                participant_id: CommunicationParticipantIdV1::new(identifier(
                    b"hermes.communications.participant.v1\0",
                    &[
                        &conversation.conversation_id.bytes(),
                        &participant_cursor.bytes(),
                    ],
                )),
                sender_id,
                conversation_id: conversation.conversation_id,
                participant_cursor,
                display_label: summary.participant_display_label.clone(),
                observed_at_unix_seconds: summary.observed_at_unix_seconds,
            })
        }
        _ => None,
    };
    let attachment_anchor = match (&message, summary.media_cursor) {
        (Some(message), Some(media_cursor)) => Some(CanonicalAttachmentAnchorProjectionV1 {
            attachment_anchor_id: CommunicationAttachmentAnchorIdV1::new(identifier(
                b"hermes.communications.attachment-anchor.v1\0",
                &[&message.message_id.bytes(), &media_cursor.bytes()],
            )),
            message_id: message.message_id,
            media_cursor,
            descriptor: summary.attachment_descriptor.clone(),
            observed_at_unix_seconds: summary.observed_at_unix_seconds,
        }),
        _ => None,
    };
    let message_references = if let Some(message) = message.as_ref()
        && message.mutation == CanonicalMessageMutationV1::Create
    {
        let source_message_id = message.message_id;
        let mut references = Vec::with_capacity(2);
        if let Some(target_source_cursor) = summary.reply_to_source_cursor {
            references.push(CanonicalMessageReferenceProjectionV1 {
                source_message_id,
                target_source_cursor,
                kind: CommunicationMessageReferenceKindV1::Reply,
                observed_at_unix_seconds: summary.observed_at_unix_seconds,
            });
        }
        if let Some(target_source_cursor) = summary.forward_origin_source_cursor {
            references.push(CanonicalMessageReferenceProjectionV1 {
                source_message_id,
                target_source_cursor,
                kind: CommunicationMessageReferenceKindV1::Forward,
                observed_at_unix_seconds: summary.observed_at_unix_seconds,
            });
        }
        references
    } else {
        Vec::new()
    };
    Ok(CanonicalCommunicationProjectionV1 {
        summary: summary.clone(),
        account,
        conversation,
        message,
        participant,
        attachment_anchor,
        message_references,
    })
}

fn identifier(domain: &[u8], values: &[&[u8]]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for value in values {
        hasher.update(value);
    }
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("fixed SHA-256 prefix length")
}

pub fn convert_client_query_error(error: CommunicationsDomainError) -> CommunicationsClientError {
    match error {
        CommunicationsDomainError::InvalidObservedTime
        | CommunicationsDomainError::InvalidRecordedTime
        | CommunicationsDomainError::MissingMessageScope => {
            CommunicationsClientError::DraftValidationFailed
        }
        CommunicationsDomainError::InvalidAttachmentScope
        | CommunicationsDomainError::InvalidParticipantMetadata
        | CommunicationsDomainError::InvalidMessageSubject => {
            CommunicationsClientError::DraftValidationFailed
        }
        CommunicationsDomainError::InvalidAttachmentSafetyTransition => {
            CommunicationsClientError::DraftValidationFailed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_communications_api::{
        CommunicationBodyStateV1, CommunicationDirectionV1, CommunicationProviderProvenanceV1,
        CommunicationSourceCursorV1,
    };

    fn cursor(value: u8) -> CommunicationSourceCursorV1 {
        CommunicationSourceCursorV1::new([value; 32])
    }

    #[test]
    fn message_projection_uses_stable_source_and_conversation_identities() {
        let summary = accept_command(RecordCommunicationEvidenceV1 {
            observation_id: CommunicationObservationIdV1::new([1; 16]),
            causation_message_id: None,
            correlation_id: CommunicationObservationIdV1::new([6; 16]),
            source_cursor: cursor(2),
            account_cursor: Some(cursor(3)),
            conversation_cursor: Some(cursor(4)),
            participant_cursor: Some(cursor(5)),
            participant_display_label: None,
            message_subject: Some("Quarterly update".to_owned()),
            media_cursor: None,
            reply_to_source_cursor: None,
            forward_origin_source_cursor: None,
            provider: CommunicationProviderProvenanceV1::Telegram,
            direction: CommunicationDirectionV1::Unknown,
            kind: CanonicalCommunicationEvidenceKindV1::ChatMessage,
            body: CommunicationBodyStateV1::MetadataOnly,
            body_blob: None,
            body_media_type: None,
            body_admission_failure: None,
            attachment_descriptor: None,
            observed_at_unix_seconds: 1,
            recorded_at_unix_seconds: 2,
            recorded_at_nanos: 3,
        })
        .expect("valid message evidence");

        let first = canonicalize_communication(&summary).expect("projection");
        let second = canonicalize_communication(&summary).expect("projection");

        assert_eq!(first, second);
        assert!(matches!(
            first.message.as_ref().map(|value| value.mutation),
            Some(CanonicalMessageMutationV1::Create)
        ));
        assert_eq!(first.summary.causation_message_id, None);
        assert_eq!(
            first.summary.message_subject.as_deref(),
            Some("Quarterly update")
        );
        assert_eq!(
            first.summary.correlation_id,
            CommunicationObservationIdV1::new([6; 16])
        );
        assert_eq!(first.summary.recorded_at_unix_seconds, 2);
        assert_eq!(first.summary.recorded_at_nanos, 3);
    }

    #[test]
    fn deleted_message_is_a_typed_transition_not_a_new_message() {
        let summary = accept_command(RecordCommunicationEvidenceV1 {
            observation_id: CommunicationObservationIdV1::new([1; 16]),
            causation_message_id: None,
            correlation_id: CommunicationObservationIdV1::new([6; 16]),
            source_cursor: cursor(2),
            account_cursor: Some(cursor(3)),
            conversation_cursor: Some(cursor(4)),
            participant_cursor: None,
            participant_display_label: None,
            message_subject: None,
            media_cursor: None,
            reply_to_source_cursor: None,
            forward_origin_source_cursor: None,
            provider: CommunicationProviderProvenanceV1::WhatsAppWeb,
            direction: CommunicationDirectionV1::Unknown,
            kind: CanonicalCommunicationEvidenceKindV1::MessageDeleted,
            body: CommunicationBodyStateV1::MetadataOnly,
            body_blob: None,
            body_media_type: None,
            body_admission_failure: None,
            attachment_descriptor: None,
            observed_at_unix_seconds: 1,
            recorded_at_unix_seconds: 2,
            recorded_at_nanos: 3,
        })
        .expect("valid deletion evidence");

        let projection = canonicalize_communication(&summary).expect("projection");

        assert!(matches!(
            projection.message.as_ref().map(|value| value.mutation),
            Some(CanonicalMessageMutationV1::Delete)
        ));
    }

    #[test]
    fn message_transitions_require_canonical_message_scope() {
        let command = RecordCommunicationEvidenceV1 {
            observation_id: CommunicationObservationIdV1::new([1; 16]),
            causation_message_id: None,
            correlation_id: CommunicationObservationIdV1::new([6; 16]),
            source_cursor: cursor(2),
            account_cursor: Some(cursor(3)),
            conversation_cursor: None,
            participant_cursor: None,
            participant_display_label: None,
            message_subject: None,
            media_cursor: None,
            reply_to_source_cursor: None,
            forward_origin_source_cursor: None,
            provider: CommunicationProviderProvenanceV1::Telegram,
            direction: CommunicationDirectionV1::Incoming,
            kind: CanonicalCommunicationEvidenceKindV1::EmailMessage,
            body: CommunicationBodyStateV1::MetadataOnly,
            body_blob: None,
            body_media_type: None,
            body_admission_failure: None,
            attachment_descriptor: None,
            observed_at_unix_seconds: 1,
            recorded_at_unix_seconds: 2,
            recorded_at_nanos: 3,
        };
        for kind in [
            CanonicalCommunicationEvidenceKindV1::EmailMessage,
            CanonicalCommunicationEvidenceKindV1::ChatMessage,
            CanonicalCommunicationEvidenceKindV1::MessageEdited,
            CanonicalCommunicationEvidenceKindV1::MessageDeleted,
            CanonicalCommunicationEvidenceKindV1::ReactionChanged,
            CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged,
            CanonicalCommunicationEvidenceKindV1::MediaChanged,
        ] {
            let mut scoped = command.clone();
            scoped.kind = kind;
            assert_eq!(
                accept_command(scoped),
                Err(CommunicationsDomainError::MissingMessageScope)
            );
        }
    }

    #[test]
    fn message_references_are_typed_and_immutable_projection_inputs() {
        let summary = accept_command(RecordCommunicationEvidenceV1 {
            observation_id: CommunicationObservationIdV1::new([1; 16]),
            causation_message_id: None,
            correlation_id: CommunicationObservationIdV1::new([6; 16]),
            source_cursor: cursor(2),
            account_cursor: Some(cursor(3)),
            conversation_cursor: Some(cursor(4)),
            participant_cursor: None,
            participant_display_label: None,
            message_subject: None,
            media_cursor: None,
            reply_to_source_cursor: Some(cursor(5)),
            forward_origin_source_cursor: Some(cursor(6)),
            provider: CommunicationProviderProvenanceV1::Telegram,
            direction: CommunicationDirectionV1::Unknown,
            kind: CanonicalCommunicationEvidenceKindV1::ChatMessage,
            body: CommunicationBodyStateV1::MetadataOnly,
            body_blob: None,
            body_media_type: None,
            body_admission_failure: None,
            attachment_descriptor: None,
            observed_at_unix_seconds: 1,
            recorded_at_unix_seconds: 2,
            recorded_at_nanos: 3,
        })
        .expect("valid message evidence");

        let projection = canonicalize_communication(&summary).expect("projection");

        assert_eq!(projection.message_references.len(), 2);
        assert!(
            projection
                .message_references
                .iter()
                .any(|reference| reference.kind == CommunicationMessageReferenceKindV1::Reply)
        );
        assert!(
            projection
                .message_references
                .iter()
                .any(|reference| reference.kind == CommunicationMessageReferenceKindV1::Forward)
        );
    }

    #[test]
    fn incoming_participant_projects_a_cross_conversation_sender_identity() {
        let summary = accept_command(RecordCommunicationEvidenceV1 {
            observation_id: CommunicationObservationIdV1::new([1; 16]),
            causation_message_id: None,
            correlation_id: CommunicationObservationIdV1::new([6; 16]),
            source_cursor: cursor(2),
            account_cursor: Some(cursor(3)),
            conversation_cursor: Some(cursor(4)),
            participant_cursor: Some(cursor(5)),
            participant_display_label: Some("Ada <ada@example.test>".to_owned()),
            message_subject: None,
            media_cursor: None,
            reply_to_source_cursor: None,
            forward_origin_source_cursor: None,
            provider: CommunicationProviderProvenanceV1::MailImap,
            direction: CommunicationDirectionV1::Incoming,
            kind: CanonicalCommunicationEvidenceKindV1::EmailMessage,
            body: CommunicationBodyStateV1::MetadataOnly,
            body_blob: None,
            body_media_type: None,
            body_admission_failure: None,
            attachment_descriptor: None,
            observed_at_unix_seconds: 1,
            recorded_at_unix_seconds: 2,
            recorded_at_nanos: 3,
        })
        .expect("valid incoming sender evidence");

        let projection = canonicalize_communication(&summary).expect("projection");
        let participant = projection.participant.expect("participant projection");

        assert!(participant.sender_id.is_some());
        assert_eq!(
            participant.display_label.as_deref(),
            Some("Ada <ada@example.test>")
        );
    }
}
