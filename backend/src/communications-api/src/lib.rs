//! Public typed contract for the canonical Communications owner.

pub const PACKAGE: &str = "hermes-communications-api";

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/hermes.communications.v1.rs"));
}

pub mod query_wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/hermes.communications.query.v1.rs"
    ));
}

mod attachment;
mod query_projection;
pub use query_projection::CommunicationsQueryProjectionErrorV1;

pub use attachment::{
    AttachmentDescriptorV1, AttachmentDescriptorViolationV1, AttachmentDispositionV1,
    AttachmentSafetyStateV1, AttachmentSafetyTransitionCommandV1,
    AttachmentSafetyTransitionDecisionV1, AttachmentSafetyTransitionV1,
    AttachmentSafetyTransitionViolationV1,
};

include!(concat!(
    env!("OUT_DIR"),
    "/communications_evidence_schema.rs"
));
include!(concat!(env!("OUT_DIR"), "/communications_query_schema.rs"));

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CommunicationObservationIdV1([u8; 16]);
impl CommunicationObservationIdV1 {
    pub const fn new(value: [u8; 16]) -> Self {
        Self(value)
    }
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
    pub fn as_hex(self) -> String {
        hex(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CommunicationSourceCursorV1([u8; 32]);
impl CommunicationSourceCursorV1 {
    pub const fn new(value: [u8; 32]) -> Self {
        Self(value)
    }
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
    pub fn as_hex(self) -> String {
        hex(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CommunicationConversationIdV1([u8; 16]);
impl CommunicationConversationIdV1 {
    pub const fn new(value: [u8; 16]) -> Self {
        Self(value)
    }
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CommunicationMessageIdV1([u8; 16]);
impl CommunicationMessageIdV1 {
    pub const fn new(value: [u8; 16]) -> Self {
        Self(value)
    }
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CommunicationParticipantIdV1([u8; 16]);
impl CommunicationParticipantIdV1 {
    pub const fn new(value: [u8; 16]) -> Self {
        Self(value)
    }
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CommunicationSenderIdV1([u8; 16]);
impl CommunicationSenderIdV1 {
    pub const fn new(value: [u8; 16]) -> Self {
        Self(value)
    }
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CommunicationAttachmentAnchorIdV1([u8; 16]);
impl CommunicationAttachmentAnchorIdV1 {
    pub const fn new(value: [u8; 16]) -> Self {
        Self(value)
    }
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CommunicationAccountIdV1([u8; 16]);
impl CommunicationAccountIdV1 {
    pub const fn new(value: [u8; 16]) -> Self {
        Self(value)
    }
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationProviderProvenanceV1 {
    MailImap,
    Telegram,
    WhatsAppWeb,
    MailSmtp,
    Zulip,
    MailGmail,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationDirectionV1 {
    Incoming,
    Outgoing,
    Unknown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalCommunicationEvidenceKindV1 {
    EmailMessage,
    ChatMessage,
    MessageEdited,
    MessageDeleted,
    ReactionChanged,
    DeliveryStateChanged,
    ConversationStateChanged,
    ParticipantChanged,
    MediaChanged,
    TopicChanged,
    TypingChanged,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationBodyStateV1 {
    MetadataOnly,
    PendingBlob,
    Unavailable,
    AdmittedBlob,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationBodyAdmissionFailureV1 {
    SourceUnavailable,
    SizeLimitExceeded,
    IntegrityMismatch,
    PolicyRejected,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationBodyBlobReferenceV1 {
    pub blob_ref: String,
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordCommunicationEvidenceV1 {
    pub observation_id: CommunicationObservationIdV1,
    pub causation_message_id: Option<CommunicationObservationIdV1>,
    pub correlation_id: CommunicationObservationIdV1,
    pub source_cursor: CommunicationSourceCursorV1,
    pub account_cursor: Option<CommunicationSourceCursorV1>,
    pub conversation_cursor: Option<CommunicationSourceCursorV1>,
    pub participant_cursor: Option<CommunicationSourceCursorV1>,
    pub participant_display_label: Option<String>,
    pub message_subject: Option<String>,
    pub media_cursor: Option<CommunicationSourceCursorV1>,
    pub reply_to_source_cursor: Option<CommunicationSourceCursorV1>,
    pub forward_origin_source_cursor: Option<CommunicationSourceCursorV1>,
    pub provider: CommunicationProviderProvenanceV1,
    pub direction: CommunicationDirectionV1,
    pub kind: CanonicalCommunicationEvidenceKindV1,
    pub body: CommunicationBodyStateV1,
    pub body_blob: Option<CommunicationBodyBlobReferenceV1>,
    pub body_media_type: Option<String>,
    pub body_admission_failure: Option<CommunicationBodyAdmissionFailureV1>,
    pub attachment_descriptor: Option<AttachmentDescriptorV1>,
    pub observed_at_unix_seconds: i64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationSummary {
    pub evidence_id: CommunicationObservationIdV1,
    pub observation_id: CommunicationObservationIdV1,
    pub causation_message_id: Option<CommunicationObservationIdV1>,
    pub correlation_id: CommunicationObservationIdV1,
    pub source_cursor: CommunicationSourceCursorV1,
    pub account_cursor: Option<CommunicationSourceCursorV1>,
    pub conversation_cursor: Option<CommunicationSourceCursorV1>,
    pub participant_cursor: Option<CommunicationSourceCursorV1>,
    pub participant_display_label: Option<String>,
    pub message_subject: Option<String>,
    pub media_cursor: Option<CommunicationSourceCursorV1>,
    pub reply_to_source_cursor: Option<CommunicationSourceCursorV1>,
    pub forward_origin_source_cursor: Option<CommunicationSourceCursorV1>,
    pub provider: CommunicationProviderProvenanceV1,
    pub direction: CommunicationDirectionV1,
    pub kind: CanonicalCommunicationEvidenceKindV1,
    pub body: CommunicationBodyStateV1,
    pub body_blob: Option<CommunicationBodyBlobReferenceV1>,
    pub body_media_type: Option<String>,
    pub body_admission_failure: Option<CommunicationBodyAdmissionFailureV1>,
    pub attachment_descriptor: Option<AttachmentDescriptorV1>,
    pub observed_at_unix_seconds: i64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalMessageMutationV1 {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalConversationProjectionV1 {
    pub conversation_id: CommunicationConversationIdV1,
    pub account_cursor: CommunicationSourceCursorV1,
    pub conversation_cursor: CommunicationSourceCursorV1,
    pub provider: CommunicationProviderProvenanceV1,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalAccountProjectionV1 {
    pub account_id: CommunicationAccountIdV1,
    pub account_cursor: CommunicationSourceCursorV1,
    pub provider: CommunicationProviderProvenanceV1,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalMessageProjectionV1 {
    pub message_id: CommunicationMessageIdV1,
    pub conversation_id: CommunicationConversationIdV1,
    pub source_cursor: CommunicationSourceCursorV1,
    pub body: CommunicationBodyStateV1,
    pub direction: CommunicationDirectionV1,
    pub observed_at_unix_seconds: i64,
    pub mutation: CanonicalMessageMutationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalObservedParticipantProjectionV1 {
    pub participant_id: CommunicationParticipantIdV1,
    pub sender_id: Option<CommunicationSenderIdV1>,
    pub conversation_id: CommunicationConversationIdV1,
    pub participant_cursor: CommunicationSourceCursorV1,
    pub display_label: Option<String>,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalAttachmentAnchorProjectionV1 {
    pub attachment_anchor_id: CommunicationAttachmentAnchorIdV1,
    pub message_id: CommunicationMessageIdV1,
    pub media_cursor: CommunicationSourceCursorV1,
    pub descriptor: Option<AttachmentDescriptorV1>,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationMessageReferenceKindV1 {
    Reply,
    Forward,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalMessageReferenceProjectionV1 {
    pub source_message_id: CommunicationMessageIdV1,
    pub target_source_cursor: CommunicationSourceCursorV1,
    pub kind: CommunicationMessageReferenceKindV1,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCommunicationProjectionV1 {
    pub summary: CommunicationSummary,
    pub account: Option<CanonicalAccountProjectionV1>,
    pub conversation: Option<CanonicalConversationProjectionV1>,
    pub message: Option<CanonicalMessageProjectionV1>,
    pub participant: Option<CanonicalObservedParticipantProjectionV1>,
    pub attachment_anchor: Option<CanonicalAttachmentAnchorProjectionV1>,
    pub message_references: Vec<CanonicalMessageReferenceProjectionV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationMessageLifecycleStateV1 {
    Active,
    Deleted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationConversationSummaryV1 {
    pub conversation_id: CommunicationConversationIdV1,
    pub account_cursor: CommunicationSourceCursorV1,
    pub conversation_cursor: CommunicationSourceCursorV1,
    pub provider: CommunicationProviderProvenanceV1,
    pub first_observed_at_unix_seconds: i64,
    pub last_observed_at_unix_seconds: i64,
    pub last_evidence_id: CommunicationObservationIdV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationAccountSummaryV1 {
    pub account_id: CommunicationAccountIdV1,
    pub account_cursor: CommunicationSourceCursorV1,
    pub provider: CommunicationProviderProvenanceV1,
    pub first_observed_at_unix_seconds: i64,
    pub last_observed_at_unix_seconds: i64,
    pub last_evidence_id: CommunicationObservationIdV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationMessageSummaryV1 {
    pub message_id: CommunicationMessageIdV1,
    pub conversation_id: CommunicationConversationIdV1,
    pub source_cursor: CommunicationSourceCursorV1,
    pub body: CommunicationBodyStateV1,
    pub direction: CommunicationDirectionV1,
    pub lifecycle_state: CommunicationMessageLifecycleStateV1,
    pub first_observed_at_unix_seconds: i64,
    pub last_observed_at_unix_seconds: i64,
    pub last_evidence_id: CommunicationObservationIdV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationObservedParticipantSummaryV1 {
    pub participant_id: CommunicationParticipantIdV1,
    pub conversation_id: CommunicationConversationIdV1,
    pub participant_cursor: CommunicationSourceCursorV1,
    pub display_label: Option<String>,
    pub first_observed_at_unix_seconds: i64,
    pub last_observed_at_unix_seconds: i64,
    pub last_evidence_id: CommunicationObservationIdV1,
}

pub type CommunicationAttachmentAnchorStateV1 = AttachmentSafetyStateV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationAttachmentAnchorSummaryV1 {
    pub attachment_anchor_id: CommunicationAttachmentAnchorIdV1,
    pub message_id: CommunicationMessageIdV1,
    pub media_cursor: CommunicationSourceCursorV1,
    pub state: CommunicationAttachmentAnchorStateV1,
    pub descriptor: Option<AttachmentDescriptorV1>,
    pub first_observed_at_unix_seconds: i64,
    pub last_observed_at_unix_seconds: i64,
    pub last_evidence_id: CommunicationObservationIdV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationMessageReferenceSummaryV1 {
    pub source_message_id: CommunicationMessageIdV1,
    pub kind: CommunicationMessageReferenceKindV1,
    pub target_source_cursor: CommunicationSourceCursorV1,
    pub target_message_id: Option<CommunicationMessageIdV1>,
    pub observed_at_unix_seconds: i64,
    pub evidence_id: CommunicationObservationIdV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GetCommunicationConversationV1 {
    pub conversation_id: CommunicationConversationIdV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GetCommunicationMessageV1 {
    pub message_id: CommunicationMessageIdV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListCommunicationAccountsV1 {
    pub limit: u16,
    pub cursor: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListCommunicationConversationsV1 {
    pub account_cursor: Option<CommunicationSourceCursorV1>,
    pub limit: u16,
    pub cursor: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListConversationMessagesV1 {
    pub conversation_id: CommunicationConversationIdV1,
    pub limit: u16,
    pub cursor: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListConversationParticipantsV1 {
    pub conversation_id: CommunicationConversationIdV1,
    pub limit: u16,
    pub cursor: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListMessageAttachmentAnchorsV1 {
    pub message_id: CommunicationMessageIdV1,
    pub limit: u16,
    pub cursor: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListMessageReferencesV1 {
    pub message_id: CommunicationMessageIdV1,
    pub limit: u16,
    pub cursor: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListMessageEvidenceV1 {
    pub message_id: CommunicationMessageIdV1,
    pub limit: u16,
    pub cursor: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchCommunicationsV1 {
    pub query: String,
    pub limit: u16,
    pub cursor: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationSearchHitV1 {
    pub evidence_id: CommunicationObservationIdV1,
    pub message_id: CommunicationMessageIdV1,
    pub conversation_id: CommunicationConversationIdV1,
    pub observed_at_unix_seconds: i64,
    pub matched_token_count: u16,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GetCommunicationEvidenceV1 {
    pub evidence_id: CommunicationObservationIdV1,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationsClientError {
    UnknownCommunication,
    InvalidCursor,
    DraftValidationFailed,
    DuplicateObservation,
    Unavailable,
}

fn hex(value: &[u8]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use prost::Message;

    use super::wire::{CommunicationBodyStateV1, CommunicationEvidenceRecordedV1};

    #[test]
    fn canonical_evidence_wire_preserves_body_admission_failure() {
        let encoded = CommunicationEvidenceRecordedV1 {
            body: CommunicationBodyStateV1::Unavailable as i32,
            ..Default::default()
        }
        .encode_to_vec();
        let decoded = CommunicationEvidenceRecordedV1::decode(encoded.as_slice())
            .expect("canonical evidence must decode");
        assert_eq!(
            CommunicationBodyStateV1::try_from(decoded.body),
            Ok(CommunicationBodyStateV1::Unavailable),
        );
    }

    #[test]
    fn message_evidence_history_wire_is_metadata_only() {
        let request = super::query_wire::ListMessageEvidenceRequestV1 {
            message_id: vec![7; 16],
            limit: 25,
            cursor: Vec::new(),
        };
        let decoded = super::query_wire::ListMessageEvidenceRequestV1::decode(
            request.encode_to_vec().as_slice(),
        )
        .expect("message evidence request must decode");
        assert_eq!(decoded.message_id, vec![7; 16]);
        assert_eq!(decoded.limit, 25);

        let response = super::query_wire::ListMessageEvidenceResponseV1::default();
        assert!(response.evidence.is_empty());
    }
}
