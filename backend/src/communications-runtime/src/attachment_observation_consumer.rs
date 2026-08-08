//! Typed external attachment facts consumed without Blob, scanner, or integration calls.

use makosh_communications_api::{
    AttachmentSafetyStateV1, AttachmentSafetyTransitionCommandV1, AttachmentSafetyTransitionV1,
    CommunicationAttachmentAnchorIdV1, CommunicationObservationIdV1,
};
use makosh_communications_attachment_contract::{
    admission::{
        communication_attachment_blob_admission_observed_contract_reference_v1,
        communication_attachment_safety_verdict_observed_contract_reference_v1,
    },
    blob_admission_v1::AttachmentBlobAdmissionObservationV1,
    safety_verdict_v1::AttachmentSafetyVerdictObservationV1,
};
use makosh_communications_persistence::CommunicationsDurablePersistence;
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{DurableEnvelopeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

use crate::{
    attachment_safety::{
        AttachmentSafetyTransitionApplyErrorV1, apply_attachment_safety_transition,
    },
    canonical_outbox::CanonicalEventContextV1,
    consumer::{CommunicationsDeliveryErrorV1, CommunicationsEventConsumeErrorV1},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentObservationConsumeOutcomeV1 {
    Applied,
    Conflict,
}

pub async fn consume_next_attachment_blob_admission_observation_v1(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &CanonicalEventContextV1,
) -> Result<AttachmentObservationConsumeOutcomeV1, CommunicationsDeliveryErrorV1> {
    consume_next_attachment_observation_v1(
        persistence,
        connection,
        permit,
        context,
        attachment_blob_command_from_envelope,
        communication_attachment_blob_admission_observed_contract_reference_v1,
    )
    .await
}

pub async fn consume_next_attachment_safety_verdict_observation_v1(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &CanonicalEventContextV1,
) -> Result<AttachmentObservationConsumeOutcomeV1, CommunicationsDeliveryErrorV1> {
    consume_next_attachment_observation_v1(
        persistence,
        connection,
        permit,
        context,
        attachment_safety_command_from_envelope,
        communication_attachment_safety_verdict_observed_contract_reference_v1,
    )
    .await
}

async fn consume_next_attachment_observation_v1(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &CanonicalEventContextV1,
    decode: fn(
        &DurableEnvelopeV1,
    )
        -> Result<AttachmentSafetyTransitionCommandV1, CommunicationsEventConsumeErrorV1>,
    expected_contract: fn() -> ContractReferenceV1,
) -> Result<AttachmentObservationConsumeOutcomeV1, CommunicationsDeliveryErrorV1> {
    if !exact_contract(permit.contract(), &expected_contract()) {
        return Err(CommunicationsDeliveryErrorV1::Consume(
            CommunicationsEventConsumeErrorV1::WrongContract,
        ));
    }
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(delivery_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationsDeliveryErrorV1::InvalidEnvelope)?;
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationsDeliveryErrorV1::InvalidEnvelope)?;
    let command = decode(&envelope).map_err(CommunicationsDeliveryErrorV1::Consume)?;
    let causation_message_id = envelope.message_id.as_slice().try_into().map_err(|_| {
        CommunicationsDeliveryErrorV1::Consume(CommunicationsEventConsumeErrorV1::WrongContract)
    })?;
    let correlation_id =
        id16(&envelope.correlation_id).map_err(CommunicationsDeliveryErrorV1::Consume)?;
    let outcome = match apply_attachment_safety_transition(
        persistence,
        command,
        causation_message_id,
        correlation_id,
        context,
    )
    .await
    {
        Ok(_) => AttachmentObservationConsumeOutcomeV1::Applied,
        Err(AttachmentSafetyTransitionApplyErrorV1::Conflict) => {
            AttachmentObservationConsumeOutcomeV1::Conflict
        }
        Err(AttachmentSafetyTransitionApplyErrorV1::InvalidTransition) => {
            return Err(CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::DomainRejected,
            ));
        }
        Err(AttachmentSafetyTransitionApplyErrorV1::Unavailable) => {
            return Err(CommunicationsDeliveryErrorV1::Unavailable);
        }
    };
    delivery.acknowledge().await.map_err(delivery_error)?;
    Ok(outcome)
}

fn attachment_blob_command_from_envelope(
    envelope: &DurableEnvelopeV1,
) -> Result<AttachmentSafetyTransitionCommandV1, CommunicationsEventConsumeErrorV1> {
    validate_attachment_observation_envelope(
        envelope,
        &communication_attachment_blob_admission_observed_contract_reference_v1(),
    )?;
    let payload = AttachmentBlobAdmissionObservationV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationsEventConsumeErrorV1::InvalidPayload)?;
    let expected = blob_expected_state(payload.expected_state)?;
    let transition = match payload.transition {
        1 if expected == AttachmentSafetyStateV1::DescriptorOnly
            && payload.blob_reference_binding_sha256.is_empty() =>
        {
            AttachmentSafetyTransitionV1::BlobAdmissionRequested
        }
        2 if expected == AttachmentSafetyStateV1::BlobPending
            && valid_nonzero_sha256(&payload.blob_reference_binding_sha256) =>
        {
            AttachmentSafetyTransitionV1::BlobAdmitted
        }
        3 if payload.blob_reference_binding_sha256.is_empty() => {
            AttachmentSafetyTransitionV1::Rejected
        }
        _ => return Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
    };
    attachment_command(
        payload.attachment_anchor_id,
        expected,
        transition,
        payload.evidence_id,
        payload.observed_at_unix_seconds,
    )
}

fn attachment_safety_command_from_envelope(
    envelope: &DurableEnvelopeV1,
) -> Result<AttachmentSafetyTransitionCommandV1, CommunicationsEventConsumeErrorV1> {
    validate_attachment_observation_envelope(
        envelope,
        &communication_attachment_safety_verdict_observed_contract_reference_v1(),
    )?;
    let payload = AttachmentSafetyVerdictObservationV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationsEventConsumeErrorV1::InvalidPayload)?;
    let expected = safety_expected_state(payload.expected_state)?;
    let transition = match payload.verdict {
        1 if expected == AttachmentSafetyStateV1::BlobAdmitted => {
            AttachmentSafetyTransitionV1::DeclaredClean
        }
        2 => AttachmentSafetyTransitionV1::Quarantined,
        3 => AttachmentSafetyTransitionV1::Rejected,
        _ => return Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
    };
    attachment_command(
        payload.attachment_anchor_id,
        expected,
        transition,
        payload.evidence_id,
        payload.observed_at_unix_seconds,
    )
}

fn validate_attachment_observation_envelope(
    envelope: &DurableEnvelopeV1,
    expected: &ContractReferenceV1,
) -> Result<(), CommunicationsEventConsumeErrorV1> {
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else {
        return Err(CommunicationsEventConsumeErrorV1::WrongContract);
    };
    if !exact_envelope_contract(envelope.contract.as_ref(), expected)
        || metadata.observation_id != envelope.message_id
        || metadata.source_cursor_sha256.len() != 32
    {
        return Err(CommunicationsEventConsumeErrorV1::WrongContract);
    }
    Ok(())
}

fn attachment_command(
    anchor: Vec<u8>,
    expected: AttachmentSafetyStateV1,
    transition: AttachmentSafetyTransitionV1,
    evidence: Vec<u8>,
    observed_at_unix_seconds: i64,
) -> Result<AttachmentSafetyTransitionCommandV1, CommunicationsEventConsumeErrorV1> {
    let attachment_anchor_id = id16(&anchor)?;
    let evidence_id = id16(&evidence)?;
    if !(-62_135_596_800..=253_402_300_799).contains(&observed_at_unix_seconds) {
        return Err(CommunicationsEventConsumeErrorV1::InvalidPayload);
    }
    Ok(AttachmentSafetyTransitionCommandV1 {
        attachment_anchor_id: CommunicationAttachmentAnchorIdV1::new(attachment_anchor_id),
        current_state: expected,
        transition,
        evidence_id: CommunicationObservationIdV1::new(evidence_id),
        observed_at_unix_seconds,
    })
}

fn blob_expected_state(
    value: i32,
) -> Result<AttachmentSafetyStateV1, CommunicationsEventConsumeErrorV1> {
    match value {
        1 => Ok(AttachmentSafetyStateV1::DescriptorOnly),
        2 => Ok(AttachmentSafetyStateV1::BlobPending),
        3 => Ok(AttachmentSafetyStateV1::BlobAdmitted),
        _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload),
    }
}

fn safety_expected_state(
    value: i32,
) -> Result<AttachmentSafetyStateV1, CommunicationsEventConsumeErrorV1> {
    blob_expected_state(value)
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsEventConsumeErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| CommunicationsEventConsumeErrorV1::InvalidPayload)?;
    (!value.iter().all(|byte| *byte == 0))
        .then_some(value)
        .ok_or(CommunicationsEventConsumeErrorV1::InvalidPayload)
}

fn valid_nonzero_sha256(value: &[u8]) -> bool {
    value.len() == 32 && !value.iter().all(|byte| *byte == 0)
}

fn exact_contract(value: Option<&ContractReferenceV1>, expected: &ContractReferenceV1) -> bool {
    value.is_some_and(|value| {
        value.owner == expected.owner
            && value.name == expected.name
            && value.major == expected.major
            && value.revision == expected.revision
            && value.schema_sha256 == expected.schema_sha256
    })
}

fn exact_envelope_contract(
    value: Option<&makosh_events_protocol::v1::ContractRefV1>,
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

fn delivery_error(_: RuntimePullDeliveryErrorV1) -> CommunicationsDeliveryErrorV1 {
    CommunicationsDeliveryErrorV1::Unavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_admission_requires_the_expected_state_and_integrity_binding() {
        assert_eq!(
            blob_expected_state(2),
            Ok(AttachmentSafetyStateV1::BlobPending)
        );
        assert!(valid_nonzero_sha256(&[7; 32]));
        assert!(!valid_nonzero_sha256(&[0; 32]));
    }

    #[test]
    fn scanner_clean_verdict_is_only_valid_from_blob_admitted() {
        assert_eq!(
            safety_expected_state(3),
            Ok(AttachmentSafetyStateV1::BlobAdmitted)
        );
        assert!(safety_expected_state(4).is_err());
    }
}
