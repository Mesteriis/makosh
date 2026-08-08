//! Zulip-specific anti-corruption mapper. No transport or domain implementation dependency.

use makosh_communications_ingress::{
    AttachmentDescriptorV1, AttachmentDispositionV1, BodyAvailabilityV1, CommunicationDirectionV1,
    CommunicationEvidenceKindV1, CommunicationObservationDraft, ProviderProvenanceV1,
    SourceEnvelope, SourceScopeEnvelope, new_scoped_communication_observation_draft,
};
use makosh_vault_protocol::{
    DEFAULT_LEASE_TTL_SECONDS, SecretClassV1, VaultActionV1, VaultPurposeRequestV1,
};
use makosh_zulip_api::{ZulipAttachmentV1, ZulipEventV1};

pub const PACKAGE: &str = "makosh-zulip-core";
pub const ZULIP_API_KEY_PURPOSE_ID: &str = "zulip_api_key";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipCoreError {
    InvalidEvent,
    CredentialLeaseRejected,
}

#[cfg(test)]
mod attachment_tests {
    use makosh_communications_ingress::CommunicationEvidenceKindV1;
    use makosh_zulip_api::{ZulipAttachmentV1, ZulipEventV1, operational::ZulipConversationKindV1};

    use super::observation_drafts;

    #[test]
    fn emits_separate_metadata_only_media_observation() {
        let drafts = observation_drafts(&ZulipEventV1::Message {
            account_id: "account".into(),
            event_id: 1,
            provider_message_id: "message".into(),
            provider_conversation_id: "stream:1:topic".into(),
            conversation_kind: ZulipConversationKindV1::StreamTopic,
            stream_id: Some("1".into()),
            stream_name: None,
            topic: Some("topic".into()),
            direct_recipient_id: None,
            sender_id: "sender".into(),
            is_outgoing: false,
            content: None,
            sent_at_unix_seconds: None,
            attachments: vec![ZulipAttachmentV1 {
                provider_attachment_id: "a/report.pdf".into(),
                filename: Some("report.pdf".into()),
            }],
            reactions: Vec::new(),
        })
        .expect("drafts");
        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[1].kind, CommunicationEvidenceKindV1::MediaChanged);
        assert_eq!(
            drafts[1]
                .attachment_descriptor
                .as_ref()
                .expect("attachment")
                .media_type,
            "application/octet-stream"
        );
    }
}

pub fn credential_lease_purpose(
    account_id: &str,
    configuration_instance_id: &str,
) -> Result<VaultPurposeRequestV1, ZulipCoreError> {
    if account_id.trim().is_empty() || configuration_instance_id.trim().is_empty() {
        return Err(ZulipCoreError::CredentialLeaseRejected);
    }
    VaultPurposeRequestV1::new(
        ZULIP_API_KEY_PURPOSE_ID.to_owned(),
        configuration_instance_id.to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::Resolve],
        DEFAULT_LEASE_TTL_SECONDS,
    )
    .map_err(|_| ZulipCoreError::CredentialLeaseRejected)
}

pub fn observation_draft(
    event: &ZulipEventV1,
) -> Result<CommunicationObservationDraft, ZulipCoreError> {
    observation_drafts(event)?
        .into_iter()
        .next()
        .ok_or(ZulipCoreError::InvalidEvent)
}

pub fn observation_drafts(
    event: &ZulipEventV1,
) -> Result<Vec<CommunicationObservationDraft>, ZulipCoreError> {
    let body = matches!(event, ZulipEventV1::Message { content: Some(content), .. } if !content.trim().is_empty())
        .then_some(BodyAvailabilityV1::Unavailable)
        .unwrap_or(BodyAvailabilityV1::MetadataOnly);
    let observed = match event {
        ZulipEventV1::Message {
            account_id,
            event_id,
            provider_message_id,
            provider_conversation_id,
            sender_id,
            is_outgoing,
            ..
        } => ObservedEvent {
            account_id,
            event_id: *event_id,
            record_id: provider_message_id,
            conversation_id: Some(provider_conversation_id),
            participant_id: Some(sender_id),
            kind: CommunicationEvidenceKindV1::ChatMessage,
            direction: if *is_outgoing {
                CommunicationDirectionV1::Outgoing
            } else {
                CommunicationDirectionV1::Incoming
            },
        },
        ZulipEventV1::MessageUpdated {
            account_id,
            event_id,
            provider_message_id,
            ..
        } => ObservedEvent::change(
            account_id,
            *event_id,
            provider_message_id,
            CommunicationEvidenceKindV1::MessageEdited,
        ),
        ZulipEventV1::MessageDeleted {
            account_id,
            event_id,
            provider_message_id,
        } => ObservedEvent::change(
            account_id,
            *event_id,
            provider_message_id,
            CommunicationEvidenceKindV1::MessageDeleted,
        ),
        ZulipEventV1::ReactionChanged {
            account_id,
            event_id,
            provider_message_id,
            actor_id,
            ..
        } => ObservedEvent {
            account_id,
            event_id: *event_id,
            record_id: provider_message_id,
            conversation_id: None,
            participant_id: Some(actor_id),
            kind: CommunicationEvidenceKindV1::ReactionChanged,
            direction: CommunicationDirectionV1::Unknown,
        },
    };
    if observed.account_id.trim().is_empty()
        || observed.record_id.trim().is_empty()
        || observed.event_id <= 0
    {
        return Err(ZulipCoreError::InvalidEvent);
    }
    let message = new_scoped_communication_observation_draft(
        format!("zulip:{}:{}", observed.account_id, observed.event_id),
        SourceEnvelope {
            provider: ProviderProvenanceV1::Zulip,
            external_record_id: observed.record_id.clone(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: observed.account_id.clone(),
                external_conversation_id: observed.conversation_id.cloned(),
                external_participant_id: observed.participant_id.cloned(),
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        observed.kind,
        body,
        observed.direction,
        None,
    )
    .map_err(|_| ZulipCoreError::InvalidEvent)?;
    let mut drafts = vec![message];
    if let ZulipEventV1::Message {
        account_id,
        event_id,
        provider_conversation_id,
        attachments,
        ..
    } = event
    {
        for (index, attachment) in attachments.iter().enumerate() {
            drafts.push(attachment_draft(
                account_id,
                *event_id,
                provider_conversation_id,
                index,
                attachment,
            )?);
        }
    }
    Ok(drafts)
}

fn attachment_draft(
    account_id: &str,
    event_id: i64,
    conversation_id: &str,
    index: usize,
    attachment: &ZulipAttachmentV1,
) -> Result<CommunicationObservationDraft, ZulipCoreError> {
    if attachment.provider_attachment_id.trim().is_empty() {
        return Err(ZulipCoreError::InvalidEvent);
    }
    let draft = new_scoped_communication_observation_draft(
        format!("zulip:{account_id}:{event_id}:attachment:{index}"),
        SourceEnvelope {
            provider: ProviderProvenanceV1::Zulip,
            external_record_id: attachment.provider_attachment_id.clone(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: account_id.to_owned(),
                external_conversation_id: Some(conversation_id.to_owned()),
                external_participant_id: None,
                external_media_id: Some(attachment.provider_attachment_id.clone()),
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        CommunicationEvidenceKindV1::MediaChanged,
        BodyAvailabilityV1::MetadataOnly,
        CommunicationDirectionV1::Unknown,
        None,
    )
    .map_err(|_| ZulipCoreError::InvalidEvent)?;
    let filename = attachment
        .filename
        .as_ref()
        .filter(|value| !value.is_empty() && value.is_ascii())
        .cloned();
    makosh_communications_ingress::with_attachment_descriptor(
        draft,
        AttachmentDescriptorV1 {
            filename,
            media_type: "application/octet-stream".to_owned(),
            declared_bytes: 0,
            sha256: None,
            disposition: AttachmentDispositionV1::Unknown,
        },
    )
    .map_err(|_| ZulipCoreError::InvalidEvent)
}

struct ObservedEvent<'a> {
    account_id: &'a String,
    event_id: i64,
    record_id: &'a String,
    conversation_id: Option<&'a String>,
    participant_id: Option<&'a String>,
    kind: CommunicationEvidenceKindV1,
    direction: CommunicationDirectionV1,
}

impl<'a> ObservedEvent<'a> {
    fn change(
        account_id: &'a String,
        event_id: i64,
        record_id: &'a String,
        kind: CommunicationEvidenceKindV1,
    ) -> Self {
        Self {
            account_id,
            event_id,
            record_id,
            conversation_id: None,
            participant_id: None,
            kind,
            direction: CommunicationDirectionV1::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_purpose_is_stable_and_configuration_scoped() {
        let purpose =
            credential_lease_purpose("account-1", "configuration-1").expect("credential purpose");

        assert_eq!(purpose.purpose_id(), ZULIP_API_KEY_PURPOSE_ID);
        assert_eq!(purpose.configuration_instance_id(), "configuration-1");
    }

    #[test]
    fn maps_zulip_message_without_content() {
        let draft = observation_draft(&ZulipEventV1::Message {
            account_id: "a".into(),
            event_id: 1,
            provider_message_id: "m".into(),
            provider_conversation_id: "s:t".into(),
            conversation_kind: makosh_zulip_api::operational::ZulipConversationKindV1::StreamTopic,
            stream_id: Some("1".into()),
            stream_name: None,
            topic: Some("t".into()),
            direct_recipient_id: None,
            sender_id: "u".into(),
            is_outgoing: false,
            content: None,
            sent_at_unix_seconds: None,
            attachments: Vec::new(),
            reactions: Vec::new(),
        })
        .expect("draft");

        assert_eq!(draft.source.provider, ProviderProvenanceV1::Zulip);
        assert_eq!(draft.direction, CommunicationDirectionV1::Incoming);
        assert_eq!(draft.body, BodyAvailabilityV1::MetadataOnly);
    }

    #[test]
    fn marks_provider_content_for_runtime_body_admission() {
        let draft = observation_draft(&ZulipEventV1::Message {
            account_id: "a".into(),
            event_id: 2,
            provider_message_id: "m".into(),
            provider_conversation_id: "s:t".into(),
            conversation_kind: makosh_zulip_api::operational::ZulipConversationKindV1::StreamTopic,
            stream_id: Some("1".into()),
            stream_name: None,
            topic: Some("t".into()),
            direct_recipient_id: None,
            sender_id: "u".into(),
            is_outgoing: false,
            content: Some("provider markdown body".into()),
            sent_at_unix_seconds: None,
            attachments: Vec::new(),
            reactions: Vec::new(),
        })
        .expect("draft");

        assert_eq!(draft.body, BodyAvailabilityV1::Unavailable);
    }
}
