//! Mail integration implementation helpers for ADR-0239.

use std::collections::HashMap;

use makosh_communications_ingress::{
    AttachmentDescriptorV1, AttachmentDispositionV1, BodyAvailabilityV1, CommunicationDirectionV1,
    CommunicationEvidenceKindV1, CommunicationObservationDraft, ProviderProvenanceV1,
    SourceEnvelope, SourceScopeEnvelope, new_scoped_communication_observation_draft,
    with_attachment_descriptor, with_message_subject, with_participant_display_label,
};
use makosh_mail_api::{
    DEFAULT_WINDOW, MAX_PLAIN_TEXT_BYTES, MAX_WINDOWS, MailContractError::WindowLimitExceeded,
    OutgoingMailV1, SYNC_DEADLINE_SECONDS, WINDOW_DEADLINE_SECONDS, valid_host,
    valid_message_bytes, valid_port, valid_window,
};

pub mod oauth;
pub mod outbound_mime;
pub mod rfc822;

pub use outbound_mime::{
    MAX_OUTBOUND_ATTACHMENT_BYTES, MAX_OUTBOUND_RFC822_BYTES, OutboundAttachmentDispositionV1,
    OutboundAttachmentV1, compose_rfc822_with_attachments,
};

pub use makosh_mail_api::{
    MailConnection, MailConnectionId, MailConnectionState, MailContractError, MailOperation,
    MailOperationId,
};

pub const PACKAGE: &str = "makosh-mail-core";

#[derive(Clone, Debug)]
pub struct SyncPlan {
    pub window: u32,
    pub windows: u32,
    pub total_deadline_seconds: u64,
    pub window_deadline_seconds: u64,
}

impl SyncPlan {
    pub const fn default() -> Self {
        Self {
            window: DEFAULT_WINDOW,
            windows: 1,
            total_deadline_seconds: SYNC_DEADLINE_SECONDS,
            window_deadline_seconds: WINDOW_DEADLINE_SECONDS,
        }
    }

    pub fn bounded(window: u32, windows: u32) -> Option<Self> {
        valid_window(window, windows).then_some(Self {
            window,
            windows,
            total_deadline_seconds: SYNC_DEADLINE_SECONDS,
            window_deadline_seconds: WINDOW_DEADLINE_SECONDS,
        })
    }
}

#[derive(Clone, Debug)]
pub struct MailStatePolicy {
    pub max_sync_windows: u32,
}

impl MailStatePolicy {
    pub const fn new() -> Self {
        Self {
            max_sync_windows: MAX_WINDOWS,
        }
    }
}

impl Default for MailStatePolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct ConnectionTracker {
    operations: HashMap<MailOperationId, MailOperation>,
    status: HashMap<MailConnectionId, MailConnectionState>,
}

impl ConnectionTracker {
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
            status: HashMap::new(),
        }
    }

    pub fn register_connection(&mut self, connection: &MailConnection) {
        self.status
            .insert(connection.id.clone(), MailConnectionState::Provisioning);
    }

    pub fn set_ready(&mut self, connection_id: &str) {
        self.status
            .insert(connection_id.to_string(), MailConnectionState::Ready);
    }

    pub fn set_syncing(&mut self, connection_id: &str, operation: MailOperation) {
        self.status
            .insert(connection_id.to_string(), MailConnectionState::Syncing);
        self.operations
            .insert(operation.operation_id.clone(), operation);
    }

    pub fn set_degraded(&mut self, connection_id: &str) {
        self.status
            .insert(connection_id.to_string(), MailConnectionState::Degraded);
    }

    pub fn set_retired(&mut self, connection_id: &str) {
        self.status
            .insert(connection_id.to_string(), MailConnectionState::Retired);
    }

    pub fn operation_status(&self, operation_id: &str) -> Option<&MailOperation> {
        self.operations.get(operation_id)
    }

    pub fn status_of(&self, connection_id: &str) -> Option<MailConnectionState> {
        self.status.get(connection_id).copied()
    }
}

pub fn validate_sync_request(
    host: &str,
    port: u16,
    body_bytes: usize,
) -> Result<(), MailContractError> {
    if !valid_host(host) {
        return Err(MailContractError::InvalidHost);
    }
    if !valid_port(port) {
        return Err(MailContractError::InvalidPort);
    }
    if body_bytes > MAX_PLAIN_TEXT_BYTES {
        return Err(MailContractError::InvalidPayload);
    }
    if !valid_message_bytes(body_bytes) {
        return Err(MailContractError::InvalidPayload);
    }
    Ok(())
}

pub fn compose_rfc822(
    from_address: &str,
    message: &OutgoingMailV1,
) -> Result<String, MailContractError> {
    outbound_mime::validate_message(from_address, message)?;
    Ok(outbound_mime::plain_text_message(from_address, message))
}

pub fn draft_delivery_observation(
    provider: ProviderProvenanceV1,
    message: &OutgoingMailV1,
) -> Result<CommunicationObservationDraft, MailContractError> {
    if message.operation_id.trim().is_empty()
        || message.connection_id.trim().is_empty()
        || message.provider_conversation_id.trim().is_empty()
    {
        return Err(MailContractError::InvalidPayload);
    }
    new_scoped_communication_observation_draft(
        format!(
            "{}:{}:{}",
            provider.as_str(),
            message.connection_id,
            message.operation_id
        ),
        SourceEnvelope {
            provider,
            external_record_id: format!(
                "{}:{}:{}",
                provider.as_str(),
                message.connection_id,
                message.operation_id
            ),
            scope: Some(SourceScopeEnvelope {
                external_account_id: message.connection_id.clone(),
                external_conversation_id: Some(message.provider_conversation_id.clone()),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        CommunicationEvidenceKindV1::EmailMessage,
        BodyAvailabilityV1::Unavailable,
        CommunicationDirectionV1::Outgoing,
        None,
    )
    .map_err(|_| MailContractError::InvalidPayload)
}

pub fn draft_ingress_observation(
    operation_id: &str,
    provider: ProviderProvenanceV1,
    account_id: impl Into<String>,
    source_id: impl Into<String>,
    body_bytes: usize,
) -> Result<CommunicationObservationDraft, MailContractError> {
    if body_bytes > MAX_PLAIN_TEXT_BYTES {
        return Err(MailContractError::InvalidPayload);
    }
    let source_id = source_id.into();
    new_scoped_communication_observation_draft(
        operation_id,
        SourceEnvelope {
            provider,
            external_record_id: source_id.clone(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: account_id.into(),
                external_conversation_id: Some(source_id),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        CommunicationEvidenceKindV1::EmailMessage,
        if body_bytes > 0 {
            BodyAvailabilityV1::Unavailable
        } else {
            BodyAvailabilityV1::MetadataOnly
        },
        CommunicationDirectionV1::Incoming,
        None,
    )
    .map_err(|_| MailContractError::InvalidPayload)
}

pub fn draft_ingress_observation_with_body(
    operation_id: &str,
    provider: ProviderProvenanceV1,
    account_id: impl Into<String>,
    source_id: impl Into<String>,
    body: BodyAvailabilityV1,
) -> Result<CommunicationObservationDraft, MailContractError> {
    let source_id = source_id.into();
    new_scoped_communication_observation_draft(
        operation_id,
        SourceEnvelope {
            provider,
            external_record_id: source_id.clone(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: account_id.into(),
                external_conversation_id: Some(source_id),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        CommunicationEvidenceKindV1::EmailMessage,
        body,
        CommunicationDirectionV1::Incoming,
        None,
    )
    .map_err(|_| MailContractError::InvalidPayload)
}

pub fn draft_ingress_observation_with_sender_subject_body(
    operation_id: &str,
    provider: ProviderProvenanceV1,
    account_id: impl Into<String>,
    source_id: impl Into<String>,
    sender: Option<String>,
    subject: Option<String>,
    body: BodyAvailabilityV1,
) -> Result<CommunicationObservationDraft, MailContractError> {
    let source_id = source_id.into();
    let sender = sender.map(normalize_mail_sender).transpose()?;
    let draft = new_scoped_communication_observation_draft(
        operation_id,
        SourceEnvelope {
            provider,
            external_record_id: source_id.clone(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: account_id.into(),
                external_conversation_id: Some(source_id),
                external_participant_id: sender.as_ref().map(|sender| sender.external_id.clone()),
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        CommunicationEvidenceKindV1::EmailMessage,
        body,
        CommunicationDirectionV1::Incoming,
        None,
    )
    .map_err(|_| MailContractError::InvalidPayload)?;
    let draft = with_participant_display_label(draft, sender.map(|sender| sender.display_label))
        .map_err(|_| MailContractError::InvalidPayload)?;
    with_message_subject(draft, subject).map_err(|_| MailContractError::InvalidPayload)
}

struct MailSenderMetadataV1 {
    external_id: String,
    display_label: String,
}

fn normalize_mail_sender(value: String) -> Result<MailSenderMetadataV1, MailContractError> {
    let display_label = value.trim();
    if display_label.is_empty()
        || display_label.len() > 256
        || display_label.chars().any(char::is_control)
    {
        return Err(MailContractError::InvalidPayload);
    }
    let external_id = display_label
        .rfind('<')
        .and_then(|start| display_label[start + 1..].strip_suffix('>').map(str::trim))
        .filter(|value| !value.is_empty())
        .unwrap_or(display_label)
        .to_lowercase();
    Ok(MailSenderMetadataV1 {
        external_id,
        display_label: display_label.to_owned(),
    })
}

pub struct MailAttachmentIngressRequestV1 {
    pub provider: ProviderProvenanceV1,
    pub account_id: String,
    pub message_source_id: String,
    pub media_id: String,
    pub filename: Option<String>,
    pub media_type: String,
    pub declared_bytes: u64,
    pub disposition: AttachmentDispositionV1,
}

pub fn draft_attachment_ingress_observation(
    operation_id: &str,
    request: MailAttachmentIngressRequestV1,
) -> Result<CommunicationObservationDraft, MailContractError> {
    let draft = new_scoped_communication_observation_draft(
        operation_id,
        SourceEnvelope {
            provider: request.provider,
            external_record_id: request.message_source_id.clone(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: request.account_id,
                external_conversation_id: Some(request.message_source_id),
                external_participant_id: None,
                external_media_id: Some(request.media_id),
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        CommunicationEvidenceKindV1::MediaChanged,
        BodyAvailabilityV1::MetadataOnly,
        CommunicationDirectionV1::Incoming,
        None,
    )
    .map_err(|_| MailContractError::InvalidPayload)?;
    with_attachment_descriptor(
        draft,
        AttachmentDescriptorV1 {
            filename: request.filename,
            media_type: request.media_type,
            declared_bytes: request.declared_bytes,
            sha256: None,
            disposition: request.disposition,
        },
    )
    .map_err(|_| MailContractError::InvalidPayload)
}

pub fn bounded_window(window: u32, windows: u32) -> Result<SyncPlan, MailContractError> {
    SyncPlan::bounded(window, windows).ok_or(WindowLimitExceeded)
}

pub mod constants {
    pub use makosh_mail_api::MAX_WINDOWS;
}

#[cfg(test)]
mod rfc822_composition_tests {
    use super::*;
    use makosh_mail_api::OutgoingMailV1;

    fn message() -> OutgoingMailV1 {
        OutgoingMailV1 {
            operation_id: "operation".to_owned(),
            connection_id: "connection".to_owned(),
            provider_conversation_id: "thread-1".to_owned(),
            recipients: vec!["recipient@example.test".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "Report".to_owned(),
            text_body: "line one\nline two".to_owned(),
        }
    }

    #[test]
    fn composes_bounded_plain_text_rfc822_message() {
        let mut message = message();
        message.cc_recipients = vec!["cc@example.test".to_owned()];
        message.bcc_recipients = vec!["private@example.test".to_owned()];
        let rendered = compose_rfc822("owner@example.test", &message).expect("valid message");
        assert!(rendered.starts_with(
            "From: owner@example.test\r\nTo: recipient@example.test\r\nCc: cc@example.test\r\nSubject: Report\r\n"
        ));
        assert!(!rendered.contains("private@example.test"));
        assert!(!rendered.contains("\r\nBcc:"));
        assert!(rendered.ends_with("\r\n\r\nline one\r\nline two"));
    }

    #[test]
    fn rejects_header_injection_in_subject_and_recipients() {
        let mut subject_injection = message();
        subject_injection.subject = "Report\r\nBcc: attacker@example.test".to_owned();
        assert_eq!(
            compose_rfc822("owner@example.test", &subject_injection),
            Err(MailContractError::InvalidPayload)
        );

        let mut recipient_injection = message();
        recipient_injection.recipients =
            vec!["recipient@example.test\r\nBcc: attacker@example.test".to_owned()];
        assert_eq!(
            compose_rfc822("owner@example.test", &recipient_injection),
            Err(MailContractError::InvalidPayload)
        );
    }

    #[test]
    fn delivery_observation_is_outgoing_and_provider_scoped() {
        let draft = draft_delivery_observation(ProviderProvenanceV1::MailSmtp, &message())
            .expect("valid delivery observation");
        assert_eq!(draft.source.provider, ProviderProvenanceV1::MailSmtp);
        assert_eq!(draft.direction, CommunicationDirectionV1::Outgoing);
        assert_eq!(
            draft
                .source
                .scope
                .as_ref()
                .and_then(|scope| scope.external_conversation_id.as_deref()),
            Some("thread-1")
        );
    }

    #[test]
    fn inbound_observation_preserves_sender_identity_and_subject() {
        let draft = draft_ingress_observation_with_sender_subject_body(
            "operation-inbound",
            ProviderProvenanceV1::MailImap,
            "account-1",
            "message-1",
            Some("Alice Example <Alice@Example.test>".to_owned()),
            Some("Quarterly update".to_owned()),
            BodyAvailabilityV1::MetadataOnly,
        )
        .expect("valid inbound observation");

        let scope = draft.source.scope.expect("provider scope");
        assert_eq!(
            scope.external_participant_id.as_deref(),
            Some("alice@example.test")
        );
        assert_eq!(
            draft.participant_display_label.as_deref(),
            Some("Alice Example <Alice@Example.test>")
        );
        assert_eq!(draft.message_subject.as_deref(), Some("Quarterly update"));
    }
}
