//! Mail-owned validation and persistence of the Communications anchor handoff.

use makosh_communications_attachment_contract::{
    admission::communication_attachment_anchor_recorded_contract_reference_v1,
    anchor_recorded_v1::AttachmentAnchorRecordedV1,
};
use makosh_communications_ingress::admission::communication_observed_contract_reference_v1;
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, DurableEnvelopeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_persistence::{
    MailAttachmentAnchorMappingOutcomeV1, MailAttachmentAnchorMappingV1, MailDurablePersistence,
    MailDurablePersistenceError,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

use crate::admission::MAIL_MODULE_ID;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAttachmentAnchorMappingErrorV1 {
    Unavailable,
    InvalidEnvelope,
    InvalidPayload,
    SourceObservationMismatch,
    Persistence(MailDurablePersistenceError),
}

pub async fn map_attachment_anchor_recorded_v1(
    durable: &MailDurablePersistence,
    exact_handoff_bytes: &[u8],
    consumed_at_unix_seconds: i64,
) -> Result<MailAttachmentAnchorMappingOutcomeV1, MailAttachmentAnchorMappingErrorV1> {
    let handoff_record = OutboxRecordV1::accept(exact_handoff_bytes.to_vec())
        .map_err(|_| MailAttachmentAnchorMappingErrorV1::InvalidEnvelope)?;
    let envelope = decode_envelope_v1(handoff_record.exact_bytes())
        .map_err(|_| MailAttachmentAnchorMappingErrorV1::InvalidEnvelope)?;
    let payload = decode_handoff(&envelope)?;
    let source_observation_id = id16(&payload.source_observation_id)?;
    let correlation_id = id16(&envelope.correlation_id)?;
    if envelope.causation_message_id.as_slice() != source_observation_id.as_slice() {
        return Err(MailAttachmentAnchorMappingErrorV1::SourceObservationMismatch);
    }
    let source_record = durable
        .communications_outbox_record(source_observation_id)
        .await
        .map_err(MailAttachmentAnchorMappingErrorV1::Persistence)?
        .ok_or(MailAttachmentAnchorMappingErrorV1::SourceObservationMismatch)?;
    validate_mail_source_observation(&source_record, source_observation_id)?;
    durable
        .persist_attachment_anchor_mapping(
            &handoff_record,
            &MailAttachmentAnchorMappingV1 {
                source_observation_id,
                attachment_anchor_id: id16(&payload.attachment_anchor_id)?,
                correlation_id,
                media_cursor_sha256: sha256(&payload.media_cursor_sha256)?,
                observed_at_unix_seconds: payload.observed_at_unix_seconds,
            },
            consumed_at_unix_seconds,
        )
        .await
        .map_err(MailAttachmentAnchorMappingErrorV1::Persistence)
}

pub async fn consume_next_attachment_anchor_recorded_v1(
    durable: &MailDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    consumed_at_unix_seconds: i64,
) -> Result<MailAttachmentAnchorMappingOutcomeV1, MailAttachmentAnchorMappingErrorV1> {
    if !exact_permit_contract(
        permit.contract(),
        &communication_attachment_anchor_recorded_contract_reference_v1(),
    ) {
        return Err(MailAttachmentAnchorMappingErrorV1::InvalidEnvelope);
    }
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| MailAttachmentAnchorMappingErrorV1::Unavailable)?;
    let outcome = map_attachment_anchor_recorded_v1(
        durable,
        delivery.exact_bytes(),
        consumed_at_unix_seconds,
    )
    .await?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailAttachmentAnchorMappingErrorV1::Unavailable)?;
    Ok(outcome)
}

fn decode_handoff(
    envelope: &DurableEnvelopeV1,
) -> Result<AttachmentAnchorRecordedV1, MailAttachmentAnchorMappingErrorV1> {
    if !exact_contract(
        envelope.contract.as_ref(),
        &communication_attachment_anchor_recorded_contract_reference_v1(),
    ) || !matches!(envelope.semantics, Some(Semantics::Event(_)))
        || id16(&envelope.message_id).is_err()
        || id16(&envelope.correlation_id).is_err()
    {
        return Err(MailAttachmentAnchorMappingErrorV1::InvalidEnvelope);
    }
    let payload = AttachmentAnchorRecordedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailAttachmentAnchorMappingErrorV1::InvalidPayload)?;
    if payload.initial_state != 1
        || id16(&payload.attachment_anchor_id).is_err()
        || id16(&payload.source_observation_id).is_err()
        || sha256(&payload.media_cursor_sha256).is_err()
        || !(-62_135_596_800..=253_402_300_799).contains(&payload.observed_at_unix_seconds)
    {
        return Err(MailAttachmentAnchorMappingErrorV1::InvalidPayload);
    }
    Ok(payload)
}

fn validate_mail_source_observation(
    record: &OutboxRecordV1,
    source_observation_id: [u8; 16],
) -> Result<(), MailAttachmentAnchorMappingErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailAttachmentAnchorMappingErrorV1::SourceObservationMismatch)?;
    if envelope.message_id.as_slice() != source_observation_id.as_slice()
        || !exact_contract(
            envelope.contract.as_ref(),
            &communication_observed_contract_reference_v1(),
        )
        || !matches!(envelope.semantics, Some(Semantics::Observation(_)))
        || envelope
            .source
            .as_ref()
            .is_none_or(|source| source.module_id != MAIL_MODULE_ID)
    {
        return Err(MailAttachmentAnchorMappingErrorV1::SourceObservationMismatch);
    }
    Ok(())
}

fn exact_contract(value: Option<&ContractRefV1>, expected: &ContractReferenceV1) -> bool {
    value.is_some_and(|value| {
        value.owner == expected.owner
            && value.name == expected.name
            && value.major == expected.major
            && value.revision == expected.revision
            && value.schema_sha256 == expected.schema_sha256
    })
}

fn exact_permit_contract(
    value: Option<&ContractReferenceV1>,
    expected: &ContractReferenceV1,
) -> bool {
    value.is_some_and(|value| {
        value.owner == expected.owner
            && value.name == expected.name
            && value.major == expected.major
            && value.revision == expected.revision
            && value.schema_sha256 == expected.schema_sha256
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailAttachmentAnchorMappingErrorV1> {
    let id: [u8; 16] = value
        .try_into()
        .map_err(|_| MailAttachmentAnchorMappingErrorV1::InvalidPayload)?;
    (!id.iter().all(|byte| *byte == 0))
        .then_some(id)
        .ok_or(MailAttachmentAnchorMappingErrorV1::InvalidPayload)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], MailAttachmentAnchorMappingErrorV1> {
    let digest: [u8; 32] = value
        .try_into()
        .map_err(|_| MailAttachmentAnchorMappingErrorV1::InvalidPayload)?;
    (!digest.iter().all(|byte| *byte == 0))
        .then_some(digest)
        .ok_or(MailAttachmentAnchorMappingErrorV1::InvalidPayload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::EventMetadataV1;

    fn handoff(payload: AttachmentAnchorRecordedV1) -> DurableEnvelopeV1 {
        DurableEnvelopeV1 {
            message_id: vec![1; 16],
            causation_message_id: vec![2; 16],
            correlation_id: vec![5; 16],
            contract: Some(wire_contract(
                communication_attachment_anchor_recorded_contract_reference_v1(),
            )),
            semantics: Some(Semantics::Event(EventMetadataV1 { occurred_at: None })),
            payload: payload.encode_to_vec(),
            ..DurableEnvelopeV1::default()
        }
    }

    #[test]
    fn handoff_requires_descriptor_only_initial_state() {
        let envelope = handoff(AttachmentAnchorRecordedV1 {
            attachment_anchor_id: vec![3; 16],
            source_observation_id: vec![2; 16],
            media_cursor_sha256: vec![4; 32],
            initial_state: 2,
            observed_at_unix_seconds: 1_700_000_000,
        });

        assert_eq!(
            decode_handoff(&envelope),
            Err(MailAttachmentAnchorMappingErrorV1::InvalidPayload)
        );
    }

    #[test]
    fn handoff_rejects_an_observation_contract() {
        let mut envelope = handoff(AttachmentAnchorRecordedV1 {
            attachment_anchor_id: vec![3; 16],
            source_observation_id: vec![2; 16],
            media_cursor_sha256: vec![4; 32],
            initial_state: 1,
            observed_at_unix_seconds: 1_700_000_000,
        });
        envelope.contract = Some(wire_contract(communication_observed_contract_reference_v1()));

        assert_eq!(
            decode_handoff(&envelope),
            Err(MailAttachmentAnchorMappingErrorV1::InvalidEnvelope)
        );
    }

    #[test]
    fn handoff_requires_a_non_zero_correlation_id() {
        let mut envelope = handoff(AttachmentAnchorRecordedV1 {
            attachment_anchor_id: vec![3; 16],
            source_observation_id: vec![2; 16],
            media_cursor_sha256: vec![4; 32],
            initial_state: 1,
            observed_at_unix_seconds: 1_700_000_000,
        });
        envelope.correlation_id = vec![0; 16];

        assert_eq!(
            decode_handoff(&envelope),
            Err(MailAttachmentAnchorMappingErrorV1::InvalidEnvelope)
        );
    }

    fn wire_contract(reference: ContractReferenceV1) -> ContractRefV1 {
        ContractRefV1 {
            owner: reference.owner,
            name: reference.name,
            major: reference.major,
            revision: reference.revision,
            schema_sha256: reference.schema_sha256,
        }
    }
}
