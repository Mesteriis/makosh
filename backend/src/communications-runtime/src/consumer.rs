use hermes_communications_api::{
    AttachmentDescriptorV1, AttachmentDispositionV1, CanonicalCommunicationEvidenceKindV1,
    CommunicationBodyAdmissionFailureV1, CommunicationBodyStateV1, CommunicationDirectionV1,
    CommunicationObservationIdV1, CommunicationProviderProvenanceV1, CommunicationSourceCursorV1,
    RecordCommunicationEvidenceV1,
};
use hermes_communications_domain::{
    COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1, accept_command, canonicalize_communication,
    decide_search_index_v1,
};
use hermes_communications_ingress::{
    BodyAvailabilityV1, CommunicationDirectionV1 as IngressDirectionV1,
    CommunicationEvidenceKindV1, ProviderProvenanceV1,
    admission::{
        communication_observed_contract_reference_v1,
        communication_observed_prior_contract_reference_v1,
    },
    v1::CommunicationObservationV1,
};
use hermes_communications_persistence::{
    CommunicationsConsumeOutcomeV1, CommunicationsDurablePersistence,
    CommunicationsPersistenceError, PendingCommunicationsBodyCustodyTransferV1,
    PersistedCommunicationsObservationV1,
};
use hermes_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use hermes_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{DurableEnvelopeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use prost::Message;

use crate::{
    canonical_outbox::{
        CanonicalEventContextV1, build_attachment_anchor_recorded_outbox_v1,
        build_evidence_recorded_outbox_v1,
    },
    search_job::derived_index_work_from_decision_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsEventConsumeErrorV1 {
    InvalidEnvelope,
    WrongContract,
    InvalidPayload,
    DomainRejected,
    PersistenceRejected,
}

/// Consumes one already authorized Event Hub delivery. The caller supplies a
/// permit derived by Kernel, so this runtime cannot create or widen a
/// subscription by choosing a subject or budget itself.
pub async fn consume_next_observation_v1(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    canonical_event_context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsDeliveryErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(delivery_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationsDeliveryErrorV1::InvalidEnvelope)?;
    let outcome =
        consume_communication_observation_durable_v1(persistence, &record, canonical_event_context)
            .await
            .map_err(CommunicationsDeliveryErrorV1::Consume)?;
    delivery.acknowledge().await.map_err(delivery_error)?;
    Ok(outcome)
}

fn delivery_error(_: RuntimePullDeliveryErrorV1) -> CommunicationsDeliveryErrorV1 {
    CommunicationsDeliveryErrorV1::Unavailable
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDeliveryErrorV1 {
    Unavailable,
    InvalidEnvelope,
    Consume(CommunicationsEventConsumeErrorV1),
}

pub async fn consume_communication_observation_durable_v1(
    persistence: &CommunicationsDurablePersistence,
    record: &OutboxRecordV1,
    canonical_event_context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsEventConsumeErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationsEventConsumeErrorV1::InvalidEnvelope)?;
    let decoded = command_from_envelope(&envelope)?;
    let summary = accept_command(decoded.command)
        .map_err(|_| CommunicationsEventConsumeErrorV1::DomainRejected)?;
    let projection = canonicalize_communication(&summary)
        .map_err(|_| CommunicationsEventConsumeErrorV1::DomainRejected)?;
    let causation_message_id: [u8; 16] = envelope
        .message_id
        .as_slice()
        .try_into()
        .map_err(|_| CommunicationsEventConsumeErrorV1::WrongContract)?;
    let canonical_outbox_record =
        build_evidence_recorded_outbox_v1(&summary, causation_message_id, canonical_event_context)
            .map_err(|_| CommunicationsEventConsumeErrorV1::DomainRejected)?;
    let attachment_anchor_outbox_record = projection
        .attachment_anchor
        .as_ref()
        .map(|anchor| {
            build_attachment_anchor_recorded_outbox_v1(
                anchor,
                summary.observation_id.bytes(),
                causation_message_id,
                summary.correlation_id.bytes(),
                canonical_event_context,
            )
        })
        .transpose()
        .map_err(|_| CommunicationsEventConsumeErrorV1::DomainRejected)?;
    let derived_index_work = derived_index_work_from_decision_v1(
        decide_search_index_v1(&projection, COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1),
        canonical_event_context.recorded_at_unix_seconds,
    );
    let derived_index_job = derived_index_work
        .as_ref()
        .and_then(|work| work.job.as_ref());
    let derived_index_failure = derived_index_work
        .as_ref()
        .and_then(|work| work.failure.as_ref());
    persistence
        .persist_consumed_observation(
            record,
            PersistedCommunicationsObservationV1 {
                projection,
                pending_custody_transfer: decoded
                    .source_body
                    .map(|source| PendingCommunicationsBodyCustodyTransferV1 {
                        evidence_id: summary.evidence_id,
                        envelope_sha256: *record.envelope_sha256(),
                        source_blob_ref: source.blob_ref,
                        source_reference_id: source.reference_id,
                        declared_bytes: source.declared_bytes,
                        plaintext_sha256: source.sha256,
                        source_custody_proof: source.custody_transfer_source_proof,
                    })
                    .as_ref(),
                derived_index_job,
                derived_index_failure,
                canonical_outbox_record: &canonical_outbox_record,
                attachment_anchor_outbox_record: attachment_anchor_outbox_record.as_ref(),
                created_at_unix_seconds: canonical_event_context.recorded_at_unix_seconds,
            },
        )
        .await
        .map_err(persistence_error)
}

struct DecodedCommunicationObservationV1 {
    command: RecordCommunicationEvidenceV1,
    source_body: Option<SourceBodyCustodyReceiptV1>,
}

struct SourceBodyCustodyReceiptV1 {
    blob_ref: String,
    reference_id: [u8; 16],
    declared_bytes: u64,
    sha256: [u8; 32],
    custody_transfer_source_proof: Vec<u8>,
    media_type: String,
}

fn command_from_envelope(
    envelope: &DurableEnvelopeV1,
) -> Result<DecodedCommunicationObservationV1, CommunicationsEventConsumeErrorV1> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(CommunicationsEventConsumeErrorV1::WrongContract)?;
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else {
        return Err(CommunicationsEventConsumeErrorV1::WrongContract);
    };
    let expected_contract = communication_observed_contract_reference_v1();
    let prior_contract = communication_observed_prior_contract_reference_v1();
    let mut transitional_contract = expected_contract.clone();
    transitional_contract.revision = prior_contract.revision;
    let is_current_contract = contract.owner == expected_contract.owner
        && contract.name == expected_contract.name
        && contract.major == expected_contract.major
        && contract.revision == expected_contract.revision
        && contract.schema_sha256 == expected_contract.schema_sha256;
    let is_prior_contract = contract.owner == prior_contract.owner
        && contract.name == prior_contract.name
        && contract.major == prior_contract.major
        && contract.revision == prior_contract.revision
        && contract.schema_sha256 == prior_contract.schema_sha256;
    let is_transitional_contract = contract.owner == transitional_contract.owner
        && contract.name == transitional_contract.name
        && contract.major == transitional_contract.major
        && contract.revision == transitional_contract.revision
        && contract.schema_sha256 == transitional_contract.schema_sha256;
    if (!is_current_contract && !is_prior_contract && !is_transitional_contract)
        || metadata.observation_id != envelope.message_id
        || metadata.source_cursor_sha256.len() != 32
    {
        return Err(CommunicationsEventConsumeErrorV1::WrongContract);
    }
    let payload = CommunicationObservationV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationsEventConsumeErrorV1::InvalidPayload)?;
    let observed_body = body_from_wire(payload.body)?;
    let source_body = source_body_from_wire(payload.body_blob, is_prior_contract)?;
    let (body, body_blob) = match (observed_body, source_body.as_ref()) {
        (BodyAvailabilityV1::AdmittedBlob, Some(_)) => {
            (CommunicationBodyStateV1::PendingBlob, None)
        }
        (BodyAvailabilityV1::AdmittedBlob, None) | (_, Some(_)) => {
            return Err(CommunicationsEventConsumeErrorV1::InvalidPayload);
        }
        (body, None) => (canonical_body(body), None),
    };
    let body_admission_failure = body_admission_failure_from_wire(payload.body_admission_failure)?;
    Ok(DecodedCommunicationObservationV1 {
        command: RecordCommunicationEvidenceV1 {
            observation_id: CommunicationObservationIdV1::new(id16(&metadata.observation_id)?),
            causation_message_id: optional_id16(&envelope.causation_message_id)?,
            correlation_id: CommunicationObservationIdV1::new(id16(&envelope.correlation_id)?),
            source_cursor: CommunicationSourceCursorV1::new(id32(&metadata.source_cursor_sha256)?),
            account_cursor: optional_cursor(&payload.account_cursor_sha256)?,
            conversation_cursor: optional_cursor(&payload.conversation_cursor_sha256)?,
            participant_cursor: optional_cursor(&payload.participant_cursor_sha256)?,
            participant_display_label: participant_display_label_from_wire(
                payload.participant_display_label,
            )?,
            message_subject: message_subject_from_wire(payload.message_subject)?,
            media_cursor: optional_cursor(&payload.media_cursor_sha256)?,
            reply_to_source_cursor: optional_cursor(&payload.reply_to_source_cursor_sha256)?,
            forward_origin_source_cursor: optional_cursor(
                &payload.forward_origin_source_cursor_sha256,
            )?,
            provider: canonical_provider(provider_from_wire(payload.provider)?),
            direction: canonical_direction(direction_from_wire(payload.direction)?),
            kind: canonical_kind(kind_from_wire(payload.kind)?),
            body,
            body_blob,
            body_media_type: source_body.as_ref().map(|value| value.media_type.clone()),
            body_admission_failure,
            attachment_descriptor: attachment_descriptor_from_wire(payload.attachment_descriptor)?,
            observed_at_unix_seconds: metadata
                .observed_at
                .as_ref()
                .ok_or(CommunicationsEventConsumeErrorV1::WrongContract)?
                .seconds,
            recorded_at_unix_seconds: envelope
                .recorded_at
                .as_ref()
                .ok_or(CommunicationsEventConsumeErrorV1::WrongContract)?
                .seconds,
            recorded_at_nanos: envelope
                .recorded_at
                .as_ref()
                .ok_or(CommunicationsEventConsumeErrorV1::WrongContract)?
                .nanos,
        },
        source_body,
    })
}

fn optional_id16(
    value: &[u8],
) -> Result<Option<CommunicationObservationIdV1>, CommunicationsEventConsumeErrorV1> {
    if value.is_empty() {
        Ok(None)
    } else {
        id16(value).map(CommunicationObservationIdV1::new).map(Some)
    }
}
fn provider_from_wire(
    value: i32,
) -> Result<ProviderProvenanceV1, CommunicationsEventConsumeErrorV1> {
    match value {
        1 => Ok(ProviderProvenanceV1::MailImap),
        2 => Ok(ProviderProvenanceV1::Telegram),
        3 => Ok(ProviderProvenanceV1::WhatsAppWeb),
        4 => Ok(ProviderProvenanceV1::MailSmtp),
        5 => Ok(ProviderProvenanceV1::Zulip),
        6 => Ok(ProviderProvenanceV1::MailGmail),
        _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
    }
}
fn direction_from_wire(
    value: i32,
) -> Result<IngressDirectionV1, CommunicationsEventConsumeErrorV1> {
    match value {
        1 => Ok(IngressDirectionV1::Incoming),
        2 => Ok(IngressDirectionV1::Outgoing),
        3 => Ok(IngressDirectionV1::Unknown),
        _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
    }
}
fn kind_from_wire(
    value: i32,
) -> Result<CommunicationEvidenceKindV1, CommunicationsEventConsumeErrorV1> {
    match value {
        1 => Ok(CommunicationEvidenceKindV1::EmailMessage),
        2 => Ok(CommunicationEvidenceKindV1::ChatMessage),
        3 => Ok(CommunicationEvidenceKindV1::MessageEdited),
        4 => Ok(CommunicationEvidenceKindV1::MessageDeleted),
        5 => Ok(CommunicationEvidenceKindV1::ReactionChanged),
        6 => Ok(CommunicationEvidenceKindV1::DeliveryStateChanged),
        7 => Ok(CommunicationEvidenceKindV1::ConversationStateChanged),
        8 => Ok(CommunicationEvidenceKindV1::ParticipantChanged),
        9 => Ok(CommunicationEvidenceKindV1::MediaChanged),
        10 => Ok(CommunicationEvidenceKindV1::TopicChanged),
        11 => Ok(CommunicationEvidenceKindV1::TypingChanged),
        _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
    }
}
fn body_from_wire(value: i32) -> Result<BodyAvailabilityV1, CommunicationsEventConsumeErrorV1> {
    match value {
        1 => Ok(BodyAvailabilityV1::MetadataOnly),
        2 => Ok(BodyAvailabilityV1::PendingBlob),
        3 => Ok(BodyAvailabilityV1::Unavailable),
        4 => Ok(BodyAvailabilityV1::AdmittedBlob),
        _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
    }
}
fn source_body_from_wire(
    value: Option<hermes_communications_ingress::v1::BodyBlobReceiptV1>,
    is_prior_contract: bool,
) -> Result<Option<SourceBodyCustodyReceiptV1>, CommunicationsEventConsumeErrorV1> {
    let Some(value) = value else { return Ok(None) };
    let reference_id = id16(&value.reference_id)?;
    let sha256 = id32(&value.sha256)?;
    let media_type = match value.media_type.as_str() {
        "text/plain" | "text/html" => value.media_type,
        "" if is_prior_contract => "text/plain".to_owned(),
        _ => return Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
    };
    if value.blob_ref.trim().is_empty()
        || value.blob_ref.len() > 512
        || !value.blob_ref.is_ascii()
        || reference_id.iter().all(|byte| *byte == 0)
        || !(1..=64 * 1024 * 1024).contains(&value.declared_bytes)
        || !(1..=2_048).contains(&value.custody_transfer_source_proof.len())
    {
        return Err(CommunicationsEventConsumeErrorV1::InvalidPayload);
    }
    Ok(Some(SourceBodyCustodyReceiptV1 {
        blob_ref: value.blob_ref,
        reference_id,
        declared_bytes: value.declared_bytes,
        sha256,
        custody_transfer_source_proof: value.custody_transfer_source_proof,
        media_type,
    }))
}
fn body_admission_failure_from_wire(
    value: i32,
) -> Result<Option<CommunicationBodyAdmissionFailureV1>, CommunicationsEventConsumeErrorV1> {
    match value {
        0 => Ok(None),
        1 => Ok(Some(CommunicationBodyAdmissionFailureV1::SourceUnavailable)),
        2 => Ok(Some(CommunicationBodyAdmissionFailureV1::SizeLimitExceeded)),
        3 => Ok(Some(CommunicationBodyAdmissionFailureV1::IntegrityMismatch)),
        4 => Ok(Some(CommunicationBodyAdmissionFailureV1::PolicyRejected)),
        _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
    }
}
fn attachment_descriptor_from_wire(
    value: Option<hermes_communications_ingress::v1::AttachmentDescriptorV1>,
) -> Result<Option<AttachmentDescriptorV1>, CommunicationsEventConsumeErrorV1> {
    let Some(value) = value else { return Ok(None) };
    let disposition = match value.disposition {
        1 => AttachmentDispositionV1::Attachment,
        2 => AttachmentDispositionV1::Inline,
        3 => AttachmentDispositionV1::Unknown,
        _ => return Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
    };
    let sha256 = if value.sha256.is_empty() {
        None
    } else {
        Some(id32(&value.sha256)?)
    };
    AttachmentDescriptorV1::new(
        (!value.filename.is_empty()).then_some(value.filename),
        value.media_type,
        value.declared_bytes,
        sha256,
        disposition,
    )
    .map(Some)
    .map_err(|_| CommunicationsEventConsumeErrorV1::InvalidPayload)
}
const fn canonical_provider(value: ProviderProvenanceV1) -> CommunicationProviderProvenanceV1 {
    match value {
        ProviderProvenanceV1::MailImap => CommunicationProviderProvenanceV1::MailImap,
        ProviderProvenanceV1::Telegram => CommunicationProviderProvenanceV1::Telegram,
        ProviderProvenanceV1::WhatsAppWeb => CommunicationProviderProvenanceV1::WhatsAppWeb,
        ProviderProvenanceV1::MailSmtp => CommunicationProviderProvenanceV1::MailSmtp,
        ProviderProvenanceV1::Zulip => CommunicationProviderProvenanceV1::Zulip,
        ProviderProvenanceV1::MailGmail => CommunicationProviderProvenanceV1::MailGmail,
    }
}
const fn canonical_direction(value: IngressDirectionV1) -> CommunicationDirectionV1 {
    match value {
        IngressDirectionV1::Incoming => CommunicationDirectionV1::Incoming,
        IngressDirectionV1::Outgoing => CommunicationDirectionV1::Outgoing,
        IngressDirectionV1::Unknown => CommunicationDirectionV1::Unknown,
    }
}
const fn canonical_kind(
    value: CommunicationEvidenceKindV1,
) -> CanonicalCommunicationEvidenceKindV1 {
    match value {
        CommunicationEvidenceKindV1::EmailMessage => {
            CanonicalCommunicationEvidenceKindV1::EmailMessage
        }
        CommunicationEvidenceKindV1::ChatMessage => {
            CanonicalCommunicationEvidenceKindV1::ChatMessage
        }
        CommunicationEvidenceKindV1::MessageEdited => {
            CanonicalCommunicationEvidenceKindV1::MessageEdited
        }
        CommunicationEvidenceKindV1::MessageDeleted => {
            CanonicalCommunicationEvidenceKindV1::MessageDeleted
        }
        CommunicationEvidenceKindV1::ReactionChanged => {
            CanonicalCommunicationEvidenceKindV1::ReactionChanged
        }
        CommunicationEvidenceKindV1::DeliveryStateChanged => {
            CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged
        }
        CommunicationEvidenceKindV1::ConversationStateChanged => {
            CanonicalCommunicationEvidenceKindV1::ConversationStateChanged
        }
        CommunicationEvidenceKindV1::ParticipantChanged => {
            CanonicalCommunicationEvidenceKindV1::ParticipantChanged
        }
        CommunicationEvidenceKindV1::MediaChanged => {
            CanonicalCommunicationEvidenceKindV1::MediaChanged
        }
        CommunicationEvidenceKindV1::TopicChanged => {
            CanonicalCommunicationEvidenceKindV1::TopicChanged
        }
        CommunicationEvidenceKindV1::TypingChanged => {
            CanonicalCommunicationEvidenceKindV1::TypingChanged
        }
    }
}
const fn canonical_body(value: BodyAvailabilityV1) -> CommunicationBodyStateV1 {
    match value {
        BodyAvailabilityV1::MetadataOnly => CommunicationBodyStateV1::MetadataOnly,
        BodyAvailabilityV1::PendingBlob => CommunicationBodyStateV1::PendingBlob,
        BodyAvailabilityV1::Unavailable => CommunicationBodyStateV1::Unavailable,
        BodyAvailabilityV1::AdmittedBlob => CommunicationBodyStateV1::AdmittedBlob,
    }
}
fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsEventConsumeErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsEventConsumeErrorV1::WrongContract)
}
fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsEventConsumeErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsEventConsumeErrorV1::WrongContract)
}
fn optional_cursor(
    value: &[u8],
) -> Result<Option<CommunicationSourceCursorV1>, CommunicationsEventConsumeErrorV1> {
    if value.is_empty() {
        return Ok(None);
    }
    Ok(Some(CommunicationSourceCursorV1::new(id32(value)?)))
}

fn participant_display_label_from_wire(
    value: Option<String>,
) -> Result<Option<String>, CommunicationsEventConsumeErrorV1> {
    value
        .map(|value| {
            let normalized = value.trim();
            if normalized.is_empty()
                || normalized.len() > 256
                || normalized.chars().any(char::is_control)
            {
                return Err(CommunicationsEventConsumeErrorV1::InvalidPayload);
            }
            Ok(normalized.to_owned())
        })
        .transpose()
}

fn message_subject_from_wire(
    value: Option<String>,
) -> Result<Option<String>, CommunicationsEventConsumeErrorV1> {
    value
        .map(|value| {
            let normalized = value.trim();
            if normalized.is_empty()
                || normalized.len() > 998
                || normalized != value
                || normalized.chars().any(char::is_control)
            {
                return Err(CommunicationsEventConsumeErrorV1::InvalidPayload);
            }
            Ok(value)
        })
        .transpose()
}
fn persistence_error(error: CommunicationsPersistenceError) -> CommunicationsEventConsumeErrorV1 {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_communications_persistence_error={error:?}");
    }
    CommunicationsEventConsumeErrorV1::PersistenceRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_communications_ingress::{
        BodyAvailabilityV1, CommunicationDirectionV1, CommunicationEvidenceKindV1,
        ObservationEnvelopeContextV1, ProviderProvenanceV1, SourceEnvelope, SourceScopeEnvelope,
        build_observation_outbox_record_v1, new_scoped_communication_observation_draft,
    };
    #[test]
    fn applies_whatsapp_event_once_without_access_to_provider_locator() {
        let draft = new_scoped_communication_observation_draft(
            "provider-local-id",
            SourceEnvelope {
                provider: ProviderProvenanceV1::WhatsAppWeb,
                external_record_id: "private-chat-and-message".to_owned(),
                scope: Some(SourceScopeEnvelope {
                    external_account_id: "private-account".to_owned(),
                    external_conversation_id: Some("private-chat".to_owned()),
                    external_participant_id: Some("private-sender".to_owned()),
                    external_media_id: None,
                    external_reply_to_record_id: None,
                    external_forward_origin_record_id: None,
                }),
            },
            CommunicationEvidenceKindV1::ChatMessage,
            BodyAvailabilityV1::MetadataOnly,
            CommunicationDirectionV1::Unknown,
            Some(10),
        )
        .expect("draft");
        let record = build_observation_outbox_record_v1(
            &draft,
            &ObservationEnvelopeContextV1 {
                runtime_instance_id: "whatsapp_runtime_1".to_owned(),
                runtime_generation: 1,
                module_id: "whatsapp-runtime".to_owned(),
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .expect("record");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let command = command_from_envelope(&envelope).expect("command");
        assert_eq!(
            command.command.observation_id,
            CommunicationObservationIdV1::new(*record.message_id())
        );
    }

    #[test]
    fn accepts_zulip_provenance_from_the_public_ingress_contract() {
        let draft = new_scoped_communication_observation_draft(
            "provider-local-id",
            SourceEnvelope {
                provider: ProviderProvenanceV1::Zulip,
                external_record_id: "message-7".to_owned(),
                scope: Some(SourceScopeEnvelope {
                    external_account_id: "account-1".to_owned(),
                    external_conversation_id: Some("stream-1".to_owned()),
                    external_participant_id: None,
                    external_media_id: None,
                    external_reply_to_record_id: None,
                    external_forward_origin_record_id: None,
                }),
            },
            CommunicationEvidenceKindV1::ChatMessage,
            BodyAvailabilityV1::MetadataOnly,
            CommunicationDirectionV1::Incoming,
            Some(10),
        )
        .expect("draft");
        let record = build_observation_outbox_record_v1(
            &draft,
            &ObservationEnvelopeContextV1 {
                runtime_instance_id: "zulip-runtime-1".to_owned(),
                runtime_generation: 1,
                module_id: "zulip-runtime".to_owned(),
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .expect("record");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let command = command_from_envelope(&envelope).expect("command");
        assert_eq!(
            command.command.provider,
            CommunicationProviderProvenanceV1::Zulip
        );
    }

    #[test]
    fn accepts_exact_prior_contract_for_durable_backlog() {
        let draft = new_scoped_communication_observation_draft(
            "provider-local-id",
            SourceEnvelope {
                provider: ProviderProvenanceV1::MailImap,
                external_record_id: "message-8".to_owned(),
                scope: Some(SourceScopeEnvelope {
                    external_account_id: "account-1".to_owned(),
                    external_conversation_id: None,
                    external_participant_id: None,
                    external_media_id: None,
                    external_reply_to_record_id: None,
                    external_forward_origin_record_id: None,
                }),
            },
            CommunicationEvidenceKindV1::EmailMessage,
            BodyAvailabilityV1::MetadataOnly,
            CommunicationDirectionV1::Incoming,
            Some(10),
        )
        .expect("draft");
        let record = build_observation_outbox_record_v1(
            &draft,
            &ObservationEnvelopeContextV1 {
                runtime_instance_id: "mail-runtime-1".to_owned(),
                runtime_generation: 1,
                module_id: "mail-runtime".to_owned(),
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .expect("record");
        let mut envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let prior = communication_observed_prior_contract_reference_v1();
        let contract = envelope.contract.as_mut().expect("contract");
        contract.owner = prior.owner;
        contract.name = prior.name;
        contract.major = prior.major;
        contract.revision = prior.revision;
        contract.schema_sha256 = prior.schema_sha256;

        command_from_envelope(&envelope).expect("prior contract backlog");
    }

    #[test]
    fn prior_body_receipt_defaults_to_plain_text_without_widening_current_contract() {
        let legacy = hermes_communications_ingress::v1::BodyBlobReceiptV1 {
            blob_ref: "legacy-body".to_owned(),
            reference_id: vec![1; 16],
            declared_bytes: 1,
            sha256: vec![2; 32],
            custody_transfer_source_proof: vec![3],
            media_type: String::new(),
        };

        assert_eq!(
            source_body_from_wire(Some(legacy.clone()), true)
                .expect("prior receipt")
                .expect("body")
                .media_type,
            "text/plain",
        );
        assert!(matches!(
            source_body_from_wire(Some(legacy), false),
            Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
        ));
    }

    #[test]
    fn accepts_exact_transitional_revision_for_already_published_backlog() {
        let draft = new_scoped_communication_observation_draft(
            "provider-local-id",
            SourceEnvelope {
                provider: ProviderProvenanceV1::MailImap,
                external_record_id: "message-9".to_owned(),
                scope: Some(SourceScopeEnvelope {
                    external_account_id: "account-1".to_owned(),
                    external_conversation_id: None,
                    external_participant_id: None,
                    external_media_id: None,
                    external_reply_to_record_id: None,
                    external_forward_origin_record_id: None,
                }),
            },
            CommunicationEvidenceKindV1::EmailMessage,
            BodyAvailabilityV1::MetadataOnly,
            CommunicationDirectionV1::Incoming,
            Some(10),
        )
        .expect("draft");
        let record = build_observation_outbox_record_v1(
            &draft,
            &ObservationEnvelopeContextV1 {
                runtime_instance_id: "mail-runtime-1".to_owned(),
                runtime_generation: 1,
                module_id: "mail-runtime".to_owned(),
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .expect("record");
        let mut envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        envelope.contract.as_mut().expect("contract").revision = 2;

        command_from_envelope(&envelope).expect("transitional backlog");
    }
}
