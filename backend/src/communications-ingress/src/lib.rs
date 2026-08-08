//! Versioned provider-neutral ingress owned by Communications.

pub mod admission;

use hermes_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ObservationMetadataV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

pub mod v1 {
    include!(concat!(
        env!("OUT_DIR"),
        "/hermes.communications.ingress.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communications_observation_schema.rs"
));

pub const PACKAGE: &str = "hermes-communications-ingress";
pub const COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID: &str = "communications";
pub const COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID: &str = "hermes-communications-runtime";
pub const COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID: &str = "communications.blob.v1";
pub const MAX_OBSERVATION_ID_LEN: usize = 256;
pub const MAX_EXTERNAL_RECORD_ID_LEN: usize = 512;
pub const MAX_SOURCE_SCOPE_ID_LEN: usize = 512;
pub const MAX_PARTICIPANT_DISPLAY_LABEL_BYTES: usize = 256;
pub const MAX_MESSAGE_SUBJECT_BYTES: usize = 998;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderProvenanceV1 {
    MailImap,
    MailGmail,
    Telegram,
    WhatsAppWeb,
    MailSmtp,
    Zulip,
}

impl ProviderProvenanceV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MailImap => "mail-imap",
            Self::MailGmail => "mail-gmail",
            Self::Telegram => "telegram",
            Self::WhatsAppWeb => "whatsapp-web",
            Self::MailSmtp => "mail-smtp",
            Self::Zulip => "zulip",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationEvidenceKindV1 {
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
pub enum CommunicationDirectionV1 {
    Incoming,
    Outgoing,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyAvailabilityV1 {
    MetadataOnly,
    PendingBlob,
    Unavailable,
    AdmittedBlob,
}

impl BodyAvailabilityV1 {
    pub const fn has_pending_blob(self) -> bool {
        matches!(self, Self::PendingBlob)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyAdmissionFailureV1 {
    SourceUnavailable,
    SizeLimitExceeded,
    IntegrityMismatch,
    PolicyRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BodyBlobReceiptV1 {
    pub blob_ref: String,
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub media_type: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentDispositionV1 {
    Attachment,
    Inline,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentDescriptorV1 {
    pub filename: Option<String>,
    pub media_type: String,
    pub declared_bytes: u64,
    pub sha256: Option<[u8; 32]>,
    pub disposition: AttachmentDispositionV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceEnvelope {
    pub provider: ProviderProvenanceV1,
    pub external_record_id: String,
    pub scope: Option<SourceScopeEnvelope>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceScopeEnvelope {
    pub external_account_id: String,
    pub external_conversation_id: Option<String>,
    pub external_participant_id: Option<String>,
    pub external_media_id: Option<String>,
    pub external_reply_to_record_id: Option<String>,
    pub external_forward_origin_record_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationObservationDraft {
    pub observation_id: String,
    pub source: SourceEnvelope,
    pub kind: CommunicationEvidenceKindV1,
    pub direction: CommunicationDirectionV1,
    pub body: BodyAvailabilityV1,
    pub body_blob: Option<BodyBlobReceiptV1>,
    pub body_admission_failure: Option<BodyAdmissionFailureV1>,
    pub attachment_descriptor: Option<AttachmentDescriptorV1>,
    pub participant_display_label: Option<String>,
    pub message_subject: Option<String>,
    pub observed_at_unix_seconds: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IngressDraftError {
    EmptyObservationId,
    ObservationIdTooLong,
    EmptyExternalRecordId,
    ExternalRecordIdTooLong,
    EmptyExternalAccountId,
    ExternalAccountIdTooLong,
    EmptyExternalConversationId,
    ExternalConversationIdTooLong,
    EmptyExternalParticipantId,
    ExternalParticipantIdTooLong,
    EmptyExternalMediaId,
    ExternalMediaIdTooLong,
    EmptyExternalReplyToRecordId,
    ExternalReplyToRecordIdTooLong,
    EmptyExternalForwardOriginRecordId,
    ExternalForwardOriginRecordIdTooLong,
    InvalidAttachmentDescriptor,
    InvalidBodyAdmission,
    InvalidParticipantDisplayLabel,
    InvalidMessageSubject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationEnvelopeContextV1 {
    /// Exact live managed-runtime identity supplied by Kernel admission. It is
    /// converted to an opaque, stable source reference before serialization.
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub module_id: String,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn new_communication_observation_draft(
    observation_id: impl Into<String>,
    source: SourceEnvelope,
    kind: CommunicationEvidenceKindV1,
    body: BodyAvailabilityV1,
    direction: CommunicationDirectionV1,
    observed_at_unix_seconds: Option<i64>,
) -> Result<CommunicationObservationDraft, IngressDraftError> {
    let observation_id = observation_id.into();
    if observation_id.trim().is_empty() {
        return Err(IngressDraftError::EmptyObservationId);
    }
    if observation_id.len() > MAX_OBSERVATION_ID_LEN {
        return Err(IngressDraftError::ObservationIdTooLong);
    }
    if source.external_record_id.trim().is_empty() {
        return Err(IngressDraftError::EmptyExternalRecordId);
    }
    if source.external_record_id.len() > MAX_EXTERNAL_RECORD_ID_LEN {
        return Err(IngressDraftError::ExternalRecordIdTooLong);
    }
    Ok(CommunicationObservationDraft {
        observation_id,
        source,
        kind,
        direction,
        body,
        body_blob: None,
        body_admission_failure: None,
        attachment_descriptor: None,
        participant_display_label: None,
        message_subject: None,
        observed_at_unix_seconds,
    })
}

pub fn with_admitted_body_blob(
    mut draft: CommunicationObservationDraft,
    receipt: BodyBlobReceiptV1,
) -> Result<CommunicationObservationDraft, IngressDraftError> {
    if draft.body != BodyAvailabilityV1::AdmittedBlob || !valid_body_blob_receipt(&receipt) {
        return Err(IngressDraftError::InvalidBodyAdmission);
    }
    draft.body_blob = Some(receipt);
    Ok(draft)
}

pub fn with_body_admission_failure(
    mut draft: CommunicationObservationDraft,
    failure: BodyAdmissionFailureV1,
) -> Result<CommunicationObservationDraft, IngressDraftError> {
    if draft.body != BodyAvailabilityV1::Unavailable {
        return Err(IngressDraftError::InvalidBodyAdmission);
    }
    draft.body_admission_failure = Some(failure);
    Ok(draft)
}

pub fn with_attachment_descriptor(
    mut draft: CommunicationObservationDraft,
    descriptor: AttachmentDescriptorV1,
) -> Result<CommunicationObservationDraft, IngressDraftError> {
    if draft.kind != CommunicationEvidenceKindV1::MediaChanged
        || draft
            .source
            .scope
            .as_ref()
            .and_then(|scope| scope.external_media_id.as_ref())
            .is_none()
        || !valid_attachment_descriptor(&descriptor)
    {
        return Err(IngressDraftError::InvalidAttachmentDescriptor);
    }
    draft.attachment_descriptor = Some(descriptor);
    Ok(draft)
}

pub fn with_participant_display_label(
    mut draft: CommunicationObservationDraft,
    display_label: Option<String>,
) -> Result<CommunicationObservationDraft, IngressDraftError> {
    draft.participant_display_label = display_label
        .map(|value| normalize_participant_display_label(&value))
        .transpose()?;
    Ok(draft)
}

pub fn with_message_subject(
    mut draft: CommunicationObservationDraft,
    message_subject: Option<String>,
) -> Result<CommunicationObservationDraft, IngressDraftError> {
    draft.message_subject = message_subject
        .map(|value| normalize_message_subject(&value))
        .transpose()?;
    Ok(draft)
}

pub fn new_scoped_communication_observation_draft(
    observation_id: impl Into<String>,
    source: SourceEnvelope,
    kind: CommunicationEvidenceKindV1,
    body: BodyAvailabilityV1,
    direction: CommunicationDirectionV1,
    observed_at_unix_seconds: Option<i64>,
) -> Result<CommunicationObservationDraft, IngressDraftError> {
    let scope = source
        .scope
        .as_ref()
        .ok_or(IngressDraftError::EmptyExternalAccountId)?;
    validate_source_scope(scope)?;
    new_communication_observation_draft(
        observation_id,
        source,
        kind,
        body,
        direction,
        observed_at_unix_seconds,
    )
}

/// Builds the immutable broker record. Callers must supply the live Kernel-issued runtime identity.
pub fn build_observation_outbox_record_v1(
    draft: &CommunicationObservationDraft,
    context: &ObservationEnvelopeContextV1,
) -> Result<OutboxRecordV1, ObservationEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let message_id = observation_message_id(draft);
    let correlation_id = observation_correlation_id(draft);
    let cursor = source_cursor_sha256(draft);
    let scope_cursors = source_scope_cursors(draft);
    let timestamp = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let payload = v1::CommunicationObservationV1 {
        provider: provider_value(draft.source.provider),
        kind: evidence_kind_value(draft.kind),
        direction: direction_value(draft.direction),
        body: body_availability_value(draft.body),
        account_cursor_sha256: scope_cursors
            .account
            .map_or_else(Vec::new, |value| value.to_vec()),
        conversation_cursor_sha256: scope_cursors
            .conversation
            .map_or_else(Vec::new, |value| value.to_vec()),
        participant_cursor_sha256: scope_cursors
            .participant
            .map_or_else(Vec::new, |value| value.to_vec()),
        media_cursor_sha256: scope_cursors
            .media
            .map_or_else(Vec::new, |value| value.to_vec()),
        reply_to_source_cursor_sha256: scope_cursors
            .reply_to_source
            .map_or_else(Vec::new, |value| value.to_vec()),
        forward_origin_source_cursor_sha256: scope_cursors
            .forward_origin_source
            .map_or_else(Vec::new, |value| value.to_vec()),
        attachment_descriptor: draft.attachment_descriptor.as_ref().map(|descriptor| {
            v1::AttachmentDescriptorV1 {
                filename: descriptor.filename.clone().unwrap_or_default(),
                media_type: descriptor.media_type.clone(),
                declared_bytes: descriptor.declared_bytes,
                sha256: descriptor
                    .sha256
                    .map_or_else(Vec::new, |value| value.to_vec()),
                disposition: attachment_disposition_value(descriptor.disposition),
            }
        }),
        body_blob: draft
            .body_blob
            .as_ref()
            .map(|receipt| v1::BodyBlobReceiptV1 {
                blob_ref: receipt.blob_ref.clone(),
                reference_id: receipt.reference_id.to_vec(),
                declared_bytes: receipt.declared_bytes,
                sha256: receipt.sha256.to_vec(),
                custody_transfer_source_proof: receipt.custody_transfer_source_proof.clone(),
                media_type: receipt.media_type.clone(),
            }),
        body_admission_failure: draft
            .body_admission_failure
            .map(body_admission_failure_value)
            .unwrap_or_default(),
        participant_display_label: draft.participant_display_label.clone(),
        message_subject: draft.message_subject.clone(),
    }
    .encode_to_vec();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: admission::COMMUNICATION_OBSERVED_CONTRACT_OWNER.to_owned(),
            name: admission::COMMUNICATION_OBSERVED_CONTRACT_NAME.to_owned(),
            major: admission::COMMUNICATION_OBSERVED_CONTRACT_MAJOR,
            revision: admission::COMMUNICATION_OBSERVED_CONTRACT_REVISION,
            schema_sha256: COMMUNICATION_OBSERVATION_SCHEMA_SHA256.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp),
        partition_key: cursor.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: correlation_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Observation(ObservationMetadataV1 {
            observation_id: message_id.to_vec(),
            observed_at: Some(timestamp),
            occurred_at: draft
                .observed_at_unix_seconds
                .map(|seconds| Timestamp { seconds, nanos: 0 }),
            source_cursor_sha256: cursor.to_vec(),
            source_sequence: None,
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| ObservationEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &ObservationEnvelopeContextV1,
) -> Result<(), ObservationEnvelopeBuildErrorV1> {
    if context.runtime_generation == 0
        || !valid_module_id(&context.module_id)
        || !valid_runtime_instance_id(&context.runtime_instance_id)
        || !valid_timestamp(context.recorded_at_unix_seconds, context.recorded_at_nanos)
    {
        return Err(ObservationEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.runtime.source-reference.v1\0");
    hasher.update(runtime_instance_id.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("fixed SHA-256 prefix length")
}

fn observation_message_id(draft: &CommunicationObservationDraft) -> [u8; 16] {
    observation_identifier(b"hermes.communications.observation-message.v1\0", draft)
}

fn observation_correlation_id(draft: &CommunicationObservationDraft) -> [u8; 16] {
    observation_identifier(b"hermes.communications.observation-correlation.v1\0", draft)
}

fn observation_identifier(domain: &[u8], draft: &CommunicationObservationDraft) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(draft.source.provider.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(draft.observation_id.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("fixed SHA-256 prefix length")
}

fn source_cursor_sha256(draft: &CommunicationObservationDraft) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.communications.source-cursor.v1\0");
    hasher.update(draft.source.provider.as_str().as_bytes());
    hasher.update(b"\0");
    if let Some(scope) = &draft.source.scope {
        hasher.update(scope.external_account_id.as_bytes());
        hasher.update(b"\0");
    }
    hasher.update(draft.source.external_record_id.as_bytes());
    hasher.finalize().into()
}

struct SourceScopeCursorsV1 {
    account: Option<[u8; 32]>,
    conversation: Option<[u8; 32]>,
    participant: Option<[u8; 32]>,
    media: Option<[u8; 32]>,
    reply_to_source: Option<[u8; 32]>,
    forward_origin_source: Option<[u8; 32]>,
}

fn source_scope_cursors(draft: &CommunicationObservationDraft) -> SourceScopeCursorsV1 {
    let Some(scope) = draft.source.scope.as_ref() else {
        return SourceScopeCursorsV1 {
            account: None,
            conversation: None,
            participant: None,
            media: None,
            reply_to_source: None,
            forward_origin_source: None,
        };
    };
    let account = scope_identifier(
        b"hermes.communications.account-cursor.v1\0",
        draft.source.provider,
        &scope.external_account_id,
    );
    let conversation = scope
        .external_conversation_id
        .as_ref()
        .map(|value| conversation_source_cursor_from_account(account, value));
    let participant = scope.external_participant_id.as_ref().map(|value| {
        let mut hasher = Sha256::new();
        hasher.update(b"hermes.communications.participant-cursor.v1\0");
        hasher.update(account);
        hasher.update(value.as_bytes());
        hasher.finalize().into()
    });
    let media = scope.external_media_id.as_ref().map(|value| {
        let mut hasher = Sha256::new();
        hasher.update(b"hermes.communications.media-cursor.v1\0");
        hasher.update(account);
        hasher.update(value.as_bytes());
        hasher.finalize().into()
    });
    let reply_to = scope.external_reply_to_record_id.as_ref().map(|value| {
        source_cursor_for_scoped_record(draft.source.provider, &scope.external_account_id, value)
    });
    let forward_origin = scope
        .external_forward_origin_record_id
        .as_ref()
        .map(|value| {
            source_cursor_for_scoped_record(
                draft.source.provider,
                &scope.external_account_id,
                value,
            )
        });
    SourceScopeCursorsV1 {
        account: Some(account),
        conversation,
        participant,
        media,
        reply_to_source: reply_to,
        forward_origin_source: forward_origin,
    }
}

/// Derives the exact opaque account cursor emitted in Communications evidence.
///
/// Provider integrations use this public contract to maintain their own
/// reverse locator projection. The raw provider locator never crosses the
/// integration boundary.
pub fn account_source_cursor_v1(
    provider: ProviderProvenanceV1,
    external_account_id: &str,
) -> Result<[u8; 32], IngressDraftError> {
    validate_source_identifier(
        external_account_id,
        MAX_SOURCE_SCOPE_ID_LEN,
        IngressDraftError::EmptyExternalAccountId,
        IngressDraftError::ExternalAccountIdTooLong,
    )?;
    Ok(scope_identifier(
        b"hermes.communications.account-cursor.v1\0",
        provider,
        external_account_id,
    ))
}

/// Derives the exact opaque conversation cursor emitted in Communications
/// evidence for a provider-owned account and conversation locator.
pub fn conversation_source_cursor_v1(
    provider: ProviderProvenanceV1,
    external_account_id: &str,
    external_conversation_id: &str,
) -> Result<[u8; 32], IngressDraftError> {
    let account = account_source_cursor_v1(provider, external_account_id)?;
    validate_source_identifier(
        external_conversation_id,
        MAX_SOURCE_SCOPE_ID_LEN,
        IngressDraftError::EmptyExternalConversationId,
        IngressDraftError::ExternalConversationIdTooLong,
    )?;
    Ok(conversation_source_cursor_from_account(
        account,
        external_conversation_id,
    ))
}

/// Derives the exact opaque source-record cursor emitted in Communications
/// evidence for a provider-owned account and record locator.
pub fn scoped_record_source_cursor_v1(
    provider: ProviderProvenanceV1,
    external_account_id: &str,
    external_record_id: &str,
) -> Result<[u8; 32], IngressDraftError> {
    validate_source_identifier(
        external_account_id,
        MAX_SOURCE_SCOPE_ID_LEN,
        IngressDraftError::EmptyExternalAccountId,
        IngressDraftError::ExternalAccountIdTooLong,
    )?;
    validate_source_identifier(
        external_record_id,
        MAX_EXTERNAL_RECORD_ID_LEN,
        IngressDraftError::EmptyExternalRecordId,
        IngressDraftError::ExternalRecordIdTooLong,
    )?;
    Ok(source_cursor_for_scoped_record(
        provider,
        external_account_id,
        external_record_id,
    ))
}

fn source_cursor_for_scoped_record(
    provider: ProviderProvenanceV1,
    account_id: &str,
    record_id: &str,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.communications.source-cursor.v1\0");
    hasher.update(provider.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(record_id.as_bytes());
    hasher.finalize().into()
}

fn scope_identifier(domain: &[u8], provider: ProviderProvenanceV1, value: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(provider.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(value.as_bytes());
    hasher.finalize().into()
}

fn conversation_source_cursor_from_account(
    account_cursor: [u8; 32],
    external_conversation_id: &str,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.communications.conversation-cursor.v1\0");
    hasher.update(account_cursor);
    hasher.update(external_conversation_id.as_bytes());
    hasher.finalize().into()
}

fn validate_source_identifier(
    value: &str,
    max_len: usize,
    empty: IngressDraftError,
    too_long: IngressDraftError,
) -> Result<(), IngressDraftError> {
    if value.is_empty() {
        return Err(empty);
    }
    if value.len() > max_len {
        return Err(too_long);
    }
    Ok(())
}

#[cfg(test)]
mod source_cursor_contract_tests {
    use super::*;

    #[test]
    fn public_locator_derivation_matches_observation_scope_cursors() {
        let draft = new_scoped_communication_observation_draft(
            "observation-1".to_owned(),
            SourceEnvelope {
                provider: ProviderProvenanceV1::Telegram,
                external_record_id: "message-1".to_owned(),
                scope: Some(SourceScopeEnvelope {
                    external_account_id: "account-1".to_owned(),
                    external_conversation_id: Some("chat-1".to_owned()),
                    external_participant_id: None,
                    external_media_id: None,
                    external_reply_to_record_id: Some("message-0".to_owned()),
                    external_forward_origin_record_id: None,
                }),
            },
            CommunicationEvidenceKindV1::ChatMessage,
            BodyAvailabilityV1::MetadataOnly,
            CommunicationDirectionV1::Incoming,
            Some(1_700_000_000),
        )
        .expect("valid draft");

        let cursors = source_scope_cursors(&draft);
        assert_eq!(
            cursors.account,
            Some(
                account_source_cursor_v1(ProviderProvenanceV1::Telegram, "account-1")
                    .expect("account cursor"),
            )
        );
        assert_eq!(
            cursors.conversation,
            Some(
                conversation_source_cursor_v1(
                    ProviderProvenanceV1::Telegram,
                    "account-1",
                    "chat-1",
                )
                .expect("conversation cursor"),
            )
        );
        assert_eq!(
            cursors.reply_to_source,
            Some(
                scoped_record_source_cursor_v1(
                    ProviderProvenanceV1::Telegram,
                    "account-1",
                    "message-0",
                )
                .expect("record cursor"),
            )
        );
    }

    #[test]
    fn public_locator_derivation_rejects_unbounded_provider_identity() {
        assert_eq!(
            account_source_cursor_v1(ProviderProvenanceV1::MailImap, ""),
            Err(IngressDraftError::EmptyExternalAccountId)
        );
        assert_eq!(
            conversation_source_cursor_v1(
                ProviderProvenanceV1::Zulip,
                "account-1",
                &"x".repeat(MAX_SOURCE_SCOPE_ID_LEN + 1),
            ),
            Err(IngressDraftError::ExternalConversationIdTooLong)
        );
        assert_eq!(
            scoped_record_source_cursor_v1(ProviderProvenanceV1::WhatsAppWeb, "account-1", "",),
            Err(IngressDraftError::EmptyExternalRecordId)
        );
    }
}

fn validate_source_scope(scope: &SourceScopeEnvelope) -> Result<(), IngressDraftError> {
    if scope.external_account_id.trim().is_empty() {
        return Err(IngressDraftError::EmptyExternalAccountId);
    }
    if scope.external_account_id.len() > MAX_SOURCE_SCOPE_ID_LEN {
        return Err(IngressDraftError::ExternalAccountIdTooLong);
    }
    if let Some(conversation_id) = &scope.external_conversation_id {
        if conversation_id.trim().is_empty() {
            return Err(IngressDraftError::EmptyExternalConversationId);
        }
        if conversation_id.len() > MAX_SOURCE_SCOPE_ID_LEN {
            return Err(IngressDraftError::ExternalConversationIdTooLong);
        }
    }
    if let Some(participant_id) = &scope.external_participant_id {
        if participant_id.trim().is_empty() {
            return Err(IngressDraftError::EmptyExternalParticipantId);
        }
        if participant_id.len() > MAX_SOURCE_SCOPE_ID_LEN {
            return Err(IngressDraftError::ExternalParticipantIdTooLong);
        }
    }
    if let Some(media_id) = &scope.external_media_id {
        if media_id.trim().is_empty() {
            return Err(IngressDraftError::EmptyExternalMediaId);
        }
        if media_id.len() > MAX_SOURCE_SCOPE_ID_LEN {
            return Err(IngressDraftError::ExternalMediaIdTooLong);
        }
    }
    for (value, empty, too_long) in [
        (
            &scope.external_reply_to_record_id,
            IngressDraftError::EmptyExternalReplyToRecordId,
            IngressDraftError::ExternalReplyToRecordIdTooLong,
        ),
        (
            &scope.external_forward_origin_record_id,
            IngressDraftError::EmptyExternalForwardOriginRecordId,
            IngressDraftError::ExternalForwardOriginRecordIdTooLong,
        ),
    ] {
        if let Some(value) = value {
            if value.trim().is_empty() {
                return Err(empty);
            }
            if value.len() > MAX_EXTERNAL_RECORD_ID_LEN {
                return Err(too_long);
            }
        }
    }
    Ok(())
}

fn valid_module_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_runtime_instance_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_timestamp(seconds: i64, nanos: i32) -> bool {
    (-62_135_596_800..=253_402_300_799).contains(&seconds) && (0..1_000_000_000).contains(&nanos)
}

fn valid_attachment_descriptor(value: &AttachmentDescriptorV1) -> bool {
    value
        .filename
        .as_deref()
        .is_none_or(|filename| !filename.is_empty() && filename.len() <= 512 && filename.is_ascii())
        && !value.media_type.is_empty()
        && value.media_type.len() <= 256
        && value.media_type.is_ascii()
        && value.media_type.contains('/')
        && !value.media_type.contains([' ', '\t', '\n', '\r', ';'])
        && value.declared_bytes <= 64 * 1024 * 1024
}

fn normalize_participant_display_label(value: &str) -> Result<String, IngressDraftError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > MAX_PARTICIPANT_DISPLAY_LABEL_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(IngressDraftError::InvalidParticipantDisplayLabel);
    }
    Ok(value.to_owned())
}

fn normalize_message_subject(value: &str) -> Result<String, IngressDraftError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > MAX_MESSAGE_SUBJECT_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(IngressDraftError::InvalidMessageSubject);
    }
    Ok(value.to_owned())
}

fn valid_body_blob_receipt(value: &BodyBlobReceiptV1) -> bool {
    !value.blob_ref.trim().is_empty()
        && value.blob_ref.len() <= 512
        && value.blob_ref.is_ascii()
        && value.reference_id.iter().any(|byte| *byte != 0)
        && (1..=64 * 1024 * 1024).contains(&value.declared_bytes)
        && (1..=2_048).contains(&value.custody_transfer_source_proof.len())
        && matches!(value.media_type.as_str(), "text/plain" | "text/html")
}

fn outbox_error(_: OutboxRecordError) -> ObservationEnvelopeBuildErrorV1 {
    ObservationEnvelopeBuildErrorV1::OutboxRejected
}
const fn provider_value(value: ProviderProvenanceV1) -> i32 {
    match value {
        ProviderProvenanceV1::MailImap => 1,
        ProviderProvenanceV1::Telegram => 2,
        ProviderProvenanceV1::WhatsAppWeb => 3,
        ProviderProvenanceV1::MailSmtp => 4,
        ProviderProvenanceV1::Zulip => 5,
        ProviderProvenanceV1::MailGmail => 6,
    }
}
const fn direction_value(value: CommunicationDirectionV1) -> i32 {
    match value {
        CommunicationDirectionV1::Incoming => 1,
        CommunicationDirectionV1::Outgoing => 2,
        CommunicationDirectionV1::Unknown => 3,
    }
}
const fn evidence_kind_value(value: CommunicationEvidenceKindV1) -> i32 {
    match value {
        CommunicationEvidenceKindV1::EmailMessage => 1,
        CommunicationEvidenceKindV1::ChatMessage => 2,
        CommunicationEvidenceKindV1::MessageEdited => 3,
        CommunicationEvidenceKindV1::MessageDeleted => 4,
        CommunicationEvidenceKindV1::ReactionChanged => 5,
        CommunicationEvidenceKindV1::DeliveryStateChanged => 6,
        CommunicationEvidenceKindV1::ConversationStateChanged => 7,
        CommunicationEvidenceKindV1::ParticipantChanged => 8,
        CommunicationEvidenceKindV1::MediaChanged => 9,
        CommunicationEvidenceKindV1::TopicChanged => 10,
        CommunicationEvidenceKindV1::TypingChanged => 11,
    }
}
const fn body_availability_value(value: BodyAvailabilityV1) -> i32 {
    match value {
        BodyAvailabilityV1::MetadataOnly => 1,
        BodyAvailabilityV1::PendingBlob => 2,
        BodyAvailabilityV1::Unavailable => 3,
        BodyAvailabilityV1::AdmittedBlob => 4,
    }
}
const fn body_admission_failure_value(value: BodyAdmissionFailureV1) -> i32 {
    match value {
        BodyAdmissionFailureV1::SourceUnavailable => 1,
        BodyAdmissionFailureV1::SizeLimitExceeded => 2,
        BodyAdmissionFailureV1::IntegrityMismatch => 3,
        BodyAdmissionFailureV1::PolicyRejected => 4,
    }
}
const fn attachment_disposition_value(value: AttachmentDispositionV1) -> i32 {
    match value {
        AttachmentDispositionV1::Attachment => 1,
        AttachmentDispositionV1::Inline => 2,
        AttachmentDispositionV1::Unknown => 3,
    }
}

#[cfg(test)]
mod body_admission_tests {
    use super::*;
    use hermes_events_protocol::validation::envelope::decode_envelope_v1;
    use prost::Message;

    #[test]
    fn encodes_an_admitted_body_as_an_opaque_blob_receipt() {
        let draft = new_communication_observation_draft(
            "observation-1",
            SourceEnvelope {
                provider: ProviderProvenanceV1::MailImap,
                external_record_id: "provider-record-1".to_owned(),
                scope: None,
            },
            CommunicationEvidenceKindV1::EmailMessage,
            BodyAvailabilityV1::AdmittedBlob,
            CommunicationDirectionV1::Incoming,
            Some(1_782_504_000),
        )
        .expect("draft");
        let draft = with_admitted_body_blob(
            draft,
            BodyBlobReceiptV1 {
                blob_ref: "blob-opaque-reference".to_owned(),
                reference_id: [7; 16],
                declared_bytes: 64,
                sha256: [9; 32],
                custody_transfer_source_proof: vec![4; 96],
                media_type: "text/html".to_owned(),
            },
        )
        .expect("receipt");
        let record = build_observation_outbox_record_v1(
            &draft,
            &ObservationEnvelopeContextV1 {
                runtime_instance_id: "mail-runtime-1".to_owned(),
                runtime_generation: 1,
                module_id: "mail-runtime".to_owned(),
                recorded_at_unix_seconds: 1_782_504_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("record");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let payload =
            v1::CommunicationObservationV1::decode(envelope.payload.as_slice()).expect("payload");

        assert_eq!(payload.body, 4);
        assert_eq!(
            payload.body_blob.expect("body blob").reference_id,
            vec![7; 16]
        );
        assert_eq!(payload.body_admission_failure, 0);
    }

    #[test]
    fn subject_is_bounded_normalized_and_control_free() {
        let draft = new_communication_observation_draft(
            "observation-subject",
            SourceEnvelope {
                provider: ProviderProvenanceV1::MailImap,
                external_record_id: "provider-record-subject".to_owned(),
                scope: None,
            },
            CommunicationEvidenceKindV1::EmailMessage,
            BodyAvailabilityV1::MetadataOnly,
            CommunicationDirectionV1::Incoming,
            Some(1_782_504_000),
        )
        .expect("draft");

        let draft = with_message_subject(draft, Some("  Quarterly update  ".to_owned()))
            .expect("normalized subject");
        assert_eq!(draft.message_subject.as_deref(), Some("Quarterly update"));

        assert_eq!(
            with_message_subject(draft.clone(), Some("line one\nline two".to_owned())),
            Err(IngressDraftError::InvalidMessageSubject)
        );
        assert_eq!(
            with_message_subject(
                draft,
                Some("x".repeat(MAX_MESSAGE_SUBJECT_BYTES.saturating_add(1)))
            ),
            Err(IngressDraftError::InvalidMessageSubject)
        );
    }
}
