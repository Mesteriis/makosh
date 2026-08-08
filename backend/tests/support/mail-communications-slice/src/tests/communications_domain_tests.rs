use makosh_communications_api::{
    CanonicalCommunicationEvidenceKindV1, CommunicationBodyStateV1, CommunicationDirectionV1,
    CommunicationObservationIdV1, CommunicationProviderProvenanceV1, CommunicationSourceCursorV1,
    CommunicationsClientError, RecordCommunicationEvidenceV1,
};
use makosh_communications_domain::{
    CommunicationsDomainError, accept_command, canonicalize_communication,
    convert_client_query_error,
};

fn command(observed_at_unix_seconds: i64) -> RecordCommunicationEvidenceV1 {
    RecordCommunicationEvidenceV1 {
        observation_id: CommunicationObservationIdV1::new([1; 16]),
        causation_message_id: None,
        correlation_id: CommunicationObservationIdV1::new([3; 16]),
        source_cursor: CommunicationSourceCursorV1::new([2; 32]),
        account_cursor: Some(CommunicationSourceCursorV1::new([3; 32])),
        conversation_cursor: Some(CommunicationSourceCursorV1::new([4; 32])),
        participant_cursor: None,
        participant_display_label: None,
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
        observed_at_unix_seconds,
        recorded_at_unix_seconds: observed_at_unix_seconds,
        recorded_at_nanos: 0,
    }
}

#[test]
fn accepts_a_typed_owner_command() {
    let summary = accept_command(command(1)).expect("command accepted");
    assert_eq!(
        summary.evidence_id,
        CommunicationObservationIdV1::new([1; 16])
    );
}

#[test]
fn rejects_invalid_commands_and_maps_errors() {
    assert!(matches!(
        accept_command(command(i64::MAX)),
        Err(CommunicationsDomainError::InvalidObservedTime)
    ));
    assert_eq!(
        convert_client_query_error(CommunicationsDomainError::InvalidObservedTime),
        CommunicationsClientError::DraftValidationFailed
    );
}

#[test]
fn canonicalization_wraps_summary() {
    let summary = accept_command(command(2)).expect("command accepted");
    let canonical = canonicalize_communication(&summary).expect("canonical projection");
    assert_eq!(canonical.summary.evidence_id, summary.evidence_id);
}
