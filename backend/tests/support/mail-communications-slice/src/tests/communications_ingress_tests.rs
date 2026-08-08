use makosh_communications_ingress::{
    BodyAvailabilityV1, CommunicationDirectionV1, CommunicationEvidenceKindV1,
    ObservationEnvelopeContextV1, ProviderProvenanceV1, SourceEnvelope, SourceScopeEnvelope,
    build_observation_outbox_record_v1, new_scoped_communication_observation_draft,
};
use makosh_events_protocol::validation::envelope::decode_envelope_v1;
use prost::Message;

#[test]
fn accepts_a_bounded_typed_observation() {
    let draft = new_scoped_communication_observation_draft(
        "observation-1",
        SourceEnvelope {
            provider: ProviderProvenanceV1::MailImap,
            external_record_id: "source-1".to_owned(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: "account-1".to_owned(),
                external_conversation_id: Some("conversation-1".to_owned()),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        CommunicationEvidenceKindV1::EmailMessage,
        BodyAvailabilityV1::PendingBlob,
        CommunicationDirectionV1::Incoming,
        Some(1),
    )
    .expect("draft");
    assert_eq!(draft.observation_id, "observation-1");
    assert_eq!(draft.source.provider, ProviderProvenanceV1::MailImap);
}

#[test]
fn retains_no_private_payload() {
    let source = SourceEnvelope {
        provider: ProviderProvenanceV1::MailImap,
        external_record_id: "source-2".to_owned(),
        scope: Some(SourceScopeEnvelope {
            external_account_id: "account-1".to_owned(),
            external_conversation_id: Some("conversation-1".to_owned()),
            external_participant_id: None,
            external_media_id: None,
            external_reply_to_record_id: None,
            external_forward_origin_record_id: None,
        }),
    };
    let draft = new_scoped_communication_observation_draft(
        "observation-2",
        source,
        CommunicationEvidenceKindV1::EmailMessage,
        BodyAvailabilityV1::MetadataOnly,
        CommunicationDirectionV1::Unknown,
        None,
    )
    .expect("metadata-only draft");
    assert_eq!(draft.body, BodyAvailabilityV1::MetadataOnly);
}

#[test]
fn builds_a_fenced_exact_envelope_without_provider_locator() {
    let draft = new_scoped_communication_observation_draft(
        "observation-3",
        SourceEnvelope {
            provider: ProviderProvenanceV1::MailImap,
            external_record_id: "private-imap-uid-42".to_owned(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: "account-1".to_owned(),
                external_conversation_id: Some("conversation-1".to_owned()),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        CommunicationEvidenceKindV1::EmailMessage,
        BodyAvailabilityV1::PendingBlob,
        CommunicationDirectionV1::Incoming,
        Some(1),
    )
    .expect("draft");
    let record = build_observation_outbox_record_v1(
        &draft,
        &ObservationEnvelopeContextV1 {
            runtime_instance_id: "mail_runtime_1".to_owned(),
            runtime_generation: 1,
            module_id: "mail-runtime".to_owned(),
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        },
    )
    .expect("outbox record");

    let envelope = decode_envelope_v1(record.exact_bytes()).expect("valid envelope");
    assert_eq!(
        envelope.contract.expect("contract").name,
        "communication_observed"
    );
    assert!(
        !record
            .exact_bytes()
            .windows(b"private-imap-uid-42".len())
            .any(|window| { window == b"private-imap-uid-42" })
    );
    let payload = makosh_communications_ingress::v1::CommunicationObservationV1::decode(
        envelope.payload.as_slice(),
    )
    .expect("typed payload");
    assert_eq!(payload.provider, 1);
    assert_eq!(payload.kind, 1);
    assert_eq!(payload.body, 2);
}
